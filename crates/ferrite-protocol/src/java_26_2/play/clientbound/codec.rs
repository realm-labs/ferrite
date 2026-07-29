use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::play::clientbound::command::{self, CommandTreeError};
use crate::java_26_2::play::clientbound::packet::{
    BorderInitialization, ChangeDifficulty, ClockState, CommonSpawnInfo, DefaultSpawnPosition,
    EntityEvent, GameEvent, GameMode, GlobalBlockPosition, PlayClientboundPacket, PlayLogin,
    PlayerAbilities, PlayerPosition, ServerData, SetTime, TickingState, Vector3,
};
use crate::java_26_2::play::clientbound::player_info::{self, PlayerInfoError};
use crate::java_26_2::play::clientbound::recipe::{self, RecipeError};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::registry::{
    DIMENSION_TYPE, PlayRegistries, PlayRegistryError, WORLD_CLOCK,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, PartialEq, Error)]
pub enum PlayClientboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidIdentifier(#[from] IdentifierError),
    #[error(transparent)]
    InvalidNbt(#[from] NbtError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error(transparent)]
    CommandTree(#[from] CommandTreeError),
    #[error(transparent)]
    PlayerInfo(#[from] PlayerInfoError),
    #[error(transparent)]
    Recipe(#[from] RecipeError),
    #[error("play clientbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("play clientbound packet {identity} is not part of the required C1 entry family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
    #[error("clock state repeats world-clock identity {clock}")]
    DuplicateClock { clock: Identifier },
}

pub fn decode_packet(
    body: &[u8],
    context: PlayDecodeContext<'_>,
) -> Result<PlayClientboundPacket, PlayClientboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor =
        PacketCatalog::by_wire_id(ConnectionState::Play, PacketDirection::Clientbound, wire_id)
            .ok_or(PlayClientboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:change_difficulty" => {
            PlayClientboundPacket::ChangeDifficulty(ChangeDifficulty {
                raw_difficulty: reader.read_var_i32()?,
                locked: reader.read_bool()?,
            })
        }
        "minecraft:commands" => {
            PlayClientboundPacket::Commands(command::read(&mut reader, context.registries)?)
        }
        "minecraft:entity_event" => PlayClientboundPacket::EntityEvent(EntityEvent {
            entity_id: reader.read_i32()?,
            event: reader.read_i8()?,
        }),
        "minecraft:game_event" => PlayClientboundPacket::GameEvent(GameEvent {
            event: reader.read_u8()?,
            parameter: reader.read_f32()?,
        }),
        "minecraft:initialize_border" => {
            PlayClientboundPacket::InitializeBorder(read_border(&mut reader)?)
        }
        "minecraft:login" => {
            PlayClientboundPacket::Login(read_login(&mut reader, context.registries)?)
        }
        "minecraft:player_abilities" => PlayClientboundPacket::PlayerAbilities(PlayerAbilities {
            flags: reader.read_u8()?,
            flying_speed: reader.read_f32()?,
            walking_speed: reader.read_f32()?,
        }),
        "minecraft:player_info_update" => {
            PlayClientboundPacket::PlayerInfoUpdate(player_info::read(&mut reader)?)
        }
        "minecraft:player_position" => {
            PlayClientboundPacket::PlayerPosition(read_position(&mut reader)?)
        }
        "minecraft:recipe_book_add" => {
            PlayClientboundPacket::RecipeBookAdd(recipe::read_book_add(&mut reader, context)?)
        }
        "minecraft:recipe_book_settings" => {
            PlayClientboundPacket::RecipeBookSettings(recipe::read_book_settings(&mut reader)?)
        }
        "minecraft:server_data" => {
            let nbt = NetworkNbt::read(&mut reader, NbtQuota::Trusted)?;
            let motd = TextComponentNbt::from_network_nbt(nbt)?;
            let icon = if reader.read_bool()? {
                Some(reader.read_byte_array(MAX_INFLATED_PACKET_LENGTH)?.to_vec())
            } else {
                None
            };
            PlayClientboundPacket::ServerData(ServerData { motd, icon })
        }
        "minecraft:set_default_spawn_position" => {
            PlayClientboundPacket::SetDefaultSpawnPosition(DefaultSpawnPosition {
                position: GlobalBlockPosition {
                    dimension: read_identifier(&mut reader)?,
                    packed_position: reader.read_i64()?,
                },
                yaw: reader.read_f32()?,
                pitch: reader.read_f32()?,
            })
        }
        "minecraft:set_held_slot" => PlayClientboundPacket::SetHeldSlot(reader.read_var_i32()?),
        "minecraft:set_time" => {
            PlayClientboundPacket::SetTime(read_time(&mut reader, context.registries)?)
        }
        "minecraft:ticking_state" => PlayClientboundPacket::TickingState(TickingState {
            tick_rate: reader.read_f32()?,
            frozen: reader.read_bool()?,
        }),
        "minecraft:ticking_step" => PlayClientboundPacket::TickingStep(reader.read_var_i32()?),
        "minecraft:update_recipes" => {
            PlayClientboundPacket::UpdateRecipes(recipe::read_projection(&mut reader, context)?)
        }
        identity => {
            return Err(PlayClientboundCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &PlayClientboundPacket,
    registries: &PlayRegistries,
) -> Result<Vec<u8>, PlayClientboundCodecError> {
    let identity = packet_identity(packet);
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Play,
        PacketDirection::Clientbound,
        identity,
    )
    .ok_or(PlayClientboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        PlayClientboundPacket::ChangeDifficulty(packet) => {
            writer.write_var_i32(packet.raw_difficulty)?;
            writer.write_bool(packet.locked)?;
        }
        PlayClientboundPacket::Commands(tree) => command::write(&mut writer, tree, registries)?,
        PlayClientboundPacket::EntityEvent(packet) => {
            writer.write_i32(packet.entity_id)?;
            writer.write_i8(packet.event)?;
        }
        PlayClientboundPacket::GameEvent(packet) => {
            writer.write_u8(packet.event)?;
            writer.write_f32(packet.parameter)?;
        }
        PlayClientboundPacket::InitializeBorder(border) => write_border(&mut writer, border)?,
        PlayClientboundPacket::Login(login) => write_login(&mut writer, login, registries)?,
        PlayClientboundPacket::PlayerAbilities(abilities) => {
            writer.write_u8(abilities.flags)?;
            writer.write_f32(abilities.flying_speed)?;
            writer.write_f32(abilities.walking_speed)?;
        }
        PlayClientboundPacket::PlayerInfoUpdate(update) => {
            player_info::write(&mut writer, update)?;
        }
        PlayClientboundPacket::PlayerPosition(position) => {
            write_position(&mut writer, position)?;
        }
        PlayClientboundPacket::RecipeBookAdd(packet) => {
            recipe::write_book_add(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::RecipeBookSettings(settings) => {
            recipe::write_book_settings(&mut writer, *settings)?;
        }
        PlayClientboundPacket::ServerData(data) => {
            data.motd.network_nbt().write(&mut writer)?;
            writer.write_bool(data.icon.is_some())?;
            if let Some(icon) = &data.icon {
                writer.write_byte_array(icon, MAX_INFLATED_PACKET_LENGTH)?;
            }
        }
        PlayClientboundPacket::SetDefaultSpawnPosition(spawn) => {
            spawn.position.dimension.write(&mut writer)?;
            writer.write_i64(spawn.position.packed_position)?;
            writer.write_f32(spawn.yaw)?;
            writer.write_f32(spawn.pitch)?;
        }
        PlayClientboundPacket::SetHeldSlot(slot) => writer.write_var_i32(*slot)?,
        PlayClientboundPacket::SetTime(time) => write_time(&mut writer, time, registries)?,
        PlayClientboundPacket::TickingState(state) => {
            writer.write_f32(state.tick_rate)?;
            writer.write_bool(state.frozen)?;
        }
        PlayClientboundPacket::TickingStep(steps) => writer.write_var_i32(*steps)?,
        PlayClientboundPacket::UpdateRecipes(projection) => {
            recipe::write_projection(&mut writer, projection, registries)?;
        }
    }
    Ok(writer.into_inner())
}

fn packet_identity(packet: &PlayClientboundPacket) -> &'static str {
    match packet {
        PlayClientboundPacket::ChangeDifficulty(_) => "minecraft:change_difficulty",
        PlayClientboundPacket::Commands(_) => "minecraft:commands",
        PlayClientboundPacket::EntityEvent(_) => "minecraft:entity_event",
        PlayClientboundPacket::GameEvent(_) => "minecraft:game_event",
        PlayClientboundPacket::InitializeBorder(_) => "minecraft:initialize_border",
        PlayClientboundPacket::Login(_) => "minecraft:login",
        PlayClientboundPacket::PlayerAbilities(_) => "minecraft:player_abilities",
        PlayClientboundPacket::PlayerInfoUpdate(_) => "minecraft:player_info_update",
        PlayClientboundPacket::PlayerPosition(_) => "minecraft:player_position",
        PlayClientboundPacket::RecipeBookAdd(_) => "minecraft:recipe_book_add",
        PlayClientboundPacket::RecipeBookSettings(_) => "minecraft:recipe_book_settings",
        PlayClientboundPacket::ServerData(_) => "minecraft:server_data",
        PlayClientboundPacket::SetDefaultSpawnPosition(_) => "minecraft:set_default_spawn_position",
        PlayClientboundPacket::SetHeldSlot(_) => "minecraft:set_held_slot",
        PlayClientboundPacket::SetTime(_) => "minecraft:set_time",
        PlayClientboundPacket::TickingState(_) => "minecraft:ticking_state",
        PlayClientboundPacket::TickingStep(_) => "minecraft:ticking_step",
        PlayClientboundPacket::UpdateRecipes(_) => "minecraft:update_recipes",
    }
}

fn read_border(
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

fn write_border(
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

fn read_login(
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
    let dimension_type = registries.resolve(DIMENSION_TYPE, reader.read_var_i32()?)?;
    let dimension = read_identifier(reader)?;
    let obfuscated_seed = reader.read_i64()?;
    let game_mode = GameMode::from_i8_or_survival(reader.read_i8()?);
    let previous_raw = reader.read_i8()?;
    let previous_game_mode =
        (previous_raw != -1).then(|| GameMode::from_i8_or_survival(previous_raw));
    let is_debug = reader.read_bool()?;
    let is_flat = reader.read_bool()?;
    let last_death = if reader.read_bool()? {
        Some(GlobalBlockPosition {
            dimension: read_identifier(reader)?,
            packed_position: reader.read_i64()?,
        })
    } else {
        None
    };
    let portal_cooldown = reader.read_var_i32()?;
    let sea_level = reader.read_var_i32()?;
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
        spawn: CommonSpawnInfo {
            dimension_type,
            dimension,
            obfuscated_seed,
            game_mode,
            previous_game_mode,
            is_debug,
            is_flat,
            last_death,
            portal_cooldown,
            sea_level,
        },
        online_mode,
        enforces_secure_chat,
    })
}

fn write_login(
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
    writer.write_var_i32(registries.raw_id(DIMENSION_TYPE, &login.spawn.dimension_type)?)?;
    login.spawn.dimension.write(writer)?;
    writer.write_i64(login.spawn.obfuscated_seed)?;
    writer.write_i8(login.spawn.game_mode.id() as i8)?;
    writer.write_i8(
        login
            .spawn
            .previous_game_mode
            .map_or(-1, |mode| mode.id() as i8),
    )?;
    writer.write_bool(login.spawn.is_debug)?;
    writer.write_bool(login.spawn.is_flat)?;
    writer.write_bool(login.spawn.last_death.is_some())?;
    if let Some(last_death) = &login.spawn.last_death {
        last_death.dimension.write(writer)?;
        writer.write_i64(last_death.packed_position)?;
    }
    writer.write_var_i32(login.spawn.portal_cooldown)?;
    writer.write_var_i32(login.spawn.sea_level)?;
    writer.write_bool(login.online_mode)?;
    writer.write_bool(login.enforces_secure_chat)?;
    Ok(())
}

fn read_position(reader: &mut WireReader<'_>) -> Result<PlayerPosition, PlayClientboundCodecError> {
    Ok(PlayerPosition {
        teleport_id: reader.read_var_i32()?,
        position: Vector3 {
            x: reader.read_f64()?,
            y: reader.read_f64()?,
            z: reader.read_f64()?,
        },
        motion: Vector3 {
            x: reader.read_f64()?,
            y: reader.read_f64()?,
            z: reader.read_f64()?,
        },
        yaw: reader.read_f32()?,
        pitch: reader.read_f32()?,
        relative_flags: reader.read_i32()? as u32,
    })
}

fn write_position(
    writer: &mut WireWriter,
    position: &PlayerPosition,
) -> Result<(), PlayClientboundCodecError> {
    writer.write_var_i32(position.teleport_id)?;
    writer.write_f64(position.position.x)?;
    writer.write_f64(position.position.y)?;
    writer.write_f64(position.position.z)?;
    writer.write_f64(position.motion.x)?;
    writer.write_f64(position.motion.y)?;
    writer.write_f64(position.motion.z)?;
    writer.write_f32(position.yaw)?;
    writer.write_f32(position.pitch)?;
    writer.write_i32(position.relative_flags as i32)?;
    Ok(())
}

fn read_time(
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

fn write_time(
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

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, PlayClientboundCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
