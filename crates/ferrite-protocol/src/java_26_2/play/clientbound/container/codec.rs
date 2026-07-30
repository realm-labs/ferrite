use thiserror::Error;

use crate::java_26_2::play::clientbound::container::packet::{
    ContainerClose, ContainerSetContent, ContainerSetData, ContainerSetSlot, OpenScreen,
    SetCursorItem, SetPlayerInventory,
};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::item::{ItemCodecError, read_optional_stack, write_optional_stack};
use crate::java_26_2::play::registry::{MENU, PlayRegistries, PlayRegistryError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_close(
    reader: &mut WireReader<'_>,
) -> Result<ContainerClose, ContainerCodecError> {
    Ok(ContainerClose {
        container_id: reader.read_var_i32()?,
    })
}

pub(crate) fn write_close(
    writer: &mut WireWriter,
    packet: ContainerClose,
) -> Result<(), ContainerCodecError> {
    writer.write_var_i32(packet.container_id)?;
    Ok(())
}

pub(crate) fn read_content(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<ContainerSetContent, ContainerCodecError> {
    let container_id = reader.read_var_i32()?;
    let state_id = reader.read_var_i32()?;
    let count = reader.read_count("container items", reader.remaining())?;
    let mut slots = Vec::with_capacity(count);
    for _ in 0..count {
        slots.push(read_optional_stack(reader, context)?);
    }
    let carried = read_optional_stack(reader, context)?;
    Ok(ContainerSetContent {
        container_id,
        state_id,
        slots,
        carried,
    })
}

pub(crate) fn write_content(
    writer: &mut WireWriter,
    packet: &ContainerSetContent,
    registries: &PlayRegistries,
) -> Result<(), ContainerCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_var_i32(packet.state_id)?;
    writer.write_count(
        "container items",
        packet.slots.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for stack in &packet.slots {
        write_optional_stack(writer, stack, registries)?;
    }
    write_optional_stack(writer, &packet.carried, registries)?;
    Ok(())
}

pub(crate) fn read_data(
    reader: &mut WireReader<'_>,
) -> Result<ContainerSetData, ContainerCodecError> {
    Ok(ContainerSetData {
        container_id: reader.read_var_i32()?,
        property_id: reader.read_i16()?,
        value: reader.read_i16()?,
    })
}

pub(crate) fn write_data(
    writer: &mut WireWriter,
    packet: ContainerSetData,
) -> Result<(), ContainerCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_i16(packet.property_id)?;
    writer.write_i16(packet.value)?;
    Ok(())
}

pub(crate) fn read_slot(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<ContainerSetSlot, ContainerCodecError> {
    Ok(ContainerSetSlot {
        container_id: reader.read_var_i32()?,
        state_id: reader.read_var_i32()?,
        slot: reader.read_i16()?,
        item: read_optional_stack(reader, context)?,
    })
}

pub(crate) fn write_slot(
    writer: &mut WireWriter,
    packet: &ContainerSetSlot,
    registries: &PlayRegistries,
) -> Result<(), ContainerCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_var_i32(packet.state_id)?;
    writer.write_i16(packet.slot)?;
    write_optional_stack(writer, &packet.item, registries)?;
    Ok(())
}

pub(crate) fn read_open(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<OpenScreen, ContainerCodecError> {
    let container_id = reader.read_var_i32()?;
    let menu_type = registries.resolve(MENU, reader.read_var_i32()?)?;
    let title = TextComponentNbt::from_network_nbt(NetworkNbt::read(reader, NbtQuota::Trusted)?)?;
    Ok(OpenScreen {
        container_id,
        menu_type,
        title,
    })
}

pub(crate) fn write_open(
    writer: &mut WireWriter,
    packet: &OpenScreen,
    registries: &PlayRegistries,
) -> Result<(), ContainerCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_var_i32(registries.raw_id(MENU, &packet.menu_type)?)?;
    packet.title.network_nbt().write(writer)?;
    Ok(())
}

pub(crate) fn read_cursor(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<SetCursorItem, ContainerCodecError> {
    Ok(SetCursorItem {
        item: read_optional_stack(reader, context)?,
    })
}

pub(crate) fn write_cursor(
    writer: &mut WireWriter,
    packet: &SetCursorItem,
    registries: &PlayRegistries,
) -> Result<(), ContainerCodecError> {
    write_optional_stack(writer, &packet.item, registries)?;
    Ok(())
}

pub(crate) fn read_player_inventory(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<SetPlayerInventory, ContainerCodecError> {
    Ok(SetPlayerInventory {
        slot: reader.read_var_i32()?,
        item: read_optional_stack(reader, context)?,
    })
}

pub(crate) fn write_player_inventory(
    writer: &mut WireWriter,
    packet: &SetPlayerInventory,
    registries: &PlayRegistries,
) -> Result<(), ContainerCodecError> {
    writer.write_var_i32(packet.slot)?;
    write_optional_stack(writer, &packet.item, registries)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContainerCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Item(#[from] ItemCodecError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
}
