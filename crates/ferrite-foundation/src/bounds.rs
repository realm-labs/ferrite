//! Validated inclusive and half-open spatial bounds.

use crate::coordinate::{BlockPos, ChunkPos};
use crate::numeric::{NumericError, inclusive_span, multiply_u64};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "BlockBoundsRepr", into = "BlockBoundsRepr")]
pub struct BlockBounds {
    minimum: BlockPos,
    maximum: BlockPos,
}

impl BlockBounds {
    pub const fn new(minimum: BlockPos, maximum: BlockPos) -> Result<Self, BoundsError> {
        if minimum.x > maximum.x {
            return Err(BoundsError::InvertedAxis { axis: "x" });
        }
        if minimum.y > maximum.y {
            return Err(BoundsError::InvertedAxis { axis: "y" });
        }
        if minimum.z > maximum.z {
            return Err(BoundsError::InvertedAxis { axis: "z" });
        }
        Ok(Self { minimum, maximum })
    }

    pub const fn minimum(self) -> BlockPos {
        self.minimum
    }

    pub const fn maximum(self) -> BlockPos {
        self.maximum
    }

    pub const fn contains(self, position: BlockPos) -> bool {
        position.x >= self.minimum.x
            && position.x <= self.maximum.x
            && position.y >= self.minimum.y
            && position.y <= self.maximum.y
            && position.z >= self.minimum.z
            && position.z <= self.maximum.z
    }

    pub fn checked_volume(self) -> Result<u64, NumericError> {
        let width = inclusive_span(self.minimum.x, self.maximum.x)?;
        let height = inclusive_span(self.minimum.y, self.maximum.y)?;
        let depth = inclusive_span(self.minimum.z, self.maximum.z)?;
        multiply_u64(multiply_u64(width, height)?, depth)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct BlockBoundsRepr {
    minimum: BlockPos,
    maximum: BlockPos,
}

impl TryFrom<BlockBoundsRepr> for BlockBounds {
    type Error = BoundsError;

    fn try_from(value: BlockBoundsRepr) -> Result<Self, Self::Error> {
        Self::new(value.minimum, value.maximum)
    }
}

impl From<BlockBounds> for BlockBoundsRepr {
    fn from(value: BlockBounds) -> Self {
        Self {
            minimum: value.minimum,
            maximum: value.maximum,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ChunkBoundsRepr", into = "ChunkBoundsRepr")]
pub struct ChunkBounds {
    minimum: ChunkPos,
    maximum_exclusive: ChunkPos,
}

impl ChunkBounds {
    pub const fn new(minimum: ChunkPos, maximum_exclusive: ChunkPos) -> Result<Self, BoundsError> {
        if minimum.x >= maximum_exclusive.x {
            return Err(BoundsError::EmptyOrInvertedAxis { axis: "x" });
        }
        if minimum.z >= maximum_exclusive.z {
            return Err(BoundsError::EmptyOrInvertedAxis { axis: "z" });
        }
        Ok(Self {
            minimum,
            maximum_exclusive,
        })
    }

    pub const fn minimum(self) -> ChunkPos {
        self.minimum
    }

    pub const fn maximum_exclusive(self) -> ChunkPos {
        self.maximum_exclusive
    }

    pub const fn contains(self, position: ChunkPos) -> bool {
        position.x >= self.minimum.x
            && position.x < self.maximum_exclusive.x
            && position.z >= self.minimum.z
            && position.z < self.maximum_exclusive.z
    }

    pub const fn width(self) -> u64 {
        (self.maximum_exclusive.x as i64 - self.minimum.x as i64) as u64
    }

    pub const fn depth(self) -> u64 {
        (self.maximum_exclusive.z as i64 - self.minimum.z as i64) as u64
    }

    pub const fn checked_area(self) -> Result<u64, NumericError> {
        multiply_u64(self.width(), self.depth())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct ChunkBoundsRepr {
    minimum: ChunkPos,
    maximum_exclusive: ChunkPos,
}

impl TryFrom<ChunkBoundsRepr> for ChunkBounds {
    type Error = BoundsError;

    fn try_from(value: ChunkBoundsRepr) -> Result<Self, Self::Error> {
        Self::new(value.minimum, value.maximum_exclusive)
    }
}

impl From<ChunkBounds> for ChunkBoundsRepr {
    fn from(value: ChunkBounds) -> Self {
        Self {
            minimum: value.minimum,
            maximum_exclusive: value.maximum_exclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "VerticalBoundsRepr", into = "VerticalBoundsRepr")]
pub struct VerticalBounds {
    minimum_y: i32,
    maximum_y_exclusive: i32,
}

impl VerticalBounds {
    pub const fn new(minimum_y: i32, maximum_y_exclusive: i32) -> Result<Self, BoundsError> {
        if minimum_y >= maximum_y_exclusive {
            return Err(BoundsError::EmptyOrInvertedAxis { axis: "y" });
        }
        Ok(Self {
            minimum_y,
            maximum_y_exclusive,
        })
    }

    pub const fn minimum_y(self) -> i32 {
        self.minimum_y
    }

    pub const fn maximum_y_exclusive(self) -> i32 {
        self.maximum_y_exclusive
    }

    pub const fn contains(self, y: i32) -> bool {
        y >= self.minimum_y && y < self.maximum_y_exclusive
    }

    pub const fn height(self) -> u32 {
        (self.maximum_y_exclusive as i64 - self.minimum_y as i64) as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct VerticalBoundsRepr {
    minimum_y: i32,
    maximum_y_exclusive: i32,
}

impl TryFrom<VerticalBoundsRepr> for VerticalBounds {
    type Error = BoundsError;

    fn try_from(value: VerticalBoundsRepr) -> Result<Self, Self::Error> {
        Self::new(value.minimum_y, value.maximum_y_exclusive)
    }
}

impl From<VerticalBounds> for VerticalBoundsRepr {
    fn from(value: VerticalBounds) -> Self {
        Self {
            minimum_y: value.minimum_y,
            maximum_y_exclusive: value.maximum_y_exclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BoundsError {
    #[error("inclusive bounds are inverted on the {axis} axis")]
    InvertedAxis { axis: &'static str },
    #[error("half-open bounds are empty or inverted on the {axis} axis")]
    EmptyOrInvertedAxis { axis: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclusive_block_bounds_include_both_edges() {
        let bounds = BlockBounds::new(BlockPos::new(-1, 2, 3), BlockPos::new(1, 3, 4)).unwrap();
        assert!(bounds.contains(BlockPos::new(-1, 2, 3)));
        assert!(bounds.contains(BlockPos::new(1, 3, 4)));
        assert!(!bounds.contains(BlockPos::new(2, 3, 4)));
        assert_eq!(bounds.checked_volume().unwrap(), 12);
    }

    #[test]
    fn half_open_chunk_and_vertical_bounds_exclude_maximum() {
        let chunks = ChunkBounds::new(ChunkPos::new(-8, 4), ChunkPos::new(0, 12)).unwrap();
        assert_eq!(chunks.checked_area().unwrap(), 64);
        assert!(chunks.contains(ChunkPos::new(-8, 4)));
        assert!(!chunks.contains(ChunkPos::new(0, 4)));

        let vertical = VerticalBounds::new(-64, 320).unwrap();
        assert_eq!(vertical.height(), 384);
        assert!(vertical.contains(-64));
        assert!(!vertical.contains(320));
    }

    #[test]
    fn deserialization_revalidates_bounds() {
        let invalid = r#"{"minimum":{"x":1,"z":0},"maximum_exclusive":{"x":1,"z":8}}"#;
        assert!(serde_json::from_str::<ChunkBounds>(invalid).is_err());
    }

    #[test]
    fn full_coordinate_area_is_checked() {
        let bounds = ChunkBounds::new(
            ChunkPos::new(i32::MIN, i32::MIN),
            ChunkPos::new(i32::MAX, i32::MAX),
        )
        .unwrap();
        assert_eq!(bounds.width(), u64::from(u32::MAX));
        assert!(bounds.checked_area().is_ok());
    }
}
