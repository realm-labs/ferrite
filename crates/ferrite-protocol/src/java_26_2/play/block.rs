//! Shared 26.2 block-position and direction wire values.

use ferrite_foundation::coordinate::{BlockPos, SectionPos};
use ferrite_foundation::direction::Direction;
use thiserror::Error;

#[must_use]
pub const fn unpack_block_position(value: i64) -> BlockPos {
    BlockPos::new(
        sign_extend(value >> 38, 26),
        sign_extend(value, 12),
        sign_extend(value >> 12, 26),
    )
}

#[must_use]
pub const fn pack_block_position(position: BlockPos) -> i64 {
    ((position.x as i64 & 0x3ff_ffff) << 38)
        | ((position.z as i64 & 0x3ff_ffff) << 12)
        | (position.y as i64 & 0xfff)
}

#[must_use]
pub const fn unpack_section_position(value: i64) -> SectionPos {
    SectionPos::new(
        sign_extend(value >> 42, 22),
        sign_extend(value, 20),
        sign_extend(value >> 20, 22),
    )
}

#[must_use]
pub const fn pack_section_position(position: SectionPos) -> i64 {
    ((position.x as i64 & 0x3f_ffff) << 42)
        | ((position.z as i64 & 0x3f_ffff) << 20)
        | (position.y as i64 & 0xf_ffff)
}

#[must_use]
pub fn direction_from_player_action(value: u8) -> Direction {
    match value % 6 {
        0 => Direction::Down,
        1 => Direction::Up,
        2 => Direction::North,
        3 => Direction::South,
        4 => Direction::West,
        _ => Direction::East,
    }
}

pub const fn direction_from_index(value: i32) -> Result<Direction, BlockWireValueError> {
    match value {
        0 => Ok(Direction::Down),
        1 => Ok(Direction::Up),
        2 => Ok(Direction::North),
        3 => Ok(Direction::South),
        4 => Ok(Direction::West),
        5 => Ok(Direction::East),
        _ => Err(BlockWireValueError::InvalidDirection(value)),
    }
}

#[must_use]
pub const fn direction_index(direction: Direction) -> i32 {
    match direction {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

const fn sign_extend(value: i64, bits: u32) -> i32 {
    let shift = 64 - bits;
    ((value << shift) >> shift) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BlockWireValueError {
    #[error("block direction ordinal {0} is outside 0..=5")]
    InvalidDirection(i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_positions_preserve_signed_field_boundaries() {
        for position in [
            BlockPos::new(-33_554_432, -2_048, -33_554_432),
            BlockPos::new(-1, -1, -1),
            BlockPos::new(33_554_431, 2_047, 33_554_431),
        ] {
            assert_eq!(
                unpack_block_position(pack_block_position(position)),
                position
            );
        }
    }

    #[test]
    fn player_action_directions_accept_every_byte_by_modulo() {
        assert_eq!(direction_from_player_action(255), Direction::South);
        assert!(direction_from_index(6).is_err());
    }
}
