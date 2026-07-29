use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::play::serverbound::packet::{
    AcceptTeleportation, PlayServerboundEntryPacket,
};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const ACCEPT_TELEPORTATION: &str = "minecraft:accept_teleportation";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlayServerboundEntryCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("play serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("play serverbound packet {identity} is not part of the required C1 entry family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
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
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Play,
        PacketDirection::Serverbound,
        ACCEPT_TELEPORTATION,
    )
    .ok_or(PlayServerboundEntryCodecError::MissingCatalogIdentity {
        identity: ACCEPT_TELEPORTATION,
    })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        PlayServerboundEntryPacket::AcceptTeleportation(packet) => {
            writer.write_var_i32(packet.challenge)?;
        }
    }
    Ok(writer.into_inner())
}
