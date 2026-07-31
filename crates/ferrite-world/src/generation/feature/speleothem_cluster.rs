//! Speleothem-cluster feature with per-column scans, pools, paired heights, and merging.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::speleothem::{
    SpeleothemError, SpeleothemWorld, grow_pointed_column,
};
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClusterFloatProvider {
    Constant(f32),
    Uniform {
        minimum: f32,
        maximum: f32,
    },
    ClampedNormal {
        mean: f32,
        deviation: f32,
        minimum: f32,
        maximum: f32,
    },
}

impl ClusterFloatProvider {
    fn sample(self, random: &mut impl GenerationRandom) -> Result<f32, SpeleothemClusterError> {
        match self {
            Self::Constant(value) => {
                validate_float(value)?;
                Ok(value)
            }
            Self::Uniform { minimum, maximum } => {
                validate_range(minimum, maximum)?;
                Ok(minimum + random.next_f32() * (maximum - minimum))
            }
            Self::ClampedNormal {
                mean,
                deviation,
                minimum,
                maximum,
            } => {
                validate_range(minimum, maximum)?;
                if !mean.is_finite() || !deviation.is_finite() || deviation < 0.0 {
                    return Err(SpeleothemClusterError::InvalidFloatProvider);
                }
                Ok((random.next_gaussian() as f32 * deviation + mean).clamp(minimum, maximum))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeleothemClusterConfig {
    pub base_block: BlockStateId,
    pub pointed_block: BlockStateId,
    pub water: BlockStateId,
    pub floor_to_ceiling_search_range: u32,
    pub height: IntProvider,
    pub wetness: ClusterFloatProvider,
    pub density: ClusterFloatProvider,
    pub radius: IntProvider,
    pub maximum_stalagmite_stalactite_height_difference: i32,
    pub height_deviation: f32,
    pub base_layer_thickness: IntProvider,
    pub chance_at_edge: f64,
    pub maximum_edge_distance: i32,
    pub maximum_height_bias_distance: i32,
}

pub trait SpeleothemClusterWorld: SpeleothemWorld {
    fn is_lava_block(&self, state: BlockStateId) -> bool;

    fn is_water_block_identity(&self, state: BlockStateId) -> bool;

    fn has_water_tagged_fluid(&mut self, position: BlockPos, state: BlockStateId) -> bool;

    fn is_base_stone_overworld(&self, state: BlockStateId) -> bool;
}

pub fn place_speleothem_cluster<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &SpeleothemClusterConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, SpeleothemClusterError>
where
    R: GenerationRandom,
    W: SpeleothemClusterWorld,
{
    validate_config(config)?;
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let origin_state = world.block_state(origin);
    if !world.is_air_or_water_block(origin_state) {
        return Ok(false);
    }
    let maximum_height = sample_nonnegative(&config.height, random)?;
    let wetness = config.wetness.sample(random)?;
    let density = config.density.sample(random)?;
    let radius_x = sample_nonnegative(&config.radius, random)?;
    let radius_z = sample_nonnegative(&config.radius, random)?;
    for x_offset in -radius_x..=radius_x {
        for z_offset in -radius_z..=radius_z {
            let edge_distance = (radius_x - x_offset.abs()).min(radius_z - z_offset.abs());
            let chance = mapped_column_chance(edge_distance, config);
            let candidate = offset_xyz(origin, x_offset, 0, z_offset)?;
            let inputs = ColumnInputs {
                x_offset,
                z_offset,
                maximum_height,
                wetness,
                density,
                chance,
            };
            place_cluster_column(world, candidate, inputs, config, random)?;
        }
    }
    Ok(true)
}

struct ColumnInputs {
    x_offset: i32,
    z_offset: i32,
    maximum_height: i32,
    wetness: f32,
    density: f32,
    chance: f64,
}

fn place_cluster_column<R, W>(
    world: &mut W,
    candidate: BlockPos,
    inputs: ColumnInputs,
    config: &SpeleothemClusterConfig,
    random: &mut R,
) -> Result<(), SpeleothemClusterError>
where
    R: GenerationRandom,
    W: SpeleothemClusterWorld,
{
    let candidate_state = world.block_state(candidate);
    if !world.is_air_or_water_block(candidate_state) {
        return Ok(());
    }
    let ceiling = scan_non_open_boundary(
        world,
        candidate,
        Direction::Up,
        config.floor_to_ceiling_search_range,
    )?;
    let mut floor = scan_non_open_boundary(
        world,
        candidate,
        Direction::Down,
        config.floor_to_ceiling_search_range,
    )?;
    if ceiling.is_none() && floor.is_none() {
        return Ok(());
    }

    let wet_draw = random.next_f32();
    if wet_draw < inputs.wetness
        && let Some(old_floor) = floor
        && can_place_pool(world, old_floor, config)?
    {
        let _ = world.offer_speleothem_block(old_floor, config.water, 2);
        floor = Some(offset(old_floor, Direction::Down)?);
    }

    let stalactite_gate = random.next_f64() < inputs.chance;
    let stalagmite_gate = random.next_f64() < inputs.chance;
    let stalactite_admitted = stalactite_gate && boundary_is_non_lava(world, ceiling);
    let stalagmite_admitted = stalagmite_gate && boundary_is_non_lava(world, floor);
    if stalactite_admitted {
        replace_base_layer(
            world,
            ceiling.expect("checked"),
            Direction::Up,
            config,
            random,
        )?;
    }
    if stalagmite_admitted {
        replace_base_layer(
            world,
            floor.expect("checked"),
            Direction::Down,
            config,
            random,
        )?;
    }

    let mut stalactite_height = if stalactite_admitted {
        let capped = if let (Some(ceiling), Some(floor)) = (ceiling, floor) {
            inputs.maximum_height.min(ceiling.y - floor.y)
        } else {
            inputs.maximum_height
        };
        sample_biased_height(
            random,
            capped,
            inputs.density,
            inputs.x_offset,
            inputs.z_offset,
            config,
        )
    } else {
        0
    };
    let mut stalagmite_height = if stalagmite_admitted {
        if ceiling.is_some() {
            let difference = config.maximum_stalagmite_stalactite_height_difference;
            (stalactite_height + sample_inclusive(random, -difference, difference)?).max(0)
        } else {
            sample_biased_height(
                random,
                inputs.maximum_height,
                inputs.density,
                inputs.x_offset,
                inputs.z_offset,
                config,
            )
        }
    } else {
        0
    };

    let finite_open_height = if let (Some(ceiling), Some(floor)) = (ceiling, floor) {
        Some(ceiling.y - floor.y - 1)
    } else {
        None
    };
    if let (Some(ceiling), Some(floor), Some(open_height)) = (ceiling, floor, finite_open_height)
        && stalactite_height + stalagmite_height >= open_height
    {
        let lower = (ceiling.y - stalactite_height).max(floor.y + 1);
        let upper = (floor.y + stalagmite_height).min(ceiling.y - 1);
        let meeting = sample_inclusive(random, lower, upper + 1)?;
        stalactite_height = ceiling.y - meeting;
        stalagmite_height = meeting - 1 - floor.y;
    }
    let merge_draw = random.next_bool();
    let merge = merge_draw
        && stalactite_height > 0
        && stalagmite_height > 0
        && finite_open_height.is_some_and(|height| stalactite_height + stalagmite_height == height);

    if let Some(ceiling) = ceiling {
        let start = offset(ceiling, Direction::Down)?;
        grow_pointed_column(
            world,
            start,
            Direction::Down,
            stalactite_height,
            merge,
            config.pointed_block,
        )?;
    }
    if let Some(floor) = floor {
        let start = offset(floor, Direction::Up)?;
        grow_pointed_column(
            world,
            start,
            Direction::Up,
            stalagmite_height,
            merge,
            config.pointed_block,
        )?;
    }
    Ok(())
}

fn scan_non_open_boundary<W: SpeleothemClusterWorld>(
    world: &mut W,
    origin: BlockPos,
    direction: Direction,
    range: u32,
) -> Result<Option<BlockPos>, SpeleothemClusterError> {
    let mut cursor = origin;
    let mut distance = 1_u32;
    while distance < range {
        let state = world.block_state(cursor);
        if !world.is_air_or_water_block(state) {
            break;
        }
        cursor = offset(cursor, direction)?;
        distance += 1;
    }
    let state = world.block_state(cursor);
    Ok((!world.is_air_or_water_block(state)).then_some(cursor))
}

fn can_place_pool<W: SpeleothemClusterWorld>(
    world: &mut W,
    floor: BlockPos,
    config: &SpeleothemClusterConfig,
) -> Result<bool, SpeleothemClusterError> {
    let floor_state = world.block_state(floor);
    if world.is_water_block_identity(floor_state)
        || floor_state == config.base_block
        || floor_state == config.pointed_block
    {
        return Ok(false);
    }
    let above = offset(floor, Direction::Up)?;
    let above_state = world.block_state(above);
    if world.has_water_tagged_fluid(above, above_state) {
        return Ok(false);
    }
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
        Direction::Down,
    ] {
        let neighbor = offset(floor, direction)?;
        let state = world.block_state(neighbor);
        if !world.is_base_stone_overworld(state) && !world.has_water_tagged_fluid(neighbor, state) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn replace_base_layer<R, W>(
    world: &mut W,
    boundary: BlockPos,
    direction: Direction,
    config: &SpeleothemClusterConfig,
    random: &mut R,
) -> Result<(), SpeleothemClusterError>
where
    R: GenerationRandom,
    W: SpeleothemClusterWorld,
{
    let thickness = sample_nonnegative(&config.base_layer_thickness, random)?;
    let mut cursor = boundary;
    for _ in 0..thickness {
        let state = world.block_state(cursor);
        if !world.is_replaceable_speleothem_block(state) {
            break;
        }
        let _ = world.offer_speleothem_block(cursor, config.base_block, 2);
        cursor = offset(cursor, direction)?;
    }
    Ok(())
}

fn sample_biased_height(
    random: &mut impl GenerationRandom,
    maximum_height: i32,
    density: f32,
    x_offset: i32,
    z_offset: i32,
    config: &SpeleothemClusterConfig,
) -> i32 {
    if maximum_height <= 0 || random.next_f32() > density {
        return 0;
    }
    let distance = x_offset.abs() + z_offset.abs();
    let bias = if config.maximum_height_bias_distance <= 0 {
        0.0
    } else {
        (1.0 - distance as f32 / config.maximum_height_bias_distance as f32).clamp(0.0, 1.0)
            * maximum_height as f32
            / 2.0
    };
    (random.next_gaussian() as f32 * config.height_deviation + bias)
        .clamp(0.0, maximum_height as f32) as i32
}

fn boundary_is_non_lava<W: SpeleothemClusterWorld>(
    world: &mut W,
    boundary: Option<BlockPos>,
) -> bool {
    boundary.is_some_and(|position| {
        let state = world.block_state(position);
        !world.is_lava_block(state)
    })
}

fn mapped_column_chance(edge_distance: i32, config: &SpeleothemClusterConfig) -> f64 {
    if config.maximum_edge_distance <= 0 {
        return 1.0;
    }
    let fraction =
        (f64::from(edge_distance) / f64::from(config.maximum_edge_distance)).clamp(0.0, 1.0);
    config.chance_at_edge + (1.0 - config.chance_at_edge) * fraction
}

fn sample_nonnegative(
    provider: &IntProvider,
    random: &mut impl GenerationRandom,
) -> Result<i32, SpeleothemClusterError> {
    let value = provider.sample(random)?;
    if value < 0 {
        Err(SpeleothemClusterError::NegativeProviderValue { value })
    } else {
        Ok(value)
    }
}

fn sample_inclusive(
    random: &mut impl GenerationRandom,
    minimum: i32,
    maximum: i32,
) -> Result<i32, SpeleothemClusterError> {
    let width = i64::from(maximum) - i64::from(minimum) + 1;
    let bound = u32::try_from(width)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(SpeleothemClusterError::InvalidIntegerRange)?;
    i64::from(minimum)
        .checked_add(i64::from(random.next_u32(bound)))
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(SpeleothemClusterError::InvalidIntegerRange)
}

fn validate_config(config: &SpeleothemClusterConfig) -> Result<(), SpeleothemClusterError> {
    if config.floor_to_ceiling_search_range > 512
        || config.maximum_stalagmite_stalactite_height_difference < 0
        || !config.height_deviation.is_finite()
        || config.height_deviation < 0.0
        || !config.chance_at_edge.is_finite()
        || !(0.0..=1.0).contains(&config.chance_at_edge)
    {
        return Err(SpeleothemClusterError::InvalidConfiguration);
    }
    Ok(())
}

fn validate_float(value: f32) -> Result<(), SpeleothemClusterError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SpeleothemClusterError::InvalidFloatProvider)
    }
}

fn validate_range(minimum: f32, maximum: f32) -> Result<(), SpeleothemClusterError> {
    validate_float(minimum)?;
    validate_float(maximum)?;
    if minimum <= maximum {
        Ok(())
    } else {
        Err(SpeleothemClusterError::InvalidFloatProvider)
    }
}

fn offset(origin: BlockPos, direction: Direction) -> Result<BlockPos, SpeleothemClusterError> {
    let [x, y, z] = direction.step();
    offset_xyz(origin, x, y, z)
}

fn offset_xyz(
    origin: BlockPos,
    x: i32,
    y: i32,
    z: i32,
) -> Result<BlockPos, SpeleothemClusterError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(SpeleothemClusterError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(SpeleothemClusterError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(SpeleothemClusterError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error)]
pub enum SpeleothemClusterError {
    #[error("speleothem-cluster integer provider failed")]
    Provider(#[from] ProviderError),
    #[error("speleothem writer failed")]
    Speleothem(#[from] SpeleothemError),
    #[error("speleothem-cluster provider returned negative value {value}")]
    NegativeProviderValue { value: i32 },
    #[error("speleothem-cluster float provider is invalid")]
    InvalidFloatProvider,
    #[error("speleothem-cluster integer range is invalid")]
    InvalidIntegerRange,
    #[error("speleothem-cluster configuration is invalid")]
    InvalidConfiguration,
    #[error("speleothem-cluster position arithmetic overflowed")]
    PositionOverflow,
}
