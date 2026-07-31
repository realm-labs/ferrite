//! Huge-fungus stem, hat, destruction, decor, and hanging-vine algorithms.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HugeFungusConfig {
    pub valid_base: BlockStateId,
    pub stem: BlockStateId,
    pub hat: BlockStateId,
    pub decor: BlockStateId,
    pub planted: bool,
    pub crimson_vines: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FungusVinePlacement {
    Body,
    Head { age: u8 },
}

pub trait HugeFungusWorld {
    fn generation_depth(&self) -> i32;

    fn canonical_air(&self) -> BlockStateId;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn same_block_type(&self, left: BlockStateId, right: BlockStateId) -> bool;

    fn can_be_replaced(&self, state: BlockStateId) -> bool;

    fn matches_stem_replacement(&mut self, state: BlockStateId, position: BlockPos) -> bool;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn offer_fungus_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn destroy_fungus_target(&mut self, position: BlockPos, drop_items: bool) -> bool;

    fn offer_fungus_vine(
        &mut self,
        position: BlockPos,
        placement: FungusVinePlacement,
        flags: u32,
    ) -> bool;
}

pub trait FungusBonemealWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_required_fungus_base(&self, state: BlockStateId) -> bool;

    fn is_inside_build_height(&self, position: BlockPos) -> bool;

    fn resolve_planted_fungus(&mut self) -> bool;

    fn place_planted_fungus<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> bool;
}

pub fn is_valid_fungus_bonemeal_target<W: FungusBonemealWorld>(
    world: &mut W,
    position: BlockPos,
) -> Result<bool, HugeFungusError> {
    let below = offset_xyz(position, 0, -1, 0)?;
    let base = world.block_state(below);
    let above = offset_xyz(position, 0, 1, 0)?;
    Ok(world.is_required_fungus_base(base) && world.is_inside_build_height(above))
}

pub fn is_fungus_bonemeal_success(random: &mut impl GenerationRandom) -> bool {
    random.next_f32() < 0.4
}

pub fn perform_fungus_bonemeal<R, W>(world: &mut W, position: BlockPos, random: &mut R)
where
    R: GenerationRandom,
    W: FungusBonemealWorld,
{
    if world.resolve_planted_fungus() {
        let _ = world.place_planted_fungus(position, random);
    }
}

pub fn place_huge_fungus<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: HugeFungusConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, HugeFungusError>
where
    R: GenerationRandom,
    W: HugeFungusWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let below = offset_xyz(origin, 0, -1, 0)?;
    let base = world.block_state(below);
    if !world.same_block_type(base, config.valid_base) {
        return Ok(false);
    }
    let mut height =
        4 + random.next_u32(NonZeroU32::new(10).expect("fungus height bound is nonzero")) as i32;
    if random.next_u32(NonZeroU32::new(12).expect("fungus double bound is nonzero")) == 0 {
        height = height
            .checked_mul(2)
            .ok_or(HugeFungusError::PositionOverflow)?;
    }
    if !config.planted {
        let top = origin
            .y
            .checked_add(height)
            .and_then(|value| value.checked_add(1))
            .ok_or(HugeFungusError::PositionOverflow)?;
        if top >= world.generation_depth() {
            return Ok(false);
        }
    }
    let wide = !config.planted && random.next_f32() < 0.06;
    let _ = world.offer_fungus_block(origin, world.canonical_air(), 260);
    place_stem(world, origin, height, wide, config, random)?;
    place_hat(world, origin, height, wide, config, random)?;
    Ok(true)
}

fn place_stem<R, W>(
    world: &mut W,
    origin: BlockPos,
    height: i32,
    wide: bool,
    config: HugeFungusConfig,
    random: &mut R,
) -> Result<(), HugeFungusError>
where
    R: GenerationRandom,
    W: HugeFungusWorld,
{
    let radius = i32::from(wide);
    for x in -radius..=radius {
        for z in -radius..=radius {
            for y in 0..height {
                let target = offset_xyz(origin, x, y, z)?;
                let current = world.block_state(target);
                if !world.can_be_replaced(current)
                    && !world.matches_stem_replacement(current, target)
                {
                    continue;
                }
                let corner = wide && x.abs() == 1 && z.abs() == 1;
                if corner && random.next_f32() >= 0.1 {
                    continue;
                }
                offer_planted_block(world, target, config.stem, config.planted)?;
            }
        }
    }
    Ok(())
}

fn place_hat<R, W>(
    world: &mut W,
    origin: BlockPos,
    height: i32,
    wide: bool,
    config: HugeFungusConfig,
    random: &mut R,
) -> Result<(), HugeFungusError>
where
    R: GenerationRandom,
    W: HugeFungusWorld,
{
    let depth_bound = u32::try_from(1 + height / 3)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(HugeFungusError::PositionOverflow)?;
    let depth = (5 + random.next_u32(depth_bound) as i32).min(height);
    let start_y = height - depth;
    for layer_y in start_y..=height {
        let radius_draw =
            random.next_u32(NonZeroU32::new(3).expect("fungus hat radius bound is nonzero")) as i32;
        let mut radius = if layer_y < height - radius_draw { 2 } else { 1 };
        if depth > 8 && layer_y < start_y + 4 {
            radius = 3;
        }
        if wide {
            radius += 1;
        }
        for x in -radius..=radius {
            for z in -radius..=radius {
                let target = offset_xyz(origin, x, layer_y, z)?;
                let current = world.block_state(target);
                if !world.can_be_replaced(current) {
                    continue;
                }
                if config.planted {
                    destroy_when_supported(world, target)?;
                }
                let edge_x = x.abs() == radius;
                let edge_z = z.abs() == radius;
                let corner = edge_x && edge_z;
                let interior = !edge_x && !edge_z && layer_y != height;
                let lower_region = layer_y < start_y + 3;
                if lower_region {
                    if !interior {
                        place_drop_hat(world, target, config, random)?;
                    }
                    continue;
                }
                let (decor_chance, hat_chance, vine_chance) = if interior {
                    (0.1, 0.2, 0.1)
                } else if corner {
                    (0.01, 0.7, 0.083)
                } else {
                    (0.0005, 0.98, 0.07)
                };
                place_probabilistic_hat(
                    world,
                    target,
                    config,
                    decor_chance,
                    hat_chance,
                    vine_chance,
                    random,
                )?;
            }
        }
    }
    Ok(())
}

fn place_drop_hat(
    world: &mut impl HugeFungusWorld,
    target: BlockPos,
    config: HugeFungusConfig,
    random: &mut impl GenerationRandom,
) -> Result<(), HugeFungusError> {
    let below = offset_xyz(target, 0, -1, 0)?;
    let below_state = world.block_state(below);
    if world.same_block_type(below_state, config.hat) {
        let _ = world.offer_fungus_block(target, config.hat, 3);
        return Ok(());
    }
    if random.next_f32() < 0.15 {
        let _ = world.offer_fungus_block(target, config.hat, 3);
        if config.crimson_vines
            && random
                .next_u32(NonZeroU32::new(11).expect("fungus drop-vine chance bound is nonzero"))
                == 0
        {
            try_place_vine(world, target, random)?;
        }
    }
    Ok(())
}

fn place_probabilistic_hat(
    world: &mut impl HugeFungusWorld,
    target: BlockPos,
    config: HugeFungusConfig,
    decor_chance: f32,
    hat_chance: f32,
    vine_chance: f32,
    random: &mut impl GenerationRandom,
) -> Result<(), HugeFungusError> {
    if random.next_f32() < decor_chance {
        let _ = world.offer_fungus_block(target, config.decor, 3);
        return Ok(());
    }
    if random.next_f32() >= hat_chance {
        return Ok(());
    }
    let _ = world.offer_fungus_block(target, config.hat, 3);
    let vine_draw = random.next_f32();
    if config.crimson_vines && vine_draw < vine_chance {
        try_place_vine(world, target, random)?;
    }
    Ok(())
}

fn try_place_vine(
    world: &mut impl HugeFungusWorld,
    hat_position: BlockPos,
    random: &mut impl GenerationRandom,
) -> Result<(), HugeFungusError> {
    let start = offset_xyz(hat_position, 0, -1, 0)?;
    let state = world.block_state(start);
    if !world.is_air(state) {
        return Ok(());
    }
    let mut length = 1 + random
        .next_u32(NonZeroU32::new(5).expect("fungus vine length bound is nonzero"))
        as i32;
    if random.next_u32(NonZeroU32::new(7).expect("fungus vine double bound is nonzero")) == 0 {
        length = length
            .checked_mul(2)
            .ok_or(HugeFungusError::PositionOverflow)?;
    }
    place_vine_column(world, start, length, random)
}

fn place_vine_column(
    world: &mut impl HugeFungusWorld,
    start: BlockPos,
    length: i32,
    random: &mut impl GenerationRandom,
) -> Result<(), HugeFungusError> {
    let mut cursor = start;
    for index in 0..=length {
        let current = world.block_state(cursor);
        if world.is_air(current) {
            let terminal = index == length;
            let blocked_below = if terminal {
                false
            } else {
                let below = offset_xyz(cursor, 0, -1, 0)?;
                let below_state = world.block_state(below);
                !world.is_air(below_state)
            };
            if terminal || blocked_below {
                let age = 23
                    + random.next_u32(NonZeroU32::new(3).expect("fungus vine age bound is nonzero"))
                        as u8;
                let _ = world.offer_fungus_vine(cursor, FungusVinePlacement::Head { age }, 2);
                return Ok(());
            }
            let _ = world.offer_fungus_vine(cursor, FungusVinePlacement::Body, 2);
        }
        if index != length {
            cursor = offset_xyz(cursor, 0, -1, 0)?;
        }
    }
    Ok(())
}

fn offer_planted_block(
    world: &mut impl HugeFungusWorld,
    target: BlockPos,
    state: BlockStateId,
    planted: bool,
) -> Result<(), HugeFungusError> {
    if planted {
        destroy_when_supported(world, target)?;
    }
    let _ = world.offer_fungus_block(target, state, 3);
    Ok(())
}

fn destroy_when_supported(
    world: &mut impl HugeFungusWorld,
    target: BlockPos,
) -> Result<(), HugeFungusError> {
    let below = offset_xyz(target, 0, -1, 0)?;
    let below_state = world.block_state(below);
    if !world.is_air(below_state) {
        let _ = world.destroy_fungus_target(target, true);
    }
    Ok(())
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, HugeFungusError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(HugeFungusError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(HugeFungusError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(HugeFungusError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HugeFungusError {
    #[error("huge-fungus position overflow")]
    PositionOverflow,
}
