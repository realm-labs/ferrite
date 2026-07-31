use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::play::block::{pack_block_position, unpack_block_position};
use crate::java_26_2::play::clientbound::admin_presentation::packet::{
    AdminPresentationPacket, Vec3i,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_UTF_CODE_UNITS: usize = 32_767;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AdminPresentationCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("play clientbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("play clientbound packet {identity} is not admin presentation")]
    OtherPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing admin-presentation identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
}

pub fn decode_packet(body: &[u8]) -> Result<AdminPresentationPacket, AdminPresentationCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor =
        PacketCatalog::by_wire_id(ConnectionState::Play, PacketDirection::Clientbound, wire_id)
            .ok_or(AdminPresentationCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:game_rule_values" => decode_game_rules(&mut reader)?,
        "minecraft:game_test_highlight_pos" => AdminPresentationPacket::GameTestHighlightPosition {
            absolute: unpack_block_position(reader.read_i64()?),
            relative: unpack_block_position(reader.read_i64()?),
        },
        "minecraft:low_disk_space_warning" => AdminPresentationPacket::LowDiskSpaceWarning,
        "minecraft:test_instance_block_status" => {
            AdminPresentationPacket::TestInstanceBlockStatus {
                status: TextComponentNbt::from_network_nbt(NetworkNbt::read(
                    &mut reader,
                    NbtQuota::Trusted,
                )?)?,
                size: if reader.read_bool()? {
                    Some(Vec3i {
                        x: reader.read_var_i32()?,
                        y: reader.read_var_i32()?,
                        z: reader.read_var_i32()?,
                    })
                } else {
                    None
                },
            }
        }
        identity => return Err(AdminPresentationCodecError::OtherPacketIdentity { identity }),
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &AdminPresentationPacket,
) -> Result<Vec<u8>, AdminPresentationCodecError> {
    let identity = packet.kind().identity();
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Play,
        PacketDirection::Clientbound,
        identity,
    )
    .ok_or(AdminPresentationCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        AdminPresentationPacket::GameRuleValues(values) => {
            writer.write_count("game-rule values", values.len(), MAX_INFLATED_PACKET_LENGTH)?;
            for (key, value) in values {
                key.write(&mut writer)?;
                writer.write_utf(value, MAX_UTF_CODE_UNITS)?;
            }
        }
        AdminPresentationPacket::GameTestHighlightPosition { absolute, relative } => {
            writer.write_i64(pack_block_position(*absolute))?;
            writer.write_i64(pack_block_position(*relative))?;
        }
        AdminPresentationPacket::LowDiskSpaceWarning => {}
        AdminPresentationPacket::TestInstanceBlockStatus { status, size } => {
            status.network_nbt().write(&mut writer)?;
            writer.write_bool(size.is_some())?;
            if let Some(size) = size {
                writer.write_var_i32(size.x)?;
                writer.write_var_i32(size.y)?;
                writer.write_var_i32(size.z)?;
            }
        }
    }
    Ok(writer.into_inner())
}

fn decode_game_rules(
    reader: &mut WireReader<'_>,
) -> Result<AdminPresentationPacket, AdminPresentationCodecError> {
    let count = reader.read_count("game-rule values", MAX_INFLATED_PACKET_LENGTH)?;
    let mut values = BTreeMap::new();
    for _ in 0..count {
        values.insert(
            read_identifier(reader)?,
            reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
        );
    }
    Ok(AdminPresentationPacket::GameRuleValues(values))
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, AdminPresentationCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
