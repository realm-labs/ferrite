//! Nether-vine configured features with exact candidate-search and column RNG ordering.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TwistingVinesConfig {
    pub spread_width: i32,
    pub spread_height: i32,
    pub maximum_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwistingVinesPlacement {
    Body,
    Head { age: u8 },
}

pub trait TwistingVinesWorld {
    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_twisting_vines_support(&self, state: BlockStateId) -> bool;

    fn is_outside_build_height(&self, position: BlockPos) -> bool;

    fn offer_twisting_vines(
        &mut self,
        position: BlockPos,
        placement: TwistingVinesPlacement,
        flags: u32,
    ) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeepingVinesPlacement {
    Body,
    Head { age: u8 },
}

pub trait WeepingVinesWorld {
    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_weeping_vines_support(&self, state: BlockStateId) -> bool;

    fn offer_nether_wart(&mut self, position: BlockPos, flags: u32) -> bool;

    fn offer_weeping_vines(
        &mut self,
        position: BlockPos,
        placement: WeepingVinesPlacement,
        flags: u32,
    ) -> bool;
}

pub fn place_twisting_vines<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: TwistingVinesConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, NetherVinesError>
where
    R: GenerationRandom,
    W: TwistingVinesWorld,
{
    validate_twisting_config(config)?;
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    if invalid_twisting_location(world, origin)? {
        return Ok(false);
    }
    let attempts = config.spread_width.wrapping_mul(config.spread_width);
    for _ in 0..attempts {
        let candidate = offset(
            origin,
            sample_inclusive(random, -config.spread_width, config.spread_width)?,
            sample_inclusive(random, -config.spread_height, config.spread_height)?,
            sample_inclusive(random, -config.spread_width, config.spread_width)?,
        )?;
        let Some(cursor) = first_air_above_ground(world, candidate)? else {
            continue;
        };
        if invalid_twisting_location(world, cursor)? {
            continue;
        }
        let mut length = sample_inclusive(random, 1, config.maximum_height)?;
        if random.next_u32(NonZeroU32::new(6).expect("twisting double bound is nonzero")) == 0 {
            length = length.wrapping_mul(2);
        }
        if random.next_u32(NonZeroU32::new(5).expect("twisting one bound is nonzero")) == 0 {
            length = 1;
        }
        place_twisting_column(world, cursor, length, random)?;
    }
    Ok(true)
}

pub fn place_weeping_vines<R, W>(
    world: &mut W,
    origin: BlockPos,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, NetherVinesError>
where
    R: GenerationRandom,
    W: WeepingVinesWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    if !world.is_empty_block(origin) {
        return Ok(false);
    }
    let above = offset(origin, 0, 1, 0)?;
    let above_state = world.block_state(above);
    if !world.is_weeping_vines_support(above_state) {
        return Ok(false);
    }
    let _ = world.offer_nether_wart(origin, 2);

    for _ in 0..200 {
        let candidate = offset(
            origin,
            draw_difference(random, 6, 6),
            draw_difference(random, 2, 5),
            draw_difference(random, 6, 6),
        )?;
        if !world.is_empty_block(candidate) {
            continue;
        }
        let mut supporting_neighbors = 0_u8;
        for (x, y, z) in [
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
            (-1, 0, 0),
            (1, 0, 0),
        ] {
            let neighbor = offset(candidate, x, y, z)?;
            let state = world.block_state(neighbor);
            if world.is_weeping_vines_support(state) {
                supporting_neighbors += 1;
                if supporting_neighbors == 2 {
                    break;
                }
            }
        }
        if supporting_neighbors == 1 {
            let _ = world.offer_nether_wart(candidate, 2);
        }
    }

    for _ in 0..100 {
        let candidate = offset(
            origin,
            draw_difference(random, 8, 8),
            draw_difference(random, 2, 7),
            draw_difference(random, 8, 8),
        )?;
        if !world.is_empty_block(candidate) {
            continue;
        }
        let above = offset(candidate, 0, 1, 0)?;
        let above_state = world.block_state(above);
        if !world.is_weeping_vines_support(above_state) {
            continue;
        }
        let mut length = 1 + random
            .next_u32(NonZeroU32::new(8).expect("weeping column-length bound is nonzero"))
            as i32;
        if random.next_u32(NonZeroU32::new(6).expect("weeping double bound is nonzero")) == 0 {
            length = length.wrapping_mul(2);
        }
        if random.next_u32(NonZeroU32::new(5).expect("weeping one bound is nonzero")) == 0 {
            length = 1;
        }
        place_weeping_column(world, candidate, length, random)?;
    }
    Ok(true)
}

fn validate_twisting_config(config: TwistingVinesConfig) -> Result<(), NetherVinesError> {
    if config.spread_width <= 0 || config.spread_height <= 0 || config.maximum_height <= 0 {
        return Err(NetherVinesError::NonPositiveConfiguration);
    }
    Ok(())
}

fn invalid_twisting_location<W: TwistingVinesWorld>(
    world: &mut W,
    position: BlockPos,
) -> Result<bool, NetherVinesError> {
    if !world.is_empty_block(position) {
        return Ok(true);
    }
    let below = offset(position, 0, -1, 0)?;
    let support = world.block_state(below);
    Ok(!world.is_twisting_vines_support(support))
}

fn first_air_above_ground<W: TwistingVinesWorld>(
    world: &mut W,
    start: BlockPos,
) -> Result<Option<BlockPos>, NetherVinesError> {
    let mut cursor = start;
    loop {
        cursor = offset(cursor, 0, -1, 0)?;
        if world.is_outside_build_height(cursor) {
            return Ok(None);
        }
        if world.is_empty_block(cursor) {
            continue;
        }
        return Ok(Some(offset(cursor, 0, 1, 0)?));
    }
}

fn place_twisting_column<R, W>(
    world: &mut W,
    origin: BlockPos,
    length: i32,
    random: &mut R,
) -> Result<(), NetherVinesError>
where
    R: GenerationRandom,
    W: TwistingVinesWorld,
{
    let mut cursor = origin;
    for index in 1..=length {
        if world.is_empty_block(cursor) {
            let terminal = index == length;
            let blocked_above = if terminal {
                false
            } else {
                !world.is_empty_block(offset(cursor, 0, 1, 0)?)
            };
            if terminal || blocked_above {
                let age = 17
                    + random
                        .next_u32(NonZeroU32::new(9).expect("twisting head-age bound is nonzero"))
                        as u8;
                let _ = world.offer_twisting_vines(cursor, TwistingVinesPlacement::Head { age }, 2);
                return Ok(());
            }
            let _ = world.offer_twisting_vines(cursor, TwistingVinesPlacement::Body, 2);
        }
        if index != length {
            cursor = offset(cursor, 0, 1, 0)?;
        }
    }
    Ok(())
}

fn place_weeping_column<R, W>(
    world: &mut W,
    origin: BlockPos,
    length: i32,
    random: &mut R,
) -> Result<(), NetherVinesError>
where
    R: GenerationRandom,
    W: WeepingVinesWorld,
{
    let mut cursor = origin;
    for index in 0..=length {
        if world.is_empty_block(cursor) {
            let terminal = index == length;
            let blocked_below = if terminal {
                false
            } else {
                !world.is_empty_block(offset(cursor, 0, -1, 0)?)
            };
            if terminal || blocked_below {
                let age = 17
                    + random
                        .next_u32(NonZeroU32::new(9).expect("weeping head-age bound is nonzero"))
                        as u8;
                let _ = world.offer_weeping_vines(cursor, WeepingVinesPlacement::Head { age }, 2);
                return Ok(());
            }
            let _ = world.offer_weeping_vines(cursor, WeepingVinesPlacement::Body, 2);
        }
        if index != length {
            cursor = offset(cursor, 0, -1, 0)?;
        }
    }
    Ok(())
}

fn draw_difference(random: &mut impl GenerationRandom, left_bound: u32, right_bound: u32) -> i32 {
    let left = random.next_u32(NonZeroU32::new(left_bound).expect("left bound is nonzero")) as i32;
    let right =
        random.next_u32(NonZeroU32::new(right_bound).expect("right bound is nonzero")) as i32;
    left - right
}

fn sample_inclusive(
    random: &mut impl GenerationRandom,
    minimum: i32,
    maximum: i32,
) -> Result<i32, NetherVinesError> {
    if minimum > maximum {
        return Err(NetherVinesError::InvalidInclusiveRange);
    }
    if minimum == maximum {
        return Ok(minimum);
    }
    let width = i64::from(maximum) - i64::from(minimum) + 1;
    let bound = u32::try_from(width)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(NetherVinesError::InvalidInclusiveRange)?;
    let offset = random.next_u32(bound);
    i64::from(minimum)
        .checked_add(i64::from(offset))
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(NetherVinesError::InvalidInclusiveRange)
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, NetherVinesError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(NetherVinesError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(NetherVinesError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(NetherVinesError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NetherVinesError {
    #[error("twisting-vines configuration values must be positive")]
    NonPositiveConfiguration,
    #[error("twisting-vines inclusive range is not representable")]
    InvalidInclusiveRange,
    #[error("Nether-vines position arithmetic overflowed")]
    PositionOverflow,
}
