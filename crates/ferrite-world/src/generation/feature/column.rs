//! Layered block-column placement with prewrite lookahead and deterministic truncation.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockColumnConfig {
    pub layer_heights: Vec<IntProvider>,
    pub direction: Direction,
    pub prioritize_tip: bool,
}

pub trait BlockColumnWorld<R: GenerationRandom> {
    fn allowed_placement(&mut self, position: BlockPos) -> bool;

    fn provide_layer_state(
        &mut self,
        layer_index: usize,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn offer_column_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_block_column<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &BlockColumnConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BlockColumnError>
where
    R: GenerationRandom,
    W: BlockColumnWorld<R>,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let mut heights = Vec::with_capacity(config.layer_heights.len());
    let mut total = 0_i32;
    for provider in &config.layer_heights {
        let height = provider.sample(random)?;
        if height < 0 {
            return Err(BlockColumnError::NegativeLayerHeight { height });
        }
        heights.push(height);
        total = total.wrapping_add(height);
    }
    if total == 0 {
        return Ok(false);
    }

    if total > 0 {
        let step = config.direction.step();
        let mut predicate_cursor = offset(origin, step)?;
        let mut failure_capacity = None;
        for index in 0..total {
            if !world.allowed_placement(predicate_cursor) {
                failure_capacity = Some(index);
                break;
            }
            predicate_cursor = offset(predicate_cursor, step)?;
        }
        if let Some(capacity) = failure_capacity {
            truncate_heights(&mut heights, total - capacity, config.prioritize_tip);
        }
    }

    let step = config.direction.step();
    let mut write_cursor = origin;
    for (layer_index, height) in heights.into_iter().enumerate() {
        for _ in 0..height {
            let state = world.provide_layer_state(layer_index, write_cursor, random);
            let _ = world.offer_column_block(write_cursor, state, 2);
            write_cursor = offset(write_cursor, step)?;
        }
    }
    Ok(true)
}

fn truncate_heights(heights: &mut [i32], mut excess: i32, prioritize_tip: bool) {
    if prioritize_tip {
        for height in heights {
            let removed = (*height).min(excess);
            *height -= removed;
            excess -= removed;
            if excess == 0 {
                break;
            }
        }
    } else {
        for height in heights.iter_mut().rev() {
            let removed = (*height).min(excess);
            *height -= removed;
            excess -= removed;
            if excess == 0 {
                break;
            }
        }
    }
}

fn offset(origin: BlockPos, step: [i32; 3]) -> Result<BlockPos, BlockColumnError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(step[0])
            .ok_or(BlockColumnError::PositionOverflow)?,
        origin
            .y
            .checked_add(step[1])
            .ok_or(BlockColumnError::PositionOverflow)?,
        origin
            .z
            .checked_add(step[2])
            .ok_or(BlockColumnError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BlockColumnError {
    #[error("block-column integer provider failed")]
    Provider(#[from] ProviderError),
    #[error("block-column layer height {height} is negative")]
    NegativeLayerHeight { height: i32 },
    #[error("block-column position arithmetic overflowed")]
    PositionOverflow,
}
