use thiserror::Error;

use crate::java_26_2::play::serverbound::entity_session::low_precision;
use crate::java_26_2::play::serverbound::entity_session::packet::{
    Attack, ClientCommand, ClientCommandKind, Interact, PickItemFromEntity, SpectatorAction,
    TeleportToEntity,
};
use crate::java_26_2::play::serverbound::packet::Hand;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EntitySessionCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("client command ordinal {0} is invalid")]
    InvalidClientCommand(i32),
}

pub fn decode_attack(reader: &mut WireReader<'_>) -> Result<Attack, EntitySessionCodecError> {
    Ok(Attack {
        target_entity_id: reader.read_var_i32()?,
    })
}

pub fn encode_attack(
    writer: &mut WireWriter,
    packet: Attack,
) -> Result<(), EntitySessionCodecError> {
    writer.write_var_i32(packet.target_entity_id)?;
    Ok(())
}

pub fn decode_client_command(
    reader: &mut WireReader<'_>,
) -> Result<ClientCommand, EntitySessionCodecError> {
    let ordinal = reader.read_var_i32()?;
    let action = ClientCommandKind::from_index(ordinal)
        .ok_or(EntitySessionCodecError::InvalidClientCommand(ordinal))?;
    Ok(ClientCommand { action })
}

pub fn encode_client_command(
    writer: &mut WireWriter,
    packet: ClientCommand,
) -> Result<(), EntitySessionCodecError> {
    writer.write_var_i32(packet.action.index())?;
    Ok(())
}

pub fn decode_interact(reader: &mut WireReader<'_>) -> Result<Interact, EntitySessionCodecError> {
    Ok(Interact {
        target_entity_id: reader.read_var_i32()?,
        hand: decode_fallback_hand(reader.read_var_i32()?),
        location: low_precision::read(reader)?,
        secondary_action: reader.read_bool()?,
    })
}

pub fn encode_interact(
    writer: &mut WireWriter,
    packet: Interact,
) -> Result<(), EntitySessionCodecError> {
    writer.write_var_i32(packet.target_entity_id)?;
    writer.write_var_i32(packet.hand.index())?;
    low_precision::write(writer, packet.location)?;
    writer.write_bool(packet.secondary_action)?;
    Ok(())
}

pub fn decode_pick(
    reader: &mut WireReader<'_>,
) -> Result<PickItemFromEntity, EntitySessionCodecError> {
    Ok(PickItemFromEntity {
        target_entity_id: reader.read_var_i32()?,
        include_data: reader.read_bool()?,
    })
}

pub fn encode_pick(
    writer: &mut WireWriter,
    packet: PickItemFromEntity,
) -> Result<(), EntitySessionCodecError> {
    writer.write_var_i32(packet.target_entity_id)?;
    writer.write_bool(packet.include_data)?;
    Ok(())
}

pub fn decode_spectator_action(
    reader: &mut WireReader<'_>,
) -> Result<SpectatorAction, EntitySessionCodecError> {
    let biased = reader.read_var_i32()?;
    Ok(SpectatorAction {
        target_entity_id: (biased != 0).then(|| biased.wrapping_sub(1)),
    })
}

pub fn encode_spectator_action(
    writer: &mut WireWriter,
    packet: SpectatorAction,
) -> Result<(), EntitySessionCodecError> {
    writer.write_var_i32(
        packet
            .target_entity_id
            .map_or(0, |entity_id| entity_id.wrapping_add(1)),
    )?;
    Ok(())
}

pub fn decode_teleport(
    reader: &mut WireReader<'_>,
) -> Result<TeleportToEntity, EntitySessionCodecError> {
    Ok(TeleportToEntity {
        target_uuid: reader.read_u128()?,
    })
}

pub fn encode_teleport(
    writer: &mut WireWriter,
    packet: TeleportToEntity,
) -> Result<(), EntitySessionCodecError> {
    writer.write_u128(packet.target_uuid)?;
    Ok(())
}

fn decode_fallback_hand(ordinal: i32) -> Hand {
    if ordinal == 1 { Hand::Off } else { Hand::Main }
}
