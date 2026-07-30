//! Generic revenge, classic neutral, and Piglin universal-anger models.

pub const UNIVERSAL_ANGER_DEFAULT: bool = false;
pub const GUARDED_CONTAINER_RADIUS: f64 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevengeAdmission {
    NoNewHurt,
    NoAttacker,
    SuppressedUniversalPlayer,
    CheckIgnoreClassesAndCombatTargeting,
}

#[must_use]
pub const fn revenge_admission(
    last_hurt_timestamp: i32,
    goal_timestamp: i32,
    attacker_present: bool,
    attacker_exact_player_type: bool,
    universal_anger: bool,
) -> RevengeAdmission {
    if last_hurt_timestamp == goal_timestamp {
        RevengeAdmission::NoNewHurt
    } else if !attacker_present {
        RevengeAdmission::NoAttacker
    } else if attacker_exact_player_type && universal_anger {
        RevengeAdmission::SuppressedUniversalPlayer
    } else {
        RevengeAdmission::CheckIgnoreClassesAndCombatTargeting
    }
}

#[must_use]
pub const fn suppressed_event_remains_unconsumed(admission: RevengeAdmission) -> bool {
    matches!(admission, RevengeAdmission::SuppressedUniversalPlayer)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeutralAngerInput {
    pub can_attack: bool,
    pub candidate_player: bool,
    pub creative_or_spectator: bool,
    pub peaceful: bool,
    pub universal_anger: bool,
    pub anger_end_time: i64,
    pub game_time: i64,
    pub persistent_target_present: bool,
    pub persistent_target_matches: bool,
}

#[must_use]
pub const fn neutral_is_angry_at(input: NeutralAngerInput) -> bool {
    if !input.can_attack {
        return false;
    }
    let universal = input.candidate_player
        && !input.creative_or_spectator
        && !input.peaceful
        && input.universal_anger
        && input.anger_end_time > input.game_time
        && !input.persistent_target_present;
    universal || (input.persistent_target_present && input.persistent_target_matches)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResetGoalAdmission {
    pub admitted: bool,
    pub consume_hurt_event_on_start_only: bool,
}

#[must_use]
pub const fn reset_goal_admission(
    universal_anger: bool,
    last_attacker_exact_player: bool,
    last_hurt_timestamp: i32,
    goal_timestamp: i32,
) -> ResetGoalAdmission {
    ResetGoalAdmission {
        admitted: universal_anger
            && last_attacker_exact_player
            && last_hurt_timestamp > goal_timestamp,
        consume_hurt_event_on_start_only: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResetCommit {
    pub clear_last_attacker: bool,
    pub clear_persistent_target: bool,
    pub clear_live_target: bool,
    pub intermediate_end_time: i64,
    pub new_targetless_duration: u16,
}

#[must_use]
pub const fn reset_commit(timer_draw: u32) -> ResetCommit {
    ResetCommit {
        clear_last_attacker: true,
        clear_persistent_target: true,
        clear_live_target: true,
        intermediate_end_time: -1,
        new_targetless_duration: 400 + (timer_draw % 381) as u16,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeutralKind {
    Bee,
    IronGolem,
    PolarBear,
    Wolf,
    Enderman,
    ZombifiedPiglin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResetRegistration {
    pub priority: u8,
    pub group_alert: bool,
}

#[must_use]
pub const fn reset_registration(kind: NeutralKind) -> ResetRegistration {
    match kind {
        NeutralKind::Bee => ResetRegistration {
            priority: 3,
            group_alert: true,
        },
        NeutralKind::IronGolem => ResetRegistration {
            priority: 4,
            group_alert: false,
        },
        NeutralKind::PolarBear => ResetRegistration {
            priority: 5,
            group_alert: false,
        },
        NeutralKind::Wolf => ResetRegistration {
            priority: 8,
            group_alert: true,
        },
        NeutralKind::Enderman => ResetRegistration {
            priority: 4,
            group_alert: false,
        },
        NeutralKind::ZombifiedPiglin => ResetRegistration {
            priority: 3,
            group_alert: true,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupAlertQuery {
    pub horizontal_inflation: f32,
    pub vertical_inflation: f32,
    pub exclude_spectators: bool,
    pub exclude_only_starter_after_query: bool,
    pub reset_every_returned_peer_in_order: bool,
}

#[must_use]
pub const fn group_alert_query(follow_range: f32) -> GroupAlertQuery {
    GroupAlertQuery {
        horizontal_inflation: follow_range,
        vertical_inflation: 10.0,
        exclude_spectators: true,
        exclude_only_starter_after_query: true,
        reset_every_returned_peer_in_order: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiglinTarget {
    TriggeringPlayer,
    NearestVisibleAttackablePlayer,
    ExactAttacker,
}

#[must_use]
pub const fn guarded_container_target(
    universal_anger: bool,
    nearest_visible_attackable_player_present: bool,
) -> PiglinTarget {
    if universal_anger && nearest_visible_attackable_player_present {
        PiglinTarget::NearestVisibleAttackablePlayer
    } else {
        PiglinTarget::TriggeringPlayer
    }
}

#[must_use]
pub const fn guarded_piglin_admitted(
    brain_idle: bool,
    require_visibility: bool,
    triggering_player_visible: bool,
) -> bool {
    brain_idle && (!require_visibility || triggering_player_visible)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiglinAngerWrite {
    pub erase_cant_reach_since: bool,
    pub angry_at_ttl: u16,
    pub write_universal_anger: bool,
    pub universal_anger_ttl: u16,
}

#[must_use]
pub const fn piglin_anger_write(
    selected_attackable_ignoring_sight: bool,
    selected_is_player: bool,
    live_universal_anger: bool,
) -> Option<PiglinAngerWrite> {
    if !selected_attackable_ignoring_sight {
        return None;
    }
    Some(PiglinAngerWrite {
        erase_cant_reach_since: true,
        angry_at_ttl: 600,
        write_universal_anger: selected_is_player && live_universal_anger,
        universal_anger_ttl: 600,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retaliation {
    RejectAvoid,
    RejectUnattackable,
    RejectMuchFartherThanCurrent,
    UniversalPlayer,
    ExactAttacker,
}

#[must_use]
pub const fn retaliation(
    avoid_active: bool,
    attacker_attackable_ignoring_sight: bool,
    much_farther_than_current_by_four: bool,
    attacker_is_player: bool,
    universal_anger: bool,
) -> Retaliation {
    if avoid_active {
        Retaliation::RejectAvoid
    } else if !attacker_attackable_ignoring_sight {
        Retaliation::RejectUnattackable
    } else if much_farther_than_current_by_four {
        Retaliation::RejectMuchFartherThanCurrent
    } else if attacker_is_player && universal_anger {
        Retaliation::UniversalPlayer
    } else {
        Retaliation::ExactAttacker
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedAttackTarget {
    NoneNearZombified,
    AngryAt,
    UniversalNearestPlayer,
    NearestNemesis,
    NearestNonGoldPlayer,
    None,
}

#[must_use]
pub const fn resolve_attack_target(input: ResolveTargetInput) -> ResolvedAttackTarget {
    if input.near_zombified {
        ResolvedAttackTarget::NoneNearZombified
    } else if input.angry_at_resolves_attackable {
        ResolvedAttackTarget::AngryAt
    } else if input.universal_memory_present && input.nearest_visible_attackable_player_present {
        ResolvedAttackTarget::UniversalNearestPlayer
    } else if input.nearest_nemesis_present {
        ResolvedAttackTarget::NearestNemesis
    } else if input.nearest_non_gold_player_attackable {
        ResolvedAttackTarget::NearestNonGoldPlayer
    } else {
        ResolvedAttackTarget::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolveTargetInput {
    pub near_zombified: bool,
    pub angry_at_resolves_attackable: bool,
    pub universal_memory_present: bool,
    pub nearest_visible_attackable_player_present: bool,
    pub nearest_nemesis_present: bool,
    pub nearest_non_gold_player_attackable: bool,
}
