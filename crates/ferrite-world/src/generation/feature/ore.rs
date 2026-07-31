//! Ore configured features and their shared target/exposure admission.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

pub trait OreTargetRule<R: GenerationRandom> {
    fn matches(&mut self, state: BlockStateId, random: &mut R) -> bool;

    fn output_state(&self) -> BlockStateId;
}

pub trait ScatteredOreWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn offer_ore(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OreConfig {
    pub size: u8,
    pub discard_chance_on_air_exposure: f32,
}

pub trait OreVolumeWorld: ScatteredOreWorld {
    fn ocean_floor_worldgen_height(&mut self, x: i32, z: i32) -> i32;

    fn is_outside_build_height(&self, position: BlockPos) -> bool;

    fn can_write_ore(&mut self, position: BlockPos) -> bool;

    fn acquire_ore_section(&mut self, position: BlockPos) -> bool;

    fn set_ore_state_direct(&mut self, position: BlockPos, state: BlockStateId);

    fn release_ore_sections(&mut self);
}

pub fn place_scattered_ore<R, W, T>(
    world: &mut W,
    origin: BlockPos,
    size: u8,
    discard_chance_on_air_exposure: f32,
    targets: &mut [T],
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, OreFeatureError>
where
    R: GenerationRandom,
    W: ScatteredOreWorld,
    T: OreTargetRule<R>,
{
    if size > 64 {
        return Err(OreFeatureError::SizeOutOfRange { size });
    }
    if !discard_chance_on_air_exposure.is_finite()
        || !(0.0..=1.0).contains(&discard_chance_on_air_exposure)
    {
        return Err(OreFeatureError::InvalidDiscardChance);
    }
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let attempt_bound =
        NonZeroU32::new(u32::from(size) + 1).expect("scattered-ore attempt bound is nonzero");
    let attempts = random.next_u32(attempt_bound);
    for attempt in 0..attempts {
        let distance = attempt.min(7) as f32;
        let x_offset = triangular_offset(random, distance);
        let y_offset = triangular_offset(random, distance);
        let z_offset = triangular_offset(random, distance);
        let position = offset(origin, x_offset, y_offset, z_offset)?;
        let current = world.block_state(position);
        for target in targets.iter_mut() {
            if !target.matches(current, random) {
                continue;
            }
            if !ore_exposure_admits(world, position, discard_chance_on_air_exposure, random)? {
                continue;
            }
            let _ = world.offer_ore(position, target.output_state(), 2);
            break;
        }
    }
    Ok(true)
}

pub fn place_ore<R, W, T>(
    world: &mut W,
    origin: BlockPos,
    config: OreConfig,
    targets: &mut [T],
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, OreFeatureError>
where
    R: GenerationRandom,
    W: OreVolumeWorld,
    T: OreTargetRule<R>,
{
    validate_ore_config(config)?;
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let angle = random.next_f32() * std::f32::consts::PI;
    let size = i32::from(config.size);
    let horizontal_half_span = size as f32 / 8.0;
    let padding = (size as f32 / 16.0 + 0.5).ceil() as i32;
    let horizontal_radius = horizontal_half_span.ceil() as i32;
    let minimum_x = origin
        .x
        .checked_sub(horizontal_radius)
        .and_then(|value| value.checked_sub(padding))
        .ok_or(OreFeatureError::PositionOverflow)?;
    let minimum_y = origin
        .y
        .checked_sub(2)
        .and_then(|value| value.checked_sub(padding))
        .ok_or(OreFeatureError::PositionOverflow)?;
    let minimum_z = origin
        .z
        .checked_sub(horizontal_radius)
        .and_then(|value| value.checked_sub(padding))
        .ok_or(OreFeatureError::PositionOverflow)?;
    let horizontal_extent = horizontal_radius
        .checked_add(padding)
        .and_then(|value| value.checked_mul(2))
        .ok_or(OreFeatureError::PositionOverflow)?;
    let vertical_extent = padding
        .checked_add(2)
        .and_then(|value| value.checked_mul(2))
        .ok_or(OreFeatureError::PositionOverflow)?;

    let sin = angle.sin() * horizontal_half_span;
    let cos = angle.cos() * horizontal_half_span;
    let first_x = f64::from(origin.x) + f64::from(sin);
    let second_x = f64::from(origin.x) - f64::from(sin);
    let first_z = f64::from(origin.z) + f64::from(cos);
    let second_z = f64::from(origin.z) - f64::from(cos);
    let first_y = origin
        .y
        .checked_add(
            random.next_u32(NonZeroU32::new(3).expect("ore endpoint bound is nonzero")) as i32,
        )
        .and_then(|value| value.checked_sub(2))
        .ok_or(OreFeatureError::PositionOverflow)?;
    let second_y = origin
        .y
        .checked_add(
            random.next_u32(NonZeroU32::new(3).expect("ore endpoint bound is nonzero")) as i32,
        )
        .and_then(|value| value.checked_sub(2))
        .ok_or(OreFeatureError::PositionOverflow)?;

    let maximum_x = minimum_x
        .checked_add(horizontal_extent)
        .ok_or(OreFeatureError::PositionOverflow)?;
    let maximum_z = minimum_z
        .checked_add(horizontal_extent)
        .ok_or(OreFeatureError::PositionOverflow)?;
    let mut surface_admitted = false;
    'surface: for x in minimum_x..=maximum_x {
        for z in minimum_z..=maximum_z {
            if world.ocean_floor_worldgen_height(x, z) >= minimum_y {
                surface_admitted = true;
                break 'surface;
            }
        }
    }
    if !surface_admitted {
        return Ok(false);
    }

    let nodes = build_and_prune_nodes(
        config.size,
        [first_x, f64::from(first_y), first_z],
        [second_x, f64::from(second_y), second_z],
        random,
    );
    let result = place_ore_nodes(
        world,
        &nodes,
        OreVolumeBounds {
            minimum: [minimum_x, minimum_y, minimum_z],
            horizontal_extent,
            vertical_extent,
            discard_chance: config.discard_chance_on_air_exposure,
        },
        targets,
        random,
    );
    world.release_ore_sections();
    result
}

#[derive(Debug, Clone, Copy)]
struct OreNode {
    center: [f64; 3],
    radius: f64,
}

#[derive(Debug, Clone, Copy)]
struct OreVolumeBounds {
    minimum: [i32; 3],
    horizontal_extent: i32,
    vertical_extent: i32,
    discard_chance: f32,
}

fn build_and_prune_nodes(
    size: u8,
    first: [f64; 3],
    second: [f64; 3],
    random: &mut impl GenerationRandom,
) -> Vec<OreNode> {
    let mut nodes = Vec::with_capacity(usize::from(size));
    for index in 0..size {
        let fraction = f32::from(index) / f32::from(size);
        let fraction64 = f64::from(fraction);
        let q = random.next_f64() * f64::from(size) / 16.0;
        let radius = ((f64::sin(std::f64::consts::PI * fraction64) + 1.0) * q + 1.0) / 2.0;
        nodes.push(OreNode {
            center: [
                lerp(fraction64, first[0], second[0]),
                lerp(fraction64, first[1], second[1]),
                lerp(fraction64, first[2], second[2]),
            ],
            radius,
        });
    }
    for outer in 0..nodes.len().saturating_sub(1) {
        if nodes[outer].radius <= 0.0 {
            continue;
        }
        for inner in (outer + 1)..nodes.len() {
            if nodes[inner].radius <= 0.0 {
                continue;
            }
            let radius_difference = nodes[outer].radius - nodes[inner].radius;
            let center_distance = nodes[outer]
                .center
                .into_iter()
                .zip(nodes[inner].center)
                .map(|(left, right)| {
                    let difference = left - right;
                    difference * difference
                })
                .sum::<f64>();
            if radius_difference * radius_difference > center_distance {
                if radius_difference > 0.0 {
                    nodes[inner].radius = -1.0;
                } else {
                    nodes[outer].radius = -1.0;
                }
            }
        }
    }
    nodes
}

fn place_ore_nodes<R, W, T>(
    world: &mut W,
    nodes: &[OreNode],
    bounds: OreVolumeBounds,
    targets: &mut [T],
    random: &mut R,
) -> Result<bool, OreFeatureError>
where
    R: GenerationRandom,
    W: OreVolumeWorld,
    T: OreTargetRule<R>,
{
    let mut visited = BTreeSet::new();
    let mut writes = 0_u32;
    for node in nodes.iter().filter(|node| node.radius > 0.0) {
        let lower = [
            (node.center[0] - node.radius).floor() as i32,
            (node.center[1] - node.radius).floor() as i32,
            (node.center[2] - node.radius).floor() as i32,
        ];
        let lower = [
            lower[0].max(bounds.minimum[0]),
            lower[1].max(bounds.minimum[1]),
            lower[2].max(bounds.minimum[2]),
        ];
        let upper = [
            ((node.center[0] + node.radius).floor() as i32).max(lower[0]),
            ((node.center[1] + node.radius).floor() as i32).max(lower[1]),
            ((node.center[2] + node.radius).floor() as i32).max(lower[2]),
        ];
        for x in lower[0]..=upper[0] {
            let normalized_x = (f64::from(x) + 0.5 - node.center[0]) / node.radius;
            let x_squared = normalized_x * normalized_x;
            if x_squared >= 1.0 {
                continue;
            }
            for y in lower[1]..=upper[1] {
                let normalized_y = (f64::from(y) + 0.5 - node.center[1]) / node.radius;
                let xy_squared = x_squared + normalized_y * normalized_y;
                if xy_squared >= 1.0 {
                    continue;
                }
                for z in lower[2]..=upper[2] {
                    let normalized_z = (f64::from(z) + 0.5 - node.center[2]) / node.radius;
                    if xy_squared + normalized_z * normalized_z >= 1.0 {
                        continue;
                    }
                    let position = BlockPos::new(x, y, z);
                    if world.is_outside_build_height(position) {
                        continue;
                    }
                    let bit_index = ore_bit_index(
                        position,
                        bounds.minimum,
                        bounds.horizontal_extent,
                        bounds.vertical_extent,
                    )?;
                    if !visited.insert(bit_index) {
                        continue;
                    }
                    if !world.can_write_ore(position) || !world.acquire_ore_section(position) {
                        continue;
                    }
                    let current = world.block_state(position);
                    for target in targets.iter_mut() {
                        if !target.matches(current, random)
                            || !ore_exposure_admits(world, position, bounds.discard_chance, random)?
                        {
                            continue;
                        }
                        world.set_ore_state_direct(position, target.output_state());
                        writes += 1;
                        break;
                    }
                }
            }
        }
    }
    Ok(writes != 0)
}

fn ore_bit_index(
    position: BlockPos,
    minimum: [i32; 3],
    horizontal_extent: i32,
    vertical_extent: i32,
) -> Result<usize, OreFeatureError> {
    let x = i64::from(position.x) - i64::from(minimum[0]);
    let y = i64::from(position.y) - i64::from(minimum[1]);
    let z = i64::from(position.z) - i64::from(minimum[2]);
    let horizontal = i64::from(horizontal_extent);
    let vertical = i64::from(vertical_extent);
    x.checked_add(
        y.checked_mul(horizontal)
            .ok_or(OreFeatureError::PositionOverflow)?,
    )
    .and_then(|value| {
        z.checked_mul(horizontal)
            .and_then(|z_value| z_value.checked_mul(vertical))
            .and_then(|z_value| value.checked_add(z_value))
    })
    .and_then(|value| usize::try_from(value).ok())
    .ok_or(OreFeatureError::PositionOverflow)
}

fn validate_ore_config(config: OreConfig) -> Result<(), OreFeatureError> {
    if config.size > 64 {
        return Err(OreFeatureError::SizeOutOfRange { size: config.size });
    }
    if !config.discard_chance_on_air_exposure.is_finite()
        || !(0.0..=1.0).contains(&config.discard_chance_on_air_exposure)
    {
        return Err(OreFeatureError::InvalidDiscardChance);
    }
    Ok(())
}

fn lerp(fraction: f64, first: f64, second: f64) -> f64 {
    first + fraction * (second - first)
}

fn triangular_offset(random: &mut impl GenerationRandom, distance: f32) -> i32 {
    java_float_round((random.next_f32() - random.next_f32()) * distance)
}

fn java_float_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

fn ore_exposure_admits<R, W>(
    world: &mut W,
    position: BlockPos,
    chance: f32,
    random: &mut R,
) -> Result<bool, OreFeatureError>
where
    R: GenerationRandom,
    W: ScatteredOreWorld,
{
    if chance <= 0.0 {
        return Ok(true);
    }
    if chance < 1.0 && random.next_f32() >= chance {
        return Ok(true);
    }
    for (x, y, z) in [
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
        (-1, 0, 0),
        (1, 0, 0),
    ] {
        let neighbor = offset(position, x, y, z)?;
        let state = world.block_state(neighbor);
        if world.is_air(state) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, OreFeatureError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(OreFeatureError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(OreFeatureError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(OreFeatureError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OreFeatureError {
    #[error("ore size {size} is outside 0..=64")]
    SizeOutOfRange { size: u8 },
    #[error("ore discard chance must be finite and in the inclusive range 0..=1")]
    InvalidDiscardChance,
    #[error("ore position arithmetic overflowed")]
    PositionOverflow,
}
