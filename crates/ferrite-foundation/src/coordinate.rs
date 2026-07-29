//! Block, chunk, section, and local coordinates with explicit conversions.

use crate::direction::Direction;
use crate::numeric::{NumericError, add_i32, floor_div_i32, floor_rem_u16, multiply_i32};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU16;
use thiserror::Error;

pub const CHUNK_SIDE: NonZeroU16 = NonZeroU16::new(16).expect("16 is nonzero");
pub const SECTION_SIDE: NonZeroU16 = CHUNK_SIDE;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub const fn chunk(self) -> ChunkPos {
        ChunkPos::new(
            floor_div_i32(self.x, CHUNK_SIDE),
            floor_div_i32(self.z, CHUNK_SIDE),
        )
    }

    pub const fn section(self) -> SectionPos {
        SectionPos::new(
            floor_div_i32(self.x, SECTION_SIDE),
            floor_div_i32(self.y, SECTION_SIDE),
            floor_div_i32(self.z, SECTION_SIDE),
        )
    }

    pub const fn local(self) -> LocalBlockPos {
        LocalBlockPos {
            x: floor_rem_u16(self.x, CHUNK_SIDE) as u8,
            y: floor_rem_u16(self.y, SECTION_SIDE) as u8,
            z: floor_rem_u16(self.z, CHUNK_SIDE) as u8,
        }
    }

    pub const fn checked_offset(
        self,
        direction: Direction,
        distance: i32,
    ) -> Result<Self, NumericError> {
        let [step_x, step_y, step_z] = direction.step();
        let delta_x = match multiply_i32(step_x, distance) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        let delta_y = match multiply_i32(step_y, distance) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        let delta_z = match multiply_i32(step_z, distance) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        let x = match add_i32(self.x, delta_x) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        let y = match add_i32(self.y, delta_y) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        let z = match add_i32(self.z, delta_z) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        Ok(Self { x, y, z })
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub const fn checked_min_block_x(self) -> Result<i32, NumericError> {
        multiply_i32(self.x, CHUNK_SIDE.get() as i32)
    }

    pub const fn checked_min_block_z(self) -> Result<i32, NumericError> {
        multiply_i32(self.z, CHUNK_SIDE.get() as i32)
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SectionPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LocalBlockPos {
    x: u8,
    y: u8,
    z: u8,
}

impl LocalBlockPos {
    pub const fn new(x: u8, y: u8, z: u8) -> Result<Self, LocalCoordinateError> {
        if x >= CHUNK_SIDE.get() as u8
            || y >= SECTION_SIDE.get() as u8
            || z >= CHUNK_SIDE.get() as u8
        {
            return Err(LocalCoordinateError { x, y, z });
        }
        Ok(Self { x, y, z })
    }

    pub const fn x(self) -> u8 {
        self.x
    }

    pub const fn y(self) -> u8 {
        self.y
    }

    pub const fn z(self) -> u8 {
        self.z
    }

    pub const fn linear_index(self) -> u16 {
        ((self.y as u16 * SECTION_SIDE.get()) + self.z as u16) * CHUNK_SIDE.get() + self.x as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("local block coordinate ({x}, {y}, {z}) is outside a 16×16×16 section")]
pub struct LocalCoordinateError {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_block_coordinates_map_to_euclidean_chunks_and_locals() {
        let position = BlockPos::new(-1, -17, -16);
        assert_eq!(position.chunk(), ChunkPos::new(-1, -1));
        assert_eq!(position.section(), SectionPos::new(-1, -2, -1));
        assert_eq!(position.local(), LocalBlockPos::new(15, 15, 0).unwrap());
    }

    #[test]
    fn local_index_is_y_z_x_ordered() {
        assert_eq!(LocalBlockPos::new(0, 0, 0).unwrap().linear_index(), 0);
        assert_eq!(LocalBlockPos::new(1, 0, 0).unwrap().linear_index(), 1);
        assert_eq!(LocalBlockPos::new(0, 0, 1).unwrap().linear_index(), 16);
        assert_eq!(LocalBlockPos::new(0, 1, 0).unwrap().linear_index(), 256);
        assert_eq!(LocalBlockPos::new(15, 15, 15).unwrap().linear_index(), 4095);
    }

    #[test]
    fn offset_and_chunk_origins_are_checked() {
        assert_eq!(
            BlockPos::new(1, 2, 3)
                .checked_offset(Direction::North, 4)
                .unwrap(),
            BlockPos::new(1, 2, -1)
        );
        assert!(
            BlockPos::new(i32::MAX, 0, 0)
                .checked_offset(Direction::East, 1)
                .is_err()
        );
        assert!(ChunkPos::new(i32::MAX, 0).checked_min_block_x().is_err());
    }
}
