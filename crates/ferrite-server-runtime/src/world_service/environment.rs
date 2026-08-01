use ferrite_foundation::identity::DimensionId;
use ferrite_gameplay::environment::weather::{
    WeatherData, WeatherDimension, WeatherPacket, WeatherRandom, WeatherStrengths,
    run_weather_phase,
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelEnvironment {
    game_time: i64,
    day_time: i64,
    weather: WeatherData,
    strengths: WeatherStrengths,
    random_state: u64,
}

impl LevelEnvironment {
    #[must_use]
    pub fn new(seed: i64, dimension: &DimensionId) -> Self {
        let mut random_state = seed as u64 ^ 0x9e37_79b9_7f4a_7c15;
        for byte in dimension.to_string().bytes() {
            random_state = (random_state ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self {
            game_time: 0,
            day_time: 0,
            weather: WeatherData::default(),
            strengths: WeatherStrengths::default(),
            random_state,
        }
    }

    pub fn tick(
        &mut self,
        dimension: &DimensionId,
    ) -> Result<EnvironmentProjection, LevelEnvironmentError> {
        self.game_time = self
            .game_time
            .checked_add(1)
            .ok_or(LevelEnvironmentError::TimeExhausted)?;
        self.day_time = self
            .day_time
            .checked_add(1)
            .ok_or(LevelEnvironmentError::TimeExhausted)?;
        let weather_dimension = weather_dimension(dimension);
        let mut random = EnvironmentRandom(self.random_state);
        let phase = run_weather_phase(
            &mut self.weather,
            &mut self.strengths,
            weather_dimension,
            true,
            true,
            &mut random,
        );
        self.random_state = random.0;
        Ok(EnvironmentProjection {
            game_time: self.game_time,
            day_time: self.day_time,
            weather: phase.packets,
        })
    }

    #[must_use]
    pub const fn game_time(self) -> i64 {
        self.game_time
    }

    #[must_use]
    pub const fn day_time(self) -> i64 {
        self.day_time
    }

    #[must_use]
    pub const fn weather(self) -> WeatherData {
        self.weather
    }

    #[must_use]
    pub const fn strengths(self) -> WeatherStrengths {
        self.strengths
    }

    #[must_use]
    pub const fn random_state(self) -> u64 {
        self.random_state
    }

    pub(crate) const fn from_durable(
        game_time: i64,
        day_time: i64,
        weather: WeatherData,
        strengths: WeatherStrengths,
        random_state: u64,
    ) -> Self {
        Self {
            game_time,
            day_time,
            weather,
            strengths,
            random_state,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentProjection {
    pub game_time: i64,
    pub day_time: i64,
    pub weather: Vec<WeatherPacket>,
}

struct EnvironmentRandom(u64);

impl WeatherRandom for EnvironmentRandom {
    fn next_int(&mut self, bound: u32) -> u32 {
        debug_assert!(bound != 0);
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 32) as u32) % bound
    }
}

fn weather_dimension(dimension: &DimensionId) -> WeatherDimension {
    WeatherDimension {
        has_sky_light: dimension.to_string() == "minecraft:overworld",
        has_ceiling: dimension.to_string() == "minecraft:the_nether",
        is_end: dimension.to_string() == "minecraft:the_end",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LevelEnvironmentError {
    #[error("level time exhausted its signed durable range")]
    TimeExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_and_weather_random_stream_continue_deterministically() {
        let dimension = DimensionId::new("minecraft:overworld".parse().unwrap());
        let mut first = LevelEnvironment::new(42, &dimension);
        let mut second = first;
        for expected in 1..=12 {
            let left = first.tick(&dimension).unwrap();
            let right = second.tick(&dimension).unwrap();
            assert_eq!(left, right);
            assert_eq!(left.game_time, expected);
        }
    }
}
