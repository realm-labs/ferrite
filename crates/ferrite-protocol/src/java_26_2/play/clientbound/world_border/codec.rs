use crate::java_26_2::play::clientbound::world_border::packet::{
    SetBorderCenter, SetBorderLerpSize, SetBorderSize, SetBorderWarningDelay,
    SetBorderWarningDistance,
};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_center(reader: &mut WireReader<'_>) -> Result<SetBorderCenter, WireError> {
    Ok(SetBorderCenter {
        center_x: reader.read_f64()?,
        center_z: reader.read_f64()?,
    })
}

pub(crate) fn write_center(
    writer: &mut WireWriter,
    packet: SetBorderCenter,
) -> Result<(), WireError> {
    writer.write_f64(packet.center_x)?;
    writer.write_f64(packet.center_z)?;
    Ok(())
}

pub(crate) fn read_lerp(reader: &mut WireReader<'_>) -> Result<SetBorderLerpSize, WireError> {
    Ok(SetBorderLerpSize {
        old_size: reader.read_f64()?,
        new_size: reader.read_f64()?,
        duration_millis: reader.read_var_i64()?,
    })
}

pub(crate) fn write_lerp(
    writer: &mut WireWriter,
    packet: SetBorderLerpSize,
) -> Result<(), WireError> {
    writer.write_f64(packet.old_size)?;
    writer.write_f64(packet.new_size)?;
    writer.write_var_i64(packet.duration_millis)?;
    Ok(())
}

pub(crate) fn read_size(reader: &mut WireReader<'_>) -> Result<SetBorderSize, WireError> {
    Ok(SetBorderSize {
        size: reader.read_f64()?,
    })
}

pub(crate) fn write_size(writer: &mut WireWriter, packet: SetBorderSize) -> Result<(), WireError> {
    writer.write_f64(packet.size)?;
    Ok(())
}

pub(crate) fn read_warning_delay(
    reader: &mut WireReader<'_>,
) -> Result<SetBorderWarningDelay, WireError> {
    Ok(SetBorderWarningDelay {
        warning_time: reader.read_var_i32()?,
    })
}

pub(crate) fn write_warning_delay(
    writer: &mut WireWriter,
    packet: SetBorderWarningDelay,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.warning_time)?;
    Ok(())
}

pub(crate) fn read_warning_distance(
    reader: &mut WireReader<'_>,
) -> Result<SetBorderWarningDistance, WireError> {
    Ok(SetBorderWarningDistance {
        warning_blocks: reader.read_var_i32()?,
    })
}

pub(crate) fn write_warning_distance(
    writer: &mut WireWriter,
    packet: SetBorderWarningDistance,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.warning_blocks)?;
    Ok(())
}
