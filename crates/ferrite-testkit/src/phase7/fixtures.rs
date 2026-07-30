//! Shared deterministic Phase 7 conformance fixtures.

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_server_runtime::phase7::model::{EntityPayload, EntityPersistentState};
use ferrite_server_runtime::phase7::runtime::{Phase7RegionRuntime, Phase7RuntimeLimits};

pub fn region(coordinate_x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).expect("fixture world identity is nonzero"),
        DimensionId::new(
            ResourceId::minecraft("overworld").expect("fixture dimension identity is valid"),
        ),
        RegionCoord::new(coordinate_x, 0),
        RegionMappingVersion::V1,
    )
}

#[must_use]
pub const fn chunk(coordinate_x: i32, local_x: i32) -> ChunkPos {
    ChunkPos::new(coordinate_x.wrapping_mul(8).wrapping_add(local_x), 0)
}

pub fn entity(value: u128) -> StableEntityId {
    StableEntityId::new(value).expect("fixture entity identity is nonzero")
}

pub fn payload(bytes: impl Into<Vec<u8>>) -> EntityPayload {
    EntityPayload::new(bytes.into()).expect("fixture entity payload is bounded")
}

pub fn state(coordinate_x: i32, marker: u8) -> EntityPersistentState {
    EntityPersistentState::active(
        ResourceId::minecraft("zombie").expect("fixture entity kind is valid"),
        chunk(coordinate_x, 0),
        payload(vec![marker]),
    )
}

#[must_use]
pub const fn limits(capacity: usize) -> Phase7RuntimeLimits {
    Phase7RuntimeLimits::new(capacity, capacity, capacity, capacity)
}

pub fn runtime(coordinate_x: i32, capacity: usize) -> Phase7RegionRuntime {
    Phase7RegionRuntime::new(
        region(coordinate_x),
        ActivationGeneration::INITIAL,
        RegionMapping::V1,
        limits(capacity),
    )
    .expect("fixture Phase 7 runtime is valid")
}
