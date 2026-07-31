//! Improved, octave-Perlin, and double-Perlin normal noise.

use thiserror::Error;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};

const WRAP_PERIOD: f64 = 33_554_432.0;
const NORMAL_INPUT_FACTOR: f64 = 1.018_126_888_217_522_7;
const SMEAR_EPSILON: f64 = 1.000_000_011_686_097_4e-7;

const GRADIENTS: [[i32; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

#[derive(Debug, Clone)]
pub struct ImprovedNoise {
    permutation: [u8; 256],
    pub x_offset: f64,
    pub y_offset: f64,
    pub z_offset: f64,
}

impl ImprovedNoise {
    pub fn new(random: &mut impl GenerationRandom) -> Self {
        let x_offset = random.next_f64() * 256.0;
        let y_offset = random.next_f64() * 256.0;
        let z_offset = random.next_f64() * 256.0;
        let mut permutation = [0_u8; 256];
        for (index, value) in permutation.iter_mut().enumerate() {
            *value = index as u8;
        }
        for index in 0..256 {
            let bound = 256 - index;
            let offset = random.next_u32(
                std::num::NonZeroU32::new(bound as u32)
                    .expect("improved-noise shuffle bound is nonzero"),
            ) as usize;
            permutation.swap(index, index + offset);
        }
        Self {
            permutation,
            x_offset,
            y_offset,
            z_offset,
        }
    }

    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        self.sample_with_smear(x, y, z, 0.0, 0.0)
    }

    pub fn sample_with_smear(&self, x: f64, y: f64, z: f64, y_scale: f64, y_fudge: f64) -> f64 {
        let shifted_x = x + self.x_offset;
        let shifted_y = y + self.y_offset;
        let shifted_z = z + self.z_offset;
        let floor_x = minecraft_floor(shifted_x);
        let floor_y = minecraft_floor(shifted_y);
        let floor_z = minecraft_floor(shifted_z);
        let local_x = shifted_x - f64::from(floor_x);
        let local_y = shifted_y - f64::from(floor_y);
        let local_z = shifted_z - f64::from(floor_z);
        let smear = if y_scale != 0.0 {
            let limit = if y_fudge >= 0.0 && y_fudge < local_y {
                y_fudge
            } else {
                local_y
            };
            (limit / y_scale + SMEAR_EPSILON).floor() * y_scale
        } else {
            0.0
        };
        self.sample_and_lerp(
            [floor_x, floor_y, floor_z],
            [local_x, local_y - smear, local_z],
            local_y,
        )
    }

    pub fn permutation(&self) -> &[u8; 256] {
        &self.permutation
    }

    fn sample_and_lerp(&self, lattice: [i32; 3], local: [f64; 3], original_y: f64) -> f64 {
        let [x, y, z] = lattice;
        let [local_x, local_y, local_z] = local;
        let x0 = self.p(x);
        let x1 = self.p(x.wrapping_add(1));
        let xy00 = self.p(x0.wrapping_add(y));
        let xy01 = self.p(x0.wrapping_add(y).wrapping_add(1));
        let xy10 = self.p(x1.wrapping_add(y));
        let xy11 = self.p(x1.wrapping_add(y).wrapping_add(1));
        let d000 = self.gradient(self.p(xy00.wrapping_add(z)), local_x, local_y, local_z);
        let d100 = self.gradient(
            self.p(xy10.wrapping_add(z)),
            local_x - 1.0,
            local_y,
            local_z,
        );
        let d010 = self.gradient(
            self.p(xy01.wrapping_add(z)),
            local_x,
            local_y - 1.0,
            local_z,
        );
        let d110 = self.gradient(
            self.p(xy11.wrapping_add(z)),
            local_x - 1.0,
            local_y - 1.0,
            local_z,
        );
        let d001 = self.gradient(
            self.p(xy00.wrapping_add(z).wrapping_add(1)),
            local_x,
            local_y,
            local_z - 1.0,
        );
        let d101 = self.gradient(
            self.p(xy10.wrapping_add(z).wrapping_add(1)),
            local_x - 1.0,
            local_y,
            local_z - 1.0,
        );
        let d011 = self.gradient(
            self.p(xy01.wrapping_add(z).wrapping_add(1)),
            local_x,
            local_y - 1.0,
            local_z - 1.0,
        );
        let d111 = self.gradient(
            self.p(xy11.wrapping_add(z).wrapping_add(1)),
            local_x - 1.0,
            local_y - 1.0,
            local_z - 1.0,
        );
        lerp3(
            smoothstep(local_x),
            smoothstep(original_y),
            smoothstep(local_z),
            [d000, d100, d010, d110, d001, d101, d011, d111],
        )
    }

    fn p(&self, index: i32) -> i32 {
        i32::from(self.permutation[(index & 255) as usize])
    }

    fn gradient(&self, hash: i32, x: f64, y: f64, z: f64) -> f64 {
        let gradient = GRADIENTS[(hash & 15) as usize];
        f64::from(gradient[0]) * x + f64::from(gradient[1]) * y + f64::from(gradient[2]) * z
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoiseParameters {
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct PerlinNoise {
    levels: Vec<Option<ImprovedNoise>>,
    parameters: NoiseParameters,
    lowest_frequency_input_factor: f64,
    lowest_frequency_value_factor: f64,
    maximum: f64,
}

impl PerlinNoise {
    pub fn keyed(
        parameters: NoiseParameters,
        mut source_for_octave: impl FnMut(i32) -> LegacyRandom,
    ) -> Self {
        let mut levels = Vec::with_capacity(parameters.amplitudes.len());
        for (index, amplitude) in parameters.amplitudes.iter().copied().enumerate() {
            if amplitude == 0.0 {
                levels.push(None);
            } else {
                let octave = parameters.first_octave.wrapping_add(index as i32);
                levels.push(Some(ImprovedNoise::new(&mut source_for_octave(octave))));
            }
        }
        Self::from_levels(parameters, levels)
    }

    pub fn legacy(
        random: &mut LegacyRandom,
        parameters: NoiseParameters,
    ) -> Result<Self, NoiseError> {
        let octave_count = parameters.amplitudes.len();
        let zero_octave_index = parameters.first_octave.wrapping_neg();
        let mut levels = vec![None; octave_count];
        let zero = ImprovedNoise::new(random);
        if let Ok(index) = usize::try_from(zero_octave_index)
            && index < octave_count
            && parameters.amplitudes[index] != 0.0
        {
            levels[index] = Some(zero);
        }
        for index in (0..zero_octave_index).rev() {
            if let Ok(index) = usize::try_from(index)
                && index < octave_count
                && parameters.amplitudes[index] != 0.0
            {
                levels[index] = Some(ImprovedNoise::new(random));
            } else {
                consume_unbounded(random, 262);
            }
        }
        let actual = levels.iter().filter(|level| level.is_some()).count();
        let expected = parameters
            .amplitudes
            .iter()
            .filter(|amplitude| **amplitude != 0.0)
            .count();
        if actual != expected {
            return Err(NoiseError::LegacyLevelMismatch);
        }
        if zero_octave_index < octave_count as i32 - 1 {
            return Err(NoiseError::PositiveLegacyOctave);
        }
        Ok(Self::from_levels(parameters, levels))
    }

    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        self.sample_with_smear(x, y, z, 0.0, 0.0)
    }

    pub fn sample_with_smear(&self, x: f64, y: f64, z: f64, y_scale: f64, y_fudge: f64) -> f64 {
        let mut result = 0.0;
        let mut frequency = self.lowest_frequency_input_factor;
        let mut weight = self.lowest_frequency_value_factor;
        for (index, level) in self.levels.iter().enumerate() {
            if let Some(level) = level {
                result += self.parameters.amplitudes[index]
                    * level.sample_with_smear(
                        wrap(x * frequency),
                        wrap(y * frequency),
                        wrap(z * frequency),
                        y_scale * frequency,
                        y_fudge * frequency,
                    )
                    * weight;
            }
            frequency *= 2.0;
            weight /= 2.0;
        }
        result
    }

    pub fn maximum(&self) -> f64 {
        self.maximum
    }

    pub fn max_broken_value(&self, y_scale: f64) -> f64 {
        self.edge_value(y_scale + 2.0)
    }

    pub fn levels(&self) -> &[Option<ImprovedNoise>] {
        &self.levels
    }

    pub fn octave_from_high(&self, index: usize) -> Option<&ImprovedNoise> {
        self.levels
            .len()
            .checked_sub(1 + index)
            .and_then(|index| self.levels[index].as_ref())
    }

    fn from_levels(parameters: NoiseParameters, levels: Vec<Option<ImprovedNoise>>) -> Self {
        let count = parameters.amplitudes.len() as i32;
        let zero_octave_index = parameters.first_octave.wrapping_neg();
        let lowest_frequency_input_factor =
            2.0_f64.powf(f64::from(zero_octave_index.wrapping_neg()));
        let lowest_frequency_value_factor =
            2.0_f64.powf(f64::from(count - 1)) / (2.0_f64.powf(f64::from(count)) - 1.0);
        let mut result = Self {
            levels,
            parameters,
            lowest_frequency_input_factor,
            lowest_frequency_value_factor,
            maximum: 0.0,
        };
        result.maximum = result.edge_value(2.0);
        result
    }

    fn edge_value(&self, noise_value: f64) -> f64 {
        let mut result = 0.0;
        let mut weight = self.lowest_frequency_value_factor;
        for (index, level) in self.levels.iter().enumerate() {
            if level.is_some() {
                result += self.parameters.amplitudes[index] * noise_value * weight;
            }
            weight /= 2.0;
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct NormalNoise {
    first: PerlinNoise,
    second: PerlinNoise,
    value_factor: f64,
    maximum: f64,
}

impl NormalNoise {
    pub fn keyed(
        parameters: NoiseParameters,
        mut source_for_octave: impl FnMut(u8, i32) -> LegacyRandom,
    ) -> Self {
        let first = PerlinNoise::keyed(parameters.clone(), |octave| source_for_octave(0, octave));
        let second = PerlinNoise::keyed(parameters.clone(), |octave| source_for_octave(1, octave));
        Self::from_halves(parameters, first, second)
    }

    pub fn legacy(
        random: &mut LegacyRandom,
        parameters: NoiseParameters,
    ) -> Result<Self, NoiseError> {
        let first = PerlinNoise::legacy(random, parameters.clone())?;
        let second = PerlinNoise::legacy(random, parameters.clone())?;
        Ok(Self::from_halves(parameters, first, second))
    }

    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        let scaled_x = x * NORMAL_INPUT_FACTOR;
        let scaled_y = y * NORMAL_INPUT_FACTOR;
        let scaled_z = z * NORMAL_INPUT_FACTOR;
        (self.first.sample(x, y, z) + self.second.sample(scaled_x, scaled_y, scaled_z))
            * self.value_factor
    }

    pub fn maximum(&self) -> f64 {
        self.maximum
    }

    pub fn halves(&self) -> (&PerlinNoise, &PerlinNoise) {
        (&self.first, &self.second)
    }

    fn from_halves(parameters: NoiseParameters, first: PerlinNoise, second: PerlinNoise) -> Self {
        let mut minimum = i32::MAX;
        let mut maximum = i32::MIN;
        for (index, amplitude) in parameters.amplitudes.iter().enumerate() {
            if *amplitude != 0.0 {
                minimum = minimum.min(index as i32);
                maximum = maximum.max(index as i32);
            }
        }
        let span = maximum.wrapping_sub(minimum);
        let expected_deviation = 0.1 * (1.0 + 1.0 / f64::from(span.wrapping_add(1)));
        let value_factor = (1.0 / 6.0) / expected_deviation;
        let combined_maximum = first.maximum() + second.maximum();
        Self {
            first,
            second,
            value_factor,
            maximum: combined_maximum * value_factor,
        }
    }
}

pub fn wrap(value: f64) -> f64 {
    value - (value / WRAP_PERIOD + 0.5).floor() * WRAP_PERIOD
}

fn consume_unbounded(random: &mut LegacyRandom, count: usize) {
    for _ in 0..count {
        let _ = random.next_i32();
    }
}

fn minecraft_floor(value: f64) -> i32 {
    let truncated = value as i32;
    if value < f64::from(truncated) {
        truncated.wrapping_sub(1)
    } else {
        truncated
    }
}

fn smoothstep(value: f64) -> f64 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

fn lerp3(x: f64, y: f64, z: f64, values: [f64; 8]) -> f64 {
    lerp(
        z,
        lerp(
            y,
            lerp(x, values[0], values[1]),
            lerp(x, values[2], values[3]),
        ),
        lerp(
            y,
            lerp(x, values[4], values[5]),
            lerp(x, values[6], values[7]),
        ),
    )
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NoiseError {
    #[error("legacy Perlin initialization did not allocate every nonzero amplitude")]
    LegacyLevelMismatch,
    #[error("legacy Perlin initialization does not support positive octaves")]
    PositiveLegacyOctave,
}
