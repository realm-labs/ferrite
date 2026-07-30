//! Firework, spit, shulker, fishing, eye, and fang projectile-family transitions.

use crate::entity::runtime::ent_004::geometry::Vector3;

#[must_use]
pub const fn firework_lifetime(flight: u8, draw_six: u8, draw_seven: u8) -> u16 {
    10 * (1 + flight as u16) + (draw_six % 6) as u16 + (draw_seven % 7) as u16
}

#[must_use]
pub const fn attached_firework_velocity(velocity: Vector3, owner_look: Vector3) -> Vector3 {
    velocity
        .add(owner_look.scale(0.1))
        .add(owner_look.scale(1.5).add(velocity.scale(-1.0)).scale(0.5))
}

#[must_use]
pub const fn firework_damage(explosion_count: u8) -> u16 {
    5 + 2 * explosion_count as u16
}

#[must_use]
pub const fn firework_target_admitted(squared_distance: f64, line_of_sight: bool) -> bool {
    squared_distance <= 25.0 && line_of_sight
}

#[must_use]
pub const fn llama_spit_motion(velocity: Vector3) -> Vector3 {
    Vector3::new(
        velocity.x * 0.99,
        (velocity.y - 0.06) * 0.99,
        velocity.z * 0.99,
    )
}

#[must_use]
pub const fn llama_spit_damage() -> u8 {
    1
}

#[must_use]
pub const fn shulker_homing_leg(draw_five: u8) -> u8 {
    10 * (1 + draw_five % 5)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShulkerBulletHit {
    pub damage: u8,
    pub levitation_ticks: u16,
}

#[must_use]
pub const fn shulker_bullet_hit() -> ShulkerBulletHit {
    ShulkerBulletHit {
        damage: 4,
        levitation_ticks: 200,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishingState {
    Flying,
    Hooked,
    Bobbing,
}

#[must_use]
pub const fn fishing_transition(
    state: FishingState,
    entity_hit: bool,
    in_water: bool,
) -> FishingState {
    match (state, entity_hit, in_water) {
        (FishingState::Flying, true, _) => FishingState::Hooked,
        (FishingState::Flying, false, true) => FishingState::Bobbing,
        _ => state,
    }
}

#[must_use]
pub const fn fishing_ground_expires(in_ground_ticks: u16) -> bool {
    in_ground_ticks >= 1_200
}

#[must_use]
pub const fn fishing_owner_in_range(squared_distance: f64) -> bool {
    squared_distance <= 1_024.0
}

#[must_use]
pub const fn fishing_loot_evaluated(retrieved: bool, loot_already_evaluated: bool) -> bool {
    retrieved && !loot_already_evaluated
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeTarget {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[must_use]
pub fn eye_target(origin: Vector3, target: Vector3) -> EyeTarget {
    let delta = Vector3::new(
        target.x - origin.x,
        target.y - origin.y,
        target.z - origin.z,
    );
    let horizontal = delta.x.hypot(delta.z);
    if horizontal > 12.0 {
        EyeTarget {
            x: origin.x + delta.x / horizontal * 12.0,
            y: origin.y + 8.0,
            z: origin.z + delta.z / horizontal * 12.0,
        }
    } else {
        EyeTarget {
            x: target.x,
            y: target.y,
            z: target.z,
        }
    }
}

#[must_use]
pub const fn eye_expires(life: u8) -> bool {
    life > 80
}

#[must_use]
pub const fn eye_survives(draw_five: u8) -> bool {
    !draw_five.is_multiple_of(5)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvokerFangStep {
    pub warmup: i8,
    pub life: u8,
    pub attack: bool,
    pub damage: u8,
    pub discard: bool,
}

#[must_use]
pub const fn evoker_fang_step(warmup: i8, life: u8) -> EvokerFangStep {
    let warmup = warmup.saturating_sub(1);
    let life = if warmup < 0 {
        life.saturating_sub(1)
    } else {
        life
    };
    EvokerFangStep {
        warmup,
        life,
        attack: warmup == -8,
        damage: if warmup == -8 { 6 } else { 0 },
        discard: warmup < 0 && life == 0,
    }
}
