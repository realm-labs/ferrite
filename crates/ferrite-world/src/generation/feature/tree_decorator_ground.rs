//! Alter-ground and place-on-ground tree decorators.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceOnGroundConfig {
    pub tries: u32,
    pub radius: i32,
    pub height: i32,
}

pub trait GroundDecoratorWorld: TreeWorld {
    fn sample_optional_altered_ground(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> Option<BlockStateId>;

    fn is_solid_render(&self, state: BlockStateId) -> bool;

    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32;

    fn sample_ground_decoration(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> BlockStateId;
}

pub fn decorate_alter_ground<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
) -> Result<(), GroundDecoratorError>
where
    R: GenerationRandom,
    W: GroundDecoratorWorld,
{
    let positions = context.lowest_trunk_or_root();
    let Some(minimum_y) = positions.first().map(|position| position.y) else {
        return Ok(());
    };
    for position in positions
        .into_iter()
        .filter(|position| position.y == minimum_y)
    {
        for (x, z) in [(-1, -1), (2, -1), (-1, 2), (2, 2)] {
            place_circle(context, offset_xyz(position, x, 0, z)?, random)?;
        }
        for _ in 0..5 {
            let draw = bounded(random, 64) as i32;
            let x = draw % 8;
            let z = draw / 8;
            if x == 0 || x == 7 || z == 0 || z == 7 {
                place_circle(context, offset_xyz(position, x - 3, 0, z - 3)?, random)?;
            }
        }
    }
    Ok(())
}

pub fn decorate_place_on_ground<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    config: PlaceOnGroundConfig,
    random: &mut R,
) -> Result<(), GroundDecoratorError>
where
    R: GenerationRandom,
    W: GroundDecoratorWorld,
{
    if config.tries == 0 || config.radius < 0 || config.height < 0 {
        return Err(GroundDecoratorError::InvalidConfiguration);
    }
    let positions = context.lowest_trunk_or_root();
    let Some(first) = positions.first().copied() else {
        return Ok(());
    };
    let mut minimum_x = first.x;
    let mut maximum_x = first.x;
    let mut minimum_z = first.z;
    let mut maximum_z = first.z;
    for position in positions
        .into_iter()
        .filter(|position| position.y == first.y)
    {
        minimum_x = minimum_x.min(position.x);
        maximum_x = maximum_x.max(position.x);
        minimum_z = minimum_z.min(position.z);
        maximum_z = maximum_z.max(position.z);
    }
    minimum_x = minimum_x
        .checked_sub(config.radius)
        .ok_or(GroundDecoratorError::PositionOverflow)?;
    maximum_x = maximum_x
        .checked_add(config.radius)
        .ok_or(GroundDecoratorError::PositionOverflow)?;
    minimum_z = minimum_z
        .checked_sub(config.radius)
        .ok_or(GroundDecoratorError::PositionOverflow)?;
    maximum_z = maximum_z
        .checked_add(config.radius)
        .ok_or(GroundDecoratorError::PositionOverflow)?;
    let minimum_y = first
        .y
        .checked_sub(config.height)
        .ok_or(GroundDecoratorError::PositionOverflow)?;
    let maximum_y = first
        .y
        .checked_add(config.height)
        .ok_or(GroundDecoratorError::PositionOverflow)?;

    for _ in 0..config.tries {
        let position = BlockPos::new(
            inclusive(random, minimum_x, maximum_x)?,
            inclusive(random, minimum_y, maximum_y)?,
            inclusive(random, minimum_z, maximum_z)?,
        );
        attempt_place_above(context, position, random)?;
    }
    Ok(())
}

fn place_circle<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    center: BlockPos,
    random: &mut R,
) -> Result<(), GroundDecoratorError>
where
    R: GenerationRandom,
    W: GroundDecoratorWorld,
{
    for x in -2_i32..=2 {
        for z in -2_i32..=2 {
            if x.abs() == 2 && z.abs() == 2 {
                continue;
            }
            place_column(context, offset_xyz(center, x, 0, z)?, random)?;
        }
    }
    Ok(())
}

fn place_column<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    position: BlockPos,
    random: &mut R,
) -> Result<(), GroundDecoratorError>
where
    R: GenerationRandom,
    W: GroundDecoratorWorld,
{
    for y in (-3..=2).rev() {
        let cursor = offset_xyz(position, 0, y, 0)?;
        if let Some(state) = context
            .world()
            .sample_optional_altered_ground(cursor, random)
        {
            context.offer_decorator(cursor, state);
            break;
        }
        let old_state = context.world().block_state(cursor);
        if !context.world().is_air(old_state) && y < 0 {
            break;
        }
    }
    Ok(())
}

fn attempt_place_above<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    position: BlockPos,
    random: &mut R,
) -> Result<(), GroundDecoratorError>
where
    R: GenerationRandom,
    W: GroundDecoratorWorld,
{
    let above = offset_xyz(position, 0, 1, 0)?;
    let above_state = context.world().block_state(above);
    let open = {
        let world = context.world();
        world.is_air(above_state) || world.is_vine(above_state)
    };
    if !open {
        return Ok(());
    }
    let base_state = context.world().block_state(position);
    if !context.world().is_solid_render(base_state) {
        return Ok(());
    }
    if context
        .world()
        .motion_blocking_no_leaves_height(position.x, position.z)
        > above.y
    {
        return Ok(());
    }
    let state = context.world().sample_ground_decoration(above, random);
    context.offer_decorator(above, state);
    Ok(())
}

fn inclusive(
    random: &mut impl GenerationRandom,
    minimum: i32,
    maximum: i32,
) -> Result<i32, GroundDecoratorError> {
    let width = i64::from(maximum) - i64::from(minimum) + 1;
    let bound = u32::try_from(width)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(GroundDecoratorError::PositionOverflow)?;
    let draw = random.next_u32(bound);
    i64::from(minimum)
        .checked_add(i64::from(draw))
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(GroundDecoratorError::PositionOverflow)
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("ground-decorator bound is nonzero"))
}

fn offset_xyz(
    position: BlockPos,
    x: i32,
    y: i32,
    z: i32,
) -> Result<BlockPos, GroundDecoratorError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(GroundDecoratorError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(GroundDecoratorError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(GroundDecoratorError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GroundDecoratorError {
    #[error("ground-decorator configuration violates codec bounds")]
    InvalidConfiguration,
    #[error("ground-decorator position overflow")]
    PositionOverflow,
}
