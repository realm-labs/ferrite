//! Wandering-trader two-level cadence, persisted chance, placement, and llama attachment.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraderSavedState {
    pub spawn_delay: i32,
    pub spawn_chance: i32,
}

impl Default for TraderSavedState {
    fn default() -> Self {
        Self {
            spawn_delay: 24_000,
            spawn_chance: 25,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraderTick {
    pub tick_delay: i32,
    pub load_saved_state: bool,
    pub timer_changed: bool,
}

#[must_use]
pub const fn trader_tick(tick_delay: i32, spawn_wandering_traders: bool) -> TraderTick {
    if !spawn_wandering_traders {
        return TraderTick {
            tick_delay,
            load_saved_state: false,
            timer_changed: false,
        };
    }
    let decremented = tick_delay.saturating_sub(1);
    if decremented > 0 {
        TraderTick {
            tick_delay: decremented,
            load_saved_state: false,
            timer_changed: true,
        }
    } else {
        TraderTick {
            tick_delay: 1_200,
            load_saved_state: true,
            timer_changed: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraderSavedTick {
    pub state: TraderSavedState,
    pub delay_changed: bool,
    pub chance_changed: bool,
    pub consume_chance_draw: bool,
    pub attempt_spawn: bool,
}

#[must_use]
pub const fn trader_saved_tick(
    state: TraderSavedState,
    chance_draw_below_hundred: i32,
) -> TraderSavedTick {
    let delay = state.spawn_delay.wrapping_sub(1_200);
    if delay > 0 {
        return TraderSavedTick {
            state: TraderSavedState {
                spawn_delay: delay,
                spawn_chance: state.spawn_chance,
            },
            delay_changed: true,
            chance_changed: false,
            consume_chance_draw: false,
            attempt_spawn: false,
        };
    }
    let increased = state.spawn_chance.wrapping_add(25);
    let next_chance = if increased < 25 {
        25
    } else if increased > 75 {
        75
    } else {
        increased
    };
    TraderSavedTick {
        state: TraderSavedState {
            spawn_delay: 24_000,
            spawn_chance: next_chance,
        },
        delay_changed: true,
        chance_changed: next_chance != state.spawn_chance,
        consume_chance_draw: true,
        attempt_spawn: chance_draw_below_hundred <= state.spawn_chance,
    }
}

#[must_use]
pub const fn chance_after_spawn(spawn_returned_true: bool, elevated_chance: i32) -> i32 {
    if spawn_returned_true {
        25
    } else {
        elevated_chance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraderPlayerSelection {
    pub spawn_returns_true: bool,
    pub reset_chance: bool,
    pub consume_one_in_ten_draw: bool,
    pub proceed: bool,
}

#[must_use]
pub const fn trader_player_selection(
    alive_player_count: usize,
    one_in_ten_draw: u8,
) -> TraderPlayerSelection {
    if alive_player_count == 0 {
        TraderPlayerSelection {
            spawn_returns_true: true,
            reset_chance: true,
            consume_one_in_ten_draw: false,
            proceed: false,
        }
    } else {
        let proceed = one_in_ten_draw.is_multiple_of(10);
        TraderPlayerSelection {
            spawn_returns_true: false,
            reset_chance: false,
            consume_one_in_ten_draw: true,
            proceed,
        }
    }
}

#[must_use]
pub const fn trader_player_index(draw: usize, alive_player_count: usize) -> Option<usize> {
    if alive_player_count == 0 {
        None
    } else {
        Some(draw % alive_player_count)
    }
}

#[must_use]
pub const fn encounter_first_meeting_or_player(
    first_meeting: Option<(i32, i32, i32)>,
    player: (i32, i32, i32),
) -> (i32, i32, i32) {
    match first_meeting {
        Some(meeting) => meeting,
        None => player,
    }
}

#[must_use]
pub const fn sampled_offset(radius: u32, draw: u32) -> i32 {
    if radius == 0 {
        0
    } else {
        (draw % radius.saturating_mul(2)) as i32 - radius as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraderCandidateFailure {
    WorldBorder,
    InvalidGround,
    CandidateOccupied,
    AboveOccupied,
}

pub const fn trader_candidate(
    inside_world_border: bool,
    valid_ground: bool,
    candidate_empty: bool,
    above_empty: bool,
) -> Result<(), TraderCandidateFailure> {
    if !inside_world_border {
        Err(TraderCandidateFailure::WorldBorder)
    } else if !valid_ground {
        Err(TraderCandidateFailure::InvalidGround)
    } else if !candidate_empty {
        Err(TraderCandidateFailure::CandidateOccupied)
    } else if !above_empty {
        Err(TraderCandidateFailure::AboveOccupied)
    } else {
        Ok(())
    }
}

#[must_use]
pub fn trader_space_empty(collision_cells_in_iteration_order: &[bool]) -> bool {
    collision_cells_in_iteration_order.len() == 12
        && collision_cells_in_iteration_order
            .iter()
            .all(|empty| *empty)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraderCommit {
    pub insertion_result_ignored: bool,
    pub play_ambient_sound: bool,
    pub llama_calls: u8,
    pub trader_despawn_delay: u32,
    pub set_wander_target_to_meeting: bool,
    pub set_home_to_meeting: bool,
    pub home_radius: u8,
}

pub const TRADER_COMMIT: TraderCommit = TraderCommit {
    insertion_result_ignored: true,
    play_ambient_sound: true,
    llama_calls: 2,
    trader_despawn_delay: 48_000,
    set_wander_target_to_meeting: true,
    set_home_to_meeting: true,
    home_radius: 16,
};

pub const TRADER_CANDIDATE_ATTEMPTS: u8 = 10;
pub const TRADER_SEARCH_RADIUS: u8 = 48;
pub const TRADER_COLLISION_CELLS: u8 = 12;

#[must_use]
pub const fn trader_biome_allows(without_wandering_trader_spawns: bool) -> bool {
    !without_wandering_trader_spawns
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LlamaCommit {
    pub search_attempts: u8,
    pub search_radius: u8,
    pub use_trader_heightmap_and_placement: bool,
    pub use_space_or_biome_check: bool,
    pub constructor_despawn_delay: u32,
    pub force_adult: bool,
    pub finalize_strength_then_variant: bool,
    pub leash_with_broadcast: bool,
    pub insertion_result_ignored: bool,
}

pub const LLAMA_COMMIT: LlamaCommit = LlamaCommit {
    search_attempts: 10,
    search_radius: 4,
    use_trader_heightmap_and_placement: true,
    use_space_or_biome_check: false,
    constructor_despawn_delay: 47_999,
    force_adult: true,
    finalize_strength_then_variant: true,
    leash_with_broadcast: true,
    insertion_result_ignored: true,
};
