//! Field codecs for the initial Play projection packets.

use std::collections::{BTreeMap, BTreeSet};

use crate::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, read_common_spawn, read_identifier, write_common_spawn,
};
use crate::java_26_2::play::clientbound::packet::{
    BorderInitialization, ClockState, PlayLogin, PlayerPosition, SetTime, Vector3,
};
use crate::java_26_2::play::registry::{PlayRegistries, WORLD_CLOCK};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub(super) fn read_border(
    reader: &mut WireReader<'_>,
) -> Result<BorderInitialization, PlayClientboundCodecError> {
    Ok(BorderInitialization {
        center_x: reader.read_f64()?,
        center_z: reader.read_f64()?,
        old_size: reader.read_f64()?,
        new_size: reader.read_f64()?,
        lerp_millis: reader.read_var_i64()?,
        absolute_maximum: reader.read_var_i32()?,
        warning_blocks: reader.read_var_i32()?,
        warning_time: reader.read_var_i32()?,
    })
}

pub(super) fn write_border(
    writer: &mut WireWriter,
    border: &BorderInitialization,
) -> Result<(), PlayClientboundCodecError> {
    writer.write_f64(border.center_x)?;
    writer.write_f64(border.center_z)?;
    writer.write_f64(border.old_size)?;
    writer.write_f64(border.new_size)?;
    writer.write_var_i64(border.lerp_millis)?;
    writer.write_var_i32(border.absolute_maximum)?;
    writer.write_var_i32(border.warning_blocks)?;
    writer.write_var_i32(border.warning_time)?;
    Ok(())
}

pub(super) fn read_login(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<PlayLogin, PlayClientboundCodecError> {
    let player_entity_id = reader.read_i32()?;
    let hardcore = reader.read_bool()?;
    let level_count = reader.read_count("play levels", reader.remaining())?;
    let mut levels = BTreeSet::new();
    for _ in 0..level_count {
        levels.insert(read_identifier(reader)?);
    }
    let max_players = reader.read_var_i32()?;
    let chunk_radius = reader.read_var_i32()?;
    let simulation_distance = reader.read_var_i32()?;
    let reduced_debug_info = reader.read_bool()?;
    let show_death_screen = reader.read_bool()?;
    let limited_crafting = reader.read_bool()?;
    let spawn = read_common_spawn(reader, registries)?;
    let online_mode = reader.read_bool()?;
    let enforces_secure_chat = reader.read_bool()?;
    Ok(PlayLogin {
        player_entity_id,
        hardcore,
        levels,
        max_players,
        chunk_radius,
        simulation_distance,
        reduced_debug_info,
        show_death_screen,
        limited_crafting,
        spawn,
        online_mode,
        enforces_secure_chat,
    })
}

pub(super) fn write_login(
    writer: &mut WireWriter,
    login: &PlayLogin,
    registries: &PlayRegistries,
) -> Result<(), PlayClientboundCodecError> {
    writer.write_i32(login.player_entity_id)?;
    writer.write_bool(login.hardcore)?;
    writer.write_count(
        "play levels",
        login.levels.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for level in &login.levels {
        level.write(writer)?;
    }
    writer.write_var_i32(login.max_players)?;
    writer.write_var_i32(login.chunk_radius)?;
    writer.write_var_i32(login.simulation_distance)?;
    writer.write_bool(login.reduced_debug_info)?;
    writer.write_bool(login.show_death_screen)?;
    writer.write_bool(login.limited_crafting)?;
    write_common_spawn(writer, &login.spawn, registries)?;
    writer.write_bool(login.online_mode)?;
    writer.write_bool(login.enforces_secure_chat)?;
    Ok(())
}

pub(super) fn read_position(
    reader: &mut WireReader<'_>,
) -> Result<PlayerPosition, PlayClientboundCodecError> {
    Ok(PlayerPosition {
        teleport_id: reader.read_var_i32()?,
        position: read_vector(reader)?,
        motion: read_vector(reader)?,
        yaw: reader.read_f32()?,
        pitch: reader.read_f32()?,
        relative_flags: reader.read_i32()? as u32,
    })
}

pub(super) fn write_position(
    writer: &mut WireWriter,
    position: &PlayerPosition,
) -> Result<(), PlayClientboundCodecError> {
    writer.write_var_i32(position.teleport_id)?;
    write_vector(writer, position.position)?;
    write_vector(writer, position.motion)?;
    writer.write_f32(position.yaw)?;
    writer.write_f32(position.pitch)?;
    writer.write_i32(position.relative_flags as i32)?;
    Ok(())
}

pub(super) fn read_vector(
    reader: &mut WireReader<'_>,
) -> Result<Vector3, PlayClientboundCodecError> {
    Ok(Vector3 {
        x: reader.read_f64()?,
        y: reader.read_f64()?,
        z: reader.read_f64()?,
    })
}

pub(super) fn write_vector(
    writer: &mut WireWriter,
    vector: Vector3,
) -> Result<(), PlayClientboundCodecError> {
    writer.write_f64(vector.x)?;
    writer.write_f64(vector.y)?;
    writer.write_f64(vector.z)?;
    Ok(())
}

pub(super) fn read_time(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<SetTime, PlayClientboundCodecError> {
    let game_time = reader.read_i64()?;
    let count = reader.read_count("world clocks", reader.remaining())?;
    let mut clocks = BTreeMap::new();
    for _ in 0..count {
        let clock = registries.resolve(WORLD_CLOCK, reader.read_var_i32()?)?;
        let state = ClockState {
            total_ticks: reader.read_var_i64()?,
            partial_tick: reader.read_f32()?,
            rate: reader.read_f32()?,
        };
        if clocks.insert(clock.clone(), state).is_some() {
            return Err(PlayClientboundCodecError::DuplicateClock { clock });
        }
    }
    Ok(SetTime { game_time, clocks })
}

pub(super) fn write_time(
    writer: &mut WireWriter,
    time: &SetTime,
    registries: &PlayRegistries,
) -> Result<(), PlayClientboundCodecError> {
    writer.write_i64(time.game_time)?;
    writer.write_count(
        "world clocks",
        time.clocks.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for (clock, state) in &time.clocks {
        writer.write_var_i32(registries.raw_id(WORLD_CLOCK, clock)?)?;
        writer.write_var_i64(state.total_ticks)?;
        writer.write_f32(state.partial_tick)?;
        writer.write_f32(state.rate)?;
    }
    Ok(())
}
