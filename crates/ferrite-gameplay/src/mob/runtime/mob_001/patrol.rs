//! Patrol custom-spawner timer, admission, group walk, and member transaction.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrolTick {
    pub next_tick: i32,
    pub timer_changed: bool,
    pub schedule_draw_consumed: bool,
    pub attempt_due: bool,
}

#[must_use]
pub const fn patrol_tick(
    next_tick: i32,
    cached_hostile_policy: bool,
    spawn_patrols: bool,
    schedule_draw_below_1200: u32,
) -> PatrolTick {
    if !cached_hostile_policy || !spawn_patrols {
        return PatrolTick {
            next_tick,
            timer_changed: false,
            schedule_draw_consumed: false,
            attempt_due: false,
        };
    }
    let decremented = next_tick.saturating_sub(1);
    if decremented > 0 {
        PatrolTick {
            next_tick: decremented,
            timer_changed: true,
            schedule_draw_consumed: false,
            attempt_due: false,
        }
    } else {
        PatrolTick {
            next_tick: decremented
                .saturating_add(12_000 + (schedule_draw_below_1200 % 1_200) as i32),
            timer_changed: true,
            schedule_draw_consumed: true,
            attempt_due: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatrolAttemptFailure {
    OutsideDark,
    Chance,
    NoPlayers,
    SelectedSpectator,
    NearVillage,
    InitialSquareUnloaded,
    EnvironmentAttribute,
}

pub const fn patrol_attempt(input: PatrolAttemptInput) -> Result<(), PatrolAttemptFailure> {
    if !input.bright_outside {
        Err(PatrolAttemptFailure::OutsideDark)
    } else if input.chance_draw_below_five != 0 {
        Err(PatrolAttemptFailure::Chance)
    } else if input.player_count == 0 {
        Err(PatrolAttemptFailure::NoPlayers)
    } else if input.selected_spectator {
        Err(PatrolAttemptFailure::SelectedSpectator)
    } else if input.close_to_village {
        Err(PatrolAttemptFailure::NearVillage)
    } else if !input.initial_square_loaded {
        Err(PatrolAttemptFailure::InitialSquareUnloaded)
    } else if !input.environment_attribute {
        Err(PatrolAttemptFailure::EnvironmentAttribute)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrolAttemptInput {
    pub bright_outside: bool,
    pub chance_draw_below_five: u8,
    pub player_count: usize,
    pub selected_spectator: bool,
    pub close_to_village: bool,
    pub initial_square_loaded: bool,
    pub environment_attribute: bool,
}

#[must_use]
pub const fn selected_player_index(draw: usize, player_count: usize) -> Option<usize> {
    if player_count == 0 {
        None
    } else {
        Some(draw % player_count)
    }
}

#[must_use]
pub const fn player_offset(distance_draw: u32, sign_draw: bool) -> i32 {
    let magnitude = 24 + (distance_draw % 24) as i32;
    if sign_draw { magnitude } else { -magnitude }
}

#[must_use]
pub fn group_count(effective_difficulty: f32) -> u32 {
    effective_difficulty.ceil() as u32 + 1
}

#[must_use]
pub const fn member_walk(first_draw: u32, second_draw: u32) -> i32 {
    (first_draw % 5) as i32 - (second_draw % 5) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatrolMemberFailure {
    InvalidEmptyBlock,
    BlockLight,
    InvalidSupport,
    NullConstruction,
}

pub const fn patrol_member(
    valid_empty_block: bool,
    block_light: u8,
    valid_support: bool,
    construction_succeeded: bool,
) -> Result<(), PatrolMemberFailure> {
    if !valid_empty_block {
        Err(PatrolMemberFailure::InvalidEmptyBlock)
    } else if block_light > 8 {
        Err(PatrolMemberFailure::BlockLight)
    } else if !valid_support {
        Err(PatrolMemberFailure::InvalidSupport)
    } else if !construction_succeeded {
        Err(PatrolMemberFailure::NullConstruction)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrolLeader {
    pub mark_patrolling_before_target: bool,
    pub target_draws_from_entity_rng: u8,
    pub target_before_placement: bool,
    pub equip_ominous_banner: bool,
    pub banner_drop_chance: u8,
}

pub const PATROL_LEADER: PatrolLeader = PatrolLeader {
    mark_patrolling_before_target: true,
    target_draws_from_entity_rng: 2,
    target_before_placement: true,
    equip_ominous_banner: true,
    banner_drop_chance: 2,
};

#[must_use]
pub const fn leader_target_offset(draw_below_thousand: u32) -> i32 {
    -500 + (draw_below_thousand % 1_000) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrolMemberCommit {
    pub finalize_with_patrol_reason: bool,
    pub equip_crossbow_and_enchant: bool,
    pub mark_patrolling: bool,
    pub insert_with_passengers: bool,
    pub observe_insertion_result: bool,
}

pub const PATROL_MEMBER_COMMIT: PatrolMemberCommit = PatrolMemberCommit {
    finalize_with_patrol_reason: true,
    equip_crossbow_and_enchant: true,
    mark_patrolling: true,
    insert_with_passengers: true,
    observe_insertion_result: false,
};

#[must_use]
pub const fn continue_group(member_index: u32, member_succeeded: bool) -> bool {
    member_index != 0 || member_succeeded
}
