use crate::java_26_2::play::clientbound::entity_effects::codec::EntityEffectsCodecError;
use crate::java_26_2::play::clientbound::entity_effects::particle_codec;
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::clientbound::particle::packet::LevelParticles;
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::registry::PlayRegistries;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<LevelParticles, EntityEffectsCodecError> {
    Ok(LevelParticles {
        override_limiter: reader.read_bool()?,
        always_show: reader.read_bool()?,
        position: Vector3 {
            x: reader.read_f64()?,
            y: reader.read_f64()?,
            z: reader.read_f64()?,
        },
        spread: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        max_speed: reader.read_f32()?,
        count: reader.read_i32()?,
        particle: particle_codec::read(reader, context)?,
    })
}

pub(crate) fn write(
    writer: &mut WireWriter,
    packet: &LevelParticles,
    registries: &PlayRegistries,
) -> Result<(), EntityEffectsCodecError> {
    writer.write_bool(packet.override_limiter)?;
    writer.write_bool(packet.always_show)?;
    writer.write_f64(packet.position.x)?;
    writer.write_f64(packet.position.y)?;
    writer.write_f64(packet.position.z)?;
    for spread in packet.spread {
        writer.write_f32(spread)?;
    }
    writer.write_f32(packet.max_speed)?;
    writer.write_i32(packet.count)?;
    particle_codec::write(writer, &packet.particle, registries)
}
