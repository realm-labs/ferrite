//! World-border command tick conversion, bounds, and set/add dispatch.

use crate::generation::border::state::{BorderMutation, WorldBorder};

pub const MIN_COMMAND_SIZE: f64 = 1.0;
pub const MAX_COMMAND_SIZE: f64 = 59_999_968.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeSuffix {
    None,
    Tick,
    Second,
    Day,
}

impl TimeSuffix {
    const fn multiplier(self) -> f32 {
        match self {
            Self::None | Self::Tick => 1.0,
            Self::Second => 20.0,
            Self::Day => 24_000.0,
        }
    }
}

pub fn command_ticks(value: f32, suffix: TimeSuffix) -> i64 {
    java_round_f32(value * suffix.multiplier())
}

fn java_round_f32(value: f32) -> i64 {
    if value.is_nan() {
        0
    } else {
        i64::from((value + 0.5).floor() as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderCommandError {
    TargetOutOfRange(f64),
}

pub fn set_size_command(
    border: &mut WorldBorder,
    target: f64,
    duration_ticks: i64,
    begin_game_time: i64,
) -> Result<BorderMutation, BorderCommandError> {
    validate_target(target)?;
    if duration_ticks == 0 {
        Ok(border.set_size(target))
    } else {
        Ok(border.lerp_size_between(border.get_size(), target, duration_ticks, begin_game_time))
    }
}

pub fn add_size_command(
    border: &mut WorldBorder,
    delta: f64,
    extra_ticks: i64,
    begin_game_time: i64,
) -> Result<BorderMutation, BorderCommandError> {
    let current = border.get_size();
    let target = current + delta;
    validate_target(target)?;
    let duration = border.remaining_ticks().wrapping_add(extra_ticks);
    if duration == 0 {
        Ok(border.set_size(target))
    } else {
        Ok(border.lerp_size_between(current, target, duration, begin_game_time))
    }
}

fn validate_target(target: f64) -> Result<(), BorderCommandError> {
    if (MIN_COMMAND_SIZE..=MAX_COMMAND_SIZE).contains(&target) {
        Ok(())
    } else {
        Err(BorderCommandError::TargetOutOfRange(target))
    }
}
