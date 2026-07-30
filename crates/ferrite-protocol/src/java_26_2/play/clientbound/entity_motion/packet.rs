use crate::java_26_2::play::clientbound::packet::Vector3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionMoveRotation {
    pub position: Vector3,
    pub motion: Vector3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityPositionSync {
    pub entity_id: i32,
    pub change: PositionMoveRotation,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelativePosition {
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelativePositionRotation {
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinecartStep {
    pub position: Vector3,
    pub motion: Vector3,
    pub yaw: i8,
    pub pitch: i8,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveMinecartAlongTrack {
    pub entity_id: i32,
    pub steps: Vec<MinecartStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelativeRotation {
    pub entity_id: i32,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotateHead {
    pub entity_id: i32,
    pub head_yaw: i8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetEntityMotion {
    pub entity_id: i32,
    pub motion: Vector3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeleportEntity {
    pub entity_id: i32,
    pub change: PositionMoveRotation,
    pub relative_flags: u32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectilePower {
    pub entity_id: i32,
    pub acceleration_power: f64,
}

#[must_use]
pub const fn decode_rotation(value: i8) -> f32 {
    value as f32 * (360.0 / 256.0)
}

#[must_use]
pub fn encode_rotation(value: f32) -> i8 {
    (f64::from(value) * 256.0 / 360.0).floor() as i32 as i8
}
