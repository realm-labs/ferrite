use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::play::block::{
    pack_block_position, pack_section_position, unpack_block_position, unpack_section_position,
};
use crate::java_26_2::play::clientbound::command::{self, CommandTreeError};
use crate::java_26_2::play::clientbound::packet::{
    BlockChangedAck, BlockDestruction, BlockEntityData, BlockEvent, BlockUpdate,
    BorderInitialization, ChangeDifficulty, ClockState, CommonSpawnInfo, DefaultSpawnPosition,
    EntityEvent, GameEvent, GameMode, GlobalBlockPosition, KeepAlive, PlayClientboundPacket,
    PlayLogin, PlayerAbilities, PlayerPosition, PlayerRotation, SectionBlockChange,
    SectionBlocksUpdate, ServerData, SetTime, TickingState, Vector3, VehiclePosition,
};
use crate::java_26_2::play::clientbound::player_info::{self, PlayerInfoError};
use crate::java_26_2::play::clientbound::recipe::{self, RecipeError};
use crate::java_26_2::play::clientbound::session;
use crate::java_26_2::play::clientbound::terrain::codec::{
    self as terrain_codec, TerrainCodecContext, TerrainCodecError,
};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::registry::{
    BIOME, DIMENSION_TYPE, PlayRegistries, PlayRegistryError, WORLD_CLOCK,
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
    #[error(transparent)]
    Terrain(#[from] TerrainCodecError),
    #[error("play clientbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("play clientbound packet {identity} is not part of the required C1 entry family")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
    #[error("clock state repeats world-clock identity {clock}")]
    DuplicateClock { clock: Identifier },
    #[error("global block-state raw ID {0} is outside the locked 0..=32365 range")]
    InvalidBlockState(i32),
    #[error("section block relative position {0} is outside 0..=4095")]
    InvalidRelativeBlockPosition(u16),
    #[error("block-entity type raw ID {0} is outside the locked 0..=48 range")]
    InvalidBlockEntityType(i32),
    #[error("block raw ID {0} is outside the locked 0..=1195 range")]
    InvalidBlock(i32),
    #[error("standalone block-entity data requires a compound NBT root, got tag {0}")]
    InvalidBlockEntityTag(u8),
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
        "minecraft:block_changed_ack" => PlayClientboundPacket::BlockChangedAck(BlockChangedAck {
            sequence: reader.read_var_i32()?,
        }),
        "minecraft:block_destruction" => {
            PlayClientboundPacket::BlockDestruction(BlockDestruction {
                breaker_entity_id: reader.read_var_i32()?,
                position: unpack_block_position(reader.read_i64()?),
                progress: reader.read_u8()?,
            })
        }
        "minecraft:block_entity_data" => {
            let position = unpack_block_position(reader.read_i64()?);
            let type_raw_id = reader.read_var_i32()?;
            validate_block_entity_type(type_raw_id)?;
            let update_tag = NetworkNbt::read(&mut reader, NbtQuota::Trusted)?;
            validate_compound_tag(&update_tag)?;
            PlayClientboundPacket::BlockEntityData(BlockEntityData {
                position,
                type_raw_id,
                update_tag,
            })
        }
        "minecraft:block_event" => {
            let position = unpack_block_position(reader.read_i64()?);
            let action = reader.read_u8()?;
            let parameter = reader.read_u8()?;
            let block_raw_id = reader.read_var_i32()?;
            validate_block(block_raw_id)?;
            PlayClientboundPacket::BlockEvent(BlockEvent {
                position,
                action,
                parameter,
                block_raw_id,
            })
        }
        "minecraft:block_update" => PlayClientboundPacket::BlockUpdate(BlockUpdate {
            position: unpack_block_position(reader.read_i64()?),
            state: read_block_state(&mut reader)?,
        }),
        "minecraft:change_difficulty" => {
            PlayClientboundPacket::ChangeDifficulty(ChangeDifficulty {
                raw_difficulty: reader.read_var_i32()?,
                locked: reader.read_bool()?,
            })
        }
        "minecraft:commands" => {
            PlayClientboundPacket::Commands(command::read(&mut reader, context.registries)?)
        }
        "minecraft:disconnect" => {
            let nbt = NetworkNbt::read(&mut reader, NbtQuota::Trusted)?;
            PlayClientboundPacket::Disconnect(TextComponentNbt::from_network_nbt(nbt)?)
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
        "minecraft:keep_alive" => PlayClientboundPacket::KeepAlive(KeepAlive {
            challenge: reader.read_i64()?,
        }),
        "minecraft:login" => {
            PlayClientboundPacket::Login(read_login(&mut reader, context.registries)?)
        }
        "minecraft:move_vehicle" => PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: read_vector(&mut reader)?,
            yaw: reader.read_f32()?,
            pitch: reader.read_f32()?,
        }),
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
        "minecraft:player_rotation" => PlayClientboundPacket::PlayerRotation(PlayerRotation {
            yaw: reader.read_f32()?,
            relative_yaw: reader.read_bool()?,
            pitch: reader.read_f32()?,
            relative_pitch: reader.read_bool()?,
        }),
        "minecraft:recipe_book_add" => {
            PlayClientboundPacket::RecipeBookAdd(recipe::read_book_add(&mut reader, context)?)
        }
        "minecraft:recipe_book_settings" => {
            PlayClientboundPacket::RecipeBookSettings(recipe::read_book_settings(&mut reader)?)
        }
        "minecraft:respawn" => {
            PlayClientboundPacket::Respawn(session::read(&mut reader, context.registries)?)
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
        "minecraft:section_blocks_update" => {
            PlayClientboundPacket::SectionBlocksUpdate(read_section_blocks(&mut reader)?)
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
        identity if terrain_codec::is_terrain_identity(identity) => {
            let biome_registry_size = context.registries.len(BIOME)?;
            PlayClientboundPacket::Terrain(terrain_codec::decode_body(
                identity,
                &mut reader,
                TerrainCodecContext {
                    section_count: context.dimension_section_count,
                    biome_registry_size,
                },
            )?)
        }
        identity => return Err(PlayClientboundCodecError::UnsupportedPacketIdentity { identity }),
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
        PlayClientboundPacket::BlockChangedAck(packet) => {
            writer.write_var_i32(packet.sequence)?;
        }
        PlayClientboundPacket::BlockDestruction(packet) => {
            writer.write_var_i32(packet.breaker_entity_id)?;
            writer.write_i64(pack_block_position(packet.position))?;
            writer.write_u8(packet.progress)?;
        }
        PlayClientboundPacket::BlockEntityData(packet) => {
            validate_block_entity_type(packet.type_raw_id)?;
            validate_compound_tag(&packet.update_tag)?;
            writer.write_i64(pack_block_position(packet.position))?;
            writer.write_var_i32(packet.type_raw_id)?;
            packet.update_tag.write(&mut writer)?;
        }
        PlayClientboundPacket::BlockEvent(packet) => {
            validate_block(packet.block_raw_id)?;
            writer.write_i64(pack_block_position(packet.position))?;
            writer.write_u8(packet.action)?;
            writer.write_u8(packet.parameter)?;
            writer.write_var_i32(packet.block_raw_id)?;
        }
        PlayClientboundPacket::BlockUpdate(packet) => {
            validate_block_state(packet.state)?;
            writer.write_i64(pack_block_position(packet.position))?;
            writer.write_var_i32(packet.state)?;
        }
        PlayClientboundPacket::ChangeDifficulty(packet) => {
            writer.write_var_i32(packet.raw_difficulty)?;
            writer.write_bool(packet.locked)?;
        }
        PlayClientboundPacket::Commands(tree) => command::write(&mut writer, tree, registries)?,
        PlayClientboundPacket::Disconnect(reason) => reason.network_nbt().write(&mut writer)?,
        PlayClientboundPacket::EntityEvent(packet) => {
            writer.write_i32(packet.entity_id)?;
            writer.write_i8(packet.event)?;
        }
        PlayClientboundPacket::GameEvent(packet) => {
            writer.write_u8(packet.event)?;
            writer.write_f32(packet.parameter)?;
        }
        PlayClientboundPacket::InitializeBorder(border) => write_border(&mut writer, border)?,
        PlayClientboundPacket::KeepAlive(packet) => writer.write_i64(packet.challenge)?,
        PlayClientboundPacket::Login(login) => write_login(&mut writer, login, registries)?,
        PlayClientboundPacket::MoveVehicle(packet) => {
            write_vector(&mut writer, packet.position)?;
            writer.write_f32(packet.yaw)?;
            writer.write_f32(packet.pitch)?;
        }
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
        PlayClientboundPacket::PlayerRotation(rotation) => {
            writer.write_f32(rotation.yaw)?;
            writer.write_bool(rotation.relative_yaw)?;
            writer.write_f32(rotation.pitch)?;
            writer.write_bool(rotation.relative_pitch)?;
        }
        PlayClientboundPacket::RecipeBookAdd(packet) => {
            recipe::write_book_add(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::RecipeBookSettings(settings) => {
            recipe::write_book_settings(&mut writer, *settings)?;
        }
        PlayClientboundPacket::Respawn(packet) => {
            session::write(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::ServerData(data) => {
            data.motd.network_nbt().write(&mut writer)?;
            writer.write_bool(data.icon.is_some())?;
            if let Some(icon) = &data.icon {
                writer.write_byte_array(icon, MAX_INFLATED_PACKET_LENGTH)?;
            }
        }
        PlayClientboundPacket::SectionBlocksUpdate(packet) => {
            write_section_blocks(&mut writer, packet)?;
        }
        PlayClientboundPacket::SetDefaultSpawnPosition(spawn) => {
            spawn.position.dimension.write(&mut writer)?;
            writer.write_i64(spawn.position.packed_position)?;
            writer.write_f32(spawn.yaw)?;
            writer.write_f32(spawn.pitch)?;
        }
        PlayClientboundPacket::SetHeldSlot(slot) => writer.write_var_i32(*slot)?,
        PlayClientboundPacket::SetTime(time) => write_time(&mut writer, time, registries)?,
        PlayClientboundPacket::Terrain(packet) => terrain_codec::encode_body(
            packet,
            &mut writer,
            TerrainCodecContext {
                section_count: terrain_section_count(packet),
                biome_registry_size: registries.len(BIOME)?,
            },
        )?,
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

pub(crate) fn packet_identity(packet: &PlayClientboundPacket) -> &'static str {
    match packet {
        PlayClientboundPacket::BlockChangedAck(_) => "minecraft:block_changed_ack",
        PlayClientboundPacket::BlockDestruction(_) => "minecraft:block_destruction",
        PlayClientboundPacket::BlockEntityData(_) => "minecraft:block_entity_data",
        PlayClientboundPacket::BlockEvent(_) => "minecraft:block_event",
        PlayClientboundPacket::BlockUpdate(_) => "minecraft:block_update",
        PlayClientboundPacket::ChangeDifficulty(_) => "minecraft:change_difficulty",
        PlayClientboundPacket::Commands(_) => "minecraft:commands",
        PlayClientboundPacket::Disconnect(_) => "minecraft:disconnect",
        PlayClientboundPacket::EntityEvent(_) => "minecraft:entity_event",
        PlayClientboundPacket::GameEvent(_) => "minecraft:game_event",
        PlayClientboundPacket::InitializeBorder(_) => "minecraft:initialize_border",
        PlayClientboundPacket::KeepAlive(_) => "minecraft:keep_alive",
        PlayClientboundPacket::Login(_) => "minecraft:login",
        PlayClientboundPacket::MoveVehicle(_) => "minecraft:move_vehicle",
        PlayClientboundPacket::PlayerAbilities(_) => "minecraft:player_abilities",
        PlayClientboundPacket::PlayerInfoUpdate(_) => "minecraft:player_info_update",
        PlayClientboundPacket::PlayerPosition(_) => "minecraft:player_position",
        PlayClientboundPacket::PlayerRotation(_) => "minecraft:player_rotation",
        PlayClientboundPacket::RecipeBookAdd(_) => "minecraft:recipe_book_add",
        PlayClientboundPacket::RecipeBookSettings(_) => "minecraft:recipe_book_settings",
        PlayClientboundPacket::Respawn(_) => "minecraft:respawn",
        PlayClientboundPacket::ServerData(_) => "minecraft:server_data",
        PlayClientboundPacket::SectionBlocksUpdate(_) => "minecraft:section_blocks_update",
        PlayClientboundPacket::SetDefaultSpawnPosition(_) => "minecraft:set_default_spawn_position",
        PlayClientboundPacket::SetHeldSlot(_) => "minecraft:set_held_slot",
        PlayClientboundPacket::SetTime(_) => "minecraft:set_time",
        PlayClientboundPacket::Terrain(packet) => terrain_codec::identity(packet),
        PlayClientboundPacket::TickingState(_) => "minecraft:ticking_state",
        PlayClientboundPacket::TickingStep(_) => "minecraft:ticking_step",
        PlayClientboundPacket::UpdateRecipes(_) => "minecraft:update_recipes",
    }
}

fn terrain_section_count(
    packet: &crate::java_26_2::play::clientbound::terrain::packet::TerrainPacket,
) -> usize {
    use crate::java_26_2::play::clientbound::terrain::packet::TerrainPacket;

    let count = match packet {
        TerrainPacket::LevelChunkWithLight(chunk) => Some(chunk.sections.len()),
        TerrainPacket::ChunksBiomes(chunks) => chunks.first().map(|chunk| chunk.sections.len()),
        TerrainPacket::LightUpdate(update) => update.light.sky.len().checked_sub(2),
        _ => None,
    };
    count.unwrap_or(24)
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

pub(super) fn read_common_spawn(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<CommonSpawnInfo, PlayClientboundCodecError> {
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
    Ok(CommonSpawnInfo {
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
    write_common_spawn(writer, &login.spawn, registries)?;
    writer.write_bool(login.online_mode)?;
    writer.write_bool(login.enforces_secure_chat)?;
    Ok(())
}

pub(super) fn write_common_spawn(
    writer: &mut WireWriter,
    spawn: &CommonSpawnInfo,
    registries: &PlayRegistries,
) -> Result<(), PlayClientboundCodecError> {
    writer.write_var_i32(registries.raw_id(DIMENSION_TYPE, &spawn.dimension_type)?)?;
    spawn.dimension.write(writer)?;
    writer.write_i64(spawn.obfuscated_seed)?;
    writer.write_i8(spawn.game_mode.id() as i8)?;
    writer.write_i8(spawn.previous_game_mode.map_or(-1, |mode| mode.id() as i8))?;
    writer.write_bool(spawn.is_debug)?;
    writer.write_bool(spawn.is_flat)?;
    writer.write_bool(spawn.last_death.is_some())?;
    if let Some(last_death) = &spawn.last_death {
        last_death.dimension.write(writer)?;
        writer.write_i64(last_death.packed_position)?;
    }
    writer.write_var_i32(spawn.portal_cooldown)?;
    writer.write_var_i32(spawn.sea_level)?;
    Ok(())
}

fn read_position(reader: &mut WireReader<'_>) -> Result<PlayerPosition, PlayClientboundCodecError> {
    Ok(PlayerPosition {
        teleport_id: reader.read_var_i32()?,
        position: read_vector(reader)?,
        motion: read_vector(reader)?,
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
    write_vector(writer, position.position)?;
    write_vector(writer, position.motion)?;
    writer.write_f32(position.yaw)?;
    writer.write_f32(position.pitch)?;
    writer.write_i32(position.relative_flags as i32)?;
    Ok(())
}

fn read_vector(reader: &mut WireReader<'_>) -> Result<Vector3, PlayClientboundCodecError> {
    Ok(Vector3 {
        x: reader.read_f64()?,
        y: reader.read_f64()?,
        z: reader.read_f64()?,
    })
}

fn write_vector(writer: &mut WireWriter, vector: Vector3) -> Result<(), PlayClientboundCodecError> {
    writer.write_f64(vector.x)?;
    writer.write_f64(vector.y)?;
    writer.write_f64(vector.z)?;
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

fn read_block_state(reader: &mut WireReader<'_>) -> Result<i32, PlayClientboundCodecError> {
    let state = reader.read_var_i32()?;
    validate_block_state(state)?;
    Ok(state)
}

fn validate_block_state(state: i32) -> Result<(), PlayClientboundCodecError> {
    if (0..=32_365).contains(&state) {
        Ok(())
    } else {
        Err(PlayClientboundCodecError::InvalidBlockState(state))
    }
}

fn validate_block_entity_type(type_raw_id: i32) -> Result<(), PlayClientboundCodecError> {
    if (0..=48).contains(&type_raw_id) {
        Ok(())
    } else {
        Err(PlayClientboundCodecError::InvalidBlockEntityType(
            type_raw_id,
        ))
    }
}

fn validate_block(block_raw_id: i32) -> Result<(), PlayClientboundCodecError> {
    if (0..=1_195).contains(&block_raw_id) {
        Ok(())
    } else {
        Err(PlayClientboundCodecError::InvalidBlock(block_raw_id))
    }
}

fn validate_compound_tag(tag: &NetworkNbt) -> Result<(), PlayClientboundCodecError> {
    if tag.root_tag_id() == 10 {
        Ok(())
    } else {
        Err(PlayClientboundCodecError::InvalidBlockEntityTag(
            tag.root_tag_id(),
        ))
    }
}

fn read_section_blocks(
    reader: &mut WireReader<'_>,
) -> Result<SectionBlocksUpdate, PlayClientboundCodecError> {
    let section = unpack_section_position(reader.read_i64()?);
    let count = reader.read_count("section block changes", reader.remaining())?;
    let mut changes = Vec::with_capacity(count);
    for _ in 0..count {
        let packed = reader.read_var_i64()? as u64;
        changes.push(SectionBlockChange {
            relative_position: (packed & 0xfff) as u16,
            state: (0..=32_365)
                .contains(&((packed >> 12) as i32))
                .then_some((packed >> 12) as i32),
        });
    }
    Ok(SectionBlocksUpdate { section, changes })
}

fn write_section_blocks(
    writer: &mut WireWriter,
    packet: &SectionBlocksUpdate,
) -> Result<(), PlayClientboundCodecError> {
    writer.write_i64(pack_section_position(packet.section))?;
    writer.write_count(
        "section block changes",
        packet.changes.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for change in &packet.changes {
        let state = change
            .state
            .ok_or(PlayClientboundCodecError::InvalidBlockState(-1))?;
        validate_block_state(state)?;
        if change.relative_position > 4095 {
            return Err(PlayClientboundCodecError::InvalidRelativeBlockPosition(
                change.relative_position,
            ));
        }
        let packed = (i64::from(state) << 12) | i64::from(change.relative_position);
        writer.write_var_i64(packed)?;
    }
    Ok(())
}
