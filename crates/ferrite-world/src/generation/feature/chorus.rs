//! Chorus-plant generation with bounded depth-first branching and connection recomputation.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

pub trait ChorusPlantWorld {
    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn supports_chorus_plant(&self, state: BlockStateId) -> bool;

    fn connected_chorus_plant_state(
        &mut self,
        position: BlockPos,
        neighbors: [BlockStateId; 6],
    ) -> BlockStateId;

    fn chorus_flower_state(&self, age: u8) -> BlockStateId;

    fn offer_chorus_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_chorus_plant<R, W>(
    world: &mut W,
    origin: BlockPos,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, ChorusFeatureError>
where
    R: GenerationRandom,
    W: ChorusPlantWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    if !world.is_empty_block(origin) {
        return Ok(false);
    }
    let below = offset(origin, Direction::Down)?;
    let support = world.block_state(below);
    if !world.supports_chorus_plant(support) {
        return Ok(false);
    }
    offer_connected_plant(world, origin)?;
    grow_recursive(world, origin, origin, 0, random)?;
    Ok(true)
}

fn grow_recursive<R, W>(
    world: &mut W,
    root: BlockPos,
    base: BlockPos,
    depth: u8,
    random: &mut R,
) -> Result<(), ChorusFeatureError>
where
    R: GenerationRandom,
    W: ChorusPlantWorld,
{
    let segment_length = 1
        + random.next_u32(NonZeroU32::new(4).expect("chorus segment bound is nonzero")) as i32
        + i32::from(depth == 0);
    let mut top = base;
    for _ in 0..segment_length {
        top = offset(top, Direction::Up)?;
        if !all_horizontal_empty(world, top, None)? {
            return Ok(());
        }
        offer_connected_plant(world, top)?;
        let below = offset(top, Direction::Down)?;
        offer_connected_plant(world, below)?;
    }

    let mut branched = false;
    if depth < 4 {
        let attempts = random
            .next_u32(NonZeroU32::new(4).expect("chorus branch-count bound is nonzero"))
            + u32::from(depth == 0);
        for _ in 0..attempts {
            let direction = horizontal_direction(random);
            let candidate = offset(top, direction)?;
            if candidate.x.abs_diff(root.x) >= 8
                || candidate.z.abs_diff(root.z) >= 8
                || !world.is_empty_block(candidate)
            {
                continue;
            }
            let below = offset(candidate, Direction::Down)?;
            if !world.is_empty_block(below)
                || !all_horizontal_empty(world, candidate, Some(direction.opposite()))?
            {
                continue;
            }
            branched = true;
            offer_connected_plant(world, candidate)?;
            offer_connected_plant(world, top)?;
            grow_recursive(world, root, candidate, depth + 1, random)?;
        }
    }
    if !branched {
        let flower = world.chorus_flower_state(5);
        let _ = world.offer_chorus_block(top, flower, 2);
    }
    Ok(())
}

fn all_horizontal_empty<W: ChorusPlantWorld>(
    world: &mut W,
    position: BlockPos,
    excluded: Option<Direction>,
) -> Result<bool, ChorusFeatureError> {
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        if excluded == Some(direction) {
            continue;
        }
        let neighbor = offset(position, direction)?;
        if !world.is_empty_block(neighbor) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn offer_connected_plant<W: ChorusPlantWorld>(
    world: &mut W,
    position: BlockPos,
) -> Result<(), ChorusFeatureError> {
    let neighbors = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ]
    .map(|direction| offset(position, direction).map(|neighbor| world.block_state(neighbor)));
    let mut states = [BlockStateId::new(0); 6];
    for (index, state) in neighbors.into_iter().enumerate() {
        states[index] = state?;
    }
    let plant = world.connected_chorus_plant_state(position, states);
    let _ = world.offer_chorus_block(position, plant, 2);
    Ok(())
}

fn horizontal_direction(random: &mut impl GenerationRandom) -> Direction {
    let index = random.next_u32(NonZeroU32::new(4).expect("horizontal direction bound is nonzero"));
    [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ][index as usize]
}

fn offset(origin: BlockPos, direction: Direction) -> Result<BlockPos, ChorusFeatureError> {
    let [x, y, z] = direction.step();
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(ChorusFeatureError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(ChorusFeatureError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(ChorusFeatureError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ChorusFeatureError {
    #[error("chorus position arithmetic overflowed")]
    PositionOverflow,
}
