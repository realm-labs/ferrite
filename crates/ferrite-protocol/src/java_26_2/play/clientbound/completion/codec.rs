use thiserror::Error;

use crate::java_26_2::play::clientbound::completion::packet::{
    CommandSuggestions, CustomChatCompletions, CustomCompletionAction, SuggestionEntry,
};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const DEFAULT_UTF_LIMIT: usize = 32_767;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CompletionCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error("custom completion action ordinal {ordinal} is outside 0..=2")]
    UnknownAction { ordinal: i32 },
}

pub fn read_command(
    reader: &mut WireReader<'_>,
) -> Result<CommandSuggestions, CompletionCodecError> {
    let transaction = reader.read_var_i32()?;
    let start = reader.read_var_i32()?;
    let length = reader.read_var_i32()?;
    let count = reader.read_count("command suggestions", reader.remaining())?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let text = reader.read_utf(DEFAULT_UTF_LIMIT)?.into_owned();
        let tooltip = reader
            .read_bool()?
            .then(|| read_tooltip(reader))
            .transpose()?;
        entries.push(SuggestionEntry { text, tooltip });
    }
    Ok(CommandSuggestions {
        transaction,
        start,
        length,
        entries,
    })
}

pub fn write_command(
    writer: &mut WireWriter,
    packet: &CommandSuggestions,
) -> Result<(), CompletionCodecError> {
    writer.write_var_i32(packet.transaction)?;
    writer.write_var_i32(packet.start)?;
    writer.write_var_i32(packet.length)?;
    writer.write_count(
        "command suggestions",
        packet.entries.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for entry in &packet.entries {
        writer.write_utf(&entry.text, DEFAULT_UTF_LIMIT)?;
        writer.write_bool(entry.tooltip.is_some())?;
        if let Some(tooltip) = &entry.tooltip {
            tooltip.network_nbt().write(writer)?;
        }
    }
    Ok(())
}

pub fn read_custom(
    reader: &mut WireReader<'_>,
) -> Result<CustomChatCompletions, CompletionCodecError> {
    let ordinal = reader.read_var_i32()?;
    let action = match ordinal {
        0 => CustomCompletionAction::Add,
        1 => CustomCompletionAction::Remove,
        2 => CustomCompletionAction::Set,
        _ => return Err(CompletionCodecError::UnknownAction { ordinal }),
    };
    let count = reader.read_count("custom chat completions", reader.remaining())?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(reader.read_utf(DEFAULT_UTF_LIMIT)?.into_owned());
    }
    Ok(CustomChatCompletions { action, entries })
}

pub fn write_custom(
    writer: &mut WireWriter,
    packet: &CustomChatCompletions,
) -> Result<(), CompletionCodecError> {
    writer.write_var_i32(packet.action.ordinal())?;
    writer.write_count(
        "custom chat completions",
        packet.entries.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for entry in &packet.entries {
        writer.write_utf(entry, DEFAULT_UTF_LIMIT)?;
    }
    Ok(())
}

fn read_tooltip(reader: &mut WireReader<'_>) -> Result<TextComponentNbt, CompletionCodecError> {
    Ok(TextComponentNbt::from_network_nbt(NetworkNbt::read(
        reader,
        NbtQuota::Trusted,
    )?)?)
}
