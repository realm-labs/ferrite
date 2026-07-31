use rsa::RsaPublicKey;
use rsa::pkcs8::DecodePublicKey;
use thiserror::Error;

use crate::java_26_2::play::clientbound::chat_presentation::packet::{
    MESSAGE_SIGNATURE_BYTES, MessageSignature,
};
use crate::java_26_2::play::serverbound::chat::packet::{
    ArgumentSignature, ChatAck, ChatCommand, ChatCommandSigned, ChatMessage, ChatSessionUpdate,
    CommandSuggestion, LastSeenUpdate, ProfilePublicKeyData,
};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const DEFAULT_UTF_LIMIT: usize = 32_767;
const CHAT_UTF_LIMIT: usize = 256;
const SUGGESTION_UTF_LIMIT: usize = 32_500;
const ARGUMENT_NAME_UTF_LIMIT: usize = 16;
const ARGUMENT_SIGNATURE_LIMIT: usize = 8;
const PUBLIC_KEY_LIMIT: usize = 512;
const KEY_SIGNATURE_LIMIT: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChatCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("profile public key is not a valid X.509 RSA public key")]
    InvalidPublicKey,
}

pub(crate) fn decode_ack(reader: &mut WireReader<'_>) -> Result<ChatAck, WireError> {
    Ok(ChatAck {
        offset: reader.read_var_i32()?,
    })
}

pub(crate) fn encode_ack(writer: &mut WireWriter, packet: ChatAck) -> Result<(), WireError> {
    writer.write_var_i32(packet.offset)
}

pub(crate) fn decode_command(reader: &mut WireReader<'_>) -> Result<ChatCommand, WireError> {
    Ok(ChatCommand {
        command: reader.read_utf(DEFAULT_UTF_LIMIT)?.into_owned(),
    })
}

pub(crate) fn encode_command(
    writer: &mut WireWriter,
    packet: &ChatCommand,
) -> Result<(), WireError> {
    writer.write_utf(&packet.command, DEFAULT_UTF_LIMIT)
}

pub(crate) fn decode_signed_command(
    reader: &mut WireReader<'_>,
) -> Result<ChatCommandSigned, WireError> {
    let command = reader.read_utf(DEFAULT_UTF_LIMIT)?.into_owned();
    let timestamp_millis = reader.read_i64()?;
    let salt = reader.read_i64()?;
    let count = reader.read_count("argument signatures", ARGUMENT_SIGNATURE_LIMIT)?;
    let mut argument_signatures = Vec::with_capacity(count);
    for _ in 0..count {
        argument_signatures.push(ArgumentSignature {
            name: reader.read_utf(ARGUMENT_NAME_UTF_LIMIT)?.into_owned(),
            signature: read_signature(reader)?,
        });
    }
    Ok(ChatCommandSigned {
        command,
        timestamp_millis,
        salt,
        argument_signatures,
        last_seen: read_last_seen(reader)?,
    })
}

pub(crate) fn encode_signed_command(
    writer: &mut WireWriter,
    packet: &ChatCommandSigned,
) -> Result<(), WireError> {
    writer.write_utf(&packet.command, DEFAULT_UTF_LIMIT)?;
    writer.write_i64(packet.timestamp_millis)?;
    writer.write_i64(packet.salt)?;
    writer.write_count(
        "argument signatures",
        packet.argument_signatures.len(),
        ARGUMENT_SIGNATURE_LIMIT,
    )?;
    for argument in &packet.argument_signatures {
        writer.write_utf(&argument.name, ARGUMENT_NAME_UTF_LIMIT)?;
        write_signature(writer, &argument.signature)?;
    }
    write_last_seen(writer, packet.last_seen)
}

pub(crate) fn decode_chat(reader: &mut WireReader<'_>) -> Result<ChatMessage, WireError> {
    let message = reader.read_utf(CHAT_UTF_LIMIT)?.into_owned();
    let timestamp_millis = reader.read_i64()?;
    let salt = reader.read_i64()?;
    let signature = reader
        .read_bool()?
        .then(|| read_signature(reader))
        .transpose()?;
    Ok(ChatMessage {
        message,
        timestamp_millis,
        salt,
        signature,
        last_seen: read_last_seen(reader)?,
    })
}

pub(crate) fn encode_chat(writer: &mut WireWriter, packet: &ChatMessage) -> Result<(), WireError> {
    writer.write_utf(&packet.message, CHAT_UTF_LIMIT)?;
    writer.write_i64(packet.timestamp_millis)?;
    writer.write_i64(packet.salt)?;
    writer.write_bool(packet.signature.is_some())?;
    if let Some(signature) = &packet.signature {
        write_signature(writer, signature)?;
    }
    write_last_seen(writer, packet.last_seen)
}

pub(crate) fn decode_session(
    reader: &mut WireReader<'_>,
) -> Result<ChatSessionUpdate, ChatCodecError> {
    let packet = ChatSessionUpdate {
        session_id: reader.read_u128()?,
        profile_key: ProfilePublicKeyData {
            expires_at_millis: reader.read_i64()?,
            public_key: reader.read_byte_array(PUBLIC_KEY_LIMIT)?.to_vec(),
            key_signature: reader.read_byte_array(KEY_SIGNATURE_LIMIT)?.to_vec(),
        },
    };
    validate_public_key(&packet.profile_key.public_key)?;
    Ok(packet)
}

pub(crate) fn encode_session(
    writer: &mut WireWriter,
    packet: &ChatSessionUpdate,
) -> Result<(), ChatCodecError> {
    validate_public_key(&packet.profile_key.public_key)?;
    writer.write_u128(packet.session_id)?;
    writer.write_i64(packet.profile_key.expires_at_millis)?;
    writer.write_byte_array(&packet.profile_key.public_key, PUBLIC_KEY_LIMIT)?;
    writer.write_byte_array(&packet.profile_key.key_signature, KEY_SIGNATURE_LIMIT)?;
    Ok(())
}

fn validate_public_key(encoded: &[u8]) -> Result<(), ChatCodecError> {
    RsaPublicKey::from_public_key_der(encoded)
        .map(|_| ())
        .map_err(|_| ChatCodecError::InvalidPublicKey)
}

pub(crate) fn decode_suggestion(
    reader: &mut WireReader<'_>,
) -> Result<CommandSuggestion, WireError> {
    Ok(CommandSuggestion {
        transaction_id: reader.read_var_i32()?,
        input: reader.read_utf(SUGGESTION_UTF_LIMIT)?.into_owned(),
    })
}

pub(crate) fn encode_suggestion(
    writer: &mut WireWriter,
    packet: &CommandSuggestion,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.transaction_id)?;
    writer.write_utf(&packet.input, SUGGESTION_UTF_LIMIT)
}

fn read_last_seen(reader: &mut WireReader<'_>) -> Result<LastSeenUpdate, WireError> {
    Ok(LastSeenUpdate {
        offset: reader.read_var_i32()?,
        acknowledged: [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?],
        checksum: reader.read_i8()?,
    })
}

fn write_last_seen(writer: &mut WireWriter, update: LastSeenUpdate) -> Result<(), WireError> {
    writer.write_var_i32(update.offset)?;
    writer.write_bytes(&update.acknowledged)?;
    writer.write_i8(update.checksum)
}

fn read_signature(reader: &mut WireReader<'_>) -> Result<MessageSignature, WireError> {
    let bytes = reader.read_bytes(MESSAGE_SIGNATURE_BYTES, "message signature")?;
    Ok(MessageSignature(Box::new(bytes.try_into().expect(
        "fixed-length signature slice converts to an array",
    ))))
}

fn write_signature(writer: &mut WireWriter, signature: &MessageSignature) -> Result<(), WireError> {
    writer.write_bytes(signature.0.as_ref())
}
