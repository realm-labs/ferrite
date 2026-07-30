//! Monotonic deadline correction and one-iteration pacing.

use thiserror::Error;

pub const NANOS_PER_SECOND: i64 = 1_000_000_000;
pub const OVERLOAD_BASE_THRESHOLD_NANOS: i64 = NANOS_PER_SECOND;
pub const OVERLOAD_BASE_WARNING_INTERVAL_NANOS: i64 = 10 * NANOS_PER_SECOND;
pub const OVERLOAD_THRESHOLD_TICKS: i64 = 20;
pub const OVERLOAD_WARNING_TICKS: i64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeBudget {
    AlwaysFalse,
    RemainingUntilDeadline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IterationTiming {
    pub interval_nanos: i64,
    pub sprinting: bool,
    pub behind_nanos: i64,
    pub missed_intervals: i64,
    pub overload_warning: bool,
    pub next_tick_time_nanos: i64,
    pub last_overload_warning_nanos: i64,
    pub time_budget: TimeBudget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeadlineClock {
    next_tick_time_nanos: i64,
    last_overload_warning_nanos: i64,
}

impl DeadlineClock {
    pub const fn at(monotonic_now_nanos: i64) -> Self {
        Self {
            next_tick_time_nanos: monotonic_now_nanos,
            last_overload_warning_nanos: 0,
        }
    }

    pub const fn with_state(next_tick_time_nanos: i64, last_overload_warning_nanos: i64) -> Self {
        Self {
            next_tick_time_nanos,
            last_overload_warning_nanos,
        }
    }

    pub const fn next_tick_time_nanos(self) -> i64 {
        self.next_tick_time_nanos
    }

    pub fn plan_iteration(
        &mut self,
        monotonic_now_nanos: i64,
        ordinary_interval_nanos: i64,
        sprint_tick_admitted: bool,
    ) -> Result<IterationTiming, PacingError> {
        if sprint_tick_admitted {
            self.next_tick_time_nanos = monotonic_now_nanos;
            self.last_overload_warning_nanos = monotonic_now_nanos;
            return Ok(self.finish_plan(0, true, 0, 0, false));
        }
        if ordinary_interval_nanos <= 0 {
            return Err(PacingError::NonPositiveInterval(ordinary_interval_nanos));
        }

        let behind_nanos = monotonic_now_nanos.wrapping_sub(self.next_tick_time_nanos);
        let overload_threshold = OVERLOAD_BASE_THRESHOLD_NANOS
            .wrapping_add(OVERLOAD_THRESHOLD_TICKS.wrapping_mul(ordinary_interval_nanos));
        let warning_spacing = OVERLOAD_BASE_WARNING_INTERVAL_NANOS
            .wrapping_add(OVERLOAD_WARNING_TICKS.wrapping_mul(ordinary_interval_nanos));
        let warning_elapsed = self
            .next_tick_time_nanos
            .wrapping_sub(self.last_overload_warning_nanos);
        let overload_warning =
            behind_nanos > overload_threshold && warning_elapsed >= warning_spacing;
        let missed_intervals = if overload_warning {
            behind_nanos / ordinary_interval_nanos
        } else {
            0
        };
        if overload_warning {
            self.next_tick_time_nanos = self
                .next_tick_time_nanos
                .wrapping_add(missed_intervals.wrapping_mul(ordinary_interval_nanos));
            self.last_overload_warning_nanos = self.next_tick_time_nanos;
        }
        Ok(self.finish_plan(
            ordinary_interval_nanos,
            false,
            behind_nanos,
            missed_intervals,
            overload_warning,
        ))
    }

    pub fn delayed_tasks_deadline(
        self,
        monotonic_after_tick_nanos: i64,
        interval_nanos: i64,
    ) -> i64 {
        monotonic_after_tick_nanos
            .wrapping_add(interval_nanos)
            .max(self.next_tick_time_nanos)
    }

    fn finish_plan(
        &mut self,
        interval_nanos: i64,
        sprinting: bool,
        behind_nanos: i64,
        missed_intervals: i64,
        overload_warning: bool,
    ) -> IterationTiming {
        self.next_tick_time_nanos = self.next_tick_time_nanos.wrapping_add(interval_nanos);
        IterationTiming {
            interval_nanos,
            sprinting,
            behind_nanos,
            missed_intervals,
            overload_warning,
            next_tick_time_nanos: self.next_tick_time_nanos,
            last_overload_warning_nanos: self.last_overload_warning_nanos,
            time_budget: if sprinting {
                TimeBudget::AlwaysFalse
            } else {
                TimeBudget::RemainingUntilDeadline
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PacingError {
    #[error("ordinary server tick interval must be positive, got {0}")]
    NonPositiveInterval(i64),
}
