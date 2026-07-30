use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const FIELD_MAX: u64 = 32_766;
const COMPONENT_LIMIT: f64 = 17_179_869_183.0;
const ZERO_THRESHOLD: f64 = 3.051_944_088_384_301e-5;

pub(crate) fn read(reader: &mut WireReader<'_>) -> Result<Vector3, WireError> {
    let lowest = reader.read_u8()?;
    if lowest == 0 {
        return Ok(Vector3::default());
    }
    let middle = reader.read_u8()?;
    let highest = reader.read_i32()? as u32;
    let packed = u64::from(lowest) | (u64::from(middle) << 8) | (u64::from(highest) << 16);
    let mut scale = u64::from(lowest & 0x03);
    if lowest & 0x04 != 0 {
        scale |= u64::from(reader.read_var_i32()? as u32) << 2;
    }
    let scale = scale as f64;
    Ok(Vector3 {
        x: unpack((packed >> 3) & 0x7fff) * scale,
        y: unpack((packed >> 18) & 0x7fff) * scale,
        z: unpack((packed >> 33) & 0x7fff) * scale,
    })
}

pub(crate) fn write(writer: &mut WireWriter, vector: Vector3) -> Result<(), WireError> {
    let vector = Vector3 {
        x: sanitize(vector.x),
        y: sanitize(vector.y),
        z: sanitize(vector.z),
    };
    let largest = vector.x.abs().max(vector.y.abs()).max(vector.z.abs());
    if largest < ZERO_THRESHOLD {
        return writer.write_u8(0);
    }
    let scale = largest.ceil() as u64;
    let mut packed = (pack(vector.x / scale as f64) << 3)
        | (pack(vector.y / scale as f64) << 18)
        | (pack(vector.z / scale as f64) << 33)
        | (scale & 0x03);
    if scale > 3 {
        packed |= 0x04;
    }
    writer.write_u8(packed as u8)?;
    writer.write_u8((packed >> 8) as u8)?;
    writer.write_i32((packed >> 16) as u32 as i32)?;
    if scale > 3 {
        writer.write_var_i32((scale >> 2) as u32 as i32)?;
    }
    Ok(())
}

fn sanitize(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-COMPONENT_LIMIT, COMPONENT_LIMIT)
    }
}

fn pack(value: f64) -> u64 {
    ((value + 1.0) * 0.5 * FIELD_MAX as f64).round() as u64
}

fn unpack(value: u64) -> f64 {
    value.min(FIELD_MAX) as f64 * 2.0 / FIELD_MAX as f64 - 1.0
}
