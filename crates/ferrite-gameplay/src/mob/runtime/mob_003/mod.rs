//! MOB-003 despawn admission, cadence, and subtype policies.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DespawnCategory {
    WaterAmbient,
    Other,
}

#[must_use]
pub const fn hard_distance_squared(category: DespawnCategory) -> f64 {
    match category {
        DespawnCategory::WaterAmbient => 64.0 * 64.0,
        DespawnCategory::Other => 128.0 * 128.0,
    }
}

pub const SOFT_DISTANCE_SQUARED: f64 = 32.0 * 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DespawnExit {
    PeacefulDiscard,
    PersistentReset,
    NoPlayer,
    DistanceEvaluation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DespawnOutcome {
    pub exit: DespawnExit,
    pub discard_peaceful: bool,
    pub discard_hard: bool,
    pub discard_soft: bool,
    pub reset_no_action_time: bool,
    pub random_draw_consumed: bool,
    pub remove_policy_calls: u8,
    pub continue_soft_after_hard_discard: bool,
}

#[must_use]
pub const fn check_despawn(input: DespawnInput) -> DespawnOutcome {
    if input.peaceful && !input.type_allowed_in_peaceful {
        return DespawnOutcome {
            exit: DespawnExit::PeacefulDiscard,
            discard_peaceful: true,
            discard_hard: false,
            discard_soft: false,
            reset_no_action_time: false,
            random_draw_consumed: false,
            remove_policy_calls: 0,
            continue_soft_after_hard_discard: false,
        };
    }
    if input.stored_persistence || input.custom_persistence {
        return DespawnOutcome {
            exit: DespawnExit::PersistentReset,
            discard_peaceful: false,
            discard_hard: false,
            discard_soft: false,
            reset_no_action_time: true,
            random_draw_consumed: false,
            remove_policy_calls: 0,
            continue_soft_after_hard_discard: false,
        };
    }
    if !input.nearest_nonspectator_player_present {
        return DespawnOutcome {
            exit: DespawnExit::NoPlayer,
            discard_peaceful: false,
            discard_hard: false,
            discard_soft: false,
            reset_no_action_time: false,
            random_draw_consumed: false,
            remove_policy_calls: 0,
            continue_soft_after_hard_discard: false,
        };
    }

    let beyond_hard = input.distance_squared > hard_distance_squared(input.category);
    let hard_policy_call = beyond_hard;
    let discard_hard = beyond_hard && input.remove_when_far_away;
    let random_draw_consumed = input.no_action_time > 600;
    let soft_distance_reached = random_draw_consumed
        && input.random_draw_below_eight_hundred == 0
        && input.distance_squared > SOFT_DISTANCE_SQUARED;
    let discard_soft = soft_distance_reached && input.remove_when_far_away;
    DespawnOutcome {
        exit: DespawnExit::DistanceEvaluation,
        discard_peaceful: false,
        discard_hard,
        discard_soft,
        reset_no_action_time: !discard_soft && input.distance_squared < SOFT_DISTANCE_SQUARED,
        random_draw_consumed,
        remove_policy_calls: (if hard_policy_call { 1 } else { 0 })
            + (if soft_distance_reached { 1 } else { 0 }),
        continue_soft_after_hard_discard: discard_hard,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DespawnInput {
    pub peaceful: bool,
    pub type_allowed_in_peaceful: bool,
    pub stored_persistence: bool,
    pub custom_persistence: bool,
    pub nearest_nonspectator_player_present: bool,
    pub category: DespawnCategory,
    pub distance_squared: f64,
    pub no_action_time: u32,
    pub random_draw_below_eight_hundred: u16,
    pub remove_when_far_away: bool,
}

#[must_use]
pub const fn no_action_time_after_ai_step(no_action_time: u32, effective_ai: bool) -> u32 {
    if effective_ai {
        no_action_time.saturating_add(1)
    } else {
        no_action_time
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomPersistence {
    Base,
    Fish { from_bucket: bool },
    Nautilus { tamed: bool },
    SulfurCube { body_item: bool, from_bucket: bool },
    Enderman { carrying_block: bool },
    Raider { current_raid: bool },
}

#[must_use]
pub const fn requires_custom_persistence(
    passenger: bool,
    leashed: bool,
    policy: CustomPersistence,
) -> bool {
    let base = passenger || leashed;
    base || match policy {
        CustomPersistence::Base => false,
        CustomPersistence::Fish { from_bucket } => from_bucket,
        CustomPersistence::Nautilus { tamed } => tamed,
        CustomPersistence::SulfurCube {
            body_item,
            from_bucket,
        } => body_item || from_bucket,
        CustomPersistence::Enderman { carrying_block } => carrying_block,
        CustomPersistence::Raider { current_raid } => current_raid,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalPolicy {
    Base,
    Never,
    Chicken {
        chicken_jockey: bool,
    },
    Cat {
        tamed: bool,
        tick_count: u32,
    },
    Ocelot {
        trusting: bool,
        tick_count: u32,
    },
    Fish {
        from_bucket: bool,
        custom_named: bool,
    },
    Nautilus,
    Raider {
        current_raid: bool,
    },
    Patrolling {
        patrolling: bool,
    },
    Piglin {
        stored_persistence: bool,
    },
    Always,
    ZombieVillager {
        converting: bool,
        villager_xp: u32,
    },
}

#[must_use]
pub fn remove_when_far_away(policy: RemovalPolicy, distance_squared: f64) -> bool {
    match policy {
        RemovalPolicy::Base | RemovalPolicy::Nautilus | RemovalPolicy::Always => true,
        RemovalPolicy::Never => false,
        RemovalPolicy::Chicken { chicken_jockey } => chicken_jockey,
        RemovalPolicy::Cat { tamed, tick_count } => !tamed && tick_count > 2_400,
        RemovalPolicy::Ocelot {
            trusting,
            tick_count,
        } => !trusting && tick_count > 2_400,
        RemovalPolicy::Fish {
            from_bucket,
            custom_named,
        } => !from_bucket && !custom_named,
        RemovalPolicy::Raider { current_raid } => !current_raid,
        RemovalPolicy::Patrolling { patrolling } => !patrolling || distance_squared > 16_384.0,
        RemovalPolicy::Piglin { stored_persistence } => !stored_persistence,
        RemovalPolicy::ZombieVillager {
            converting,
            villager_xp,
        } => !converting && villager_xp == 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvocationPolicy {
    pub check_before_current_chunk_ticking_admission: bool,
    pub root_entities_only: bool,
    pub valid_passenger_checked_independently: bool,
    pub discard_without_death_loot_or_xp: bool,
}

pub const INVOCATION_POLICY: InvocationPolicy = InvocationPolicy {
    check_before_current_chunk_ticking_admission: true,
    root_entities_only: true,
    valid_passenger_checked_independently: false,
    discard_without_death_loot_or_xp: true,
};
