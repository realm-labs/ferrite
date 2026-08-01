//! Region-owned chunk partitions and immutable voxel views.

use crate::chunk::{ChunkAccessError, ChunkColumn, ChunkLayout};
use crate::id::BlockStateId;
use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::region::{RegionCoord, RegionMapping, SimulationRegionKey};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug)]
pub struct RegionVoxelState {
    key: SimulationRegionKey,
    mapping: RegionMapping,
    layout: ChunkLayout,
    chunks: BTreeMap<ChunkPos, ChunkColumn>,
}

impl RegionVoxelState {
    pub fn new(
        key: SimulationRegionKey,
        mapping: RegionMapping,
        layout: ChunkLayout,
    ) -> Result<Self, RegionVoxelError> {
        if key.mapping_version() != mapping.version() {
            return Err(RegionVoxelError::MappingVersionMismatch);
        }
        Ok(Self {
            key,
            mapping,
            layout,
            chunks: BTreeMap::new(),
        })
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub const fn layout(&self) -> ChunkLayout {
        self.layout
    }

    pub fn view(&self) -> RegionVoxelView<'_> {
        RegionVoxelView { state: self }
    }

    pub fn ensure_chunk(
        &mut self,
        position: ChunkPos,
    ) -> Result<&mut ChunkColumn, RegionVoxelError> {
        self.validate_owner(position)?;
        Ok(self
            .chunks
            .entry(position)
            .or_insert_with(|| ChunkColumn::new(position, self.layout)))
    }

    pub fn insert_chunk(&mut self, chunk: ChunkColumn) -> Result<(), RegionVoxelError> {
        self.validate_owner(chunk.position())?;
        if chunk.layout() != self.layout {
            return Err(RegionVoxelError::LayoutMismatch);
        }
        if self.chunks.contains_key(&chunk.position()) {
            return Err(RegionVoxelError::DuplicateChunk(chunk.position()));
        }
        self.chunks.insert(chunk.position(), chunk);
        Ok(())
    }

    pub fn remove_chunk(&mut self, position: ChunkPos) -> Option<ChunkColumn> {
        self.chunks.remove(&position)
    }

    pub fn set_block(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
    ) -> Result<BlockStateId, RegionVoxelError> {
        self.validate_owner(position.chunk())?;
        let chunk = self
            .chunks
            .get_mut(&position.chunk())
            .ok_or(RegionVoxelError::ChunkNotLoaded(position.chunk()))?;
        Ok(chunk.set_block(position, state)?)
    }

    pub fn recompute_chunk_light(&mut self, position: ChunkPos) -> Result<(), RegionVoxelError> {
        self.validate_owner(position)?;
        let chunk = self
            .chunks
            .get_mut(&position)
            .ok_or(RegionVoxelError::ChunkNotLoaded(position))?;
        crate::light::recompute_chunk_light(chunk)?;
        Ok(())
    }

    fn validate_owner(&self, position: ChunkPos) -> Result<(), RegionVoxelError> {
        let actual =
            self.mapping
                .region_for_chunk(self.key.world(), self.key.dimension().clone(), position);
        if actual != self.key {
            return Err(RegionVoxelError::WrongOwner {
                position,
                expected: self.key.coordinate(),
                actual: actual.coordinate(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegionVoxelView<'a> {
    state: &'a RegionVoxelState,
}

impl<'a> RegionVoxelView<'a> {
    pub const fn key(self) -> &'a SimulationRegionKey {
        &self.state.key
    }

    pub fn chunk(self, position: ChunkPos) -> Option<&'a ChunkColumn> {
        self.state.chunks.get(&position)
    }

    pub fn block_state(self, position: BlockPos) -> Result<BlockStateId, RegionVoxelError> {
        self.state.validate_owner(position.chunk())?;
        let chunk = self
            .state
            .chunks
            .get(&position.chunk())
            .ok_or(RegionVoxelError::ChunkNotLoaded(position.chunk()))?;
        Ok(chunk.block_state(position)?)
    }

    pub fn chunks(self) -> impl ExactSizeIterator<Item = (&'a ChunkPos, &'a ChunkColumn)> + 'a {
        self.state.chunks.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegionVoxelError {
    #[error("Region key and voxel mapping versions differ")]
    MappingVersionMismatch,
    #[error("chunk {position:?} belongs to Region {actual:?}, not {expected:?}")]
    WrongOwner {
        position: ChunkPos,
        expected: RegionCoord,
        actual: RegionCoord,
    },
    #[error("chunk {0:?} is not loaded")]
    ChunkNotLoaded(ChunkPos),
    #[error("chunk {0:?} is already loaded")]
    DuplicateChunk(ChunkPos),
    #[error("chunk layout does not match its Region")]
    LayoutMismatch,
    #[error(transparent)]
    Chunk(#[from] ChunkAccessError),
    #[error(transparent)]
    Light(#[from] crate::light::ChunkLightError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::VerticalSectionRange;
    use crate::id::BiomeId;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};
    use ferrite_foundation::resource::ResourceId;

    fn state(coordinate: RegionCoord) -> RegionVoxelState {
        let key = SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            coordinate,
            RegionMappingVersion::V1,
        );
        RegionVoxelState::new(
            key,
            RegionMapping::V1,
            ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).unwrap(),
                BlockStateId::new(0),
                BiomeId::new(1),
            ),
        )
        .unwrap()
    }

    #[test]
    fn negative_chunks_have_one_region_owner() {
        let mut region = state(RegionCoord::new(-1, 0));
        region.ensure_chunk(ChunkPos::new(-1, 0)).unwrap();
        assert!(region.ensure_chunk(ChunkPos::new(-9, 0)).is_err());
        assert!(region.ensure_chunk(ChunkPos::new(0, 0)).is_err());
    }

    #[test]
    fn immutable_views_are_ordered_and_do_not_imply_unloaded_air() {
        let mut region = state(RegionCoord::new(0, 0));
        region.ensure_chunk(ChunkPos::new(2, 0)).unwrap();
        region.ensure_chunk(ChunkPos::new(0, 0)).unwrap();
        let view = region.view();
        assert_eq!(
            view.chunks()
                .map(|(position, _)| *position)
                .collect::<Vec<_>>(),
            [ChunkPos::new(0, 0), ChunkPos::new(2, 0)]
        );
        assert!(
            view.block_state(BlockPos::new(16, 0, 0)).is_err(),
            "an owned but unloaded chunk is not implicit air"
        );
    }

    #[test]
    fn cross_region_writes_fail_before_mutation() {
        let mut region = state(RegionCoord::new(0, 0));
        assert!(
            region
                .set_block(BlockPos::new(128, 0, 0), BlockStateId::new(9))
                .is_err()
        );
        assert_eq!(region.view().chunks().len(), 0);
        assert!(
            region
                .set_block(BlockPos::new(0, 0, 0), BlockStateId::new(9))
                .is_err()
        );
        assert_eq!(region.view().chunks().len(), 0);
    }
}
