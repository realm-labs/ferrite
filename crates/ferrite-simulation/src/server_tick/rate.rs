//! Tick-rate, freeze, step, sprint, autosave, and tick-time arithmetic.

use thiserror::Error;

use crate::server_tick::pacing::NANOS_PER_SECOND;

pub const DEFAULT_TICK_RATE: f32 = 20.0;
pub const MIN_TICK_RATE: f32 = 1.0;
pub const MAX_COMMAND_TICK_RATE: f32 = 10_000.0;
pub const DEFAULT_NANOS_PER_TICK: i64 = 50_000_000;
pub const INITIAL_AUTOSAVE_TICKS: i32 = 6_000;
pub const MIN_AUTOSAVE_TICKS: i32 = 100;
pub const AUTOSAVE_SECONDS: f32 = 300.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickRateState {
    tick_rate: f32,
    nanoseconds_per_tick: i64,
    frozen_ticks_to_run: i32,
    run_game_elements: bool,
    frozen: bool,
}

impl Default for TickRateState {
    fn default() -> Self {
        Self {
            tick_rate: DEFAULT_TICK_RATE,
            nanoseconds_per_tick: DEFAULT_NANOS_PER_TICK,
            frozen_ticks_to_run: 0,
            run_game_elements: true,
            frozen: false,
        }
    }
}

impl TickRateState {
    pub const fn tick_rate(self) -> f32 {
        self.tick_rate
    }

    pub const fn nanoseconds_per_tick(self) -> i64 {
        self.nanoseconds_per_tick
    }

    pub fn milliseconds_per_tick(self) -> f32 {
        self.nanoseconds_per_tick as f32 / 1_000_000.0_f32
    }

    pub const fn frozen_ticks_to_run(self) -> i32 {
        self.frozen_ticks_to_run
    }

    pub const fn runs_normally(self) -> bool {
        self.run_game_elements
    }

    pub const fn is_frozen(self) -> bool {
        self.frozen
    }

    pub fn set_tick_rate(&mut self, rate: f32) {
        self.tick_rate = if rate.is_nan() {
            f32::NAN
        } else {
            rate.max(MIN_TICK_RATE)
        };
        self.nanoseconds_per_tick = (NANOS_PER_SECOND as f64 / f64::from(self.tick_rate)) as i64;
    }

    pub fn set_command_tick_rate(&mut self, rate: f32) -> Result<(), TickRateError> {
        if !(MIN_TICK_RATE..=MAX_COMMAND_TICK_RATE).contains(&rate) {
            return Err(TickRateError::CommandRateOutOfRange);
        }
        self.set_tick_rate(rate);
        Ok(())
    }

    pub const fn set_frozen(&mut self, frozen: bool) {
        self.frozen = frozen;
    }

    pub const fn set_frozen_ticks_to_run(&mut self, ticks: i32) {
        self.frozen_ticks_to_run = ticks;
    }

    pub fn tick(&mut self) -> TickAdmission {
        self.run_game_elements = !self.frozen || self.frozen_ticks_to_run > 0;
        let consumed_step = self.frozen_ticks_to_run > 0;
        if consumed_step {
            self.frozen_ticks_to_run -= 1;
        }
        TickAdmission {
            run_game_elements: self.run_game_elements,
            consumed_step,
            remaining_steps: self.frozen_ticks_to_run,
        }
    }

    pub const fn entity_is_frozen(self, is_player: bool, player_passengers: u32) -> bool {
        !self.run_game_elements && !is_player && player_passengers == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickAdmission {
    pub run_game_elements: bool,
    pub consumed_step: bool,
    pub remaining_steps: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SprintReport {
    pub completed_ticks: i64,
    pub elapsed_milliseconds: f64,
    pub ticks_per_second: i32,
    pub milliseconds_per_tick: f64,
    pub restored_frozen: bool,
    pub recompute_autosave_interval: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreezeCommandResult {
    pub sprint_report: Option<SprintReport>,
    pub stepping_stopped: bool,
    pub frozen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SprintCheck {
    NotScheduled,
    WaitingForRunElements,
    AdmitSprintTick { remaining_ticks: i64 },
    Finished(SprintReport),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ServerTickRateState {
    rate: TickRateState,
    remaining_sprint_ticks: i64,
    sprint_tick_start_time: i64,
    sprint_time_spent: i64,
    scheduled_sprint_ticks: i64,
    previous_frozen: bool,
}

impl ServerTickRateState {
    pub const fn rate(self) -> TickRateState {
        self.rate
    }

    pub const fn rate_mut(&mut self) -> &mut TickRateState {
        &mut self.rate
    }

    pub const fn is_sprinting(self) -> bool {
        self.scheduled_sprint_ticks > 0
    }

    pub const fn remaining_sprint_ticks(self) -> i64 {
        self.remaining_sprint_ticks
    }

    pub fn step_game_if_paused(&mut self, ticks: i32) -> bool {
        if !self.rate.is_frozen() {
            return false;
        }
        self.rate.set_frozen_ticks_to_run(ticks);
        true
    }

    pub fn stop_stepping(&mut self) -> bool {
        if self.rate.frozen_ticks_to_run() <= 0 {
            return false;
        }
        self.rate.set_frozen_ticks_to_run(0);
        true
    }

    pub fn request_sprint(&mut self, ticks: i32) -> bool {
        let interrupted = self.remaining_sprint_ticks > 0;
        let ticks = i64::from(ticks);
        self.sprint_time_spent = 0;
        self.scheduled_sprint_ticks = ticks;
        self.remaining_sprint_ticks = ticks;
        self.previous_frozen = self.rate.is_frozen();
        self.rate.set_frozen(false);
        interrupted
    }

    pub fn check_should_sprint_this_tick(&mut self, monotonic_now_nanos: i64) -> SprintCheck {
        if !self.is_sprinting() {
            return SprintCheck::NotScheduled;
        }
        if !self.rate.runs_normally() {
            return SprintCheck::WaitingForRunElements;
        }
        if self.remaining_sprint_ticks > 0 {
            self.sprint_tick_start_time = monotonic_now_nanos;
            self.remaining_sprint_ticks -= 1;
            return SprintCheck::AdmitSprintTick {
                remaining_ticks: self.remaining_sprint_ticks,
            };
        }
        SprintCheck::Finished(self.finish_sprint())
    }

    pub fn end_sprint_tick_work(&mut self, monotonic_now_nanos: i64) {
        self.sprint_time_spent = self
            .sprint_time_spent
            .wrapping_add(monotonic_now_nanos.wrapping_sub(self.sprint_tick_start_time));
    }

    pub fn stop_sprinting(&mut self) -> Option<SprintReport> {
        (self.remaining_sprint_ticks > 0).then(|| self.finish_sprint())
    }

    pub fn apply_freeze_command(&mut self, frozen: bool) -> FreezeCommandResult {
        let (sprint_report, stepping_stopped) = if frozen {
            (self.stop_sprinting(), self.stop_stepping())
        } else {
            (None, false)
        };
        self.rate.set_frozen(frozen);
        FreezeCommandResult {
            sprint_report,
            stepping_stopped,
            frozen,
        }
    }

    fn finish_sprint(&mut self) -> SprintReport {
        let completed_ticks = self
            .scheduled_sprint_ticks
            .wrapping_sub(self.remaining_sprint_ticks);
        let elapsed_milliseconds = (self.sprint_time_spent as f64).max(1.0) / 1_000_000.0;
        let report_milliseconds = 1_000_i64.wrapping_mul(completed_ticks);
        let ticks_per_second = (report_milliseconds as f64 / elapsed_milliseconds) as i32;
        let milliseconds_per_tick = if completed_ticks == 0 {
            f64::from(self.rate.milliseconds_per_tick())
        } else {
            elapsed_milliseconds / completed_ticks as f64
        };
        self.scheduled_sprint_ticks = 0;
        self.sprint_time_spent = 0;
        self.remaining_sprint_ticks = 0;
        self.rate.set_frozen(self.previous_frozen);
        SprintReport {
            completed_ticks,
            elapsed_milliseconds,
            ticks_per_second,
            milliseconds_per_tick,
            restored_frozen: self.previous_frozen,
            recompute_autosave_interval: true,
        }
    }
}

pub fn compute_next_autosave_interval(
    tick_rate: f32,
    sprinting: bool,
    average_tick_time_nanos: i64,
) -> i32 {
    let ticks_per_second = if sprinting {
        let estimated = average_tick_time_nanos.wrapping_add(1);
        NANOS_PER_SECOND as f32 / estimated as f32
    } else {
        tick_rate
    };
    ((ticks_per_second * AUTOSAVE_SECONDS) as i32).max(MIN_AUTOSAVE_TICKS)
}

pub const fn apply_changed_autosave_interval(
    current_ticks_until_autosave: i32,
    newly_computed_interval: i32,
) -> i32 {
    if newly_computed_interval < current_ticks_until_autosave {
        newly_computed_interval
    } else {
        current_ticks_until_autosave
    }
}

pub fn smooth_tick_time(old_millis: f32, elapsed_nanos: i64) -> f32 {
    old_millis * 0.8_f32 + elapsed_nanos as f32 / 1_000_000.0_f32 * 0.19999999_f32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TickRateError {
    #[error("command tick rate must be between 1 and 10000 inclusive")]
    CommandRateOutOfRange,
}
