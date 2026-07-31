//! Audited integer providers and vertical anchors.

use std::num::NonZeroU32;

use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;

#[derive(Debug, Clone, PartialEq)]
pub enum IntProvider {
    Constant(i32),
    Uniform {
        minimum: i32,
        maximum: i32,
    },
    Weighted(Vec<WeightedInt>),
    Clamped {
        source: Box<Self>,
        minimum: i32,
        maximum: i32,
    },
    BiasedToBottom {
        minimum: i32,
        maximum: i32,
    },
    ClampedNormal {
        mean: f64,
        deviation: f64,
        minimum: i32,
        maximum: i32,
    },
    ZeroPlateauTrapezoid {
        radius: u32,
    },
}

impl IntProvider {
    pub fn sample(&self, random: &mut impl GenerationRandom) -> Result<i32, ProviderError> {
        match self {
            Self::Constant(value) => Ok(*value),
            Self::Uniform { minimum, maximum } => inclusive(random, *minimum, *maximum),
            Self::Weighted(entries) => sample_weighted(entries, random),
            Self::Clamped {
                source,
                minimum,
                maximum,
            } => {
                validate_bounds(*minimum, *maximum)?;
                Ok(source.sample(random)?.clamp(*minimum, *maximum))
            }
            Self::BiasedToBottom { minimum, maximum } => {
                validate_bounds(*minimum, *maximum)?;
                let width = inclusive_width(*minimum, *maximum)?;
                let outer = random.next_u32(width);
                let inner =
                    random.next_u32(NonZeroU32::new(outer + 1).expect("outer + one is nonzero"));
                i64::from(*minimum)
                    .checked_add(i64::from(inner))
                    .and_then(|value| i32::try_from(value).ok())
                    .ok_or(ProviderError::RangeTooWide)
            }
            Self::ClampedNormal {
                mean,
                deviation,
                minimum,
                maximum,
            } => {
                validate_bounds(*minimum, *maximum)?;
                if !mean.is_finite() || !deviation.is_finite() || *deviation < 0.0 {
                    return Err(ProviderError::InvalidNormal);
                }
                let sampled = random.next_gaussian().mul_add(*deviation, *mean);
                Ok(
                    (sampled.clamp(f64::from(*minimum), f64::from(*maximum)) as i32)
                        .clamp(*minimum, *maximum),
                )
            }
            Self::ZeroPlateauTrapezoid { radius } => {
                i32::try_from(*radius).map_err(|_| ProviderError::RangeTooWide)?;
                let bound = radius
                    .checked_add(1)
                    .and_then(NonZeroU32::new)
                    .ok_or(ProviderError::RangeTooWide)?;
                let left = random.next_u32(bound) as i32;
                let right = random.next_u32(bound) as i32;
                Ok(left - right)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedInt {
    pub weight: NonZeroU32,
    pub provider: IntProvider,
}

fn sample_weighted(
    entries: &[WeightedInt],
    random: &mut impl GenerationRandom,
) -> Result<i32, ProviderError> {
    let total = entries.iter().try_fold(0_u32, |total, entry| {
        total
            .checked_add(entry.weight.get())
            .ok_or(ProviderError::WeightOverflow)
    })?;
    let total = NonZeroU32::new(total).ok_or(ProviderError::EmptyWeights)?;
    let draw = random.next_u32(total);
    let mut cumulative = 0_u32;
    for entry in entries {
        cumulative += entry.weight.get();
        if draw < cumulative {
            return entry.provider.sample(random);
        }
    }
    unreachable!("bounded weighted draw belongs to one entry")
}

fn inclusive(
    random: &mut impl GenerationRandom,
    minimum: i32,
    maximum: i32,
) -> Result<i32, ProviderError> {
    validate_bounds(minimum, maximum)?;
    let offset = random.next_u32(inclusive_width(minimum, maximum)?);
    i64::from(minimum)
        .checked_add(i64::from(offset))
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(ProviderError::RangeTooWide)
}

fn inclusive_width(minimum: i32, maximum: i32) -> Result<NonZeroU32, ProviderError> {
    let width = i64::from(maximum) - i64::from(minimum) + 1;
    u32::try_from(width)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(ProviderError::RangeTooWide)
}

fn validate_bounds(minimum: i32, maximum: i32) -> Result<(), ProviderError> {
    if minimum <= maximum {
        Ok(())
    } else {
        Err(ProviderError::InvertedBounds)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightAnchor {
    Absolute(i32),
    AboveBottom(i32),
    BelowTop(i32),
}

impl HeightAnchor {
    pub fn resolve(self, context: HeightContext) -> Result<i32, ProviderError> {
        if context.depth <= 0 {
            return Err(ProviderError::InvalidGenerationDepth);
        }
        match self {
            Self::Absolute(value) => Ok(value),
            Self::AboveBottom(offset) => context
                .minimum_y
                .checked_add(offset)
                .ok_or(ProviderError::AnchorOverflow),
            Self::BelowTop(offset) => context
                .minimum_y
                .checked_add(context.depth - 1)
                .and_then(|top| top.checked_sub(offset))
                .ok_or(ProviderError::AnchorOverflow),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightContext {
    pub minimum_y: i32,
    pub depth: i32,
}

pub fn uniform_height(
    minimum: HeightAnchor,
    maximum: HeightAnchor,
    context: HeightContext,
    random: &mut impl GenerationRandom,
) -> Result<i32, ProviderError> {
    let minimum = minimum.resolve(context)?;
    let maximum = maximum.resolve(context)?;
    if minimum > maximum {
        return Ok(minimum);
    }
    inclusive(random, minimum, maximum)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ProviderError {
    #[error("provider minimum exceeds maximum")]
    InvertedBounds,
    #[error("provider range cannot be represented by the random-source bound")]
    RangeTooWide,
    #[error("weighted provider cannot be empty")]
    EmptyWeights,
    #[error("weighted provider total overflowed")]
    WeightOverflow,
    #[error("normal provider parameters must be finite with nonnegative deviation")]
    InvalidNormal,
    #[error("generation depth must be positive")]
    InvalidGenerationDepth,
    #[error("height anchor arithmetic overflowed")]
    AnchorOverflow,
}
