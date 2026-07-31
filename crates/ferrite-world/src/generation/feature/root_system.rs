//! Root-system candidate search and rooted/hanging postpasses.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootSystemConfig {
    pub required_vertical_space_for_tree: u32,
    pub root_radius: u32,
    pub root_placement_attempts: u32,
    pub root_column_max_height: u32,
    pub hanging_root_radius: u32,
    pub hanging_roots_vertical_span: u32,
    pub hanging_root_placement_attempts: u32,
    pub allowed_vertical_water_for_tree: u32,
    pub level_test_distance: u32,
    pub maximum_level_deviation: u32,
}

pub trait RootSystemWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn has_water_tagged_fluid(&self, state: BlockStateId) -> bool;

    fn has_lava_fluid(&self, state: BlockStateId) -> bool;

    fn is_solid(&self, state: BlockStateId) -> bool;

    fn world_surface(&mut self, x: i32, z: i32) -> i32;

    fn allowed_tree_position(&mut self, position: BlockPos) -> bool;

    fn place_root_child<R: GenerationRandom>(&mut self, position: BlockPos, random: &mut R)
    -> bool;

    fn is_root_replaceable(&self, state: BlockStateId) -> bool;

    fn sample_root_state<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn sample_hanging_root_state<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn can_hanging_root_survive(&mut self, state: BlockStateId, position: BlockPos) -> bool;

    fn has_sturdy_downward_face_at(
        &mut self,
        above_position: BlockPos,
        above_state: BlockStateId,
        queried_position: BlockPos,
    ) -> bool;

    fn offer_root_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_root_system<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: RootSystemConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, RootSystemError>
where
    R: GenerationRandom,
    W: RootSystemWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    validate_config(config)?;
    let origin_state = world.block_state(origin);
    if !world.is_air(origin_state) {
        return Ok(false);
    }

    let mut candidate = origin;
    for index in 0..config.root_column_max_height {
        candidate = offset(candidate, Direction::Up)?;
        if world.world_surface(candidate.x, candidate.z) < candidate.y {
            break;
        }
        if !world.allowed_tree_position(candidate)
            || !has_tree_clearance(world, candidate, config)?
            || !passes_level_test(world, candidate, config)?
        {
            continue;
        }
        let support = offset(candidate, Direction::Down)?;
        let support_state = world.block_state(support);
        if world.has_lava_fluid(support_state) || !world.is_solid(support_state) {
            break;
        }
        if !world.place_root_child(candidate, random) {
            continue;
        }
        place_rooted_layers(world, origin, index, config, random)?;
        place_hanging_roots(world, origin, config, random)?;
        break;
    }
    Ok(true)
}

fn has_tree_clearance<W: RootSystemWorld>(
    world: &mut W,
    candidate: BlockPos,
    config: RootSystemConfig,
) -> Result<bool, RootSystemError> {
    let mut cursor = candidate;
    for one_based_index in 1..=config.required_vertical_space_for_tree {
        cursor = offset(cursor, Direction::Up)?;
        let state = world.block_state(cursor);
        if world.is_air(state) {
            continue;
        }
        let water_limit = one_based_index
            .checked_add(1)
            .ok_or(RootSystemError::PositionOverflow)?;
        if water_limit > config.allowed_vertical_water_for_tree
            || !world.has_water_tagged_fluid(state)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn passes_level_test<W: RootSystemWorld>(
    world: &mut W,
    candidate: BlockPos,
    config: RootSystemConfig,
) -> Result<bool, RootSystemError> {
    if config.level_test_distance == 0 {
        return Ok(true);
    }
    let distance =
        i32::try_from(config.level_test_distance).map_err(|_| RootSystemError::PositionOverflow)?;
    let deviation = i32::try_from(config.maximum_level_deviation)
        .map_err(|_| RootSystemError::PositionOverflow)?;
    for direction in [
        Direction::South,
        Direction::West,
        Direction::North,
        Direction::East,
    ] {
        let side = offset_n(candidate, direction, distance)?;
        let below = offset_xyz(side, 0, -deviation, 0)?;
        let below_state = world.block_state(below);
        if world.is_air(below_state) {
            return Ok(false);
        }
        let above = offset_xyz(side, 0, deviation, 0)?;
        let above_state = world.block_state(above);
        if !world.is_air(above_state) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn place_rooted_layers<R, W>(
    world: &mut W,
    origin: BlockPos,
    layer_count: u32,
    config: RootSystemConfig,
    random: &mut R,
) -> Result<(), RootSystemError>
where
    R: GenerationRandom,
    W: RootSystemWorld,
{
    let radius = NonZeroU32::new(config.root_radius).ok_or(RootSystemError::InvalidRootRadius)?;
    for layer in 0..layer_count {
        let y = origin
            .y
            .checked_add(i32::try_from(layer).map_err(|_| RootSystemError::PositionOverflow)?)
            .ok_or(RootSystemError::PositionOverflow)?;
        for _ in 0..config.root_placement_attempts {
            let x = triangular_offset(random, radius);
            let z = triangular_offset(random, radius);
            let candidate = offset_xyz(origin, x, y - origin.y, z)?;
            let state = world.block_state(candidate);
            if world.is_root_replaceable(state) {
                let root = world.sample_root_state(candidate, random);
                let _ = world.offer_root_block(candidate, root, 2);
            }
        }
    }
    Ok(())
}

fn place_hanging_roots<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: RootSystemConfig,
    random: &mut R,
) -> Result<(), RootSystemError>
where
    R: GenerationRandom,
    W: RootSystemWorld,
{
    let radius =
        NonZeroU32::new(config.hanging_root_radius).ok_or(RootSystemError::InvalidHangingRadius)?;
    let span = NonZeroU32::new(config.hanging_roots_vertical_span)
        .ok_or(RootSystemError::InvalidHangingSpan)?;
    for _ in 0..config.hanging_root_placement_attempts {
        let x = triangular_offset(random, radius);
        let y = triangular_offset(random, span);
        let z = triangular_offset(random, radius);
        let candidate = offset_xyz(origin, x, y, z)?;
        let current = world.block_state(candidate);
        if !world.is_air(current) {
            continue;
        }
        let root = world.sample_hanging_root_state(candidate, random);
        if !world.can_hanging_root_survive(root, candidate) {
            continue;
        }
        let above = offset(candidate, Direction::Up)?;
        let above_state = world.block_state(above);
        if !world.has_sturdy_downward_face_at(above, above_state, candidate) {
            continue;
        }
        let _ = world.offer_root_block(candidate, root, 2);
    }
    Ok(())
}

fn triangular_offset(random: &mut impl GenerationRandom, bound: NonZeroU32) -> i32 {
    random.next_u32(bound) as i32 - random.next_u32(bound) as i32
}

fn validate_config(config: RootSystemConfig) -> Result<(), RootSystemError> {
    if !(1..=64).contains(&config.required_vertical_space_for_tree)
        || !(1..=64).contains(&config.allowed_vertical_water_for_tree)
    {
        return Err(RootSystemError::InvalidVerticalSpace);
    }
    if !(1..=64).contains(&config.root_radius) {
        return Err(RootSystemError::InvalidRootRadius);
    }
    if !(1..=256).contains(&config.root_placement_attempts)
        || !(1..=256).contains(&config.hanging_root_placement_attempts)
    {
        return Err(RootSystemError::InvalidAttempts);
    }
    if !(1..=4096).contains(&config.root_column_max_height) {
        return Err(RootSystemError::InvalidColumnHeight);
    }
    if !(1..=64).contains(&config.hanging_root_radius) {
        return Err(RootSystemError::InvalidHangingRadius);
    }
    if !(1..=16).contains(&config.hanging_roots_vertical_span) {
        return Err(RootSystemError::InvalidHangingSpan);
    }
    if config.level_test_distance > 16 || config.maximum_level_deviation > 64 {
        return Err(RootSystemError::InvalidLevelTest);
    }
    Ok(())
}

fn offset_n(
    position: BlockPos,
    direction: Direction,
    distance: i32,
) -> Result<BlockPos, RootSystemError> {
    let [x, y, z] = direction.step();
    offset_xyz(
        position,
        x.checked_mul(distance)
            .ok_or(RootSystemError::PositionOverflow)?,
        y.checked_mul(distance)
            .ok_or(RootSystemError::PositionOverflow)?,
        z.checked_mul(distance)
            .ok_or(RootSystemError::PositionOverflow)?,
    )
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, RootSystemError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, RootSystemError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(RootSystemError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(RootSystemError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(RootSystemError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RootSystemError {
    #[error("root-system vertical-space fields must be in 1..=64")]
    InvalidVerticalSpace,
    #[error("root radius must be in 1..=64")]
    InvalidRootRadius,
    #[error("root-system attempt counts must be in 1..=256")]
    InvalidAttempts,
    #[error("root-system column height must be in 1..=4096")]
    InvalidColumnHeight,
    #[error("hanging-root radius must be in 1..=64")]
    InvalidHangingRadius,
    #[error("hanging-root vertical span must be in 1..=16")]
    InvalidHangingSpan,
    #[error("root-system level-test fields exceed codec bounds")]
    InvalidLevelTest,
    #[error("root-system position overflow")]
    PositionOverflow,
}
