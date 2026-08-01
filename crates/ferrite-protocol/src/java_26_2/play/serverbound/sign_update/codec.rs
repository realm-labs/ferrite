use crate::java_26_2::play::block::{pack_block_position, unpack_block_position};
use crate::java_26_2::play::serverbound::sign_update::packet::SignUpdate;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const SERVER_LINE_LIMIT: usize = 384;
const MEMBER_ENCODER_LINE_LIMIT: usize = 32_767;

pub(crate) fn decode(reader: &mut WireReader<'_>) -> Result<SignUpdate, WireError> {
    Ok(SignUpdate {
        position: unpack_block_position(reader.read_i64()?),
        front_text: reader.read_bool()?,
        lines: [
            reader.read_utf(SERVER_LINE_LIMIT)?.into_owned(),
            reader.read_utf(SERVER_LINE_LIMIT)?.into_owned(),
            reader.read_utf(SERVER_LINE_LIMIT)?.into_owned(),
            reader.read_utf(SERVER_LINE_LIMIT)?.into_owned(),
        ],
    })
}

pub(crate) fn encode(writer: &mut WireWriter, packet: &SignUpdate) -> Result<(), WireError> {
    writer.write_i64(pack_block_position(packet.position))?;
    writer.write_bool(packet.front_text)?;
    for line in &packet.lines {
        writer.write_utf(line, MEMBER_ENCODER_LINE_LIMIT)?;
    }
    Ok(())
}
