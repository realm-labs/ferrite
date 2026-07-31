//! Seed-derived badlands clay-band generation and lookup.

use std::num::NonZeroU32;

use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

const BAND_COUNT: usize = 192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClayBandStates {
    pub terracotta: BlockStateId,
    pub orange: BlockStateId,
    pub yellow: BlockStateId,
    pub brown: BlockStateId,
    pub red: BlockStateId,
    pub white: BlockStateId,
    pub light_gray: BlockStateId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClayBands {
    values: [BlockStateId; BAND_COUNT],
}

impl ClayBands {
    pub fn generate(random: &mut impl GenerationRandom, states: ClayBandStates) -> Self {
        let mut values = [states.terracotta; BAND_COUNT];
        let mut index = 0;
        while index < BAND_COUNT {
            index += bounded(random, 5) + 1;
            if index < BAND_COUNT {
                values[index] = states.orange;
            }
            index += 1;
        }
        make_bands(random, &mut values, 1, states.yellow);
        make_bands(random, &mut values, 2, states.brown);
        make_bands(random, &mut values, 1, states.red);
        let white_count = bounded(random, 7) + 9;
        let mut start = 0;
        for _ in 0..white_count {
            if start >= BAND_COUNT {
                break;
            }
            values[start] = states.white;
            if start > 1 && random.next_bool() {
                values[start - 1] = states.light_gray;
            }
            if start + 1 < BAND_COUNT && random.next_bool() {
                values[start + 1] = states.light_gray;
            }
            start += bounded(random, 16) + 4;
        }
        Self { values }
    }

    pub fn state(&self, y: i32, offset_noise: f64) -> Result<BlockStateId, ClayBandError> {
        let rounded = java_round_to_i32(offset_noise * 4.0);
        let raw = y.wrapping_add(rounded).wrapping_add(BAND_COUNT as i32) % BAND_COUNT as i32;
        let index = usize::try_from(raw).map_err(|_| ClayBandError::NegativeIndex)?;
        Ok(self.values[index])
    }

    pub fn values(&self) -> &[BlockStateId; BAND_COUNT] {
        &self.values
    }
}

fn make_bands(
    random: &mut impl GenerationRandom,
    values: &mut [BlockStateId; BAND_COUNT],
    base_width: usize,
    state: BlockStateId,
) {
    let count = bounded(random, 10) + 6;
    for _ in 0..count {
        let width = base_width + bounded(random, 3);
        let start = bounded(random, BAND_COUNT);
        let end = start.saturating_add(width).min(BAND_COUNT);
        values[start..end].fill(state);
    }
}

fn java_round_to_i32(value: f64) -> i32 {
    ((value + 0.5).floor() as i64) as i32
}

fn bounded(random: &mut impl GenerationRandom, bound: usize) -> usize {
    random.next_u32(NonZeroU32::new(bound as u32).expect("clay-band random bound is nonzero"))
        as usize
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ClayBandError {
    #[error("Java remainder produced a negative clay-band index")]
    NegativeIndex,
}
