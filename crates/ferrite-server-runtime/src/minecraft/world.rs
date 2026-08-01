use std::error::Error;

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMapping, RegionMappingVersion};
use ferrite_foundation::resource::ResourceId;
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
use crate::entity_service::runtime::EntityServiceRuntimeLimits;
use crate::session::route::{InitialWorldRoute, VirtualHostRoutes};
use crate::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
use crate::simulation::runtime::SimulationRuntimeConfig;
use crate::world_service::model::WorldServiceRuntimeConfig;

type DynError = Box<dyn Error + Send + Sync>;

const PRELOADED_REGION_RADIUS: i32 = 2;

pub(super) struct WorldBootstrap {
    pub(super) routes: VirtualHostRoutes,
    pub(super) router: CompositeRegionRouter,
    pub(super) terrain: MinimalTerrain,
}

pub(super) fn load(
    maximum_mailbox: usize,
    maximum_sessions: usize,
) -> Result<WorldBootstrap, WorldError> {
    load_inner(maximum_mailbox, maximum_sessions).map_err(|source| WorldError { source })
}

fn load_inner(maximum_mailbox: usize, maximum_sessions: usize) -> Result<WorldBootstrap, DynError> {
    let world = WorldId::new(1)?;
    let dimension = DimensionId::new(ResourceId::minecraft("overworld")?);
    let mapping = RegionMapping::V1;
    let route = InitialWorldRoute {
        world,
        dimension: dimension.clone(),
        spawn_chunk: ChunkPos::new(0, 0),
        mapping,
    };
    let routes = VirtualHostRoutes::new(route, 64)?;
    let runner_capacity = maximum_mailbox.max(64);
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
    )?;
    let mut runtimes = Vec::new();
    for x in -PRELOADED_REGION_RADIUS..=PRELOADED_REGION_RADIUS {
        for z in -PRELOADED_REGION_RADIUS..=PRELOADED_REGION_RADIUS {
            let key = ferrite_foundation::region::SimulationRegionKey::new(
                world,
                dimension.clone(),
                RegionCoord::new(x, z),
                RegionMappingVersion::V1,
            );
            let voxels = RegionVoxelState::new(key.clone(), mapping, layout)?;
            runner.insert_region(
                RegionSimulationState::new(voxels),
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
            )?;
            runtimes.push(CompositeProductionRegionRuntime::new(
                key,
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
                0,
                [],
                composite_config.clone(),
            )?);
        }
    }
    let terrain = MinimalTerrain::new(
        layout,
        BlockStateId::new(0),
        BlockStateId::new(1),
        BiomeId::new(0),
        63,
    )?;
    Ok(WorldBootstrap {
        routes,
        router: CompositeRegionRouter::new(runner, runtimes)?,
        terrain,
    })
}

fn composite_config(
    capacity: usize,
    maximum_sessions: usize,
    layout: ChunkLayout,
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
            event_capacity: capacity,
            content_manifest: *blake3::hash(b"ferrite:formal-gateway-world-v1").as_bytes(),
        },
        player_capacity: service_capacity,
        projection_capacity_per_player: capacity,
    })
}

fn chunk_layout() -> ChunkLayout {
    ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).expect("locked vertical range is valid"),
        BlockStateId::new(0),
        BiomeId::new(0),
    )
}

#[derive(Debug, Error)]
#[error("Minecraft local-world bootstrap failed: {source}")]
pub(super) struct WorldError {
    #[source]
    source: DynError,
}
