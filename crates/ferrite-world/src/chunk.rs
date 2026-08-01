//! Sparse vertical chunk columns with checked revisions.

use crate::id::{BiomeId, BlockStateId};
use crate::projection::{BlockEntitySnapshot, ChunkSnapshot, ClientHeightmap, LightSnapshot};
use crate::section::{ChunkSection, RevisionError};
use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::resource::ResourceId;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkLayout {
    sections: VerticalSectionRange,
    default_block: BlockStateId,
    default_biome: BiomeId,
}

impl ChunkLayout {
    pub const fn new(
        sections: VerticalSectionRange,
        default_block: BlockStateId,
        default_biome: BiomeId,
    ) -> Self {
        Self {
            sections,
            default_block,
            default_biome,
        }
    }

    pub const fn sections(self) -> VerticalSectionRange {
        self.sections
    }

    pub const fn default_block(self) -> BlockStateId {
        self.default_block
    }

    pub const fn default_biome(self) -> BiomeId {
        self.default_biome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalSectionRange {
    minimum: i32,
    count: u16,
}

impl VerticalSectionRange {
    pub const fn new(minimum: i32, count: u16) -> Result<Self, ChunkLayoutError> {
        if count == 0 {
            return Err(ChunkLayoutError::EmptyVerticalRange);
        }
        if minimum.checked_add(count as i32).is_none() {
            return Err(ChunkLayoutError::VerticalRangeOverflow);
        }
        Ok(Self { minimum, count })
    }

    pub const fn minimum(self) -> i32 {
        self.minimum
    }

    pub const fn count(self) -> u16 {
        self.count
    }

    pub const fn maximum_exclusive(self) -> i32 {
        self.minimum + self.count as i32
    }

    pub const fn contains(self, section_y: i32) -> bool {
        section_y >= self.minimum && section_y < self.maximum_exclusive()
    }

    const fn index(self, section_y: i32) -> Option<usize> {
        if !self.contains(section_y) {
            return None;
        }
        Some((section_y - self.minimum) as usize)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkColumn {
    position: ChunkPos,
    layout: ChunkLayout,
    sections: Box<[Option<ChunkSection>]>,
    block_entities: BTreeMap<BlockPos, ResourceId>,
    revision: ChunkRevision,
}

impl ChunkColumn {
    pub fn new(position: ChunkPos, layout: ChunkLayout) -> Self {
        let sections = vec![None; usize::from(layout.sections.count())].into_boxed_slice();
        Self {
            position,
            layout,
            sections,
            block_entities: BTreeMap::new(),
            revision: ChunkRevision::INITIAL,
        }
    }

    pub const fn position(&self) -> ChunkPos {
        self.position
    }

    pub const fn layout(&self) -> ChunkLayout {
        self.layout
    }

    pub const fn revision(&self) -> ChunkRevision {
        self.revision
    }

    pub fn section(&self, section_y: i32) -> Result<Option<&ChunkSection>, ChunkAccessError> {
        let index = self.section_index(section_y)?;
        Ok(self.sections[index].as_ref())
    }

    pub fn block_state(&self, position: BlockPos) -> Result<BlockStateId, ChunkAccessError> {
        self.validate_position(position)?;
        let section_y = position.section().y;
        Ok(match self.section(section_y)? {
            Some(section) => section.block(position.local()),
            None => self.layout.default_block,
        })
    }

    pub fn set_block(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
    ) -> Result<BlockStateId, ChunkAccessError> {
        self.validate_position(position)?;
        let section_y = position.section().y;
        let index = self.section_index(section_y)?;
        let previous = match &self.sections[index] {
            Some(section) => section.block(position.local()),
            None => self.layout.default_block,
        };
        if previous == state {
            return Ok(previous);
        }
        let revision = self.revision.checked_next()?;
        let section = self.sections[index].get_or_insert_with(|| {
            ChunkSection::new(self.layout.default_block, self.layout.default_biome)
        });
        section.set_block(position.local(), state)?;
        self.revision = revision;
        Ok(previous)
    }

    pub fn set_uniform_section(
        &mut self,
        section_y: i32,
        block: BlockStateId,
        biome: BiomeId,
    ) -> Result<(), ChunkAccessError> {
        let index = self.section_index(section_y)?;
        let replacement = ChunkSection::filled(block, biome);
        if self.sections[index].as_ref() == Some(&replacement) {
            return Ok(());
        }
        self.revision = self.revision.checked_next()?;
        self.sections[index] = Some(replacement);
        Ok(())
    }

    pub fn insert_block_entity(
        &mut self,
        position: BlockPos,
        kind: ResourceId,
    ) -> Result<Option<ResourceId>, ChunkAccessError> {
        self.validate_position(position)?;
        self.section_index(position.section().y)?;
        if self.block_entities.get(&position) == Some(&kind) {
            return Ok(Some(kind));
        }
        self.revision = self.revision.checked_next()?;
        Ok(self.block_entities.insert(position, kind))
    }

    pub fn remove_block_entity(
        &mut self,
        position: BlockPos,
    ) -> Result<Option<ResourceId>, ChunkAccessError> {
        self.validate_position(position)?;
        self.section_index(position.section().y)?;
        if !self.block_entities.contains_key(&position) {
            return Ok(None);
        }
        self.revision = self.revision.checked_next()?;
        Ok(self.block_entities.remove(&position))
    }

    pub fn snapshot(
        &self,
        light: LightSnapshot,
        heightmap_includes: impl FnMut(ClientHeightmap, BlockStateId) -> bool,
    ) -> Result<ChunkSnapshot, crate::projection::ChunkProjectionError> {
        let sections = self
            .sections
            .iter()
            .map(|section| {
                section.clone().unwrap_or_else(|| {
                    ChunkSection::new(self.layout.default_block, self.layout.default_biome)
                })
            })
            .collect();
        let block_entities = self
            .block_entities
            .iter()
            .map(|(position, kind)| BlockEntitySnapshot {
                position: *position,
                kind: kind.clone(),
            })
            .collect();
        ChunkSnapshot::new(
            self.position,
            self.layout,
            self.revision,
            sections,
            block_entities,
            light,
            heightmap_includes,
        )
    }

    pub(crate) fn section_slots(&self) -> &[Option<ChunkSection>] {
        &self.sections
    }

    pub(crate) fn block_entities(&self) -> &BTreeMap<BlockPos, ResourceId> {
        &self.block_entities
    }

    pub(crate) fn from_durable_parts(
        position: ChunkPos,
        layout: ChunkLayout,
        sections: Vec<Option<ChunkSection>>,
        block_entities: BTreeMap<BlockPos, ResourceId>,
        revision: u64,
    ) -> Result<Self, ChunkAccessError> {
        if sections.len() != usize::from(layout.sections.count()) {
            return Err(ChunkAccessError::DurableSectionCount);
        }
        let chunk = Self {
            position,
            layout,
            sections: sections.into_boxed_slice(),
            block_entities,
            revision: ChunkRevision(revision),
        };
        for block_position in chunk.block_entities.keys() {
            chunk.validate_position(*block_position)?;
            chunk.section_index(block_position.section().y)?;
        }
        Ok(chunk)
    }

    fn section_index(&self, section_y: i32) -> Result<usize, ChunkAccessError> {
        self.layout
            .sections
            .index(section_y)
            .ok_or(ChunkAccessError::SectionOutsideVerticalRange {
                section_y,
                minimum: self.layout.sections.minimum(),
                maximum_exclusive: self.layout.sections.maximum_exclusive(),
            })
    }

    fn validate_position(&self, position: BlockPos) -> Result<(), ChunkAccessError> {
        let actual = position.chunk();
        if actual != self.position {
            return Err(ChunkAccessError::WrongChunk {
                expected: self.position,
                actual,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkRevision(u64);

impl ChunkRevision {
    pub const INITIAL: Self = Self(0);

    pub const fn get(self) -> u64 {
        self.0
    }

    const fn checked_next(self) -> Result<Self, RevisionError> {
        match self.0.checked_add(1) {
            Some(value) => Ok(Self(value)),
            None => Err(RevisionError::Exhausted),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ChunkLayoutError {
    #[error("vertical section range cannot be empty")]
    EmptyVerticalRange,
    #[error("vertical section range overflows i32")]
    VerticalRangeOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ChunkAccessError {
    #[error("block belongs to chunk {actual:?}, expected {expected:?}")]
    WrongChunk {
        expected: ChunkPos,
        actual: ChunkPos,
    },
    #[error("section Y {section_y} is outside vertical range [{minimum}, {maximum_exclusive})")]
    SectionOutsideVerticalRange {
        section_y: i32,
        minimum: i32,
        maximum_exclusive: i32,
    },
    #[error(transparent)]
    Section(#[from] crate::section::SectionError),
    #[error(transparent)]
    Revision(#[from] RevisionError),
    #[error("durable chunk section count does not match its layout")]
    DurableSectionCount,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> ChunkLayout {
        ChunkLayout::new(
            VerticalSectionRange::new(-4, 24).unwrap(),
            BlockStateId::new(0),
            BiomeId::new(1),
        )
    }

    #[test]
    fn sections_are_sparse_and_default_reads_do_not_allocate() {
        let column = ChunkColumn::new(ChunkPos::new(-1, 2), layout());
        assert_eq!(
            column.block_state(BlockPos::new(-1, 0, 32)).unwrap(),
            BlockStateId::new(0)
        );
        assert!(column.section(0).unwrap().is_none());
        assert_eq!(column.revision(), ChunkRevision::INITIAL);
    }

    #[test]
    fn writes_validate_chunk_and_vertical_ownership() {
        let mut column = ChunkColumn::new(ChunkPos::new(-1, 2), layout());
        let position = BlockPos::new(-1, -64, 32);
        assert_eq!(
            column.set_block(position, BlockStateId::new(9)).unwrap(),
            BlockStateId::new(0)
        );
        assert_eq!(column.block_state(position).unwrap(), BlockStateId::new(9));
        assert_eq!(column.revision().get(), 1);
        assert!(
            column
                .set_block(BlockPos::new(0, 0, 32), BlockStateId::new(2))
                .is_err()
        );
        assert!(
            column
                .set_block(BlockPos::new(-1, -65, 32), BlockStateId::new(2))
                .is_err()
        );
    }
}
