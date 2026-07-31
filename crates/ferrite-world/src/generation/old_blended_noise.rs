//! Legacy three-stack blended density used by vanilla terrain routers.

use thiserror::Error;

use crate::generation::feature::random::LegacyRandom;
use crate::generation::noise::{NoiseError, NoiseParameters, PerlinNoise, wrap};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OldBlendedNoiseConfig {
    pub xz_scale: f64,
    pub y_scale: f64,
    pub xz_factor: f64,
    pub y_factor: f64,
    pub smear_scale_multiplier: f64,
}

#[derive(Debug, Clone)]
pub struct OldBlendedNoise {
    minimum_limit: PerlinNoise,
    maximum_limit: PerlinNoise,
    main: PerlinNoise,
    xz_multiplier: f64,
    y_multiplier: f64,
    xz_factor: f64,
    y_factor: f64,
    smear_scale_multiplier: f64,
    maximum: f64,
}

impl OldBlendedNoise {
    pub fn new(
        random: &mut LegacyRandom,
        config: OldBlendedNoiseConfig,
    ) -> Result<Self, OldBlendedNoiseError> {
        validate(config)?;
        let limit_parameters = NoiseParameters {
            first_octave: -15,
            amplitudes: vec![1.0; 16],
        };
        let main_parameters = NoiseParameters {
            first_octave: -7,
            amplitudes: vec![1.0; 8],
        };
        let minimum_limit = PerlinNoise::legacy(random, limit_parameters.clone())?;
        let maximum_limit = PerlinNoise::legacy(random, limit_parameters)?;
        let main = PerlinNoise::legacy(random, main_parameters)?;
        let xz_multiplier = 684.412 * config.xz_scale;
        let y_multiplier = 684.412 * config.y_scale;
        let maximum = minimum_limit.max_broken_value(y_multiplier);
        Ok(Self {
            minimum_limit,
            maximum_limit,
            main,
            xz_multiplier,
            y_multiplier,
            xz_factor: config.xz_factor,
            y_factor: config.y_factor,
            smear_scale_multiplier: config.smear_scale_multiplier,
            maximum,
        })
    }

    pub fn sample(&self, x: i32, y: i32, z: i32) -> f64 {
        let limit_x = f64::from(x) * self.xz_multiplier;
        let limit_y = f64::from(y) * self.y_multiplier;
        let limit_z = f64::from(z) * self.xz_multiplier;
        let main_x = limit_x / self.xz_factor;
        let main_y = limit_y / self.y_factor;
        let main_z = limit_z / self.xz_factor;
        let limit_smear = self.y_multiplier * self.smear_scale_multiplier;
        let main_smear = limit_smear / self.y_factor;
        let mut main_value = 0.0;
        let mut power = 1.0;
        for index in 0..8 {
            if let Some(noise) = self.main.octave_from_high(index) {
                main_value += noise.sample_with_smear(
                    wrap(main_x * power),
                    wrap(main_y * power),
                    wrap(main_z * power),
                    main_smear * power,
                    main_y * power,
                ) / power;
            }
            power /= 2.0;
        }
        let factor = (main_value / 10.0 + 1.0) / 2.0;
        let maximum_only = factor >= 1.0;
        let minimum_only = factor <= 0.0;
        let mut minimum_value = 0.0;
        let mut maximum_value = 0.0;
        power = 1.0;
        for index in 0..16 {
            let wrapped_x = wrap(limit_x * power);
            let wrapped_y = wrap(limit_y * power);
            let wrapped_z = wrap(limit_z * power);
            let smear = limit_smear * power;
            if !maximum_only && let Some(noise) = self.minimum_limit.octave_from_high(index) {
                minimum_value += noise.sample_with_smear(
                    wrapped_x,
                    wrapped_y,
                    wrapped_z,
                    smear,
                    limit_y * power,
                ) / power;
            }
            if !minimum_only && let Some(noise) = self.maximum_limit.octave_from_high(index) {
                maximum_value += noise.sample_with_smear(
                    wrapped_x,
                    wrapped_y,
                    wrapped_z,
                    smear,
                    limit_y * power,
                ) / power;
            }
            power /= 2.0;
        }
        clamped_lerp(factor, minimum_value / 512.0, maximum_value / 512.0) / 128.0
    }

    pub fn bounds(&self) -> (f64, f64) {
        (-self.maximum, self.maximum)
    }

    pub fn stacks(&self) -> (&PerlinNoise, &PerlinNoise, &PerlinNoise) {
        (&self.minimum_limit, &self.maximum_limit, &self.main)
    }
}

fn validate(config: OldBlendedNoiseConfig) -> Result<(), OldBlendedNoiseError> {
    let scales = [
        config.xz_scale,
        config.y_scale,
        config.xz_factor,
        config.y_factor,
    ];
    if scales
        .iter()
        .any(|value| !value.is_finite() || !(0.001..=1_000.0).contains(value))
        || !config.smear_scale_multiplier.is_finite()
        || !(1.0..=8.0).contains(&config.smear_scale_multiplier)
    {
        Err(OldBlendedNoiseError::InvalidConfiguration)
    } else {
        Ok(())
    }
}

fn clamped_lerp(delta: f64, minimum: f64, maximum: f64) -> f64 {
    if delta < 0.0 {
        minimum
    } else if delta > 1.0 {
        maximum
    } else {
        minimum + delta * (maximum - minimum)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OldBlendedNoiseError {
    #[error(transparent)]
    Noise(#[from] NoiseError),
    #[error("old-blended noise parameters violate codec bounds")]
    InvalidConfiguration,
}
