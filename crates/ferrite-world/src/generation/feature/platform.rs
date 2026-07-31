//! Void-start platform, End platform, and End gateway generation.

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use thiserror::Error;

use crate::generation::status::chebyshev_distance;
use crate::id::BlockStateId;

pub const VOID_PLATFORM_WRITE_FLAGS: u32 = 2;
pub const END_PLATFORM_WRITE_FLAGS: u32 = 3;

pub trait PlatformWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn offer_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn destroy_block_with_drops(&mut self, position: BlockPos) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoidPlatformStates {
    pub cobblestone: BlockStateId,
    pub stone: BlockStateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndPlatformStates {
    pub obsidian: BlockStateId,
    pub air: BlockStateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndGatewayConfig {
    pub gateway: BlockStateId,
    pub bedrock: BlockStateId,
    pub air: BlockStateId,
    pub exit: Option<BlockPos>,
    pub exact: bool,
}

pub trait EndGatewayWorld {
    fn offer_gateway_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn configure_gateway_exit(&mut self, position: BlockPos, exit: BlockPos, exact: bool) -> bool;
}

pub fn place_void_start_platform<W: PlatformWorld>(
    world: &mut W,
    origin: BlockPos,
    states: VoidPlatformStates,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, PlatformError> {
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let chunk = origin.chunk();
    if chebyshev_distance(chunk, ChunkPos::new(0, 0)) > 1 {
        return Ok(true);
    }
    let center_y = origin
        .y
        .checked_add(3)
        .ok_or(PlatformError::PositionOverflow)?;
    let chunk_min_x = chunk
        .x
        .checked_mul(16)
        .ok_or(PlatformError::PositionOverflow)?;
    let chunk_min_z = chunk
        .z
        .checked_mul(16)
        .ok_or(PlatformError::PositionOverflow)?;
    for z_offset in 0..16 {
        let z = chunk_min_z
            .checked_add(z_offset)
            .ok_or(PlatformError::PositionOverflow)?;
        for x_offset in 0..16 {
            let x = chunk_min_x
                .checked_add(x_offset)
                .ok_or(PlatformError::PositionOverflow)?;
            if x.abs_diff(8).max(z.abs_diff(8)) > 16 {
                continue;
            }
            let state = if x == 8 && z == 8 {
                states.cobblestone
            } else {
                states.stone
            };
            let _ = world.offer_block(
                BlockPos::new(x, center_y, z),
                state,
                VOID_PLATFORM_WRITE_FLAGS,
            );
        }
    }
    Ok(true)
}

pub fn place_end_platform<W: PlatformWorld>(
    world: &mut W,
    origin: BlockPos,
    states: EndPlatformStates,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, PlatformError> {
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    create_end_platform(world, origin, states, false)?;
    Ok(true)
}

pub fn create_end_platform<W: PlatformWorld>(
    world: &mut W,
    origin: BlockPos,
    states: EndPlatformStates,
    destroy_existing: bool,
) -> Result<(), PlatformError> {
    for z_offset in -2..=2 {
        for x_offset in -2..=2 {
            for y_offset in -1..=2 {
                let position = offset(origin, x_offset, y_offset, z_offset)?;
                let target = if y_offset == -1 {
                    states.obsidian
                } else {
                    states.air
                };
                if world.block_state(position) == target {
                    continue;
                }
                if destroy_existing {
                    let _ = world.destroy_block_with_drops(position);
                }
                let _ = world.offer_block(position, target, END_PLATFORM_WRITE_FLAGS);
            }
        }
    }
    Ok(())
}

pub fn place_end_gateway<W: EndGatewayWorld>(
    world: &mut W,
    origin: BlockPos,
    config: EndGatewayConfig,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, PlatformError> {
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    for z_offset in -1..=1 {
        for y_offset in -2_i32..=2 {
            for x_offset in -1_i32..=1 {
                let position = offset(origin, x_offset, y_offset, z_offset)?;
                let state = if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                    config.gateway
                } else if y_offset == 0 {
                    config.air
                } else if y_offset.abs() == 1 && (x_offset == 0 || z_offset == 0)
                    || y_offset.abs() == 2 && x_offset == 0 && z_offset == 0
                {
                    config.bedrock
                } else {
                    config.air
                };
                let _ = world.offer_gateway_block(position, state, END_PLATFORM_WRITE_FLAGS);
                if position == origin
                    && let Some(exit) = config.exit
                {
                    let _ = world.configure_gateway_exit(origin, exit, config.exact);
                }
            }
        }
    }
    Ok(true)
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, PlatformError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(PlatformError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(PlatformError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(PlatformError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PlatformError {
    #[error("platform position arithmetic overflowed")]
    PositionOverflow,
}
