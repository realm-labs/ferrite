use crate::java_26_2::login::profile::ProfileProperty;
use crate::java_26_2::play::block::{pack_block_position, unpack_block_position};
use crate::java_26_2::play::clientbound::entity_effects::particle_codec;
use crate::java_26_2::play::clientbound::entity_state::codec::EntityStateCodecError;
use crate::java_26_2::play::clientbound::entity_state::metadata::{
    GlobalPos, HumanoidArm, MetadataEntry, MetadataSerializer, MetadataValue, PlayerSkinModel,
    PlayerSkinPatch, ResolvableProfile, VillagerData,
};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::item::{read_optional_stack, write_optional_stack};
use crate::java_26_2::play::registry::{
    CAT_SOUND_VARIANT, CAT_VARIANT, CHICKEN_SOUND_VARIANT, CHICKEN_VARIANT, COW_SOUND_VARIANT,
    COW_VARIANT, FROG_VARIANT, PAINTING_VARIANT, PIG_SOUND_VARIANT, PIG_VARIANT, PlayRegistries,
    VILLAGER_PROFESSION, VILLAGER_TYPE, WOLF_SOUND_VARIANT, WOLF_VARIANT, ZOMBIE_NAUTILUS_VARIANT,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierReadError};
use crate::java_26_2::value::nbt::{NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_PROFILE_PROPERTIES: usize = 16;

pub(super) fn read_entries(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<Vec<MetadataEntry>, EntityStateCodecError> {
    let mut values = Vec::new();
    loop {
        let slot = reader.read_u8()?;
        if slot == u8::MAX {
            return Ok(values);
        }
        let raw_id = reader.read_var_i32()?;
        let serializer = MetadataSerializer::from_raw_id(raw_id)
            .ok_or(EntityStateCodecError::UnknownMetadataSerializer { raw_id })?;
        values.push(MetadataEntry {
            slot,
            serializer,
            value: read_value(reader, context, serializer)?,
        });
    }
}

pub(super) fn write_entries(
    writer: &mut WireWriter,
    entries: &[MetadataEntry],
    registries: &PlayRegistries,
) -> Result<(), EntityStateCodecError> {
    for entry in entries {
        if entry.slot == u8::MAX {
            return Err(EntityStateCodecError::MetadataTerminatorSlot);
        }
        let actual = entry.value.serializer();
        if actual != entry.serializer {
            return Err(EntityStateCodecError::MetadataValueMismatch {
                declared: entry.serializer,
                actual,
            });
        }
        writer.write_u8(entry.slot)?;
        writer.write_var_i32(entry.serializer.raw_id())?;
        write_value(writer, &entry.value, registries)?;
    }
    writer.write_u8(u8::MAX)?;
    Ok(())
}

fn read_value(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
    serializer: MetadataSerializer,
) -> Result<MetadataValue, EntityStateCodecError> {
    Ok(match serializer {
        MetadataSerializer::Byte => MetadataValue::Byte(reader.read_i8()?),
        MetadataSerializer::Int => MetadataValue::Int(reader.read_var_i32()?),
        MetadataSerializer::Long => MetadataValue::Long(reader.read_var_i64()?),
        MetadataSerializer::Float => MetadataValue::Float(reader.read_f32()?),
        MetadataSerializer::String => MetadataValue::String(reader.read_utf(32_767)?.into_owned()),
        MetadataSerializer::Component => MetadataValue::Component(read_component(reader)?),
        MetadataSerializer::OptionalComponent => {
            MetadataValue::OptionalComponent(read_optional(reader, read_component)?)
        }
        MetadataSerializer::ItemStack => {
            MetadataValue::ItemStack(read_optional_stack(reader, context)?)
        }
        MetadataSerializer::Boolean => MetadataValue::Boolean(reader.read_bool()?),
        MetadataSerializer::Rotations => MetadataValue::Rotations(read_float3(reader)?),
        MetadataSerializer::BlockPos => {
            MetadataValue::BlockPos(unpack_block_position(reader.read_i64()?))
        }
        MetadataSerializer::OptionalBlockPos => {
            MetadataValue::OptionalBlockPos(read_optional(reader, |reader| {
                Ok(unpack_block_position(reader.read_i64()?))
            })?)
        }
        MetadataSerializer::Direction => {
            MetadataValue::Direction(reader.read_var_i32()?.rem_euclid(6) as u8)
        }
        MetadataSerializer::OptionalLivingEntityReference => {
            MetadataValue::OptionalLivingEntityReference(read_optional(reader, |reader| {
                Ok(reader.read_u128()?)
            })?)
        }
        MetadataSerializer::BlockState => {
            let state = reader.read_var_i32()?;
            MetadataValue::BlockState((0..=32_365).contains(&state).then_some(state))
        }
        MetadataSerializer::OptionalBlockState => {
            let encoded = reader.read_var_i32()?;
            MetadataValue::OptionalBlockState(if encoded == 0 {
                None
            } else {
                let state = encoded.wrapping_sub(1);
                Some(if (0..=32_365).contains(&state) {
                    state
                } else {
                    0
                })
            })
        }
        MetadataSerializer::Particle => {
            MetadataValue::Particle(particle_codec::read(reader, context)?)
        }
        MetadataSerializer::Particles => {
            let count = reader.read_count("metadata particles", reader.remaining())?;
            let mut particles = Vec::with_capacity(count);
            for _ in 0..count {
                particles.push(particle_codec::read(reader, context)?);
            }
            MetadataValue::Particles(particles)
        }
        MetadataSerializer::VillagerData => MetadataValue::VillagerData(VillagerData {
            villager_type: context
                .registries
                .resolve(VILLAGER_TYPE, reader.read_var_i32()?)?,
            profession: context
                .registries
                .resolve(VILLAGER_PROFESSION, reader.read_var_i32()?)?,
            level: reader.read_var_i32()?,
        }),
        MetadataSerializer::OptionalUnsignedInt => {
            let encoded = reader.read_var_i32()?;
            MetadataValue::OptionalUnsignedInt((encoded != 0).then_some(encoded.wrapping_sub(1)))
        }
        MetadataSerializer::Pose => {
            let raw = reader.read_var_i32()?;
            MetadataValue::Pose(if (0..=17).contains(&raw) {
                raw as u8
            } else {
                0
            })
        }
        serializer if holder_registry(serializer).is_some() => MetadataValue::Holder {
            serializer,
            identity: context.registries.resolve(
                holder_registry(serializer).expect("guarded holder serializer"),
                reader.read_var_i32()?,
            )?,
        },
        MetadataSerializer::OptionalGlobalPos => {
            MetadataValue::OptionalGlobalPos(read_optional(reader, read_global_pos)?)
        }
        serializer @ (MetadataSerializer::SnifferState
        | MetadataSerializer::ArmadilloState
        | MetadataSerializer::CopperGolemState
        | MetadataSerializer::WeatheringCopperState) => MetadataValue::EnumState {
            serializer,
            value: normalize_enum(serializer, reader.read_var_i32()?),
        },
        MetadataSerializer::Vector3 => MetadataValue::Vector3(read_float3(reader)?),
        MetadataSerializer::Quaternion => MetadataValue::Quaternion([
            reader.read_f32()?,
            reader.read_f32()?,
            reader.read_f32()?,
            reader.read_f32()?,
        ]),
        MetadataSerializer::ResolvableProfile => {
            MetadataValue::ResolvableProfile(read_profile(reader)?)
        }
        MetadataSerializer::HumanoidArm => {
            MetadataValue::HumanoidArm(if reader.read_var_i32()? == 1 {
                HumanoidArm::Right
            } else {
                HumanoidArm::Left
            })
        }
        _ => {
            return Err(EntityStateCodecError::InvalidHolderSerializer { serializer });
        }
    })
}

fn write_value(
    writer: &mut WireWriter,
    value: &MetadataValue,
    registries: &PlayRegistries,
) -> Result<(), EntityStateCodecError> {
    match value {
        MetadataValue::Byte(value) => writer.write_i8(*value)?,
        MetadataValue::Int(value) => writer.write_var_i32(*value)?,
        MetadataValue::Long(value) => writer.write_var_i64(*value)?,
        MetadataValue::Float(value) => writer.write_f32(*value)?,
        MetadataValue::String(value) => writer.write_utf(value, 32_767)?,
        MetadataValue::Component(value) => value.network_nbt().write(writer)?,
        MetadataValue::OptionalComponent(value) => {
            write_optional(writer, value.as_ref(), |writer, value| {
                value.network_nbt().write(writer).map_err(Into::into)
            })?;
        }
        MetadataValue::ItemStack(value) => write_optional_stack(writer, value, registries)?,
        MetadataValue::Boolean(value) => writer.write_bool(*value)?,
        MetadataValue::Rotations(value) | MetadataValue::Vector3(value) => {
            write_float3(writer, *value)?;
        }
        MetadataValue::BlockPos(value) => writer.write_i64(pack_block_position(*value))?,
        MetadataValue::OptionalBlockPos(value) => {
            write_optional(writer, value.as_ref(), |writer, value| {
                writer
                    .write_i64(pack_block_position(*value))
                    .map_err(Into::into)
            })?;
        }
        MetadataValue::Direction(value) => writer.write_var_i32(i32::from(*value % 6))?,
        MetadataValue::OptionalLivingEntityReference(value) => {
            write_optional(writer, value.as_ref(), |writer, value| {
                writer.write_u128(*value).map_err(Into::into)
            })?;
        }
        MetadataValue::BlockState(value) => {
            if let Some(state) = value {
                validate_block_state(*state)?;
            }
            writer.write_var_i32(value.unwrap_or(-1))?;
        }
        MetadataValue::OptionalBlockState(value) => {
            writer.write_var_i32(match value {
                Some(state) => {
                    validate_block_state(*state)?;
                    state.wrapping_add(1)
                }
                None => 0,
            })?;
        }
        MetadataValue::Particle(value) => particle_codec::write(writer, value, registries)?,
        MetadataValue::Particles(values) => {
            writer.write_count(
                "metadata particles",
                values.len(),
                MAX_INFLATED_PACKET_LENGTH,
            )?;
            for value in values {
                particle_codec::write(writer, value, registries)?;
            }
        }
        MetadataValue::VillagerData(value) => {
            writer.write_var_i32(registries.raw_id(VILLAGER_TYPE, &value.villager_type)?)?;
            writer.write_var_i32(registries.raw_id(VILLAGER_PROFESSION, &value.profession)?)?;
            writer.write_var_i32(value.level)?;
        }
        MetadataValue::OptionalUnsignedInt(value) => {
            writer.write_var_i32(value.map_or(0, |value| value.wrapping_add(1)))?;
        }
        MetadataValue::Pose(value) => writer.write_var_i32(i32::from(*value))?,
        MetadataValue::Holder {
            serializer,
            identity,
        } => {
            let registry = holder_registry(*serializer).ok_or(
                EntityStateCodecError::InvalidHolderSerializer {
                    serializer: *serializer,
                },
            )?;
            writer.write_var_i32(registries.raw_id(registry, identity)?)?;
        }
        MetadataValue::OptionalGlobalPos(value) => {
            write_optional(writer, value.as_ref(), write_global_pos)?;
        }
        MetadataValue::EnumState { serializer, value } => {
            validate_enum(*serializer, *value)?;
            writer.write_var_i32(i32::from(*value))?;
        }
        MetadataValue::Quaternion(value) => {
            for component in value {
                writer.write_f32(*component)?;
            }
        }
        MetadataValue::ResolvableProfile(value) => write_profile(writer, value)?,
        MetadataValue::HumanoidArm(value) => {
            writer.write_var_i32(i32::from(*value == HumanoidArm::Right))?;
        }
    }
    Ok(())
}

fn read_component(reader: &mut WireReader<'_>) -> Result<TextComponentNbt, EntityStateCodecError> {
    Ok(TextComponentNbt::from_network_nbt(NetworkNbt::read(
        reader,
        NbtQuota::Trusted,
    )?)?)
}

fn read_optional<T>(
    reader: &mut WireReader<'_>,
    read: impl FnOnce(&mut WireReader<'_>) -> Result<T, EntityStateCodecError>,
) -> Result<Option<T>, EntityStateCodecError> {
    if reader.read_bool()? {
        read(reader).map(Some)
    } else {
        Ok(None)
    }
}

fn write_optional<T>(
    writer: &mut WireWriter,
    value: Option<&T>,
    write: impl FnOnce(&mut WireWriter, &T) -> Result<(), EntityStateCodecError>,
) -> Result<(), EntityStateCodecError> {
    writer.write_bool(value.is_some())?;
    if let Some(value) = value {
        write(writer, value)?;
    }
    Ok(())
}

fn read_float3(reader: &mut WireReader<'_>) -> Result<[f32; 3], EntityStateCodecError> {
    Ok([reader.read_f32()?, reader.read_f32()?, reader.read_f32()?])
}

fn write_float3(writer: &mut WireWriter, value: [f32; 3]) -> Result<(), EntityStateCodecError> {
    for component in value {
        writer.write_f32(component)?;
    }
    Ok(())
}

fn read_global_pos(reader: &mut WireReader<'_>) -> Result<GlobalPos, EntityStateCodecError> {
    Ok(GlobalPos {
        dimension: read_identifier(reader)?,
        position: unpack_block_position(reader.read_i64()?),
    })
}

fn write_global_pos(
    writer: &mut WireWriter,
    value: &GlobalPos,
) -> Result<(), EntityStateCodecError> {
    value.dimension.write(writer)?;
    writer.write_i64(pack_block_position(value.position))?;
    Ok(())
}

fn read_profile(reader: &mut WireReader<'_>) -> Result<ResolvableProfile, EntityStateCodecError> {
    let resolved = reader.read_bool()?;
    let resolved_identity = if resolved {
        Some((reader.read_u128()?, reader.read_utf(16)?.into_owned()))
    } else {
        None
    };
    let partial_identity = if resolved {
        None
    } else {
        Some((
            read_optional(reader, |reader| Ok(reader.read_utf(16)?.into_owned()))?,
            read_optional(reader, |reader| Ok(reader.read_u128()?))?,
        ))
    };
    let count = reader.read_count("profile properties", MAX_PROFILE_PROPERTIES)?;
    let mut properties = Vec::with_capacity(count);
    for _ in 0..count {
        properties.push(read_property(reader)?);
    }
    let skin = read_skin(reader)?;
    Ok(if let Some((uuid, name)) = resolved_identity {
        ResolvableProfile::Resolved {
            uuid,
            name,
            properties,
            skin,
        }
    } else {
        let (name, uuid) = partial_identity.expect("partial profile identity is present");
        ResolvableProfile::Partial {
            name,
            uuid,
            properties,
            skin,
        }
    })
}

fn write_profile(
    writer: &mut WireWriter,
    profile: &ResolvableProfile,
) -> Result<(), EntityStateCodecError> {
    let (properties, skin) = match profile {
        ResolvableProfile::Resolved {
            uuid,
            name,
            properties,
            skin,
        } => {
            writer.write_bool(true)?;
            writer.write_u128(*uuid)?;
            writer.write_utf(name, 16)?;
            (properties, skin)
        }
        ResolvableProfile::Partial {
            name,
            uuid,
            properties,
            skin,
        } => {
            writer.write_bool(false)?;
            write_optional(writer, name.as_ref(), |writer, name| {
                writer.write_utf(name, 16).map_err(Into::into)
            })?;
            write_optional(writer, uuid.as_ref(), |writer, uuid| {
                writer.write_u128(*uuid).map_err(Into::into)
            })?;
            (properties, skin)
        }
    };
    writer.write_count(
        "profile properties",
        properties.len(),
        MAX_PROFILE_PROPERTIES,
    )?;
    for property in properties {
        write_property(writer, property)?;
    }
    write_skin(writer, skin)
}

fn read_property(reader: &mut WireReader<'_>) -> Result<ProfileProperty, EntityStateCodecError> {
    Ok(ProfileProperty {
        name: reader.read_utf(64)?.into_owned(),
        value: reader.read_utf(32_767)?.into_owned(),
        signature: read_optional(reader, |reader| Ok(reader.read_utf(1_024)?.into_owned()))?,
    })
}

fn write_property(
    writer: &mut WireWriter,
    property: &ProfileProperty,
) -> Result<(), EntityStateCodecError> {
    writer.write_utf(&property.name, 64)?;
    writer.write_utf(&property.value, 32_767)?;
    write_optional(writer, property.signature.as_ref(), |writer, signature| {
        writer.write_utf(signature, 1_024).map_err(Into::into)
    })
}

fn read_skin(reader: &mut WireReader<'_>) -> Result<PlayerSkinPatch, EntityStateCodecError> {
    Ok(PlayerSkinPatch {
        body: read_optional(reader, read_identifier)?,
        cape: read_optional(reader, read_identifier)?,
        elytra: read_optional(reader, read_identifier)?,
        model: read_optional(reader, |reader| {
            Ok(if reader.read_bool()? {
                PlayerSkinModel::Slim
            } else {
                PlayerSkinModel::Wide
            })
        })?,
    })
}

fn write_skin(
    writer: &mut WireWriter,
    skin: &PlayerSkinPatch,
) -> Result<(), EntityStateCodecError> {
    for texture in [&skin.body, &skin.cape, &skin.elytra] {
        write_optional(writer, texture.as_ref(), |writer, identity| {
            identity.write(writer).map_err(Into::into)
        })?;
    }
    write_optional(writer, skin.model.as_ref(), |writer, model| {
        writer
            .write_bool(*model == PlayerSkinModel::Slim)
            .map_err(Into::into)
    })
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, EntityStateCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

fn holder_registry(serializer: MetadataSerializer) -> Option<&'static str> {
    Some(match serializer {
        MetadataSerializer::CatVariant => CAT_VARIANT,
        MetadataSerializer::CatSoundVariant => CAT_SOUND_VARIANT,
        MetadataSerializer::CowVariant => COW_VARIANT,
        MetadataSerializer::CowSoundVariant => COW_SOUND_VARIANT,
        MetadataSerializer::WolfVariant => WOLF_VARIANT,
        MetadataSerializer::WolfSoundVariant => WOLF_SOUND_VARIANT,
        MetadataSerializer::FrogVariant => FROG_VARIANT,
        MetadataSerializer::PigVariant => PIG_VARIANT,
        MetadataSerializer::PigSoundVariant => PIG_SOUND_VARIANT,
        MetadataSerializer::ChickenVariant => CHICKEN_VARIANT,
        MetadataSerializer::ChickenSoundVariant => CHICKEN_SOUND_VARIANT,
        MetadataSerializer::ZombieNautilusVariant => ZOMBIE_NAUTILUS_VARIANT,
        MetadataSerializer::PaintingVariant => PAINTING_VARIANT,
        _ => return None,
    })
}

fn normalize_enum(serializer: MetadataSerializer, raw: i32) -> u8 {
    match serializer {
        MetadataSerializer::WeatheringCopperState => raw.clamp(0, 3) as u8,
        MetadataSerializer::SnifferState if (0..=6).contains(&raw) => raw as u8,
        MetadataSerializer::ArmadilloState if (0..=3).contains(&raw) => raw as u8,
        MetadataSerializer::CopperGolemState if (0..=4).contains(&raw) => raw as u8,
        _ => 0,
    }
}

fn validate_enum(serializer: MetadataSerializer, value: u8) -> Result<(), EntityStateCodecError> {
    let valid = match serializer {
        MetadataSerializer::SnifferState => value <= 6,
        MetadataSerializer::ArmadilloState | MetadataSerializer::WeatheringCopperState => {
            value <= 3
        }
        MetadataSerializer::CopperGolemState => value <= 4,
        _ => {
            return Err(EntityStateCodecError::InvalidEnumStateSerializer { serializer });
        }
    };
    if valid {
        Ok(())
    } else {
        Err(EntityStateCodecError::InvalidMetadataEnumState { serializer, value })
    }
}

fn validate_block_state(state: i32) -> Result<(), EntityStateCodecError> {
    if (0..=32_365).contains(&state) {
        Ok(())
    } else {
        Err(EntityStateCodecError::InvalidMetadataBlockState { state })
    }
}
