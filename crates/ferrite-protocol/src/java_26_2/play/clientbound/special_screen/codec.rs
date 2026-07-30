use thiserror::Error;

use crate::java_26_2::play::block::{pack_block_position, unpack_block_position};
use crate::java_26_2::play::clientbound::special_screen::packet::{
    InteractionHand, MountScreenOpen, OpenSignEditor,
};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_mount(
    reader: &mut WireReader<'_>,
) -> Result<MountScreenOpen, SpecialScreenCodecError> {
    Ok(MountScreenOpen {
        container_id: reader.read_var_i32()?,
        inventory_columns: reader.read_var_i32()?,
        entity_id: reader.read_i32()?,
    })
}

pub(crate) fn write_mount(
    writer: &mut WireWriter,
    packet: MountScreenOpen,
) -> Result<(), SpecialScreenCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_var_i32(packet.inventory_columns)?;
    writer.write_i32(packet.entity_id)?;
    Ok(())
}

pub(crate) fn read_hand(
    reader: &mut WireReader<'_>,
) -> Result<InteractionHand, SpecialScreenCodecError> {
    let ordinal = reader.read_var_i32()?;
    InteractionHand::from_ordinal(ordinal).ok_or(SpecialScreenCodecError::InvalidHand { ordinal })
}

pub(crate) fn write_hand(
    writer: &mut WireWriter,
    hand: InteractionHand,
) -> Result<(), SpecialScreenCodecError> {
    writer.write_var_i32(hand.ordinal())?;
    Ok(())
}

pub(crate) fn read_sign(
    reader: &mut WireReader<'_>,
) -> Result<OpenSignEditor, SpecialScreenCodecError> {
    Ok(OpenSignEditor {
        position: unpack_block_position(reader.read_i64()?),
        front_text: reader.read_bool()?,
    })
}

pub(crate) fn write_sign(
    writer: &mut WireWriter,
    packet: OpenSignEditor,
) -> Result<(), SpecialScreenCodecError> {
    writer.write_i64(pack_block_position(packet.position))?;
    writer.write_bool(packet.front_text)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SpecialScreenCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("interaction-hand ordinal {ordinal} is outside 0..=1")]
    InvalidHand { ordinal: i32 },
}
