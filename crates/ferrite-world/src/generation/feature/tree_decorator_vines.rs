//! Trunk-vine, leaf-vine, and pale-moss tree decorators.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use crate::id::BlockStateId;

pub trait VineDecoratorWorld: TreeWorld {
    fn vine_with_face(&self, face: Direction) -> BlockStateId;

    fn pale_hanging_moss(&self, tip: bool) -> BlockStateId;

    fn try_place_registered_pale_moss_patch(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    );
}

pub fn decorate_trunk_vines<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
) -> Result<(), TreeDecoratorError>
where
    R: GenerationRandom,
    W: VineDecoratorWorld,
{
    for log in context.ordered_trunks() {
        for (direction, face) in vine_directions() {
            if bounded(random, 3) > 0 {
                let candidate = offset(log, direction)?;
                if is_air(context.world(), candidate) {
                    let vine = context.world().vine_with_face(face);
                    context.offer_decorator(candidate, vine);
                }
            }
        }
    }
    Ok(())
}

pub fn decorate_leaf_vines<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    probability: f32,
    random: &mut R,
) -> Result<(), TreeDecoratorError>
where
    R: GenerationRandom,
    W: VineDecoratorWorld,
{
    validate_probability(probability)?;
    for leaf in context.ordered_foliage() {
        for (direction, face) in vine_directions() {
            if random.next_f32() < probability {
                let candidate = offset(leaf, direction)?;
                if is_air(context.world(), candidate) {
                    add_hanging_vine(context, candidate, face)?;
                }
            }
        }
    }
    Ok(())
}

pub fn decorate_pale_moss<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    leaves_probability: f32,
    trunk_probability: f32,
    ground_probability: f32,
    random: &mut R,
) -> Result<(), TreeDecoratorError>
where
    R: GenerationRandom,
    W: VineDecoratorWorld,
{
    for probability in [leaves_probability, trunk_probability, ground_probability] {
        validate_probability(probability)?;
    }
    let logs = context.ordered_trunks();
    let mut shuffled = logs.clone();
    shuffle(&mut shuffled, random);
    let Some(origin) = shuffled.iter().min_by_key(|position| position.y).copied() else {
        return Ok(());
    };
    if random.next_f32() < ground_probability {
        let above = offset(origin, Direction::Up)?;
        context
            .world()
            .try_place_registered_pale_moss_patch(above, random);
    }
    for position in logs {
        maybe_add_moss_hanger(context, position, trunk_probability, random)?;
    }
    for position in context.ordered_foliage() {
        maybe_add_moss_hanger(context, position, leaves_probability, random)?;
    }
    Ok(())
}

fn maybe_add_moss_hanger<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    position: BlockPos,
    probability: f32,
    random: &mut R,
) -> Result<(), TreeDecoratorError>
where
    R: GenerationRandom,
    W: VineDecoratorWorld,
{
    if random.next_f32() < probability {
        let below = offset(position, Direction::Down)?;
        if is_air(context.world(), below) {
            add_moss_hanger(context, below, random)?;
        }
    }
    Ok(())
}

fn add_hanging_vine<W: VineDecoratorWorld>(
    context: &mut TreePlacementContext<'_, W>,
    mut position: BlockPos,
    face: Direction,
) -> Result<(), TreeDecoratorError> {
    let vine = context.world().vine_with_face(face);
    context.offer_decorator(position, vine);
    position = offset(position, Direction::Down)?;
    let mut remaining = 4;
    while is_air(context.world(), position) && remaining > 0 {
        let vine = context.world().vine_with_face(face);
        context.offer_decorator(position, vine);
        position = offset(position, Direction::Down)?;
        remaining -= 1;
    }
    Ok(())
}

fn add_moss_hanger<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    mut position: BlockPos,
    random: &mut R,
) -> Result<(), TreeDecoratorError>
where
    R: GenerationRandom,
    W: VineDecoratorWorld,
{
    loop {
        let below = offset(position, Direction::Down)?;
        if !is_air(context.world(), below) || random.next_f32() < 0.5 {
            let tip = context.world().pale_hanging_moss(true);
            context.offer_decorator(position, tip);
            return Ok(());
        }
        let body = context.world().pale_hanging_moss(false);
        context.offer_decorator(position, body);
        position = below;
    }
}

fn vine_directions() -> [(Direction, Direction); 4] {
    [
        (Direction::West, Direction::East),
        (Direction::East, Direction::West),
        (Direction::North, Direction::South),
        (Direction::South, Direction::North),
    ]
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

fn validate_probability(probability: f32) -> Result<(), TreeDecoratorError> {
    if (0.0..=1.0).contains(&probability) {
        Ok(())
    } else {
        Err(TreeDecoratorError::InvalidProbability)
    }
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, TreeDecoratorError> {
    let [x, y, z] = direction.step();
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(TreeDecoratorError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(TreeDecoratorError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(TreeDecoratorError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TreeDecoratorError {
    #[error("tree-decorator probability is outside [0, 1]")]
    InvalidProbability,
    #[error("tree-decorator position overflow")]
    PositionOverflow,
}
