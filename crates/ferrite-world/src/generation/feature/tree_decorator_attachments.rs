//! Attached-to-leaves and attached-to-logs decorators.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::java_hash_set::JavaBlockPosSet;
use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use crate::id::BlockStateId;

#[derive(Debug, Clone, PartialEq)]
pub struct AttachedToLeavesConfig {
    pub probability: f32,
    pub exclusion_radius_xz: i32,
    pub exclusion_radius_y: i32,
    pub required_empty_blocks: i32,
    pub directions: Vec<Direction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttachedToLogsConfig {
    pub probability: f32,
    pub directions: Vec<Direction>,
}

pub trait AttachmentDecoratorWorld: TreeWorld {
    fn sample_attached_to_leaves(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> BlockStateId;

    fn sample_attached_to_logs(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> BlockStateId;
}

pub fn decorate_attached_to_leaves<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    config: &AttachedToLeavesConfig,
    random: &mut R,
) -> Result<(), AttachmentDecoratorError>
where
    R: GenerationRandom,
    W: AttachmentDecoratorWorld,
{
    validate_leaves_config(config)?;
    let mut exclusion = JavaBlockPosSet::new();
    let mut leaves = context.ordered_foliage();
    shuffle(&mut leaves, random);
    let direction_bound =
        NonZeroU32::new(config.directions.len() as u32).expect("validated directions are nonempty");
    for leaf in leaves {
        let direction = config.directions[random.next_u32(direction_bound) as usize];
        let candidate = move_by(leaf, direction, 1)?;
        if exclusion.contains(&candidate) || random.next_f32() >= config.probability {
            continue;
        }
        let mut clear = true;
        for distance in 1..=config.required_empty_blocks {
            let position = move_by(leaf, direction, distance)?;
            if !is_air(context.world(), position) {
                clear = false;
                break;
            }
        }
        if !clear {
            continue;
        }
        reserve_cuboid(&mut exclusion, candidate, config)?;
        let state = context.world().sample_attached_to_leaves(candidate, random);
        context.offer_decorator(candidate, state);
    }
    Ok(())
}

pub fn decorate_attached_to_logs<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    config: &AttachedToLogsConfig,
    random: &mut R,
) -> Result<(), AttachmentDecoratorError>
where
    R: GenerationRandom,
    W: AttachmentDecoratorWorld,
{
    validate_probability_and_directions(config.probability, &config.directions)?;
    let mut logs = context.ordered_trunks();
    shuffle(&mut logs, random);
    let direction_bound =
        NonZeroU32::new(config.directions.len() as u32).expect("validated directions are nonempty");
    for log in logs {
        let direction = config.directions[random.next_u32(direction_bound) as usize];
        if random.next_f32() > config.probability {
            continue;
        }
        let candidate = move_by(log, direction, 1)?;
        if !is_air(context.world(), candidate) {
            continue;
        }
        let state = context.world().sample_attached_to_logs(candidate, random);
        context.offer_decorator(candidate, state);
    }
    Ok(())
}

fn reserve_cuboid(
    exclusion: &mut JavaBlockPosSet,
    center: BlockPos,
    config: &AttachedToLeavesConfig,
) -> Result<(), AttachmentDecoratorError> {
    for x in -config.exclusion_radius_xz..=config.exclusion_radius_xz {
        for y in -config.exclusion_radius_y..=config.exclusion_radius_y {
            for z in -config.exclusion_radius_xz..=config.exclusion_radius_xz {
                exclusion.insert(offset_xyz(center, x, y, z)?);
            }
        }
    }
    Ok(())
}

fn validate_leaves_config(config: &AttachedToLeavesConfig) -> Result<(), AttachmentDecoratorError> {
    validate_probability_and_directions(config.probability, &config.directions)?;
    if !(0..=16).contains(&config.exclusion_radius_xz)
        || !(0..=16).contains(&config.exclusion_radius_y)
        || !(1..=16).contains(&config.required_empty_blocks)
    {
        Err(AttachmentDecoratorError::InvalidConfiguration)
    } else {
        Ok(())
    }
}

fn validate_probability_and_directions(
    probability: f32,
    directions: &[Direction],
) -> Result<(), AttachmentDecoratorError> {
    if !(0.0..=1.0).contains(&probability) || directions.is_empty() {
        Err(AttachmentDecoratorError::InvalidConfiguration)
    } else {
        Ok(())
    }
}

fn shuffle<T>(values: &mut [T], random: &mut impl GenerationRandom) {
    for length in (2..=values.len()).rev() {
        let bound = NonZeroU32::new(length as u32).expect("shuffle bound is nonzero");
        let index = random.next_u32(bound) as usize;
        values.swap(length - 1, index);
    }
}

fn is_air<W: TreeWorld>(world: &mut W, position: BlockPos) -> bool {
    let state = world.block_state(position);
    world.is_air(state)
}

fn move_by(
    position: BlockPos,
    direction: Direction,
    distance: i32,
) -> Result<BlockPos, AttachmentDecoratorError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x * distance, y * distance, z * distance)
}

fn offset_xyz(
    position: BlockPos,
    x: i32,
    y: i32,
    z: i32,
) -> Result<BlockPos, AttachmentDecoratorError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(AttachmentDecoratorError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(AttachmentDecoratorError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(AttachmentDecoratorError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AttachmentDecoratorError {
    #[error("attachment-decorator configuration violates codec bounds")]
    InvalidConfiguration,
    #[error("attachment-decorator position overflow")]
    PositionOverflow,
}
