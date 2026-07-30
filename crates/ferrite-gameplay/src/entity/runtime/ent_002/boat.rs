//! Boat status, local-authority floating, controls, bubbles, and riders.

use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn scale(self, scale: f64) -> Self {
        Self::new(self.x * scale, self.y * scale, self.z * scale)
    }

    #[must_use]
    pub const fn horizontal_squared(self) -> f64 {
        self.x * self.x + self.z * self.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoatStatus {
    UnderFlowingWater,
    UnderWater,
    InWater,
    OnLand { friction: f64 },
    InAir,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatInput {
    pub previous_status: BoatStatus,
    pub status: BoatStatus,
    pub velocity: Vector3,
    pub delta_rotation: f32,
    pub water_level: f64,
    pub y: f64,
    pub height: f64,
    pub water_above: f64,
    pub player_controller: bool,
    pub air_to_water_collision_free: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatOutcome {
    pub status: BoatStatus,
    pub velocity: Vector3,
    pub delta_rotation: f32,
    pub snapped_y: Option<f64>,
}

#[must_use]
pub fn float_boat(input: FloatInput) -> FloatOutcome {
    if matches!(input.previous_status, BoatStatus::InAir)
        && !matches!(input.status, BoatStatus::InAir | BoatStatus::OnLand { .. })
        && input.air_to_water_collision_free
    {
        return FloatOutcome {
            status: BoatStatus::InWater,
            velocity: Vector3::new(input.velocity.x, 0.0, input.velocity.z),
            delta_rotation: input.delta_rotation,
            snapped_y: Some(input.water_above - input.height + 0.101),
        };
    }

    let mut vertical = -0.04;
    let mut buoyancy = 0.0;
    let friction = match input.status {
        BoatStatus::InWater => {
            buoyancy = (input.water_level - input.y) / input.height;
            0.9
        }
        BoatStatus::UnderFlowingWater => {
            vertical = -0.0007;
            0.9
        }
        BoatStatus::UnderWater => {
            buoyancy = 0.01;
            0.45
        }
        BoatStatus::InAir => 0.9,
        BoatStatus::OnLand { friction } => {
            if input.player_controller {
                friction * 0.5
            } else {
                friction
            }
        }
    };
    let mut velocity = Vector3::new(
        input.velocity.x * friction,
        input.velocity.y + vertical,
        input.velocity.z * friction,
    );
    if buoyancy > 0.0 {
        velocity.y = (velocity.y + buoyancy * (0.04 / 0.65)) * 0.75;
    }
    FloatOutcome {
        status: input.status,
        velocity,
        delta_rotation: input.delta_rotation * friction as f32,
        snapped_y: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatInput {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub backward: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoatControl {
    pub velocity: Vector3,
    pub delta_rotation: f32,
    pub left_paddle: bool,
    pub right_paddle: bool,
}

#[must_use]
pub fn control_boat(
    input: BoatInput,
    yaw_degrees: f32,
    velocity: Vector3,
    mut delta_rotation: f32,
) -> BoatControl {
    if input.left {
        delta_rotation -= 1.0;
    }
    if input.right {
        delta_rotation += 1.0;
    }
    let mut acceleration = 0.0;
    if input.left != input.right && !input.forward && !input.backward {
        acceleration += 0.005;
    }
    if input.forward {
        acceleration += 0.04;
    }
    if input.backward {
        acceleration -= 0.005;
    }
    let yaw = f64::from(yaw_degrees) * PI / 180.0;
    BoatControl {
        velocity: Vector3::new(
            velocity.x - yaw.sin() * acceleration,
            velocity.y,
            velocity.z + yaw.cos() * acceleration,
        ),
        delta_rotation,
        left_paddle: input.right && !input.left || input.forward,
        right_paddle: input.left && !input.right || input.forward,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnderwaterStep {
    pub underwater_ticks: u8,
    pub eject_passengers: bool,
}

#[must_use]
pub const fn underwater_step(status: BoatStatus, underwater_ticks: u8) -> UnderwaterStep {
    if matches!(
        status,
        BoatStatus::UnderFlowingWater | BoatStatus::UnderWater
    ) {
        let underwater_ticks = underwater_ticks.saturating_add(1);
        UnderwaterStep {
            underwater_ticks,
            eject_passengers: underwater_ticks >= 60,
        }
    } else {
        UnderwaterStep {
            underwater_ticks: 0,
            eject_passengers: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleDirection {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BubbleExpiry {
    pub velocity_y: f64,
    pub eject_passengers: bool,
}

#[must_use]
pub const fn bubble_expiry(
    direction: BubbleDirection,
    has_player_passenger: bool,
    velocity_y: f64,
) -> BubbleExpiry {
    match direction {
        BubbleDirection::Down => BubbleExpiry {
            velocity_y: velocity_y - 0.7,
            eject_passengers: true,
        },
        BubbleDirection::Up => BubbleExpiry {
            velocity_y: if has_player_passenger { 2.7 } else { 0.6 },
            eject_passengers: false,
        },
    }
}

#[must_use]
pub const fn may_mount_boat(eye_in_water: bool, passenger_count: u8, passenger_limit: u8) -> bool {
    !eye_in_water && passenger_count < passenger_limit && passenger_limit <= 2
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatContact {
    AutoMount,
    Push,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatContactInput {
    pub server_side: bool,
    pub has_player_controller: bool,
    pub passenger_count: u8,
    pub entity_is_living: bool,
    pub entity_is_passenger: bool,
    pub entity_width_fits: bool,
    pub entity_tag_allows_mount: bool,
    pub vertical_boxes_overlap: bool,
}

#[must_use]
pub const fn boat_contact(input: BoatContactInput) -> BoatContact {
    if !input.vertical_boxes_overlap {
        BoatContact::Ignore
    } else if input.server_side
        && !input.has_player_controller
        && input.passenger_count < 2
        && input.entity_is_living
        && !input.entity_is_passenger
        && input.entity_width_fits
        && input.entity_tag_allows_mount
    {
        BoatContact::AutoMount
    } else {
        BoatContact::Push
    }
}

#[must_use]
pub fn clamp_passenger_yaw(delta: f32) -> f32 {
    wrap_degrees(delta).clamp(-105.0, 105.0)
}

#[must_use]
pub const fn animal_seat_yaw(entity_id: i32) -> f32 {
    if entity_id & 1 == 0 { 90.0 } else { 270.0 }
}

fn wrap_degrees(value: f32) -> f32 {
    (value + 180.0).rem_euclid(360.0) - 180.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DismountPose {
    Standing,
    Crouching,
    Swimming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DismountChoice {
    pub pose: DismountPose,
    pub target_index: usize,
}

#[must_use]
pub fn first_boat_dismount(
    poses: &[DismountPose],
    target_safe_by_pose: &[Vec<bool>],
) -> Option<DismountChoice> {
    for (pose_index, pose) in poses.iter().copied().enumerate() {
        for (target_index, safe) in target_safe_by_pose
            .get(pose_index)
            .into_iter()
            .flatten()
            .copied()
            .enumerate()
        {
            if safe {
                return Some(DismountChoice { pose, target_index });
            }
        }
    }
    None
}
