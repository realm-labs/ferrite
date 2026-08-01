use crate::java_26_2::play::block::{pack_block_position, unpack_block_position};
use crate::java_26_2::play::clientbound::world_effect::packet::LevelEvent;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read(reader: &mut WireReader<'_>) -> Result<LevelEvent, WireError> {
    Ok(LevelEvent {
        event_type: reader.read_i32()?,
        position: unpack_block_position(reader.read_i64()?),
        data: reader.read_i32()?,
        global: reader.read_bool()?,
    })
}

pub(crate) fn write(writer: &mut WireWriter, packet: LevelEvent) -> Result<(), WireError> {
    writer.write_i32(packet.event_type)?;
    writer.write_i64(pack_block_position(packet.position))?;
    writer.write_i32(packet.data)?;
    writer.write_bool(packet.global)?;
    Ok(())
}
