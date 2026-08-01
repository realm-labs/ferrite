use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::play::clientbound::common_services::packet::{
    CommonCustomPayload, CommonServicePacket, DialogHolder, ResourcePackPush, ServerLink,
    ServerLinkLabel,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_UTF_CODE_UNITS: usize = 32_767;
const MAX_HASH_CODE_UNITS: usize = 40;
const MAX_COOKIE_BYTES: usize = 5_120;
const MAX_CUSTOM_PAYLOAD: usize = 1_048_576;
const MAX_REPORT_DETAILS: usize = 32;
const MAX_REPORT_KEY_CODE_UNITS: usize = 128;
const MAX_REPORT_VALUE_CODE_UNITS: usize = 4_096;
const BRAND_CHANNEL: &str = "minecraft:brand";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommonServicesCodecError {
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
    #[error("play clientbound packet {identity} is not a common service")]
    OtherPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing common-service identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
    #[error("registered dialog raw ID {raw_id} is outside 0..{registry_len}")]
    UnknownDialog { raw_id: i32, registry_len: usize },
    #[error("dialog holder encoded invalid signed value {encoded}")]
    InvalidDialogHolder { encoded: i32 },
}

pub fn decode_packet(
    body: &[u8],
    dialog_registry_len: usize,
) -> Result<CommonServicePacket, CommonServicesCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor =
        PacketCatalog::by_wire_id(ConnectionState::Play, PacketDirection::Clientbound, wire_id)
            .ok_or(CommonServicesCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:cookie_request" => CommonServicePacket::CookieRequest {
            key: read_identifier(&mut reader)?,
        },
        "minecraft:custom_payload" => decode_custom_payload(&mut reader)?,
        "minecraft:pong_response" => CommonServicePacket::PongResponse {
            token: reader.read_i64()?,
        },
        "minecraft:resource_pack_pop" => CommonServicePacket::ResourcePackPop {
            pack_id: reader
                .read_bool()?
                .then(|| reader.read_u128())
                .transpose()?,
        },
        "minecraft:resource_pack_push" => decode_resource_pack_push(&mut reader)?,
        "minecraft:store_cookie" => CommonServicePacket::StoreCookie {
            key: read_identifier(&mut reader)?,
            value: reader.read_byte_array(MAX_COOKIE_BYTES)?.to_vec(),
        },
        "minecraft:transfer" => CommonServicePacket::Transfer {
            host: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
            port: reader.read_var_i32()?,
        },
        "minecraft:custom_report_details" => decode_report_details(&mut reader)?,
        "minecraft:server_links" => decode_server_links(&mut reader)?,
        "minecraft:clear_dialog" => CommonServicePacket::ClearDialog,
        "minecraft:show_dialog" => CommonServicePacket::ShowDialog {
            dialog: decode_dialog_holder(&mut reader, dialog_registry_len)?,
        },
        identity => return Err(CommonServicesCodecError::OtherPacketIdentity { identity }),
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &CommonServicePacket,
    dialog_registry_len: usize,
) -> Result<Vec<u8>, CommonServicesCodecError> {
    let identity = packet.kind().identity();
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Play,
        PacketDirection::Clientbound,
        identity,
    )
    .ok_or(CommonServicesCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        CommonServicePacket::CookieRequest { key } => key.write(&mut writer)?,
        CommonServicePacket::CustomPayload(payload) => encode_custom_payload(&mut writer, payload)?,
        CommonServicePacket::PongResponse { token } => writer.write_i64(*token)?,
        CommonServicePacket::ResourcePackPop { pack_id } => {
            writer.write_bool(pack_id.is_some())?;
            if let Some(pack_id) = pack_id {
                writer.write_u128(*pack_id)?;
            }
        }
        CommonServicePacket::ResourcePackPush(push) => {
            encode_resource_pack_push(&mut writer, push)?
        }
        CommonServicePacket::StoreCookie { key, value } => {
            key.write(&mut writer)?;
            writer.write_byte_array(value, MAX_COOKIE_BYTES)?;
        }
        CommonServicePacket::Transfer { host, port } => {
            writer.write_utf(host, MAX_UTF_CODE_UNITS)?;
            writer.write_var_i32(*port)?;
        }
        CommonServicePacket::CustomReportDetails(details) => {
            writer.write_count("custom report details", details.len(), MAX_REPORT_DETAILS)?;
            for (key, value) in details {
                writer.write_utf(key, MAX_REPORT_KEY_CODE_UNITS)?;
                writer.write_utf(value, MAX_REPORT_VALUE_CODE_UNITS)?;
            }
        }
        CommonServicePacket::ServerLinks(links) => {
            writer.write_count("server links", links.len(), MAX_INFLATED_PACKET_LENGTH)?;
            for link in links {
                encode_server_link(&mut writer, link)?;
            }
        }
        CommonServicePacket::ClearDialog => {}
        CommonServicePacket::ShowDialog { dialog } => {
            encode_dialog_holder(&mut writer, dialog, dialog_registry_len)?;
        }
    }
    Ok(writer.into_inner())
}

fn decode_custom_payload(
    reader: &mut WireReader<'_>,
) -> Result<CommonServicePacket, CommonServicesCodecError> {
    let channel = read_identifier(reader)?;
    if channel.to_string() == BRAND_CHANNEL {
        Ok(CommonServicePacket::CustomPayload(
            CommonCustomPayload::Brand(reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned()),
        ))
    } else {
        Ok(CommonServicePacket::CustomPayload(
            CommonCustomPayload::Discarded {
                channel,
                payload: reader
                    .read_bounded_remaining("play custom payload", MAX_CUSTOM_PAYLOAD)?
                    .to_vec(),
            },
        ))
    }
}

fn encode_custom_payload(
    writer: &mut WireWriter,
    payload: &CommonCustomPayload,
) -> Result<(), CommonServicesCodecError> {
    match payload {
        CommonCustomPayload::Brand(brand) => {
            Identifier::parse(BRAND_CHANNEL)?.write(writer)?;
            writer.write_utf(brand, MAX_UTF_CODE_UNITS)?;
        }
        CommonCustomPayload::Discarded { channel, payload } => {
            channel.write(writer)?;
            if payload.len() > MAX_CUSTOM_PAYLOAD {
                return Err(WireError::LengthLimit {
                    field: "play custom payload",
                    length: payload.len(),
                    maximum: MAX_CUSTOM_PAYLOAD,
                }
                .into());
            }
            writer.write_bytes(payload)?;
        }
    }
    Ok(())
}

fn decode_resource_pack_push(
    reader: &mut WireReader<'_>,
) -> Result<CommonServicePacket, CommonServicesCodecError> {
    Ok(CommonServicePacket::ResourcePackPush(ResourcePackPush {
        pack_id: reader.read_u128()?,
        url: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
        hash: reader.read_utf(MAX_HASH_CODE_UNITS)?.into_owned(),
        required: reader.read_bool()?,
        prompt: if reader.read_bool()? {
            Some(TextComponentNbt::from_network_nbt(NetworkNbt::read(
                reader,
                NbtQuota::Trusted,
            )?)?)
        } else {
            None
        },
    }))
}

fn encode_resource_pack_push(
    writer: &mut WireWriter,
    push: &ResourcePackPush,
) -> Result<(), CommonServicesCodecError> {
    writer.write_u128(push.pack_id)?;
    writer.write_utf(&push.url, MAX_UTF_CODE_UNITS)?;
    writer.write_utf(&push.hash, MAX_HASH_CODE_UNITS)?;
    writer.write_bool(push.required)?;
    writer.write_bool(push.prompt.is_some())?;
    if let Some(prompt) = &push.prompt {
        prompt.network_nbt().write(writer)?;
    }
    Ok(())
}

fn decode_report_details(
    reader: &mut WireReader<'_>,
) -> Result<CommonServicePacket, CommonServicesCodecError> {
    let count = reader.read_count("custom report details", MAX_REPORT_DETAILS)?;
    let mut details = BTreeMap::new();
    for _ in 0..count {
        details.insert(
            reader.read_utf(MAX_REPORT_KEY_CODE_UNITS)?.into_owned(),
            reader.read_utf(MAX_REPORT_VALUE_CODE_UNITS)?.into_owned(),
        );
    }
    Ok(CommonServicePacket::CustomReportDetails(details))
}

fn decode_server_links(
    reader: &mut WireReader<'_>,
) -> Result<CommonServicePacket, CommonServicesCodecError> {
    let count = reader.read_count("server links", MAX_INFLATED_PACKET_LENGTH)?;
    let mut links = Vec::with_capacity(count);
    for _ in 0..count {
        let label = if reader.read_bool()? {
            ServerLinkLabel::Known(reader.read_var_i32()?)
        } else {
            ServerLinkLabel::Custom(TextComponentNbt::from_network_nbt(NetworkNbt::read(
                reader,
                NbtQuota::Trusted,
            )?)?)
        };
        links.push(ServerLink {
            label,
            url: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
        });
    }
    Ok(CommonServicePacket::ServerLinks(links))
}

fn encode_server_link(
    writer: &mut WireWriter,
    link: &ServerLink,
) -> Result<(), CommonServicesCodecError> {
    writer.write_bool(matches!(link.label, ServerLinkLabel::Known(_)))?;
    match &link.label {
        ServerLinkLabel::Known(raw_id) => writer.write_var_i32(*raw_id)?,
        ServerLinkLabel::Custom(label) => label.network_nbt().write(writer)?,
    }
    writer.write_utf(&link.url, MAX_UTF_CODE_UNITS)?;
    Ok(())
}

fn decode_dialog_holder(
    reader: &mut WireReader<'_>,
    registry_len: usize,
) -> Result<DialogHolder, CommonServicesCodecError> {
    let encoded = reader.read_var_i32()?;
    if encoded == 0 {
        return Ok(DialogHolder::Direct(NetworkNbt::read(
            reader,
            NbtQuota::Trusted,
        )?));
    }
    let raw_id = encoded
        .checked_sub(1)
        .ok_or(CommonServicesCodecError::InvalidDialogHolder { encoded })?;
    let index = usize::try_from(raw_id)
        .map_err(|_| CommonServicesCodecError::InvalidDialogHolder { encoded })?;
    if index >= registry_len {
        return Err(CommonServicesCodecError::UnknownDialog {
            raw_id,
            registry_len,
        });
    }
    Ok(DialogHolder::Registered(raw_id))
}

fn encode_dialog_holder(
    writer: &mut WireWriter,
    holder: &DialogHolder,
    registry_len: usize,
) -> Result<(), CommonServicesCodecError> {
    match holder {
        DialogHolder::Direct(dialog) => {
            writer.write_var_i32(0)?;
            dialog.write(writer)?;
        }
        DialogHolder::Registered(raw_id) => {
            let index = usize::try_from(*raw_id)
                .map_err(|_| CommonServicesCodecError::InvalidDialogHolder { encoded: *raw_id })?;
            if index >= registry_len {
                return Err(CommonServicesCodecError::UnknownDialog {
                    raw_id: *raw_id,
                    registry_len,
                });
            }
            let encoded = raw_id
                .checked_add(1)
                .ok_or(CommonServicesCodecError::InvalidDialogHolder { encoded: *raw_id })?;
            writer.write_var_i32(encoded)?;
        }
    }
    Ok(())
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, CommonServicesCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
