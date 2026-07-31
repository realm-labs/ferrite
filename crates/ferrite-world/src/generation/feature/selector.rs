//! Composite configured-feature selectors with exact draw and child-call order.

use std::num::NonZeroU32;

use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;

pub fn random_selector(
    chances: &[f32],
    random: &mut impl GenerationRandom,
    mut place: impl FnMut(usize) -> bool,
) -> Result<bool, SelectorError> {
    for (index, chance) in chances.iter().copied().enumerate() {
        validate_probability(chance)?;
        if random.next_f32() < chance {
            return Ok(place(index));
        }
    }
    Ok(place(chances.len()))
}

pub fn weighted_random_selector(
    weights: &[u32],
    random: &mut impl GenerationRandom,
    mut place: impl FnMut(usize) -> bool,
) -> Result<bool, SelectorError> {
    let total = weights.iter().try_fold(0_u32, |total, weight| {
        total
            .checked_add(*weight)
            .filter(|sum| *sum <= i32::MAX as u32)
            .ok_or(SelectorError::WeightOverflow)
    })?;
    let Some(total) = NonZeroU32::new(total) else {
        return Ok(false);
    };
    let draw = random.next_u32(total);
    let mut cumulative = 0_u32;
    for (index, weight) in weights.iter().copied().enumerate() {
        cumulative += weight;
        if draw < cumulative {
            return Ok(place(index));
        }
    }
    unreachable!("bounded draw belongs to one positive cumulative interval")
}

pub fn simple_random_selector(
    child_count: usize,
    random: &mut impl GenerationRandom,
    place: impl FnOnce(usize) -> bool,
) -> Result<bool, SelectorError> {
    let count = u32::try_from(child_count).map_err(|_| SelectorError::TooManyChildren)?;
    let count = NonZeroU32::new(count).ok_or(SelectorError::EmptyChildren)?;
    Ok(place(random.next_u32(count) as usize))
}

#[must_use]
pub fn random_boolean_selector(
    random: &mut impl GenerationRandom,
    feature_true: impl FnOnce() -> bool,
    feature_false: impl FnOnce() -> bool,
) -> bool {
    if random.next_bool() {
        feature_true()
    } else {
        feature_false()
    }
}

pub fn sequence(
    child_count: usize,
    mut place: impl FnMut(usize) -> bool,
) -> Result<bool, SelectorError> {
    if child_count == 0 {
        return Err(SelectorError::EmptyChildren);
    }
    for index in 0..child_count {
        if !place(index) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn validate_probability(value: f32) -> Result<(), SelectorError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(SelectorError::InvalidProbability)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SelectorError {
    #[error("selector probability must be finite and in the inclusive range 0..=1")]
    InvalidProbability,
    #[error("selector child set cannot be empty")]
    EmptyChildren,
    #[error("selector has more children than the encoded index range")]
    TooManyChildren,
    #[error("selector weight sum exceeds the signed 32-bit bound")]
    WeightOverflow,
}
