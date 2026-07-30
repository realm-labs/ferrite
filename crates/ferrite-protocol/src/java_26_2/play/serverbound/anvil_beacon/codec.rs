use thiserror::Error;

use crate::java_26_2::play::registry::{MOB_EFFECT, PlayRegistries, PlayRegistryError};
use crate::java_26_2::play::serverbound::anvil_beacon::packet::{RenameItem, SetBeacon};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const DEFAULT_UTF_CODE_UNITS: usize = 32_767;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AnvilBeaconCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
}

pub fn decode_rename(reader: &mut WireReader<'_>) -> Result<RenameItem, AnvilBeaconCodecError> {
    Ok(RenameItem {
        name: reader.read_utf(DEFAULT_UTF_CODE_UNITS)?.into_owned(),
    })
}

pub fn encode_rename(
    writer: &mut WireWriter,
    packet: &RenameItem,
) -> Result<(), AnvilBeaconCodecError> {
    writer.write_utf(&packet.name, DEFAULT_UTF_CODE_UNITS)?;
    Ok(())
}

pub fn decode_set_beacon(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<SetBeacon, AnvilBeaconCodecError> {
    Ok(SetBeacon {
        primary: read_optional_effect(reader, registries)?,
        secondary: read_optional_effect(reader, registries)?,
    })
}

pub fn encode_set_beacon(
    writer: &mut WireWriter,
    packet: &SetBeacon,
    registries: &PlayRegistries,
) -> Result<(), AnvilBeaconCodecError> {
    write_optional_effect(writer, packet.primary.as_ref(), registries)?;
    write_optional_effect(writer, packet.secondary.as_ref(), registries)?;
    Ok(())
}

fn read_optional_effect(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<Option<Identifier>, AnvilBeaconCodecError> {
    if reader.read_bool()? {
        Ok(Some(
            registries.resolve(MOB_EFFECT, reader.read_var_i32()?)?,
        ))
    } else {
        Ok(None)
    }
}

fn write_optional_effect(
    writer: &mut WireWriter,
    effect: Option<&Identifier>,
    registries: &PlayRegistries,
) -> Result<(), AnvilBeaconCodecError> {
    writer.write_bool(effect.is_some())?;
    if let Some(effect) = effect {
        writer.write_var_i32(registries.raw_id(MOB_EFFECT, effect)?)?;
    }
    Ok(())
}
