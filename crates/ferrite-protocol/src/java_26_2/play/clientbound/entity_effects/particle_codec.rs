use crate::java_26_2::play::block::{pack_block_position, unpack_block_position};
use crate::java_26_2::play::clientbound::entity_effects::codec::EntityEffectsCodecError;
use crate::java_26_2::play::clientbound::entity_effects::particle::{
    PARTICLE_TYPE_COUNT, Particle, ParticleOptions, ParticleVector, PositionSource,
};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::item::{read_stack_template, write_stack_template};
use crate::java_26_2::play::registry::PlayRegistries;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<Particle, EntityEffectsCodecError> {
    let raw_type = reader.read_var_i32()?;
    if !(0..PARTICLE_TYPE_COUNT).contains(&raw_type) {
        return Err(EntityEffectsCodecError::UnknownParticleType { raw_type });
    }
    let options = match raw_type {
        1 | 2 | 36 | 118 | 122 => {
            let state = reader.read_var_i32()?;
            validate_block_state(state)?;
            ParticleOptions::BlockState(state)
        }
        7 | 10 => ParticleOptions::Geyser {
            water_blocks: reader.read_i32()?,
        },
        8 | 9 => ParticleOptions::GeyserBase {
            water_blocks: reader.read_i32()?,
            burst_impulse_base: reader.read_f32()?,
        },
        15 => ParticleOptions::Power(reader.read_f32()?),
        21 => ParticleOptions::Dust {
            color: reader.read_i32()?,
            scale: clamp_scale(reader.read_f32()?),
        },
        22 => ParticleOptions::DustTransition {
            from_color: reader.read_i32()?,
            to_color: reader.read_i32()?,
            scale: clamp_scale(reader.read_f32()?),
        },
        23 | 53 => ParticleOptions::Spell {
            color: reader.read_i32()?,
            power: reader.read_f32()?,
        },
        28 | 43 | 49 => ParticleOptions::Color(reader.read_i32()?),
        45 => ParticleOptions::Power(reader.read_f32()?),
        54 => ParticleOptions::Item(read_stack_template(reader, context)?),
        55 => ParticleOptions::Vibration {
            destination: read_position_source(reader)?,
            arrival_ticks: reader.read_var_i32()?,
        },
        56 => ParticleOptions::Trail {
            target: ParticleVector {
                x: reader.read_f64()?,
                y: reader.read_f64()?,
                z: reader.read_f64()?,
            },
            color: reader.read_i32()?,
            duration: reader.read_var_i32()?,
        },
        112 => ParticleOptions::Shriek(reader.read_var_i32()?),
        _ => ParticleOptions::Simple,
    };
    Ok(Particle { raw_type, options })
}

pub(crate) fn write(
    writer: &mut WireWriter,
    particle: &Particle,
    registries: &PlayRegistries,
) -> Result<(), EntityEffectsCodecError> {
    if !(0..PARTICLE_TYPE_COUNT).contains(&particle.raw_type) {
        return Err(EntityEffectsCodecError::UnknownParticleType {
            raw_type: particle.raw_type,
        });
    }
    writer.write_var_i32(particle.raw_type)?;
    match (particle.raw_type, &particle.options) {
        (1 | 2 | 36 | 118 | 122, ParticleOptions::BlockState(state)) => {
            validate_block_state(*state)?;
            writer.write_var_i32(*state)?;
        }
        (7 | 10, ParticleOptions::Geyser { water_blocks }) => {
            writer.write_i32(*water_blocks)?;
        }
        (
            8 | 9,
            ParticleOptions::GeyserBase {
                water_blocks,
                burst_impulse_base,
            },
        ) => {
            writer.write_i32(*water_blocks)?;
            writer.write_f32(*burst_impulse_base)?;
        }
        (15 | 45, ParticleOptions::Power(value)) => writer.write_f32(*value)?,
        (21, ParticleOptions::Dust { color, scale }) => {
            writer.write_i32(*color)?;
            writer.write_f32(clamp_scale(*scale))?;
        }
        (
            22,
            ParticleOptions::DustTransition {
                from_color,
                to_color,
                scale,
            },
        ) => {
            writer.write_i32(*from_color)?;
            writer.write_i32(*to_color)?;
            writer.write_f32(clamp_scale(*scale))?;
        }
        (23 | 53, ParticleOptions::Spell { color, power }) => {
            writer.write_i32(*color)?;
            writer.write_f32(*power)?;
        }
        (28 | 43 | 49, ParticleOptions::Color(color)) => writer.write_i32(*color)?,
        (54, ParticleOptions::Item(item)) => write_stack_template(writer, item, registries)?,
        (
            55,
            ParticleOptions::Vibration {
                destination,
                arrival_ticks,
            },
        ) => {
            write_position_source(writer, destination)?;
            writer.write_var_i32(*arrival_ticks)?;
        }
        (
            56,
            ParticleOptions::Trail {
                target,
                color,
                duration,
            },
        ) => {
            writer.write_f64(target.x)?;
            writer.write_f64(target.y)?;
            writer.write_f64(target.z)?;
            writer.write_i32(*color)?;
            writer.write_var_i32(*duration)?;
        }
        (112, ParticleOptions::Shriek(delay)) => writer.write_var_i32(*delay)?,
        (raw_type, ParticleOptions::Simple) if !option_bearing(raw_type) => {}
        _ => {
            return Err(EntityEffectsCodecError::ParticleOptionsMismatch {
                raw_type: particle.raw_type,
            });
        }
    }
    Ok(())
}

const fn option_bearing(raw_type: i32) -> bool {
    matches!(
        raw_type,
        1 | 2
            | 7
            | 8
            | 9
            | 10
            | 15
            | 21
            | 22
            | 23
            | 28
            | 36
            | 43
            | 45
            | 49
            | 53
            | 54
            | 55
            | 56
            | 112
            | 118
            | 122
    )
}

fn read_position_source(
    reader: &mut WireReader<'_>,
) -> Result<PositionSource, EntityEffectsCodecError> {
    match reader.read_var_i32()? {
        0 => Ok(PositionSource::Block(unpack_block_position(
            reader.read_i64()?,
        ))),
        1 => Ok(PositionSource::Entity {
            entity_id: reader.read_var_i32()?,
            y_offset: reader.read_f32()?,
        }),
        raw_type => Err(EntityEffectsCodecError::UnknownPositionSourceType { raw_type }),
    }
}

fn write_position_source(
    writer: &mut WireWriter,
    source: &PositionSource,
) -> Result<(), EntityEffectsCodecError> {
    match source {
        PositionSource::Block(position) => {
            writer.write_var_i32(0)?;
            writer.write_i64(pack_block_position(*position))?;
        }
        PositionSource::Entity {
            entity_id,
            y_offset,
        } => {
            writer.write_var_i32(1)?;
            writer.write_var_i32(*entity_id)?;
            writer.write_f32(*y_offset)?;
        }
    }
    Ok(())
}

fn validate_block_state(state: i32) -> Result<(), EntityEffectsCodecError> {
    if (0..=32_365).contains(&state) {
        Ok(())
    } else {
        Err(EntityEffectsCodecError::InvalidBlockState { state })
    }
}

fn clamp_scale(scale: f32) -> f32 {
    scale.clamp(0.01, 4.0)
}
