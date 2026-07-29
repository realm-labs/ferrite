use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::play::block::{
    direction_from_index, direction_from_player_action, direction_index, pack_block_position,
    unpack_block_position,
};
use crate::java_26_2::play::serverbound::packet::{
    AcceptTeleportation, BlockHit, ChunkBatchReceived, Hand, KeepAlive, MovePlayerPosition,
    MovePlayerPositionRotation, MovePlayerRotation, MovePlayerStatusOnly, MovementFlags,
    PickItemFromBlock, PlayServerboundEntryPacket, PlayerAction, PlayerActionKind, PlayerPosition,
    PlayerRotation, Swing, UseItem, UseItemOn,
};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const ACCEPT_TELEPORTATION: &str = "minecraft:accept_teleportation";
const CHUNK_BATCH_RECEIVED: &str = "minecraft:chunk_batch_received";
const CLIENT_TICK_END: &str = "minecraft:client_tick_end";
const KEEP_ALIVE: &str = "minecraft:keep_alive";
const MOVE_PLAYER_POS: &str = "minecraft:move_player_pos";
const MOVE_PLAYER_POS_ROT: &str = "minecraft:move_player_pos_rot";
const MOVE_PLAYER_ROT: &str = "minecraft:move_player_rot";
const MOVE_PLAYER_STATUS_ONLY: &str = "minecraft:move_player_status_only";
const PICK_ITEM_FROM_BLOCK: &str = "minecraft:pick_item_from_block";
const PLAYER_ACTION: &str = "minecraft:player_action";
const PLAYER_LOADED: &str = "minecraft:player_loaded";
const SWING: &str = "minecraft:swing";
const USE_ITEM_ON: &str = "minecraft:use_item_on";
const USE_ITEM: &str = "minecraft:use_item";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlayServerboundEntryCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("play serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("play serverbound packet {identity} is not part of the required C1/C2 session family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
    #[error("{field} ordinal {value} is invalid")]
    InvalidEnum { field: &'static str, value: i32 },
}

pub fn decode_packet(
    body: &[u8],
) -> Result<PlayServerboundEntryPacket, PlayServerboundEntryCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor =
        PacketCatalog::by_wire_id(ConnectionState::Play, PacketDirection::Serverbound, wire_id)
            .ok_or(PlayServerboundEntryCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        ACCEPT_TELEPORTATION => {
            PlayServerboundEntryPacket::AcceptTeleportation(AcceptTeleportation {
                challenge: reader.read_var_i32()?,
            })
        }
        CHUNK_BATCH_RECEIVED => {
            PlayServerboundEntryPacket::ChunkBatchReceived(ChunkBatchReceived {
                desired_chunks_per_tick: reader.read_f32()?,
            })
        }
        CLIENT_TICK_END => PlayServerboundEntryPacket::ClientTickEnd,
        KEEP_ALIVE => PlayServerboundEntryPacket::KeepAlive(KeepAlive {
            challenge: reader.read_i64()?,
        }),
        MOVE_PLAYER_POS => PlayServerboundEntryPacket::MovePlayerPosition(MovePlayerPosition {
            position: read_position(&mut reader)?,
            flags: MovementFlags::from_wire(reader.read_u8()?),
        }),
        MOVE_PLAYER_POS_ROT => {
            PlayServerboundEntryPacket::MovePlayerPositionRotation(MovePlayerPositionRotation {
                position: read_position(&mut reader)?,
                rotation: read_rotation(&mut reader)?,
                flags: MovementFlags::from_wire(reader.read_u8()?),
            })
        }
        MOVE_PLAYER_ROT => PlayServerboundEntryPacket::MovePlayerRotation(MovePlayerRotation {
            rotation: read_rotation(&mut reader)?,
            flags: MovementFlags::from_wire(reader.read_u8()?),
        }),
        MOVE_PLAYER_STATUS_ONLY => {
            PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
                flags: MovementFlags::from_wire(reader.read_u8()?),
            })
        }
        PICK_ITEM_FROM_BLOCK => PlayServerboundEntryPacket::PickItemFromBlock(PickItemFromBlock {
            position: unpack_block_position(reader.read_i64()?),
            include_data: reader.read_bool()?,
        }),
        PLAYER_ACTION => PlayServerboundEntryPacket::PlayerAction(PlayerAction {
            action: read_action(&mut reader)?,
            position: unpack_block_position(reader.read_i64()?),
            direction: direction_from_player_action(reader.read_u8()?),
            sequence: reader.read_var_i32()?,
        }),
        PLAYER_LOADED => PlayServerboundEntryPacket::PlayerLoaded,
        SWING => PlayServerboundEntryPacket::Swing(Swing {
            hand: read_hand(&mut reader)?,
        }),
        USE_ITEM_ON => PlayServerboundEntryPacket::UseItemOn(UseItemOn {
            hand: read_hand(&mut reader)?,
            hit: read_block_hit(&mut reader)?,
            sequence: reader.read_var_i32()?,
        }),
        USE_ITEM => PlayServerboundEntryPacket::UseItem(UseItem {
            hand: read_hand(&mut reader)?,
            sequence: reader.read_var_i32()?,
            yaw: reader.read_f32()?,
            pitch: reader.read_f32()?,
        }),
        identity => {
            return Err(PlayServerboundEntryCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: PlayServerboundEntryPacket,
) -> Result<Vec<u8>, PlayServerboundEntryCodecError> {
    let identity = packet_identity(packet);
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Play,
        PacketDirection::Serverbound,
        identity,
    )
    .ok_or(PlayServerboundEntryCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        PlayServerboundEntryPacket::AcceptTeleportation(packet) => {
            writer.write_var_i32(packet.challenge)?;
        }
        PlayServerboundEntryPacket::ChunkBatchReceived(packet) => {
            writer.write_f32(packet.desired_chunks_per_tick)?;
        }
        PlayServerboundEntryPacket::ClientTickEnd | PlayServerboundEntryPacket::PlayerLoaded => {}
        PlayServerboundEntryPacket::KeepAlive(packet) => writer.write_i64(packet.challenge)?,
        PlayServerboundEntryPacket::MovePlayerPosition(packet) => {
            write_position(&mut writer, packet.position)?;
            writer.write_u8(packet.flags.to_wire())?;
        }
        PlayServerboundEntryPacket::MovePlayerPositionRotation(packet) => {
            write_position(&mut writer, packet.position)?;
            write_rotation(&mut writer, packet.rotation)?;
            writer.write_u8(packet.flags.to_wire())?;
        }
        PlayServerboundEntryPacket::MovePlayerRotation(packet) => {
            write_rotation(&mut writer, packet.rotation)?;
            writer.write_u8(packet.flags.to_wire())?;
        }
        PlayServerboundEntryPacket::MovePlayerStatusOnly(packet) => {
            writer.write_u8(packet.flags.to_wire())?;
        }
        PlayServerboundEntryPacket::PickItemFromBlock(packet) => {
            writer.write_i64(pack_block_position(packet.position))?;
            writer.write_bool(packet.include_data)?;
        }
        PlayServerboundEntryPacket::PlayerAction(packet) => {
            writer.write_var_i32(packet.action.index())?;
            writer.write_i64(pack_block_position(packet.position))?;
            writer.write_u8(direction_index(packet.direction) as u8)?;
            writer.write_var_i32(packet.sequence)?;
        }
        PlayServerboundEntryPacket::Swing(packet) => {
            writer.write_var_i32(packet.hand.index())?;
        }
        PlayServerboundEntryPacket::UseItemOn(packet) => {
            writer.write_var_i32(packet.hand.index())?;
            write_block_hit(&mut writer, packet.hit)?;
            writer.write_var_i32(packet.sequence)?;
        }
        PlayServerboundEntryPacket::UseItem(packet) => {
            writer.write_var_i32(packet.hand.index())?;
            writer.write_var_i32(packet.sequence)?;
            writer.write_f32(packet.yaw)?;
            writer.write_f32(packet.pitch)?;
        }
    }
    Ok(writer.into_inner())
}

#[must_use]
pub const fn packet_identity(packet: PlayServerboundEntryPacket) -> &'static str {
    match packet {
        PlayServerboundEntryPacket::AcceptTeleportation(_) => ACCEPT_TELEPORTATION,
        PlayServerboundEntryPacket::ChunkBatchReceived(_) => CHUNK_BATCH_RECEIVED,
        PlayServerboundEntryPacket::ClientTickEnd => CLIENT_TICK_END,
        PlayServerboundEntryPacket::KeepAlive(_) => KEEP_ALIVE,
        PlayServerboundEntryPacket::MovePlayerPosition(_) => MOVE_PLAYER_POS,
        PlayServerboundEntryPacket::MovePlayerPositionRotation(_) => MOVE_PLAYER_POS_ROT,
        PlayServerboundEntryPacket::MovePlayerRotation(_) => MOVE_PLAYER_ROT,
        PlayServerboundEntryPacket::MovePlayerStatusOnly(_) => MOVE_PLAYER_STATUS_ONLY,
        PlayServerboundEntryPacket::PickItemFromBlock(_) => PICK_ITEM_FROM_BLOCK,
        PlayServerboundEntryPacket::PlayerAction(_) => PLAYER_ACTION,
        PlayServerboundEntryPacket::PlayerLoaded => PLAYER_LOADED,
        PlayServerboundEntryPacket::Swing(_) => SWING,
        PlayServerboundEntryPacket::UseItemOn(_) => USE_ITEM_ON,
        PlayServerboundEntryPacket::UseItem(_) => USE_ITEM,
    }
}

fn read_position(
    reader: &mut WireReader<'_>,
) -> Result<PlayerPosition, PlayServerboundEntryCodecError> {
    Ok(PlayerPosition {
        x: reader.read_f64()?,
        y: reader.read_f64()?,
        z: reader.read_f64()?,
    })
}

fn write_position(
    writer: &mut WireWriter,
    position: PlayerPosition,
) -> Result<(), PlayServerboundEntryCodecError> {
    writer.write_f64(position.x)?;
    writer.write_f64(position.y)?;
    writer.write_f64(position.z)?;
    Ok(())
}

fn read_rotation(
    reader: &mut WireReader<'_>,
) -> Result<PlayerRotation, PlayServerboundEntryCodecError> {
    Ok(PlayerRotation {
        yaw: reader.read_f32()?,
        pitch: reader.read_f32()?,
    })
}

fn write_rotation(
    writer: &mut WireWriter,
    rotation: PlayerRotation,
) -> Result<(), PlayServerboundEntryCodecError> {
    writer.write_f32(rotation.yaw)?;
    writer.write_f32(rotation.pitch)?;
    Ok(())
}

fn read_action(
    reader: &mut WireReader<'_>,
) -> Result<PlayerActionKind, PlayServerboundEntryCodecError> {
    let value = reader.read_var_i32()?;
    PlayerActionKind::from_index(value).ok_or(PlayServerboundEntryCodecError::InvalidEnum {
        field: "player action",
        value,
    })
}

fn read_hand(reader: &mut WireReader<'_>) -> Result<Hand, PlayServerboundEntryCodecError> {
    let value = reader.read_var_i32()?;
    Hand::from_index(value).ok_or(PlayServerboundEntryCodecError::InvalidEnum {
        field: "hand",
        value,
    })
}

fn read_block_hit(reader: &mut WireReader<'_>) -> Result<BlockHit, PlayServerboundEntryCodecError> {
    let position = unpack_block_position(reader.read_i64()?);
    let direction_value = reader.read_var_i32()?;
    let direction = direction_from_index(direction_value).map_err(|_| {
        PlayServerboundEntryCodecError::InvalidEnum {
            field: "block hit direction",
            value: direction_value,
        }
    })?;
    Ok(BlockHit {
        position,
        direction,
        offset_x: reader.read_f32()?,
        offset_y: reader.read_f32()?,
        offset_z: reader.read_f32()?,
        inside: reader.read_bool()?,
        world_border_hit: reader.read_bool()?,
    })
}

fn write_block_hit(
    writer: &mut WireWriter,
    hit: BlockHit,
) -> Result<(), PlayServerboundEntryCodecError> {
    writer.write_i64(pack_block_position(hit.position))?;
    writer.write_var_i32(direction_index(hit.direction))?;
    writer.write_f32(hit.offset_x)?;
    writer.write_f32(hit.offset_y)?;
    writer.write_f32(hit.offset_z)?;
    writer.write_bool(hit.inside)?;
    writer.write_bool(hit.world_border_hit)?;
    Ok(())
}
