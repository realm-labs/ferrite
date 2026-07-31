//! Multiface growth placement and optional one-step spreading.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultifaceGrowthConfig {
    pub block: BlockStateId,
    pub place_on_ceiling: bool,
    pub place_on_floor: bool,
    pub place_on_walls: bool,
    pub search_range: u32,
    pub chance_of_spreading: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultifaceSpreadType {
    SamePosition,
    SamePlane,
    WrapAround,
}

pub trait MultifaceWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_water_block_identity(&self, state: BlockStateId) -> bool;

    fn is_configured_multiface(&self, state: BlockStateId, configured: BlockStateId) -> bool;

    fn is_multiface_spreadable(&self, configured: BlockStateId) -> bool;

    fn can_be_placed_on(&self, support: BlockStateId) -> bool;

    fn placement_state(
        &mut self,
        configured: BlockStateId,
        current: BlockStateId,
        position: BlockPos,
        face: Direction,
    ) -> Option<BlockStateId>;

    fn has_face(&self, state: BlockStateId, face: Direction) -> bool;

    fn can_spread_into(
        &mut self,
        source: BlockPos,
        target: BlockPos,
        target_face: Direction,
        spread_type: MultifaceSpreadType,
    ) -> bool;

    fn offer_multiface(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn mark_for_postprocessing(&mut self, position: BlockPos);
}

pub fn place_multiface_growth<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: MultifaceGrowthConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, MultifaceGrowthError>
where
    R: GenerationRandom,
    W: MultifaceWorld,
{
    validate_config(config)?;
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let initial = world.block_state(origin);
    if !is_growth_space(world, initial, config.block) {
        return Ok(false);
    }
    if !world.is_multiface_spreadable(config.block) {
        return Ok(false);
    }

    let mut directions = valid_directions(config);
    shuffle(&mut directions, random)?;
    let origin_state = world.block_state(origin);
    if place_growth_if_possible(world, origin, origin_state, &directions, config, random)? {
        return Ok(true);
    }

    for travel in directions.iter().copied() {
        let mut faces = directions
            .iter()
            .copied()
            .filter(|face| *face != travel.opposite())
            .collect::<Vec<_>>();
        shuffle(&mut faces, random)?;
        for _ in 0..config.search_range {
            // This deliberately resets to the distance-one cell on every retry.
            let candidate = offset(origin, travel)?;
            let state = world.block_state(candidate);
            if !is_growth_space(world, state, config.block) {
                break;
            }
            if place_growth_if_possible(world, candidate, state, &faces, config, random)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn place_growth_if_possible<R, W>(
    world: &mut W,
    candidate: BlockPos,
    current: BlockStateId,
    faces: &[Direction],
    config: MultifaceGrowthConfig,
    random: &mut R,
) -> Result<bool, MultifaceGrowthError>
where
    R: GenerationRandom,
    W: MultifaceWorld,
{
    for face in faces.iter().copied() {
        let support_position = offset(candidate, face)?;
        let support = world.block_state(support_position);
        if !world.can_be_placed_on(support) {
            continue;
        }
        let Some(placed) = world.placement_state(config.block, current, candidate, face) else {
            return Ok(false);
        };
        let _ = world.offer_multiface(candidate, placed, 3);
        world.mark_for_postprocessing(candidate);
        if random.next_f32() < config.chance_of_spreading {
            spread_from_face(world, candidate, placed, face, config.block, random)?;
        }
        return Ok(true);
    }
    Ok(false)
}

fn spread_from_face<R, W>(
    world: &mut W,
    source: BlockPos,
    source_state: BlockStateId,
    placed_face: Direction,
    configured: BlockStateId,
    random: &mut R,
) -> Result<(), MultifaceGrowthError>
where
    R: GenerationRandom,
    W: MultifaceWorld,
{
    let mut directions = Direction::ALL.to_vec();
    shuffle(&mut directions, random)?;
    for direction in directions {
        if direction.axis() == placed_face.axis()
            || !world.has_face(source_state, placed_face)
            || world.has_face(source_state, direction)
        {
            continue;
        }
        let same_plane = offset(source, direction)?;
        let around = offset(same_plane, placed_face)?;
        let candidates = [
            (source, placed_face, MultifaceSpreadType::SamePosition),
            (same_plane, placed_face, MultifaceSpreadType::SamePlane),
            (
                around,
                direction.opposite(),
                MultifaceSpreadType::WrapAround,
            ),
        ];
        for (target, target_face, spread_type) in candidates {
            if !world.can_spread_into(source, target, target_face, spread_type) {
                continue;
            }
            let current = world.block_state(target);
            let Some(state) = world.placement_state(configured, current, target, target_face)
            else {
                continue;
            };
            world.mark_for_postprocessing(target);
            if world.offer_multiface(target, state, 2) {
                return Ok(());
            }
        }
    }
    Ok(())
}

fn valid_directions(config: MultifaceGrowthConfig) -> Vec<Direction> {
    let mut directions = Vec::with_capacity(6);
    if config.place_on_ceiling {
        directions.push(Direction::Up);
    }
    if config.place_on_floor {
        directions.push(Direction::Down);
    }
    if config.place_on_walls {
        directions.extend([
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]);
    }
    directions
}

fn is_growth_space<W: MultifaceWorld>(
    world: &W,
    state: BlockStateId,
    configured: BlockStateId,
) -> bool {
    world.is_air(state)
        || world.is_water_block_identity(state)
        || world.is_configured_multiface(state, configured)
}

fn shuffle(
    directions: &mut [Direction],
    random: &mut impl GenerationRandom,
) -> Result<(), MultifaceGrowthError> {
    for remaining in (2..=directions.len()).rev() {
        let bound = u32::try_from(remaining)
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or(MultifaceGrowthError::DirectionCountOverflow)?;
        let index = random.next_u32(bound) as usize;
        directions.swap(remaining - 1, index);
    }
    Ok(())
}

fn validate_config(config: MultifaceGrowthConfig) -> Result<(), MultifaceGrowthError> {
    if !(1..=64).contains(&config.search_range) {
        return Err(MultifaceGrowthError::InvalidSearchRange(
            config.search_range,
        ));
    }
    if !config.chance_of_spreading.is_finite() || !(0.0..=1.0).contains(&config.chance_of_spreading)
    {
        return Err(MultifaceGrowthError::InvalidSpreadChance);
    }
    Ok(())
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, MultifaceGrowthError> {
    let [x, y, z] = direction.step();
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(MultifaceGrowthError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(MultifaceGrowthError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(MultifaceGrowthError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MultifaceGrowthError {
    #[error("multiface search range must be in 1..=64, got {0}")]
    InvalidSearchRange(u32),
    #[error("multiface spreading chance must be finite and in 0..=1")]
    InvalidSpreadChance,
    #[error("direction count does not fit the generation RNG bound")]
    DirectionCountOverflow,
    #[error("multiface feature position overflow")]
    PositionOverflow,
}
