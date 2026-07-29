use std::collections::BTreeSet;

use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::configuration::clientbound::packet::{
    ConfigurationClientboundPacket, CustomPayload, RegistryData, RegistryEntry, RegistryTags,
    TagDefinition,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::known_pack::KnownPack;
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_UTF_CODE_UNITS: usize = 32_767;
const MAX_UNKNOWN_CLIENTBOUND_PAYLOAD: usize = 1_048_576;
const BRAND_CHANNEL: &str = "minecraft:brand";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigurationClientboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidIdentifier(#[from] IdentifierError),
    #[error(transparent)]
    InvalidNbt(#[from] NbtError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("configuration clientbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("configuration clientbound packet {identity} is not part of the required C1 family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
    #[error("discarded custom payload {channel} cannot be re-encoded")]
    CannotEncodeDiscardedPayload { channel: Identifier },
}

pub fn decode_packet(
    body: &[u8],
) -> Result<ConfigurationClientboundPacket, ConfigurationClientboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Configuration,
        PacketDirection::Clientbound,
        wire_id,
    )
    .ok_or(ConfigurationClientboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:custom_payload" => decode_custom_payload(&mut reader)?,
        "minecraft:disconnect" => {
            let nbt = NetworkNbt::read(&mut reader, NbtQuota::Trusted)?;
            ConfigurationClientboundPacket::Disconnect(TextComponentNbt::from_network_nbt(nbt)?)
        }
        "minecraft:finish_configuration" => ConfigurationClientboundPacket::FinishConfiguration,
        "minecraft:keep_alive" => ConfigurationClientboundPacket::KeepAlive(reader.read_i64()?),
        "minecraft:ping" => ConfigurationClientboundPacket::Ping(reader.read_i32()?),
        "minecraft:registry_data" => decode_registry_data(&mut reader)?,
        "minecraft:update_enabled_features" => decode_features(&mut reader)?,
        "minecraft:update_tags" => decode_tags(&mut reader)?,
        "minecraft:select_known_packs" => decode_known_packs(&mut reader)?,
        identity => {
            return Err(ConfigurationClientboundCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &ConfigurationClientboundPacket,
) -> Result<Vec<u8>, ConfigurationClientboundCodecError> {
    let identity = packet_identity(packet);
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Configuration,
        PacketDirection::Clientbound,
        identity,
    )
    .ok_or(ConfigurationClientboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        ConfigurationClientboundPacket::CustomPayload(payload) => {
            encode_custom_payload(&mut writer, payload)?;
        }
        ConfigurationClientboundPacket::Disconnect(reason) => {
            reason.network_nbt().write(&mut writer)?;
        }
        ConfigurationClientboundPacket::FinishConfiguration => {}
        ConfigurationClientboundPacket::KeepAlive(token) => writer.write_i64(*token)?,
        ConfigurationClientboundPacket::Ping(token) => writer.write_i32(*token)?,
        ConfigurationClientboundPacket::RegistryData(data) => {
            encode_registry_data(&mut writer, data)?;
        }
        ConfigurationClientboundPacket::UpdateEnabledFeatures(features) => {
            write_count(&mut writer, "enabled features", features.len())?;
            for feature in features {
                feature.write(&mut writer)?;
            }
        }
        ConfigurationClientboundPacket::UpdateTags(registries) => {
            encode_tags(&mut writer, registries)?;
        }
        ConfigurationClientboundPacket::SelectKnownPacks(packs) => {
            write_count(&mut writer, "known packs", packs.len())?;
            for pack in packs {
                writer.write_utf(&pack.namespace, MAX_UTF_CODE_UNITS)?;
                writer.write_utf(&pack.id, MAX_UTF_CODE_UNITS)?;
                writer.write_utf(&pack.version, MAX_UTF_CODE_UNITS)?;
            }
        }
    }
    Ok(writer.into_inner())
}

fn packet_identity(packet: &ConfigurationClientboundPacket) -> &'static str {
    match packet {
        ConfigurationClientboundPacket::CustomPayload(_) => "minecraft:custom_payload",
        ConfigurationClientboundPacket::Disconnect(_) => "minecraft:disconnect",
        ConfigurationClientboundPacket::FinishConfiguration => "minecraft:finish_configuration",
        ConfigurationClientboundPacket::KeepAlive(_) => "minecraft:keep_alive",
        ConfigurationClientboundPacket::Ping(_) => "minecraft:ping",
        ConfigurationClientboundPacket::RegistryData(_) => "minecraft:registry_data",
        ConfigurationClientboundPacket::UpdateEnabledFeatures(_) => {
            "minecraft:update_enabled_features"
        }
        ConfigurationClientboundPacket::UpdateTags(_) => "minecraft:update_tags",
        ConfigurationClientboundPacket::SelectKnownPacks(_) => "minecraft:select_known_packs",
    }
}

fn decode_custom_payload(
    reader: &mut WireReader<'_>,
) -> Result<ConfigurationClientboundPacket, ConfigurationClientboundCodecError> {
    let channel = read_identifier(reader)?;
    if channel.to_string() == BRAND_CHANNEL {
        let brand = reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned();
        Ok(ConfigurationClientboundPacket::CustomPayload(
            CustomPayload::Brand(brand),
        ))
    } else {
        let length = reader
            .read_bounded_remaining(
                "unknown clientbound custom payload",
                MAX_UNKNOWN_CLIENTBOUND_PAYLOAD,
            )?
            .len();
        Ok(ConfigurationClientboundPacket::CustomPayload(
            CustomPayload::Discarded { channel, length },
        ))
    }
}

fn encode_custom_payload(
    writer: &mut WireWriter,
    payload: &CustomPayload,
) -> Result<(), ConfigurationClientboundCodecError> {
    match payload {
        CustomPayload::Brand(brand) => {
            Identifier::parse(BRAND_CHANNEL)?.write(writer)?;
            writer.write_utf(brand, MAX_UTF_CODE_UNITS)?;
            Ok(())
        }
        CustomPayload::Discarded { channel, .. } => Err(
            ConfigurationClientboundCodecError::CannotEncodeDiscardedPayload {
                channel: channel.clone(),
            },
        ),
    }
}

fn decode_registry_data(
    reader: &mut WireReader<'_>,
) -> Result<ConfigurationClientboundPacket, ConfigurationClientboundCodecError> {
    let registry = read_identifier(reader)?;
    let count = read_count(reader, "registry entries")?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let id = read_identifier(reader)?;
        let data = if reader.read_bool()? {
            Some(NetworkNbt::read(reader, NbtQuota::Default)?)
        } else {
            None
        };
        entries.push(RegistryEntry { id, data });
    }
    Ok(ConfigurationClientboundPacket::RegistryData(RegistryData {
        registry,
        entries,
    }))
}

fn encode_registry_data(
    writer: &mut WireWriter,
    data: &RegistryData,
) -> Result<(), ConfigurationClientboundCodecError> {
    data.registry.write(writer)?;
    write_count(writer, "registry entries", data.entries.len())?;
    for entry in &data.entries {
        entry.id.write(writer)?;
        writer.write_bool(entry.data.is_some())?;
        if let Some(nbt) = &entry.data {
            nbt.write(writer)?;
        }
    }
    Ok(())
}

fn decode_features(
    reader: &mut WireReader<'_>,
) -> Result<ConfigurationClientboundPacket, ConfigurationClientboundCodecError> {
    let count = read_count(reader, "enabled features")?;
    let mut features = BTreeSet::new();
    for _ in 0..count {
        features.insert(read_identifier(reader)?);
    }
    Ok(ConfigurationClientboundPacket::UpdateEnabledFeatures(
        features,
    ))
}

fn decode_tags(
    reader: &mut WireReader<'_>,
) -> Result<ConfigurationClientboundPacket, ConfigurationClientboundCodecError> {
    let registry_count = read_count(reader, "tag registries")?;
    let mut registries = Vec::new();
    for _ in 0..registry_count {
        let registry = read_identifier(reader)?;
        let tag_count = read_count(reader, "registry tags")?;
        let mut tags = Vec::new();
        for _ in 0..tag_count {
            let id = read_identifier(reader)?;
            let member_count = read_count(reader, "tag members")?;
            let mut members = Vec::new();
            for _ in 0..member_count {
                members.push(reader.read_var_i32()?);
            }
            tags.push(TagDefinition { id, members });
        }
        registries.push(RegistryTags { registry, tags });
    }
    Ok(ConfigurationClientboundPacket::UpdateTags(registries))
}

fn encode_tags(
    writer: &mut WireWriter,
    registries: &[RegistryTags],
) -> Result<(), ConfigurationClientboundCodecError> {
    write_count(writer, "tag registries", registries.len())?;
    for registry in registries {
        registry.registry.write(writer)?;
        write_count(writer, "registry tags", registry.tags.len())?;
        for tag in &registry.tags {
            tag.id.write(writer)?;
            write_count(writer, "tag members", tag.members.len())?;
            for member in &tag.members {
                writer.write_var_i32(*member)?;
            }
        }
    }
    Ok(())
}

fn decode_known_packs(
    reader: &mut WireReader<'_>,
) -> Result<ConfigurationClientboundPacket, ConfigurationClientboundCodecError> {
    let count = read_count(reader, "known packs")?;
    let mut packs = Vec::new();
    for _ in 0..count {
        packs.push(KnownPack {
            namespace: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
            id: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
            version: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
        });
    }
    Ok(ConfigurationClientboundPacket::SelectKnownPacks(packs))
}

fn read_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Identifier, ConfigurationClientboundCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

fn read_count(
    reader: &mut WireReader<'_>,
    field: &'static str,
) -> Result<usize, ConfigurationClientboundCodecError> {
    Ok(reader.read_count(field, reader.remaining())?)
}

fn write_count(
    writer: &mut WireWriter,
    field: &'static str,
    count: usize,
) -> Result<(), ConfigurationClientboundCodecError> {
    writer.write_count(field, count, MAX_INFLATED_PACKET_LENGTH)?;
    Ok(())
}
