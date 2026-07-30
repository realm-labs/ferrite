//! Hurting-projectile motion, deflection, fireballs, skulls, and wind charges.

use crate::entity::runtime::ent_004::geometry::Vector3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HurtingTickStage {
    Acceleration,
    Inertia,
    Sweep,
    Move,
    BaseTick,
    ResolveHit,
    Trail,
}

pub const HURTING_TICK_ORDER: [HurtingTickStage; 7] = [
    HurtingTickStage::Acceleration,
    HurtingTickStage::Inertia,
    HurtingTickStage::Sweep,
    HurtingTickStage::Move,
    HurtingTickStage::BaseTick,
    HurtingTickStage::ResolveHit,
    HurtingTickStage::Trail,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HurtingMotion {
    pub velocity: Vector3,
    pub discard: bool,
}

#[must_use]
pub fn hurting_motion(
    velocity: Vector3,
    acceleration: Vector3,
    in_liquid: bool,
    server_side: bool,
    owner_missing_or_removed: bool,
    chunk_loaded: bool,
) -> HurtingMotion {
    let discard = server_side && (owner_missing_or_removed || !chunk_loaded);
    let accelerated = velocity.add(acceleration.normalize().scale(0.1));
    HurtingMotion {
        velocity: accelerated.scale(if in_liquid { 0.8 } else { 0.95 }),
        discard,
    }
}

#[must_use]
pub fn deflected_acceleration(direction: Vector3, attack_deflection: bool) -> Vector3 {
    direction
        .normalize()
        .scale(if attack_deflection { 0.1 } else { 0.05 })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LargeFireballHit {
    pub damage: u8,
    pub explosion_power: u8,
}

#[must_use]
pub const fn large_fireball_hit(stored_explosion_power: u8) -> LargeFireballHit {
    LargeFireballHit {
        damage: 6,
        explosion_power: stored_explosion_power,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmallFireballEntityHit {
    pub damage: u8,
    pub restore_prior_fire_on_failed_damage: bool,
}

#[must_use]
pub const fn small_fireball_entity_hit(damage_succeeded: bool) -> SmallFireballEntityHit {
    SmallFireballEntityHit {
        damage: 5,
        restore_prior_fire_on_failed_damage: !damage_succeeded,
    }
}

#[must_use]
pub const fn small_fireball_places_fire(
    empty_target: bool,
    owner_is_mob: bool,
    mob_griefing: bool,
) -> bool {
    empty_target && (!owner_is_mob || mob_griefing)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitherSkullHit {
    pub damage: u8,
    pub heal_owner: u8,
    pub wither_ticks: u16,
    pub wither_amplifier: u8,
    pub explosion_power: u8,
}

#[must_use]
pub const fn wither_skull_hit(
    has_owner: bool,
    target_killed: bool,
    difficulty: Difficulty,
) -> WitherSkullHit {
    let wither_ticks = match difficulty {
        Difficulty::Normal => 200,
        Difficulty::Hard => 800,
        Difficulty::Peaceful | Difficulty::Easy => 0,
    };
    WitherSkullHit {
        damage: if has_owner { 8 } else { 5 },
        heal_owner: if has_owner && target_killed { 5 } else { 0 },
        wither_ticks,
        wither_amplifier: 1,
        explosion_power: 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonCloud {
    pub duration: u16,
    pub radius: f32,
    pub radius_per_tick: f32,
    pub instant_damage: bool,
}

#[must_use]
pub const fn dragon_fireball_cloud(initial_radius: f32) -> DragonCloud {
    DragonCloud {
        duration: 600,
        radius: initial_radius,
        radius_per_tick: (7.0 - initial_radius) / 600.0,
        instant_damage: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindChargeOwner {
    Player,
    Breeze,
}

#[must_use]
pub const fn wind_charge_acceleration() -> f64 {
    0.0
}

#[must_use]
pub const fn wind_charge_inertia() -> f64 {
    1.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindChargeHit {
    pub damage: u8,
    pub explosion_radius: f32,
    pub block_offset: Vector3,
}

#[must_use]
pub const fn wind_charge_hit(
    owner: WindChargeOwner,
    block_normal: Option<Vector3>,
) -> WindChargeHit {
    WindChargeHit {
        damage: 1,
        explosion_radius: match owner {
            WindChargeOwner::Player => 1.2,
            WindChargeOwner::Breeze => 3.0,
        },
        block_offset: match block_normal {
            Some(normal) => normal.scale(0.25),
            None => Vector3::ZERO,
        },
    }
}

#[must_use]
pub const fn wind_charge_deflectable(owner: WindChargeOwner, tick_count: u32) -> bool {
    !matches!(owner, WindChargeOwner::Player) || tick_count >= 5
}

#[must_use]
pub const fn wind_charge_ignores(target_wind_charge: bool, target_end_crystal: bool) -> bool {
    target_wind_charge || target_end_crystal
}

#[must_use]
pub const fn wind_charge_explodes_above_height(impact_y: i32, maximum_build_height: i32) -> bool {
    impact_y > maximum_build_height.saturating_add(30)
}
