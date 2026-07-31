//! Pointed speleothem feature with source-ordered support selection and base spreading.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeleothemConfig {
    pub base_block: BlockStateId,
    pub pointed_block: BlockStateId,
    pub chance_of_directional_spread: f32,
    pub chance_of_spread_radius2: f32,
    pub chance_of_spread_radius3: f32,
    pub chance_of_taller_generation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointedThickness {
    Tip,
    TipMerge,
    Frustum,
    Middle,
    Base,
}

pub trait SpeleothemWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_base_block_identity(&self, state: BlockStateId, base: BlockStateId) -> bool;

    fn is_replaceable_speleothem_block(&self, state: BlockStateId) -> bool;

    fn is_air_or_water_block(&self, state: BlockStateId) -> bool;

    fn is_water_at(&mut self, position: BlockPos) -> bool;

    fn configure_pointed_state(
        &mut self,
        default_state: BlockStateId,
        direction: Direction,
        thickness: PointedThickness,
        waterlogged: bool,
    ) -> Option<BlockStateId>;

    fn offer_speleothem_block(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        flags: u32,
    ) -> bool;
}

pub fn place_speleothem<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: SpeleothemConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, SpeleothemError>
where
    R: GenerationRandom,
    W: SpeleothemWorld,
{
    validate_config(config)?;
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let above = offset(origin, Direction::Up)?;
    let below = offset(origin, Direction::Down)?;
    let above_state = world.block_state(above);
    let below_state = world.block_state(below);
    let above_support = is_support(world, above_state, config.base_block);
    let below_support = is_support(world, below_state, config.base_block);
    let growth_direction = match (above_support, below_support) {
        (false, false) => return Ok(false),
        (true, false) => Direction::Down,
        (false, true) => Direction::Up,
        (true, true) => {
            if random.next_bool() {
                Direction::Down
            } else {
                Direction::Up
            }
        }
    };
    let support = offset(origin, growth_direction.opposite())?;
    create_base_patch(world, support, config, random)?;

    let mut height = 1;
    if random.next_f32() < config.chance_of_taller_generation {
        let outward = offset(origin, growth_direction)?;
        let state = world.block_state(outward);
        if world.is_air_or_water_block(state) {
            height = 2;
        }
    }
    let support_state = world.block_state(support);
    if !is_support(world, support_state, config.base_block) {
        return Ok(true);
    }
    grow_pointed_column(
        world,
        origin,
        growth_direction,
        height,
        false,
        config.pointed_block,
    )?;
    Ok(true)
}

fn create_base_patch<R, W>(
    world: &mut W,
    support: BlockPos,
    config: SpeleothemConfig,
    random: &mut R,
) -> Result<(), SpeleothemError>
where
    R: GenerationRandom,
    W: SpeleothemWorld,
{
    attempt_base_block(world, support, config.base_block);
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        if random.next_f32() > config.chance_of_directional_spread {
            continue;
        }
        let radius_one = offset(support, direction)?;
        attempt_base_block(world, radius_one, config.base_block);
        if random.next_f32() <= config.chance_of_spread_radius2 {
            let radius_two = offset(radius_one, random_direction(random))?;
            attempt_base_block(world, radius_two, config.base_block);
            if random.next_f32() <= config.chance_of_spread_radius3 {
                let radius_three = offset(radius_two, random_direction(random))?;
                attempt_base_block(world, radius_three, config.base_block);
            }
        }
    }
    Ok(())
}

fn attempt_base_block<W: SpeleothemWorld>(world: &mut W, position: BlockPos, base: BlockStateId) {
    let state = world.block_state(position);
    if world.is_replaceable_speleothem_block(state) {
        let _ = world.offer_speleothem_block(position, base, 2);
    }
}

pub(crate) fn grow_pointed_column<W: SpeleothemWorld>(
    world: &mut W,
    origin: BlockPos,
    direction: Direction,
    height: i32,
    merge_tip: bool,
    pointed_default: BlockStateId,
) -> Result<(), SpeleothemError> {
    let mut cursor = origin;
    for index in 0..height {
        let thickness = if index + 1 == height {
            if merge_tip {
                PointedThickness::TipMerge
            } else {
                PointedThickness::Tip
            }
        } else if index + 2 == height {
            PointedThickness::Frustum
        } else if index == 0 {
            PointedThickness::Base
        } else {
            PointedThickness::Middle
        };
        let waterlogged = world.is_water_at(cursor);
        let state = world
            .configure_pointed_state(pointed_default, direction, thickness, waterlogged)
            .ok_or(SpeleothemError::MissingPointedProperties)?;
        let _ = world.offer_speleothem_block(cursor, state, 2);
        cursor = offset(cursor, direction)?;
    }
    Ok(())
}

fn is_support<W: SpeleothemWorld>(world: &W, state: BlockStateId, base: BlockStateId) -> bool {
    world.is_base_block_identity(state, base) || world.is_replaceable_speleothem_block(state)
}

fn random_direction(random: &mut impl GenerationRandom) -> Direction {
    Direction::ALL
        [random.next_u32(NonZeroU32::new(6).expect("direction bound is nonzero")) as usize]
}

fn validate_config(config: SpeleothemConfig) -> Result<(), SpeleothemError> {
    for probability in [
        config.chance_of_directional_spread,
        config.chance_of_spread_radius2,
        config.chance_of_spread_radius3,
        config.chance_of_taller_generation,
    ] {
        if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
            return Err(SpeleothemError::InvalidProbability);
        }
    }
    Ok(())
}

fn offset(origin: BlockPos, direction: Direction) -> Result<BlockPos, SpeleothemError> {
    let [x, y, z] = direction.step();
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(SpeleothemError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(SpeleothemError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(SpeleothemError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SpeleothemError {
    #[error("speleothem probability must be finite and in the inclusive range 0..=1")]
    InvalidProbability,
    #[error("configured pointed block lacks required properties")]
    MissingPointedProperties,
    #[error("speleothem position arithmetic overflowed")]
    PositionOverflow,
}
