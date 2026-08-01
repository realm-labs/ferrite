use std::error::Error;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::{RegionCoord, RegionMapping, RegionMappingVersion};
use ferrite_persistence::snapshot::SnapshotRecord;
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;
use ferrite_world::terrain::MinimalTerrain;
use thiserror::Error;

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
use crate::world_service::lifecycle::WorldLifecycleRuntime;
use crate::world_service::metadata;
use crate::world_service::model::WorldServiceRuntimeConfig;

type DynError = Box<dyn Error + Send + Sync>;

const PRELOADED_REGION_RADIUS: i32 = 2;

pub(super) struct WorldBootstrap {
    pub(super) routes: VirtualHostRoutes,
    pub(super) router: CompositeRegionRouter,
    pub(super) terrain: MinimalTerrain,
    pub(super) chunk_lifecycle: FormalChunkLifecycle,
    pub(super) persistence: FormalWorldPersistence,
    pub(super) lifecycle: WorldLifecycleRuntime,
    pub(super) metadata_record: SnapshotRecord,
    pub(super) committed_tick: GameTick,
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
    let route = InitialWorldRoute {
        world,
        dimension: dimension.clone(),
        spawn: durable.metadata().spawn(),
        mapping,
    };
    let routes = VirtualHostRoutes::new(route, 64)?;
    let runner_capacity = maximum_mailbox.max(64);
    let region_keys = formal_region_keys(world, &dimension);
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
        world,
        RegionMappingVersion::V1,
        dimension.clone(),
        durable.metadata().dimensions().iter().skip(1).cloned(),
        control_generation,
        content_manifest,
        maximum_mailbox.max(64),
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
    let layout = chunk_layout();
    let composite_config = composite_config(
        runner_capacity.max(maximum_sessions),
        maximum_sessions,
        layout,
        content_manifest,
    )?;
    let chunk_lifecycle = FormalChunkLifecycle::new(
        world,
        dimension.clone(),
        mapping,
        config.config().world.seed,
        FormalChunkLifecycleConfig {
            maximum_tickets: maximum_sessions.saturating_mul(290).max(1),
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
        let voxels = minimal_region_voxels(key.clone(), mapping, layout)?;
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
    let terrain = MinimalTerrain::new(
        layout,
        BlockStateId::new(0),
        BlockStateId::new(1),
        BiomeId::new(0),
        63,
    )?;
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
    Ok(WorldBootstrap {
        routes,
        router,
        terrain,
        chunk_lifecycle,
        persistence,
        lifecycle,
        metadata_record: durable.metadata_record()?,
        committed_tick: recovery.resume_tick(),
    })
}

fn formal_region_keys(
    world: ferrite_foundation::identity::WorldId,
    dimension: &ferrite_foundation::identity::DimensionId,
) -> Vec<ferrite_foundation::region::SimulationRegionKey> {
    let mut keys = Vec::new();
    for x in -PRELOADED_REGION_RADIUS..=PRELOADED_REGION_RADIUS {
        for z in -PRELOADED_REGION_RADIUS..=PRELOADED_REGION_RADIUS {
            keys.push(ferrite_foundation::region::SimulationRegionKey::new(
                world,
                dimension.clone(),
                RegionCoord::new(x, z),
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

fn chunk_layout() -> ChunkLayout {
    ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).expect("locked vertical range is valid"),
        BlockStateId::new(0),
        BiomeId::new(0),
    )
}

fn minimal_region_voxels(
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
    fn minimal_region_authority_matches_projected_surface() {
        let key = ferrite_foundation::region::SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(-1, 0),
            RegionMappingVersion::V1,
        );
        let voxels = minimal_region_voxels(key, RegionMapping::V1, chunk_layout()).unwrap();
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
        let config = ServerConfig::from_toml(&config.to_toml().unwrap()).unwrap();
        let bootstrap = load_inner(&config).unwrap();
        let route = bootstrap.routes.resolve(&VirtualHost {
            host: "localhost".to_owned(),
            port: 25_565,
        });
        assert_eq!(route.world.get(), 42);
        assert_eq!(route.dimension.to_string(), "minecraft:overworld");
        assert_eq!(route.spawn, BlockPos::new(-17, 70, 33));
        assert_eq!(route.region().coordinate(), RegionCoord::new(-1, 0));
    }
}
