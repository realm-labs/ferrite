use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::{RegionCoord, RegionMapping, RegionMappingVersion};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::SnapshotRecord;
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::ChunkLayout;
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;
use thiserror::Error;

use crate::chunk::ticket::{ACCESSIBLE_LEVEL, ChunkTicket, TicketLevel, TicketSource};
use crate::composite::gateway::CompositeRegionRouter;
use crate::composite::runtime::CompositeRuntimeConfig;
use crate::composite::services::{
    CompositeProductionRegionRuntime, CompositeProductionRuntimeConfig,
};
use crate::config::ValidatedServerConfig;
use crate::entity_service::runtime::EntityServiceRuntimeLimits;
use crate::session::route::{InitialWorldRoute, VirtualHostRoutes};
use crate::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
use crate::simulation::runtime::SimulationRuntimeConfig;
use crate::world_service::formal_lifecycle::{FormalChunkLifecycle, FormalChunkLifecycleConfig};
use crate::world_service::formal_persistence::FormalWorldPersistence;
use crate::world_service::lifecycle::{WorldLifecycleBootstrap, WorldLifecycleRuntime};
use crate::world_service::metadata;
use crate::world_service::model::WorldServiceRuntimeConfig;
use crate::world_service::spawn::{overworld_layout, resolve_respawn};

type DynError = Box<dyn Error + Send + Sync>;

const PRELOADED_REGION_RADIUS: i32 = 2;
const MAXIMUM_SPAWN_PREPARATION_TICKS: usize = 16;
const RESPAWN_SEARCH_RADIUS: i32 = 10;

pub(super) struct WorldBootstrap {
    pub(super) routes: VirtualHostRoutes,
    pub(super) router: CompositeRegionRouter,
    pub(super) chunk_lifecycle: FormalChunkLifecycle,
    pub(super) persistence: FormalWorldPersistence,
    pub(super) lifecycle: WorldLifecycleRuntime,
    pub(super) metadata_record: SnapshotRecord,
    pub(super) committed_tick: GameTick,
    pub(super) view_distance: u16,
    pub(super) simulation_distance: u16,
    pub(super) world_spawn: BlockPos,
    pub(super) respawn: BlockPos,
}

pub(super) fn load(config: &ValidatedServerConfig) -> Result<WorldBootstrap, WorldError> {
    load_inner(config).map_err(|source| WorldError { source })
}

fn load_inner(config: &ValidatedServerConfig) -> Result<WorldBootstrap, DynError> {
    let maximum_mailbox = config.config().limits.max_region_mailbox;
    let maximum_sessions = config.config().limits.max_sessions;
    let content_manifest = formal_content_manifest();
    let durable = metadata::load_or_create(config, content_manifest)?;
    let world = durable.metadata().world();
    let dimension = durable.metadata().overworld().clone();
    let mapping = RegionMapping::V1;
    let runner_capacity = maximum_mailbox.max(64);
    let region_keys = formal_region_keys(world, &dimension, durable.metadata().spawn().chunk());
    let (mut persistence, recovery) = FormalWorldPersistence::open(
        &config.config().storage.root,
        &durable,
        region_keys.iter().cloned(),
        content_manifest,
        config.config().world.save.autosave_interval_ticks,
    )?;
    let control_generation = recovery
        .point(durable.control_point().snapshot().key())
        .expect("metadata control point is selected at the checkpoint")
        .snapshot()
        .generation()
        .checked_next()?;
    let mut lifecycle = WorldLifecycleRuntime::bootstrap(
        WorldLifecycleBootstrap {
            world,
            mapping: RegionMappingVersion::V1,
            overworld: dimension.clone(),
            generation: control_generation,
            seed: durable.metadata().seed(),
            content_manifest,
            event_capacity: maximum_mailbox.max(64),
        },
        durable.metadata().dimensions().iter().skip(1).cloned(),
    )?;
    lifecycle.apply_level_records(&crate::world_service::continuity::materialized_records(
        durable.control_point(),
    ))?;
    lifecycle.prepare_levels()?;
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig {
        command_capacity: runner_capacity,
        boundary_capacity: runner_capacity,
        immediate_effect_capacity: runner_capacity,
        transfer_capacity: runner_capacity,
        journal_capacity: runner_capacity.saturating_mul(4),
        phase_output_capacity: runner_capacity,
        maximum_future_command_ticks: 4,
    })?;
    let layout = overworld_layout();
    let composite_config = composite_config(
        runner_capacity.max(maximum_sessions),
        maximum_sessions,
        layout,
        content_manifest,
    )?;
    let mut chunk_lifecycle = FormalChunkLifecycle::new(
        world,
        dimension.clone(),
        mapping,
        config.config().world.seed,
        FormalChunkLifecycleConfig {
            maximum_tickets: maximum_sessions
                .saturating_mul(maximum_session_tickets(config.config().world.view_distance))
                .max(1),
            maximum_generation_in_flight: runner_capacity,
            maximum_generation_results_per_tick: runner_capacity.min(4),
            maximum_lifecycle_actions_per_tick: runner_capacity,
            maximum_events_per_region_per_tick: runner_capacity.saturating_mul(4),
        },
    )?;
    let mut runtimes = Vec::new();
    for key in region_keys {
        let point = recovery.point(&key);
        let generation = match point {
            Some(point) => point.snapshot().generation().checked_next()?,
            None => ActivationGeneration::INITIAL,
        };
        let voxels = bootstrap_region_voxels(key.clone(), mapping, layout)?;
        runner.insert_region(
            RegionSimulationState::new(voxels),
            generation,
            recovery.checkpoint_tick(),
        )?;
        let mut runtime = match point {
            Some(point) => CompositeProductionRegionRuntime::restore(
                point,
                generation,
                composite_config.clone(),
            )?,
            None => CompositeProductionRegionRuntime::new(
                key.clone(),
                generation,
                recovery.checkpoint_tick(),
                i64::try_from(recovery.checkpoint_tick().get()).unwrap_or(i64::MAX),
                [],
                composite_config.clone(),
            )?,
        };
        if key == *durable.control_point().snapshot().key() {
            runtime.replace_world_auxiliary_records(vec![
                durable.metadata_record()?,
                lifecycle.level_record(&key, generation)?,
            ])?;
        }
        runtimes.push(runtime);
    }
    let mut router = CompositeRegionRouter::new(runner, runtimes)?;
    let mut catch_up_tick = recovery.checkpoint_tick();
    while catch_up_tick < recovery.resume_tick() {
        catch_up_tick = catch_up_tick.checked_next()?;
        let report = router.run_tick(catch_up_tick)?;
        let generations = report
            .regions()
            .map(|(key, _)| {
                (
                    key.clone(),
                    router
                        .activation_generation(key)
                        .expect("report Region remains owned"),
                )
            })
            .collect();
        persistence.capture(&report, &generations)?;
    }
    let spawn_chunks = spawn_search_chunks(durable.metadata().spawn())?;
    catch_up_tick = prepare_spawn_chunks(
        &mut chunk_lifecycle,
        &mut router,
        &mut persistence,
        &spawn_chunks,
        catch_up_tick,
    )?;
    let border = lifecycle
        .level(&dimension)
        .ok_or("formal overworld has no control state")?
        .border
        .clone();
    let spawn_snapshots = router.projectable_world_snapshots(spawn_chunks.iter().copied())?;
    let respawn = resolve_respawn(durable.metadata().spawn(), &border, &spawn_snapshots)
        .ok_or("prepared spawn area contains no safe respawn placement")?;
    let routes = VirtualHostRoutes::new(
        InitialWorldRoute {
            world,
            dimension: dimension.clone(),
            spawn: respawn,
            mapping,
        },
        64,
    )?;
    Ok(WorldBootstrap {
        routes,
        router,
        chunk_lifecycle,
        persistence,
        lifecycle,
        metadata_record: durable.metadata_record()?,
        committed_tick: catch_up_tick,
        view_distance: config.config().world.view_distance,
        simulation_distance: config.config().world.simulation_distance,
        world_spawn: durable.metadata().spawn(),
        respawn,
    })
}

fn prepare_spawn_chunks(
    lifecycle: &mut FormalChunkLifecycle,
    router: &mut CompositeRegionRouter,
    persistence: &mut FormalWorldPersistence,
    spawn_chunks: &BTreeSet<ChunkPos>,
    mut tick: GameTick,
) -> Result<GameTick, DynError> {
    let source = TicketSource::Generation(ResourceId::new("ferrite", "spawn_search")?);
    let tickets = spawn_chunks
        .iter()
        .map(|position| ChunkTicket {
            source: source.clone(),
            position: *position,
            level: TicketLevel::new(ACCESSIBLE_LEVEL),
            expires_at: None,
        })
        .collect::<Vec<_>>();
    for _ in 0..MAXIMUM_SPAWN_PREPARATION_TICKS {
        if router
            .projectable_world_snapshots(spawn_chunks.iter().copied())?
            .len()
            == spawn_chunks.len()
        {
            return Ok(tick);
        }
        tick = tick.checked_next()?;
        lifecycle.drive(tick, tickets.clone(), router)?;
        let report = router.run_tick(tick)?;
        let generations = report
            .regions()
            .map(|(key, _)| {
                router
                    .activation_generation(key)
                    .map(|generation| (key.clone(), generation))
                    .ok_or_else(|| format!("spawn preparation lost Region {key:?}"))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        persistence.capture(&report, &generations)?;
    }
    Err(format!(
        "spawn search area did not become projectable within {MAXIMUM_SPAWN_PREPARATION_TICKS} ticks"
    )
    .into())
}

fn spawn_search_chunks(spawn: BlockPos) -> Result<BTreeSet<ChunkPos>, DynError> {
    let minimum_x = spawn
        .x
        .checked_sub(RESPAWN_SEARCH_RADIUS)
        .ok_or("spawn search minimum X overflow")?;
    let maximum_x = spawn
        .x
        .checked_add(RESPAWN_SEARCH_RADIUS)
        .ok_or("spawn search maximum X overflow")?;
    let minimum_z = spawn
        .z
        .checked_sub(RESPAWN_SEARCH_RADIUS)
        .ok_or("spawn search minimum Z overflow")?;
    let maximum_z = spawn
        .z
        .checked_add(RESPAWN_SEARCH_RADIUS)
        .ok_or("spawn search maximum Z overflow")?;
    Ok((minimum_x.div_euclid(16)..=maximum_x.div_euclid(16))
        .flat_map(|x| {
            (minimum_z.div_euclid(16)..=maximum_z.div_euclid(16)).map(move |z| ChunkPos::new(x, z))
        })
        .collect())
}

fn formal_region_keys(
    world: ferrite_foundation::identity::WorldId,
    dimension: &ferrite_foundation::identity::DimensionId,
    spawn: ChunkPos,
) -> Vec<ferrite_foundation::region::SimulationRegionKey> {
    let center = RegionMapping::V1.region_for_chunk(world, dimension.clone(), spawn);
    let mut keys = Vec::new();
    for x in -PRELOADED_REGION_RADIUS..=PRELOADED_REGION_RADIUS {
        for z in -PRELOADED_REGION_RADIUS..=PRELOADED_REGION_RADIUS {
            keys.push(ferrite_foundation::region::SimulationRegionKey::new(
                world,
                dimension.clone(),
                RegionCoord::new(
                    center.coordinate().x().saturating_add(x),
                    center.coordinate().z().saturating_add(z),
                ),
                RegionMappingVersion::V1,
            ));
        }
    }
    keys
}

fn composite_config(
    capacity: usize,
    maximum_sessions: usize,
    layout: ChunkLayout,
    content_manifest: [u8; 32],
) -> Result<CompositeProductionRuntimeConfig, DynError> {
    let continuity_capacity = capacity.saturating_mul(4).max(64);
    let service_capacity = maximum_sessions.max(1);
    Ok(CompositeProductionRuntimeConfig {
        coordinator: CompositeRuntimeConfig {
            command_capacity: capacity,
            event_capacity: 64,
            projection_capacity: capacity,
            continuity_record_capacity: continuity_capacity,
            maximum_future_ticks: 4,
            maximum_payload_bytes: 1024 * 1024,
        },
        simulation: SimulationRuntimeConfig {
            mapping: RegionMapping::V1,
            budget: SimulationQueueBudget::new([
                (SimulationQueueKind::ScheduledBlocks, capacity),
                (SimulationQueueKind::ScheduledFluids, capacity),
                (SimulationQueueKind::BoundaryTransactions, capacity),
                (SimulationQueueKind::ImmediateNeighbors, capacity),
                (SimulationQueueKind::Fluids, capacity),
                (SimulationQueueKind::Redstone, capacity),
                (SimulationQueueKind::Lighting, capacity),
                (SimulationQueueKind::ProjectionPositions, capacity),
            ])?,
            projection_capacity: capacity,
            receipt_capacity: capacity,
            gameplay_random_seed: 0x4645_5252_4954_4503,
        },
        entities: EntityServiceRuntimeLimits::new(
            service_capacity,
            service_capacity,
            capacity,
            capacity,
        ),
        world: WorldServiceRuntimeConfig {
            mapping: RegionMapping::V1,
            layout,
            region_side_chunks: 8,
            chunk_capacity: capacity,
            event_capacity: capacity.saturating_mul(4),
            content_manifest,
        },
        player_capacity: service_capacity,
        projection_capacity_per_player: capacity,
    })
}

fn formal_content_manifest() -> [u8; 32] {
    *blake3::hash(b"ferrite:formal-gateway-world-v1").as_bytes()
}

fn maximum_session_tickets(view_distance: u16) -> usize {
    let diameter = usize::from(view_distance)
        .saturating_mul(2)
        .saturating_add(1);
    diameter.saturating_mul(diameter).saturating_add(1)
}

fn bootstrap_region_voxels(
    key: ferrite_foundation::region::SimulationRegionKey,
    mapping: RegionMapping,
    layout: ChunkLayout,
) -> Result<RegionVoxelState, DynError> {
    let bounds = mapping.chunk_bounds(key.coordinate())?;
    let mut voxels = RegionVoxelState::new(key, mapping, layout)?;
    for chunk_x in bounds.minimum().x..bounds.maximum_exclusive().x {
        for chunk_z in bounds.minimum().z..bounds.maximum_exclusive().z {
            let chunk = voxels.ensure_chunk(ChunkPos::new(chunk_x, chunk_z))?;
            for section_y in layout.sections().minimum()..=3 {
                chunk.set_uniform_section(section_y, BlockStateId::new(1), BiomeId::new(0))?;
            }
        }
    }
    debug_assert_eq!(
        voxels.view().block_state(BlockPos::new(
            bounds.minimum().x * 16,
            63,
            bounds.minimum().z * 16,
        ))?,
        BlockStateId::new(1)
    );
    Ok(voxels)
}

#[derive(Debug, Error)]
#[error("Minecraft local-world bootstrap failed: {source}")]
pub(super) struct WorldError {
    #[source]
    source: DynError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::resource::ResourceId;
    use ferrite_protocol::semantic::VirtualHost;

    use crate::config::ServerConfig;
    use crate::world_config::SpawnPolicy;

    #[test]
    fn simulation_bootstrap_keeps_the_pre_collision_surface_contract() {
        let key = ferrite_foundation::region::SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(-1, 0),
            RegionMappingVersion::V1,
        );
        let voxels = bootstrap_region_voxels(key, RegionMapping::V1, overworld_layout()).unwrap();
        assert_eq!(
            voxels.view().block_state(BlockPos::new(-1, 63, 0)).unwrap(),
            BlockStateId::new(1)
        );
        assert_eq!(
            voxels.view().block_state(BlockPos::new(-1, 64, 0)).unwrap(),
            BlockStateId::new(0)
        );
    }

    #[test]
    fn formal_bootstrap_uses_configured_world_identity_and_spawn() {
        let temporary = tempfile::tempdir().unwrap();
        let mut config = ServerConfig::development_node(1, 1, 30_000, temporary.path()).unwrap();
        config.world.id = "0000000000000000000000000000002a".to_owned();
        config.world.spawn = SpawnPolicy::Fixed {
            x: -17,
            y: 70,
            z: 33,
        };
        config.world.view_distance = 12;
        config.world.simulation_distance = 7;
        let config = ServerConfig::from_toml(&config.to_toml().unwrap()).unwrap();
        let bootstrap = load_inner(&config).unwrap();
        let route = bootstrap.routes.resolve(&VirtualHost {
            host: "localhost".to_owned(),
            port: 25_565,
        });
        assert_eq!(route.world.get(), 42);
        assert_eq!(route.dimension.to_string(), "minecraft:overworld");
        assert_eq!(route.spawn, bootstrap.respawn);
        assert_eq!(route.region().coordinate(), RegionCoord::new(-1, 0));
        assert_eq!(bootstrap.view_distance, 12);
        assert_eq!(bootstrap.simulation_distance, 7);
        assert_eq!(bootstrap.world_spawn, BlockPos::new(-17, 70, 33));
        assert!(bootstrap.respawn.x.abs_diff(bootstrap.world_spawn.x) <= 10);
        assert!(bootstrap.respawn.z.abs_diff(bootstrap.world_spawn.z) <= 10);
        assert!(bootstrap.committed_tick > GameTick::ZERO);
        assert!(
            bootstrap
                .router
                .projectable_world_snapshots([bootstrap.world_spawn.chunk()])
                .unwrap()
                .contains_key(&bootstrap.world_spawn.chunk())
        );
    }
}
