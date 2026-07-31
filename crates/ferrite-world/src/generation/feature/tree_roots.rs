//! Root-placer common admission and the mangrove preflight algorithm.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use crate::id::BlockStateId;

#[derive(Debug, Clone, PartialEq)]
pub struct MangroveRootConfig {
    pub trunk_offset_y: IntProvider,
    pub above_root_placement_chance: Option<f32>,
    pub max_root_width: i32,
    pub max_root_length: usize,
    pub random_skew_chance: f32,
}

pub trait MangroveRootWorld: TreeWorld {
    fn can_grow_through_mangrove_roots(&self, state: BlockStateId) -> bool;

    fn is_muddy_roots_material(&self, state: BlockStateId) -> bool;

    fn sample_root(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> BlockStateId;

    fn sample_muddy_root(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> BlockStateId;

    fn sample_above_root(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> BlockStateId;

    fn supports_waterlogged(&self, state: BlockStateId) -> bool;

    fn has_water_fluid(&mut self, position: BlockPos) -> bool;

    fn with_waterlogged(&self, state: BlockStateId, waterlogged: bool) -> BlockStateId;
}

impl MangroveRootConfig {
    pub fn trunk_origin(
        &self,
        origin: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> Result<BlockPos, MangroveRootError> {
        self.validate()?;
        offset_xyz(origin, 0, self.trunk_offset_y.sample(random)?, 0)
    }

    fn validate(&self) -> Result<(), MangroveRootError> {
        if !(1..=12).contains(&self.max_root_width)
            || !(1..=64).contains(&self.max_root_length)
            || !(0.0..=1.0).contains(&self.random_skew_chance)
            || self
                .above_root_placement_chance
                .is_some_and(|chance| !(0.0..=1.0).contains(&chance))
        {
            Err(MangroveRootError::InvalidConfiguration)
        } else {
            Ok(())
        }
    }
}

pub fn place_mangrove_roots<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    origin: BlockPos,
    trunk_origin: BlockPos,
    config: &MangroveRootConfig,
    random: &mut R,
) -> Result<bool, MangroveRootError>
where
    R: GenerationRandom,
    W: MangroveRootWorld,
{
    config.validate()?;
    let mut column = origin;
    while column.y < trunk_origin.y {
        if !can_place_mangrove_root(context.world(), column) {
            return Ok(false);
        }
        column = offset_xyz(column, 0, 1, 0)?;
    }

    let mut staged = vec![offset_xyz(trunk_origin, 0, -1, 0)?];
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        let start = offset(trunk_origin, direction)?;
        let mut direction_positions = Vec::new();
        if !simulate_roots_inner(
            context.world(),
            random,
            start,
            &mut direction_positions,
            0,
            SimulationInputs {
                direction,
                origin: trunk_origin,
                config,
            },
        )? {
            return Ok(false);
        }
        staged.extend(direction_positions);
        staged.push(start);
    }

    for position in staged {
        place_root(context, random, position, config)?;
    }
    Ok(true)
}

#[derive(Clone, Copy)]
struct SimulationInputs<'a> {
    direction: Direction,
    origin: BlockPos,
    config: &'a MangroveRootConfig,
}

fn simulate_roots_inner<R, W>(
    world: &mut W,
    random: &mut R,
    position: BlockPos,
    staged: &mut Vec<BlockPos>,
    depth: usize,
    inputs: SimulationInputs<'_>,
) -> Result<bool, MangroveRootError>
where
    R: GenerationRandom,
    W: MangroveRootWorld,
{
    if depth == inputs.config.max_root_length || staged.len() > inputs.config.max_root_length {
        return Ok(false);
    }
    let candidates = potential_positions(position, random, inputs)?;
    for candidate in candidates.into_iter().flatten() {
        if !can_place_mangrove_root(world, candidate) {
            continue;
        }
        staged.push(candidate);
        if !simulate_roots_inner(world, random, candidate, staged, depth + 1, inputs)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn potential_positions(
    position: BlockPos,
    random: &mut impl GenerationRandom,
    inputs: SimulationInputs<'_>,
) -> Result<[Option<BlockPos>; 2], MangroveRootError> {
    let below = offset_xyz(position, 0, -1, 0)?;
    let outward = offset(position, inputs.direction)?;
    let outward_below = offset_xyz(outward, 0, -1, 0)?;
    let distance = manhattan(position, inputs.origin);
    let width = inputs.config.max_root_width;
    let skew = inputs.config.random_skew_chance;
    if distance > width - 3 && distance <= width {
        return if random.next_f32() < skew {
            Ok([Some(below), Some(outward_below)])
        } else {
            Ok([Some(below), None])
        };
    }
    if distance > width {
        return Ok([Some(below), None]);
    }
    if random.next_f32() < skew {
        Ok([Some(below), None])
    } else if random.next_bool() {
        Ok([Some(outward), None])
    } else {
        Ok([Some(below), None])
    }
}

fn place_root<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    position: BlockPos,
    config: &MangroveRootConfig,
) -> Result<(), MangroveRootError>
where
    R: GenerationRandom,
    W: MangroveRootWorld,
{
    let state = context.world().block_state(position);
    if context.world().is_muddy_roots_material(state) {
        let muddy = context.world().sample_muddy_root(position, random);
        let muddy = potentially_waterlogged(context.world(), position, muddy);
        context.offer_root(position, muddy);
        return Ok(());
    }
    if !can_place_mangrove_root(context.world(), position) {
        return Ok(());
    }
    let root = context.world().sample_root(position, random);
    let root = potentially_waterlogged(context.world(), position, root);
    context.offer_root(position, root);
    let Some(chance) = config.above_root_placement_chance else {
        return Ok(());
    };
    if random.next_f32() < chance {
        let above = offset_xyz(position, 0, 1, 0)?;
        let above_state = context.world().block_state(above);
        if context.world().is_air(above_state) {
            let above_root = context.world().sample_above_root(above, random);
            let above_root = potentially_waterlogged(context.world(), above, above_root);
            context.offer_root(above, above_root);
        }
    }
    Ok(())
}

fn potentially_waterlogged<W: MangroveRootWorld>(
    world: &mut W,
    position: BlockPos,
    state: BlockStateId,
) -> BlockStateId {
    if world.supports_waterlogged(state) {
        let waterlogged = world.has_water_fluid(position);
        world.with_waterlogged(state, waterlogged)
    } else {
        state
    }
}

fn can_place_mangrove_root<W: MangroveRootWorld>(world: &mut W, position: BlockPos) -> bool {
    let state = world.block_state(position);
    can_place_mangrove_root_state(world, state)
}

fn can_place_mangrove_root_state<W: MangroveRootWorld>(world: &W, state: BlockStateId) -> bool {
    world.is_air(state)
        || world.is_replaceable_by_trees(state)
        || world.can_grow_through_mangrove_roots(state)
}

fn manhattan(left: BlockPos, right: BlockPos) -> i32 {
    left.x.abs_diff(right.x) as i32
        + left.y.abs_diff(right.y) as i32
        + left.z.abs_diff(right.z) as i32
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, MangroveRootError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, MangroveRootError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(MangroveRootError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(MangroveRootError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(MangroveRootError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MangroveRootError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("mangrove-root configuration violates codec bounds")]
    InvalidConfiguration,
    #[error("mangrove-root position overflow")]
    PositionOverflow,
}
