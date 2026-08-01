//! Shared deterministic Region fixtures for Simulation conformance.

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_server_runtime::chunk::projection::JavaTerrainRegistryMap;
use ferrite_server_runtime::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
use ferrite_server_runtime::simulation::runtime::{
    SimulationRegionRuntime, SimulationRuntimeConfig,
};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;

pub const fn chunk_for_region(coordinate_x: i32) -> ChunkPos {
    ChunkPos::new(coordinate_x.wrapping_mul(8), 0)
}

pub const fn block_for_region(coordinate_x: i32, local_x: i32) -> BlockPos {
    BlockPos::new(
        coordinate_x
            .wrapping_mul(8)
            .wrapping_mul(16)
            .wrapping_add(local_x),
        64,
        0,
    )
}

pub fn region(coordinate_x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).expect("fixture world ID is nonzero"),
        DimensionId::new(
            ResourceId::minecraft("overworld").expect("fixture dimension ID is valid"),
        ),
        RegionCoord::new(coordinate_x, 0),
        RegionMappingVersion::V1,
    )
}

pub fn voxel_state(coordinate_x: i32) -> RegionVoxelState {
    let mut state = RegionVoxelState::new(
        region(coordinate_x),
        RegionMapping::V1,
        ChunkLayout::new(
            VerticalSectionRange::new(-4, 24).expect("fixture section range is valid"),
            BlockStateId::new(0),
            BiomeId::new(0),
        ),
    )
    .expect("fixture Region mapping is valid");
    state
        .ensure_chunk(chunk_for_region(coordinate_x))
        .expect("fixture chunk belongs to its Region");
    state
}

pub fn simulation_state(coordinate_x: i32) -> RegionSimulationState {
    RegionSimulationState::new(voxel_state(coordinate_x))
}

pub fn simulation_config(capacity: usize) -> SimulationRuntimeConfig {
    SimulationRuntimeConfig {
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
        ])
        .expect("fixture queue capacities are nonzero"),
        projection_capacity: capacity,
        receipt_capacity: capacity,
        gameplay_random_seed: 0x5eed_cafe,
    }
}

pub fn simulation_runtime(coordinate_x: i32) -> SimulationRegionRuntime {
    SimulationRegionRuntime::new(
        region(coordinate_x),
        ActivationGeneration::INITIAL,
        GameTick::new(7),
        100,
        [chunk_for_region(coordinate_x)],
        simulation_config(64),
    )
    .expect("fixture Simulation runtime is valid")
}

pub fn registry_map() -> JavaTerrainRegistryMap {
    let mut map = JavaTerrainRegistryMap::new(8, BlockStateId::new(0))
        .expect("fixture registry capacity is nonzero");
    for raw in 0..=6 {
        map.insert_block_state(BlockStateId::new(raw as u32), raw)
            .expect("fixture raw block state is valid");
    }
    map
}
