//! Vegetation configured features with explicit random and world-access ordering.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BambooStates {
    pub trunk: BlockStateId,
    pub final_large: BlockStateId,
    pub top_large: BlockStateId,
    pub top_small: BlockStateId,
    pub podzol: BlockStateId,
}

pub trait BambooWorld {
    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn supports_bamboo(&self, state: BlockStateId) -> bool;

    fn world_surface_height(&mut self, x: i32, z: i32) -> i32;

    fn beneath_bamboo_podzol_replaceable(&self, state: BlockStateId) -> bool;

    fn offer_bamboo_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_bamboo<R, W>(
    world: &mut W,
    origin: BlockPos,
    podzol_probability: f32,
    states: BambooStates,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BambooFeatureError>
where
    R: GenerationRandom,
    W: BambooWorld,
{
    if !podzol_probability.is_finite() || !(0.0..=1.0).contains(&podzol_probability) {
        return Err(BambooFeatureError::InvalidProbability);
    }
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    if !world.is_empty_block(origin) {
        return Ok(false);
    }
    let below = offset(origin, 0, -1, 0)?;
    let support = world.block_state(below);
    if !world.supports_bamboo(support) {
        return Ok(false);
    }

    let target_height =
        5 + random.next_u32(NonZeroU32::new(12).expect("bamboo height bound is nonzero")) as i32;
    if random.next_f32() < podzol_probability {
        let radius =
            1 + random.next_u32(NonZeroU32::new(4).expect("bamboo radius bound is nonzero")) as i32;
        place_podzol_disk(world, origin, radius, states.podzol)?;
    }

    let mut cursor = origin;
    let mut growth = 0_i32;
    while growth < target_height && world.is_empty_block(cursor) {
        let _ = world.offer_bamboo_block(cursor, states.trunk, 2);
        cursor = offset(cursor, 0, 1, 0)?;
        growth += 1;
    }
    if growth >= 3 {
        let _ = world.offer_bamboo_block(cursor, states.final_large, 2);
        cursor = offset(cursor, 0, -1, 0)?;
        let _ = world.offer_bamboo_block(cursor, states.top_large, 2);
        cursor = offset(cursor, 0, -1, 0)?;
        let _ = world.offer_bamboo_block(cursor, states.top_small, 2);
    }
    Ok(true)
}

fn place_podzol_disk<W: BambooWorld>(
    world: &mut W,
    origin: BlockPos,
    radius: i32,
    podzol: BlockStateId,
) -> Result<(), BambooFeatureError> {
    for x_offset in -radius..=radius {
        for z_offset in -radius..=radius {
            if x_offset * x_offset + z_offset * z_offset > radius * radius {
                continue;
            }
            let column = offset(origin, x_offset, 0, z_offset)?;
            let surface_y = world
                .world_surface_height(column.x, column.z)
                .checked_sub(1)
                .ok_or(BambooFeatureError::PositionOverflow)?;
            let surface = BlockPos::new(column.x, surface_y, column.z);
            let state = world.block_state(surface);
            if world.beneath_bamboo_podzol_replaceable(state) {
                let _ = world.offer_bamboo_block(surface, podzol, 2);
            }
        }
    }
    Ok(())
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, BambooFeatureError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(BambooFeatureError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(BambooFeatureError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(BambooFeatureError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BambooFeatureError {
    #[error("bamboo position arithmetic overflowed")]
    PositionOverflow,
    #[error("bamboo podzol probability must be finite and in the inclusive range 0..=1")]
    InvalidProbability,
}
