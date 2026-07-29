use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use ferrite_gameplay::player::state::Vec3;
use ferrite_simulation::command::{CommandError, CommandSource, RegionCommand};
use ferrite_simulation::tick::GameTick;
use ferrite_world::id::BlockStateId;
use thiserror::Error;

pub const BLOCK_INTERACTION_PATH: &str = "player/block_interaction";
pub const BLOCK_RESULT_PATH: &str = "player/block_result";
pub const BLOCK_UPDATE_PATH: &str = "world/block_update";
const COMMAND_MAGIC: &[u8; 4] = b"FBI1";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockInteractionCommand {
    pub player: StableEntityId,
    pub intent: BlockIntent,
    pub eye: Vec3,
    pub interaction_range: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockIntent {
    StartDestroy {
        position: BlockPos,
    },
    AbortDestroy {
        position: BlockPos,
    },
    StopDestroy {
        position: BlockPos,
    },
    UseOn {
        position: BlockPos,
        direction: Direction,
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
        inside: bool,
        world_border_hit: bool,
        interaction_allowed: bool,
        placement_state: BlockStateId,
    },
}

impl BlockInteractionCommand {
    pub fn into_region_command(
        self,
        target: SimulationRegionKey,
        tick: GameTick,
        sequence: u64,
    ) -> Result<RegionCommand, BlockCommandError> {
        Ok(RegionCommand::new(
            target,
            tick,
            CommandSource::Player(self.player),
            sequence,
            ResourceId::new("ferrite", BLOCK_INTERACTION_PATH)?,
            self.encode(),
        )?)
    }

    #[must_use]
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(58);
        bytes.extend_from_slice(COMMAND_MAGIC);
        bytes.push(intent_tag(self.intent));
        bytes.extend_from_slice(&self.player.to_be_bytes());
        let position = intent_position(self.intent);
        push_position(&mut bytes, position);
        match self.intent {
            BlockIntent::UseOn {
                direction,
                offset_x,
                offset_y,
                offset_z,
                inside,
                world_border_hit,
                interaction_allowed,
                placement_state,
                ..
            } => {
                bytes.push(direction_tag(direction));
                bytes.extend_from_slice(&offset_x.to_bits().to_be_bytes());
                bytes.extend_from_slice(&offset_y.to_bits().to_be_bytes());
                bytes.extend_from_slice(&offset_z.to_bits().to_be_bytes());
                bytes.push(u8::from(inside));
                bytes.push(u8::from(world_border_hit));
                bytes.push(u8::from(interaction_allowed));
                bytes.extend_from_slice(&placement_state.get().to_be_bytes());
            }
            _ => bytes.extend_from_slice(&[0; 20]),
        }
        for value in [self.eye.x, self.eye.y, self.eye.z, self.interaction_range] {
            bytes.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlockCommandError> {
        if bytes.len() != 85 || &bytes[..4] != COMMAND_MAGIC {
            return Err(BlockCommandError::InvalidPayload);
        }
        let tag = bytes[4];
        let player = StableEntityId::new(read_u128(bytes, 5)?)
            .map_err(|_| BlockCommandError::InvalidPayload)?;
        let position = read_position(bytes, 21)?;
        let direction = read_direction(bytes[33])?;
        let offset_x = f32::from_bits(read_u32(bytes, 34)?);
        let offset_y = f32::from_bits(read_u32(bytes, 38)?);
        let offset_z = f32::from_bits(read_u32(bytes, 42)?);
        let inside = read_bool(bytes[46])?;
        let world_border_hit = read_bool(bytes[47])?;
        let interaction_allowed = read_bool(bytes[48])?;
        let placement_state = BlockStateId::new(read_u32(bytes, 49)?);
        let intent = match tag {
            0 => BlockIntent::StartDestroy { position },
            1 => BlockIntent::AbortDestroy { position },
            2 => BlockIntent::StopDestroy { position },
            3 => BlockIntent::UseOn {
                position,
                direction,
                offset_x,
                offset_y,
                offset_z,
                inside,
                world_border_hit,
                interaction_allowed,
                placement_state,
            },
            _ => return Err(BlockCommandError::InvalidPayload),
        };
        Ok(Self {
            player,
            intent,
            eye: Vec3::new(
                f64::from_bits(read_u64(bytes, 53)?),
                f64::from_bits(read_u64(bytes, 61)?),
                f64::from_bits(read_u64(bytes, 69)?),
            ),
            interaction_range: f64::from_bits(read_u64(bytes, 77)?),
        })
    }
}

const fn intent_tag(intent: BlockIntent) -> u8 {
    match intent {
        BlockIntent::StartDestroy { .. } => 0,
        BlockIntent::AbortDestroy { .. } => 1,
        BlockIntent::StopDestroy { .. } => 2,
        BlockIntent::UseOn { .. } => 3,
    }
}

const fn intent_position(intent: BlockIntent) -> BlockPos {
    match intent {
        BlockIntent::StartDestroy { position }
        | BlockIntent::AbortDestroy { position }
        | BlockIntent::StopDestroy { position }
        | BlockIntent::UseOn { position, .. } => position,
    }
}

const fn direction_tag(direction: Direction) -> u8 {
    match direction {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

fn read_direction(value: u8) -> Result<Direction, BlockCommandError> {
    Direction::ALL
        .get(usize::from(value))
        .copied()
        .ok_or(BlockCommandError::InvalidPayload)
}

fn read_bool(value: u8) -> Result<bool, BlockCommandError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(BlockCommandError::InvalidPayload),
    }
}

fn push_position(bytes: &mut Vec<u8>, position: BlockPos) {
    bytes.extend_from_slice(&position.x.to_be_bytes());
    bytes.extend_from_slice(&position.y.to_be_bytes());
    bytes.extend_from_slice(&position.z.to_be_bytes());
}

fn read_position(bytes: &[u8], offset: usize) -> Result<BlockPos, BlockCommandError> {
    Ok(BlockPos::new(
        read_i32(bytes, offset)?,
        read_i32(bytes, offset + 4)?,
        read_i32(bytes, offset + 8)?,
    ))
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, BlockCommandError> {
    Ok(i32::from_be_bytes(read_array(bytes, offset)?))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, BlockCommandError> {
    Ok(u32::from_be_bytes(read_array(bytes, offset)?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, BlockCommandError> {
    Ok(u64::from_be_bytes(read_array(bytes, offset)?))
}

fn read_u128(bytes: &[u8], offset: usize) -> Result<u128, BlockCommandError> {
    Ok(u128::from_be_bytes(read_array(bytes, offset)?))
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N], BlockCommandError> {
    bytes
        .get(offset..offset + N)
        .and_then(|slice| slice.try_into().ok())
        .ok_or(BlockCommandError::InvalidPayload)
}

#[derive(Debug, Error)]
pub enum BlockCommandError {
    #[error("block interaction command payload is invalid")]
    InvalidPayload,
    #[error(transparent)]
    Resource(#[from] ResourceIdError),
    #[error(transparent)]
    Command(#[from] CommandError),
}
