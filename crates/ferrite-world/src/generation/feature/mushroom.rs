//! Huge mushroom configured features with source-ordered validation, caps, and trunks.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugeMushroomKind {
    Brown,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HugeMushroomConfig {
    pub foliage_radius: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MushroomCapProperties {
    pub west: bool,
    pub east: bool,
    pub north: bool,
    pub south: bool,
    pub up: Option<bool>,
}

pub trait HugeMushroomWorld<R: GenerationRandom> {
    fn minimum_y(&self) -> i32;

    fn maximum_y_exclusive(&self) -> i32;

    fn can_place_mushroom_on(&mut self, position: BlockPos, random: &mut R) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_leaves(&self, state: BlockStateId) -> bool;

    fn is_replaceable_by_mushrooms(&self, state: BlockStateId) -> bool;

    fn provide_cap_state(&mut self, provider_position: BlockPos, random: &mut R) -> BlockStateId;

    fn provide_stem_state(&mut self, provider_position: BlockPos, random: &mut R) -> BlockStateId;

    fn configure_brown_cap(
        &mut self,
        state: BlockStateId,
        properties: MushroomCapProperties,
    ) -> BlockStateId;

    fn configure_red_cap(
        &mut self,
        state: BlockStateId,
        properties: MushroomCapProperties,
    ) -> BlockStateId;

    fn offer_mushroom_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32)
    -> bool;
}

pub fn place_huge_mushroom<R, W>(
    world: &mut W,
    origin: BlockPos,
    kind: HugeMushroomKind,
    config: HugeMushroomConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, HugeMushroomError>
where
    R: GenerationRandom,
    W: HugeMushroomWorld<R>,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let mut height =
        4 + random.next_u32(NonZeroU32::new(3).expect("mushroom height bound is nonzero")) as i32;
    if random.next_u32(NonZeroU32::new(12).expect("mushroom doubling bound is nonzero")) == 0 {
        height = height.wrapping_mul(2);
    }
    if !valid_mushroom_position(world, origin, kind, config, height, random)? {
        return Ok(false);
    }
    match kind {
        HugeMushroomKind::Brown => {
            place_brown_cap(world, origin, config.foliage_radius, height, random)?;
        }
        HugeMushroomKind::Red => {
            place_red_cap(world, origin, config.foliage_radius, height, random)?;
        }
    }
    place_trunk(world, origin, height, random)?;
    Ok(true)
}

fn valid_mushroom_position<R, W>(
    world: &mut W,
    origin: BlockPos,
    kind: HugeMushroomKind,
    config: HugeMushroomConfig,
    height: i32,
    random: &mut R,
) -> Result<bool, HugeMushroomError>
where
    R: GenerationRandom,
    W: HugeMushroomWorld<R>,
{
    let minimum_origin_y = world
        .minimum_y()
        .checked_add(1)
        .ok_or(HugeMushroomError::PositionOverflow)?;
    let top_exclusive = origin
        .y
        .checked_add(height)
        .and_then(|value| value.checked_add(1))
        .ok_or(HugeMushroomError::PositionOverflow)?;
    if origin.y < minimum_origin_y || top_exclusive > world.maximum_y_exclusive() {
        return Ok(false);
    }
    let below = offset(origin, 0, -1, 0)?;
    if !world.can_place_mushroom_on(below, random) {
        return Ok(false);
    }
    for layer in 0..=height {
        let radius = match kind {
            HugeMushroomKind::Brown if layer >= 4 => config.foliage_radius,
            _ => 0,
        };
        let start = radius.wrapping_neg();
        if start > radius {
            continue;
        }
        for x_offset in start..=radius {
            for z_offset in start..=radius {
                let position = offset(origin, x_offset, layer, z_offset)?;
                let state = world.block_state(position);
                if !world.is_air(state) && !world.is_leaves(state) {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

fn place_brown_cap<R, W>(
    world: &mut W,
    origin: BlockPos,
    radius: i32,
    height: i32,
    random: &mut R,
) -> Result<(), HugeMushroomError>
where
    R: GenerationRandom,
    W: HugeMushroomWorld<R>,
{
    let start = radius.wrapping_neg();
    if start > radius {
        return Ok(());
    }
    for x_offset in start..=radius {
        for z_offset in start..=radius {
            let x_edge = x_offset == start || x_offset == radius;
            let z_edge = z_offset == start || z_offset == radius;
            if x_edge && z_edge {
                continue;
            }
            let position = offset(origin, x_offset, height, z_offset)?;
            let state = world.provide_cap_state(origin, random);
            let state = world.configure_brown_cap(
                state,
                MushroomCapProperties {
                    west: x_offset == start || x_offset == 1_i32.wrapping_sub(radius) && z_edge,
                    east: x_offset == radius || x_offset == radius.wrapping_sub(1) && z_edge,
                    north: z_offset == start || z_offset == 1_i32.wrapping_sub(radius) && x_edge,
                    south: z_offset == radius || z_offset == radius.wrapping_sub(1) && x_edge,
                    up: None,
                },
            );
            place_provided_state(world, position, state);
        }
    }
    Ok(())
}

fn place_red_cap<R, W>(
    world: &mut W,
    origin: BlockPos,
    foliage_radius: i32,
    height: i32,
    random: &mut R,
) -> Result<(), HugeMushroomError>
where
    R: GenerationRandom,
    W: HugeMushroomWorld<R>,
{
    let first_layer = height
        .checked_sub(3)
        .ok_or(HugeMushroomError::PositionOverflow)?;
    let inner = foliage_radius.wrapping_sub(2);
    for layer in first_layer..=height {
        let top = layer == height;
        let radius = if top {
            foliage_radius.wrapping_sub(1)
        } else {
            foliage_radius
        };
        let start = radius.wrapping_neg();
        if start > radius {
            continue;
        }
        for x_offset in start..=radius {
            for z_offset in start..=radius {
                if !top {
                    let x_edge = x_offset == start || x_offset == radius;
                    let z_edge = z_offset == start || z_offset == radius;
                    if x_edge == z_edge {
                        continue;
                    }
                }
                let position = offset(origin, x_offset, layer, z_offset)?;
                let state = world.provide_cap_state(origin, random);
                let state = world.configure_red_cap(
                    state,
                    MushroomCapProperties {
                        west: x_offset < inner.wrapping_neg(),
                        east: x_offset > inner,
                        north: z_offset < inner.wrapping_neg(),
                        south: z_offset > inner,
                        up: Some(layer >= height - 1),
                    },
                );
                place_provided_state(world, position, state);
            }
        }
    }
    Ok(())
}

fn place_trunk<R, W>(
    world: &mut W,
    origin: BlockPos,
    height: i32,
    random: &mut R,
) -> Result<(), HugeMushroomError>
where
    R: GenerationRandom,
    W: HugeMushroomWorld<R>,
{
    for y_offset in 0..height {
        let position = offset(origin, 0, y_offset, 0)?;
        let state = world.provide_stem_state(origin, random);
        place_provided_state(world, position, state);
    }
    Ok(())
}

fn place_provided_state<R, W>(world: &mut W, position: BlockPos, state: BlockStateId)
where
    R: GenerationRandom,
    W: HugeMushroomWorld<R>,
{
    let current = world.block_state(position);
    if world.is_air(current) || world.is_replaceable_by_mushrooms(current) {
        let _ = world.offer_mushroom_block(position, state, 3);
    }
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, HugeMushroomError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(HugeMushroomError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(HugeMushroomError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(HugeMushroomError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum HugeMushroomError {
    #[error("huge-mushroom position arithmetic overflowed")]
    PositionOverflow,
}
