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

use crate::session::route::{InitialWorldRoute, VirtualHostRoutes};

type DynError = Box<dyn Error + Send + Sync>;

const PRELOADED_REGION_RADIUS: i32 = 2;

pub(super) struct WorldBootstrap {
    pub(super) routes: VirtualHostRoutes,
    pub(super) runner: LocalRegionRunner,
    pub(super) terrain: MinimalTerrain,
}

pub(super) fn load(maximum_mailbox: usize) -> Result<WorldBootstrap, WorldError> {
    load_inner(maximum_mailbox).map_err(|source| WorldError { source })
}

fn load_inner(maximum_mailbox: usize) -> Result<WorldBootstrap, DynError> {
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
    for x in -PRELOADED_REGION_RADIUS..=PRELOADED_REGION_RADIUS {
        for z in -PRELOADED_REGION_RADIUS..=PRELOADED_REGION_RADIUS {
            let key = ferrite_foundation::region::SimulationRegionKey::new(
                world,
                dimension.clone(),
                RegionCoord::new(x, z),
                RegionMappingVersion::V1,
            );
            let voxels = RegionVoxelState::new(key, mapping, layout)?;
            runner.insert_region(
                RegionSimulationState::new(voxels),
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
            )?;
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
        runner,
        terrain,
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
