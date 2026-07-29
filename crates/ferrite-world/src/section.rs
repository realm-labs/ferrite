//! Section-sized block and biome storage.

use crate::id::{BiomeId, BlockStateId};
use crate::palette::{PaletteError, PalettedContainer};
use ferrite_foundation::coordinate::LocalBlockPos;
use thiserror::Error;

pub const BLOCKS_PER_SECTION: usize = 16 * 16 * 16;
pub const BIOMES_PER_SECTION: usize = 4 * 4 * 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSection {
    blocks: PalettedContainer<BlockStateId, BLOCKS_PER_SECTION>,
    biomes: PalettedContainer<BiomeId, BIOMES_PER_SECTION>,
    revision: SectionRevision,
}

impl ChunkSection {
    pub fn new(default_block: BlockStateId, default_biome: BiomeId) -> Self {
        Self {
            blocks: PalettedContainer::new(default_block),
            biomes: PalettedContainer::new(default_biome),
            revision: SectionRevision::INITIAL,
        }
    }

    pub fn filled(block: BlockStateId, biome: BiomeId) -> Self {
        Self::new(block, biome)
    }

    pub const fn revision(&self) -> SectionRevision {
        self.revision
    }

    pub const fn blocks(&self) -> &PalettedContainer<BlockStateId, BLOCKS_PER_SECTION> {
        &self.blocks
    }

    pub const fn biomes(&self) -> &PalettedContainer<BiomeId, BIOMES_PER_SECTION> {
        &self.biomes
    }

    pub fn block(&self, position: LocalBlockPos) -> BlockStateId {
        self.blocks
            .get(usize::from(position.linear_index()))
            .expect("validated local positions fit a section")
    }

    pub fn set_block(
        &mut self,
        position: LocalBlockPos,
        state: BlockStateId,
    ) -> Result<BlockStateId, SectionError> {
        let index = usize::from(position.linear_index());
        let previous = self.blocks.get(index)?;
        if previous == state {
            return Ok(previous);
        }
        let revision = self.revision.checked_next()?;
        self.blocks.set(index, state)?;
        self.revision = revision;
        Ok(previous)
    }

    pub fn biome(&self, index: usize) -> Result<BiomeId, SectionError> {
        Ok(self.biomes.get(index)?)
    }

    pub fn set_biome(&mut self, index: usize, biome: BiomeId) -> Result<BiomeId, SectionError> {
        let previous = self.biomes.get(index)?;
        if previous == biome {
            return Ok(previous);
        }
        let revision = self.revision.checked_next()?;
        self.biomes.set(index, biome)?;
        self.revision = revision;
        Ok(previous)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionRevision(u64);

impl SectionRevision {
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
pub enum RevisionError {
    #[error("storage revision is exhausted")]
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SectionError {
    #[error(transparent)]
    Palette(#[from] PaletteError),
    #[error(transparent)]
    Revision(#[from] RevisionError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_op_writes_do_not_advance_revision() {
        let mut section = ChunkSection::new(BlockStateId::new(0), BiomeId::new(1));
        let position = LocalBlockPos::new(2, 3, 4).unwrap();
        assert_eq!(
            section
                .set_block(position, BlockStateId::new(0))
                .unwrap()
                .get(),
            0
        );
        assert_eq!(section.revision(), SectionRevision::INITIAL);
        assert_eq!(
            section
                .set_block(position, BlockStateId::new(9))
                .unwrap()
                .get(),
            0
        );
        assert_eq!(section.block(position), BlockStateId::new(9));
        assert_eq!(section.revision().get(), 1);
    }
}
