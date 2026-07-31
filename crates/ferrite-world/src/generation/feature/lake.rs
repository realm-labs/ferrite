//! Lake feature with an atomic boundary audit and ordered cavity/coating passes.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

const WIDTH: usize = 16;
const HEIGHT: usize = 8;
const MASK_SIZE: usize = WIDTH * WIDTH * HEIGHT;

pub trait LakeWorld {
    fn minimum_y(&self) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_liquid(&self, state: BlockStateId) -> bool;

    fn is_solid(&self, state: BlockStateId) -> bool;

    fn cave_air(&self) -> BlockStateId;

    fn default_ice(&self) -> BlockStateId;

    fn sample_fluid<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn sample_barrier<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn can_place_feature(&mut self, position: BlockPos) -> bool;

    fn can_replace_with_air_or_fluid(&mut self, position: BlockPos) -> bool;

    fn can_replace_with_barrier(&mut self, position: BlockPos) -> bool;

    fn fluid_is_water_tagged(&self, state: BlockStateId) -> bool;

    fn should_freeze_without_edge(&mut self, position: BlockPos) -> bool;

    fn offer_lake_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn schedule_zero_delay_tick(&mut self, position: BlockPos, block: BlockStateId);

    fn mark_for_postprocessing(&mut self, position: BlockPos);
}

pub fn place_lake<R, W>(
    world: &mut W,
    origin: BlockPos,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, LakeError>
where
    R: GenerationRandom,
    W: LakeWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let minimum_origin_y = world
        .minimum_y()
        .checked_add(4)
        .ok_or(LakeError::PositionOverflow)?;
    if origin.y <= minimum_origin_y {
        return Ok(false);
    }
    let base = offset_xyz(origin, -8, -4, -8)?;
    let mask = create_mask(random);
    let fluid = world.sample_fluid(base, random);
    if !audit_boundary(world, base, &mask, fluid)? {
        return Ok(false);
    }

    carve_cavity(world, base, &mask, fluid)?;
    let barrier = world.sample_barrier(base, random);
    if !world.is_air(barrier) {
        coat_boundary(world, base, &mask, barrier, random)?;
    }
    if world.fluid_is_water_tagged(fluid) {
        freeze_surface(world, base)?;
    }
    Ok(true)
}

fn create_mask(random: &mut impl GenerationRandom) -> [bool; MASK_SIZE] {
    let mut mask = [false; MASK_SIZE];
    let ellipsoid_count =
        random.next_u32(NonZeroU32::new(4).expect("lake ellipsoid bound is nonzero")) + 4;
    for _ in 0..ellipsoid_count {
        let diameter_x = 3.0 + random.next_f64() * 6.0;
        let diameter_y = 2.0 + random.next_f64() * 4.0;
        let diameter_z = 3.0 + random.next_f64() * 6.0;
        let center_x = 1.0 + diameter_x / 2.0 + random.next_f64() * (14.0 - diameter_x);
        let center_y = 2.0 + diameter_y / 2.0 + random.next_f64() * (4.0 - diameter_y);
        let center_z = 1.0 + diameter_z / 2.0 + random.next_f64() * (14.0 - diameter_z);
        for x in 1..15 {
            let normalized_x = (x as f64 - center_x) / (diameter_x / 2.0);
            for z in 1..15 {
                let normalized_z = (z as f64 - center_z) / (diameter_z / 2.0);
                for y in 1..7 {
                    let normalized_y = (y as f64 - center_y) / (diameter_y / 2.0);
                    if normalized_x * normalized_x
                        + normalized_y * normalized_y
                        + normalized_z * normalized_z
                        < 1.0
                    {
                        mask[index(x, y, z)] = true;
                    }
                }
            }
        }
    }
    mask
}

fn audit_boundary<W: LakeWorld>(
    world: &mut W,
    base: BlockPos,
    mask: &[bool; MASK_SIZE],
    fluid: BlockStateId,
) -> Result<bool, LakeError> {
    for x in 0..WIDTH {
        for z in 0..WIDTH {
            for y in 0..HEIGHT {
                if mask[index(x, y, z)] || !is_boundary(mask, x, y, z) {
                    continue;
                }
                let position = local_position(base, x, y, z)?;
                let state = world.block_state(position);
                if y >= 4 {
                    if world.is_liquid(state) {
                        return Ok(false);
                    }
                } else if !world.is_solid(state) && state != fluid {
                    return Ok(false);
                }
                if !world.can_place_feature(position) {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

fn carve_cavity<W: LakeWorld>(
    world: &mut W,
    base: BlockPos,
    mask: &[bool; MASK_SIZE],
    fluid: BlockStateId,
) -> Result<(), LakeError> {
    for x in 0..WIDTH {
        for z in 0..WIDTH {
            for y in 0..HEIGHT {
                if !mask[index(x, y, z)] {
                    continue;
                }
                let position = local_position(base, x, y, z)?;
                if !world.can_replace_with_air_or_fluid(position) {
                    continue;
                }
                let state = if y >= 4 { world.cave_air() } else { fluid };
                let _ = world.offer_lake_block(position, state, 2);
                if y >= 4 {
                    world.schedule_zero_delay_tick(position, state);
                    mark_above(world, position)?;
                }
            }
        }
    }
    Ok(())
}

fn coat_boundary<R, W>(
    world: &mut W,
    base: BlockPos,
    mask: &[bool; MASK_SIZE],
    barrier: BlockStateId,
    random: &mut R,
) -> Result<(), LakeError>
where
    R: GenerationRandom,
    W: LakeWorld,
{
    for x in 0..WIDTH {
        for z in 0..WIDTH {
            for y in 0..HEIGHT {
                if !is_boundary(mask, x, y, z) {
                    continue;
                }
                if y >= 4
                    && random
                        .next_u32(NonZeroU32::new(2).expect("lake coating chance bound is nonzero"))
                        == 0
                {
                    continue;
                }
                let position = local_position(base, x, y, z)?;
                let state = world.block_state(position);
                if !world.is_solid(state) || !world.can_replace_with_barrier(position) {
                    continue;
                }
                let _ = world.offer_lake_block(position, barrier, 2);
                mark_above(world, position)?;
            }
        }
    }
    Ok(())
}

fn freeze_surface<W: LakeWorld>(world: &mut W, base: BlockPos) -> Result<(), LakeError> {
    for x in 0..WIDTH {
        for z in 0..WIDTH {
            let position = local_position(base, x, 4, z)?;
            if world.should_freeze_without_edge(position)
                && world.can_replace_with_air_or_fluid(position)
            {
                let _ = world.offer_lake_block(position, world.default_ice(), 2);
            }
        }
    }
    Ok(())
}

fn mark_above<W: LakeWorld>(world: &mut W, position: BlockPos) -> Result<(), LakeError> {
    let mut cursor = position;
    for _ in 0..2 {
        cursor = offset_xyz(cursor, 0, 1, 0)?;
        let state = world.block_state(cursor);
        if world.is_air(state) {
            break;
        }
        world.mark_for_postprocessing(cursor);
    }
    Ok(())
}

fn is_boundary(mask: &[bool; MASK_SIZE], x: usize, y: usize, z: usize) -> bool {
    !mask[index(x, y, z)]
        && ((x + 1 < WIDTH && mask[index(x + 1, y, z)])
            || (x > 0 && mask[index(x - 1, y, z)])
            || (z + 1 < WIDTH && mask[index(x, y, z + 1)])
            || (z > 0 && mask[index(x, y, z - 1)])
            || (y + 1 < HEIGHT && mask[index(x, y + 1, z)])
            || (y > 0 && mask[index(x, y - 1, z)]))
}

const fn index(x: usize, y: usize, z: usize) -> usize {
    ((x * WIDTH) + z) * HEIGHT + y
}

fn local_position(base: BlockPos, x: usize, y: usize, z: usize) -> Result<BlockPos, LakeError> {
    offset_xyz(
        base,
        i32::try_from(x).map_err(|_| LakeError::PositionOverflow)?,
        i32::try_from(y).map_err(|_| LakeError::PositionOverflow)?,
        i32::try_from(z).map_err(|_| LakeError::PositionOverflow)?,
    )
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, LakeError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(LakeError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(LakeError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(LakeError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LakeError {
    #[error("lake feature position overflow")]
    PositionOverflow,
}
