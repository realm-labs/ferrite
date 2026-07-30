//! Throwable-item flight and snowball, egg, bottle, pearl, and potion hit transitions.

use crate::entity::runtime::ent_004::geometry::Vector3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrowableKind {
    Ordinary,
    Potion,
    ExperienceBottle,
}

#[must_use]
pub const fn gravity(kind: ThrowableKind) -> f64 {
    match kind {
        ThrowableKind::Ordinary => 0.03,
        ThrowableKind::Potion => 0.05,
        ThrowableKind::ExperienceBottle => 0.07,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThrowableMotion {
    pub velocity: Vector3,
    pub bubble_particles: u8,
}

#[must_use]
pub const fn throwable_motion(
    velocity: Vector3,
    kind: ThrowableKind,
    in_water: bool,
) -> ThrowableMotion {
    let after_gravity = Vector3::new(velocity.x, velocity.y - gravity(kind), velocity.z);
    let inertia = if in_water { 0.8 } else { 0.99 };
    ThrowableMotion {
        velocity: after_gravity.scale(inertia),
        bubble_particles: if in_water { 4 } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrowableTickStage {
    Gravity,
    Inertia,
    Sweep,
    Move,
    Rotate,
    ApplyBlockEffects,
    BaseTick,
    ResolveLiveHit,
}

pub const THROWABLE_TICK_ORDER: [ThrowableTickStage; 8] = [
    ThrowableTickStage::Gravity,
    ThrowableTickStage::Inertia,
    ThrowableTickStage::Sweep,
    ThrowableTickStage::Move,
    ThrowableTickStage::Rotate,
    ThrowableTickStage::ApplyBlockEffects,
    ThrowableTickStage::BaseTick,
    ThrowableTickStage::ResolveLiveHit,
];

#[must_use]
pub const fn snowball_damage(target_is_blaze: bool) -> f32 {
    if target_is_blaze { 3.0 } else { 0.0 }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleThrowableHit {
    pub damage: f32,
    pub broadcast_event: bool,
    pub discard: bool,
}

#[must_use]
pub const fn snowball_hit(target_is_blaze: bool) -> SimpleThrowableHit {
    SimpleThrowableHit {
        damage: snowball_damage(target_is_blaze),
        broadcast_event: true,
        discard: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggHatch {
    pub first_draw_consumed: bool,
    pub second_draw_consumed: bool,
    pub attempted_chickens: u8,
    pub spawned_chickens: u8,
    pub preserves_variant: bool,
    pub event_particles: u8,
    pub damage: u8,
    pub discard: bool,
}

#[must_use]
pub fn egg_hatch(first_draw_eight: u8, second_draw_thirty_two: u8, placement: &[bool]) -> EggHatch {
    if !first_draw_eight.is_multiple_of(8) {
        return EggHatch {
            first_draw_consumed: true,
            second_draw_consumed: false,
            attempted_chickens: 0,
            spawned_chickens: 0,
            preserves_variant: true,
            event_particles: 3,
            damage: 0,
            discard: true,
        };
    }
    let attempted_chickens = if second_draw_thirty_two.is_multiple_of(32) {
        4
    } else {
        1
    };
    let spawned_chickens = placement
        .iter()
        .take(usize::from(attempted_chickens))
        .take_while(|can_place| **can_place)
        .count() as u8;
    EggHatch {
        first_draw_consumed: true,
        second_draw_consumed: true,
        attempted_chickens,
        spawned_chickens,
        preserves_variant: true,
        event_particles: 3,
        damage: 0,
        discard: true,
    }
}

#[must_use]
pub const fn experience_bottle_value(first_draw_five: u8, second_draw_five: u8) -> u8 {
    3 + first_draw_five % 5 + second_draw_five % 5
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperienceDirection {
    BlockNormal,
    ReverseFlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExperienceBottleHit {
    pub experience: u8,
    pub direction: ExperienceDirection,
    pub broadcast_event: bool,
    pub discard: bool,
}

#[must_use]
pub const fn experience_bottle_hit(
    first_draw_five: u8,
    second_draw_five: u8,
    block_hit: bool,
) -> ExperienceBottleHit {
    ExperienceBottleHit {
        experience: experience_bottle_value(first_draw_five, second_draw_five),
        direction: if block_hit {
            ExperienceDirection::BlockNormal
        } else {
            ExperienceDirection::ReverseFlight
        },
        broadcast_event: true,
        discard: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PearlHitInput {
    pub owner_valid: bool,
    pub owner_is_player: bool,
    pub endermite_draw: f64,
    pub spawn_mobs: bool,
    pub spawn_monsters: bool,
    pub peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PearlHit {
    pub portal_particles: u8,
    pub endermite_draw_consumed: bool,
    pub spawn_endermite: bool,
    pub teleport_owner: bool,
    pub reset_velocity_rotation: bool,
    pub reset_fall_and_impulse: bool,
    pub owner_damage: u8,
    pub discard: bool,
}

#[must_use]
pub fn ender_pearl_hit(input: PearlHitInput) -> PearlHit {
    if !input.owner_valid {
        return PearlHit {
            portal_particles: 32,
            endermite_draw_consumed: false,
            spawn_endermite: false,
            teleport_owner: false,
            reset_velocity_rotation: false,
            reset_fall_and_impulse: false,
            owner_damage: 0,
            discard: true,
        };
    }
    let draw_consumed = input.owner_is_player;
    let spawn_endermite = draw_consumed
        && input.endermite_draw < 0.05
        && input.spawn_mobs
        && input.spawn_monsters
        && !input.peaceful;
    PearlHit {
        portal_particles: 32,
        endermite_draw_consumed: draw_consumed,
        spawn_endermite,
        teleport_owner: true,
        reset_velocity_rotation: true,
        reset_fall_and_impulse: true,
        owner_damage: u8::from(input.owner_is_player) * 5,
        discard: true,
    }
}

#[must_use]
pub const fn pearl_keeps_chunk_ticket(owner_is_live_player: bool) -> bool {
    owner_is_live_player
}

#[must_use]
pub const fn pearl_vanishes(owner_dead: bool, vanish_on_death: bool) -> bool {
    owner_dead && vanish_on_death
}

#[must_use]
pub const fn throwable_may_use_portal(portal_allowed: bool) -> bool {
    portal_allowed
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterPotionTarget {
    pub squared_distance: f64,
    pub on_fire: bool,
    pub water_sensitive: bool,
    pub axolotl: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaterPotionEffect {
    pub extinguish: bool,
    pub damage_water_sensitive: bool,
    pub rehydrate_axolotl: bool,
}

#[must_use]
pub const fn water_potion_effect(target: WaterPotionTarget) -> WaterPotionEffect {
    let inside = target.squared_distance < 16.0;
    WaterPotionEffect {
        extinguish: inside && target.on_fire,
        damage_water_sensitive: inside && target.water_sensitive,
        rehydrate_axolotl: inside && target.axolotl,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DousePosition {
    ImpactAdjacent,
    Opposite,
    North,
    South,
    West,
    East,
}

pub const WATER_POTION_DOUSE_ORDER: [DousePosition; 6] = [
    DousePosition::ImpactAdjacent,
    DousePosition::Opposite,
    DousePosition::North,
    DousePosition::South,
    DousePosition::West,
    DousePosition::East,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplashQuery {
    pub center_at_hit: bool,
    pub inflate_x: f64,
    pub inflate_y: f64,
    pub inflate_z: f64,
    pub target_margin: f64,
}

#[must_use]
pub const fn splash_query(dynamic_margin: f64) -> SplashQuery {
    SplashQuery {
        center_at_hit: true,
        inflate_x: 4.0,
        inflate_y: 2.0,
        inflate_z: 4.0,
        target_margin: dynamic_margin,
    }
}

#[must_use]
pub fn splash_scale(distance_squared: f64) -> f64 {
    (1.0 - distance_squared.sqrt() / 4.0).max(0.0)
}

#[must_use]
pub fn splash_duration(distance_squared: f64, duration: u32, component_scale: f64) -> Option<u32> {
    let scaled = (splash_scale(distance_squared) * f64::from(duration) * component_scale + 0.5)
        .floor() as u32;
    (scaled > 20).then_some(scaled)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LingeringCloud {
    pub radius: f32,
    pub radius_on_use: f32,
    pub duration: u32,
    pub wait_time: u32,
    pub radius_per_tick: f32,
}

#[must_use]
pub fn lingering_cloud() -> LingeringCloud {
    let radius = 3.0;
    let duration = 600;
    LingeringCloud {
        radius,
        radius_on_use: -0.5,
        duration,
        wait_time: 10,
        radius_per_tick: -radius / duration as f32,
    }
}
