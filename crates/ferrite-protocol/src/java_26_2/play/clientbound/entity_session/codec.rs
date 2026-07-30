use crate::java_26_2::play::clientbound::entity_session::packet::{
    Animate, DamageEvent, HurtAnimation, SetCamera, TakeItemEntity,
};
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::registry::{DAMAGE_TYPE, PlayRegistries, PlayRegistryError};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_animate(reader: &mut WireReader<'_>) -> Result<Animate, WireError> {
    Ok(Animate {
        entity_id: reader.read_var_i32()?,
        action: reader.read_u8()?,
    })
}

pub(crate) fn write_animate(writer: &mut WireWriter, packet: Animate) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_u8(packet.action)
}

pub(crate) fn read_damage(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<DamageEvent, EntitySessionCodecError> {
    let entity_id = reader.read_var_i32()?;
    let damage_type = registries.resolve(DAMAGE_TYPE, reader.read_var_i32()?)?;
    let cause_entity_id = reader.read_var_i32()?.wrapping_sub(1);
    let direct_entity_id = reader.read_var_i32()?.wrapping_sub(1);
    let source_position = if reader.read_bool()? {
        Some(read_vector(reader)?)
    } else {
        None
    };
    Ok(DamageEvent {
        entity_id,
        damage_type,
        cause_entity_id,
        direct_entity_id,
        source_position,
    })
}

pub(crate) fn write_damage(
    writer: &mut WireWriter,
    packet: &DamageEvent,
    registries: &PlayRegistries,
) -> Result<(), EntitySessionCodecError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_var_i32(registries.raw_id(DAMAGE_TYPE, &packet.damage_type)?)?;
    writer.write_var_i32(packet.cause_entity_id.wrapping_add(1))?;
    writer.write_var_i32(packet.direct_entity_id.wrapping_add(1))?;
    writer.write_bool(packet.source_position.is_some())?;
    if let Some(position) = packet.source_position {
        write_vector(writer, position)?;
    }
    Ok(())
}

pub(crate) fn read_hurt(reader: &mut WireReader<'_>) -> Result<HurtAnimation, WireError> {
    Ok(HurtAnimation {
        entity_id: reader.read_var_i32()?,
        yaw: reader.read_f32()?,
    })
}

pub(crate) fn write_hurt(writer: &mut WireWriter, packet: HurtAnimation) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_f32(packet.yaw)
}

pub(crate) fn read_camera(reader: &mut WireReader<'_>) -> Result<SetCamera, WireError> {
    Ok(SetCamera {
        entity_id: reader.read_var_i32()?,
    })
}

pub(crate) fn write_camera(writer: &mut WireWriter, packet: SetCamera) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)
}

pub(crate) fn read_take(reader: &mut WireReader<'_>) -> Result<TakeItemEntity, WireError> {
    Ok(TakeItemEntity {
        source_entity_id: reader.read_var_i32()?,
        collector_entity_id: reader.read_var_i32()?,
        amount: reader.read_var_i32()?,
    })
}

pub(crate) fn write_take(writer: &mut WireWriter, packet: TakeItemEntity) -> Result<(), WireError> {
    writer.write_var_i32(packet.source_entity_id)?;
    writer.write_var_i32(packet.collector_entity_id)?;
    writer.write_var_i32(packet.amount)
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

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EntitySessionCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
}
