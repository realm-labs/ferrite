//! Eroded-badlands and frozen-ocean surface extensions.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

pub trait SurfaceExtensionColumn {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_water(&self, state: BlockStateId) -> bool;

    fn is_same_block_as_default(&self, state: BlockStateId, default: BlockStateId) -> bool;

    fn set_extension_block(&mut self, position: BlockPos, state: BlockStateId);
}

pub trait SurfaceExtensionNoise {
    fn badlands_surface(&mut self, x: i32, z: i32) -> f64;

    fn badlands_pillar(&mut self, x: f64, z: f64) -> f64;

    fn badlands_roof(&mut self, x: f64, z: f64) -> f64;

    fn iceberg_surface(&mut self, x: i32, z: i32) -> f64;

    fn iceberg_pillar(&mut self, x: f64, z: f64) -> f64;

    fn iceberg_roof(&mut self, x: f64, z: f64) -> f64;
}

pub fn eroded_badlands_extension(
    column: &mut impl SurfaceExtensionColumn,
    noise: &mut impl SurfaceExtensionNoise,
    x: i32,
    z: i32,
    original_height: i32,
    minimum_y: i32,
    default_block: BlockStateId,
) {
    let buffer = (noise.badlands_surface(x, z) * 8.25)
        .abs()
        .min(noise.badlands_pillar(f64::from(x) * 0.2, f64::from(z) * 0.2) * 15.0);
    if buffer <= 0.0 {
        return;
    }
    let floor = (noise.badlands_roof(f64::from(x) * 0.75, f64::from(z) * 0.75) * 1.5).abs();
    let top = 64.0 + (buffer * buffer * 2.5).min((floor * 50.0).ceil() + 24.0);
    let start_y = minecraft_floor(top);
    if original_height > start_y {
        return;
    }
    for y in (minimum_y..=start_y).rev() {
        let state = column.block_state(BlockPos::new(x, y, z));
        if column.is_same_block_as_default(state, default_block) {
            break;
        }
        if column.is_water(state) {
            return;
        }
    }
    for y in (minimum_y..=start_y).rev() {
        let position = BlockPos::new(x, y, z);
        let state = column.block_state(position);
        if !column.is_air(state) {
            break;
        }
        column.set_extension_block(position, default_block);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrozenOceanStates {
    pub snow: BlockStateId,
    pub packed_ice: BlockStateId,
}

pub struct FrozenOceanInput {
    pub x: i32,
    pub z: i32,
    pub original_height: i32,
    pub minimum_surface_y: i32,
    pub sea_level: i32,
    pub melts_slightly: bool,
    pub states: FrozenOceanStates,
}

pub fn frozen_ocean_extension(
    column: &mut impl SurfaceExtensionColumn,
    noise: &mut impl SurfaceExtensionNoise,
    random: &mut impl GenerationRandom,
    input: FrozenOceanInput,
) {
    let iceberg = (noise.iceberg_surface(input.x, input.z) * 8.25)
        .abs()
        .min(noise.iceberg_pillar(f64::from(input.x) * 1.28, f64::from(input.z) * 1.28) * 15.0);
    if iceberg <= 1.8 {
        return;
    }
    let roof =
        (noise.iceberg_roof(f64::from(input.x) * 1.17, f64::from(input.z) * 1.17) * 1.5).abs();
    let mut top = (iceberg * iceberg * 1.2).min((roof * 40.0).ceil() + 14.0);
    if input.melts_slightly {
        top -= 2.0;
    }
    let bottom;
    if top > 2.0 {
        bottom = f64::from(input.sea_level) - top - 7.0;
        top += f64::from(input.sea_level);
    } else {
        top = 0.0;
        bottom = 0.0;
    }
    let maximum_snow_depth = 2 + bounded(random, 4);
    let minimum_snow_height = input
        .sea_level
        .wrapping_add(18)
        .wrapping_add(bounded(random, 10));
    let mut snow_depth = 0;
    let top_integer = top as i32;
    let bottom_integer = bottom as i32;
    let start_y = input.original_height.max(top_integer.wrapping_add(1));
    for y in (input.minimum_surface_y..=start_y).rev() {
        let position = BlockPos::new(input.x, y, input.z);
        let first_state = column.block_state(position);
        let air_admitted =
            column.is_air(first_state) && y < top_integer && random.next_f64() > 0.01;
        let admitted = if air_admitted {
            true
        } else {
            let second_state = column.block_state(position);
            column.is_water(second_state)
                && y > bottom_integer
                && y < input.sea_level
                && bottom != 0.0
                && random.next_f64() > 0.15
        };
        if !admitted {
            continue;
        }
        if snow_depth <= maximum_snow_depth && y > minimum_snow_height {
            column.set_extension_block(position, input.states.snow);
            snow_depth += 1;
        } else {
            column.set_extension_block(position, input.states.packed_ice);
        }
    }
}

fn minecraft_floor(value: f64) -> i32 {
    let truncated = value as i32;
    if value < f64::from(truncated) {
        truncated.wrapping_sub(1)
    } else {
        truncated
    }
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> i32 {
    random.next_u32(NonZeroU32::new(bound).expect("surface extension bound is nonzero")) as i32
}
