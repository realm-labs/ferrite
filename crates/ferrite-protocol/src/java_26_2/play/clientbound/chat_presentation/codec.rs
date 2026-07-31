use thiserror::Error;

use crate::java_26_2::play::clientbound::chat_presentation::packet::{
    BoundChatType, ChatDecoration, ChatParameter, ChatTypeHolder, DeleteChat, DirectChatType,
    DisguisedChat, FilterMask, MESSAGE_SIGNATURE_BYTES, MessageSignature, PackedMessageSignature,
    PlayerChat, SignedMessageBodyPacked, SystemChat,
};
use crate::java_26_2::play::registry::{CHAT_TYPE, PlayRegistries, PlayRegistryError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const DEFAULT_UTF_LIMIT: usize = 32_767;
const MESSAGE_CONTENT_LIMIT: usize = 256;
const MAX_LAST_SEEN: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChatPresentationCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error("filter-mask ordinal {ordinal} is outside 0..=2")]
    UnknownFilterMask { ordinal: i32 },
}

pub fn read_delete(reader: &mut WireReader<'_>) -> Result<DeleteChat, ChatPresentationCodecError> {
    Ok(DeleteChat {
        signature: read_packed_signature(reader)?,
    })
}

pub fn write_delete(
    writer: &mut WireWriter,
    packet: &DeleteChat,
) -> Result<(), ChatPresentationCodecError> {
    write_packed_signature(writer, &packet.signature)
}

pub fn read_disguised(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<DisguisedChat, ChatPresentationCodecError> {
    Ok(DisguisedChat {
        message: read_component(reader)?,
        chat_type: read_bound_chat_type(reader, registries)?,
    })
}

pub fn write_disguised(
    writer: &mut WireWriter,
    packet: &DisguisedChat,
    registries: &PlayRegistries,
) -> Result<(), ChatPresentationCodecError> {
    packet.message.network_nbt().write(writer)?;
    write_bound_chat_type(writer, &packet.chat_type, registries)
}

pub fn read_player(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<PlayerChat, ChatPresentationCodecError> {
    let global_index = reader.read_var_i32()?;
    let sender = reader.read_u128()?;
    let message_index = reader.read_var_i32()?;
    let signature = reader
        .read_bool()?
        .then(|| read_signature(reader))
        .transpose()?;
    let body = read_body(reader)?;
    let unsigned_content = reader
        .read_bool()?
        .then(|| read_component(reader))
        .transpose()?;
    let filter_mask = read_filter_mask(reader)?;
    let chat_type = read_bound_chat_type(reader, registries)?;
    Ok(PlayerChat {
        global_index,
        sender,
        message_index,
        signature,
        body,
        unsigned_content,
        filter_mask,
        chat_type,
    })
}

pub fn write_player(
    writer: &mut WireWriter,
    packet: &PlayerChat,
    registries: &PlayRegistries,
) -> Result<(), ChatPresentationCodecError> {
    writer.write_var_i32(packet.global_index)?;
    writer.write_u128(packet.sender)?;
    writer.write_var_i32(packet.message_index)?;
    writer.write_bool(packet.signature.is_some())?;
    if let Some(signature) = &packet.signature {
        writer.write_bytes(signature.0.as_slice())?;
    }
    write_body(writer, &packet.body)?;
    writer.write_bool(packet.unsigned_content.is_some())?;
    if let Some(content) = &packet.unsigned_content {
        content.network_nbt().write(writer)?;
    }
    write_filter_mask(writer, &packet.filter_mask)?;
    write_bound_chat_type(writer, &packet.chat_type, registries)
}

pub fn read_system(reader: &mut WireReader<'_>) -> Result<SystemChat, ChatPresentationCodecError> {
    Ok(SystemChat {
        content: read_component(reader)?,
        overlay: reader.read_bool()?,
    })
}

pub fn write_system(
    writer: &mut WireWriter,
    packet: &SystemChat,
) -> Result<(), ChatPresentationCodecError> {
    packet.content.network_nbt().write(writer)?;
    writer.write_bool(packet.overlay)?;
    Ok(())
}

fn read_body(
    reader: &mut WireReader<'_>,
) -> Result<SignedMessageBodyPacked, ChatPresentationCodecError> {
    let content = reader.read_utf(MESSAGE_CONTENT_LIMIT)?.into_owned();
    let timestamp_ms = reader.read_i64()?;
    let salt = reader.read_i64()?;
    let count = reader.read_count("last-seen signatures", MAX_LAST_SEEN)?;
    let mut last_seen = Vec::with_capacity(count);
    for _ in 0..count {
        last_seen.push(read_packed_signature(reader)?);
    }
    Ok(SignedMessageBodyPacked {
        content,
        timestamp_ms,
        salt,
        last_seen,
    })
}

fn write_body(
    writer: &mut WireWriter,
    body: &SignedMessageBodyPacked,
) -> Result<(), ChatPresentationCodecError> {
    writer.write_utf(&body.content, MESSAGE_CONTENT_LIMIT)?;
    writer.write_i64(body.timestamp_ms)?;
    writer.write_i64(body.salt)?;
    writer.write_count("last-seen signatures", body.last_seen.len(), MAX_LAST_SEEN)?;
    for signature in &body.last_seen {
        write_packed_signature(writer, signature)?;
    }
    Ok(())
}

fn read_filter_mask(reader: &mut WireReader<'_>) -> Result<FilterMask, ChatPresentationCodecError> {
    match reader.read_var_i32()? {
        0 => Ok(FilterMask::Pass),
        1 => Ok(FilterMask::FullyFiltered),
        2 => {
            let count = reader.read_count("filter-mask words", reader.remaining() / 8)?;
            let mut words = Vec::with_capacity(count);
            for _ in 0..count {
                words.push(reader.read_i64()?);
            }
            Ok(FilterMask::PartiallyFiltered(words))
        }
        ordinal => Err(ChatPresentationCodecError::UnknownFilterMask { ordinal }),
    }
}

fn write_filter_mask(
    writer: &mut WireWriter,
    mask: &FilterMask,
) -> Result<(), ChatPresentationCodecError> {
    match mask {
        FilterMask::Pass => writer.write_var_i32(0)?,
        FilterMask::FullyFiltered => writer.write_var_i32(1)?,
        FilterMask::PartiallyFiltered(words) => {
            writer.write_var_i32(2)?;
            writer.write_count(
                "filter-mask words",
                words.len(),
                MAX_INFLATED_PACKET_LENGTH / 8,
            )?;
            for word in words {
                writer.write_i64(*word)?;
            }
        }
    }
    Ok(())
}

fn read_bound_chat_type(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<BoundChatType, ChatPresentationCodecError> {
    let holder = match reader.read_var_i32()? {
        0 => ChatTypeHolder::Direct(Box::new(DirectChatType {
            chat: read_decoration(reader)?,
            narration: read_decoration(reader)?,
        })),
        encoded => {
            ChatTypeHolder::Registered(registries.resolve(CHAT_TYPE, encoded.wrapping_sub(1))?)
        }
    };
    let name = read_component(reader)?;
    let target = reader
        .read_bool()?
        .then(|| read_component(reader))
        .transpose()?;
    Ok(BoundChatType {
        holder,
        name,
        target,
    })
}

fn write_bound_chat_type(
    writer: &mut WireWriter,
    bound: &BoundChatType,
    registries: &PlayRegistries,
) -> Result<(), ChatPresentationCodecError> {
    match &bound.holder {
        ChatTypeHolder::Direct(chat_type) => {
            writer.write_var_i32(0)?;
            write_decoration(writer, &chat_type.chat)?;
            write_decoration(writer, &chat_type.narration)?;
        }
        ChatTypeHolder::Registered(identity) => {
            writer.write_var_i32(registries.raw_id(CHAT_TYPE, identity)? + 1)?;
        }
    }
    bound.name.network_nbt().write(writer)?;
    writer.write_bool(bound.target.is_some())?;
    if let Some(target) = &bound.target {
        target.network_nbt().write(writer)?;
    }
    Ok(())
}

fn read_decoration(
    reader: &mut WireReader<'_>,
) -> Result<ChatDecoration, ChatPresentationCodecError> {
    let translation_key = reader.read_utf(DEFAULT_UTF_LIMIT)?.into_owned();
    let count = reader.read_count("chat decoration parameters", reader.remaining())?;
    let mut parameters = Vec::with_capacity(count);
    for _ in 0..count {
        parameters.push(match reader.read_var_i32()? {
            1 => ChatParameter::Target,
            2 => ChatParameter::Content,
            _ => ChatParameter::Sender,
        });
    }
    let style = NetworkNbt::read(reader, NbtQuota::Trusted)?;
    Ok(ChatDecoration {
        translation_key,
        parameters,
        style,
    })
}

fn write_decoration(
    writer: &mut WireWriter,
    decoration: &ChatDecoration,
) -> Result<(), ChatPresentationCodecError> {
    writer.write_utf(&decoration.translation_key, DEFAULT_UTF_LIMIT)?;
    writer.write_count(
        "chat decoration parameters",
        decoration.parameters.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for parameter in &decoration.parameters {
        writer.write_var_i32(match parameter {
            ChatParameter::Sender => 0,
            ChatParameter::Target => 1,
            ChatParameter::Content => 2,
        })?;
    }
    decoration.style.write(writer)?;
    Ok(())
}

fn read_packed_signature(
    reader: &mut WireReader<'_>,
) -> Result<PackedMessageSignature, ChatPresentationCodecError> {
    Ok(match reader.read_var_i32()? {
        0 => PackedMessageSignature::Full(read_signature(reader)?),
        encoded => PackedMessageSignature::CacheIndex(encoded.wrapping_sub(1)),
    })
}

fn write_packed_signature(
    writer: &mut WireWriter,
    signature: &PackedMessageSignature,
) -> Result<(), ChatPresentationCodecError> {
    match signature {
        PackedMessageSignature::Full(signature) => {
            writer.write_var_i32(0)?;
            writer.write_bytes(signature.0.as_slice())?;
        }
        PackedMessageSignature::CacheIndex(index) => {
            writer.write_var_i32(index.wrapping_add(1))?;
        }
    }
    Ok(())
}

fn read_signature(reader: &mut WireReader<'_>) -> Result<MessageSignature, WireError> {
    let bytes = reader.read_bytes(MESSAGE_SIGNATURE_BYTES, "message signature")?;
    Ok(MessageSignature(Box::new(bytes.try_into().expect(
        "fixed-width signature read has the requested length",
    ))))
}

fn read_component(
    reader: &mut WireReader<'_>,
) -> Result<TextComponentNbt, ChatPresentationCodecError> {
    Ok(TextComponentNbt::from_network_nbt(NetworkNbt::read(
        reader,
        NbtQuota::Trusted,
    )?)?)
}
