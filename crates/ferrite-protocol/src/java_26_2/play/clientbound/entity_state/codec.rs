use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_effects::codec::EntityEffectsCodecError;
use crate::java_26_2::play::clientbound::entity_state::metadata_codec;
use crate::java_26_2::play::clientbound::entity_state::packet::{
    AttributeModifier, AttributeOperation, AttributeSnapshot, EquipmentEntry, EquipmentSlot,
    SetEntityData, SetEntityLink, SetEquipment, SetPassengers, UpdateAttributes,
};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::item::{ItemCodecError, read_optional_stack, write_optional_stack};
use crate::java_26_2::play::registry::{ATTRIBUTE, PlayRegistries, PlayRegistryError};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::nbt::NbtError;
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_data(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<SetEntityData, EntityStateCodecError> {
    let entity_id = reader.read_var_i32()?;
    let values = metadata_codec::read_entries(reader, context)?;
    Ok(SetEntityData { entity_id, values })
}

pub(crate) fn write_data(
    writer: &mut WireWriter,
    packet: &SetEntityData,
    registries: &PlayRegistries,
) -> Result<(), EntityStateCodecError> {
    writer.write_var_i32(packet.entity_id)?;
    metadata_codec::write_entries(writer, &packet.values, registries)
}

pub(crate) fn read_link(
    reader: &mut WireReader<'_>,
) -> Result<SetEntityLink, EntityStateCodecError> {
    Ok(SetEntityLink {
        source_entity_id: reader.read_i32()?,
        destination_entity_id: reader.read_i32()?,
    })
}

pub(crate) fn write_link(
    writer: &mut WireWriter,
    packet: SetEntityLink,
) -> Result<(), EntityStateCodecError> {
    writer.write_i32(packet.source_entity_id)?;
    writer.write_i32(packet.destination_entity_id)?;
    Ok(())
}

pub(crate) fn read_equipment(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<SetEquipment, EntityStateCodecError> {
    let entity_id = reader.read_var_i32()?;
    let mut entries = Vec::new();
    loop {
        let descriptor = reader.read_u8()?;
        let ordinal = descriptor & 0x7f;
        let slot = EquipmentSlot::from_ordinal(ordinal)
            .ok_or(EntityStateCodecError::InvalidEquipmentOrdinal { ordinal })?;
        entries.push(EquipmentEntry {
            slot,
            stack: read_optional_stack(reader, context)?,
        });
        if descriptor & 0x80 == 0 {
            break;
        }
    }
    Ok(SetEquipment { entity_id, entries })
}

pub(crate) fn write_equipment(
    writer: &mut WireWriter,
    packet: &SetEquipment,
    registries: &PlayRegistries,
) -> Result<(), EntityStateCodecError> {
    if packet.entries.is_empty() {
        return Err(EntityStateCodecError::EmptyEquipment);
    }
    writer.write_var_i32(packet.entity_id)?;
    let last = packet.entries.len() - 1;
    for (index, entry) in packet.entries.iter().enumerate() {
        let continuation = if index == last { 0 } else { 0x80 };
        writer.write_u8(entry.slot.ordinal() | continuation)?;
        write_optional_stack(writer, &entry.stack, registries)?;
    }
    Ok(())
}

pub(crate) fn read_passengers(
    reader: &mut WireReader<'_>,
) -> Result<SetPassengers, EntityStateCodecError> {
    let vehicle_id = reader.read_var_i32()?;
    let count = reader.read_count("passenger entity IDs", reader.remaining())?;
    let mut passenger_ids = Vec::with_capacity(count);
    for _ in 0..count {
        passenger_ids.push(reader.read_var_i32()?);
    }
    Ok(SetPassengers {
        vehicle_id,
        passenger_ids,
    })
}

pub(crate) fn write_passengers(
    writer: &mut WireWriter,
    packet: &SetPassengers,
) -> Result<(), EntityStateCodecError> {
    writer.write_var_i32(packet.vehicle_id)?;
    writer.write_count(
        "passenger entity IDs",
        packet.passenger_ids.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for passenger_id in &packet.passenger_ids {
        writer.write_var_i32(*passenger_id)?;
    }
    Ok(())
}

pub(crate) fn read_attributes(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<UpdateAttributes, EntityStateCodecError> {
    let entity_id = reader.read_var_i32()?;
    let count = reader.read_count("attribute snapshots", 128)?;
    let mut snapshots = Vec::with_capacity(count);
    for _ in 0..count {
        let attribute = context
            .registries
            .resolve(ATTRIBUTE, reader.read_var_i32()?)?;
        let base = reader.read_f64()?;
        let modifier_count = reader.read_count("attribute modifiers", reader.remaining())?;
        let mut modifiers = Vec::with_capacity(modifier_count);
        for _ in 0..modifier_count {
            modifiers.push(AttributeModifier {
                identity: read_identifier(reader)?,
                amount: reader.read_f64()?,
                operation: AttributeOperation::from_raw_id(reader.read_var_i32()?),
            });
        }
        snapshots.push(AttributeSnapshot {
            attribute,
            base,
            modifiers,
        });
    }
    Ok(UpdateAttributes {
        entity_id,
        snapshots,
    })
}

pub(crate) fn write_attributes(
    writer: &mut WireWriter,
    packet: &UpdateAttributes,
    registries: &PlayRegistries,
) -> Result<(), EntityStateCodecError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_count("attribute snapshots", packet.snapshots.len(), 128)?;
    for snapshot in &packet.snapshots {
        writer.write_var_i32(registries.raw_id(ATTRIBUTE, &snapshot.attribute)?)?;
        writer.write_f64(snapshot.base)?;
        writer.write_count(
            "attribute modifiers",
            snapshot.modifiers.len(),
            MAX_INFLATED_PACKET_LENGTH,
        )?;
        for modifier in &snapshot.modifiers {
            modifier.identity.write(writer)?;
            writer.write_f64(modifier.amount)?;
            writer.write_var_i32(modifier.operation.raw_id())?;
        }
    }
    Ok(())
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, EntityStateCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EntityStateCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error(transparent)]
    Item(#[from] ItemCodecError),
    #[error(transparent)]
    Particle(#[from] EntityEffectsCodecError),
    #[error("metadata serializer raw ID {raw_id} is absent from the locked 43-entry table")]
    UnknownMetadataSerializer { raw_id: i32 },
    #[error("metadata slot 255 is reserved for the list terminator")]
    MetadataTerminatorSlot,
    #[error("metadata entry declares {declared:?} but carries {actual:?}")]
    MetadataValueMismatch {
        declared: crate::java_26_2::play::clientbound::entity_state::metadata::MetadataSerializer,
        actual: crate::java_26_2::play::clientbound::entity_state::metadata::MetadataSerializer,
    },
    #[error("metadata serializer {serializer:?} cannot carry a holder identity")]
    InvalidHolderSerializer {
        serializer: crate::java_26_2::play::clientbound::entity_state::metadata::MetadataSerializer,
    },
    #[error("metadata serializer {serializer:?} cannot carry a source enum state")]
    InvalidEnumStateSerializer {
        serializer: crate::java_26_2::play::clientbound::entity_state::metadata::MetadataSerializer,
    },
    #[error("metadata block state {state} is outside the locked 0..=32365 range")]
    InvalidMetadataBlockState { state: i32 },
    #[error("metadata enum {serializer:?} value {value} is outside its canonical range")]
    InvalidMetadataEnumState {
        serializer: crate::java_26_2::play::clientbound::entity_state::metadata::MetadataSerializer,
        value: u8,
    },
    #[error("equipment descriptor ordinal {ordinal} is outside 0..=7")]
    InvalidEquipmentOrdinal { ordinal: u8 },
    #[error("set-equipment requires at least one entry")]
    EmptyEquipment,
}
