//! Iceberg generation, smoothing, submerged body, and optional cutout.

use std::f64::consts::{FRAC_PI_2, PI};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

pub trait IcebergWorld {
    fn sea_level(&self) -> i32;

    fn canonical_air(&self) -> BlockStateId;

    fn source_water(&self) -> BlockStateId;

    fn snow_block(&self) -> BlockStateId;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_snow_block(&self, state: BlockStateId) -> bool;

    fn is_ordinary_ice(&self, state: BlockStateId) -> bool;

    fn is_water_block(&self, state: BlockStateId) -> bool;

    fn is_fixed_iceberg_state(&self, state: BlockStateId) -> bool;

    fn is_snow_layer(&self, state: BlockStateId) -> bool;

    fn offer_iceberg_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_iceberg<R, W>(
    world: &mut W,
    origin: BlockPos,
    configured_state: BlockStateId,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, IcebergError>
where
    R: GenerationRandom,
    W: IcebergWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let center = BlockPos::new(origin.x, world.sea_level(), origin.z);
    let snow_on_top = random.next_f64() > 0.7;
    let angle = random.next_f64() * 2.0 * PI;
    let ellipse_a =
        11 - random.next_u32(NonZeroU32::new(5).expect("iceberg A bound is nonzero")) as i32;
    let ellipse_c =
        3 + random.next_u32(NonZeroU32::new(3).expect("iceberg C bound is nonzero")) as i32;
    let ellipse = random.next_f64() > 0.7;
    let mut height = if ellipse {
        6 + bounded_i32(random, 6)
    } else {
        3 + bounded_i32(random, 15)
    };
    if !ellipse && random.next_f64() > 0.9 {
        height = height
            .checked_add(7 + bounded_i32(random, 19))
            .ok_or(IcebergError::PositionOverflow)?;
    }
    let submerged_height = (height + bounded_i32(random, 11)).min(18);
    let width = (height + bounded_i32(random, 7) - bounded_i32(random, 5)).min(11);
    let traversal_radius = if ellipse { ellipse_a } else { 11 };

    for x in -traversal_radius..traversal_radius {
        for z in -traversal_radius..traversal_radius {
            for y in 0..height {
                let radius = if ellipse {
                    radius_ellipse(y, height, width)
                } else {
                    radius_round(random, y, height, width)
                };
                if !ellipse && x >= radius {
                    continue;
                }
                generate_cell(
                    world,
                    random,
                    center,
                    CellInputs {
                        logical_height: height,
                        x,
                        y,
                        z,
                        radius,
                        ellipse_a: traversal_radius,
                        ellipse,
                        ellipse_c,
                        angle,
                        snow_on_top,
                        configured_state,
                    },
                )?;
            }
        }
    }
    smooth(world, center, width, height, ellipse, ellipse_a)?;
    for x in -traversal_radius..traversal_radius {
        for z in -traversal_radius..traversal_radius {
            for y in (-submerged_height + 1..=-1).rev() {
                let new_a = if ellipse {
                    ((traversal_radius as f32)
                        * (1.0 - (y as f32).powi(2) / (submerged_height as f32 * 8.0)))
                        .ceil() as i32
                } else {
                    traversal_radius
                };
                let radius = radius_steep(random, -y, submerged_height, width);
                if x >= radius {
                    continue;
                }
                generate_cell(
                    world,
                    random,
                    center,
                    CellInputs {
                        logical_height: submerged_height,
                        x,
                        y,
                        z,
                        radius,
                        ellipse_a: new_a,
                        ellipse,
                        ellipse_c,
                        angle,
                        snow_on_top,
                        configured_state,
                    },
                )?;
            }
        }
    }
    let cutout = random.next_f64() > if ellipse { 0.1 } else { 0.7 };
    if cutout {
        let inputs = CutoutInputs {
            width,
            height,
            center,
            ellipse,
            ellipse_a,
            angle,
            ellipse_c,
        };
        generate_cutout(world, random, inputs)?;
    }
    Ok(true)
}

#[derive(Debug, Clone, Copy)]
struct CellInputs {
    logical_height: i32,
    x: i32,
    y: i32,
    z: i32,
    radius: i32,
    ellipse_a: i32,
    ellipse: bool,
    ellipse_c: i32,
    angle: f64,
    snow_on_top: bool,
    configured_state: BlockStateId,
}

fn generate_cell(
    world: &mut impl IcebergWorld,
    random: &mut impl GenerationRandom,
    center: BlockPos,
    inputs: CellInputs,
) -> Result<(), IcebergError> {
    let signed_distance = if inputs.ellipse {
        signed_distance_ellipse(
            inputs.x,
            inputs.z,
            BlockPos::new(0, 0, 0),
            inputs.ellipse_a,
            ellipse_c(inputs.y, inputs.logical_height, inputs.ellipse_c),
            inputs.angle,
        )
    } else {
        signed_distance_circle(inputs.x, inputs.z, inputs.radius, random)
    };
    if signed_distance >= 0.0 {
        return Ok(());
    }
    let boundary = if inputs.ellipse {
        -0.5
    } else {
        f64::from(-6 - bounded_i32(random, 3))
    };
    if signed_distance > boundary && random.next_f64() > 0.9 {
        return Ok(());
    }
    let position = offset_xyz(center, inputs.x, inputs.y, inputs.z)?;
    let cell = SetCellInputs {
        position,
        height_difference: inputs.logical_height - inputs.y,
        height: inputs.logical_height,
        ellipse: inputs.ellipse,
        snow_on_top: inputs.snow_on_top,
        configured_state: inputs.configured_state,
    };
    set_iceberg_cell(world, random, cell);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct SetCellInputs {
    position: BlockPos,
    height_difference: i32,
    height: i32,
    ellipse: bool,
    snow_on_top: bool,
    configured_state: BlockStateId,
}

fn set_iceberg_cell(
    world: &mut impl IcebergWorld,
    random: &mut impl GenerationRandom,
    inputs: SetCellInputs,
) {
    let current = world.block_state(inputs.position);
    if !world.is_air(current)
        && !world.is_snow_block(current)
        && !world.is_ordinary_ice(current)
        && !world.is_water_block(current)
    {
        return;
    }
    let snow_randomness = !inputs.ellipse || random.next_f64() > 0.05;
    let divisor = if inputs.ellipse { 3 } else { 2 };
    let snow = if inputs.snow_on_top && !world.is_water_block(current) {
        let bound = (inputs.height / divisor).max(1);
        f64::from(inputs.height_difference)
            <= f64::from(bounded_i32(random, bound)) + f64::from(inputs.height) * 0.6
            && snow_randomness
    } else {
        false
    };
    let state = if snow {
        world.snow_block()
    } else {
        inputs.configured_state
    };
    let _ = world.offer_iceberg_block(inputs.position, state, 3);
}

fn smooth(
    world: &mut impl IcebergWorld,
    center: BlockPos,
    width: i32,
    height: i32,
    ellipse: bool,
    ellipse_a: i32,
) -> Result<(), IcebergError> {
    let radius = if ellipse { ellipse_a } else { width / 2 };
    for x in -radius..=radius {
        for z in -radius..=radius {
            for y in 0..=height {
                let position = offset_xyz(center, x, y, z)?;
                let state = world.block_state(position);
                if !world.is_fixed_iceberg_state(state) && !world.is_snow_layer(state) {
                    continue;
                }
                let below = offset_xyz(position, 0, -1, 0)?;
                let below_state = world.block_state(below);
                if world.is_air(below_state) {
                    offer_air(world, position);
                    let above = offset_xyz(position, 0, 1, 0)?;
                    offer_air(world, above);
                    continue;
                }
                if !world.is_fixed_iceberg_state(state) {
                    continue;
                }
                let mut outside = 0_u8;
                for direction in [
                    Direction::West,
                    Direction::East,
                    Direction::North,
                    Direction::South,
                ] {
                    let neighbor = offset(position, direction)?;
                    let neighbor_state = world.block_state(neighbor);
                    outside += u8::from(!world.is_fixed_iceberg_state(neighbor_state));
                }
                if outside >= 3 {
                    offer_air(world, position);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct CutoutInputs {
    width: i32,
    height: i32,
    center: BlockPos,
    ellipse: bool,
    ellipse_a: i32,
    angle: f64,
    ellipse_c: i32,
}

fn generate_cutout(
    world: &mut impl IcebergWorld,
    random: &mut impl GenerationRandom,
    inputs: CutoutInputs,
) -> Result<(), IcebergError> {
    let sign_x = if random.next_bool() { -1 } else { 1 };
    let sign_z = if random.next_bool() { -1 } else { 1 };
    let mut x_offset = bounded_i32(random, (inputs.width / 2 - 2).max(1));
    if random.next_bool() {
        x_offset = inputs.width / 2 + 1
            - bounded_i32(random, (inputs.width - inputs.width / 2 - 1).max(1));
    }
    let mut z_offset = bounded_i32(random, (inputs.width / 2 - 2).max(1));
    if random.next_bool() {
        z_offset = inputs.width / 2 + 1
            - bounded_i32(random, (inputs.width - inputs.width / 2 - 1).max(1));
    }
    if inputs.ellipse {
        let shared = bounded_i32(random, (inputs.ellipse_a - 5).max(1));
        x_offset = shared;
        z_offset = shared;
    }
    let local_center = BlockPos::new(sign_x * x_offset, 0, sign_z * z_offset);
    let cutout_angle = if inputs.ellipse {
        inputs.angle + FRAC_PI_2
    } else {
        random.next_f64() * 2.0 * PI
    };
    for y in 0..inputs.height - 3 {
        let radius = radius_round(random, y, inputs.height, inputs.width);
        let carve_inputs = CarveInputs {
            radius,
            y,
            center: inputs.center,
            underwater: false,
            angle: cutout_angle,
            local_center,
            ellipse_a: inputs.ellipse_a,
            ellipse_c: inputs.ellipse_c,
        };
        carve(world, carve_inputs)?;
    }
    let mut y = -1;
    loop {
        let continuation_draw = bounded_i32(random, 5);
        if y <= -inputs.height + continuation_draw {
            break;
        }
        let radius = radius_steep(random, -y, inputs.height, inputs.width);
        let carve_inputs = CarveInputs {
            radius,
            y,
            center: inputs.center,
            underwater: true,
            angle: cutout_angle,
            local_center,
            ellipse_a: inputs.ellipse_a,
            ellipse_c: inputs.ellipse_c,
        };
        carve(world, carve_inputs)?;
        y -= 1;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct CarveInputs {
    radius: i32,
    y: i32,
    center: BlockPos,
    underwater: bool,
    angle: f64,
    local_center: BlockPos,
    ellipse_a: i32,
    ellipse_c: i32,
}

fn carve(world: &mut impl IcebergWorld, inputs: CarveInputs) -> Result<(), IcebergError> {
    let a = inputs.radius + 1 + inputs.ellipse_a / 3;
    let c = (inputs.radius - 3).min(3) + inputs.ellipse_c / 2 - 1;
    for x in -a..a {
        for z in -a..a {
            if signed_distance_ellipse(x, z, inputs.local_center, a, c, inputs.angle) >= 0.0 {
                continue;
            }
            let position = offset_xyz(inputs.center, x, inputs.y, z)?;
            let state = world.block_state(position);
            if !world.is_fixed_iceberg_state(state) && !world.is_snow_block(state) {
                continue;
            }
            if inputs.underwater {
                let water = world.source_water();
                let _ = world.offer_iceberg_block(position, water, 3);
            } else {
                offer_air(world, position);
                let above = offset_xyz(position, 0, 1, 0)?;
                let above_state = world.block_state(above);
                if world.is_snow_layer(above_state) {
                    offer_air(world, above);
                }
            }
        }
    }
    Ok(())
}

fn radius_round(random: &mut impl GenerationRandom, y: i32, height: i32, width: i32) -> i32 {
    let factor = 3.5_f32 - random.next_f32();
    let mut scale = (1.0 - (y as f32).powi(2) / (height as f32 * factor)) * width as f32;
    if height > 15 + bounded_i32(random, 5) {
        let substituted_y = if y < 3 + bounded_i32(random, 6) {
            y / 2
        } else {
            y
        };
        scale = (1.0 - substituted_y as f32 / (height as f32 * factor * 0.4)) * width as f32;
    }
    (scale / 2.0).ceil() as i32
}

fn radius_ellipse(y: i32, height: i32, width: i32) -> i32 {
    let scale = (1.0 - (y as f32).powi(2) / height as f32) * width as f32;
    (scale / 2.0).ceil() as i32
}

fn radius_steep(random: &mut impl GenerationRandom, y: i32, height: i32, width: i32) -> i32 {
    let factor = 1.0 + random.next_f32() / 2.0;
    let scale = (1.0 - y as f32 / (height as f32 * factor)) * width as f32;
    (scale / 2.0).ceil() as i32
}

fn signed_distance_circle(x: i32, z: i32, radius: i32, random: &mut impl GenerationRandom) -> f64 {
    let offset = 10.0_f32 * random.next_f32().clamp(0.2, 0.8) / radius as f32;
    f64::from(offset) + f64::from(x).powi(2) + f64::from(z).powi(2) - f64::from(radius).powi(2)
}

fn signed_distance_ellipse(x: i32, z: i32, center: BlockPos, a: i32, c: i32, angle: f64) -> f64 {
    let x = f64::from(x - center.x);
    let z = f64::from(z - center.z);
    ((x * angle.cos() - z * angle.sin()) / f64::from(a)).powi(2)
        + ((x * angle.sin() + z * angle.cos()) / f64::from(c)).powi(2)
        - 1.0
}

fn ellipse_c(y: i32, height: i32, base: i32) -> i32 {
    if y > 0 && height - y <= 3 {
        base - (4 - (height - y))
    } else {
        base
    }
}

fn offer_air(world: &mut impl IcebergWorld, position: BlockPos) {
    let _ = world.offer_iceberg_block(position, world.canonical_air(), 3);
}

fn bounded_i32(random: &mut impl GenerationRandom, bound: i32) -> i32 {
    let bound = u32::try_from(bound)
        .ok()
        .and_then(NonZeroU32::new)
        .expect("iceberg algorithm bounds are positive");
    random.next_u32(bound) as i32
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, IcebergError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, IcebergError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(IcebergError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(IcebergError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(IcebergError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IcebergError {
    #[error("iceberg position overflow")]
    PositionOverflow,
}
