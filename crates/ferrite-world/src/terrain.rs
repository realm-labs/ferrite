//! Deterministic minimal terrain used before the complete world-generation pipeline.

use ferrite_foundation::coordinate::ChunkPos;
use thiserror::Error;

use crate::chunk::{ChunkAccessError, ChunkColumn, ChunkLayout};
use crate::id::{BiomeId, BlockStateId};
use crate::projection::{ChunkProjectionError, ChunkSnapshot, LightSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimalTerrain {
    layout: ChunkLayout,
    air: BlockStateId,
    solid: BlockStateId,
    biome: BiomeId,
    surface_y: i32,
}

impl MinimalTerrain {
    pub fn new(
        layout: ChunkLayout,
        air: BlockStateId,
        solid: BlockStateId,
        biome: BiomeId,
        surface_y: i32,
    ) -> Result<Self, MinimalTerrainError> {
        let minimum_y = layout.sections().minimum() * 16;
        let maximum_y = layout.sections().maximum_exclusive() * 16;
        if surface_y < minimum_y || surface_y >= maximum_y {
            return Err(MinimalTerrainError::SurfaceOutsideBuildHeight {
                surface_y,
                minimum_y,
                maximum_y,
            });
        }
        if surface_y.rem_euclid(16) != 15 {
            return Err(MinimalTerrainError::SurfaceMustEndSection { surface_y });
        }
        Ok(Self {
            layout,
            air,
            solid,
            biome,
            surface_y,
        })
    }

    pub fn snapshot(&self, position: ChunkPos) -> Result<ChunkSnapshot, MinimalTerrainError> {
        let mut chunk = ChunkColumn::new(position, self.layout);
        let highest_solid_section = self.surface_y.div_euclid(16);
        for section_y in self.layout.sections().minimum()..=highest_solid_section {
            chunk.set_uniform_section(section_y, self.solid, self.biome)?;
        }
        let light = LightSnapshot::full_sky(self.layout.sections().count())?;
        Ok(chunk.snapshot(light, |_, state| state != self.air)?)
    }
}

#[derive(Debug, Error)]
pub enum MinimalTerrainError {
    #[error("surface Y {surface_y} is outside build height [{minimum_y}, {maximum_y})")]
    SurfaceOutsideBuildHeight {
        surface_y: i32,
        minimum_y: i32,
        maximum_y: i32,
    },
    #[error("minimal terrain surface Y {surface_y} must end a complete section")]
    SurfaceMustEndSection { surface_y: i32 },
    #[error(transparent)]
    Chunk(#[from] ChunkAccessError),
    #[error(transparent)]
    Projection(#[from] ChunkProjectionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::VerticalSectionRange;

    #[test]
    fn flat_snapshot_is_deterministic_for_negative_chunks() {
        let layout = ChunkLayout::new(
            VerticalSectionRange::new(-4, 24).unwrap(),
            BlockStateId::new(0),
            BiomeId::new(0),
        );
        let terrain = MinimalTerrain::new(
            layout,
            BlockStateId::new(0),
            BlockStateId::new(1),
            BiomeId::new(0),
            63,
        )
        .unwrap();
        let first = terrain.snapshot(ChunkPos::new(-1, 2)).unwrap();
        let second = terrain.snapshot(ChunkPos::new(-1, 2)).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.sections().len(), 24);
        assert_eq!(
            first.heightmaps().values().next().unwrap().as_ref(),
            &[64; 256]
        );
    }
}
