//! Global named clocks, persistence, network state, and sleep/time operations.

use std::collections::BTreeMap;

use super::DimensionType;
use super::timeline::Timeline;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    total_ticks: i64,
    partial_tick: f32,
    rate: f32,
    paused: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            total_ticks: 0,
            partial_tick: 0.0,
            rate: 1.0,
            paused: false,
        }
    }
}

impl Clock {
    pub fn from_saved(state: SavedClock) -> Self {
        Self {
            total_ticks: state.total_ticks,
            partial_tick: state.partial_tick,
            rate: state.rate,
            paused: state.paused,
        }
    }

    pub fn saved(self) -> SavedClock {
        SavedClock {
            total_ticks: self.total_ticks,
            partial_tick: self.partial_tick,
            rate: self.rate,
            paused: self.paused,
        }
    }

    pub fn total_ticks(self) -> i64 {
        self.total_ticks
    }

    pub fn partial_tick(self) -> f32 {
        self.partial_tick
    }

    pub fn rate(self) -> f32 {
        self.rate
    }

    pub fn paused(self) -> bool {
        self.paused
    }

    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
        self.partial_tick += self.rate;
        let whole = self.partial_tick.floor() as i64;
        self.total_ticks = self.total_ticks.wrapping_add(whole);
        self.partial_tick -= whole as f32;
    }

    pub fn set_total_ticks(&mut self, ticks: i64) {
        self.total_ticks = ticks;
    }

    pub fn add_ticks(&mut self, ticks: i64) {
        self.total_ticks = self.total_ticks.wrapping_add(ticks);
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn network_state(self, advance_time: bool) -> NetworkClock {
        NetworkClock {
            total_ticks: self.total_ticks,
            partial_tick: self.partial_tick,
            rate: if self.paused || !advance_time {
                0.0
            } else {
                self.rate
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavedClock {
    pub total_ticks: i64,
    pub partial_tick: f32,
    pub rate: f32,
    pub paused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetworkClock {
    pub total_ticks: i64,
    pub partial_tick: f32,
    pub rate: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClockManager {
    clocks: BTreeMap<String, Clock>,
    mutation_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ClockError {
    #[error("clock {0} does not exist")]
    MissingClock(String),
    #[error("dimension has no default clock")]
    MissingDefaultClock,
    #[error("timeline marker {0} does not exist")]
    MissingMarker(String),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SleepCompletion {
    pub clock_advanced: bool,
    pub players_woken: bool,
    pub weather_reset: bool,
}

impl ClockManager {
    pub fn locked() -> Self {
        Self {
            clocks: [
                ("minecraft:overworld".to_owned(), Clock::default()),
                ("minecraft:the_end".to_owned(), Clock::default()),
            ]
            .into_iter()
            .collect(),
            mutation_generation: 0,
        }
    }

    pub fn clocks(&self) -> &BTreeMap<String, Clock> {
        &self.clocks
    }

    pub fn mutation_generation(&self) -> u64 {
        self.mutation_generation
    }

    pub fn tick(&mut self, advance_time: bool) {
        if advance_time {
            for clock in self.clocks.values_mut() {
                clock.tick();
            }
        }
    }

    pub fn network_state(&self, advance_time: bool) -> BTreeMap<String, NetworkClock> {
        self.clocks
            .iter()
            .map(|(key, clock)| (key.clone(), clock.network_state(advance_time)))
            .collect()
    }

    pub fn default_time(&self, dimension: &DimensionType) -> i64 {
        dimension
            .default_clock
            .as_ref()
            .and_then(|key| self.clocks.get(key))
            .map_or(0, |clock| clock.total_ticks())
    }

    pub fn explicit_clock(&self, clock: &str) -> Result<Clock, ClockError> {
        self.clocks
            .get(clock)
            .copied()
            .ok_or_else(|| ClockError::MissingClock(clock.to_owned()))
    }

    pub fn mutate_explicit(
        &mut self,
        clock: &str,
        mutation: impl FnOnce(&mut Clock),
    ) -> Result<u64, ClockError> {
        let value = self
            .clocks
            .get_mut(clock)
            .ok_or_else(|| ClockError::MissingClock(clock.to_owned()))?;
        mutation(value);
        self.mutation_generation = self.mutation_generation.wrapping_add(1);
        Ok(self.mutation_generation)
    }

    pub fn mutate_default(
        &mut self,
        dimension: &DimensionType,
        mutation: impl FnOnce(&mut Clock),
    ) -> Result<u64, ClockError> {
        let clock = dimension
            .default_clock
            .as_deref()
            .ok_or(ClockError::MissingDefaultClock)?;
        self.mutate_explicit(clock, mutation)
    }

    pub fn complete_sleep(
        &mut self,
        dimension: &DimensionType,
        default_timeline: Option<&Timeline>,
        enough_deep_sleepers: bool,
        advance_time: bool,
        advance_weather: bool,
        raining: bool,
    ) -> Result<SleepCompletion, ClockError> {
        if !enough_deep_sleepers {
            return Ok(SleepCompletion::default());
        }
        let mut completion = SleepCompletion {
            players_woken: true,
            weather_reset: advance_weather && raining,
            ..SleepCompletion::default()
        };
        if advance_time
            && let (Some(clock_key), Some(timeline)) =
                (dimension.default_clock.as_deref(), default_timeline)
        {
            let wake = timeline
                .marker("minecraft:wake_up_from_sleep")
                .ok_or_else(|| {
                    ClockError::MissingMarker("minecraft:wake_up_from_sleep".to_owned())
                })?;
            let period = timeline.period_ticks.ok_or(ClockError::MissingMarker(
                "period for minecraft:wake_up_from_sleep".to_owned(),
            ))?;
            let clock = self
                .clocks
                .get_mut(clock_key)
                .ok_or_else(|| ClockError::MissingClock(clock_key.to_owned()))?;
            let current = clock.total_ticks();
            let within_period = current.rem_euclid(period);
            let delta = (wake - within_period).rem_euclid(period);
            clock.add_ticks(if delta == 0 { period } else { delta });
            self.mutation_generation = self.mutation_generation.wrapping_add(1);
            completion.clock_advanced = true;
        }
        Ok(completion)
    }

    pub fn should_roll_village_siege(
        &self,
        dimension: &DimensionType,
        default_timeline: Option<&Timeline>,
    ) -> bool {
        let Some(clock_key) = dimension.default_clock.as_deref() else {
            return false;
        };
        let Some(clock) = self.clocks.get(clock_key) else {
            return false;
        };
        let Some(timeline) = default_timeline else {
            return false;
        };
        let Some(period) = timeline.period_ticks else {
            return false;
        };
        timeline
            .marker("minecraft:roll_village_siege")
            .is_some_and(|marker| clock.total_ticks().rem_euclid(period) == marker)
    }
}
