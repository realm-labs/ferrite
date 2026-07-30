use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::registry::{
    DATA_COMPONENT_TYPE, ITEM, PlayRegistries, PlayRegistryError,
};
use crate::java_26_2::play::serverbound::container::packet::{
    ContainerButtonClick, ContainerClick, ContainerClose, ContainerInput,
    ContainerSlotStateChanged, HashedComponentPatch, HashedStack, HashedStackContents,
    SetCarriedItem,
};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_CHANGED_SLOTS: usize = 128;
const MAX_HASHED_COMPONENTS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContainerServerboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
}

pub fn decode_button(
    reader: &mut WireReader<'_>,
) -> Result<ContainerButtonClick, ContainerServerboundCodecError> {
    Ok(ContainerButtonClick {
        container_id: reader.read_var_i32()?,
        button_id: reader.read_var_i32()?,
    })
}

pub fn encode_button(
    writer: &mut WireWriter,
    packet: ContainerButtonClick,
) -> Result<(), ContainerServerboundCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_var_i32(packet.button_id)?;
    Ok(())
}

pub fn decode_click(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<ContainerClick, ContainerServerboundCodecError> {
    let container_id = reader.read_var_i32()?;
    let state_id = reader.read_var_i32()?;
    let slot = reader.read_i16()?;
    let button = reader.read_i8()?;
    let input = ContainerInput::from_wire(reader.read_var_i32()?);
    let changed_count = reader.read_count("changed container slots", MAX_CHANGED_SLOTS)?;
    let mut changed_slots = BTreeMap::new();
    for _ in 0..changed_count {
        changed_slots.insert(reader.read_i16()?, decode_hashed_stack(reader, registries)?);
    }
    let carried = decode_hashed_stack(reader, registries)?;
    Ok(ContainerClick {
        container_id,
        state_id,
        slot,
        button,
        input,
        changed_slots,
        carried,
    })
}

pub fn encode_click(
    writer: &mut WireWriter,
    packet: &ContainerClick,
    registries: &PlayRegistries,
) -> Result<(), ContainerServerboundCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_var_i32(packet.state_id)?;
    writer.write_i16(packet.slot)?;
    writer.write_i8(packet.button)?;
    writer.write_var_i32(packet.input.to_wire())?;
    writer.write_count(
        "changed container slots",
        packet.changed_slots.len(),
        MAX_CHANGED_SLOTS,
    )?;
    for (slot, stack) in &packet.changed_slots {
        writer.write_i16(*slot)?;
        encode_hashed_stack(writer, stack, registries)?;
    }
    encode_hashed_stack(writer, &packet.carried, registries)
}

pub fn decode_close(
    reader: &mut WireReader<'_>,
) -> Result<ContainerClose, ContainerServerboundCodecError> {
    Ok(ContainerClose {
        container_id: reader.read_var_i32()?,
    })
}

pub fn encode_close(
    writer: &mut WireWriter,
    packet: ContainerClose,
) -> Result<(), ContainerServerboundCodecError> {
    writer.write_var_i32(packet.container_id)?;
    Ok(())
}

pub fn decode_slot_state(
    reader: &mut WireReader<'_>,
) -> Result<ContainerSlotStateChanged, ContainerServerboundCodecError> {
    Ok(ContainerSlotStateChanged {
        slot_id: reader.read_var_i32()?,
        container_id: reader.read_var_i32()?,
        new_state: reader.read_bool()?,
    })
}

pub fn encode_slot_state(
    writer: &mut WireWriter,
    packet: ContainerSlotStateChanged,
) -> Result<(), ContainerServerboundCodecError> {
    writer.write_var_i32(packet.slot_id)?;
    writer.write_var_i32(packet.container_id)?;
    writer.write_bool(packet.new_state)?;
    Ok(())
}

pub fn decode_set_carried(
    reader: &mut WireReader<'_>,
) -> Result<SetCarriedItem, ContainerServerboundCodecError> {
    Ok(SetCarriedItem {
        slot: reader.read_i16()?,
    })
}

pub fn encode_set_carried(
    writer: &mut WireWriter,
    packet: SetCarriedItem,
) -> Result<(), ContainerServerboundCodecError> {
    writer.write_i16(packet.slot)?;
    Ok(())
}

fn decode_hashed_stack(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<HashedStack, ContainerServerboundCodecError> {
    if !reader.read_bool()? {
        return Ok(HashedStack::Empty);
    }
    Ok(HashedStack::Present(HashedStackContents {
        item: registries.resolve(ITEM, reader.read_var_i32()?)?,
        count: reader.read_var_i32()?,
        components: decode_hashed_patch(reader, registries)?,
    }))
}

fn encode_hashed_stack(
    writer: &mut WireWriter,
    stack: &HashedStack,
    registries: &PlayRegistries,
) -> Result<(), ContainerServerboundCodecError> {
    let HashedStack::Present(contents) = stack else {
        writer.write_bool(false)?;
        return Ok(());
    };
    writer.write_bool(true)?;
    writer.write_var_i32(registries.raw_id(ITEM, &contents.item)?)?;
    writer.write_var_i32(contents.count)?;
    encode_hashed_patch(writer, &contents.components, registries)
}

fn decode_hashed_patch(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<HashedComponentPatch, ContainerServerboundCodecError> {
    let added_count = reader.read_count("hashed added components", MAX_HASHED_COMPONENTS)?;
    let mut added = BTreeMap::new();
    for _ in 0..added_count {
        let component = registries.resolve(DATA_COMPONENT_TYPE, reader.read_var_i32()?)?;
        added.insert(component, reader.read_i32()?);
    }
    let removed_count = reader.read_count("hashed removed components", MAX_HASHED_COMPONENTS)?;
    let mut removed = BTreeSet::new();
    for _ in 0..removed_count {
        removed.insert(registries.resolve(DATA_COMPONENT_TYPE, reader.read_var_i32()?)?);
    }
    Ok(HashedComponentPatch { added, removed })
}

fn encode_hashed_patch(
    writer: &mut WireWriter,
    patch: &HashedComponentPatch,
    registries: &PlayRegistries,
) -> Result<(), ContainerServerboundCodecError> {
    writer.write_count(
        "hashed added components",
        patch.added.len(),
        MAX_HASHED_COMPONENTS,
    )?;
    for (component, hash) in &patch.added {
        writer.write_var_i32(registries.raw_id(DATA_COMPONENT_TYPE, component)?)?;
        writer.write_i32(*hash)?;
    }
    writer.write_count(
        "hashed removed components",
        patch.removed.len(),
        MAX_HASHED_COMPONENTS,
    )?;
    for component in &patch.removed {
        writer.write_var_i32(registries.raw_id(DATA_COMPONENT_TYPE, component)?)?;
    }
    Ok(())
}
