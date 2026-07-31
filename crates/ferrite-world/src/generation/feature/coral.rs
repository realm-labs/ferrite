//! Warm-ocean coral feature family with shared cell decoration semantics.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralShape {
    Claw,
    Mushroom,
    Tree,
}

pub trait CoralWorld {
    fn coral_blocks(&self) -> &[BlockStateId];

    fn corals(&self) -> &[BlockStateId];

    fn wall_corals(&self) -> &[BlockStateId];

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_exact_water(&self, state: BlockStateId) -> bool;

    fn is_coral(&self, state: BlockStateId) -> bool;

    fn sea_pickle_state(&self, count: u8) -> BlockStateId;

    fn wall_coral_facing(&self, state: BlockStateId, direction: Direction) -> BlockStateId;

    fn offer_coral_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_coral_feature<R, W>(
    world: &mut W,
    origin: BlockPos,
    shape: CoralShape,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, CoralFeatureError>
where
    R: GenerationRandom,
    W: CoralWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let Some(coral) = choose_tag_member(world.coral_blocks(), random) else {
        return Ok(false);
    };
    match shape {
        CoralShape::Claw => place_claw(world, origin, coral, random),
        CoralShape::Mushroom => place_mushroom(world, origin, coral, random),
        CoralShape::Tree => place_tree(world, origin, coral, random),
    }
}

fn place_coral_cell<R, W>(
    world: &mut W,
    position: BlockPos,
    coral: BlockStateId,
    random: &mut R,
) -> Result<bool, CoralFeatureError>
where
    R: GenerationRandom,
    W: CoralWorld,
{
    let current = world.block_state(position);
    if !world.is_exact_water(current) && !world.is_coral(current) {
        return Ok(false);
    }
    let above = offset(position, Direction::Up)?;
    let above_state = world.block_state(above);
    if !world.is_exact_water(above_state) {
        return Ok(false);
    }
    let _ = world.offer_coral_block(position, coral, 3);

    if random.next_f32() < 0.25 {
        if let Some(decoration) = choose_tag_member(world.corals(), random) {
            let _ = world.offer_coral_block(above, decoration, 2);
        }
    } else if random.next_f32() < 0.05 {
        let count = 1 + random
            .next_u32(NonZeroU32::new(4).expect("sea-pickle count bound is nonzero"))
            as u8;
        let pickles = world.sea_pickle_state(count);
        let _ = world.offer_coral_block(above, pickles, 2);
    }

    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        if random.next_f32() >= 0.2 {
            continue;
        }
        let adjacent = offset(position, direction)?;
        let adjacent_state = world.block_state(adjacent);
        if !world.is_exact_water(adjacent_state) {
            continue;
        }
        if let Some(wall) = choose_tag_member(world.wall_corals(), random) {
            let wall = world.wall_coral_facing(wall, direction);
            let _ = world.offer_coral_block(adjacent, wall, 2);
        }
    }
    Ok(true)
}

fn place_claw<R, W>(
    world: &mut W,
    origin: BlockPos,
    coral: BlockStateId,
    random: &mut R,
) -> Result<bool, CoralFeatureError>
where
    R: GenerationRandom,
    W: CoralWorld,
{
    if !place_coral_cell(world, origin, coral, random)? {
        return Ok(false);
    }
    let forward = horizontal_direction(random);
    let arm_count =
        2 + random.next_u32(NonZeroU32::new(2).expect("coral arm-count bound is nonzero"));
    let mut arms = [forward, clockwise(forward), counterclockwise(forward)];
    shuffle(&mut arms, random);
    for arm in arms.into_iter().take(arm_count as usize) {
        let first_length =
            1 + random.next_u32(NonZeroU32::new(2).expect("first-leg bound is nonzero")) as i32;
        let mut cursor = offset(origin, arm)?;
        let (continuation, second_length) = if arm == forward {
            (
                forward,
                2 + random
                    .next_u32(NonZeroU32::new(3).expect("forward second-leg bound is nonzero"))
                    as i32,
            )
        } else {
            cursor = offset(cursor, Direction::Up)?;
            let continuation = if random
                .next_u32(NonZeroU32::new(2).expect("side continuation bound is nonzero"))
                == 0
            {
                arm
            } else {
                Direction::Up
            };
            (
                continuation,
                3 + random.next_u32(NonZeroU32::new(3).expect("side second-leg bound is nonzero"))
                    as i32,
            )
        };
        for _ in 0..first_length {
            if !place_coral_cell(world, cursor, coral, random)? {
                break;
            }
            cursor = offset(cursor, continuation)?;
        }
        cursor = offset(cursor, continuation.opposite())?;
        cursor = offset(cursor, Direction::Up)?;
        for _ in 0..second_length {
            cursor = offset(cursor, arm)?;
            if !place_coral_cell(world, cursor, coral, random)? {
                break;
            }
            if random.next_f32() < 0.25 {
                cursor = offset(cursor, Direction::Up)?;
            }
        }
    }
    Ok(true)
}

fn place_mushroom<R, W>(
    world: &mut W,
    origin: BlockPos,
    coral: BlockStateId,
    random: &mut R,
) -> Result<bool, CoralFeatureError>
where
    R: GenerationRandom,
    W: CoralWorld,
{
    let y_extent = 3 + bounded_three(random);
    let x_extent = 3 + bounded_three(random);
    let z_extent = 3 + bounded_three(random);
    let downward_offset = 1 + bounded_three(random);
    for x in 0..=x_extent {
        for y in 0..=y_extent {
            for z in 0..=z_extent {
                let boundary_count = u8::from(x == 0 || x == x_extent)
                    + u8::from(y == 0 || y == y_extent)
                    + u8::from(z == 0 || z == z_extent);
                if boundary_count != 1 {
                    continue;
                }
                if random.next_f32() < 0.1 {
                    continue;
                }
                let position = offset_xyz(origin, x, y - downward_offset, z)?;
                let _ = place_coral_cell(world, position, coral, random)?;
            }
        }
    }
    Ok(true)
}

fn place_tree<R, W>(
    world: &mut W,
    origin: BlockPos,
    coral: BlockStateId,
    random: &mut R,
) -> Result<bool, CoralFeatureError>
where
    R: GenerationRandom,
    W: CoralWorld,
{
    let trunk_height = 1 + bounded_three(random);
    let mut branch_base = origin;
    for _ in 0..trunk_height {
        if !place_coral_cell(world, branch_base, coral, random)? {
            return Ok(true);
        }
        branch_base = offset(branch_base, Direction::Up)?;
    }
    let branch_count = 2 + bounded_three(random);
    let mut directions = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
    shuffle(&mut directions, random);
    for direction in directions.into_iter().take(branch_count as usize) {
        let mut cursor = offset(branch_base, direction)?;
        let length = 2 + random
            .next_u32(NonZeroU32::new(5).expect("coral-tree branch-length bound is nonzero"))
            as i32;
        let mut successes_since_horizontal = 0_i32;
        for iteration in 0..length {
            if !place_coral_cell(world, cursor, coral, random)? {
                break;
            }
            cursor = offset(cursor, Direction::Up)?;
            if iteration == 0 {
                cursor = offset(cursor, direction)?;
                successes_since_horizontal = 0;
            } else {
                successes_since_horizontal += 1;
                if successes_since_horizontal >= 2 && random.next_f32() < 0.25 {
                    cursor = offset(cursor, direction)?;
                    successes_since_horizontal = 0;
                }
            }
        }
    }
    Ok(true)
}

fn choose_tag_member(
    entries: &[BlockStateId],
    random: &mut impl GenerationRandom,
) -> Option<BlockStateId> {
    let bound = u32::try_from(entries.len())
        .ok()
        .and_then(NonZeroU32::new)?;
    entries.get(random.next_u32(bound) as usize).copied()
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

fn shuffle<const N: usize>(entries: &mut [Direction; N], random: &mut impl GenerationRandom) {
    for remaining in (2..=N).rev() {
        let bound = NonZeroU32::new(remaining as u32).expect("shuffle bound is nonzero");
        let index = random.next_u32(bound) as usize;
        entries.swap(remaining - 1, index);
    }
}

fn bounded_three(random: &mut impl GenerationRandom) -> i32 {
    random.next_u32(NonZeroU32::new(3).expect("bound three is nonzero")) as i32
}

const fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        Direction::Down | Direction::Up => direction,
    }
}

const fn counterclockwise(direction: Direction) -> Direction {
    clockwise(clockwise(clockwise(direction)))
}

fn offset(origin: BlockPos, direction: Direction) -> Result<BlockPos, CoralFeatureError> {
    let [x, y, z] = direction.step();
    offset_xyz(origin, x, y, z)
}

fn offset_xyz(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, CoralFeatureError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(CoralFeatureError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(CoralFeatureError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(CoralFeatureError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CoralFeatureError {
    #[error("coral-feature position arithmetic overflowed")]
    PositionOverflow,
}
