use crate::java_26_2::play::clientbound::entity_motion::packet::{
    EntityPositionSync, MinecartStep, MoveMinecartAlongTrack, PositionMoveRotation,
    ProjectilePower, RelativePosition, RelativePositionRotation, RelativeRotation, RotateHead,
    SetEntityMotion, TeleportEntity,
};
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::lp_vec;
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_position_sync(
    reader: &mut WireReader<'_>,
) -> Result<EntityPositionSync, WireError> {
    Ok(EntityPositionSync {
        entity_id: reader.read_var_i32()?,
        change: read_change(reader)?,
        on_ground: reader.read_bool()?,
    })
}

pub(crate) fn write_position_sync(
    writer: &mut WireWriter,
    packet: EntityPositionSync,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    write_change(writer, packet.change)?;
    writer.write_bool(packet.on_ground)
}

pub(crate) fn read_position(reader: &mut WireReader<'_>) -> Result<RelativePosition, WireError> {
    Ok(RelativePosition {
        entity_id: reader.read_var_i32()?,
        delta_x: reader.read_i16()?,
        delta_y: reader.read_i16()?,
        delta_z: reader.read_i16()?,
        on_ground: reader.read_bool()?,
    })
}

pub(crate) fn write_position(
    writer: &mut WireWriter,
    packet: RelativePosition,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_i16(packet.delta_x)?;
    writer.write_i16(packet.delta_y)?;
    writer.write_i16(packet.delta_z)?;
    writer.write_bool(packet.on_ground)
}

pub(crate) fn read_position_rotation(
    reader: &mut WireReader<'_>,
) -> Result<RelativePositionRotation, WireError> {
    Ok(RelativePositionRotation {
        entity_id: reader.read_var_i32()?,
        delta_x: reader.read_i16()?,
        delta_y: reader.read_i16()?,
        delta_z: reader.read_i16()?,
        yaw: reader.read_i8()?,
        pitch: reader.read_i8()?,
        on_ground: reader.read_bool()?,
    })
}

pub(crate) fn write_position_rotation(
    writer: &mut WireWriter,
    packet: RelativePositionRotation,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_i16(packet.delta_x)?;
    writer.write_i16(packet.delta_y)?;
    writer.write_i16(packet.delta_z)?;
    writer.write_i8(packet.yaw)?;
    writer.write_i8(packet.pitch)?;
    writer.write_bool(packet.on_ground)
}

pub(crate) fn read_minecart(
    reader: &mut WireReader<'_>,
) -> Result<MoveMinecartAlongTrack, WireError> {
    let entity_id = reader.read_var_i32()?;
    let count = reader.read_count("minecart steps", reader.remaining())?;
    let mut steps = Vec::with_capacity(count);
    for _ in 0..count {
        steps.push(MinecartStep {
            position: read_vector(reader)?,
            motion: read_vector(reader)?,
            yaw: reader.read_i8()?,
            pitch: reader.read_i8()?,
            weight: reader.read_f32()?,
        });
    }
    Ok(MoveMinecartAlongTrack { entity_id, steps })
}

pub(crate) fn write_minecart(
    writer: &mut WireWriter,
    packet: &MoveMinecartAlongTrack,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_count(
        "minecart steps",
        packet.steps.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for step in &packet.steps {
        write_vector(writer, step.position)?;
        write_vector(writer, step.motion)?;
        writer.write_i8(step.yaw)?;
        writer.write_i8(step.pitch)?;
        writer.write_f32(step.weight)?;
    }
    Ok(())
}

pub(crate) fn read_rotation(reader: &mut WireReader<'_>) -> Result<RelativeRotation, WireError> {
    Ok(RelativeRotation {
        entity_id: reader.read_var_i32()?,
        yaw: reader.read_i8()?,
        pitch: reader.read_i8()?,
        on_ground: reader.read_bool()?,
    })
}

pub(crate) fn write_rotation(
    writer: &mut WireWriter,
    packet: RelativeRotation,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_i8(packet.yaw)?;
    writer.write_i8(packet.pitch)?;
    writer.write_bool(packet.on_ground)
}

pub(crate) fn read_head(reader: &mut WireReader<'_>) -> Result<RotateHead, WireError> {
    Ok(RotateHead {
        entity_id: reader.read_var_i32()?,
        head_yaw: reader.read_i8()?,
    })
}

pub(crate) fn write_head(writer: &mut WireWriter, packet: RotateHead) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_i8(packet.head_yaw)
}

pub(crate) fn read_motion(reader: &mut WireReader<'_>) -> Result<SetEntityMotion, WireError> {
    Ok(SetEntityMotion {
        entity_id: reader.read_var_i32()?,
        motion: lp_vec::read(reader)?,
    })
}

pub(crate) fn write_motion(
    writer: &mut WireWriter,
    packet: SetEntityMotion,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    lp_vec::write(writer, packet.motion)
}

pub(crate) fn read_teleport(reader: &mut WireReader<'_>) -> Result<TeleportEntity, WireError> {
    Ok(TeleportEntity {
        entity_id: reader.read_var_i32()?,
        change: read_change(reader)?,
        relative_flags: reader.read_i32()? as u32,
        on_ground: reader.read_bool()?,
    })
}

pub(crate) fn write_teleport(
    writer: &mut WireWriter,
    packet: TeleportEntity,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    write_change(writer, packet.change)?;
    writer.write_i32(packet.relative_flags as i32)?;
    writer.write_bool(packet.on_ground)
}

pub(crate) fn read_projectile(reader: &mut WireReader<'_>) -> Result<ProjectilePower, WireError> {
    Ok(ProjectilePower {
        entity_id: reader.read_var_i32()?,
        acceleration_power: reader.read_f64()?,
    })
}

pub(crate) fn write_projectile(
    writer: &mut WireWriter,
    packet: ProjectilePower,
) -> Result<(), WireError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_f64(packet.acceleration_power)
}

fn read_change(reader: &mut WireReader<'_>) -> Result<PositionMoveRotation, WireError> {
    Ok(PositionMoveRotation {
        position: read_vector(reader)?,
        motion: read_vector(reader)?,
        yaw: reader.read_f32()?,
        pitch: reader.read_f32()?,
    })
}

fn write_change(writer: &mut WireWriter, change: PositionMoveRotation) -> Result<(), WireError> {
    write_vector(writer, change.position)?;
    write_vector(writer, change.motion)?;
    writer.write_f32(change.yaw)?;
    writer.write_f32(change.pitch)
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
