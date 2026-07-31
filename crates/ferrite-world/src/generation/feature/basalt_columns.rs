//! Basalt-column placement with eager random candidates and ordered local searches.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, PartialEq)]
pub struct BasaltColumnsConfig {
    pub reach: IntProvider,
    pub height: IntProvider,
    pub basalt: BlockStateId,
}

pub trait BasaltColumnsWorld {
    fn sea_level(&self) -> i32;

    fn minimum_y(&self) -> i32;

    fn maximum_y(&self) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_exact_lava(&self, state: BlockStateId) -> bool;

    fn is_exact_basalt(&self, state: BlockStateId) -> bool;

    fn is_banned_basalt_support(&self, state: BlockStateId) -> bool;

    fn offer_basalt_column(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_basalt_columns<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &BasaltColumnsConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasaltColumnsError>
where
    R: GenerationRandom,
    W: BasaltColumnsWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    if !can_place_at(world, origin)? {
        return Ok(false);
    }
    let height = sample_nonnegative(&config.height, random)?;
    let clustered = random.next_f32() < 0.9;
    let candidate_reach = height.min(if clustered { 5 } else { 8 });
    let candidate_count = if clustered { 50 } else { 15 };
    let mut any_offered = false;
    for _ in 0..candidate_count {
        let candidate = random_candidate(origin, candidate_reach, random)?;
        let distance = candidate.x.abs_diff(origin.x) + candidate.z.abs_diff(origin.z);
        let distance = i32::try_from(distance).map_err(|_| BasaltColumnsError::PositionOverflow)?;
        let tentative_height = height.wrapping_sub(distance);
        if tentative_height < 0 {
            continue;
        }
        let reach = sample_nonnegative(&config.reach, random)?;
        any_offered |=
            place_candidate_columns(world, candidate, tentative_height, reach, config.basalt)?;
    }
    Ok(any_offered)
}

fn random_candidate(
    origin: BlockPos,
    reach: i32,
    random: &mut impl GenerationRandom,
) -> Result<BlockPos, BasaltColumnsError> {
    let width = reach
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .and_then(|value| u32::try_from(value).ok())
        .and_then(NonZeroU32::new)
        .ok_or(BasaltColumnsError::InvalidCandidateReach { reach })?;
    let x_offset = random.next_u32(width) as i32 - reach;
    let y_offset = random.next_u32(NonZeroU32::new(1).expect("fixed Y bound is nonzero")) as i32;
    let z_offset = random.next_u32(width) as i32 - reach;
    offset(origin, x_offset, y_offset, z_offset)
}

fn place_candidate_columns<W: BasaltColumnsWorld>(
    world: &mut W,
    candidate: BlockPos,
    tentative_height: i32,
    reach: i32,
    basalt: BlockStateId,
) -> Result<bool, BasaltColumnsError> {
    let mut any_offered = false;
    for z_offset in -reach..=reach {
        for x_offset in -reach..=reach {
            let local = offset(candidate, x_offset, 0, z_offset)?;
            let distance = x_offset.unsigned_abs() + z_offset.unsigned_abs();
            let distance =
                i32::try_from(distance).map_err(|_| BasaltColumnsError::PositionOverflow)?;
            let Some(mut base) = find_column_base(world, local, distance)? else {
                continue;
            };
            let mut allowance = tentative_height - distance / 2;
            while allowance >= 0 {
                let state = world.block_state(base);
                if is_air_or_low_lava(world, state, base.y) {
                    let _ = world.offer_basalt_column(base, basalt, 3);
                    any_offered = true;
                } else if !world.is_exact_basalt(state) {
                    break;
                }
                base = offset(base, 0, 1, 0)?;
                allowance -= 1;
            }
        }
    }
    Ok(any_offered)
}

fn find_column_base<W: BasaltColumnsWorld>(
    world: &mut W,
    start: BlockPos,
    distance: i32,
) -> Result<Option<BlockPos>, BasaltColumnsError> {
    let start_state = world.block_state(start);
    if is_air_or_low_lava(world, start_state, start.y) {
        find_surface(world, start, distance)
    } else {
        find_air(world, start, distance)
    }
}

fn find_surface<W: BasaltColumnsWorld>(
    world: &mut W,
    start: BlockPos,
    distance: i32,
) -> Result<Option<BlockPos>, BasaltColumnsError> {
    let minimum_surface_y = world
        .minimum_y()
        .checked_add(1)
        .ok_or(BasaltColumnsError::PositionOverflow)?;
    let mut cursor = start;
    for _ in 0..distance {
        if cursor.y <= minimum_surface_y {
            break;
        }
        cursor = offset(cursor, 0, -1, 0)?;
        if can_place_at(world, cursor)? {
            return Ok(Some(cursor));
        }
    }
    Ok(None)
}

fn find_air<W: BasaltColumnsWorld>(
    world: &mut W,
    start: BlockPos,
    distance: i32,
) -> Result<Option<BlockPos>, BasaltColumnsError> {
    let mut cursor = start;
    for _ in 0..distance {
        cursor = offset(cursor, 0, 1, 0)?;
        if cursor.y > world.maximum_y() {
            break;
        }
        let state = world.block_state(cursor);
        if world.is_banned_basalt_support(state) {
            return Ok(None);
        }
        if world.is_air(state) {
            return Ok(Some(cursor));
        }
    }
    Ok(None)
}

fn can_place_at<W: BasaltColumnsWorld>(
    world: &mut W,
    position: BlockPos,
) -> Result<bool, BasaltColumnsError> {
    let state = world.block_state(position);
    if !is_air_or_low_lava(world, state, position.y) {
        return Ok(false);
    }
    let below = offset(position, 0, -1, 0)?;
    let below_state = world.block_state(below);
    Ok(!world.is_air(below_state) && !world.is_banned_basalt_support(below_state))
}

fn is_air_or_low_lava<W: BasaltColumnsWorld>(world: &W, state: BlockStateId, y: i32) -> bool {
    world.is_air(state) || y <= world.sea_level() && world.is_exact_lava(state)
}

fn sample_nonnegative(
    provider: &IntProvider,
    random: &mut impl GenerationRandom,
) -> Result<i32, BasaltColumnsError> {
    let value = provider.sample(random)?;
    if value < 0 {
        Err(BasaltColumnsError::NegativeProviderValue { value })
    } else {
        Ok(value)
    }
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, BasaltColumnsError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(BasaltColumnsError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(BasaltColumnsError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(BasaltColumnsError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BasaltColumnsError {
    #[error("basalt-columns integer provider failed")]
    Provider(#[from] ProviderError),
    #[error("basalt-columns provider returned negative value {value}")]
    NegativeProviderValue { value: i32 },
    #[error("basalt-columns candidate reach {reach} is not representable")]
    InvalidCandidateReach { reach: i32 },
    #[error("basalt-columns position arithmetic overflowed")]
    PositionOverflow,
}
