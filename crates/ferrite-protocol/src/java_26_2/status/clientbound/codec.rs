use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::status::clientbound::json::{self, StatusJsonError};
use crate::java_26_2::status::clientbound::packet::StatusClientboundPacket;
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_STATUS_JSON_CODE_UNITS: usize = 32_767;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StatusClientboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error(transparent)]
    Json(#[from] StatusJsonError),
    #[error("status clientbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("status clientbound packet {identity} is not part of the required C0 family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
}

pub fn decode_packet(body: &[u8]) -> Result<StatusClientboundPacket, StatusClientboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Status,
        PacketDirection::Clientbound,
        wire_id,
    )
    .ok_or(StatusClientboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:status_response" => {
            let encoded = reader.read_utf(MAX_STATUS_JSON_CODE_UNITS)?;
            StatusClientboundPacket::Response(json::decode(&encoded)?)
        }
        "minecraft:pong_response" => StatusClientboundPacket::Pong(reader.read_i64()?),
        identity => {
            return Err(StatusClientboundCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &StatusClientboundPacket,
) -> Result<Vec<u8>, StatusClientboundCodecError> {
    let identity = match packet {
        StatusClientboundPacket::Response(_) => "minecraft:status_response",
        StatusClientboundPacket::Pong(_) => "minecraft:pong_response",
    };
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Status,
        PacketDirection::Clientbound,
        identity,
    )
    .ok_or(StatusClientboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        StatusClientboundPacket::Response(status) => {
            writer.write_utf(&json::encode(status)?, MAX_STATUS_JSON_CODE_UNITS)?;
        }
        StatusClientboundPacket::Pong(token) => writer.write_i64(*token)?,
    }
    Ok(writer.into_inner())
}
