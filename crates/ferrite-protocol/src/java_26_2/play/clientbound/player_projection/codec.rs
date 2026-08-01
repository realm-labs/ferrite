use thiserror::Error;

use crate::java_26_2::play::clientbound::player_projection::packet::{
    AwardStats, Cooldown, SetExperience, SetHealth, StatisticKey,
};
use crate::java_26_2::play::registry::{
    BLOCK, CUSTOM_STAT, ENTITY_TYPE, ITEM, PlayRegistries, PlayRegistryError, STAT_TYPE,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_MAP_ENTRIES: usize = i32::MAX as usize;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlayerProjectionCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidIdentifier(#[from] IdentifierError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error("stat type {statistic_type} has no locked backing registry")]
    UnknownStatisticType { statistic_type: Identifier },
}

pub(crate) fn read_stats(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<AwardStats, PlayerProjectionCodecError> {
    let count = reader.read_count("statistics", MAX_MAP_ENTRIES)?;
    let mut values = std::collections::BTreeMap::new();
    for _ in 0..count {
        let statistic_type = registries.resolve(STAT_TYPE, reader.read_var_i32()?)?;
        let raw_value = reader.read_var_i32()?;
        let value = resolve_stat_value(registries, &statistic_type, raw_value)?;
        let amount = reader.read_var_i32()?;
        values.insert(
            StatisticKey {
                statistic_type,
                value,
            },
            amount,
        );
    }
    Ok(AwardStats { values })
}

pub(crate) fn write_stats(
    writer: &mut WireWriter,
    packet: &AwardStats,
    registries: &PlayRegistries,
) -> Result<(), PlayerProjectionCodecError> {
    writer.write_count("statistics", packet.values.len(), MAX_MAP_ENTRIES)?;
    for (statistic, value) in &packet.values {
        writer.write_var_i32(registries.raw_id(STAT_TYPE, &statistic.statistic_type)?)?;
        let backing = backing_registry(&statistic.statistic_type)?;
        writer.write_var_i32(registries.raw_id(backing, &statistic.value)?)?;
        writer.write_var_i32(*value)?;
    }
    Ok(())
}

pub(crate) fn read_cooldown(
    reader: &mut WireReader<'_>,
) -> Result<Cooldown, PlayerProjectionCodecError> {
    Ok(Cooldown {
        group: read_identifier(reader)?,
        duration_ticks: reader.read_var_i32()?,
    })
}

pub(crate) fn write_cooldown(
    writer: &mut WireWriter,
    packet: &Cooldown,
) -> Result<(), PlayerProjectionCodecError> {
    packet.group.write(writer)?;
    writer.write_var_i32(packet.duration_ticks)?;
    Ok(())
}

pub(crate) fn read_experience(
    reader: &mut WireReader<'_>,
) -> Result<SetExperience, PlayerProjectionCodecError> {
    Ok(SetExperience {
        progress: reader.read_f32()?,
        level: reader.read_var_i32()?,
        total_experience: reader.read_var_i32()?,
    })
}

pub(crate) fn write_experience(
    writer: &mut WireWriter,
    packet: SetExperience,
) -> Result<(), PlayerProjectionCodecError> {
    writer.write_f32(packet.progress)?;
    writer.write_var_i32(packet.level)?;
    writer.write_var_i32(packet.total_experience)?;
    Ok(())
}

pub(crate) fn read_health(
    reader: &mut WireReader<'_>,
) -> Result<SetHealth, PlayerProjectionCodecError> {
    Ok(SetHealth {
        health: reader.read_f32()?,
        food: reader.read_var_i32()?,
        saturation: reader.read_f32()?,
    })
}

pub(crate) fn write_health(
    writer: &mut WireWriter,
    packet: SetHealth,
) -> Result<(), PlayerProjectionCodecError> {
    writer.write_f32(packet.health)?;
    writer.write_var_i32(packet.food)?;
    writer.write_f32(packet.saturation)?;
    Ok(())
}

fn resolve_stat_value(
    registries: &PlayRegistries,
    statistic_type: &Identifier,
    raw_id: i32,
) -> Result<Identifier, PlayerProjectionCodecError> {
    let backing = backing_registry(statistic_type)?;
    match registries.resolve(backing, raw_id) {
        Ok(value) => Ok(value),
        Err(PlayRegistryError::UnknownRawId { .. }) if backing == BLOCK || backing == ITEM => {
            Ok(Identifier::parse("minecraft:air").expect("locked fallback identifier is valid"))
        }
        Err(PlayRegistryError::UnknownRawId { .. }) if backing == ENTITY_TYPE => {
            Ok(Identifier::parse("minecraft:pig").expect("locked fallback identifier is valid"))
        }
        Err(error) => Err(error.into()),
    }
}

fn backing_registry(
    statistic_type: &Identifier,
) -> Result<&'static str, PlayerProjectionCodecError> {
    match statistic_type.to_string().as_str() {
        "minecraft:mined" => Ok(BLOCK),
        "minecraft:crafted"
        | "minecraft:used"
        | "minecraft:broken"
        | "minecraft:picked_up"
        | "minecraft:dropped" => Ok(ITEM),
        "minecraft:killed" | "minecraft:killed_by" => Ok(ENTITY_TYPE),
        "minecraft:custom" => Ok(CUSTOM_STAT),
        _ => Err(PlayerProjectionCodecError::UnknownStatisticType {
            statistic_type: statistic_type.clone(),
        }),
    }
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, PlayerProjectionCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
