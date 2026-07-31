//! Direct configured-feature writes with exact read, test, and offer ordering.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

pub const DIRECT_WRITE_FLAGS: u32 = 2;
pub const MAXIMUM_FILL_LAYER_HEIGHT: u32 = 4_064;

pub trait DirectWriteWorld {
    fn minimum_y(&self) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn offer_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub trait ReplacementRule<R: GenerationRandom> {
    fn test(&self, state: BlockStateId, random: &mut R) -> bool;
}

pub struct ReplacementTarget<'a, R: GenerationRandom> {
    pub rule: &'a dyn ReplacementRule<R>,
    pub state: BlockStateId,
}

pub fn replace_single_block<R, W>(
    world: &mut W,
    origin: BlockPos,
    random: &mut R,
    targets: &[ReplacementTarget<'_, R>],
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> bool
where
    R: GenerationRandom,
    W: DirectWriteWorld,
{
    if !ensure_can_write(origin) {
        return false;
    }
    for target in targets {
        let existing = world.block_state(origin);
        if target.rule.test(existing, random) {
            let _ = world.offer_block(origin, target.state, DIRECT_WRITE_FLAGS);
            break;
        }
    }
    true
}

pub fn fill_layer<W: DirectWriteWorld>(
    world: &mut W,
    origin: BlockPos,
    height: u32,
    state: BlockStateId,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, DirectWriteError> {
    if height > MAXIMUM_FILL_LAYER_HEIGHT {
        return Err(DirectWriteError::HeightOutOfRange { height });
    }
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let y = world
        .minimum_y()
        .checked_add(height as i32)
        .ok_or(DirectWriteError::PositionOverflow)?;
    for x_offset in 0..16 {
        let x = origin
            .x
            .checked_add(x_offset)
            .ok_or(DirectWriteError::PositionOverflow)?;
        for z_offset in 0..16 {
            let z = origin
                .z
                .checked_add(z_offset)
                .ok_or(DirectWriteError::PositionOverflow)?;
            let position = BlockPos::new(x, y, z);
            let existing = world.block_state(position);
            if world.is_air(existing) {
                let _ = world.offer_block(position, state, DIRECT_WRITE_FLAGS);
            }
        }
    }
    Ok(true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DirectWriteError {
    #[error("fill-layer height {height} exceeds {MAXIMUM_FILL_LAYER_HEIGHT}")]
    HeightOutOfRange { height: u32 },
    #[error("direct-write position arithmetic overflowed")]
    PositionOverflow,
}
