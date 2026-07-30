use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_effects::packet::{
    Explosion, ExplosionParticle, RemoveMobEffect, SoundEventHolder, UpdateMobEffect,
};
use crate::java_26_2::play::clientbound::entity_effects::particle_codec;
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::item::ItemCodecError;
use crate::java_26_2::play::registry::{
    MOB_EFFECT, PlayRegistries, PlayRegistryError, SOUND_EVENT,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_explosion(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<Explosion, EntityEffectsCodecError> {
    let center = read_vector(reader)?;
    let radius = reader.read_f32()?;
    let block_count = reader.read_i32()?;
    let knockback = if reader.read_bool()? {
        Some(read_vector(reader)?)
    } else {
        None
    };
    let particle = particle_codec::read(reader, context)?;
    let sound = read_sound(reader, context.registries)?;
    let count = reader.read_count("explosion particle recipes", reader.remaining())?;
    let mut block_particles = Vec::with_capacity(count);
    let mut total_weight = 0_i32;
    for _ in 0..count {
        let particle = particle_codec::read(reader, context)?;
        let scaling = reader.read_f32()?;
        let speed = reader.read_f32()?;
        let weight = reader.read_var_i32()?;
        if weight < 0 {
            return Err(EntityEffectsCodecError::NegativeParticleWeight { weight });
        }
        total_weight = total_weight
            .checked_add(weight)
            .ok_or(EntityEffectsCodecError::ParticleWeightOverflow)?;
        block_particles.push(ExplosionParticle {
            particle,
            scaling,
            speed,
            weight,
        });
    }
    Ok(Explosion {
        center,
        radius,
        block_count,
        knockback,
        particle,
        sound,
        block_particles,
    })
}

pub(crate) fn write_explosion(
    writer: &mut WireWriter,
    packet: &Explosion,
    registries: &PlayRegistries,
) -> Result<(), EntityEffectsCodecError> {
    write_vector(writer, packet.center)?;
    writer.write_f32(packet.radius)?;
    writer.write_i32(packet.block_count)?;
    writer.write_bool(packet.knockback.is_some())?;
    if let Some(knockback) = packet.knockback {
        write_vector(writer, knockback)?;
    }
    particle_codec::write(writer, &packet.particle, registries)?;
    write_sound(writer, &packet.sound, registries)?;
    writer.write_count(
        "explosion particle recipes",
        packet.block_particles.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    let mut total_weight = 0_i32;
    for recipe in &packet.block_particles {
        if recipe.weight < 0 {
            return Err(EntityEffectsCodecError::NegativeParticleWeight {
                weight: recipe.weight,
            });
        }
        total_weight = total_weight
            .checked_add(recipe.weight)
            .ok_or(EntityEffectsCodecError::ParticleWeightOverflow)?;
        particle_codec::write(writer, &recipe.particle, registries)?;
        writer.write_f32(recipe.scaling)?;
        writer.write_f32(recipe.speed)?;
        writer.write_var_i32(recipe.weight)?;
    }
    Ok(())
}

pub(crate) fn read_remove(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<RemoveMobEffect, EntityEffectsCodecError> {
    Ok(RemoveMobEffect {
        entity_id: reader.read_var_i32()?,
        effect: registries.resolve(MOB_EFFECT, reader.read_var_i32()?)?,
    })
}

pub(crate) fn write_remove(
    writer: &mut WireWriter,
    packet: &RemoveMobEffect,
    registries: &PlayRegistries,
) -> Result<(), EntityEffectsCodecError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_var_i32(registries.raw_id(MOB_EFFECT, &packet.effect)?)?;
    Ok(())
}

pub(crate) fn read_update(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<UpdateMobEffect, EntityEffectsCodecError> {
    Ok(UpdateMobEffect {
        entity_id: reader.read_var_i32()?,
        effect: registries.resolve(MOB_EFFECT, reader.read_var_i32()?)?,
        amplifier: reader.read_var_i32()?,
        duration: reader.read_var_i32()?,
        flags: reader.read_u8()?,
    })
}

pub(crate) fn write_update(
    writer: &mut WireWriter,
    packet: &UpdateMobEffect,
    registries: &PlayRegistries,
) -> Result<(), EntityEffectsCodecError> {
    writer.write_var_i32(packet.entity_id)?;
    writer.write_var_i32(registries.raw_id(MOB_EFFECT, &packet.effect)?)?;
    writer.write_var_i32(packet.amplifier)?;
    writer.write_var_i32(packet.duration)?;
    writer.write_u8(packet.flags)?;
    Ok(())
}

fn read_sound(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<SoundEventHolder, EntityEffectsCodecError> {
    let holder = reader.read_var_i32()?;
    if holder == 0 {
        let identity = read_identifier(reader)?;
        let fixed_range = if reader.read_bool()? {
            Some(reader.read_f32()?)
        } else {
            None
        };
        Ok(SoundEventHolder::Direct {
            identity,
            fixed_range,
        })
    } else {
        let raw_id = holder.wrapping_sub(1);
        Ok(SoundEventHolder::Registered(
            registries.resolve(SOUND_EVENT, raw_id)?,
        ))
    }
}

fn write_sound(
    writer: &mut WireWriter,
    sound: &SoundEventHolder,
    registries: &PlayRegistries,
) -> Result<(), EntityEffectsCodecError> {
    match sound {
        SoundEventHolder::Direct {
            identity,
            fixed_range,
        } => {
            writer.write_var_i32(0)?;
            identity.write(writer)?;
            writer.write_bool(fixed_range.is_some())?;
            if let Some(range) = fixed_range {
                writer.write_f32(*range)?;
            }
        }
        SoundEventHolder::Registered(identity) => {
            let raw_id = registries.raw_id(SOUND_EVENT, identity)?;
            writer.write_var_i32(
                raw_id
                    .checked_add(1)
                    .ok_or(EntityEffectsCodecError::SoundHolderOverflow)?,
            )?;
        }
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
    writer.write_f64(vector.z)?;
    Ok(())
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, EntityEffectsCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EntityEffectsCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    Item(#[from] ItemCodecError),
    #[error("particle type raw ID {raw_type} is outside 0..=124")]
    UnknownParticleType { raw_type: i32 },
    #[error("particle type raw ID {raw_type} has mismatched options")]
    ParticleOptionsMismatch { raw_type: i32 },
    #[error("block-state raw ID {state} is outside 0..=32365")]
    InvalidBlockState { state: i32 },
    #[error("position-source type raw ID {raw_type} is outside 0..=1")]
    UnknownPositionSourceType { raw_type: i32 },
    #[error("explosion particle weight {weight} is negative")]
    NegativeParticleWeight { weight: i32 },
    #[error("explosion particle total weight exceeds signed int")]
    ParticleWeightOverflow,
    #[error("registered sound holder raw ID cannot be incremented")]
    SoundHolderOverflow,
}
