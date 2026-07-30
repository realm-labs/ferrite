//! Exact local state machines shared by hostile ENT-001 subtypes.

use crate::entity::runtime::ent_001::undead::Difficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatStep {
    pub resting: bool,
    pub level_event: Option<u16>,
    pub flap: bool,
}

#[must_use]
pub const fn bat_step(
    resting: bool,
    ceiling_supports: bool,
    player_nearby: bool,
    ceiling_rest_draw: u8,
    tick_count: u64,
) -> BatStep {
    if resting && (!ceiling_supports || player_nearby) {
        BatStep {
            resting: false,
            level_event: Some(1025),
            flap: false,
        }
    } else if !resting && ceiling_supports && ceiling_rest_draw == 0 {
        BatStep {
            resting: true,
            level_event: None,
            flap: false,
        }
    } else {
        BatStep {
            resting,
            level_event: None,
            flap: !resting && tick_count.is_multiple_of(10),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlazeAttack {
    None,
    Melee,
    Warmup,
    Fireball { volley_index: u8, level_event: u16 },
    Cooldown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlazeAttackStep {
    pub attack_step: u8,
    pub delay: i16,
    pub charged: bool,
    pub action: BlazeAttack,
}

#[must_use]
pub const fn blaze_attack_step(
    mut step: u8,
    delay: i16,
    target_in_melee: bool,
    target_in_fireball_range: bool,
    has_line_of_sight: bool,
) -> BlazeAttackStep {
    let delay = delay.wrapping_sub(1);
    if target_in_melee {
        let melee = has_line_of_sight && delay <= 0;
        return BlazeAttackStep {
            attack_step: step,
            delay: if melee { 20 } else { delay },
            charged: step > 1,
            action: if melee {
                BlazeAttack::Melee
            } else {
                BlazeAttack::None
            },
        };
    }
    if !target_in_fireball_range || !has_line_of_sight {
        return BlazeAttackStep {
            attack_step: step,
            delay,
            charged: step > 1,
            action: BlazeAttack::None,
        };
    }
    if delay > 0 {
        return BlazeAttackStep {
            attack_step: step,
            delay,
            charged: step > 1,
            action: BlazeAttack::None,
        };
    }
    step += 1;
    match step {
        1 => BlazeAttackStep {
            attack_step: step,
            delay: 60,
            charged: true,
            action: BlazeAttack::Warmup,
        },
        2..=4 => BlazeAttackStep {
            attack_step: step,
            delay: 6,
            charged: true,
            action: BlazeAttack::Fireball {
                volley_index: step - 1,
                level_event: 1018,
            },
        },
        _ => BlazeAttackStep {
            attack_step: 0,
            delay: 100,
            charged: false,
            action: BlazeAttack::Cooldown,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreezeShotPhase {
    Idle,
    Charging,
    Fired,
    Recovering,
    Cooldown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreezeShotStep {
    pub phase: BreezeShotPhase,
    pub ticks: u8,
    pub projectile_spawned: bool,
}

#[must_use]
pub const fn breeze_shot_step(phase: BreezeShotPhase, ticks: u8) -> BreezeShotStep {
    match phase {
        BreezeShotPhase::Idle => BreezeShotStep {
            phase: BreezeShotPhase::Charging,
            ticks: 15,
            projectile_spawned: false,
        },
        BreezeShotPhase::Charging if ticks > 1 => BreezeShotStep {
            phase,
            ticks: ticks - 1,
            projectile_spawned: false,
        },
        BreezeShotPhase::Charging => BreezeShotStep {
            phase: BreezeShotPhase::Fired,
            ticks: 1,
            projectile_spawned: true,
        },
        BreezeShotPhase::Fired => BreezeShotStep {
            phase: BreezeShotPhase::Recovering,
            ticks: 4,
            projectile_spawned: false,
        },
        BreezeShotPhase::Recovering if ticks > 1 => BreezeShotStep {
            phase,
            ticks: ticks - 1,
            projectile_spawned: false,
        },
        BreezeShotPhase::Recovering => BreezeShotStep {
            phase: BreezeShotPhase::Cooldown,
            ticks: 10,
            projectile_spawned: false,
        },
        BreezeShotPhase::Cooldown if ticks > 1 => BreezeShotStep {
            phase,
            ticks: ticks - 1,
            projectile_spawned: false,
        },
        BreezeShotPhase::Cooldown => BreezeShotStep {
            phase: BreezeShotPhase::Idle,
            ticks: 0,
            projectile_spawned: false,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreezeWindCharge {
    pub direct_damage: u8,
    pub explosion_strength: u8,
    pub damages_entities: bool,
    pub triggers_blocks: bool,
}

#[must_use]
pub const fn breeze_wind_charge_hit() -> BreezeWindCharge {
    BreezeWindCharge {
        direct_damage: 1,
        explosion_strength: 3,
        damages_entities: false,
        triggers_blocks: true,
    }
}

#[must_use]
pub const fn endermite_persists(persistent: bool, lifetime: u16) -> bool {
    persistent || lifetime < 2_400
}

#[must_use]
pub const fn endermite_pearl_spawn(
    spawn_mobs: bool,
    spawn_monsters: bool,
    difficulty: Difficulty,
    draw_twenty: u8,
) -> bool {
    spawn_mobs && spawn_monsters && !matches!(difficulty, Difficulty::Peaceful) && draw_twenty == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhastCharge {
    Idle,
    Warning,
    Charged,
    Fired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhastAttackStep {
    pub charge_time: i16,
    pub phase: GhastCharge,
    pub level_event: Option<u16>,
    pub fireball_power: i8,
}

#[must_use]
pub fn ghast_attack_step(
    charge_time: i16,
    target_valid: bool,
    silent: bool,
    explosion_power: i8,
) -> GhastAttackStep {
    if !target_valid {
        return GhastAttackStep {
            charge_time: if charge_time > 0 {
                charge_time - 1
            } else {
                charge_time
            },
            phase: GhastCharge::Idle,
            level_event: None,
            fireball_power: explosion_power,
        };
    }
    let next = charge_time + 1;
    match next {
        10 => GhastAttackStep {
            charge_time: next,
            phase: GhastCharge::Warning,
            level_event: (!silent).then_some(1015),
            fireball_power: explosion_power,
        },
        11..=19 => GhastAttackStep {
            charge_time: next,
            phase: GhastCharge::Charged,
            level_event: None,
            fireball_power: explosion_power,
        },
        20.. => GhastAttackStep {
            charge_time: -40,
            phase: GhastCharge::Fired,
            level_event: (!silent).then_some(1016),
            fireball_power: explosion_power,
        },
        _ => GhastAttackStep {
            charge_time: next,
            phase: GhastCharge::Idle,
            level_event: None,
            fireball_power: explosion_power,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeProfile {
    pub size: u8,
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub contact_damage: f32,
    pub armor: f32,
    pub experience: u8,
    pub attachment: f32,
}

#[must_use]
pub fn slime_profile(raw_size: i32, magma: bool) -> SlimeProfile {
    let size = raw_size.clamp(1, 127) as u8;
    let scalar = f32::from(size);
    SlimeProfile {
        size,
        max_health: scalar * scalar,
        movement_speed: 0.2 + 0.1 * scalar,
        attack_damage: scalar,
        contact_damage: scalar + if magma { 2.0 } else { 0.0 },
        armor: if magma { 3.0 * scalar } else { 0.0 },
        experience: size,
        attachment: 0.015625 * scalar,
    }
}

#[must_use]
pub const fn slime_jump_delay(draw_twenty: u8, magma: bool, aggressive: bool) -> u8 {
    let mut delay = draw_twenty + 10;
    if magma {
        delay *= 4;
    }
    if aggressive {
        delay /= 3;
    }
    delay
}

#[must_use]
pub const fn slime_child_count(size: u8, draw_three: u8) -> u8 {
    if size > 1 { 2 + draw_three % 3 } else { 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiderFinalization {
    pub skeleton_jockey: bool,
    pub group_effect: Option<SpiderEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiderEffect {
    Speed,
    Strength,
    Regeneration,
    Invisibility,
}

#[must_use]
pub const fn spider_finalization(
    jockey_draw: u8,
    hard_special_admitted: bool,
    effect_draw: u8,
) -> SpiderFinalization {
    let group_effect = if hard_special_admitted {
        Some(match effect_draw % 4 {
            0 => SpiderEffect::Speed,
            1 => SpiderEffect::Strength,
            2 => SpiderEffect::Regeneration,
            _ => SpiderEffect::Invisibility,
        })
    } else {
        None
    };
    SpiderFinalization {
        skeleton_jockey: jockey_draw == 0,
        group_effect,
    }
}

#[must_use]
pub const fn cave_spider_poison(difficulty: Difficulty) -> u16 {
    match difficulty {
        Difficulty::Normal => 140,
        Difficulty::Hard => 300,
        Difficulty::Peaceful | Difficulty::Easy => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomSize {
    pub stored_size: u8,
    pub scale: f32,
    pub attack_damage: f32,
}

#[must_use]
pub fn phantom_size(previous: PhantomSize, requested: i32) -> PhantomSize {
    let stored_size = requested.clamp(0, 64) as u8;
    if stored_size == previous.stored_size {
        return PhantomSize {
            stored_size,
            ..previous
        };
    }
    PhantomSize {
        stored_size,
        scale: 1.0 + 0.15 * f32::from(stored_size),
        attack_damage: 6.0 + f32::from(stored_size),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardianBeam {
    pub attack_time: i16,
    pub target_synced: bool,
    pub event: Option<u8>,
    pub magic_damage: u8,
    pub melee_damage: u8,
    pub completes: bool,
}

#[must_use]
pub fn guardian_beam_step(attack_time: i16, elder: bool, hard: bool) -> GuardianBeam {
    let next = attack_time + 1;
    let duration = if elder { 60 } else { 80 };
    GuardianBeam {
        attack_time: next,
        target_synced: next == 0,
        event: (next == 0).then_some(21),
        magic_damage: if next == duration {
            match (elder, hard) {
                (true, true) => 5,
                (true, false) | (false, true) => 3,
                (false, false) => 1,
            }
        } else {
            0
        },
        melee_damage: if next == duration {
            if elder { 8 } else { 6 }
        } else {
            0
        },
        completes: next >= duration,
    }
}

#[must_use]
pub const fn guardian_thorns(
    moving: bool,
    direct_living_attacker: bool,
    incoming_admitted: bool,
) -> u8 {
    if !moving && direct_living_attacker && incoming_admitted {
        2
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShulkerPeek {
    pub raw: u8,
    pub armor_bonus: u8,
}

#[must_use]
pub const fn shulker_peek(raw: u8) -> ShulkerPeek {
    ShulkerPeek {
        raw,
        armor_bonus: if raw == 0 { 20 } else { 0 },
    }
}

#[must_use]
pub fn shulker_bullet_cooldown(draw_ten: u8) -> u16 {
    20 + u16::from(draw_ten % 10) * 10
}

#[must_use]
pub const fn shulker_emergency_teleport(
    health: u16,
    max_health: u16,
    damage_admitted: bool,
) -> bool {
    damage_admitted && health * 2 < max_health
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VexLifeStep {
    pub life_ticks: i32,
    pub starvation_damage: u8,
}

#[must_use]
pub const fn vex_life_step(life_ticks: Option<i32>) -> Option<VexLifeStep> {
    let life_ticks = match life_ticks {
        Some(value) => value - 1,
        None => return None,
    };
    Some(VexLifeStep {
        life_ticks,
        starvation_damage: if life_ticks < 0 && life_ticks % 20 == 0 {
            1
        } else {
            0
        },
    })
}
