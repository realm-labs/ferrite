//! Deterministic cave, Nether-cave, and canyon path construction.

use std::num::NonZeroU32;
use std::sync::Arc;

use thiserror::Error;

use crate::generation::carver::CarverEllipsoid;
use crate::generation::feature::provider::{
    HeightAnchor, HeightContext, ProviderError, uniform_height,
};
use crate::generation::feature::random::{GenerationRandom, LegacyRandom};

const PATH_BOUND: i32 = 112;
const FLOAT_PI: f32 = std::f32::consts::PI;
const FLOAT_TAU: f32 = FLOAT_PI * 2.0;
const FLOAT_HALF_PI: f32 = 1.570_796_4;

pub trait CarverRandom: GenerationRandom {
    fn next_i64(&mut self) -> i64;
}

impl CarverRandom for LegacyRandom {
    fn next_i64(&mut self) -> i64 {
        LegacyRandom::next_i64(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CarverFloatProvider {
    Constant(f32),
    Uniform {
        minimum: f32,
        maximum: f32,
    },
    Trapezoid {
        minimum: f32,
        maximum: f32,
        plateau: f32,
    },
}

impl CarverFloatProvider {
    pub fn sample(self, random: &mut impl GenerationRandom) -> Result<f32, CarverPathError> {
        match self {
            Self::Constant(value) if value.is_finite() => Ok(value),
            Self::Uniform { minimum, maximum }
                if minimum.is_finite() && maximum.is_finite() && minimum <= maximum =>
            {
                Ok(minimum + random.next_f32() * (maximum - minimum))
            }
            Self::Trapezoid {
                minimum,
                maximum,
                plateau,
            } if minimum.is_finite()
                && maximum.is_finite()
                && plateau.is_finite()
                && minimum <= maximum
                && plateau >= 0.0
                && plateau <= maximum - minimum =>
            {
                let sloped = maximum - minimum - plateau;
                Ok(random.next_f32() * sloped + random.next_f32() * plateau + minimum)
            }
            _ => Err(CarverPathError::InvalidFloatProvider),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarverHeightProvider {
    Constant(HeightAnchor),
    Uniform {
        minimum: HeightAnchor,
        maximum: HeightAnchor,
    },
}

impl CarverHeightProvider {
    fn sample(
        self,
        context: HeightContext,
        random: &mut impl GenerationRandom,
    ) -> Result<i32, CarverPathError> {
        match self {
            Self::Constant(anchor) => Ok(anchor.resolve(context)?),
            Self::Uniform { minimum, maximum } => {
                Ok(uniform_height(minimum, maximum, context, random)?)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveFamily {
    Ordinary,
    Nether,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CavePathConfig {
    pub probability: f32,
    pub y: CarverHeightProvider,
    pub horizontal_radius_multiplier: CarverFloatProvider,
    pub vertical_radius_multiplier: CarverFloatProvider,
    pub floor_level: CarverFloatProvider,
    pub y_scale: CarverFloatProvider,
    pub family: CaveFamily,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanyonShapeConfig {
    pub distance_factor: CarverFloatProvider,
    pub thickness: CarverFloatProvider,
    pub horizontal_radius_factor: CarverFloatProvider,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
    pub width_smoothness: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanyonPathConfig {
    pub probability: f32,
    pub y: CarverHeightProvider,
    pub vertical_rotation: CarverFloatProvider,
    pub y_scale: CarverFloatProvider,
    pub shape: CanyonShapeConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CarverSkip {
    Cave {
        floor_level: f64,
    },
    Canyon {
        minimum_y: i32,
        width_factors: Arc<[f32]>,
    },
}

impl CarverSkip {
    pub fn should_skip(&self, x: f64, y: f64, z: f64, world_y: i32) -> bool {
        match self {
            Self::Cave { floor_level } => y <= *floor_level || x * x + y * y + z * z >= 1.0,
            Self::Canyon {
                minimum_y,
                width_factors,
            } => {
                let index = world_y - *minimum_y - 1;
                let Ok(index) = usize::try_from(index) else {
                    return true;
                };
                let Some(width) = width_factors.get(index) else {
                    return true;
                };
                (x * x + z * z) * f64::from(*width) + y * y / 6.0 >= 1.0
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CarverVolume {
    pub ellipsoid: CarverEllipsoid,
    pub skip: CarverSkip,
}

pub fn carve_cave_source<R>(
    random: &mut R,
    height: HeightContext,
    source_chunk: [i32; 2],
    target_chunk: [i32; 2],
    config: &CavePathConfig,
    emit: impl FnMut(CarverVolume),
) -> Result<bool, CarverPathError>
where
    R: CarverRandom,
{
    validate_probability(config.probability)?;
    if random.next_f32() > config.probability {
        return Ok(false);
    }
    generate_cave_path(random, height, source_chunk, target_chunk, config, emit)?;
    Ok(true)
}

pub fn generate_cave_path<R>(
    random: &mut R,
    height: HeightContext,
    source_chunk: [i32; 2],
    target_chunk: [i32; 2],
    config: &CavePathConfig,
    mut emit: impl FnMut(CarverVolume),
) -> Result<(), CarverPathError>
where
    R: CarverRandom,
{
    let cave_bound = match config.family {
        CaveFamily::Ordinary => 15,
        CaveFamily::Nether => 10,
    };
    let outer = bounded(random, cave_bound);
    let middle = bounded(random, outer + 1);
    let cave_count = bounded(random, middle + 1);
    let source_minimum_x = source_chunk[0]
        .checked_mul(16)
        .ok_or(CarverPathError::PositionOverflow)?;
    let source_minimum_z = source_chunk[1]
        .checked_mul(16)
        .ok_or(CarverPathError::PositionOverflow)?;

    for _ in 0..cave_count {
        let x = f64::from(source_minimum_x + bounded(random, 16));
        let y = f64::from(config.y.sample(height, random)?);
        let z = f64::from(source_minimum_z + bounded(random, 16));
        let horizontal_multiplier = f64::from(config.horizontal_radius_multiplier.sample(random)?);
        let vertical_multiplier = f64::from(config.vertical_radius_multiplier.sample(random)?);
        let floor_level = f64::from(config.floor_level.sample(random)?);
        let skip = CarverSkip::Cave { floor_level };
        let mut tunnels = 1;
        if bounded(random, 4) == 0 {
            let y_scale = f64::from(config.y_scale.sample(random)?);
            let thickness = 1.0 + random.next_f32() * 6.0;
            let radius = 1.5 + f64::from(minecraft_sin(1.570_796_4) * thickness);
            emit(CarverVolume {
                ellipsoid: CarverEllipsoid {
                    center_x: x + 1.0,
                    center_y: y,
                    center_z: z,
                    horizontal_radius: radius,
                    vertical_radius: radius * y_scale,
                },
                skip: skip.clone(),
            });
            tunnels += bounded(random, 4);
        }
        for _ in 0..tunnels {
            let yaw = random.next_f32() * FLOAT_TAU;
            let pitch = (random.next_f32() - 0.5) / 4.0;
            let thickness = cave_thickness(config.family, random);
            let distance = PATH_BOUND - bounded(random, PATH_BOUND / 4);
            let seed = random.next_i64();
            tunnel(
                seed,
                Tunnel {
                    x,
                    y,
                    z,
                    horizontal_multiplier,
                    vertical_multiplier,
                    thickness,
                    yaw,
                    pitch,
                    first_step: 0,
                    distance,
                    y_scale: match config.family {
                        CaveFamily::Ordinary => 1.0,
                        CaveFamily::Nether => 5.0,
                    },
                },
                target_chunk,
                &skip,
                &mut emit,
            );
        }
    }
    Ok(())
}

pub fn carve_canyon_source<R>(
    random: &mut R,
    height: HeightContext,
    source_chunk: [i32; 2],
    target_chunk: [i32; 2],
    config: &CanyonPathConfig,
    emit: impl FnMut(CarverVolume),
) -> Result<bool, CarverPathError>
where
    R: CarverRandom,
{
    validate_canyon(config)?;
    if random.next_f32() > config.probability {
        return Ok(false);
    }
    generate_canyon_path(random, height, source_chunk, target_chunk, config, emit)?;
    Ok(true)
}

pub fn generate_canyon_path<R>(
    random: &mut R,
    height: HeightContext,
    source_chunk: [i32; 2],
    target_chunk: [i32; 2],
    config: &CanyonPathConfig,
    mut emit: impl FnMut(CarverVolume),
) -> Result<(), CarverPathError>
where
    R: CarverRandom,
{
    let source_minimum_x = source_chunk[0]
        .checked_mul(16)
        .ok_or(CarverPathError::PositionOverflow)?;
    let source_minimum_z = source_chunk[1]
        .checked_mul(16)
        .ok_or(CarverPathError::PositionOverflow)?;
    let x = f64::from(source_minimum_x + bounded(random, 16));
    let y = f64::from(config.y.sample(height, random)?);
    let z = f64::from(source_minimum_z + bounded(random, 16));
    let yaw = random.next_f32() * FLOAT_TAU;
    let pitch = config.vertical_rotation.sample(random)?;
    let y_scale = f64::from(config.y_scale.sample(random)?);
    let thickness = config.shape.thickness.sample(random)?;
    let distance = (PATH_BOUND as f32 * config.shape.distance_factor.sample(random)?) as i32;
    let seed = random.next_i64();
    canyon(
        seed,
        Canyon {
            x,
            y,
            z,
            thickness,
            yaw,
            pitch,
            distance,
            y_scale,
        },
        target_chunk,
        height,
        &config.shape,
        &mut emit,
    );
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Tunnel {
    x: f64,
    y: f64,
    z: f64,
    horizontal_multiplier: f64,
    vertical_multiplier: f64,
    thickness: f32,
    yaw: f32,
    pitch: f32,
    first_step: i32,
    distance: i32,
    y_scale: f64,
}

fn tunnel(
    seed: i64,
    mut path: Tunnel,
    target_chunk: [i32; 2],
    skip: &CarverSkip,
    emit: &mut impl FnMut(CarverVolume),
) {
    let mut random = LegacyRandom::new(seed);
    let split_point = bounded(&mut random, path.distance / 2) + path.distance / 4;
    let steep = bounded(&mut random, 6) == 0;
    let mut pitch_drift = 0.0_f32;
    let mut yaw_drift = 0.0_f32;
    for step in path.first_step..path.distance {
        let phase = FLOAT_PI * step as f32 / path.distance as f32;
        let horizontal_radius = 1.5 + f64::from(minecraft_sin(phase) * path.thickness);
        let vertical_radius = horizontal_radius * path.y_scale;
        let cos_pitch = minecraft_cos(path.pitch);
        path.x += f64::from(minecraft_cos(path.yaw) * cos_pitch);
        path.y += f64::from(minecraft_sin(path.pitch));
        path.z += f64::from(minecraft_sin(path.yaw) * cos_pitch);
        path.pitch *= if steep { 0.92 } else { 0.7 };
        path.pitch += pitch_drift * 0.1;
        path.yaw += yaw_drift * 0.1;
        pitch_drift *= 0.9;
        yaw_drift *= 0.75;
        pitch_drift += (random.next_f32() - random.next_f32()) * random.next_f32() * 2.0;
        yaw_drift += (random.next_f32() - random.next_f32()) * random.next_f32() * 4.0;
        if step == split_point && path.thickness > 1.0 {
            let left_seed = random.next_i64();
            let left_thickness = random.next_f32() * 0.5 + 0.5;
            tunnel(
                left_seed,
                Tunnel {
                    thickness: left_thickness,
                    yaw: path.yaw - FLOAT_HALF_PI,
                    pitch: path.pitch / 3.0,
                    first_step: step,
                    y_scale: 1.0,
                    ..path
                },
                target_chunk,
                skip,
                emit,
            );
            let right_seed = random.next_i64();
            let right_thickness = random.next_f32() * 0.5 + 0.5;
            tunnel(
                right_seed,
                Tunnel {
                    thickness: right_thickness,
                    yaw: path.yaw + FLOAT_HALF_PI,
                    pitch: path.pitch / 3.0,
                    first_step: step,
                    y_scale: 1.0,
                    ..path
                },
                target_chunk,
                skip,
                emit,
            );
            return;
        }
        if bounded(&mut random, 4) == 0 {
            continue;
        }
        if !can_reach(
            target_chunk,
            path.x,
            path.z,
            step,
            path.distance,
            path.thickness,
        ) {
            return;
        }
        emit(CarverVolume {
            ellipsoid: CarverEllipsoid {
                center_x: path.x,
                center_y: path.y,
                center_z: path.z,
                horizontal_radius: horizontal_radius * path.horizontal_multiplier,
                vertical_radius: vertical_radius * path.vertical_multiplier,
            },
            skip: skip.clone(),
        });
    }
}

#[derive(Debug, Clone, Copy)]
struct Canyon {
    x: f64,
    y: f64,
    z: f64,
    thickness: f32,
    yaw: f32,
    pitch: f32,
    distance: i32,
    y_scale: f64,
}

fn canyon(
    seed: i64,
    mut path: Canyon,
    target_chunk: [i32; 2],
    height: HeightContext,
    shape: &CanyonShapeConfig,
    emit: &mut impl FnMut(CarverVolume),
) {
    let mut random = LegacyRandom::new(seed);
    let width_factors = Arc::<[f32]>::from(init_width_factors(
        height.depth,
        shape.width_smoothness,
        &mut random,
    ));
    let skip = CarverSkip::Canyon {
        minimum_y: height.minimum_y,
        width_factors,
    };
    let mut pitch_drift = 0.0_f32;
    let mut yaw_drift = 0.0_f32;
    for step in 0..path.distance {
        let phase = step as f32 * FLOAT_PI / path.distance as f32;
        let base_radius = 1.5 + f64::from(minecraft_sin(phase) * path.thickness);
        let horizontal_radius = base_radius
            * f64::from(
                shape
                    .horizontal_radius_factor
                    .sample(&mut random)
                    .expect("validated canyon provider"),
            );
        let center_factor = 1.0 - (0.5 - step as f32 / path.distance as f32).abs() * 2.0;
        let vertical_factor = shape.vertical_radius_default_factor
            + shape.vertical_radius_center_factor * center_factor;
        let preliminary_vertical_radius = base_radius * path.y_scale;
        let random_vertical_factor = 0.75 + random.next_f32() * 0.25;
        let vertical_radius = f64::from(vertical_factor)
            * preliminary_vertical_radius
            * f64::from(random_vertical_factor);
        let cos_pitch = minecraft_cos(path.pitch);
        path.x += f64::from(minecraft_cos(path.yaw) * cos_pitch);
        path.y += f64::from(minecraft_sin(path.pitch));
        path.z += f64::from(minecraft_sin(path.yaw) * cos_pitch);
        path.pitch *= 0.7;
        path.pitch += pitch_drift * 0.05;
        path.yaw += yaw_drift * 0.05;
        pitch_drift *= 0.8;
        yaw_drift *= 0.5;
        pitch_drift += (random.next_f32() - random.next_f32()) * random.next_f32() * 2.0;
        yaw_drift += (random.next_f32() - random.next_f32()) * random.next_f32() * 4.0;
        if bounded(&mut random, 4) == 0 {
            continue;
        }
        if !can_reach(
            target_chunk,
            path.x,
            path.z,
            step,
            path.distance,
            path.thickness,
        ) {
            return;
        }
        emit(CarverVolume {
            ellipsoid: CarverEllipsoid {
                center_x: path.x,
                center_y: path.y,
                center_z: path.z,
                horizontal_radius,
                vertical_radius,
            },
            skip: skip.clone(),
        });
    }
}

fn init_width_factors(depth: i32, smoothness: u32, random: &mut impl GenerationRandom) -> Vec<f32> {
    let mut result = Vec::with_capacity(depth.max(0) as usize);
    let mut width = 1.0_f32;
    for index in 0..depth {
        if index == 0 || bounded_u32(random, smoothness) == 0 {
            width = 1.0 + random.next_f32() * random.next_f32();
        }
        result.push(width * width);
    }
    result
}

fn cave_thickness(family: CaveFamily, random: &mut impl GenerationRandom) -> f32 {
    let base = random.next_f32() * 2.0 + random.next_f32();
    match family {
        CaveFamily::Nether => base * 2.0,
        CaveFamily::Ordinary if bounded(random, 10) == 0 => {
            base * (random.next_f32() * random.next_f32() * 3.0 + 1.0)
        }
        CaveFamily::Ordinary => base,
    }
}

fn can_reach(
    target_chunk: [i32; 2],
    x: f64,
    z: f64,
    step: i32,
    distance: i32,
    thickness: f32,
) -> bool {
    let middle_x = f64::from(target_chunk[0].wrapping_mul(16).wrapping_add(8));
    let middle_z = f64::from(target_chunk[1].wrapping_mul(16).wrapping_add(8));
    let x_distance = x - middle_x;
    let z_distance = z - middle_z;
    let remaining = f64::from(distance - step);
    let radius = f64::from(thickness + 18.0);
    x_distance * x_distance + z_distance * z_distance - remaining * remaining <= radius * radius
}

fn validate_probability(probability: f32) -> Result<(), CarverPathError> {
    if probability.is_finite() && (0.0..=1.0).contains(&probability) {
        Ok(())
    } else {
        Err(CarverPathError::InvalidProbability)
    }
}

fn validate_canyon(config: &CanyonPathConfig) -> Result<(), CarverPathError> {
    validate_probability(config.probability)?;
    if config.shape.width_smoothness == 0
        || !config.shape.vertical_radius_default_factor.is_finite()
        || !config.shape.vertical_radius_center_factor.is_finite()
    {
        return Err(CarverPathError::InvalidCanyonShape);
    }
    Ok(())
}

fn minecraft_sin(value: f32) -> f32 {
    let index = ((value * 10_430.378) as i32 & 65_535) as u32;
    (f64::from(index) * std::f64::consts::TAU / 65_536.0).sin() as f32
}

fn minecraft_cos(value: f32) -> f32 {
    minecraft_sin(value + FLOAT_HALF_PI)
}

fn bounded(random: &mut impl GenerationRandom, bound: i32) -> i32 {
    bounded_u32(random, bound as u32) as i32
}

fn bounded_u32(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("carver random bound is nonzero"))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CarverPathError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("carver probability must be finite and within zero through one")]
    InvalidProbability,
    #[error("carver float provider has invalid bounds")]
    InvalidFloatProvider,
    #[error("canyon shape has invalid factors or zero width smoothness")]
    InvalidCanyonShape,
    #[error("carver source position overflow")]
    PositionOverflow,
}
