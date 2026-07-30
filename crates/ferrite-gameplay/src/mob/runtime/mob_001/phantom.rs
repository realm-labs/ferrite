//! Phantom custom-spawner cadence and ordered per-player insomnia trials.

use crate::mob::runtime::mob_001::hostile::Difficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhantomTick {
    pub next_tick: i32,
    pub timer_changed: bool,
    pub schedule_draw_consumed: bool,
    pub attempt_due: bool,
}

#[must_use]
pub const fn phantom_tick(
    next_tick: i32,
    cached_hostile_policy: bool,
    spawn_phantoms: bool,
    schedule_draw_below_sixty: u32,
) -> PhantomTick {
    if !cached_hostile_policy || !spawn_phantoms {
        return PhantomTick {
            next_tick,
            timer_changed: false,
            schedule_draw_consumed: false,
            attempt_due: false,
        };
    }
    let decremented = next_tick.saturating_sub(1);
    if decremented > 0 {
        PhantomTick {
            next_tick: decremented,
            timer_changed: true,
            schedule_draw_consumed: false,
            attempt_due: false,
        }
    } else {
        PhantomTick {
            next_tick: decremented
                .saturating_add((60 + (schedule_draw_below_sixty % 60) as i32) * 20),
            timer_changed: true,
            schedule_draw_consumed: true,
            attempt_due: true,
        }
    }
}

#[must_use]
pub const fn level_sky_allows(has_skylight: bool, sky_darken: u8) -> bool {
    !has_skylight || sky_darken >= 5
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSkyGate {
    pub eligible: bool,
    pub consume_difficulty_draw: bool,
}

#[must_use]
pub const fn player_sky_gate(
    spectator: bool,
    has_skylight: bool,
    player_y: i32,
    sea_level: i32,
    can_see_sky: bool,
) -> PlayerSkyGate {
    let eligible = !spectator && (!has_skylight || (player_y >= sea_level && can_see_sky));
    PlayerSkyGate {
        eligible,
        consume_difficulty_draw: eligible,
    }
}

#[must_use]
pub fn difficulty_trial(effective_difficulty: f32, draw: f32) -> bool {
    effective_difficulty > draw * 3.0
}

#[must_use]
pub const fn clamped_rest(time_since_rest: i32) -> u32 {
    if time_since_rest < 1 {
        1
    } else {
        time_since_rest as u32
    }
}

#[must_use]
pub const fn insomnia_trial(time_since_rest: i32, draw: u32) -> bool {
    let rest = clamped_rest(time_since_rest);
    draw % rest >= 72_000
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhantomCandidate {
    pub x_offset: i32,
    pub y_offset: i32,
    pub z_offset: i32,
}

#[must_use]
pub const fn phantom_candidate(y_draw: u32, x_draw: u32, z_draw: u32) -> PhantomCandidate {
    PhantomCandidate {
        x_offset: -10 + (x_draw % 21) as i32,
        y_offset: 20 + (y_draw % 15) as i32,
        z_offset: -10 + (z_draw % 21) as i32,
    }
}

#[must_use]
pub const fn phantom_group_count(difficulty: Difficulty, draw: u32) -> u8 {
    1 + (draw % (difficulty.id() + 1)) as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhantomMember {
    pub skip_only_member_on_null: bool,
    pub snap_exact_block_position: bool,
    pub yaw_and_pitch_zero: bool,
    pub anchor_y_offset: u8,
    pub size: u8,
    pub finalize_with_captured_difficulty: bool,
    pub insertion_result_ignored: bool,
}

pub const PHANTOM_MEMBER: PhantomMember = PhantomMember {
    skip_only_member_on_null: true,
    snap_exact_block_position: true,
    yaw_and_pitch_zero: true,
    anchor_y_offset: 5,
    size: 0,
    finalize_with_captured_difficulty: true,
    insertion_result_ignored: true,
};
