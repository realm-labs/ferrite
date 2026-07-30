use thiserror::Error;

use crate::java_26_2::play::clientbound::combat_look::packet::{
    EntityAnchor, LookEntity, LookPosition, PlayerCombatEnd, PlayerCombatKill, PlayerLookAt,
};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read_end(
    reader: &mut WireReader<'_>,
) -> Result<PlayerCombatEnd, CombatLookCodecError> {
    Ok(PlayerCombatEnd {
        duration: reader.read_var_i32()?,
    })
}

pub(crate) fn write_end(
    writer: &mut WireWriter,
    packet: PlayerCombatEnd,
) -> Result<(), CombatLookCodecError> {
    writer.write_var_i32(packet.duration)?;
    Ok(())
}

pub(crate) fn read_kill(
    reader: &mut WireReader<'_>,
) -> Result<PlayerCombatKill, CombatLookCodecError> {
    let player_entity_id = reader.read_var_i32()?;
    let message = TextComponentNbt::from_network_nbt(NetworkNbt::read(reader, NbtQuota::Trusted)?)?;
    Ok(PlayerCombatKill {
        player_entity_id,
        message,
    })
}

pub(crate) fn write_kill(
    writer: &mut WireWriter,
    packet: &PlayerCombatKill,
) -> Result<(), CombatLookCodecError> {
    writer.write_var_i32(packet.player_entity_id)?;
    packet.message.network_nbt().write(writer)?;
    Ok(())
}

pub(crate) fn read_look(reader: &mut WireReader<'_>) -> Result<PlayerLookAt, CombatLookCodecError> {
    let from_anchor = read_anchor(reader)?;
    let fallback = LookPosition {
        x: reader.read_f64()?,
        y: reader.read_f64()?,
        z: reader.read_f64()?,
    };
    let entity = if reader.read_bool()? {
        Some(LookEntity {
            entity_id: reader.read_var_i32()?,
            anchor: read_anchor(reader)?,
        })
    } else {
        None
    };
    Ok(PlayerLookAt {
        from_anchor,
        fallback,
        entity,
    })
}

pub(crate) fn write_look(
    writer: &mut WireWriter,
    packet: PlayerLookAt,
) -> Result<(), CombatLookCodecError> {
    write_anchor(writer, packet.from_anchor)?;
    writer.write_f64(packet.fallback.x)?;
    writer.write_f64(packet.fallback.y)?;
    writer.write_f64(packet.fallback.z)?;
    writer.write_bool(packet.entity.is_some())?;
    if let Some(entity) = packet.entity {
        writer.write_var_i32(entity.entity_id)?;
        write_anchor(writer, entity.anchor)?;
    }
    Ok(())
}

fn read_anchor(reader: &mut WireReader<'_>) -> Result<EntityAnchor, CombatLookCodecError> {
    let ordinal = reader.read_var_i32()?;
    EntityAnchor::from_ordinal(ordinal).ok_or(CombatLookCodecError::InvalidAnchor { ordinal })
}

fn write_anchor(writer: &mut WireWriter, anchor: EntityAnchor) -> Result<(), CombatLookCodecError> {
    writer.write_var_i32(anchor.ordinal())?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CombatLookCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error("entity-anchor ordinal {ordinal} is outside 0..=1")]
    InvalidAnchor { ordinal: i32 },
}
