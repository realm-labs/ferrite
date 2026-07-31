//! Geode distribution fields, material layers, cracks, and inner placements.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeLayerSettings {
    pub filling: f64,
    pub inner_layer: f64,
    pub middle_layer: f64,
    pub outer_layer: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeCrackSettings {
    pub generate_chance: f32,
    pub base_crack_size: f64,
    pub crack_point_offset: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeodeConfig {
    pub distribution_points: IntProvider,
    pub outer_wall_distance: IntProvider,
    pub point_offset: IntProvider,
    pub minimum_generation_offset: i32,
    pub maximum_generation_offset: i32,
    pub invalid_blocks_threshold: i32,
    pub layers: GeodeLayerSettings,
    pub crack: GeodeCrackSettings,
    pub noise_multiplier: f64,
    pub use_alternate_layer_chance: f32,
    pub use_potential_placements_chance: f32,
    pub placements_require_alternate: bool,
    pub inner_placements: Vec<BlockStateId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeodeMaterial {
    Filling,
    Inner,
    AlternateInner,
    Middle,
    Outer,
}

#[derive(Debug, Clone, Copy)]
struct DistributionPoint {
    position: BlockPos,
    offset: i32,
}

pub trait GeodeWorld {
    fn world_seed(&self) -> u64;

    fn initialize_geode_noise(&mut self, seed: u64);

    fn geode_noise(&mut self, position: BlockPos) -> f64;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_invalid_geode_block(&self, state: BlockStateId) -> bool;

    fn is_protected_from_geode(&self, state: BlockStateId) -> bool;

    fn sample_geode_material<R: GenerationRandom>(
        &mut self,
        material: GeodeMaterial,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn canonical_air(&self) -> BlockStateId;

    fn fluid_is_nonempty_at(&mut self, position: BlockPos) -> bool;

    fn fluid_is_full(&mut self, position: BlockPos, state: BlockStateId) -> bool;

    fn is_water_block_identity(&self, state: BlockStateId) -> bool;

    fn with_facing(&self, state: BlockStateId, direction: Direction) -> BlockStateId;

    fn with_waterlogged_from_neighbor(
        &mut self,
        state: BlockStateId,
        neighbor_position: BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId;

    fn offer_geode_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn schedule_zero_delay_fluid_tick(&mut self, position: BlockPos);
}

pub fn place_geode<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &GeodeConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, GeodeError>
where
    R: GenerationRandom,
    W: GeodeWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    validate_config(config)?;
    let point_count = config.distribution_points.sample(random)?;
    if !(1..=20).contains(&point_count) {
        return Err(GeodeError::InvalidPointCount(point_count));
    }
    world.initialize_geode_noise(world.world_seed());
    let outer_maximum = provider_maximum(&config.outer_wall_distance)?;
    if !(1..=20).contains(&outer_maximum) {
        return Err(GeodeError::InvalidOuterDistance(outer_maximum));
    }
    let distance_adjustment = f64::from(point_count) / f64::from(outer_maximum);
    let thresholds = ShapeThresholds::new(config, distance_adjustment, point_count, random);
    let cracks_admitted = random.next_f32() < config.crack.generate_chance;
    let points = build_distribution_points(world, origin, point_count, config, random)?;
    let Some(points) = points else {
        return Ok(false);
    };
    let crack_points = if cracks_admitted {
        build_crack_points(origin, point_count, config.crack.crack_point_offset, random)?
    } else {
        Vec::new()
    };
    let mut potential_placements = Vec::new();
    let traversal = TraversalInputs {
        origin,
        config,
        points: &points,
        crack_points: &crack_points,
        thresholds,
    };
    traverse_cube(world, traversal, random, &mut potential_placements)?;
    place_inner_growths(world, &potential_placements, config, random)?;
    Ok(true)
}

#[derive(Debug, Clone, Copy)]
struct ShapeThresholds {
    filling: f64,
    inner: f64,
    middle: f64,
    outer: f64,
    crack: f64,
}

impl ShapeThresholds {
    fn new(
        config: &GeodeConfig,
        adjustment: f64,
        point_count: i32,
        random: &mut impl GenerationRandom,
    ) -> Self {
        let crack_adjustment = if point_count > 3 { adjustment } else { 0.0 };
        Self {
            filling: inverse_sqrt(config.layers.filling),
            inner: inverse_sqrt(config.layers.inner_layer + adjustment),
            middle: inverse_sqrt(config.layers.middle_layer + adjustment),
            outer: inverse_sqrt(config.layers.outer_layer + adjustment),
            crack: inverse_sqrt(
                config.crack.base_crack_size + random.next_f64() / 2.0 + crack_adjustment,
            ),
        }
    }
}

fn build_distribution_points<R, W>(
    world: &mut W,
    origin: BlockPos,
    count: i32,
    config: &GeodeConfig,
    random: &mut R,
) -> Result<Option<Vec<DistributionPoint>>, GeodeError>
where
    R: GenerationRandom,
    W: GeodeWorld,
{
    let mut points = Vec::with_capacity(count as usize);
    let mut invalid = 0_i32;
    for _ in 0..count {
        let x = sample_outer_distance(&config.outer_wall_distance, random)?;
        let y = sample_outer_distance(&config.outer_wall_distance, random)?;
        let z = sample_outer_distance(&config.outer_wall_distance, random)?;
        let position = offset_xyz(origin, x, y, z)?;
        let state = world.block_state(position);
        if world.is_air(state) || world.is_invalid_geode_block(state) {
            invalid += 1;
            if invalid > config.invalid_blocks_threshold {
                return Ok(None);
            }
        }
        let point_offset = config.point_offset.sample(random)?;
        if !(0..=10).contains(&point_offset) {
            return Err(GeodeError::InvalidPointOffset(point_offset));
        }
        points.push(DistributionPoint {
            position,
            offset: point_offset,
        });
    }
    Ok(Some(points))
}

fn build_crack_points(
    origin: BlockPos,
    count: i32,
    point_offset: i32,
    random: &mut impl GenerationRandom,
) -> Result<Vec<DistributionPoint>, GeodeError> {
    let q = count
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .ok_or(GeodeError::PositionOverflow)?;
    let choice =
        random.next_u32(NonZeroU32::new(4).expect("geode crack orientation bound is nonzero"));
    let (crack_x, crack_z) = match choice {
        0 => (q, 0),
        1 => (0, q),
        2 => (q, q),
        3 => (0, 0),
        _ => unreachable!("bounded crack orientation draw"),
    };
    [
        (crack_x, 7, crack_z),
        (crack_x, 5, crack_z),
        (crack_x, 1, crack_z),
    ]
    .into_iter()
    .map(|(x, y, z)| {
        Ok(DistributionPoint {
            position: offset_xyz(origin, x, y, z)?,
            offset: point_offset,
        })
    })
    .collect::<Result<Vec<_>, GeodeError>>()
}

struct TraversalInputs<'a> {
    origin: BlockPos,
    config: &'a GeodeConfig,
    points: &'a [DistributionPoint],
    crack_points: &'a [DistributionPoint],
    thresholds: ShapeThresholds,
}

fn traverse_cube<R, W>(
    world: &mut W,
    inputs: TraversalInputs<'_>,
    random: &mut R,
    potential_placements: &mut Vec<BlockPos>,
) -> Result<(), GeodeError>
where
    R: GenerationRandom,
    W: GeodeWorld,
{
    let minimum = inputs
        .config
        .minimum_generation_offset
        .min(inputs.config.maximum_generation_offset);
    let maximum = inputs
        .config
        .minimum_generation_offset
        .max(inputs.config.maximum_generation_offset);
    for z in minimum..=maximum {
        for y in minimum..=maximum {
            for x in minimum..=maximum {
                let position = offset_xyz(inputs.origin, x, y, z)?;
                let noise = world.geode_noise(position) * inputs.config.noise_multiplier;
                let shape = field_value(position, inputs.points, noise);
                let crack = field_value(position, inputs.crack_points, noise);
                if shape < inputs.thresholds.outer {
                    continue;
                }
                if crack >= inputs.thresholds.crack && shape < inputs.thresholds.filling {
                    safe_write(world, position, world.canonical_air());
                    schedule_crack_ticks(world, position)?;
                    continue;
                }
                place_material(
                    world,
                    position,
                    shape,
                    inputs.thresholds,
                    inputs.config,
                    random,
                    potential_placements,
                );
            }
        }
    }
    Ok(())
}

fn place_material(
    world: &mut impl GeodeWorld,
    position: BlockPos,
    shape: f64,
    thresholds: ShapeThresholds,
    config: &GeodeConfig,
    random: &mut impl GenerationRandom,
    potential_placements: &mut Vec<BlockPos>,
) {
    let (material, alternate) = if shape >= thresholds.filling {
        (GeodeMaterial::Filling, false)
    } else if shape >= thresholds.inner {
        let alternate = random.next_f32() < config.use_alternate_layer_chance;
        (
            if alternate {
                GeodeMaterial::AlternateInner
            } else {
                GeodeMaterial::Inner
            },
            alternate,
        )
    } else if shape >= thresholds.middle {
        (GeodeMaterial::Middle, false)
    } else {
        (GeodeMaterial::Outer, false)
    };
    let state = world.sample_geode_material(material, position, random);
    safe_write(world, position, state);
    if matches!(
        material,
        GeodeMaterial::Inner | GeodeMaterial::AlternateInner
    ) && (!config.placements_require_alternate || alternate)
        && random.next_f32() < config.use_potential_placements_chance
    {
        potential_placements.push(position);
    }
}

fn place_inner_growths(
    world: &mut impl GeodeWorld,
    placements: &[BlockPos],
    config: &GeodeConfig,
    random: &mut impl GenerationRandom,
) -> Result<(), GeodeError> {
    let bound = u32::try_from(config.inner_placements.len())
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(GeodeError::EmptyInnerPlacements)?;
    for source in placements.iter().copied() {
        let mut state = config.inner_placements[random.next_u32(bound) as usize];
        for direction in Direction::ALL {
            state = world.with_facing(state, direction);
            let target = offset(source, direction)?;
            let neighbor = world.block_state(target);
            state = world.with_waterlogged_from_neighbor(state, target, neighbor);
            if world.is_air(neighbor)
                || (world.is_water_block_identity(neighbor)
                    && world.fluid_is_full(target, neighbor))
            {
                safe_write(world, target, state);
                break;
            }
        }
    }
    Ok(())
}

fn schedule_crack_ticks(world: &mut impl GeodeWorld, position: BlockPos) -> Result<(), GeodeError> {
    for direction in Direction::ALL {
        let neighbor = offset(position, direction)?;
        if world.fluid_is_nonempty_at(neighbor) {
            world.schedule_zero_delay_fluid_tick(neighbor);
        }
    }
    Ok(())
}

fn safe_write(world: &mut impl GeodeWorld, position: BlockPos, state: BlockStateId) {
    let current = world.block_state(position);
    if !world.is_protected_from_geode(current) {
        let _ = world.offer_geode_block(position, state, 2);
    }
}

fn field_value(position: BlockPos, points: &[DistributionPoint], noise: f64) -> f64 {
    points
        .iter()
        .map(|point| {
            let x = f64::from(position.x) - f64::from(point.position.x);
            let y = f64::from(position.y) - f64::from(point.position.y);
            let z = f64::from(position.z) - f64::from(point.position.z);
            inverse_sqrt(x * x + y * y + z * z + f64::from(point.offset)) + noise
        })
        .sum()
}

fn sample_outer_distance(
    provider: &IntProvider,
    random: &mut impl GenerationRandom,
) -> Result<i32, GeodeError> {
    let sampled = provider.sample(random)?;
    if !(1..=20).contains(&sampled) {
        return Err(GeodeError::InvalidOuterDistance(sampled));
    }
    Ok(sampled)
}

fn provider_maximum(provider: &IntProvider) -> Result<i32, GeodeError> {
    match provider {
        IntProvider::Constant(value) => Ok(*value),
        IntProvider::Uniform { maximum, .. }
        | IntProvider::BiasedToBottom { maximum, .. }
        | IntProvider::ClampedNormal { maximum, .. } => Ok(*maximum),
        IntProvider::Weighted(entries) => entries
            .iter()
            .map(|entry| provider_maximum(&entry.provider))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .ok_or(GeodeError::InvalidOuterDistance(0)),
        IntProvider::Clamped {
            source,
            minimum,
            maximum,
        } => Ok(provider_maximum(source)?.clamp(*minimum, *maximum)),
        IntProvider::ZeroPlateauTrapezoid { radius } => {
            i32::try_from(*radius).map_err(|_| GeodeError::InvalidOuterDistance(i32::MAX))
        }
    }
}

fn validate_config(config: &GeodeConfig) -> Result<(), GeodeError> {
    for value in [
        config.layers.filling,
        config.layers.inner_layer,
        config.layers.middle_layer,
        config.layers.outer_layer,
    ] {
        if !value.is_finite() || !(0.01..=50.0).contains(&value) {
            return Err(GeodeError::InvalidLayer);
        }
    }
    if !config.crack.generate_chance.is_finite()
        || !(0.0..=1.0).contains(&config.crack.generate_chance)
        || !config.crack.base_crack_size.is_finite()
        || !(0.0..=5.0).contains(&config.crack.base_crack_size)
        || !(0..=10).contains(&config.crack.crack_point_offset)
    {
        return Err(GeodeError::InvalidCrack);
    }
    for chance in [
        config.use_alternate_layer_chance,
        config.use_potential_placements_chance,
    ] {
        if !chance.is_finite() || !(0.0..=1.0).contains(&chance) {
            return Err(GeodeError::InvalidChance);
        }
    }
    if !config.noise_multiplier.is_finite() || !(0.0..=1.0).contains(&config.noise_multiplier) {
        return Err(GeodeError::InvalidNoiseMultiplier);
    }
    if config.inner_placements.is_empty() {
        return Err(GeodeError::EmptyInnerPlacements);
    }
    Ok(())
}

fn inverse_sqrt(value: f64) -> f64 {
    1.0 / value.sqrt()
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, GeodeError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, GeodeError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(GeodeError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(GeodeError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(GeodeError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq)]
pub enum GeodeError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("geode point count must be in 1..=20, got {0}")]
    InvalidPointCount(i32),
    #[error("geode outer distance must be in 1..=20, got {0}")]
    InvalidOuterDistance(i32),
    #[error("geode point offset must be in 0..=10, got {0}")]
    InvalidPointOffset(i32),
    #[error("geode layer values must be finite and in 0.01..=50")]
    InvalidLayer,
    #[error("geode crack settings are outside codec bounds")]
    InvalidCrack,
    #[error("geode placement chances must be finite and in 0..=1")]
    InvalidChance,
    #[error("geode noise multiplier must be finite and in 0..=1")]
    InvalidNoiseMultiplier,
    #[error("geode inner placements must be nonempty")]
    EmptyInnerPlacements,
    #[error("geode position overflow")]
    PositionOverflow,
}
