use crate::java_26_2::play::serverbound::entity_session::packet::LowPrecisionVector;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub const MAX_COMPONENT: f64 = 17_179_869_183.0;
pub const ZERO_THRESHOLD: f64 = 3.051_944_088_384_301e-5;

pub fn read(reader: &mut WireReader<'_>) -> Result<LowPrecisionVector, WireError> {
    let lowest = reader.read_u8()?;
    if lowest == 0 {
        return Ok(LowPrecisionVector::ZERO);
    }
    let middle = reader.read_u8()?;
    let highest = reader.read_i32()? as u32;
    let packed = u64::from(highest) << 16 | u64::from(middle) << 8 | u64::from(lowest);
    let mut scale = u64::from(lowest & 3);
    if lowest & 4 != 0 {
        scale |= u64::from(reader.read_var_i32()? as u32) << 2;
    }
    let scale = scale as f64;
    Ok(LowPrecisionVector {
        x: unpack(packed >> 3) * scale,
        y: unpack(packed >> 18) * scale,
        z: unpack(packed >> 33) * scale,
    })
}

pub fn write(writer: &mut WireWriter, vector: LowPrecisionVector) -> Result<(), WireError> {
    let vector = LowPrecisionVector {
        x: canonical_component(vector.x),
        y: canonical_component(vector.y),
        z: canonical_component(vector.z),
    };
    let maximum = vector.x.abs().max(vector.y.abs()).max(vector.z.abs());
    if maximum < ZERO_THRESHOLD {
        return writer.write_u8(0);
    }

    let scale = maximum.ceil() as u64;
    let packed = pack(vector.x / scale as f64)
        | pack(vector.y / scale as f64) << 15
        | pack(vector.z / scale as f64) << 30;
    let first = ((packed << 3) as u8) | (scale as u8 & 3) | if scale > 3 { 4 } else { 0 };
    writer.write_u8(first)?;
    writer.write_u8((packed >> 5) as u8)?;
    writer.write_i32((packed >> 13) as u32 as i32)?;
    if scale > 3 {
        writer.write_var_i32((scale >> 2) as u32 as i32)?;
    }
    Ok(())
}

fn canonical_component(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-MAX_COMPONENT, MAX_COMPONENT)
    }
}

fn unpack(value: u64) -> f64 {
    (value & 0x7fff).min(32_766) as f64 * 2.0 / 32_766.0 - 1.0
}

fn pack(value: f64) -> u64 {
    ((value + 1.0) * 16_383.0).round().clamp(0.0, 32_766.0) as u64
}
