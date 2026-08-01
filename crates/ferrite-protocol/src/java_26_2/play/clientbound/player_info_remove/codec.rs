use crate::java_26_2::play::clientbound::player_info::PlayerInfoError;
use crate::java_26_2::play::clientbound::player_info_remove::PlayerInfoRemove;
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(crate) fn read(reader: &mut WireReader<'_>) -> Result<PlayerInfoRemove, PlayerInfoError> {
    let count = reader.read_count("removed player profiles", reader.remaining() / 16)?;
    let mut profile_ids = Vec::with_capacity(count);
    for _ in 0..count {
        profile_ids.push(reader.read_u128()?);
    }
    Ok(PlayerInfoRemove { profile_ids })
}

pub(crate) fn write(
    writer: &mut WireWriter,
    packet: &PlayerInfoRemove,
) -> Result<(), PlayerInfoError> {
    writer.write_count(
        "removed player profiles",
        packet.profile_ids.len(),
        MAX_INFLATED_PACKET_LENGTH / 16,
    )?;
    for profile_id in &packet.profile_ids {
        writer.write_u128(*profile_id)?;
    }
    Ok(())
}
