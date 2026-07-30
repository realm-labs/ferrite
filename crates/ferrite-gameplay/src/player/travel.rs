//! Ordinary ground and air travel, excluding fluids, fall flight, and ability flight.

use crate::player::collision::{
    CollisionScene, EntityMotion, MoveContext, MoveResult, MoverType, move_entity,
};
use crate::player::state::Vec3;

const DEG_TO_RADIANS: f32 = 0.017_453_292;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TravelInput {
    pub strafe: f32,
    pub vertical: f32,
    pub forward: f32,
}

impl TravelInput {
    pub const ZERO: Self = Self {
        strafe: 0.0,
        vertical: 0.0,
        forward: 0.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TravelAttributes {
    pub movement_speed: f64,
    pub jump_strength: f64,
    pub gravity: f64,
    pub friction_modifier: f64,
    pub air_drag_modifier: f64,
    pub step_height: f64,
}

impl Default for TravelAttributes {
    fn default() -> Self {
        Self {
            movement_speed: 0.100_000_001_490_116_12,
            jump_strength: 0.419_999_986_886_978_15,
            gravity: 0.08,
            friction_modifier: 1.0,
            air_drag_modifier: 1.0,
            step_height: 0.6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TravelContext {
    pub input: TravelInput,
    pub yaw: f32,
    pub jumping: bool,
    pub on_ground: bool,
    pub sprinting: bool,
    pub immobile: bool,
    pub climbable: bool,
    pub suppressing_ladder_slide: bool,
    pub scaffolding: bool,
    pub powder_snow_climb: bool,
    pub levitation_amplifier: Option<u8>,
    pub slow_falling: bool,
    pub jump_boost_amplifier: Option<u8>,
    pub block_friction: f32,
    pub block_jump_factor: f32,
    pub block_speed_factor: f32,
    pub client_missing_below_chunk: bool,
    pub above_minimum_build_height: bool,
    pub discard_friction: bool,
    pub omnidirectional_air: bool,
    pub airborne_acceleration: Option<f32>,
}

impl Default for TravelContext {
    fn default() -> Self {
        Self {
            input: TravelInput::ZERO,
            yaw: 0.0,
            jumping: false,
            on_ground: false,
            sprinting: false,
            immobile: false,
            climbable: false,
            suppressing_ladder_slide: false,
            scaffolding: false,
            powder_snow_climb: false,
            levitation_amplifier: None,
            slow_falling: false,
            jump_boost_amplifier: None,
            block_friction: 0.6,
            block_jump_factor: 1.0,
            block_speed_factor: 1.0,
            client_missing_below_chunk: false,
            above_minimum_build_height: true,
            discard_friction: false,
            omnidirectional_air: false,
            airborne_acceleration: None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TravelTimers {
    pub no_jump_delay: u8,
    pub needs_sync: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TravelResult {
    pub movement: MoveResult,
    pub jumped: bool,
    pub acceleration_scale: f32,
    pub friction: f32,
    pub effective_gravity: f64,
}

pub fn ordinary_travel_tick(
    motion: &mut EntityMotion,
    timers: &mut TravelTimers,
    attributes: TravelAttributes,
    mut context: TravelContext,
    scene: &CollisionScene,
) -> TravelResult {
    timers.no_jump_delay = timers.no_jump_delay.saturating_sub(1);
    if motion.velocity.horizontal_length_squared() < 9.0e-6 {
        motion.velocity.x = 0.0;
        motion.velocity.z = 0.0;
    }
    if motion.velocity.y.abs() < 0.003 {
        motion.velocity.y = 0.0;
    }
    if context.immobile {
        context.input.strafe = 0.0;
        context.input.forward = 0.0;
        context.jumping = false;
    }

    let jumped = if context.jumping && context.on_ground && timers.no_jump_delay == 0 {
        let jumped = jump_from_ground(motion, attributes, context);
        if jumped {
            timers.no_jump_delay = 10;
            timers.needs_sync = true;
        }
        jumped
    } else {
        if !context.jumping {
            timers.no_jump_delay = 0;
        }
        false
    };

    let friction = movement_friction(context, attributes);
    let acceleration_scale = acceleration_scale(context, attributes, friction);
    let acceleration = apply_relative_input(context.input, acceleration_scale, context.yaw);
    motion.velocity = motion.velocity.add(acceleration);
    if context.climbable {
        motion.velocity.x = motion
            .velocity
            .x
            .clamp(-0.150_000_005_960_464_48, 0.150_000_005_960_464_48);
        motion.velocity.z = motion
            .velocity
            .z
            .clamp(-0.150_000_005_960_464_48, 0.150_000_005_960_464_48);
        motion.velocity.y = motion.velocity.y.max(-0.150_000_005_960_464_48);
        if motion.velocity.y < 0.0 && context.suppressing_ladder_slide && !context.scaffolding {
            motion.velocity.y = 0.0;
        }
    }

    let requested = motion.velocity;
    let movement = move_entity(
        motion,
        requested,
        MoveContext {
            mover_type: MoverType::SelfMovement,
            on_ground: context.on_ground,
            max_up_step: attributes.step_height as f32,
            block_speed_factor: context.block_speed_factor,
            effective_gravity: attributes.gravity,
            ..MoveContext::default()
        },
        scene,
    );
    if (movement.horizontal_collision || context.jumping)
        && (context.climbable || context.powder_snow_climb)
    {
        motion.velocity.y = 0.2;
    }

    let effective_gravity =
        effective_gravity(attributes.gravity, motion.velocity.y, context.slow_falling);
    let vertical = if let Some(amplifier) = context.levitation_amplifier {
        motion.velocity.y
            + (0.05 * f64::from(amplifier.saturating_add(1)) - motion.velocity.y) * 0.2
    } else if context.client_missing_below_chunk {
        if context.above_minimum_build_height {
            -0.1
        } else {
            0.0
        }
    } else {
        motion.velocity.y - effective_gravity
    };

    if context.discard_friction {
        motion.velocity.y = vertical;
    } else {
        let air_drag =
            (1.0_f32 - (1.0_f32 - 0.91_f32) * attributes.air_drag_modifier as f32).clamp(0.0, 1.0);
        let vertical_drag = if context.omnidirectional_air {
            air_drag
        } else {
            (1.0_f32 - (1.0_f32 - 0.98_f32) * attributes.air_drag_modifier as f32).clamp(0.0, 1.0)
        };
        motion.velocity.x *= f64::from(friction * air_drag);
        motion.velocity.y = vertical * f64::from(vertical_drag);
        motion.velocity.z *= f64::from(friction * air_drag);
    }

    TravelResult {
        movement,
        jumped,
        acceleration_scale,
        friction,
        effective_gravity,
    }
}

fn jump_from_ground(
    motion: &mut EntityMotion,
    attributes: TravelAttributes,
    context: TravelContext,
) -> bool {
    let boost = context.jump_boost_amplifier.map_or(0.0_f32, |amplifier| {
        0.1_f32 * f32::from(amplifier.saturating_add(1))
    });
    let jump_power = attributes.jump_strength as f32 * context.block_jump_factor + boost;
    if jump_power <= 1.0e-5_f32 {
        return false;
    }
    motion.velocity.y = motion.velocity.y.max(f64::from(jump_power));
    if context.sprinting {
        let angle = context.yaw * DEG_TO_RADIANS;
        motion.velocity.x += f64::from(-angle.sin() * 0.2_f32);
        motion.velocity.z += f64::from(angle.cos() * 0.2_f32);
    }
    true
}

fn movement_friction(context: TravelContext, attributes: TravelAttributes) -> f32 {
    if context.on_ground {
        (1.0_f32 - (1.0_f32 - context.block_friction) * attributes.friction_modifier as f32)
            .clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn acceleration_scale(context: TravelContext, attributes: TravelAttributes, friction: f32) -> f32 {
    if context.on_ground {
        if friction > 0.6_f32 {
            attributes.movement_speed as f32 * 0.216_000_02_f32 / (friction * friction * friction)
        } else {
            attributes.movement_speed as f32
        }
    } else if let Some(acceleration) = context.airborne_acceleration {
        acceleration
    } else if context.sprinting {
        0.025_999_999_f32
    } else {
        0.02_f32
    }
}

#[must_use]
pub fn apply_relative_input(input: TravelInput, scale: f32, yaw: f32) -> Vec3 {
    let mut vector = Vec3::new(
        f64::from(input.strafe),
        f64::from(input.vertical),
        f64::from(input.forward),
    );
    let length_squared = vector.length_squared();
    if length_squared < 1.0e-7 {
        return Vec3::ZERO;
    }
    if length_squared > 1.0 {
        vector = vector.scale(1.0 / length_squared.sqrt());
    }
    vector = vector.scale(f64::from(scale));
    let angle = yaw * DEG_TO_RADIANS;
    let sine = f64::from(angle.sin());
    let cosine = f64::from(angle.cos());
    Vec3::new(
        vector.x * cosine - vector.z * sine,
        vector.y,
        vector.z * cosine + vector.x * sine,
    )
}

const fn effective_gravity(gravity: f64, vertical_velocity: f64, slow_falling: bool) -> f64 {
    if vertical_velocity <= 0.0 && slow_falling && gravity > 0.01 {
        0.01
    } else {
        gravity
    }
}

#[cfg(test)]
mod tests {
    use crate::player::collision::Aabb;

    use super::*;

    fn motion() -> EntityMotion {
        EntityMotion::new(
            Vec3::new(0.0, 65.0, 0.0),
            Aabb::new(Vec3::new(-0.3, 65.0, -0.3), Vec3::new(0.3, 66.8, 0.3)),
        )
    }

    #[test]
    fn grounded_default_cardinal_input_uses_locked_friction_order() {
        let mut motion = motion();
        let mut timers = TravelTimers::default();
        let result = ordinary_travel_tick(
            &mut motion,
            &mut timers,
            TravelAttributes::default(),
            TravelContext {
                input: TravelInput {
                    strafe: 0.0,
                    vertical: 0.0,
                    forward: 0.98,
                },
                on_ground: true,
                ..TravelContext::default()
            },
            &CollisionScene::default(),
        );
        assert_eq!(result.friction, 0.6);
        assert_eq!(result.acceleration_scale, 0.1);
        assert!(motion.position.z > 0.097 && motion.position.z < 0.099);
    }

    #[test]
    fn jump_power_boost_and_sprint_impulse_precede_travel() {
        let mut motion = motion();
        let mut timers = TravelTimers::default();
        let result = ordinary_travel_tick(
            &mut motion,
            &mut timers,
            TravelAttributes::default(),
            TravelContext {
                jumping: true,
                on_ground: true,
                sprinting: true,
                jump_boost_amplifier: Some(1),
                ..TravelContext::default()
            },
            &CollisionScene::default(),
        );
        assert!(result.jumped);
        assert_eq!(timers.no_jump_delay, 10);
        assert!(motion.position.y > 65.61);
        assert!(motion.position.z > 0.19);
    }
}
