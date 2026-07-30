//! Fluid, swimming, fall-flying, glider, and ability-flight movement modes.

use crate::player::collision::{
    CollisionScene, EntityMotion, MoveContext, MoveResult, MoverType, move_entity,
};
use crate::player::state::Vec3;
use crate::player::travel::{
    TravelAttributes, TravelContext, TravelInput, TravelResult, TravelTimers, apply_relative_input,
    ordinary_travel_tick,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidKind {
    Water,
    Lava,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TravelMode {
    Water,
    Lava,
    FallFlying,
    Ordinary,
}

#[must_use]
pub const fn select_travel_mode(
    in_water: bool,
    in_lava: bool,
    affected_by_fluids: bool,
    can_stand_on_current_fluid: bool,
    fall_flying: bool,
) -> TravelMode {
    if affected_by_fluids && !can_stand_on_current_fluid && in_water {
        TravelMode::Water
    } else if affected_by_fluids && !can_stand_on_current_fluid && in_lava {
        TravelMode::Lava
    } else if fall_flying {
        TravelMode::FallFlying
    } else {
        TravelMode::Ordinary
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluidTravelContext {
    pub kind: FluidKind,
    pub input: TravelInput,
    pub yaw: f32,
    pub on_ground: bool,
    pub sprinting: bool,
    pub climbable: bool,
    pub fluid_height: f64,
    pub fluid_jump_threshold: f64,
    pub water_slowdown: f32,
    pub water_movement_efficiency: f64,
    pub dolphins_grace: bool,
    pub gravity: f64,
    pub movement_speed: f64,
    pub step_height: f64,
    pub block_speed_factor: f32,
    pub vehicle: bool,
    pub can_float_while_ridden: bool,
}

impl Default for FluidTravelContext {
    fn default() -> Self {
        Self {
            kind: FluidKind::Water,
            input: TravelInput::ZERO,
            yaw: 0.0,
            on_ground: false,
            sprinting: false,
            climbable: false,
            fluid_height: 1.0,
            fluid_jump_threshold: 0.4,
            water_slowdown: 0.8,
            water_movement_efficiency: 0.0,
            dolphins_grace: false,
            gravity: 0.08,
            movement_speed: 0.100_000_001_490_116_12,
            step_height: 0.6,
            block_speed_factor: 1.0,
            vehicle: false,
            can_float_while_ridden: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluidTravelResult {
    pub movement: MoveResult,
    pub horizontal_slowdown: f32,
    pub acceleration: f32,
    pub exited_fluid: bool,
}

pub fn fluid_travel_tick(
    motion: &mut EntityMotion,
    context: FluidTravelContext,
    scene: &CollisionScene,
) -> FluidTravelResult {
    let entry_falling = motion.velocity.y <= 0.0;
    let old_y = motion.position.y;
    match context.kind {
        FluidKind::Water => travel_in_water(motion, context, entry_falling, old_y, scene),
        FluidKind::Lava => travel_in_lava(motion, context, entry_falling, old_y, scene),
    }
}

pub fn apply_swimming_steering(
    velocity: &mut Vec3,
    swimming: bool,
    passenger: bool,
    jumping: bool,
    look: Vec3,
    fluid_above_nonempty: bool,
) {
    if !swimming || passenger {
        return;
    }
    if look.y <= 0.0 || jumping || fluid_above_nonempty {
        let multiplier = if look.y < -0.2 { 0.085 } else { 0.06 };
        velocity.y += (look.y - velocity.y) * multiplier;
    }
}

pub const fn liquid_jump(velocity: Vec3) -> Vec3 {
    Vec3::new(velocity.x, velocity.y + 0.04, velocity.z)
}

pub const fn descend_in_water(velocity: Vec3) -> Vec3 {
    Vec3::new(velocity.x, velocity.y - 0.04, velocity.z)
}

fn travel_in_water(
    motion: &mut EntityMotion,
    context: FluidTravelContext,
    entry_falling: bool,
    old_y: f64,
    scene: &CollisionScene,
) -> FluidTravelResult {
    let mut slowdown = if context.sprinting {
        0.9
    } else {
        context.water_slowdown
    };
    let mut acceleration = 0.02_f32;
    let efficiency = if context.on_ground {
        context.water_movement_efficiency
    } else {
        context.water_movement_efficiency * 0.5
    };
    if efficiency > 0.0 {
        let efficiency = efficiency as f32;
        slowdown += (0.546_000_06_f32 - slowdown) * efficiency;
        acceleration += (context.movement_speed as f32 - acceleration) * efficiency;
    }
    if context.dolphins_grace {
        slowdown = 0.96;
    }
    motion.velocity = motion.velocity.add(apply_relative_input(
        context.input,
        acceleration,
        context.yaw,
    ));
    let movement = move_fluid_entity(motion, context, scene);
    if movement.horizontal_collision && context.climbable {
        motion.velocity.y = 0.2;
    }
    motion.velocity.x *= f64::from(slowdown);
    motion.velocity.y *= 0.8;
    motion.velocity.z *= f64::from(slowdown);
    motion.velocity = fluid_falling_adjusted(
        motion.velocity,
        context.gravity,
        entry_falling,
        context.sprinting,
    );
    if context.vehicle
        && context.can_float_while_ridden
        && context.fluid_height > context.fluid_jump_threshold
    {
        motion.velocity.y += 0.04;
    }
    let exited_fluid = apply_exit_impulse(motion, old_y, movement.horizontal_collision, scene);
    FluidTravelResult {
        movement,
        horizontal_slowdown: slowdown,
        acceleration,
        exited_fluid,
    }
}

fn travel_in_lava(
    motion: &mut EntityMotion,
    context: FluidTravelContext,
    entry_falling: bool,
    old_y: f64,
    scene: &CollisionScene,
) -> FluidTravelResult {
    motion.velocity = motion
        .velocity
        .add(apply_relative_input(context.input, 0.02, context.yaw));
    let movement = move_fluid_entity(motion, context, scene);
    if context.fluid_height <= context.fluid_jump_threshold {
        motion.velocity.x *= 0.5;
        motion.velocity.y *= 0.8;
        motion.velocity.z *= 0.5;
        motion.velocity = fluid_falling_adjusted(
            motion.velocity,
            context.gravity,
            entry_falling,
            context.sprinting,
        );
    } else {
        motion.velocity = motion.velocity.scale(0.5);
    }
    if context.gravity != 0.0 {
        motion.velocity.y -= context.gravity / 4.0;
    }
    let exited_fluid = apply_exit_impulse(motion, old_y, movement.horizontal_collision, scene);
    FluidTravelResult {
        movement,
        horizontal_slowdown: 0.5,
        acceleration: 0.02,
        exited_fluid,
    }
}

fn move_fluid_entity(
    motion: &mut EntityMotion,
    context: FluidTravelContext,
    scene: &CollisionScene,
) -> MoveResult {
    move_entity(
        motion,
        motion.velocity,
        MoveContext {
            mover_type: MoverType::SelfMovement,
            on_ground: context.on_ground,
            max_up_step: context.step_height as f32,
            block_speed_factor: context.block_speed_factor,
            effective_gravity: context.gravity,
            ..MoveContext::default()
        },
        scene,
    )
}

fn fluid_falling_adjusted(
    mut velocity: Vec3,
    gravity: f64,
    entry_falling: bool,
    sprinting: bool,
) -> Vec3 {
    if gravity == 0.0 || sprinting {
        return velocity;
    }
    let adjusted = velocity.y - gravity / 16.0;
    velocity.y = if entry_falling && (velocity.y - 0.005).abs() >= 0.003 && adjusted.abs() < 0.003 {
        -0.003
    } else {
        adjusted
    };
    velocity
}

fn apply_exit_impulse(
    motion: &mut EntityMotion,
    old_y: f64,
    horizontal_collision: bool,
    scene: &CollisionScene,
) -> bool {
    if !horizontal_collision {
        return false;
    }
    let candidate = Vec3::new(
        motion.velocity.x,
        motion.velocity.y + 0.6 - motion.position.y + old_y,
        motion.velocity.z,
    );
    if scene.collision_free(motion.bounds.move_by(candidate)) {
        motion.velocity.y = 0.3;
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GliderSlot {
    pub slot: u8,
    pub damage: u32,
    pub maximum_damage: u32,
    pub has_glider: bool,
    pub equippable_slot_matches: bool,
}

impl GliderSlot {
    #[must_use]
    pub const fn can_glide(self) -> bool {
        self.has_glider
            && self.equippable_slot_matches
            && self.damage.saturating_add(1) < self.maximum_damage
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FallFlyingContext<'a> {
    pub look: Vec3,
    pub pitch_radians: f32,
    pub gravity: f64,
    pub on_ground: bool,
    pub passenger: bool,
    pub levitating: bool,
    pub ability_flying: bool,
    pub climbable: bool,
    pub flight_ticks: u32,
    pub glider_slots: &'a [GliderSlot],
    pub glider_choice: Option<usize>,
    pub ordinary_input: TravelInput,
    pub yaw: f32,
    pub sprinting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallFlyingEffect {
    DamageGlider(u8),
    GlideEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallFlyingResult {
    pub movement: MoveResult,
    pub remains_fall_flying: bool,
    pub wall_damage: f32,
    pub glide_event: bool,
    pub damaged_slot: Option<u8>,
    pub effects: Vec<FallFlyingEffect>,
}

pub fn fall_flying_tick(
    motion: &mut EntityMotion,
    timers: &mut TravelTimers,
    attributes: TravelAttributes,
    context: FallFlyingContext<'_>,
    scene: &CollisionScene,
) -> FallFlyingResult {
    if context.climbable {
        let ordinary = ordinary_travel_tick(
            motion,
            timers,
            attributes,
            TravelContext {
                input: context.ordinary_input,
                yaw: context.yaw,
                climbable: true,
                sprinting: context.sprinting,
                ..TravelContext::default()
            },
            scene,
        );
        return FallFlyingResult {
            movement: ordinary.movement,
            remains_fall_flying: false,
            wall_damage: 0.0,
            glide_event: false,
            damaged_slot: None,
            effects: Vec::new(),
        };
    }

    let horizontal_look = context.look.horizontal_length_squared().sqrt();
    let old_horizontal_speed = motion.velocity.horizontal_length_squared().sqrt();
    let cosine = f64::from(context.pitch_radians.cos());
    let cosine_squared = cosine * cosine;
    motion.velocity.y += context.gravity * (-1.0 + cosine_squared * 0.75);
    if motion.velocity.y < 0.0 && horizontal_look > 0.0 {
        let amount = motion.velocity.y * -0.1 * cosine_squared;
        motion.velocity.x += context.look.x * amount / horizontal_look;
        motion.velocity.y += amount;
        motion.velocity.z += context.look.z * amount / horizontal_look;
    }
    if context.pitch_radians < 0.0 && horizontal_look > 0.0 {
        let amount = old_horizontal_speed * f64::from(-context.pitch_radians.sin()) * 0.04;
        motion.velocity.x -= context.look.x * amount / horizontal_look;
        motion.velocity.y += amount * 3.2;
        motion.velocity.z -= context.look.z * amount / horizontal_look;
    }
    if horizontal_look > 0.0 {
        motion.velocity.x +=
            (context.look.x / horizontal_look * old_horizontal_speed - motion.velocity.x) * 0.1;
        motion.velocity.z +=
            (context.look.z / horizontal_look * old_horizontal_speed - motion.velocity.z) * 0.1;
    }
    motion.velocity.x *= 0.99;
    motion.velocity.y *= 0.98;
    motion.velocity.z *= 0.99;
    let movement = move_entity(
        motion,
        motion.velocity,
        MoveContext {
            mover_type: MoverType::SelfMovement,
            max_up_step: attributes.step_height as f32,
            effective_gravity: context.gravity,
            ..MoveContext::default()
        },
        scene,
    );
    let new_horizontal_speed = motion.velocity.horizontal_length_squared().sqrt();
    let wall_damage = if movement.horizontal_collision {
        ((old_horizontal_speed - new_horizontal_speed) * 10.0 - 3.0) as f32
    } else {
        0.0
    };
    let valid_slots: Vec<_> = context
        .glider_slots
        .iter()
        .copied()
        .filter(|slot| slot.can_glide())
        .collect();
    let remains_fall_flying = !context.on_ground
        && !context.passenger
        && !context.levitating
        && !context.ability_flying
        && !valid_slots.is_empty();
    let glide_event = remains_fall_flying && context.flight_ticks.is_multiple_of(10);
    let damaged_slot = if glide_event && context.flight_ticks.is_multiple_of(20) {
        context
            .glider_choice
            .and_then(|choice| valid_slots.get(choice))
            .map(|slot| slot.slot)
    } else {
        None
    };
    let mut effects = Vec::new();
    if let Some(slot) = damaged_slot {
        effects.push(FallFlyingEffect::DamageGlider(slot));
    }
    if glide_event {
        effects.push(FallFlyingEffect::GlideEvent);
    }
    FallFlyingResult {
        movement,
        remains_fall_flying,
        wall_damage: wall_damage.max(0.0),
        glide_event,
        damaged_slot,
        effects,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbilitySuperclassTravel {
    Ordinary(TravelContext),
    Fluid(FluidTravelContext),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbilityFlightContext {
    pub jump: bool,
    pub shift: bool,
    pub sprinting: bool,
    pub flying_speed: f32,
    pub superclass: AbilitySuperclassTravel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbilityFlightResult {
    pub ordinary: Option<TravelResult>,
    pub fluid: Option<FluidTravelResult>,
    pub restored_vertical_velocity: f64,
}

pub fn ability_flight_tick(
    motion: &mut EntityMotion,
    timers: &mut TravelTimers,
    attributes: TravelAttributes,
    context: AbilityFlightContext,
    scene: &CollisionScene,
) -> AbilityFlightResult {
    let direction = i8::from(context.jump) - i8::from(context.shift);
    motion.velocity.y += f64::from(direction) * f64::from(context.flying_speed * 3.0);
    let entry_vertical_velocity = motion.velocity.y;
    let (ordinary, fluid) = match context.superclass {
        AbilitySuperclassTravel::Ordinary(mut ordinary) => {
            ordinary.airborne_acceleration = Some(if context.sprinting {
                context.flying_speed * 2.0
            } else {
                context.flying_speed
            });
            (
                Some(ordinary_travel_tick(
                    motion, timers, attributes, ordinary, scene,
                )),
                None,
            )
        }
        AbilitySuperclassTravel::Fluid(fluid) => {
            (None, Some(fluid_travel_tick(motion, fluid, scene)))
        }
    };
    motion.velocity.y = entry_vertical_velocity * 0.6;
    AbilityFlightResult {
        ordinary,
        fluid,
        restored_vertical_velocity: motion.velocity.y,
    }
}

#[cfg(test)]
mod tests {
    use crate::player::collision::Aabb;

    use super::*;

    fn motion() -> EntityMotion {
        let position = Vec3::new(0.0, 65.0, 0.0);
        EntityMotion::new(
            position,
            Aabb::new(Vec3::new(-0.3, 65.0, -0.3), Vec3::new(0.3, 66.8, 0.3)),
        )
    }

    #[test]
    fn water_efficiency_interpolates_and_airborne_halves_it() {
        let mut motion = motion();
        let result = fluid_travel_tick(
            &mut motion,
            FluidTravelContext {
                input: TravelInput {
                    strafe: 0.0,
                    vertical: 0.0,
                    forward: 1.0,
                },
                water_movement_efficiency: 1.0,
                ..FluidTravelContext::default()
            },
            &CollisionScene::default(),
        );
        assert_eq!(result.horizontal_slowdown, (0.8 + 0.546_000_06) * 0.5);
        assert_eq!(result.acceleration, 0.06);
    }

    #[test]
    fn shallow_lava_applies_sixteenth_then_quarter_gravity() {
        let mut motion = motion();
        let result = fluid_travel_tick(
            &mut motion,
            FluidTravelContext {
                kind: FluidKind::Lava,
                fluid_height: 0.4,
                fluid_jump_threshold: 0.4,
                ..FluidTravelContext::default()
            },
            &CollisionScene::default(),
        );
        assert_eq!(result.horizontal_slowdown, 0.5);
        assert_eq!(motion.velocity.y, -0.025);
    }

    #[test]
    fn ability_wrapper_restores_entry_vertical_velocity_after_super_travel() {
        let mut motion = motion();
        let mut timers = TravelTimers::default();
        let result = ability_flight_tick(
            &mut motion,
            &mut timers,
            TravelAttributes::default(),
            AbilityFlightContext {
                jump: true,
                shift: false,
                sprinting: false,
                flying_speed: 0.05,
                superclass: AbilitySuperclassTravel::Ordinary(TravelContext::default()),
            },
            &CollisionScene::default(),
        );
        assert_eq!(result.restored_vertical_velocity, f64::from(0.15_f32) * 0.6);
    }
}
