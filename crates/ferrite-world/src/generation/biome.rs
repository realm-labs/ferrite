//! Deterministic biome-source dispatch and shared query semantics.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BiomeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateInterval {
    pub minimum: i64,
    pub maximum: i64,
}

impl ClimateInterval {
    pub fn quantized(minimum: f32, maximum: f32) -> Self {
        Self {
            minimum: quantize(minimum),
            maximum: quantize(maximum),
        }
    }

    fn distance(self, value: i64) -> i64 {
        if value < self.minimum {
            self.minimum - value
        } else if value > self.maximum {
            value - self.maximum
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClimatePoint {
    pub parameters: [ClimateInterval; 6],
    pub offset: i64,
    pub biome: BiomeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndBiomes {
    pub center: BiomeId,
    pub highlands: BiomeId,
    pub midlands: BiomeId,
    pub small_islands: BiomeId,
    pub barrens: BiomeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiomeSource {
    Fixed(BiomeId),
    Checkerboard { biomes: Vec<BiomeId>, scale: u8 },
    MultiNoise { points: Vec<ClimatePoint> },
    TheEnd(EndBiomes),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MultiNoiseCache {
    last_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorizontalBiomeQuery {
    pub center: BlockPos,
    pub radius: i32,
    pub quart_step: i32,
    pub closest: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosestBiomeQuery {
    pub origin: BlockPos,
    pub radius: i32,
    pub horizontal_step: i32,
    pub vertical_step: i32,
    pub minimum_y: i32,
    pub maximum_y: i32,
}

pub trait ClimateSampler {
    fn sample_climate(&mut self, quart_x: i32, quart_y: i32, quart_z: i32) -> [f32; 6];

    fn sample_end_erosion(&mut self, block_x: i32, block_y: i32, block_z: i32) -> f64;
}

impl BiomeSource {
    pub fn validate(&self) -> Result<(), BiomeError> {
        match self {
            Self::Checkerboard { biomes, scale } if biomes.is_empty() || *scale > 62 => {
                Err(BiomeError::InvalidSource)
            }
            Self::MultiNoise { points } if points.is_empty() => Err(BiomeError::InvalidSource),
            _ => Ok(()),
        }
    }

    pub fn sample(
        &self,
        quart_x: i32,
        quart_y: i32,
        quart_z: i32,
        sampler: &mut impl ClimateSampler,
        cache: &mut MultiNoiseCache,
    ) -> Result<BiomeId, BiomeError> {
        self.validate()?;
        match self {
            Self::Fixed(biome) => Ok(*biome),
            Self::Checkerboard { biomes, scale } => {
                let shift = (u32::from(*scale) + 2) & 31;
                let sum = (quart_x >> shift).wrapping_add(quart_z >> shift);
                let index = sum.rem_euclid(biomes.len() as i32) as usize;
                Ok(biomes[index])
            }
            Self::MultiNoise { points } => {
                let sampled = sampler
                    .sample_climate(quart_x, quart_y, quart_z)
                    .map(quantize);
                let index = nearest_climate_point(points, sampled, cache);
                Ok(points[index].biome)
            }
            Self::TheEnd(biomes) => {
                let block_x = quart_x.wrapping_mul(4);
                let block_z = quart_z.wrapping_mul(4);
                let section_x = block_x >> 4;
                let section_z = block_z >> 4;
                if section_x.wrapping_mul(section_x) + section_z.wrapping_mul(section_z) <= 4_096 {
                    return Ok(biomes.center);
                }
                let erosion_x = section_x.wrapping_mul(2).wrapping_add(1).wrapping_mul(8);
                let erosion_y = quart_y.wrapping_mul(4);
                let erosion_z = section_z.wrapping_mul(2).wrapping_add(1).wrapping_mul(8);
                let erosion = sampler.sample_end_erosion(erosion_x, erosion_y, erosion_z);
                Ok(if erosion > 0.25 {
                    biomes.highlands
                } else if erosion >= -0.0625 {
                    biomes.midlands
                } else if erosion < -0.21875 {
                    biomes.small_islands
                } else {
                    biomes.barrens
                })
            }
        }
    }

    pub fn possible_biomes(&self) -> Vec<BiomeId> {
        let mut result = Vec::new();
        match self {
            Self::Fixed(biome) => push_distinct(&mut result, *biome),
            Self::Checkerboard { biomes, .. } => {
                for biome in biomes.iter().copied() {
                    push_distinct(&mut result, biome);
                }
            }
            Self::MultiNoise { points } => {
                for point in points {
                    push_distinct(&mut result, point.biome);
                }
            }
            Self::TheEnd(biomes) => {
                for biome in [
                    biomes.center,
                    biomes.highlands,
                    biomes.midlands,
                    biomes.small_islands,
                    biomes.barrens,
                ] {
                    push_distinct(&mut result, biome);
                }
            }
        }
        result
    }

    pub fn biomes_within(
        &self,
        center: BlockPos,
        radius: i32,
        sampler: &mut impl ClimateSampler,
        cache: &mut MultiNoiseCache,
    ) -> Result<Vec<BiomeId>, BiomeError> {
        if let Self::Fixed(biome) = self {
            return Ok(vec![*biome]);
        }
        let minimum_x = block_to_quart(center.x.saturating_sub(radius));
        let maximum_x = block_to_quart(center.x.saturating_add(radius));
        let minimum_y = block_to_quart(center.y.saturating_sub(radius));
        let maximum_y = block_to_quart(center.y.saturating_add(radius));
        let minimum_z = block_to_quart(center.z.saturating_sub(radius));
        let maximum_z = block_to_quart(center.z.saturating_add(radius));
        let mut result = Vec::new();
        for x in minimum_x..=maximum_x {
            for y in minimum_y..=maximum_y {
                for z in minimum_z..=maximum_z {
                    let biome = self.sample(x, y, z, sampler, cache)?;
                    push_distinct(&mut result, biome);
                }
            }
        }
        Ok(result)
    }

    pub fn find_horizontal<R, S, P>(
        &self,
        query: HorizontalBiomeQuery,
        sampler: &mut S,
        cache: &mut MultiNoiseCache,
        random: &mut R,
        predicate: P,
    ) -> Result<Option<(BlockPos, BiomeId)>, BiomeError>
    where
        R: GenerationRandom,
        S: ClimateSampler,
        P: Fn(BiomeId) -> bool,
    {
        if query.quart_step <= 0 {
            return Err(BiomeError::InvalidStep);
        }
        if let Self::Fixed(biome) = self {
            if !predicate(*biome) {
                return Ok(None);
            }
            if query.closest {
                return Ok(Some((query.center, *biome)));
            }
            let width = inclusive_width(-query.radius, query.radius)?;
            let x = query
                .center
                .x
                .checked_add(random.next_u32(width) as i32 - query.radius)
                .ok_or(BiomeError::PositionOverflow)?;
            let z = query
                .center
                .z
                .checked_add(random.next_u32(width) as i32 - query.radius)
                .ok_or(BiomeError::PositionOverflow)?;
            return Ok(Some((BlockPos::new(x, query.center.y, z), *biome)));
        }
        let center_x = block_to_quart(query.center.x);
        let center_y = block_to_quart(query.center.y);
        let center_z = block_to_quart(query.center.z);
        let quart_radius = block_to_quart(query.radius);
        if query.closest {
            for ring in (0..=quart_radius).step_by(query.quart_step as usize) {
                for z in -ring..=ring {
                    for x in -ring..=ring {
                        if ring != 0 && x != -ring && x != ring && z != -ring && z != ring {
                            continue;
                        }
                        let quart_x = center_x + x;
                        let quart_z = center_z + z;
                        let biome = self.sample(quart_x, center_y, quart_z, sampler, cache)?;
                        if predicate(biome) {
                            return Ok(Some((
                                BlockPos::new(quart_x * 4, query.center.y, quart_z * 4),
                                biome,
                            )));
                        }
                    }
                }
            }
            return Ok(None);
        }
        let mut winner = None;
        let mut matches_seen = 0_u32;
        for z in -quart_radius..=quart_radius {
            for x in -quart_radius..=quart_radius {
                let quart_x = center_x + x;
                let quart_z = center_z + z;
                let biome = self.sample(quart_x, center_y, quart_z, sampler, cache)?;
                if !predicate(biome) {
                    continue;
                }
                if matches_seen == 0
                    || random.next_u32(
                        NonZeroU32::new(matches_seen + 1).expect("match count is nonzero"),
                    ) == 0
                {
                    winner = Some((
                        BlockPos::new(quart_x * 4, query.center.y, quart_z * 4),
                        biome,
                    ));
                }
                matches_seen += 1;
            }
        }
        Ok(winner)
    }

    pub fn find_closest_3d<S, P>(
        &self,
        query: ClosestBiomeQuery,
        sampler: &mut S,
        cache: &mut MultiNoiseCache,
        predicate: P,
    ) -> Result<Option<(BlockPos, BiomeId)>, BiomeError>
    where
        S: ClimateSampler,
        P: Fn(BiomeId) -> bool,
    {
        if query.horizontal_step <= 0 || query.vertical_step <= 0 {
            return Err(BiomeError::InvalidStep);
        }
        let possible = self
            .possible_biomes()
            .into_iter()
            .filter(|biome| predicate(*biome))
            .collect::<Vec<_>>();
        if possible.is_empty() {
            return Ok(None);
        }
        if let Self::Fixed(biome) = self {
            return Ok(Some((
                BlockPos::new(
                    query.origin.x,
                    query
                        .origin
                        .y
                        .clamp(query.minimum_y + 1, query.maximum_y + 1),
                    query.origin.z,
                ),
                *biome,
            )));
        }
        let spiral_radius = query.radius.div_euclid(query.horizontal_step);
        for (offset_x, offset_z) in spiral(spiral_radius) {
            let x = query
                .origin
                .x
                .checked_add(offset_x * query.horizontal_step)
                .ok_or(BiomeError::PositionOverflow)?;
            let z = query
                .origin
                .z
                .checked_add(offset_z * query.horizontal_step)
                .ok_or(BiomeError::PositionOverflow)?;
            for y in out_from_origin(
                query.origin.y,
                query.minimum_y + 1,
                query.maximum_y + 1,
                query.vertical_step,
            )? {
                let biome = self.sample(
                    block_to_quart(x),
                    block_to_quart(y),
                    block_to_quart(z),
                    sampler,
                    cache,
                )?;
                if possible.contains(&biome) {
                    return Ok(Some((BlockPos::new(x, y, z), biome)));
                }
            }
        }
        Ok(None)
    }
}

fn nearest_climate_point(
    points: &[ClimatePoint],
    sampled: [i64; 6],
    cache: &mut MultiNoiseCache,
) -> usize {
    let cached = cache.last_index.unwrap_or(0).min(points.len() - 1);
    let mut winner = cached;
    let mut distance = climate_distance(&points[cached], sampled);
    for (index, point) in points.iter().enumerate() {
        let candidate = climate_distance(point, sampled);
        if candidate < distance {
            distance = candidate;
            winner = index;
        }
    }
    cache.last_index = Some(winner);
    winner
}

fn climate_distance(point: &ClimatePoint, sampled: [i64; 6]) -> i128 {
    point
        .parameters
        .iter()
        .zip(sampled)
        .map(|(interval, value)| i128::from(interval.distance(value)).pow(2))
        .sum::<i128>()
        + i128::from(point.offset).pow(2)
}

fn quantize(value: f32) -> i64 {
    (value * 10_000.0) as i64
}

fn block_to_quart(value: i32) -> i32 {
    value >> 2
}

fn inclusive_width(minimum: i32, maximum: i32) -> Result<NonZeroU32, BiomeError> {
    let width = i64::from(maximum) - i64::from(minimum) + 1;
    u32::try_from(width)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(BiomeError::PositionOverflow)
}

fn push_distinct(values: &mut Vec<BiomeId>, biome: BiomeId) {
    if !values.contains(&biome) {
        values.push(biome);
    }
}

fn spiral(radius: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut cursor = (0, 1);
    let mut leg = -1_i32;
    let mut leg_size = 0_i32;
    let mut leg_index = 0_i32;
    let legs = 4 * radius;
    loop {
        let direction = directions[((leg + 4) % 4) as usize];
        cursor.0 += direction.0;
        cursor.1 += direction.1;
        if leg_index >= leg_size {
            if leg >= legs {
                break;
            }
            leg += 1;
            leg_index = 0;
            leg_size = leg / 2 + 1;
        }
        leg_index += 1;
        result.push(cursor);
    }
    result
}

fn out_from_origin(origin: i32, lower: i32, upper: i32, step: i32) -> Result<Vec<i32>, BiomeError> {
    if lower > upper || step < 1 {
        return Err(BiomeError::InvalidStep);
    }
    let center = origin.clamp(lower, upper);
    let mut result = Vec::new();
    let mut cursor = center;
    loop {
        let distance = center.abs_diff(cursor) as i32;
        if center - distance < lower && center + distance > upper {
            break;
        }
        result.push(cursor);
        let previous_was_negative = cursor <= center;
        let can_move_positive = center + distance + step <= upper;
        let attempted_negative = center - distance - if previous_was_negative { step } else { 0 };
        cursor = if previous_was_negative && can_move_positive || attempted_negative < lower {
            center + distance + step
        } else {
            attempted_negative
        };
    }
    Ok(result)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BiomeError {
    #[error("biome-source configuration violates codec bounds")]
    InvalidSource,
    #[error("biome query step must be positive")]
    InvalidStep,
    #[error("biome query position or range overflow")]
    PositionOverflow,
}
