use crate::java_26_2::play::serverbound::merchant::packet::SelectTrade;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub fn decode_select_trade(reader: &mut WireReader<'_>) -> Result<SelectTrade, WireError> {
    Ok(SelectTrade {
        selection_hint: reader.read_var_i32()?,
    })
}

pub fn encode_select_trade(writer: &mut WireWriter, packet: SelectTrade) -> Result<(), WireError> {
    writer.write_var_i32(packet.selection_hint)
}
