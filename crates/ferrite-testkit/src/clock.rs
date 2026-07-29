//! Explicit fake time for deterministic tests.

use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonotonicInstant(u64);

impl MonotonicInstant {
    pub const ZERO: Self = Self(0);

    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    pub const fn as_nanos(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeClock {
    now: MonotonicInstant,
}

impl Default for FakeClock {
    fn default() -> Self {
        Self {
            now: MonotonicInstant::ZERO,
        }
    }
}

impl FakeClock {
    pub const fn at(now: MonotonicInstant) -> Self {
        Self { now }
    }

    pub const fn now(&self) -> MonotonicInstant {
        self.now
    }

    pub fn advance(&mut self, duration: Duration) -> Result<MonotonicInstant, ClockError> {
        let nanos = u64::try_from(duration.as_nanos()).map_err(|_| ClockError::Overflow)?;
        let next = self
            .now
            .as_nanos()
            .checked_add(nanos)
            .ok_or(ClockError::Overflow)?;
        self.now = MonotonicInstant::from_nanos(next);
        Ok(self.now)
    }

    pub fn advance_to(&mut self, instant: MonotonicInstant) -> Result<(), ClockError> {
        if instant < self.now {
            return Err(ClockError::WentBackwards {
                current: self.now,
                requested: instant,
            });
        }
        self.now = instant;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ClockError {
    #[error("fake clock duration overflowed its u64 nanosecond representation")]
    Overflow,
    #[error("fake clock cannot move backwards from {current:?} to {requested:?}")]
    WentBackwards {
        current: MonotonicInstant,
        requested: MonotonicInstant,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_time_is_explicit_monotonic_and_checked() {
        let mut clock = FakeClock::default();
        assert_eq!(
            clock.advance(Duration::from_millis(2)).unwrap().as_nanos(),
            2_000_000
        );
        assert!(clock.advance_to(MonotonicInstant::from_nanos(1)).is_err());
        assert_eq!(clock.now().as_nanos(), 2_000_000);
    }
}
