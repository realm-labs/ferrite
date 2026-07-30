//! Specialized effect cadence, instant effects, omens, hurt hooks, and removal hooks.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodicEffect {
    Regeneration,
    Poison,
    Wither,
}

#[must_use]
pub const fn periodic_interval(effect: PeriodicEffect, amplifier: u8) -> i32 {
    let base = match effect {
        PeriodicEffect::Regeneration => 50_i32,
        PeriodicEffect::Poison => 25,
        PeriodicEffect::Wither => 40,
    };
    base >> (amplifier & 31)
}

#[must_use]
pub const fn periodic_scheduled(effect: PeriodicEffect, amplifier: u8, cadence_value: i32) -> bool {
    let interval = periodic_interval(effect, amplifier);
    interval <= 0 || cadence_value % interval == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodicAction {
    Heal { amount: u8 },
    Damage { amount: u8 },
    None,
}

#[must_use]
pub const fn periodic_action(effect: PeriodicEffect, health: f32) -> PeriodicAction {
    match effect {
        PeriodicEffect::Regeneration => PeriodicAction::Heal { amount: 1 },
        PeriodicEffect::Poison if health > 1.0 => PeriodicAction::Damage { amount: 1 },
        PeriodicEffect::Poison => PeriodicAction::None,
        PeriodicEffect::Wither => PeriodicAction::Damage { amount: 1 },
    }
}

#[must_use]
pub fn hunger_exhaustion(amplifier: u8) -> f32 {
    0.005 * (f32::from(amplifier) + 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Saturation {
    pub food: u16,
    pub modifier: u8,
}

#[must_use]
pub const fn saturation(amplifier: u8) -> Saturation {
    Saturation {
        food: amplifier as u16 + 1,
        modifier: 1,
    }
}

#[must_use]
pub const fn absorption_floor(amplifier: u8) -> u16 {
    4 * (amplifier as u16 + 1)
}

#[must_use]
pub fn absorption_on_start(current_absorption: f32, amplifier: u8) -> f32 {
    current_absorption.max(f32::from(absorption_floor(amplifier)))
}

#[must_use]
pub const fn absorption_keeps_ticking(current_absorption: f32) -> bool {
    current_absorption > 0.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstantEffect {
    Heal,
    Harm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstantOutcome {
    pub healing: i32,
    pub damage: i32,
    pub indirect_magic_source: bool,
}

#[must_use]
pub fn instant_effect(
    effect: InstantEffect,
    amplifier: u8,
    scale: f64,
    inverted_by_entity_tag: bool,
    source_present: bool,
) -> InstantOutcome {
    let (healing, damage) = match effect {
        InstantEffect::Heal => (
            4_i32.wrapping_shl(u32::from(amplifier) & 31),
            6_i32.wrapping_shl(u32::from(amplifier) & 31),
        ),
        InstantEffect::Harm => (
            6_i32.wrapping_shl(u32::from(amplifier) & 31),
            4_i32.wrapping_shl(u32::from(amplifier) & 31),
        ),
    };
    let (healing, damage) = if inverted_by_entity_tag {
        (damage, healing)
    } else {
        (healing, damage)
    };
    InstantOutcome {
        healing: java_scaled_instant(scale, healing),
        damage: java_scaled_instant(scale, damage),
        indirect_magic_source: source_present,
    }
}

fn java_scaled_instant(scale: f64, amount: i32) -> i32 {
    (scale * f64::from(amount) + 0.5) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Applicability {
    pub infested: bool,
    pub oozing: bool,
    pub poison: bool,
    pub regeneration: bool,
}

#[must_use]
pub const fn effect_applicable(effect: ApplicabilityEffect, tags: Applicability) -> bool {
    match effect {
        ApplicabilityEffect::Infested => tags.infested,
        ApplicabilityEffect::Oozing => tags.oozing,
        ApplicabilityEffect::Poison => tags.poison,
        ApplicabilityEffect::Regeneration => tags.regeneration,
        ApplicabilityEffect::Other => true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicabilityEffect {
    Infested,
    Oozing,
    Poison,
    Regeneration,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BadOmen {
    pub remove: bool,
    pub add_raid_omen_duration: u16,
    pub save_position: bool,
}

#[must_use]
pub const fn bad_omen_tick(
    nonspectator_player: bool,
    non_peaceful: bool,
    in_village: bool,
    raid_has_capacity: bool,
) -> BadOmen {
    let convert = nonspectator_player && non_peaceful && in_village && raid_has_capacity;
    BadOmen {
        remove: convert,
        add_raid_omen_duration: if convert { 600 } else { 0 },
        save_position: convert,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaidOmen {
    pub create_or_extend_raid: bool,
    pub clear_saved_position: bool,
    pub remove: bool,
}

#[must_use]
pub const fn raid_omen_tick(remaining_duration: i32) -> RaidOmen {
    let trigger = remaining_duration == 1;
    RaidOmen {
        create_or_extend_raid: trigger,
        clear_saved_position: trigger,
        remove: trigger,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InfestedHurt {
    pub trigger: bool,
    pub silverfish: u8,
}

#[must_use]
pub fn infested_hurt(chance_draw: f32, count_draw_two: u8) -> InfestedHurt {
    let trigger = chance_draw <= 0.1;
    InfestedHurt {
        trigger,
        silverfish: if trigger { 1 + count_draw_two % 2 } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalReason {
    Killed,
    Discarded,
    Unloaded,
    ChangedDimension,
}

#[must_use]
pub fn wind_charged_explosion(reason: RemovalReason, draw: f32) -> Option<f32> {
    matches!(reason, RemovalReason::Killed).then_some(3.0 + draw * 2.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeavingRemoval {
    pub attempts: u8,
    pub samples_per_attempt: u8,
    pub may_place: bool,
}

#[must_use]
pub const fn weaving_removal(
    reason: RemovalReason,
    player_victim: bool,
    mob_griefing: bool,
    draw_two: u8,
) -> WeavingRemoval {
    let may_place = matches!(reason, RemovalReason::Killed) && (player_victim || mob_griefing);
    WeavingRemoval {
        attempts: if may_place { 2 + draw_two % 2 } else { 0 },
        samples_per_attempt: if may_place { 15 } else { 0 },
        may_place,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OozingPlan {
    pub query_nearby: bool,
    pub nearby_scan_limit: u16,
    pub spawn_attempts: u8,
}

#[must_use]
pub fn oozing_plan(
    reason: RemovalReason,
    max_cramming: i32,
    nearby_slimes_before_limit: u16,
) -> OozingPlan {
    if !matches!(reason, RemovalReason::Killed) {
        return OozingPlan {
            query_nearby: false,
            nearby_scan_limit: 0,
            spawn_attempts: 0,
        };
    }
    if max_cramming < 1 {
        return OozingPlan {
            query_nearby: false,
            nearby_scan_limit: 0,
            spawn_attempts: 2,
        };
    }
    let limit = max_cramming.min(i32::from(u16::MAX)) as u16;
    let nearby = nearby_slimes_before_limit.min(limit);
    OozingPlan {
        query_nearby: true,
        nearby_scan_limit: limit,
        spawn_attempts: i32::from(2_u8).clamp(0, i32::from(limit.saturating_sub(nearby))) as u8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OozingAttempt {
    pub created: bool,
    pub slime_size: u8,
    pub y_offset: f64,
    pub yaw: Option<f32>,
    pub pitch: f32,
    pub finalize_spawn: bool,
    pub rollback_failed_insertion: bool,
}

#[must_use]
pub fn oozing_attempt(construction_succeeded: bool, yaw_draw: f32) -> OozingAttempt {
    OozingAttempt {
        created: construction_succeeded,
        slime_size: if construction_succeeded { 2 } else { 0 },
        y_offset: if construction_succeeded { 0.5 } else { 0.0 },
        yaw: construction_succeeded.then_some(yaw_draw * 360.0),
        pitch: 0.0,
        finalize_spawn: false,
        rollback_failed_insertion: false,
    }
}

#[must_use]
pub const fn client_particle_denominator(visible: bool, ambient: bool) -> u16 {
    let base = if visible { 4 } else { 15 };
    base * if ambient { 5 } else { 1 }
}
