//! Feature-gated old/new minecart rail, slowdown, collision, and dismount mechanics.

use crate::entity::runtime::ent_002::boat::{DismountChoice, DismountPose, Vector3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinecartEngine {
    Old,
    Improved,
}

#[must_use]
pub const fn selected_engine(improvements_enabled: bool) -> MinecartEngine {
    if improvements_enabled {
        MinecartEngine::Improved
    } else {
        MinecartEngine::Old
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AscendingNorth,
    AscendingSouth,
    SouthEast,
    SouthWest,
    NorthWest,
    NorthEast,
}

impl RailShape {
    pub const ALL: [Self; 10] = [
        Self::NorthSouth,
        Self::EastWest,
        Self::AscendingEast,
        Self::AscendingWest,
        Self::AscendingNorth,
        Self::AscendingSouth,
        Self::SouthEast,
        Self::SouthWest,
        Self::NorthWest,
        Self::NorthEast,
    ];

    #[must_use]
    pub const fn is_ascending(self) -> bool {
        matches!(
            self,
            Self::AscendingEast | Self::AscendingWest | Self::AscendingNorth | Self::AscendingSouth
        )
    }
}

#[must_use]
pub fn off_rail_velocity(mut velocity: Vector3, maximum_speed: f64, on_ground: bool) -> Vector3 {
    velocity.x = velocity.x.clamp(-maximum_speed, maximum_speed);
    velocity.z = velocity.z.clamp(-maximum_speed, maximum_speed);
    if on_ground {
        velocity.scale(0.5)
    } else {
        velocity.scale(0.95)
    }
}

#[must_use]
pub const fn natural_slowdown(
    engine: MinecartEngine,
    ridden: bool,
    in_water: bool,
    velocity: Vector3,
) -> Vector3 {
    let factor = match (engine, ridden) {
        (MinecartEngine::Old, false) => 0.96,
        (MinecartEngine::Improved, false) => 0.975,
        (_, true) => 0.997,
    };
    let velocity = Vector3::new(velocity.x * factor, 0.0, velocity.z * factor);
    if in_water {
        velocity.scale(0.95)
    } else {
        velocity
    }
}

#[must_use]
pub fn slope_acceleration(
    engine: MinecartEngine,
    shape: RailShape,
    horizontal_speed: f64,
    in_water: bool,
    mut velocity: Vector3,
) -> Vector3 {
    let mut acceleration = match engine {
        MinecartEngine::Old => 0.0078125,
        MinecartEngine::Improved => (horizontal_speed * 0.02).max(0.0078125),
    };
    if in_water {
        acceleration *= 0.2;
    }
    match shape {
        RailShape::AscendingEast => velocity.x -= acceleration,
        RailShape::AscendingWest => velocity.x += acceleration,
        RailShape::AscendingNorth => velocity.z += acceleration,
        RailShape::AscendingSouth => velocity.z -= acceleration,
        _ => {}
    }
    velocity
}

#[must_use]
pub fn project_to_rail(velocity: Vector3, exit_x: f64, exit_z: f64) -> Vector3 {
    let exit_length = exit_x.hypot(exit_z);
    if exit_length == 0.0 {
        return Vector3::ZERO;
    }
    let speed = velocity.x.hypot(velocity.z).min(2.0);
    let mut direction_x = exit_x / exit_length;
    let mut direction_z = exit_z / exit_length;
    if velocity.x * direction_x + velocity.z * direction_z < 0.0 {
        direction_x = -direction_x;
        direction_z = -direction_z;
    }
    Vector3::new(direction_x * speed, velocity.y, direction_z * speed)
}

#[must_use]
pub fn old_rail_movement(velocity: Vector3, ridden: bool, in_water: bool) -> Vector3 {
    let scale = if ridden { 0.75 } else { 1.0 };
    let maximum = if in_water { 0.2 } else { 0.4 };
    let requested = velocity.scale(scale);
    let speed = requested.x.hypot(requested.z);
    if speed > maximum {
        Vector3::new(
            requested.x / speed * maximum,
            requested.y,
            requested.z / speed * maximum,
        )
    } else {
        requested
    }
}

#[must_use]
pub fn new_maximum_speed(max_minecart_speed: f64, in_water: bool) -> f64 {
    let speed = max_minecart_speed / 20.0;
    if in_water { speed * 0.5 } else { speed }
}

#[must_use]
pub fn new_substep_count(horizontal_distance: f64, maximum_speed: f64) -> u32 {
    if horizontal_distance <= 0.0 || maximum_speed <= 0.0 {
        0
    } else {
        (horizontal_distance / maximum_speed).ceil() as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerStartImpulse {
    pub velocity: Vector3,
    pub applied: bool,
}

#[must_use]
pub fn player_start_impulse(velocity: Vector3, player_input: Vector3) -> PlayerStartImpulse {
    if velocity.horizontal_squared() >= 0.01 || player_input.horizontal_squared() == 0.0 {
        return PlayerStartImpulse {
            velocity,
            applied: false,
        };
    }
    let input_length = player_input.x.hypot(player_input.z);
    PlayerStartImpulse {
        velocity: Vector3::new(
            velocity.x + player_input.x / input_length * 0.001,
            velocity.y,
            velocity.z + player_input.z / input_length * 0.001,
        ),
        applied: true,
    }
}

#[must_use]
pub fn unpowered_rail(velocity: Vector3) -> Vector3 {
    if velocity.x.hypot(velocity.z) < 0.03 {
        Vector3::ZERO
    } else {
        Vector3::new(velocity.x * 0.5, 0.0, velocity.z * 0.5)
    }
}

#[must_use]
pub fn powered_rail(
    engine: MinecartEngine,
    velocity: Vector3,
    conductor_push: Option<Vector3>,
) -> Vector3 {
    let speed = velocity.x.hypot(velocity.z);
    if speed > 0.01 {
        return Vector3::new(
            velocity.x + velocity.x / speed * 0.06,
            velocity.y,
            velocity.z + velocity.z / speed * 0.06,
        );
    }
    let Some(push) = conductor_push else {
        return velocity;
    };
    let push_length = push.x.hypot(push.z);
    if push_length == 0.0 {
        return velocity;
    }
    let strength = if matches!(engine, MinecartEngine::Old) {
        0.02
    } else {
        0.2
    };
    Vector3::new(
        push.x / push_length * strength,
        velocity.y,
        push.z / push_length * strength,
    )
}

#[must_use]
pub const fn stops_in_opposing_v(
    entry: RailShape,
    next: RailShape,
    horizontal_speed_squared: f64,
) -> bool {
    let opposite = matches!(
        (entry, next),
        (RailShape::AscendingEast, RailShape::AscendingWest)
            | (RailShape::AscendingWest, RailShape::AscendingEast)
            | (RailShape::AscendingNorth, RailShape::AscendingSouth)
            | (RailShape::AscendingSouth, RailShape::AscendingNorth)
    );
    opposite && horizontal_speed_squared < 0.005
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotationUpdate {
    pub yaw: f32,
    pub flipped: bool,
}

#[must_use]
pub fn old_rotation(previous_yaw: f32, proposed_yaw: f32, flipped: bool) -> RotationUpdate {
    let difference = wrap_degrees(proposed_yaw - previous_yaw);
    if !(-170.0..170.0).contains(&difference) {
        RotationUpdate {
            yaw: proposed_yaw + 180.0,
            flipped: !flipped,
        }
    } else {
        RotationUpdate {
            yaw: proposed_yaw,
            flipped,
        }
    }
}

fn wrap_degrees(value: f32) -> f32 {
    (value + 180.0).rem_euclid(360.0) - 180.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RideableCollision {
    AutoMount,
    Push,
}

#[must_use]
pub const fn rideable_collision(
    horizontal_speed_squared: f64,
    target_is_player: bool,
    target_is_golem: bool,
    target_is_minecart: bool,
    target_is_passenger: bool,
    cart_has_passenger: bool,
) -> RideableCollision {
    if horizontal_speed_squared >= 0.01
        && !target_is_player
        && !target_is_golem
        && !target_is_minecart
        && !target_is_passenger
        && !cart_has_passenger
    {
        RideableCollision::AutoMount
    } else {
        RideableCollision::Push
    }
}

#[must_use]
pub const fn cart_push_admitted(
    server_side: bool,
    physics_enabled: bool,
    separation_squared: f64,
    facing_dot: f64,
    improvements_enabled: bool,
) -> bool {
    server_side
        && physics_enabled
        && separation_squared >= 0.0001
        && (improvements_enabled || facing_dot >= 0.8)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartPush {
    pub first: Vector3,
    pub second: Vector3,
}

#[must_use]
pub const fn push_carts(
    first: Vector3,
    second: Vector3,
    impulse: Vector3,
    first_furnace: bool,
    second_furnace: bool,
) -> CartPush {
    if first_furnace && !second_furnace {
        CartPush {
            first: first.scale(0.95),
            second: Vector3::new(
                second.x * 0.2 + first.x - impulse.x,
                second.y,
                second.z * 0.2 + first.z - impulse.z,
            ),
        }
    } else if second_furnace && !first_furnace {
        CartPush {
            first: Vector3::new(
                first.x * 0.2 + second.x + impulse.x,
                first.y,
                first.z * 0.2 + second.z + impulse.z,
            ),
            second: second.scale(0.95),
        }
    } else {
        let mean_x = (first.x + second.x) * 0.5;
        let mean_z = (first.z + second.z) * 0.5;
        CartPush {
            first: Vector3::new(
                first.x * 0.2 + mean_x - impulse.x,
                first.y,
                first.z * 0.2 + mean_z - impulse.z,
            ),
            second: Vector3::new(
                second.x * 0.2 + mean_x + impulse.x,
                second.y,
                second.z * 0.2 + mean_z + impulse.z,
            ),
        }
    }
}

#[must_use]
pub const fn ordinary_entity_impulse(impulse: Vector3) -> Vector3 {
    impulse.scale(0.25)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinecartDismountChoice {
    pub pose: DismountPose,
    pub direction_index: usize,
    pub height_offset: i8,
}

#[must_use]
pub fn first_minecart_dismount(
    poses: &[DismountPose],
    safe: &[Vec<Vec<bool>>],
) -> Option<MinecartDismountChoice> {
    for (pose_index, pose) in poses.iter().copied().enumerate() {
        let heights: &[i8] = if matches!(pose, DismountPose::Swimming) {
            &[0, 1]
        } else {
            &[0, 1, -1]
        };
        for (direction_index, by_height) in safe.get(pose_index).into_iter().flatten().enumerate() {
            for (height_index, height) in heights.iter().copied().enumerate() {
                if by_height.get(height_index).copied().unwrap_or(false) {
                    return Some(MinecartDismountChoice {
                        pose,
                        direction_index,
                        height_offset: height,
                    });
                }
            }
        }
    }
    None
}

#[must_use]
pub const fn boat_compatible_choice(choice: MinecartDismountChoice) -> DismountChoice {
    DismountChoice {
        pose: choice.pose,
        target_index: choice.direction_index,
    }
}
