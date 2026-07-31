//! Large dripstone columns with bounded cave scans, wind, retreat, and ordered writes.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatRange {
    pub minimum: f32,
    pub maximum: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LargeDripstoneConfig {
    pub floor_to_ceiling_search_range: u32,
    pub radius_minimum: i32,
    pub radius_maximum: i32,
    pub maximum_radius_to_cave_height_ratio: f32,
    pub height_scale: FloatRange,
    pub stalactite_bluntness: FloatRange,
    pub stalagmite_bluntness: FloatRange,
    pub wind_speed: FloatRange,
    pub minimum_radius_for_wind: i32,
    pub minimum_bluntness_for_wind: f32,
    pub dripstone_block: BlockStateId,
}

pub trait LargeDripstoneWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_water_block(&self, state: BlockStateId) -> bool;

    fn is_lava_block(&self, state: BlockStateId) -> bool;

    fn is_dripstone_block(&self, state: BlockStateId) -> bool;

    fn is_replaceable_dripstone_block(&self, state: BlockStateId) -> bool;

    fn is_base_stone_overworld(&self, state: BlockStateId) -> bool;

    fn world_surface_worldgen_height(&mut self, x: i32, z: i32) -> i32;

    fn offer_large_dripstone(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        flags: u32,
    ) -> bool;
}

pub fn place_large_dripstone<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: LargeDripstoneConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, LargeDripstoneError>
where
    R: GenerationRandom,
    W: LargeDripstoneWorld,
{
    validate_config(config)?;
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let origin_state = world.block_state(origin);
    if !is_air_or_water(world, origin_state) {
        return Ok(false);
    }
    let Some(ceiling_y) = scan_boundary(
        world,
        origin,
        Direction::Up,
        config.floor_to_ceiling_search_range,
    )?
    else {
        return Ok(false);
    };
    let Some(floor_y) = scan_boundary(
        world,
        origin,
        Direction::Down,
        config.floor_to_ceiling_search_range,
    )?
    else {
        return Ok(false);
    };
    let cave_height = ceiling_y
        .checked_sub(floor_y)
        .and_then(|value| value.checked_sub(1))
        .ok_or(LargeDripstoneError::PositionOverflow)?;
    if cave_height < 4 {
        return Ok(false);
    }
    let raw_radius = (cave_height as f32 * config.maximum_radius_to_cave_height_ratio) as i32;
    let capped_maximum = raw_radius.clamp(config.radius_minimum, config.radius_maximum);
    let radius = sample_inclusive(random, config.radius_minimum, capped_maximum)?;
    let stalactite_base_y = ceiling_y
        .checked_sub(1)
        .ok_or(LargeDripstoneError::PositionOverflow)?;
    let mut stalactite = DripstoneShape {
        base: BlockPos::new(origin.x, stalactite_base_y, origin.z),
        points_down: true,
        radius,
        bluntness: sample_float(config.stalactite_bluntness, random),
        scale: sample_float(config.height_scale, random),
    };
    let stalagmite_base_y = floor_y
        .checked_add(1)
        .ok_or(LargeDripstoneError::PositionOverflow)?;
    let mut stalagmite = DripstoneShape {
        base: BlockPos::new(origin.x, stalagmite_base_y, origin.z),
        points_down: false,
        radius,
        bluntness: sample_float(config.stalagmite_bluntness, random),
        scale: sample_float(config.height_scale, random),
    };
    let wind = if suitable_for_wind(&stalactite, config) && suitable_for_wind(&stalagmite, config) {
        let speed = sample_float(config.wind_speed, random);
        let angle = random.next_f32() * std::f32::consts::PI;
        Wind {
            x: angle.cos() * speed,
            z: angle.sin() * speed,
            maximum_offset: 16 - radius,
            origin_y: origin.y,
        }
    } else {
        Wind::none(origin.y, radius)
    };

    let stalactite_ready = retreat_into_stone(world, &mut stalactite, wind)?;
    let stalagmite_ready = retreat_into_stone(world, &mut stalagmite, wind)?;
    if stalactite_ready {
        place_shape(world, stalactite, wind, config.dripstone_block, random)?;
    }
    if stalagmite_ready {
        place_shape(world, stalagmite, wind, config.dripstone_block, random)?;
    }
    Ok(true)
}

#[derive(Debug, Clone, Copy)]
struct DripstoneShape {
    base: BlockPos,
    points_down: bool,
    radius: i32,
    bluntness: f32,
    scale: f32,
}

impl DripstoneShape {
    fn height_at_distance(self, distance: f64) -> i32 {
        if distance > f64::from(self.radius) || self.radius <= 0 {
            return 0;
        }
        let distance = distance.max(f64::from(self.bluntness));
        let q = 0.384 * distance / f64::from(self.radius);
        let value = f64::from(self.scale)
            * (0.75 * q.powf(4.0 / 3.0) - q.powf(2.0 / 3.0) - q.ln() / 3.0)
            * f64::from(self.radius)
            / 0.384;
        value.max(0.0) as i32
    }

    const fn outward(self) -> Direction {
        if self.points_down {
            Direction::Down
        } else {
            Direction::Up
        }
    }

    const fn backing(self) -> Direction {
        self.outward().opposite()
    }
}

#[derive(Debug, Clone, Copy)]
struct Wind {
    x: f32,
    z: f32,
    maximum_offset: i32,
    origin_y: i32,
}

impl Wind {
    const fn none(origin_y: i32, radius: i32) -> Self {
        Self {
            x: 0.0,
            z: 0.0,
            maximum_offset: 16 - radius,
            origin_y,
        }
    }

    fn offset(self, position: BlockPos) -> Result<BlockPos, LargeDripstoneError> {
        let dy = self
            .origin_y
            .checked_sub(position.y)
            .ok_or(LargeDripstoneError::PositionOverflow)?;
        let x_offset = (self.x * dy as f32)
            .floor()
            .clamp(-self.maximum_offset as f32, self.maximum_offset as f32)
            as i32;
        let z_offset = (self.z * dy as f32)
            .floor()
            .clamp(-self.maximum_offset as f32, self.maximum_offset as f32)
            as i32;
        offset_xyz(position, x_offset, 0, z_offset)
    }
}

fn scan_boundary<W: LargeDripstoneWorld>(
    world: &mut W,
    origin: BlockPos,
    direction: Direction,
    range: u32,
) -> Result<Option<i32>, LargeDripstoneError> {
    let mut cursor = origin;
    let mut distance = 1_u32;
    while distance < range {
        let state = world.block_state(cursor);
        if !is_air_or_water(world, state) {
            break;
        }
        cursor = offset(cursor, direction)?;
        distance += 1;
    }
    let state = world.block_state(cursor);
    Ok(is_boundary(world, state).then_some(cursor.y))
}

fn retreat_into_stone<W: LargeDripstoneWorld>(
    world: &mut W,
    shape: &mut DripstoneShape,
    wind: Wind,
) -> Result<bool, LargeDripstoneError> {
    let original_base = shape.base;
    while shape.radius > 1 {
        let maximum_probes = shape.height_at_distance(0.0).min(10);
        let mut probe = original_base;
        for _ in 0..maximum_probes {
            probe = offset(probe, shape.backing())?;
            let unshifted_state = world.block_state(probe);
            if world.is_lava_block(unshifted_state) {
                return Ok(false);
            }
            let shifted = wind.offset(probe)?;
            if !embedded_circle(world, shifted, shape.radius)? {
                continue;
            }
            shape.base = probe;
            return Ok(true);
        }
        shape.radius /= 2;
    }
    Ok(false)
}

fn embedded_circle<W: LargeDripstoneWorld>(
    world: &mut W,
    center: BlockPos,
    radius: i32,
) -> Result<bool, LargeDripstoneError> {
    let center_state = world.block_state(center);
    if is_open(world, center_state) {
        return Ok(false);
    }
    let step = 6.0 / f64::from(radius);
    let mut angle = 0.0;
    while angle < std::f64::consts::TAU {
        let x_offset = (angle.cos() * f64::from(radius)) as i32;
        let z_offset = (angle.sin() * f64::from(radius)) as i32;
        let position = offset_xyz(center, x_offset, 0, z_offset)?;
        let state = world.block_state(position);
        if is_open(world, state) {
            return Ok(false);
        }
        angle += step;
    }
    Ok(true)
}

fn place_shape<R, W>(
    world: &mut W,
    shape: DripstoneShape,
    wind: Wind,
    dripstone: BlockStateId,
    random: &mut R,
) -> Result<(), LargeDripstoneError>
where
    R: GenerationRandom,
    W: LargeDripstoneWorld,
{
    for x_offset in -shape.radius..=shape.radius {
        for z_offset in -shape.radius..=shape.radius {
            let distance = f64::hypot(f64::from(x_offset), f64::from(z_offset));
            let mut height = shape.height_at_distance(distance);
            if height <= 0 {
                continue;
            }
            if random.next_f32() < 0.2 {
                let factor = 0.8 + random.next_f32() * 0.2;
                height = (height as f32 * factor) as i32;
            }
            let mut cursor = offset_xyz(shape.base, x_offset, 0, z_offset)?;
            let mut entered_open = false;
            for _ in 0..height {
                if !shape.points_down
                    && cursor.y >= world.world_surface_worldgen_height(cursor.x, cursor.z)
                {
                    break;
                }
                let shifted = wind.offset(cursor)?;
                let state = world.block_state(shifted);
                if is_open(world, state) {
                    entered_open = true;
                    let _ = world.offer_large_dripstone(shifted, dripstone, 2);
                } else if entered_open && world.is_base_stone_overworld(state) {
                    break;
                }
                cursor = offset(cursor, shape.outward())?;
            }
        }
    }
    Ok(())
}

fn suitable_for_wind(shape: &DripstoneShape, config: LargeDripstoneConfig) -> bool {
    shape.radius >= config.minimum_radius_for_wind
        && shape.bluntness >= config.minimum_bluntness_for_wind
}

fn sample_float(range: FloatRange, random: &mut impl GenerationRandom) -> f32 {
    range.minimum + random.next_f32() * (range.maximum - range.minimum)
}

fn sample_inclusive(
    random: &mut impl GenerationRandom,
    minimum: i32,
    maximum: i32,
) -> Result<i32, LargeDripstoneError> {
    let width = i64::from(maximum) - i64::from(minimum) + 1;
    let bound = u32::try_from(width)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(LargeDripstoneError::InvalidRadiusBounds)?;
    i64::from(minimum)
        .checked_add(i64::from(random.next_u32(bound)))
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(LargeDripstoneError::InvalidRadiusBounds)
}

fn validate_config(config: LargeDripstoneConfig) -> Result<(), LargeDripstoneError> {
    if config.floor_to_ceiling_search_range > 512 {
        return Err(LargeDripstoneError::SearchRangeOutOfRange);
    }
    if config.radius_minimum < 1 || config.radius_minimum > config.radius_maximum {
        return Err(LargeDripstoneError::InvalidRadiusBounds);
    }
    if !config.maximum_radius_to_cave_height_ratio.is_finite()
        || config.maximum_radius_to_cave_height_ratio < 0.0
        || !config.minimum_bluntness_for_wind.is_finite()
    {
        return Err(LargeDripstoneError::InvalidFloatConfiguration);
    }
    for range in [
        config.height_scale,
        config.stalactite_bluntness,
        config.stalagmite_bluntness,
        config.wind_speed,
    ] {
        if !range.minimum.is_finite() || !range.maximum.is_finite() || range.minimum > range.maximum
        {
            return Err(LargeDripstoneError::InvalidFloatConfiguration);
        }
    }
    Ok(())
}

fn is_air_or_water<W: LargeDripstoneWorld>(world: &W, state: BlockStateId) -> bool {
    world.is_air(state) || world.is_water_block(state)
}

fn is_open<W: LargeDripstoneWorld>(world: &W, state: BlockStateId) -> bool {
    is_air_or_water(world, state) || world.is_lava_block(state)
}

fn is_boundary<W: LargeDripstoneWorld>(world: &W, state: BlockStateId) -> bool {
    world.is_lava_block(state)
        || world.is_dripstone_block(state)
        || world.is_replaceable_dripstone_block(state)
}

fn offset(origin: BlockPos, direction: Direction) -> Result<BlockPos, LargeDripstoneError> {
    let [x, y, z] = direction.step();
    offset_xyz(origin, x, y, z)
}

fn offset_xyz(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, LargeDripstoneError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(LargeDripstoneError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(LargeDripstoneError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(LargeDripstoneError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LargeDripstoneError {
    #[error("large-dripstone search range is outside 0..=512")]
    SearchRangeOutOfRange,
    #[error("large-dripstone radius bounds are invalid")]
    InvalidRadiusBounds,
    #[error("large-dripstone float configuration is invalid")]
    InvalidFloatConfiguration,
    #[error("large-dripstone position arithmetic overflowed")]
    PositionOverflow,
}
