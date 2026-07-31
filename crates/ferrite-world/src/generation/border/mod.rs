//! Tick-counted world-border state and its independent geometry consumers.

pub mod collision;
pub mod command;
pub mod effects;
pub mod geometry;
pub mod state;

pub const DEFAULT_SIZE: f64 = 59_999_968.0;
pub const DEFAULT_ABSOLUTE_MAX: i32 = 29_999_984;
pub const DEFAULT_DAMAGE_PER_BLOCK: f64 = 0.2;
pub const DEFAULT_SAFE_ZONE: f64 = 5.0;
pub const DEFAULT_WARNING_BLOCKS: i32 = 5;
pub const DEFAULT_WARNING_TIME: i32 = 300;
pub const BORDER_EPSILON: f64 = 9.999_999_747_378_752e-6;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderPoint3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub(crate) fn java_min(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        f64::NAN
    } else {
        left.min(right)
    }
}

pub(crate) fn java_max(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        f64::NAN
    } else {
        left.max(right)
    }
}

pub(crate) fn java_clamp(value: f64, minimum: f64, maximum: f64) -> f64 {
    if value < minimum {
        minimum
    } else {
        java_min(value, maximum)
    }
}

pub(crate) fn java_floor_i32(value: f64) -> i32 {
    let truncated = value as i32;
    if value < f64::from(truncated) {
        truncated.wrapping_sub(1)
    } else {
        truncated
    }
}
