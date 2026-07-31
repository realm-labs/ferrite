//! Small structure-like configured features with explicit scan and mutation order.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

pub trait BonusChestRandom: GenerationRandom {
    fn next_i64(&mut self) -> i64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BonusChestStates {
    pub chest: BlockStateId,
    pub torch: BlockStateId,
}

pub trait BonusChestWorld {
    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32;

    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn has_empty_collision_shape(&self, state: BlockStateId, position: BlockPos) -> bool;

    fn offer_bonus_chest(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn has_randomizable_container(&mut self, position: BlockPos) -> bool;

    fn assign_bonus_chest_loot(&mut self, position: BlockPos, seed: i64);

    fn torch_can_survive(&mut self, position: BlockPos, torch: BlockStateId) -> bool;

    fn offer_bonus_torch(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_bonus_chest<R, W>(
    world: &mut W,
    origin: BlockPos,
    states: BonusChestStates,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, StructureFeatureError>
where
    R: BonusChestRandom,
    W: BonusChestWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let chunk = origin.chunk();
    let minimum_x = chunk
        .x
        .checked_mul(16)
        .ok_or(StructureFeatureError::PositionOverflow)?;
    let minimum_z = chunk
        .z
        .checked_mul(16)
        .ok_or(StructureFeatureError::PositionOverflow)?;
    let mut x_coordinates = ascending_chunk_coordinates(minimum_x)?;
    let mut z_coordinates = ascending_chunk_coordinates(minimum_z)?;
    fisher_yates(&mut x_coordinates, random);
    fisher_yates(&mut z_coordinates, random);

    for x in x_coordinates {
        for z in z_coordinates {
            let candidate = BlockPos::new(x, world.motion_blocking_no_leaves_height(x, z), z);
            let accepted = if world.is_empty_block(candidate) {
                true
            } else {
                let state = world.block_state(candidate);
                world.has_empty_collision_shape(state, candidate)
            };
            if !accepted {
                continue;
            }
            let _ = world.offer_bonus_chest(candidate, states.chest, 2);
            if world.has_randomizable_container(candidate) {
                let seed = random.next_i64();
                world.assign_bonus_chest_loot(candidate, seed);
            }
            for (x_offset, z_offset) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let torch_position = offset(candidate, x_offset, 0, z_offset)?;
                if world.torch_can_survive(torch_position, states.torch) {
                    let _ = world.offer_bonus_torch(torch_position, states.torch, 2);
                }
            }
            return Ok(true);
        }
    }
    Ok(false)
}

fn ascending_chunk_coordinates(minimum: i32) -> Result<[i32; 16], StructureFeatureError> {
    let mut coordinates = [0_i32; 16];
    for (index, coordinate) in coordinates.iter_mut().enumerate() {
        *coordinate = minimum
            .checked_add(index as i32)
            .ok_or(StructureFeatureError::PositionOverflow)?;
    }
    Ok(coordinates)
}

fn fisher_yates(randomized: &mut [i32; 16], random: &mut impl GenerationRandom) {
    for remaining in (2..=16).rev() {
        let bound = NonZeroU32::new(remaining as u32).expect("shuffle bound is nonzero");
        let swap_index = random.next_u32(bound) as usize;
        randomized.swap(remaining - 1, swap_index);
    }
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, StructureFeatureError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(StructureFeatureError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(StructureFeatureError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(StructureFeatureError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum StructureFeatureError {
    #[error("structure-feature position arithmetic overflowed")]
    PositionOverflow,
}
