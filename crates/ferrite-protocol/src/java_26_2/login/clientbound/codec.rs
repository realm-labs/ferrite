use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::login::clientbound::packet::{LoginClientboundPacket, LoginFinished};
use crate::java_26_2::login::component_json::{LoginDisconnectReason, LoginDisconnectReasonError};
use crate::java_26_2::login::profile::{GameProfile, ProfileProperty};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_DISCONNECT_COMPONENT_CODE_UNITS: usize = 262_144;
const MAX_PROFILE_NAME_CODE_UNITS: usize = 16;
const MAX_PROFILE_PROPERTIES: usize = 16;
const MAX_PROPERTY_NAME_CODE_UNITS: usize = 64;
const MAX_PROPERTY_VALUE_CODE_UNITS: usize = 32_767;
const MAX_PROPERTY_SIGNATURE_CODE_UNITS: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LoginClientboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidDisconnectReason(#[from] LoginDisconnectReasonError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("login clientbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("login clientbound packet {identity} is not part of the required C1 family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
}

pub fn decode_packet(body: &[u8]) -> Result<LoginClientboundPacket, LoginClientboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Login,
        PacketDirection::Clientbound,
        wire_id,
    )
    .ok_or(LoginClientboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:login_disconnect" => {
            let json = reader
                .read_utf(MAX_DISCONNECT_COMPONENT_CODE_UNITS)?
                .into_owned();
            LoginClientboundPacket::Disconnect(LoginDisconnectReason::from_json(json)?)
        }
        "minecraft:login_finished" => decode_finished(&mut reader)?,
        "minecraft:login_compression" => {
            LoginClientboundPacket::Compression(reader.read_var_i32()?)
        }
        identity => {
            return Err(LoginClientboundCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &LoginClientboundPacket,
) -> Result<Vec<u8>, LoginClientboundCodecError> {
    let identity = packet_identity(packet);
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Login,
        PacketDirection::Clientbound,
        identity,
    )
    .ok_or(LoginClientboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        LoginClientboundPacket::Disconnect(reason) => {
            writer.write_utf(reason.as_json(), MAX_DISCONNECT_COMPONENT_CODE_UNITS)?;
        }
        LoginClientboundPacket::Finished(finished) => encode_finished(&mut writer, finished)?,
        LoginClientboundPacket::Compression(threshold) => writer.write_var_i32(*threshold)?,
    }
    Ok(writer.into_inner())
}

fn packet_identity(packet: &LoginClientboundPacket) -> &'static str {
    match packet {
        LoginClientboundPacket::Disconnect(_) => "minecraft:login_disconnect",
        LoginClientboundPacket::Finished(_) => "minecraft:login_finished",
        LoginClientboundPacket::Compression(_) => "minecraft:login_compression",
    }
}

fn decode_finished(
    reader: &mut WireReader<'_>,
) -> Result<LoginClientboundPacket, LoginClientboundCodecError> {
    let id = reader.read_u128()?;
    let name = reader.read_utf(MAX_PROFILE_NAME_CODE_UNITS)?.into_owned();
    let property_count = reader.read_count("profile properties", MAX_PROFILE_PROPERTIES)?;
    let mut properties = Vec::with_capacity(property_count);
    for _ in 0..property_count {
        properties.push(ProfileProperty {
            name: reader.read_utf(MAX_PROPERTY_NAME_CODE_UNITS)?.into_owned(),
            value: reader.read_utf(MAX_PROPERTY_VALUE_CODE_UNITS)?.into_owned(),
            signature: if reader.read_bool()? {
                Some(
                    reader
                        .read_utf(MAX_PROPERTY_SIGNATURE_CODE_UNITS)?
                        .into_owned(),
                )
            } else {
                None
            },
        });
    }
    Ok(LoginClientboundPacket::Finished(LoginFinished {
        profile: GameProfile {
            id,
            name,
            properties,
        },
        server_session_id: reader.read_u128()?,
    }))
}

fn encode_finished(
    writer: &mut WireWriter,
    finished: &LoginFinished,
) -> Result<(), LoginClientboundCodecError> {
    writer.write_u128(finished.profile.id)?;
    writer.write_utf(&finished.profile.name, MAX_PROFILE_NAME_CODE_UNITS)?;
    writer.write_count(
        "profile properties",
        finished.profile.properties.len(),
        MAX_PROFILE_PROPERTIES,
    )?;
    for property in &finished.profile.properties {
        writer.write_utf(&property.name, MAX_PROPERTY_NAME_CODE_UNITS)?;
        writer.write_utf(&property.value, MAX_PROPERTY_VALUE_CODE_UNITS)?;
        writer.write_bool(property.signature.is_some())?;
        if let Some(signature) = &property.signature {
            writer.write_utf(signature, MAX_PROPERTY_SIGNATURE_CODE_UNITS)?;
        }
    }
    writer.write_u128(finished.server_session_id)?;
    Ok(())
}
