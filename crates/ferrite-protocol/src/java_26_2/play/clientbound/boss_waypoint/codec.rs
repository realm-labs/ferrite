use thiserror::Error;

use crate::java_26_2::play::clientbound::boss_waypoint::packet::{
    BossColor, BossEvent, BossOperation, BossOverlay, TrackedWaypoint, WaypointIcon,
    WaypointIdentifier, WaypointLocation, WaypointOperation, WaypointPacket,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierReadError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const DEFAULT_UTF_LIMIT: usize = 32_767;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BossWaypointCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error("invalid waypoint style identifier: {reason}")]
    InvalidStyle { reason: String },
    #[error("boss operation ordinal {ordinal} is outside 0..=5")]
    UnknownBossOperation { ordinal: i32 },
    #[error("boss color ordinal {ordinal} is outside 0..=6")]
    UnknownBossColor { ordinal: i32 },
    #[error("boss overlay ordinal {ordinal} is outside 0..=4")]
    UnknownBossOverlay { ordinal: i32 },
    #[error("waypoint type ordinal {ordinal} is outside 0..=3")]
    UnknownWaypointType { ordinal: i32 },
}

impl From<IdentifierReadError> for BossWaypointCodecError {
    fn from(error: IdentifierReadError) -> Self {
        Self::InvalidStyle {
            reason: error.to_string(),
        }
    }
}

pub fn read_boss(reader: &mut WireReader<'_>) -> Result<BossEvent, BossWaypointCodecError> {
    let id = reader.read_u128()?;
    let operation = match reader.read_var_i32()? {
        0 => BossOperation::Add {
            name: read_component(reader)?,
            progress: reader.read_f32()?,
            color: read_color(reader)?,
            overlay: read_overlay(reader)?,
            properties: reader.read_u8()?,
        },
        1 => BossOperation::Remove,
        2 => BossOperation::UpdateProgress(reader.read_f32()?),
        3 => BossOperation::UpdateName(read_component(reader)?),
        4 => BossOperation::UpdateStyle {
            color: read_color(reader)?,
            overlay: read_overlay(reader)?,
        },
        5 => BossOperation::UpdateProperties(reader.read_u8()?),
        ordinal => return Err(BossWaypointCodecError::UnknownBossOperation { ordinal }),
    };
    Ok(BossEvent { id, operation })
}

pub fn write_boss(
    writer: &mut WireWriter,
    packet: &BossEvent,
) -> Result<(), BossWaypointCodecError> {
    writer.write_u128(packet.id)?;
    match &packet.operation {
        BossOperation::Add {
            name,
            progress,
            color,
            overlay,
            properties,
        } => {
            writer.write_var_i32(0)?;
            name.network_nbt().write(writer)?;
            writer.write_f32(*progress)?;
            writer.write_var_i32(color_ordinal(*color))?;
            writer.write_var_i32(overlay_ordinal(*overlay))?;
            writer.write_u8(*properties)?;
        }
        BossOperation::Remove => writer.write_var_i32(1)?,
        BossOperation::UpdateProgress(progress) => {
            writer.write_var_i32(2)?;
            writer.write_f32(*progress)?;
        }
        BossOperation::UpdateName(name) => {
            writer.write_var_i32(3)?;
            name.network_nbt().write(writer)?;
        }
        BossOperation::UpdateStyle { color, overlay } => {
            writer.write_var_i32(4)?;
            writer.write_var_i32(color_ordinal(*color))?;
            writer.write_var_i32(overlay_ordinal(*overlay))?;
        }
        BossOperation::UpdateProperties(properties) => {
            writer.write_var_i32(5)?;
            writer.write_u8(*properties)?;
        }
    }
    Ok(())
}

pub fn read_waypoint(
    reader: &mut WireReader<'_>,
) -> Result<WaypointPacket, BossWaypointCodecError> {
    let operation = match reader.read_var_i32()?.rem_euclid(3) {
        0 => WaypointOperation::Track,
        1 => WaypointOperation::Untrack,
        _ => WaypointOperation::Update,
    };
    let identifier = if reader.read_bool()? {
        WaypointIdentifier::Uuid(reader.read_u128()?)
    } else {
        WaypointIdentifier::String(reader.read_utf(DEFAULT_UTF_LIMIT)?.into_owned())
    };
    let style = Identifier::read(reader)?;
    let color = if reader.read_bool()? {
        Some(
            0xff00_0000
                | (u32::from(reader.read_u8()?) << 16)
                | (u32::from(reader.read_u8()?) << 8)
                | u32::from(reader.read_u8()?),
        )
    } else {
        None
    };
    let location = match reader.read_var_i32()? {
        0 => WaypointLocation::Empty,
        1 => WaypointLocation::Position {
            x: reader.read_var_i32()?,
            y: reader.read_var_i32()?,
            z: reader.read_var_i32()?,
        },
        2 => WaypointLocation::Chunk {
            x: reader.read_var_i32()?,
            z: reader.read_var_i32()?,
        },
        3 => WaypointLocation::Azimuth {
            angle: reader.read_f32()?,
        },
        ordinal => return Err(BossWaypointCodecError::UnknownWaypointType { ordinal }),
    };
    Ok(WaypointPacket {
        operation,
        waypoint: TrackedWaypoint {
            identifier,
            icon: WaypointIcon { style, color },
            location,
        },
    })
}

pub fn write_waypoint(
    writer: &mut WireWriter,
    packet: &WaypointPacket,
) -> Result<(), BossWaypointCodecError> {
    writer.write_var_i32(match packet.operation {
        WaypointOperation::Track => 0,
        WaypointOperation::Untrack => 1,
        WaypointOperation::Update => 2,
    })?;
    match &packet.waypoint.identifier {
        WaypointIdentifier::Uuid(id) => {
            writer.write_bool(true)?;
            writer.write_u128(*id)?;
        }
        WaypointIdentifier::String(id) => {
            writer.write_bool(false)?;
            writer.write_utf(id, DEFAULT_UTF_LIMIT)?;
        }
    }
    packet.waypoint.icon.style.write(writer)?;
    writer.write_bool(packet.waypoint.icon.color.is_some())?;
    if let Some(color) = packet.waypoint.icon.color {
        writer.write_u8((color >> 16) as u8)?;
        writer.write_u8((color >> 8) as u8)?;
        writer.write_u8(color as u8)?;
    }
    match packet.waypoint.location {
        WaypointLocation::Empty => writer.write_var_i32(0)?,
        WaypointLocation::Position { x, y, z } => {
            writer.write_var_i32(1)?;
            writer.write_var_i32(x)?;
            writer.write_var_i32(y)?;
            writer.write_var_i32(z)?;
        }
        WaypointLocation::Chunk { x, z } => {
            writer.write_var_i32(2)?;
            writer.write_var_i32(x)?;
            writer.write_var_i32(z)?;
        }
        WaypointLocation::Azimuth { angle } => {
            writer.write_var_i32(3)?;
            writer.write_f32(angle)?;
        }
    }
    Ok(())
}

fn read_component(reader: &mut WireReader<'_>) -> Result<TextComponentNbt, BossWaypointCodecError> {
    Ok(TextComponentNbt::from_network_nbt(NetworkNbt::read(
        reader,
        NbtQuota::Trusted,
    )?)?)
}

fn read_color(reader: &mut WireReader<'_>) -> Result<BossColor, BossWaypointCodecError> {
    match reader.read_var_i32()? {
        0 => Ok(BossColor::Pink),
        1 => Ok(BossColor::Blue),
        2 => Ok(BossColor::Red),
        3 => Ok(BossColor::Green),
        4 => Ok(BossColor::Yellow),
        5 => Ok(BossColor::Purple),
        6 => Ok(BossColor::White),
        ordinal => Err(BossWaypointCodecError::UnknownBossColor { ordinal }),
    }
}

fn read_overlay(reader: &mut WireReader<'_>) -> Result<BossOverlay, BossWaypointCodecError> {
    match reader.read_var_i32()? {
        0 => Ok(BossOverlay::Progress),
        1 => Ok(BossOverlay::Notched6),
        2 => Ok(BossOverlay::Notched10),
        3 => Ok(BossOverlay::Notched12),
        4 => Ok(BossOverlay::Notched20),
        ordinal => Err(BossWaypointCodecError::UnknownBossOverlay { ordinal }),
    }
}

const fn color_ordinal(color: BossColor) -> i32 {
    color as i32
}

const fn overlay_ordinal(overlay: BossOverlay) -> i32 {
    overlay as i32
}
