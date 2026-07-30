use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_spawn::packet::{AddEntity, RemoveEntities};
use crate::java_26_2::play::clientbound::entity_spawn::type_registry;
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::lp_vec;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_add(reader: &mut WireReader<'_>) -> Result<AddEntity, WireError> {
    Ok(AddEntity {
        entity_id: reader.read_var_i32()?,
        uuid: reader.read_u128()?,
        entity_type: type_registry::resolve(reader.read_var_i32()?),
        position: read_vector(reader)?,
        motion: lp_vec::read(reader)?,
        pitch: reader.read_i8()?,
        yaw: reader.read_i8()?,
        head_yaw: reader.read_i8()?,
        data: reader.read_var_i32()?,
    })
}

pub(crate) fn write_add(
    writer: &mut WireWriter,
    packet: &AddEntity,
) -> Result<(), EntitySpawnCodecError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_u128(packet.uuid)?;
    let raw_type = type_registry::raw_id(&packet.entity_type).ok_or_else(|| {
        EntitySpawnCodecError::UnknownEntityType {
            identity: packet.entity_type.clone(),
        }
    })?;
    writer.write_var_i32(raw_type)?;
    write_vector(writer, packet.position)?;
    lp_vec::write(writer, packet.motion)?;
    writer.write_i8(packet.pitch)?;
    writer.write_i8(packet.yaw)?;
    writer.write_i8(packet.head_yaw)?;
    writer.write_var_i32(packet.data)?;
    Ok(())
}

pub(crate) fn read_remove(reader: &mut WireReader<'_>) -> Result<RemoveEntities, WireError> {
    let signed_count = reader.read_var_i32()?;
    if signed_count < 0 {
        return Ok(RemoveEntities {
            entity_ids: Vec::new(),
        });
    }
    let count = signed_count as usize;
    if count > reader.remaining() {
        return Err(WireError::LengthLimit {
            field: "removed entity IDs",
            length: count,
            maximum: reader.remaining(),
        });
    }
    let mut entity_ids = Vec::with_capacity(count);
    for _ in 0..count {
        entity_ids.push(reader.read_var_i32()?);
    }
    Ok(RemoveEntities { entity_ids })
}

pub(crate) fn write_remove(
    writer: &mut WireWriter,
    packet: &RemoveEntities,
) -> Result<(), WireError> {
    writer.write_count(
        "removed entity IDs",
        packet.entity_ids.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for entity_id in &packet.entity_ids {
        writer.write_var_i32(*entity_id)?;
    }
    Ok(())
}

fn read_vector(reader: &mut WireReader<'_>) -> Result<Vector3, WireError> {
    Ok(Vector3 {
        x: reader.read_f64()?,
        y: reader.read_f64()?,
        z: reader.read_f64()?,
    })
}

fn write_vector(writer: &mut WireWriter, vector: Vector3) -> Result<(), WireError> {
    writer.write_f64(vector.x)?;
    writer.write_f64(vector.y)?;
    writer.write_f64(vector.z)
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EntitySpawnCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("entity type {identity} is absent from the locked static registry")]
    UnknownEntityType { identity: Identifier },
}
