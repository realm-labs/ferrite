//! Generation-pyramid dependency and mutation admission.

use std::collections::BTreeMap;
use std::ops::Range;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use thiserror::Error;

use crate::generation::status::{ChunkStatus, chebyshev_distance};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationPyramid {
    center: ChunkPos,
    target: ChunkStatus,
    available: BTreeMap<ChunkPos, ChunkStatus>,
    upgrading: bool,
    generation_height: Range<i32>,
}

impl GenerationPyramid {
    pub fn new(
        center: ChunkPos,
        target: ChunkStatus,
        available: impl IntoIterator<Item = (ChunkPos, ChunkStatus)>,
        upgrading: bool,
        generation_height: Range<i32>,
    ) -> Result<Self, GenerationAccessError> {
        if generation_height.start >= generation_height.end {
            return Err(GenerationAccessError::EmptyGenerationHeight);
        }
        let pyramid = Self {
            center,
            target,
            available: available.into_iter().collect(),
            upgrading,
            generation_height,
        };
        pyramid.validate_dependencies()?;
        Ok(pyramid)
    }

    #[must_use]
    pub const fn center(&self) -> ChunkPos {
        self.center
    }

    #[must_use]
    pub const fn target(&self) -> ChunkStatus {
        self.target
    }

    pub fn resolve(&self, chunk: ChunkPos) -> Result<ChunkStatus, GenerationAccessError> {
        let radius = chebyshev_distance(self.center, chunk);
        let radius = u8::try_from(radius)
            .map_err(|_| GenerationAccessError::OutsidePyramid { chunk, radius })?;
        let required = self.target.direct_requirement(radius).ok_or(
            GenerationAccessError::OutsidePyramid {
                chunk,
                radius: u32::from(radius),
            },
        )?;
        let actual = self
            .available
            .get(&chunk)
            .copied()
            .ok_or(GenerationAccessError::MissingDependency { chunk, required })?;
        if actual < required {
            return Err(GenerationAccessError::DependencyTooOld {
                chunk,
                required,
                actual,
            });
        }
        Ok(actual)
    }

    #[must_use]
    pub fn ensure_can_write(&self, position: BlockPos) -> bool {
        let Some(radius) = self.target.write_radius() else {
            return false;
        };
        if chebyshev_distance(self.center, position.chunk()) > u32::from(radius) {
            return false;
        }
        !self.upgrading || self.generation_height.contains(&position.y)
    }

    fn validate_dependencies(&self) -> Result<(), GenerationAccessError> {
        for radius in 0..=8_u8 {
            let Some(required) = self.target.direct_requirement(radius) else {
                continue;
            };
            for chunk in square_ring(self.center, i32::from(radius)) {
                let actual = self
                    .available
                    .get(&chunk)
                    .copied()
                    .ok_or(GenerationAccessError::MissingDependency { chunk, required })?;
                if actual < required {
                    return Err(GenerationAccessError::DependencyTooOld {
                        chunk,
                        required,
                        actual,
                    });
                }
            }
        }
        Ok(())
    }
}

fn square_ring(center: ChunkPos, radius: i32) -> Vec<ChunkPos> {
    if radius == 0 {
        return vec![center];
    }
    let mut chunks = Vec::with_capacity((radius as usize) * 8);
    for x in -radius..=radius {
        chunks.push(ChunkPos::new(center.x + x, center.z - radius));
        chunks.push(ChunkPos::new(center.x + x, center.z + radius));
    }
    for z in (-radius + 1)..=(radius - 1) {
        chunks.push(ChunkPos::new(center.x - radius, center.z + z));
        chunks.push(ChunkPos::new(center.x + radius, center.z + z));
    }
    chunks
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GenerationAccessError {
    #[error("generation height must contain at least one block")]
    EmptyGenerationHeight,
    #[error("chunk {chunk:?} at radius {radius} is outside the direct dependency pyramid")]
    OutsidePyramid { chunk: ChunkPos, radius: u32 },
    #[error("generation dependency {chunk:?} at {required:?} is absent")]
    MissingDependency {
        chunk: ChunkPos,
        required: ChunkStatus,
    },
    #[error("generation dependency {chunk:?} is {actual:?}, below {required:?}")]
    DependencyTooOld {
        chunk: ChunkPos,
        required: ChunkStatus,
        actual: ChunkStatus,
    },
}
