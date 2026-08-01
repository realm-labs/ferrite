use thiserror::Error;

use crate::java_26_2::play::clientbound::scoreboard::packet::{
    CollisionRule, DisplaySlot, NameTagVisibility, NumberFormat, ObjectiveParameters,
    ObjectiveRenderType, ResetScore, SetDisplayObjective, SetObjective, SetPlayerTeam, SetScore,
    TeamColor, TeamParameters,
};
use crate::java_26_2::play::registry::{NUMBER_FORMAT_TYPE, PlayRegistries, PlayRegistryError};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const STRING_LIMIT: usize = 32_767;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScoreboardCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error("objective render type raw ID {raw_id} is outside 0..=1")]
    InvalidRenderType { raw_id: i32 },
    #[error("number-format registry entry {identity} is not one of blank, styled, or fixed")]
    InvalidNumberFormatType { identity: Identifier },
    #[error(
        "scoreboard packet method {method} has parameters/list fields inconsistent with its codec"
    )]
    InvalidMethodShape { method: i8 },
}

pub(crate) fn read_reset(reader: &mut WireReader<'_>) -> Result<ResetScore, ScoreboardCodecError> {
    Ok(ResetScore {
        owner: read_string(reader)?,
        objective_name: read_optional(reader, read_string)?,
    })
}

pub(crate) fn write_reset(
    writer: &mut WireWriter,
    packet: &ResetScore,
) -> Result<(), ScoreboardCodecError> {
    write_string(writer, &packet.owner)?;
    write_optional(writer, packet.objective_name.as_ref(), |writer, value| {
        write_string(writer, value)
    })
}

pub(crate) fn read_display(
    reader: &mut WireReader<'_>,
) -> Result<SetDisplayObjective, ScoreboardCodecError> {
    let slot = DisplaySlot::from_fallback_id(reader.read_var_i32()?);
    let objective_name = read_string(reader)?;
    Ok(SetDisplayObjective {
        slot,
        objective_name: (!objective_name.is_empty()).then_some(objective_name),
    })
}

pub(crate) fn write_display(
    writer: &mut WireWriter,
    packet: &SetDisplayObjective,
) -> Result<(), ScoreboardCodecError> {
    writer.write_var_i32(packet.slot.id())?;
    write_string(writer, packet.objective_name.as_deref().unwrap_or(""))
}

pub(crate) fn read_objective(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<SetObjective, ScoreboardCodecError> {
    let objective_name = read_string(reader)?;
    let method = reader.read_i8()?;
    let parameters = matches!(method, 0 | 2)
        .then(|| read_objective_parameters(reader, registries))
        .transpose()?;
    Ok(SetObjective {
        objective_name,
        method,
        parameters,
    })
}

pub(crate) fn write_objective(
    writer: &mut WireWriter,
    packet: &SetObjective,
    registries: &PlayRegistries,
) -> Result<(), ScoreboardCodecError> {
    write_string(writer, &packet.objective_name)?;
    writer.write_i8(packet.method)?;
    match (matches!(packet.method, 0 | 2), &packet.parameters) {
        (true, Some(parameters)) => write_objective_parameters(writer, parameters, registries),
        (false, None) => Ok(()),
        _ => Err(ScoreboardCodecError::InvalidMethodShape {
            method: packet.method,
        }),
    }
}

pub(crate) fn read_score(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<SetScore, ScoreboardCodecError> {
    Ok(SetScore {
        owner: read_string(reader)?,
        objective_name: read_string(reader)?,
        score: reader.read_var_i32()?,
        display: read_optional(reader, read_component)?,
        number_format: read_optional(reader, |reader| read_number_format(reader, registries))?,
    })
}

pub(crate) fn write_score(
    writer: &mut WireWriter,
    packet: &SetScore,
    registries: &PlayRegistries,
) -> Result<(), ScoreboardCodecError> {
    write_string(writer, &packet.owner)?;
    write_string(writer, &packet.objective_name)?;
    writer.write_var_i32(packet.score)?;
    write_optional(writer, packet.display.as_ref(), write_component)?;
    write_optional(writer, packet.number_format.as_ref(), |writer, format| {
        write_number_format(writer, format, registries)
    })
}

pub(crate) fn read_team(
    reader: &mut WireReader<'_>,
) -> Result<SetPlayerTeam, ScoreboardCodecError> {
    let team_name = read_string(reader)?;
    let method = reader.read_i8()?;
    let parameters = matches!(method, 0 | 2)
        .then(|| read_team_parameters(reader))
        .transpose()?;
    let players = if matches!(method, 0 | 3 | 4) {
        let count = reader.read_count("team players", reader.remaining())?;
        let mut players = Vec::with_capacity(count.min(65_536));
        for _ in 0..count {
            players.push(read_string(reader)?);
        }
        players
    } else {
        Vec::new()
    };
    Ok(SetPlayerTeam {
        team_name,
        method,
        parameters,
        players,
    })
}

pub(crate) fn write_team(
    writer: &mut WireWriter,
    packet: &SetPlayerTeam,
) -> Result<(), ScoreboardCodecError> {
    write_string(writer, &packet.team_name)?;
    writer.write_i8(packet.method)?;
    match (matches!(packet.method, 0 | 2), &packet.parameters) {
        (true, Some(parameters)) => write_team_parameters(writer, parameters)?,
        (false, None) => {}
        _ => {
            return Err(ScoreboardCodecError::InvalidMethodShape {
                method: packet.method,
            });
        }
    }
    if matches!(packet.method, 0 | 3 | 4) {
        writer.write_count(
            "team players",
            packet.players.len(),
            MAX_INFLATED_PACKET_LENGTH,
        )?;
        for player in &packet.players {
            write_string(writer, player)?;
        }
    } else if !packet.players.is_empty() {
        return Err(ScoreboardCodecError::InvalidMethodShape {
            method: packet.method,
        });
    }
    Ok(())
}

fn read_objective_parameters(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<ObjectiveParameters, ScoreboardCodecError> {
    let display_name = read_component(reader)?;
    let render_type = match reader.read_var_i32()? {
        0 => ObjectiveRenderType::Integer,
        1 => ObjectiveRenderType::Hearts,
        raw_id => return Err(ScoreboardCodecError::InvalidRenderType { raw_id }),
    };
    let number_format = read_optional(reader, |reader| read_number_format(reader, registries))?;
    Ok(ObjectiveParameters {
        display_name,
        render_type,
        number_format,
    })
}

fn write_objective_parameters(
    writer: &mut WireWriter,
    parameters: &ObjectiveParameters,
    registries: &PlayRegistries,
) -> Result<(), ScoreboardCodecError> {
    write_component(writer, &parameters.display_name)?;
    writer.write_var_i32(match parameters.render_type {
        ObjectiveRenderType::Integer => 0,
        ObjectiveRenderType::Hearts => 1,
    })?;
    write_optional(
        writer,
        parameters.number_format.as_ref(),
        |writer, format| write_number_format(writer, format, registries),
    )
}

fn read_team_parameters(
    reader: &mut WireReader<'_>,
) -> Result<TeamParameters, ScoreboardCodecError> {
    let display_name = read_component(reader)?;
    let member_prefix = read_component(reader)?;
    let member_suffix = read_component(reader)?;
    let visibility = NameTagVisibility::from_fallback_id(reader.read_var_i32()?);
    let collision_rule = CollisionRule::from_fallback_id(reader.read_var_i32()?);
    let color = reader
        .read_bool()?
        .then(|| reader.read_var_i32().map(TeamColor::from_fallback_id))
        .transpose()?;
    let options = reader.read_u8()?;
    Ok(TeamParameters {
        display_name,
        member_prefix,
        member_suffix,
        visibility,
        collision_rule,
        color,
        allow_friendly_fire: options & 1 != 0,
        see_friendly_invisibles: options & 2 != 0,
    })
}

fn write_team_parameters(
    writer: &mut WireWriter,
    parameters: &TeamParameters,
) -> Result<(), ScoreboardCodecError> {
    write_component(writer, &parameters.display_name)?;
    write_component(writer, &parameters.member_prefix)?;
    write_component(writer, &parameters.member_suffix)?;
    writer.write_var_i32(parameters.visibility.id())?;
    writer.write_var_i32(parameters.collision_rule.id())?;
    writer.write_bool(parameters.color.is_some())?;
    if let Some(color) = parameters.color {
        writer.write_var_i32(color.id())?;
    }
    let options = u8::from(parameters.allow_friendly_fire)
        | (u8::from(parameters.see_friendly_invisibles) << 1);
    writer.write_u8(options)?;
    Ok(())
}

fn read_number_format(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<NumberFormat, ScoreboardCodecError> {
    let identity = registries.resolve(NUMBER_FORMAT_TYPE, reader.read_var_i32()?)?;
    match identity.to_string().as_str() {
        "minecraft:blank" => Ok(NumberFormat::Blank),
        "minecraft:styled" => Ok(NumberFormat::Styled(NetworkNbt::read(
            reader,
            NbtQuota::Trusted,
        )?)),
        "minecraft:fixed" => Ok(NumberFormat::Fixed(read_component(reader)?)),
        _ => Err(ScoreboardCodecError::InvalidNumberFormatType { identity }),
    }
}

fn write_number_format(
    writer: &mut WireWriter,
    format: &NumberFormat,
    registries: &PlayRegistries,
) -> Result<(), ScoreboardCodecError> {
    let identity = match format {
        NumberFormat::Blank => "minecraft:blank",
        NumberFormat::Styled(_) => "minecraft:styled",
        NumberFormat::Fixed(_) => "minecraft:fixed",
    };
    writer.write_var_i32(registries.raw_id(
        NUMBER_FORMAT_TYPE,
        &Identifier::parse(identity).expect("locked number-format identity is valid"),
    )?)?;
    match format {
        NumberFormat::Blank => Ok(()),
        NumberFormat::Styled(style) => style.write(writer).map_err(Into::into),
        NumberFormat::Fixed(component) => write_component(writer, component),
    }
}

fn read_component(reader: &mut WireReader<'_>) -> Result<TextComponentNbt, ScoreboardCodecError> {
    Ok(TextComponentNbt::from_network_nbt(NetworkNbt::read(
        reader,
        NbtQuota::Trusted,
    )?)?)
}

fn write_component(
    writer: &mut WireWriter,
    component: &TextComponentNbt,
) -> Result<(), ScoreboardCodecError> {
    component.network_nbt().write(writer)?;
    Ok(())
}

fn read_string(reader: &mut WireReader<'_>) -> Result<String, ScoreboardCodecError> {
    Ok(reader.read_utf(STRING_LIMIT)?.into_owned())
}

fn write_string(writer: &mut WireWriter, value: &str) -> Result<(), ScoreboardCodecError> {
    writer.write_utf(value, STRING_LIMIT)?;
    Ok(())
}

fn read_optional<T>(
    reader: &mut WireReader<'_>,
    read: impl FnOnce(&mut WireReader<'_>) -> Result<T, ScoreboardCodecError>,
) -> Result<Option<T>, ScoreboardCodecError> {
    reader.read_bool()?.then(|| read(reader)).transpose()
}

fn write_optional<T>(
    writer: &mut WireWriter,
    value: Option<&T>,
    write: impl FnOnce(&mut WireWriter, &T) -> Result<(), ScoreboardCodecError>,
) -> Result<(), ScoreboardCodecError> {
    writer.write_bool(value.is_some())?;
    value.map_or(Ok(()), |value| write(writer, value))
}
