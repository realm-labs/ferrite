//! Damage wrapper admission, outer transforms, cooldown, attribution, and callback planning.

pub const FLOAT_STAT_UPPER_BOUND: f32 = f32::MAX / 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseImmunityInput {
    pub removed: bool,
    pub invulnerable_flag: bool,
    pub bypasses_invulnerability: bool,
    pub creative_player_source: bool,
    pub fire_source: bool,
    pub fire_immune: bool,
    pub fall_source: bool,
    pub fall_damage_immune_type: bool,
}

#[must_use]
pub const fn base_immunity(input: BaseImmunityInput) -> bool {
    input.removed
        || (input.invulnerable_flag
            && !input.bypasses_invulnerability
            && !input.creative_player_source)
        || (input.fire_source && input.fire_immune)
        || (input.fall_source && input.fall_damage_immune_type)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LivingImmunityInput {
    pub base: BaseImmunityInput,
    pub enchantment_immune: bool,
}

#[must_use]
pub const fn living_immunity(input: LivingImmunityInput) -> bool {
    base_immunity(input.base) || input.enchantment_immune
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerRuleImmunityInput {
    pub living_immune: bool,
    pub drowning_source: bool,
    pub drowning_damage: bool,
    pub fall_source: bool,
    pub fall_damage: bool,
    pub fire_source: bool,
    pub fire_damage: bool,
    pub freeze_source: bool,
    pub freeze_damage: bool,
}

#[must_use]
pub const fn player_rule_immunity(input: PlayerRuleImmunityInput) -> bool {
    if input.living_immune {
        true
    } else if input.drowning_source {
        !input.drowning_damage
    } else if input.fall_source {
        !input.fall_damage
    } else if input.fire_source {
        !input.fire_damage
    } else if input.freeze_source {
        !input.freeze_damage
    } else {
        false
    }
}

#[must_use]
pub const fn server_player_immunity(
    player_immune: bool,
    changing_dimension: bool,
    exact_ender_pearl_source: bool,
    client_loaded: bool,
) -> bool {
    player_immune || (changing_dimension && !exact_ender_pearl_source) || !client_loaded
}

#[must_use]
pub fn difficulty_scale(amount: f32, scales_with_difficulty: bool, difficulty: Difficulty) -> f32 {
    if !scales_with_difficulty {
        return amount;
    }
    match difficulty {
        Difficulty::Peaceful => 0.0,
        Difficulty::Easy => (amount / 2.0 + 1.0).min(amount),
        Difficulty::Normal => amount,
        Difficulty::Hard => amount * 3.0 / 2.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageAbort {
    ServerImmunity,
    PlayerPvp,
    ArrowOwnerPvp,
    AbilityInvulnerability,
    PlayerDead,
    PlayerDifficultyZero,
    LivingImmunity,
    LivingDead,
    FireResistance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WrapperInput {
    pub server_player: bool,
    pub player: bool,
    pub server_immunity: bool,
    pub player_pvp_disallowed: bool,
    pub arrow_owner_pvp_disallowed: bool,
    pub ability_invulnerable: bool,
    pub bypasses_invulnerability: bool,
    pub dead_or_dying: bool,
    pub scales_with_difficulty: bool,
    pub difficulty: Difficulty,
    pub living_immunity: bool,
    pub fire_source: bool,
    pub fire_resistance: bool,
    pub sleeping: bool,
    pub amount: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WrapperOutcome {
    pub abort: Option<DamageAbort>,
    pub amount: f32,
    pub no_action_time_reset: bool,
    pub shoulder_entities_removed: bool,
    pub woke_sleeping: bool,
}

#[must_use]
pub fn wrapper_admission(input: WrapperInput) -> WrapperOutcome {
    let mut outcome = WrapperOutcome {
        abort: None,
        amount: input.amount,
        no_action_time_reset: false,
        shoulder_entities_removed: false,
        woke_sleeping: false,
    };
    if input.server_player {
        if input.server_immunity {
            outcome.abort = Some(DamageAbort::ServerImmunity);
            return outcome;
        }
        if input.player_pvp_disallowed {
            outcome.abort = Some(DamageAbort::PlayerPvp);
            return outcome;
        }
        if input.arrow_owner_pvp_disallowed {
            outcome.abort = Some(DamageAbort::ArrowOwnerPvp);
            return outcome;
        }
    }
    if input.player {
        if input.ability_invulnerable && !input.bypasses_invulnerability {
            outcome.abort = Some(DamageAbort::AbilityInvulnerability);
            return outcome;
        }
        outcome.no_action_time_reset = true;
        if input.dead_or_dying {
            outcome.abort = Some(DamageAbort::PlayerDead);
            return outcome;
        }
        outcome.shoulder_entities_removed = true;
        outcome.amount =
            difficulty_scale(input.amount, input.scales_with_difficulty, input.difficulty);
        if outcome.amount == 0.0 {
            outcome.abort = Some(DamageAbort::PlayerDifficultyZero);
            return outcome;
        }
    }
    if input.living_immunity {
        outcome.abort = Some(DamageAbort::LivingImmunity);
        return outcome;
    }
    if input.dead_or_dying {
        outcome.abort = Some(DamageAbort::LivingDead);
        return outcome;
    }
    if input.fire_source && input.fire_resistance {
        outcome.abort = Some(DamageAbort::FireResistance);
        return outcome;
    }
    outcome.woke_sleeping = input.sleeping;
    outcome.no_action_time_reset = true;
    outcome
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageTransform {
    pub original: f32,
    pub blocked_amount: f32,
    pub remaining: f32,
    pub blocked: bool,
    pub damage_helmet: bool,
    pub helmet_damage_input: f32,
}

#[must_use]
pub fn transform_after_block(
    amount: f32,
    blocked_amount: f32,
    freezing_source: bool,
    freeze_hurts_extra_type: bool,
    damages_helmet: bool,
    helmet_present: bool,
) -> DamageTransform {
    let original = if amount < 0.0 { 0.0 } else { amount };
    let mut remaining = original - blocked_amount;
    if freezing_source && freeze_hurts_extra_type {
        remaining *= 5.0;
    }
    let damage_helmet = damages_helmet && helmet_present;
    let helmet_damage_input = if damage_helmet { remaining } else { 0.0 };
    if damage_helmet {
        remaining *= 0.75;
    }
    if !remaining.is_finite() {
        remaining = f32::MAX;
    }
    DamageTransform {
        original,
        blocked_amount,
        remaining,
        blocked: blocked_amount > 0.0,
        damage_helmet,
        helmet_damage_input,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CooldownDecision {
    Rejected,
    Accepted {
        selected_amount: f32,
        last_hurt: f32,
        fresh: bool,
        invulnerable_time: Option<u8>,
        hurt_time: Option<u8>,
        hurt_duration: Option<u8>,
    },
}

#[must_use]
pub fn select_cooldown_amount(
    remaining: f32,
    last_hurt: f32,
    invulnerable_time: u8,
    bypasses_cooldown: bool,
) -> CooldownDecision {
    if invulnerable_time > 10 && !bypasses_cooldown {
        if remaining <= last_hurt {
            CooldownDecision::Rejected
        } else {
            CooldownDecision::Accepted {
                selected_amount: remaining - last_hurt,
                last_hurt: remaining,
                fresh: false,
                invulnerable_time: None,
                hurt_time: None,
                hurt_duration: None,
            }
        }
    } else {
        CooldownDecision::Accepted {
            selected_amount: remaining,
            last_hurt: remaining,
            fresh: true,
            invulnerable_time: Some(20),
            hurt_time: Some(10),
            hurt_duration: Some(10),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribution {
    None,
    CausingPlayer { ticks: u8 },
    TameWolfOwner { ticks: u8 },
    Clear,
}

#[must_use]
pub const fn attribution(
    no_anger: bool,
    exempt_source: bool,
    causing_player: bool,
    tame_wolf: bool,
    wolf_owner_present: bool,
) -> Attribution {
    if no_anger || exempt_source {
        Attribution::None
    } else if causing_player {
        Attribution::CausingPlayer { ticks: 100 }
    } else if tame_wolf && wolf_owner_present {
        Attribution::TameWolfOwner { ticks: 100 }
    } else if tame_wolf {
        Attribution::Clear
    } else {
        Attribution::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcceptedHitPlan {
    pub call_on_blocked: bool,
    pub broadcast_damage_event: bool,
    pub mark_hurt: bool,
    pub call_knockback: bool,
    pub play_hurt_sounds: bool,
    pub store_last_source_and_time: bool,
    pub invoke_active_effects: bool,
    pub victim_criterion: bool,
    pub attacker_criterion: bool,
    pub shield_block_stat: Option<u32>,
    pub knockback_amount: f32,
    pub active_effect_amount: f32,
    pub meaningful: bool,
}

#[must_use]
pub fn accepted_hit_plan(input: AcceptedHitInput) -> AcceptedHitPlan {
    let meaningful = !input.blocked || input.remaining > 0.0;
    let blocked_stat = input
        .server_player_victim
        .then_some(input.blocked_amount)
        .filter(|amount| *amount > 0.0 && *amount < FLOAT_STAT_UPPER_BOUND)
        .map(|amount| (amount * 10.0).round() as u32);
    AcceptedHitPlan {
        call_on_blocked: input.fresh && input.blocked && input.snapshot_still_blocks,
        broadcast_damage_event: input.fresh && !(input.blocked && input.snapshot_still_blocks),
        mark_hurt: input.fresh && !input.no_impact && (!input.blocked || input.remaining > 0.0),
        call_knockback: input.fresh && !input.no_knockback,
        play_hurt_sounds: input.fresh && !input.dead_after_reduction,
        store_last_source_and_time: meaningful,
        invoke_active_effects: meaningful,
        victim_criterion: input.server_player_victim,
        attacker_criterion: input.causing_server_player,
        shield_block_stat: blocked_stat,
        knockback_amount: input.remaining,
        active_effect_amount: input.remaining,
        meaningful,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcceptedHitInput {
    pub fresh: bool,
    pub blocked: bool,
    pub blocked_amount: f32,
    pub remaining: f32,
    pub snapshot_still_blocks: bool,
    pub no_impact: bool,
    pub no_knockback: bool,
    pub dead_after_reduction: bool,
    pub server_player_victim: bool,
    pub causing_server_player: bool,
}

#[must_use]
pub const fn tick_hurt_timers(
    hurt_time: u8,
    invulnerable_time: u8,
    server_player: bool,
) -> (u8, u8) {
    (
        hurt_time.saturating_sub(1),
        if server_player {
            invulnerable_time
        } else {
            invulnerable_time.saturating_sub(1)
        },
    )
}
