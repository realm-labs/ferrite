//! Cocoa, creaking-heart, and beehive tree decorators.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use crate::id::BlockStateId;

pub trait LogDecoratorWorld: TreeWorld {
    fn cocoa_state(&self, facing: Direction, age: u8) -> BlockStateId;

    fn creaking_heart_state(&self) -> BlockStateId;

    fn belongs_to_logs_tag(&self, state: BlockStateId) -> bool;

    fn bee_nest_facing_south(&self) -> BlockStateId;

    fn has_beehive_block_entity(&mut self, position: BlockPos) -> bool;

    fn store_bee_occupant(&mut self, position: BlockPos, ticks_in_hive: u32, minimum_ticks: u32);
}

pub fn decorate_cocoa<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    probability: f32,
    random: &mut R,
) -> Result<(), LogDecoratorError>
where
    R: GenerationRandom,
    W: LogDecoratorWorld,
{
    validate_probability(probability)?;
    if random.next_f32() >= probability {
        return Ok(());
    }
    let logs = context.ordered_trunks();
    let Some(base_y) = logs.first().map(|position| position.y) else {
        return Ok(());
    };
    for log in logs {
        if log.y - base_y > 2 {
            continue;
        }
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            if random.next_f32() <= 0.25 {
                let candidate = offset(log, direction.opposite())?;
                if is_air(context.world(), candidate) {
                    let age = bounded(random, 3) as u8;
                    let cocoa = context.world().cocoa_state(direction, age);
                    context.offer_decorator(candidate, cocoa);
                }
            }
        }
    }
    Ok(())
}

pub fn decorate_creaking_heart<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    probability: f32,
    random: &mut R,
) -> Result<(), LogDecoratorError>
where
    R: GenerationRandom,
    W: LogDecoratorWorld,
{
    validate_probability(probability)?;
    let mut logs = context.ordered_trunks();
    if logs.is_empty() || random.next_f32() >= probability {
        return Ok(());
    }
    shuffle(&mut logs, random);
    for candidate in logs {
        let mut surrounded = true;
        for direction in Direction::ALL {
            let neighbor = offset(candidate, direction)?;
            let state = context.world().block_state(neighbor);
            if !context.world().belongs_to_logs_tag(state) {
                surrounded = false;
                break;
            }
        }
        if surrounded {
            let heart = context.world().creaking_heart_state();
            context.offer_decorator(candidate, heart);
            break;
        }
    }
    Ok(())
}

pub fn decorate_beehive<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    probability: f32,
    random: &mut R,
) -> Result<(), LogDecoratorError>
where
    R: GenerationRandom,
    W: LogDecoratorWorld,
{
    validate_probability(probability)?;
    let logs = context.ordered_trunks();
    if logs.is_empty() || random.next_f32() >= probability {
        return Ok(());
    }
    let leaves = context.ordered_foliage();
    let first_log_y = logs[0].y;
    let target_y = if let Some(first_leaf) = leaves.first() {
        (first_leaf.y - 1).max(first_log_y + 1)
    } else {
        (first_log_y + 1 + bounded(random, 3) as i32)
            .min(logs.last().expect("nonempty logs have a last element").y)
    };
    let mut candidates = Vec::new();
    for log in logs.into_iter().filter(|position| position.y == target_y) {
        for direction in [Direction::East, Direction::South, Direction::West] {
            candidates.push(offset(log, direction)?);
        }
    }
    if candidates.is_empty() {
        return Ok(());
    }
    shuffle(&mut candidates, random);
    let mut winner = None;
    for candidate in candidates {
        if !is_air(context.world(), candidate) {
            continue;
        }
        let entrance = offset(candidate, Direction::South)?;
        if is_air(context.world(), entrance) {
            winner = Some(candidate);
            break;
        }
    }
    let Some(position) = winner else {
        return Ok(());
    };
    let nest = context.world().bee_nest_facing_south();
    context.offer_decorator(position, nest);
    if !context.world().has_beehive_block_entity(position) {
        return Ok(());
    }
    let occupants = 2 + bounded(random, 2);
    for _ in 0..occupants {
        let ticks = bounded(random, 599);
        context.world().store_bee_occupant(position, ticks, 600);
    }
    Ok(())
}

fn shuffle<T>(values: &mut [T], random: &mut impl GenerationRandom) {
    for length in (2..=values.len()).rev() {
        let index = bounded(random, length as u32) as usize;
        values.swap(length - 1, index);
    }
}

fn is_air<W: TreeWorld>(world: &mut W, position: BlockPos) -> bool {
    let state = world.block_state(position);
    world.is_air(state)
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("decorator bound is nonzero"))
}

fn validate_probability(probability: f32) -> Result<(), LogDecoratorError> {
    if (0.0..=1.0).contains(&probability) {
        Ok(())
    } else {
        Err(LogDecoratorError::InvalidProbability)
    }
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, LogDecoratorError> {
    let [x, y, z] = direction.step();
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(LogDecoratorError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(LogDecoratorError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(LogDecoratorError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LogDecoratorError {
    #[error("tree-decorator probability is outside [0, 1]")]
    InvalidProbability,
    #[error("tree-decorator position overflow")]
    PositionOverflow,
}
