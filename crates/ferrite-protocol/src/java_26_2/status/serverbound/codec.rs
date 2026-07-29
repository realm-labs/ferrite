use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::status::serverbound::packet::StatusServerboundPacket;
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StatusServerboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("status serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("status serverbound packet {identity} is not part of the required C0 family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
}

pub fn decode_packet(body: &[u8]) -> Result<StatusServerboundPacket, StatusServerboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Status,
        PacketDirection::Serverbound,
        wire_id,
    )
    .ok_or(StatusServerboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:status_request" => StatusServerboundPacket::Request,
        "minecraft:ping_request" => StatusServerboundPacket::Ping(reader.read_i64()?),
        identity => {
            return Err(StatusServerboundCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: StatusServerboundPacket,
) -> Result<Vec<u8>, StatusServerboundCodecError> {
    let identity = match packet {
        StatusServerboundPacket::Request => "minecraft:status_request",
        StatusServerboundPacket::Ping(_) => "minecraft:ping_request",
    };
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Status,
        PacketDirection::Serverbound,
        identity,
    )
    .ok_or(StatusServerboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        StatusServerboundPacket::Request => {}
        StatusServerboundPacket::Ping(token) => writer.write_i64(token)?,
    }
    Ok(writer.into_inner())
}
