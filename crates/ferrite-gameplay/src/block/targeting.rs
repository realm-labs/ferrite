use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::player::state::Vec3;

pub const DEFAULT_BLOCK_INTERACTION_RANGE: f64 = 4.5;
pub const PACKET_REACH_PADDING: f64 = 1.0;
pub const PLAYER_EYE_HEIGHT: f64 = 1.62;
const HIT_COMPONENT_LIMIT: f64 = 1.000_000_1;

#[must_use]
pub fn within_block_reach(eye: Vec3, target: BlockPos, range: f64, padding: f64) -> bool {
    let distance = squared_distance_to_unit_box(eye, target);
    let admitted_range = range + padding;
    distance < admitted_range * admitted_range
}

#[must_use]
pub fn valid_reconstructed_hit(
    target: BlockPos,
    offset_x: f32,
    offset_y: f32,
    offset_z: f32,
) -> bool {
    let center_x = f64::from(target.x) + 0.5;
    let center_y = f64::from(target.y) + 0.5;
    let center_z = f64::from(target.z) + 0.5;
    let hit_x = f64::from(target.x) + f64::from(offset_x);
    let hit_y = f64::from(target.y) + f64::from(offset_y);
    let hit_z = f64::from(target.z) + f64::from(offset_z);
    (hit_x - center_x).abs() < HIT_COMPONENT_LIMIT
        && (hit_y - center_y).abs() < HIT_COMPONENT_LIMIT
        && (hit_z - center_z).abs() < HIT_COMPONENT_LIMIT
}

pub fn adjacent(position: BlockPos, direction: Direction) -> Result<BlockPos, TargetingError> {
    position
        .checked_offset(direction, 1)
        .map_err(|_| TargetingError::CoordinateOverflow)
}

#[must_use]
pub fn eye_position(feet: Vec3) -> Vec3 {
    Vec3::new(feet.x, feet.y + PLAYER_EYE_HEIGHT, feet.z)
}

fn squared_distance_to_unit_box(point: Vec3, block: BlockPos) -> f64 {
    let dx = axis_distance(point.x, f64::from(block.x));
    let dy = axis_distance(point.y, f64::from(block.y));
    let dz = axis_distance(point.z, f64::from(block.z));
    dx * dx + dy * dy + dz * dz
}

fn axis_distance(value: f64, minimum: f64) -> f64 {
    if value < minimum {
        minimum - value
    } else if value > minimum + 1.0 {
        value - (minimum + 1.0)
    } else {
        0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TargetingError {
    #[error("adjacent block coordinate overflows")]
    CoordinateOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reach_is_strict_at_the_squared_boundary() {
        let target = BlockPos::new(0, 0, 0);
        assert!(within_block_reach(
            Vec3::new(6.499_999, 0.5, 0.5),
            target,
            4.5,
            1.0
        ));
        assert!(!within_block_reach(
            Vec3::new(6.5, 0.5, 0.5),
            target,
            4.5,
            1.0
        ));
    }

    #[test]
    fn hit_components_reject_boundary_and_non_finite_values() {
        let target = BlockPos::default();
        assert!(valid_reconstructed_hit(target, 0.5, 0.5, 0.5));
        assert!(!valid_reconstructed_hit(target, 1.500_000_1, 0.5, 0.5));
        assert!(!valid_reconstructed_hit(target, f32::NAN, 0.5, 0.5));
    }
}
