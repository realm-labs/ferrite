use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use crate::java_26_2::handshake::packet::{ClientIntention, ClientIntentionPacket};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::frame::MAX_FRAME_LENGTH;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_HOST_CODE_UNITS: usize = 255;
const INTENTION_IDENTITY: &str = "minecraft:intention";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HandshakeCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("handshake serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
    #[error("handshake intention enum ID {id} is invalid")]
    InvalidIntention { id: i32 },
}

pub fn decode_packet(body: &[u8]) -> Result<ClientIntentionPacket, HandshakeCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Handshake,
        PacketDirection::Serverbound,
        wire_id,
    )
    .ok_or(HandshakeCodecError::UnknownPacketId { id: wire_id })?;
    if descriptor.identity() != INTENTION_IDENTITY {
        return Err(HandshakeCodecError::UnknownPacketId { id: wire_id });
    }
    let protocol_version = reader.read_var_i32()?;
    let host = reader.read_utf(MAX_HOST_CODE_UNITS)?.into_owned();
    let port = reader.read_u16()?;
    let intention_id = reader.read_var_i32()?;
    let intention = ClientIntention::from_id(intention_id)
        .ok_or(HandshakeCodecError::InvalidIntention { id: intention_id })?;
    reader.finish()?;
    Ok(ClientIntentionPacket {
        protocol_version,
        host,
        port,
        intention,
    })
}

pub fn encode_packet(packet: &ClientIntentionPacket) -> Result<Vec<u8>, HandshakeCodecError> {
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Handshake,
        PacketDirection::Serverbound,
        INTENTION_IDENTITY,
    )
    .ok_or(HandshakeCodecError::MissingCatalogIdentity {
        identity: INTENTION_IDENTITY,
    })?;
    let mut writer = WireWriter::new(MAX_FRAME_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    writer.write_var_i32(packet.protocol_version)?;
    writer.write_utf(&packet.host, MAX_HOST_CODE_UNITS)?;
    writer.write_u16(packet.port)?;
    writer.write_var_i32(packet.intention.id())?;
    Ok(writer.into_inner())
}
