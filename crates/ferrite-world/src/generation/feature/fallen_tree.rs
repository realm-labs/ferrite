//! Fallen-tree stump, horizontal-log, and locked decorator algorithms.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::{Axis, Direction};
use thiserror::Error;

use crate::generation::feature::java_hash_set::JavaBlockPosSet;
use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, PartialEq)]
pub enum FallenTreeDecorator {
    TrunkVine,
    AttachedToLogs {
        probability: f32,
        directions: Vec<Direction>,
        provider: Vec<WeightedBlockState>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightedBlockState {
    pub state: BlockStateId,
    pub weight: NonZeroU32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallenTreeConfig {
    pub log_length: IntProvider,
    pub stump_decorators: Vec<FallenTreeDecorator>,
    pub log_decorators: Vec<FallenTreeDecorator>,
}

pub trait FallenTreeWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_replaceable_by_trees(&self, state: BlockStateId) -> bool;

    fn is_upward_face_sturdy_at(
        &mut self,
        below_position: BlockPos,
        below_state: BlockStateId,
        queried_position: BlockPos,
    ) -> bool;

    fn sample_trunk<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn with_log_axis(&self, state: BlockStateId, axis: Axis) -> BlockStateId;

    fn vine_with_face(&self, face: Direction) -> BlockStateId;

    fn offer_fallen_tree(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn mark_for_postprocessing(&mut self, position: BlockPos);
}

pub fn place_fallen_tree<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &FallenTreeConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, FallenTreeError>
where
    R: GenerationRandom,
    W: FallenTreeWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    validate_decorators(config)?;
    let stump = world.sample_trunk(origin, random);
    let _ = world.offer_fallen_tree(origin, stump, 3);
    mark_above(world, origin)?;
    let mut stump_logs = JavaBlockPosSet::new();
    stump_logs.insert(origin);
    run_decorators(world, &stump_logs, &config.stump_decorators, random)?;

    let direction = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ][random.next_u32(NonZeroU32::new(4).expect("fallen direction bound is nonzero")) as usize];
    let length = config
        .log_length
        .sample(random)?
        .checked_sub(2)
        .ok_or(FallenTreeError::LengthOverflow)?;
    let start_distance =
        2 + random.next_u32(NonZeroU32::new(2).expect("fallen start bound is nonzero")) as i32;
    let offset_start = offset_n(origin, direction, start_distance)?;
    let mut start = offset(offset_start, Direction::Up)?;
    for _ in 0..6 {
        let state = world.block_state(start);
        if is_open(world, state) {
            let below = offset(start, Direction::Down)?;
            let below_state = world.block_state(below);
            if world.is_upward_face_sturdy_at(below, below_state, start) {
                break;
            }
        }
        start = offset(start, Direction::Down)?;
    }

    if !validate_log_run(world, start, direction, length)? {
        return Ok(true);
    }
    let mut logs = JavaBlockPosSet::new();
    let mut cursor = start;
    for _ in 0..length {
        let trunk = world.sample_trunk(cursor, random);
        let trunk = world.with_log_axis(trunk, direction.axis());
        let _ = world.offer_fallen_tree(cursor, trunk, 3);
        mark_above(world, cursor)?;
        logs.insert(cursor);
        cursor = offset(cursor, direction)?;
    }
    run_decorators(world, &logs, &config.log_decorators, random)?;
    Ok(true)
}

fn validate_log_run<W: FallenTreeWorld>(
    world: &mut W,
    start: BlockPos,
    direction: Direction,
    length: i32,
) -> Result<bool, FallenTreeError> {
    let mut cursor = start;
    let mut unsupported = 0_u8;
    for _ in 0..length {
        let state = world.block_state(cursor);
        if !is_open(world, state) {
            return Ok(false);
        }
        let below = offset(cursor, Direction::Down)?;
        let below_state = world.block_state(below);
        if world.is_upward_face_sturdy_at(below, below_state, cursor) {
            unsupported = 0;
        } else {
            unsupported += 1;
            if unsupported >= 3 {
                return Ok(false);
            }
        }
        cursor = offset(cursor, direction)?;
    }
    Ok(true)
}

fn run_decorators<R, W>(
    world: &mut W,
    logs: &JavaBlockPosSet,
    decorators: &[FallenTreeDecorator],
    random: &mut R,
) -> Result<(), FallenTreeError>
where
    R: GenerationRandom,
    W: FallenTreeWorld,
{
    let mut ordered_logs = logs.iter().collect::<Vec<_>>();
    ordered_logs.sort_by_key(|position| position.y);
    for decorator in decorators {
        match decorator {
            FallenTreeDecorator::TrunkVine => {
                place_trunk_vines(world, &ordered_logs, random)?;
            }
            FallenTreeDecorator::AttachedToLogs {
                probability,
                directions,
                provider,
            } => place_attached(
                world,
                &ordered_logs,
                *probability,
                directions,
                provider,
                random,
            )?,
        }
    }
    Ok(())
}

fn place_trunk_vines(
    world: &mut impl FallenTreeWorld,
    logs: &[BlockPos],
    random: &mut impl GenerationRandom,
) -> Result<(), FallenTreeError> {
    for log in logs.iter().copied() {
        for (candidate_direction, face) in [
            (Direction::West, Direction::East),
            (Direction::East, Direction::West),
            (Direction::North, Direction::South),
            (Direction::South, Direction::North),
        ] {
            if random.next_u32(NonZeroU32::new(3).expect("vine chance bound is nonzero")) == 0 {
                continue;
            }
            let candidate = offset(log, candidate_direction)?;
            let state = world.block_state(candidate);
            if world.is_air(state) {
                let vine = world.vine_with_face(face);
                let _ = world.offer_fallen_tree(candidate, vine, 19);
            }
        }
    }
    Ok(())
}

fn place_attached<R, W>(
    world: &mut W,
    logs: &[BlockPos],
    probability: f32,
    directions: &[Direction],
    provider: &[WeightedBlockState],
    random: &mut R,
) -> Result<(), FallenTreeError>
where
    R: GenerationRandom,
    W: FallenTreeWorld,
{
    let mut shuffled = logs.to_vec();
    shuffle(&mut shuffled, random)?;
    let direction_bound = u32::try_from(directions.len())
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(FallenTreeError::EmptyDirections)?;
    for log in shuffled {
        let direction = directions[random.next_u32(direction_bound) as usize];
        if random.next_f32() > probability {
            continue;
        }
        let candidate = offset(log, direction)?;
        let state = world.block_state(candidate);
        if !world.is_air(state) {
            continue;
        }
        let attached = sample_weighted_state(provider, random)?;
        let _ = world.offer_fallen_tree(candidate, attached, 19);
    }
    Ok(())
}

fn sample_weighted_state(
    provider: &[WeightedBlockState],
    random: &mut impl GenerationRandom,
) -> Result<BlockStateId, FallenTreeError> {
    let total = provider.iter().try_fold(0_u32, |total, entry| {
        total
            .checked_add(entry.weight.get())
            .ok_or(FallenTreeError::WeightOverflow)
    })?;
    let bound = NonZeroU32::new(total).ok_or(FallenTreeError::EmptyProvider)?;
    let draw = random.next_u32(bound);
    let mut cumulative = 0_u32;
    for entry in provider {
        cumulative += entry.weight.get();
        if draw < cumulative {
            return Ok(entry.state);
        }
    }
    unreachable!("bounded block-state draw belongs to one entry")
}

fn shuffle(
    values: &mut [BlockPos],
    random: &mut impl GenerationRandom,
) -> Result<(), FallenTreeError> {
    for remaining in (2..=values.len()).rev() {
        let bound = u32::try_from(remaining)
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or(FallenTreeError::LengthOverflow)?;
        let index = random.next_u32(bound) as usize;
        values.swap(remaining - 1, index);
    }
    Ok(())
}

fn mark_above<W: FallenTreeWorld>(
    world: &mut W,
    position: BlockPos,
) -> Result<(), FallenTreeError> {
    let mut cursor = position;
    for _ in 0..2 {
        cursor = offset(cursor, Direction::Up)?;
        let state = world.block_state(cursor);
        if world.is_air(state) {
            break;
        }
        world.mark_for_postprocessing(cursor);
    }
    Ok(())
}

fn validate_decorators(config: &FallenTreeConfig) -> Result<(), FallenTreeError> {
    for decorator in config.stump_decorators.iter().chain(&config.log_decorators) {
        if let FallenTreeDecorator::AttachedToLogs {
            probability,
            directions,
            provider,
        } = decorator
        {
            if !probability.is_finite() || !(0.0..=1.0).contains(probability) {
                return Err(FallenTreeError::InvalidProbability);
            }
            if directions.is_empty() {
                return Err(FallenTreeError::EmptyDirections);
            }
            if provider.is_empty() {
                return Err(FallenTreeError::EmptyProvider);
            }
        }
    }
    Ok(())
}

fn is_open<W: FallenTreeWorld>(world: &W, state: BlockStateId) -> bool {
    world.is_air(state) || world.is_replaceable_by_trees(state)
}

fn offset_n(
    position: BlockPos,
    direction: Direction,
    distance: i32,
) -> Result<BlockPos, FallenTreeError> {
    let [x, y, z] = direction.step();
    offset_xyz(
        position,
        x.checked_mul(distance)
            .ok_or(FallenTreeError::PositionOverflow)?,
        y.checked_mul(distance)
            .ok_or(FallenTreeError::PositionOverflow)?,
        z.checked_mul(distance)
            .ok_or(FallenTreeError::PositionOverflow)?,
    )
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, FallenTreeError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, FallenTreeError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(FallenTreeError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(FallenTreeError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(FallenTreeError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FallenTreeError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("fallen-tree length arithmetic overflow")]
    LengthOverflow,
    #[error("attached-to-logs directions must be nonempty")]
    EmptyDirections,
    #[error("attached-to-logs provider must be nonempty")]
    EmptyProvider,
    #[error("attached-to-logs weight total overflowed")]
    WeightOverflow,
    #[error("attached-to-logs probability must be finite and in 0..=1")]
    InvalidProbability,
    #[error("fallen-tree position overflow")]
    PositionOverflow,
}
