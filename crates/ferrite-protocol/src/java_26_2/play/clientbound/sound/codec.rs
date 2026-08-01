use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_effects::codec::{
    EntityEffectsCodecError, read_sound, write_sound,
};
use crate::java_26_2::play::clientbound::sound::packet::{
    SoundAtEntity, SoundAtPosition, SoundSource, StopSound,
};
use crate::java_26_2::play::registry::PlayRegistries;
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SoundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    Holder(#[from] EntityEffectsCodecError),
    #[error("sound source raw ID {raw_id} is outside 0..=10")]
    InvalidSource { raw_id: i32 },
}

pub(crate) fn read_position(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<SoundAtPosition, SoundCodecError> {
    Ok(SoundAtPosition {
        sound: read_sound(reader, registries)?,
        source: read_source(reader)?,
        encoded_position: [reader.read_i32()?, reader.read_i32()?, reader.read_i32()?],
        volume: reader.read_f32()?,
        pitch: reader.read_f32()?,
        seed: reader.read_i64()?,
    })
}

pub(crate) fn write_position(
    writer: &mut WireWriter,
    packet: &SoundAtPosition,
    registries: &PlayRegistries,
) -> Result<(), SoundCodecError> {
    write_sound(writer, &packet.sound, registries)?;
    write_source(writer, packet.source)?;
    for coordinate in packet.encoded_position {
        writer.write_i32(coordinate)?;
    }
    writer.write_f32(packet.volume)?;
    writer.write_f32(packet.pitch)?;
    writer.write_i64(packet.seed)?;
    Ok(())
}

pub(crate) fn read_entity(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<SoundAtEntity, SoundCodecError> {
    Ok(SoundAtEntity {
        sound: read_sound(reader, registries)?,
        source: read_source(reader)?,
        entity_id: reader.read_var_i32()?,
        volume: reader.read_f32()?,
        pitch: reader.read_f32()?,
        seed: reader.read_i64()?,
    })
}

pub(crate) fn write_entity(
    writer: &mut WireWriter,
    packet: &SoundAtEntity,
    registries: &PlayRegistries,
) -> Result<(), SoundCodecError> {
    write_sound(writer, &packet.sound, registries)?;
    write_source(writer, packet.source)?;
    writer.write_var_i32(packet.entity_id)?;
    writer.write_f32(packet.volume)?;
    writer.write_f32(packet.pitch)?;
    writer.write_i64(packet.seed)?;
    Ok(())
}

pub(crate) fn read_stop(reader: &mut WireReader<'_>) -> Result<StopSound, SoundCodecError> {
    let flags = reader.read_u8()?;
    Ok(StopSound {
        source: (flags & 1 != 0).then(|| read_source(reader)).transpose()?,
        sound: (flags & 2 != 0)
            .then(|| read_identifier(reader))
            .transpose()?,
    })
}

pub(crate) fn write_stop(
    writer: &mut WireWriter,
    packet: &StopSound,
) -> Result<(), SoundCodecError> {
    let flags = u8::from(packet.source.is_some()) | (u8::from(packet.sound.is_some()) << 1);
    writer.write_u8(flags)?;
    if let Some(source) = packet.source {
        write_source(writer, source)?;
    }
    if let Some(sound) = &packet.sound {
        sound.write(writer)?;
    }
    Ok(())
}

fn read_source(reader: &mut WireReader<'_>) -> Result<SoundSource, SoundCodecError> {
    let raw_id = reader.read_var_i32()?;
    SoundSource::from_id(raw_id).ok_or(SoundCodecError::InvalidSource { raw_id })
}

fn write_source(writer: &mut WireWriter, source: SoundSource) -> Result<(), SoundCodecError> {
    writer.write_var_i32(source.id())?;
    Ok(())
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, SoundCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
