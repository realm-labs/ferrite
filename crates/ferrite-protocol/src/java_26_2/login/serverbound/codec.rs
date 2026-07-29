use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::login::serverbound::packet::{LoginHello, LoginServerboundPacket};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_PROFILE_NAME_CODE_UNITS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LoginServerboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("login serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("login serverbound packet {identity} is not part of the required C1 family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
}

pub fn decode_packet(body: &[u8]) -> Result<LoginServerboundPacket, LoginServerboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Login,
        PacketDirection::Serverbound,
        wire_id,
    )
    .ok_or(LoginServerboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:hello" => LoginServerboundPacket::Hello(LoginHello {
            name: reader.read_utf(MAX_PROFILE_NAME_CODE_UNITS)?.into_owned(),
            supplied_profile_id: reader.read_u128()?,
        }),
        "minecraft:login_acknowledged" => LoginServerboundPacket::Acknowledged,
        identity => {
            return Err(LoginServerboundCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &LoginServerboundPacket,
) -> Result<Vec<u8>, LoginServerboundCodecError> {
    let identity = match packet {
        LoginServerboundPacket::Hello(_) => "minecraft:hello",
        LoginServerboundPacket::Acknowledged => "minecraft:login_acknowledged",
    };
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Login,
        PacketDirection::Serverbound,
        identity,
    )
    .ok_or(LoginServerboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        LoginServerboundPacket::Hello(hello) => {
            writer.write_utf(&hello.name, MAX_PROFILE_NAME_CODE_UNITS)?;
            writer.write_u128(hello.supplied_profile_id)?;
        }
        LoginServerboundPacket::Acknowledged => {}
    }
    Ok(writer.into_inner())
}
