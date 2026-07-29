use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::configuration::serverbound::packet::{
    ChatVisibility, ClientInformation, ConfigurationServerboundPacket, CustomPayload, MainHand,
    ParticleStatus,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::known_pack::KnownPack;
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_LANGUAGE_CODE_UNITS: usize = 16;
const MAX_UTF_CODE_UNITS: usize = 32_767;
const MAX_UNKNOWN_SERVERBOUND_PAYLOAD: usize = 32_767;
const MAX_KNOWN_PACKS: usize = 64;
const BRAND_CHANNEL: &str = "minecraft:brand";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigurationServerboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidIdentifier(#[from] IdentifierError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("configuration serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("configuration serverbound packet {identity} is not part of the required C1 family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
    #[error("{kind} enum ordinal {ordinal} is invalid")]
    InvalidEnum { kind: &'static str, ordinal: i32 },
    #[error("discarded custom payload {channel} cannot be re-encoded")]
    CannotEncodeDiscardedPayload { channel: Identifier },
}

pub fn decode_packet(
    body: &[u8],
) -> Result<ConfigurationServerboundPacket, ConfigurationServerboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Configuration,
        PacketDirection::Serverbound,
        wire_id,
    )
    .ok_or(ConfigurationServerboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:client_information" => ConfigurationServerboundPacket::ClientInformation(
            decode_client_information_body(&mut reader)?,
        ),
        "minecraft:custom_payload" => decode_custom_payload(&mut reader)?,
        "minecraft:finish_configuration" => ConfigurationServerboundPacket::FinishConfiguration,
        "minecraft:keep_alive" => ConfigurationServerboundPacket::KeepAlive(reader.read_i64()?),
        "minecraft:pong" => ConfigurationServerboundPacket::Pong(reader.read_i32()?),
        "minecraft:select_known_packs" => decode_known_packs(&mut reader)?,
        identity => {
            return Err(ConfigurationServerboundCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &ConfigurationServerboundPacket,
) -> Result<Vec<u8>, ConfigurationServerboundCodecError> {
    let identity = packet_identity(packet);
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Configuration,
        PacketDirection::Serverbound,
        identity,
    )
    .ok_or(ConfigurationServerboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        ConfigurationServerboundPacket::ClientInformation(information) => {
            encode_client_information_body(&mut writer, information)?;
        }
        ConfigurationServerboundPacket::CustomPayload(payload) => {
            encode_custom_payload(&mut writer, payload)?;
        }
        ConfigurationServerboundPacket::FinishConfiguration => {}
        ConfigurationServerboundPacket::KeepAlive(token) => writer.write_i64(*token)?,
        ConfigurationServerboundPacket::Pong(token) => writer.write_i32(*token)?,
        ConfigurationServerboundPacket::SelectKnownPacks(packs) => {
            writer.write_count("known packs", packs.len(), MAX_KNOWN_PACKS)?;
            for pack in packs {
                writer.write_utf(&pack.namespace, MAX_UTF_CODE_UNITS)?;
                writer.write_utf(&pack.id, MAX_UTF_CODE_UNITS)?;
                writer.write_utf(&pack.version, MAX_UTF_CODE_UNITS)?;
            }
        }
    }
    Ok(writer.into_inner())
}

fn packet_identity(packet: &ConfigurationServerboundPacket) -> &'static str {
    match packet {
        ConfigurationServerboundPacket::ClientInformation(_) => "minecraft:client_information",
        ConfigurationServerboundPacket::CustomPayload(_) => "minecraft:custom_payload",
        ConfigurationServerboundPacket::FinishConfiguration => "minecraft:finish_configuration",
        ConfigurationServerboundPacket::KeepAlive(_) => "minecraft:keep_alive",
        ConfigurationServerboundPacket::Pong(_) => "minecraft:pong",
        ConfigurationServerboundPacket::SelectKnownPacks(_) => "minecraft:select_known_packs",
    }
}

pub(crate) fn decode_client_information_body(
    reader: &mut WireReader<'_>,
) -> Result<ClientInformation, ConfigurationServerboundCodecError> {
    let language = reader.read_utf(MAX_LANGUAGE_CODE_UNITS)?.into_owned();
    let view_distance = reader.read_i8()?;
    let chat_ordinal = reader.read_var_i32()?;
    let chat_visibility = ChatVisibility::from_ordinal(chat_ordinal).ok_or(
        ConfigurationServerboundCodecError::InvalidEnum {
            kind: "chat visibility",
            ordinal: chat_ordinal,
        },
    )?;
    let chat_colors = reader.read_bool()?;
    let model_customization = reader.read_u8()?;
    let hand_ordinal = reader.read_var_i32()?;
    let main_hand = MainHand::from_ordinal(hand_ordinal).ok_or(
        ConfigurationServerboundCodecError::InvalidEnum {
            kind: "main hand",
            ordinal: hand_ordinal,
        },
    )?;
    let text_filtering = reader.read_bool()?;
    let allows_listing = reader.read_bool()?;
    let particle_ordinal = reader.read_var_i32()?;
    let particle_status = ParticleStatus::from_ordinal(particle_ordinal).ok_or(
        ConfigurationServerboundCodecError::InvalidEnum {
            kind: "particle status",
            ordinal: particle_ordinal,
        },
    )?;
    Ok(ClientInformation {
        language,
        view_distance,
        chat_visibility,
        chat_colors,
        model_customization,
        main_hand,
        text_filtering,
        allows_listing,
        particle_status,
    })
}

pub(crate) fn encode_client_information_body(
    writer: &mut WireWriter,
    information: &ClientInformation,
) -> Result<(), ConfigurationServerboundCodecError> {
    writer.write_utf(&information.language, MAX_LANGUAGE_CODE_UNITS)?;
    writer.write_i8(information.view_distance)?;
    writer.write_var_i32(information.chat_visibility.ordinal())?;
    writer.write_bool(information.chat_colors)?;
    writer.write_u8(information.model_customization)?;
    writer.write_var_i32(information.main_hand.ordinal())?;
    writer.write_bool(information.text_filtering)?;
    writer.write_bool(information.allows_listing)?;
    writer.write_var_i32(information.particle_status.ordinal())?;
    Ok(())
}

fn decode_custom_payload(
    reader: &mut WireReader<'_>,
) -> Result<ConfigurationServerboundPacket, ConfigurationServerboundCodecError> {
    let channel = read_identifier(reader)?;
    if channel.to_string() == BRAND_CHANNEL {
        Ok(ConfigurationServerboundPacket::CustomPayload(
            CustomPayload::Brand(reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned()),
        ))
    } else {
        let length = reader
            .read_bounded_remaining(
                "unknown serverbound custom payload",
                MAX_UNKNOWN_SERVERBOUND_PAYLOAD,
            )?
            .len();
        Ok(ConfigurationServerboundPacket::CustomPayload(
            CustomPayload::Discarded { channel, length },
        ))
    }
}

fn encode_custom_payload(
    writer: &mut WireWriter,
    payload: &CustomPayload,
) -> Result<(), ConfigurationServerboundCodecError> {
    match payload {
        CustomPayload::Brand(brand) => {
            Identifier::parse(BRAND_CHANNEL)?.write(writer)?;
            writer.write_utf(brand, MAX_UTF_CODE_UNITS)?;
            Ok(())
        }
        CustomPayload::Discarded { channel, .. } => Err(
            ConfigurationServerboundCodecError::CannotEncodeDiscardedPayload {
                channel: channel.clone(),
            },
        ),
    }
}

fn decode_known_packs(
    reader: &mut WireReader<'_>,
) -> Result<ConfigurationServerboundPacket, ConfigurationServerboundCodecError> {
    let count = reader.read_count("known packs", MAX_KNOWN_PACKS)?;
    let mut packs = Vec::new();
    for _ in 0..count {
        packs.push(KnownPack {
            namespace: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
            id: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
            version: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
        });
    }
    Ok(ConfigurationServerboundPacket::SelectKnownPacks(packs))
}

fn read_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Identifier, ConfigurationServerboundCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
