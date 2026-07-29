//! Combined Region-owned voxel and entity state.

use crate::entity::{RegionEntityPartition, RegionEntityView};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_world::region::{RegionVoxelState, RegionVoxelView};

pub struct RegionSimulationState {
    key: SimulationRegionKey,
    voxels: RegionVoxelState,
    entities: RegionEntityPartition,
}

impl RegionSimulationState {
    pub fn new(voxels: RegionVoxelState) -> Self {
        let key = voxels.key().clone();
        let entities = RegionEntityPartition::new(key.clone());
        Self {
            key,
            voxels,
            entities,
        }
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub fn view(&self) -> RegionSimulationView<'_> {
        RegionSimulationView {
            key: &self.key,
            voxels: self.voxels.view(),
            entities: self.entities.view(),
        }
    }

    pub fn voxels_mut(&mut self) -> &mut RegionVoxelState {
        &mut self.voxels
    }

    pub fn entities_mut(&mut self) -> &mut RegionEntityPartition {
        &mut self.entities
    }

    pub fn into_parts(self) -> (RegionVoxelState, RegionEntityPartition) {
        (self.voxels, self.entities)
    }
}

#[derive(Clone, Copy)]
pub struct RegionSimulationView<'a> {
    key: &'a SimulationRegionKey,
    voxels: RegionVoxelView<'a>,
    entities: RegionEntityView<'a>,
}

impl<'a> RegionSimulationView<'a> {
    pub const fn key(self) -> &'a SimulationRegionKey {
        self.key
    }

    pub const fn voxels(self) -> RegionVoxelView<'a> {
        self.voxels
    }

    pub const fn entities(self) -> RegionEntityView<'a> {
        self.entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::{DimensionId, StableEntityId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMapping, RegionMappingVersion};
    use ferrite_foundation::resource::ResourceId;
    use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
    use ferrite_world::id::{BiomeId, BlockStateId};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, bevy_ecs::prelude::Component)]
    struct Marker;

    fn state() -> RegionSimulationState {
        let key = SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        );
        let voxels = RegionVoxelState::new(
            key,
            RegionMapping::V1,
            ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).unwrap(),
                BlockStateId::new(0),
                BiomeId::new(1),
            ),
        )
        .unwrap();
        RegionSimulationState::new(voxels)
    }

    #[test]
    fn one_query_view_observes_only_one_region_partition() {
        let mut state = state();
        let id = StableEntityId::new(1).unwrap();
        state.entities_mut().spawn(id).unwrap();
        state.entities_mut().insert_component(id, Marker).unwrap();
        state
            .voxels_mut()
            .ensure_chunk(ferrite_foundation::coordinate::ChunkPos::new(0, 0))
            .unwrap();
        let view = state.view();
        assert_eq!(view.voxels().chunks().len(), 1);
        assert!(view.entities().contains(id));
        assert_eq!(view.key(), view.voxels().key());
        assert_eq!(view.key(), view.entities().key());
    }
}
