use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::play::block::{
    pack_block_position, pack_section_position, unpack_block_position, unpack_section_position,
};
use crate::java_26_2::play::clientbound::boss_waypoint::{
    codec as boss_waypoint_codec, codec::BossWaypointCodecError,
};
use crate::java_26_2::play::clientbound::chat_presentation::{
    codec as chat_codec, codec::ChatPresentationCodecError,
};
use crate::java_26_2::play::clientbound::combat_look::{
    codec as combat_look_codec, codec::CombatLookCodecError,
};
use crate::java_26_2::play::clientbound::command::{self, CommandTreeError};
use crate::java_26_2::play::clientbound::completion::{
    codec as completion_codec, codec::CompletionCodecError,
};
use crate::java_26_2::play::clientbound::container::{
    codec as container_codec, codec::ContainerCodecError,
};
use crate::java_26_2::play::clientbound::entity_effects::{
    codec as entity_effects_codec, codec::EntityEffectsCodecError,
};
use crate::java_26_2::play::clientbound::entity_motion::codec as entity_motion_codec;
use crate::java_26_2::play::clientbound::entity_session::{
    codec as entity_session_codec, codec::EntitySessionCodecError,
};
use crate::java_26_2::play::clientbound::entity_spawn::{
    codec as entity_spawn_codec, codec::EntitySpawnCodecError,
};
use crate::java_26_2::play::clientbound::entity_state::{
    codec as entity_state_codec, codec::EntityStateCodecError,
};
use crate::java_26_2::play::clientbound::entry_codec;
use crate::java_26_2::play::clientbound::inventory_progression::{
    codec as inventory_codec, codec::InventoryProgressionCodecError,
};
use crate::java_26_2::play::clientbound::merchant::{
    codec as merchant_codec, codec::MerchantCodecError,
};
use crate::java_26_2::play::clientbound::packet::{
    BlockChangedAck, BlockDestruction, BlockEntityData, BlockEvent, BlockUpdate, ChangeDifficulty,
    CommonSpawnInfo, DefaultSpawnPosition, EntityEvent, GameEvent, GameMode, GlobalBlockPosition,
    KeepAlive, Ping, PlayClientboundPacket, PlayerAbilities, PlayerRotation, SectionBlockChange,
    SectionBlocksUpdate, ServerData, TickingState, VehiclePosition,
};
use crate::java_26_2::play::clientbound::particle::codec as particle_codec;
use crate::java_26_2::play::clientbound::player_info::{self, PlayerInfoError};
use crate::java_26_2::play::clientbound::player_info_remove;
use crate::java_26_2::play::clientbound::player_projection::{
    codec as player_projection_codec, codec::PlayerProjectionCodecError,
};
use crate::java_26_2::play::clientbound::recipe::{self, RecipeError};
use crate::java_26_2::play::clientbound::scoreboard::{
    codec as scoreboard_codec, codec::ScoreboardCodecError,
};
use crate::java_26_2::play::clientbound::session;
use crate::java_26_2::play::clientbound::sound::{codec as sound_codec, codec::SoundCodecError};
use crate::java_26_2::play::clientbound::special_screen::{
    codec as special_screen_codec, codec::SpecialScreenCodecError,
};
use crate::java_26_2::play::clientbound::terrain::codec::{
    self as terrain_codec, TerrainCodecContext, TerrainCodecError,
};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::registry::{BIOME, DIMENSION_TYPE, PlayRegistries, PlayRegistryError};
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
    Completion(#[from] CompletionCodecError),
    #[error(transparent)]
    BossWaypoint(#[from] BossWaypointCodecError),
    #[error(transparent)]
    ChatPresentation(#[from] ChatPresentationCodecError),
    #[error(transparent)]
    CombatLook(#[from] CombatLookCodecError),
    #[error(transparent)]
    Container(#[from] ContainerCodecError),
    #[error(transparent)]
    EntityEffects(#[from] EntityEffectsCodecError),
    #[error(transparent)]
    EntitySession(#[from] EntitySessionCodecError),
    #[error(transparent)]
    EntitySpawn(#[from] EntitySpawnCodecError),
    #[error(transparent)]
    EntityState(#[from] EntityStateCodecError),
    #[error(transparent)]
    InventoryProgression(#[from] InventoryProgressionCodecError),
    #[error(transparent)]
    Merchant(#[from] MerchantCodecError),
    #[error(transparent)]
    PlayerInfo(#[from] PlayerInfoError),
    #[error(transparent)]
    PlayerProjection(#[from] PlayerProjectionCodecError),
    #[error(transparent)]
    Recipe(#[from] RecipeError),
    #[error(transparent)]
    Scoreboard(#[from] ScoreboardCodecError),
    #[error(transparent)]
    Sound(#[from] SoundCodecError),
    #[error(transparent)]
    SpecialScreen(#[from] SpecialScreenCodecError),
    #[error(transparent)]
    Terrain(#[from] TerrainCodecError),
    #[error("play clientbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("play clientbound packet {identity} has no implemented family codec")]
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
        "minecraft:add_entity" => {
            PlayClientboundPacket::AddEntity(Box::new(entity_spawn_codec::read_add(&mut reader)?))
        }
        "minecraft:animate" => {
            PlayClientboundPacket::Animate(entity_session_codec::read_animate(&mut reader)?)
        }
        "minecraft:award_stats" => PlayClientboundPacket::AwardStats(
            player_projection_codec::read_stats(&mut reader, context.registries)?,
        ),
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
        "minecraft:boss_event" => {
            PlayClientboundPacket::BossEvent(boss_waypoint_codec::read_boss(&mut reader)?)
        }
        "minecraft:change_difficulty" => {
            PlayClientboundPacket::ChangeDifficulty(ChangeDifficulty {
                raw_difficulty: reader.read_var_i32()?,
                locked: reader.read_bool()?,
            })
        }
        "minecraft:command_suggestions" => PlayClientboundPacket::CommandSuggestions(Box::new(
            completion_codec::read_command(&mut reader)?,
        )),
        "minecraft:commands" => {
            PlayClientboundPacket::Commands(command::read(&mut reader, context.registries)?)
        }
        "minecraft:container_close" => {
            PlayClientboundPacket::ContainerClose(container_codec::read_close(&mut reader)?)
        }
        "minecraft:container_set_content" => PlayClientboundPacket::ContainerSetContent(
            container_codec::read_content(&mut reader, context)?,
        ),
        "minecraft:container_set_data" => {
            PlayClientboundPacket::ContainerSetData(container_codec::read_data(&mut reader)?)
        }
        "minecraft:container_set_slot" => PlayClientboundPacket::ContainerSetSlot(
            container_codec::read_slot(&mut reader, context)?,
        ),
        "minecraft:cooldown" => {
            PlayClientboundPacket::Cooldown(player_projection_codec::read_cooldown(&mut reader)?)
        }
        "minecraft:custom_chat_completions" => PlayClientboundPacket::CustomChatCompletions(
            Box::new(completion_codec::read_custom(&mut reader)?),
        ),
        "minecraft:damage_event" => PlayClientboundPacket::DamageEvent(
            entity_session_codec::read_damage(&mut reader, context.registries)?,
        ),
        "minecraft:delete_chat" => {
            PlayClientboundPacket::DeleteChat(chat_codec::read_delete(&mut reader)?)
        }
        "minecraft:disconnect" => {
            let nbt = NetworkNbt::read(&mut reader, NbtQuota::Trusted)?;
            PlayClientboundPacket::Disconnect(TextComponentNbt::from_network_nbt(nbt)?)
        }
        "minecraft:disguised_chat" => PlayClientboundPacket::DisguisedChat(
            chat_codec::read_disguised(&mut reader, context.registries)?,
        ),
        "minecraft:entity_event" => PlayClientboundPacket::EntityEvent(EntityEvent {
            entity_id: reader.read_i32()?,
            event: reader.read_i8()?,
        }),
        "minecraft:entity_position_sync" => PlayClientboundPacket::EntityPositionSync(
            entity_motion_codec::read_position_sync(&mut reader)?,
        ),
        "minecraft:explode" => PlayClientboundPacket::Explosion(Box::new(
            entity_effects_codec::read_explosion(&mut reader, context)?,
        )),
        "minecraft:game_event" => PlayClientboundPacket::GameEvent(GameEvent {
            event: reader.read_u8()?,
            parameter: reader.read_f32()?,
        }),
        "minecraft:hurt_animation" => {
            PlayClientboundPacket::HurtAnimation(entity_session_codec::read_hurt(&mut reader)?)
        }
        "minecraft:initialize_border" => {
            PlayClientboundPacket::InitializeBorder(entry_codec::read_border(&mut reader)?)
        }
        "minecraft:keep_alive" => PlayClientboundPacket::KeepAlive(KeepAlive {
            challenge: reader.read_i64()?,
        }),
        "minecraft:login" => {
            PlayClientboundPacket::Login(entry_codec::read_login(&mut reader, context.registries)?)
        }
        "minecraft:level_particles" => PlayClientboundPacket::LevelParticles(Box::new(
            particle_codec::read(&mut reader, context)?,
        )),
        "minecraft:map_item_data" => PlayClientboundPacket::MapItemData(inventory_codec::read_map(
            &mut reader,
            context.registries,
        )?),
        "minecraft:merchant_offers" => {
            PlayClientboundPacket::MerchantOffers(merchant_codec::read(&mut reader, context)?)
        }
        "minecraft:mount_screen_open" => {
            PlayClientboundPacket::MountScreenOpen(special_screen_codec::read_mount(&mut reader)?)
        }
        "minecraft:move_entity_pos" => PlayClientboundPacket::MoveEntityPosition(
            entity_motion_codec::read_position(&mut reader)?,
        ),
        "minecraft:move_entity_pos_rot" => PlayClientboundPacket::MoveEntityPositionRotation(
            entity_motion_codec::read_position_rotation(&mut reader)?,
        ),
        "minecraft:move_entity_rot" => PlayClientboundPacket::MoveEntityRotation(
            entity_motion_codec::read_rotation(&mut reader)?,
        ),
        "minecraft:move_minecart_along_track" => PlayClientboundPacket::MoveMinecartAlongTrack(
            entity_motion_codec::read_minecart(&mut reader)?,
        ),
        "minecraft:move_vehicle" => PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: entry_codec::read_vector(&mut reader)?,
            yaw: reader.read_f32()?,
            pitch: reader.read_f32()?,
        }),
        "minecraft:open_screen" => PlayClientboundPacket::OpenScreen(container_codec::read_open(
            &mut reader,
            context.registries,
        )?),
        "minecraft:open_book" => {
            PlayClientboundPacket::OpenBook(special_screen_codec::read_hand(&mut reader)?)
        }
        "minecraft:open_sign_editor" => {
            PlayClientboundPacket::OpenSignEditor(special_screen_codec::read_sign(&mut reader)?)
        }
        "minecraft:ping" => PlayClientboundPacket::Ping(Ping {
            payload: reader.read_i32()?,
        }),
        "minecraft:place_ghost_recipe" => PlayClientboundPacket::PlaceGhostRecipe(Box::new(
            recipe::book::codec::read_ghost(&mut reader, context)?,
        )),
        "minecraft:player_abilities" => PlayClientboundPacket::PlayerAbilities(PlayerAbilities {
            flags: reader.read_u8()?,
            flying_speed: reader.read_f32()?,
            walking_speed: reader.read_f32()?,
        }),
        "minecraft:player_chat" => PlayClientboundPacket::PlayerChat(Box::new(
            chat_codec::read_player(&mut reader, context.registries)?,
        )),
        "minecraft:player_combat_end" => {
            PlayClientboundPacket::PlayerCombatEnd(combat_look_codec::read_end(&mut reader)?)
        }
        "minecraft:player_combat_enter" => PlayClientboundPacket::PlayerCombatEnter,
        "minecraft:player_combat_kill" => {
            PlayClientboundPacket::PlayerCombatKill(combat_look_codec::read_kill(&mut reader)?)
        }
        "minecraft:player_info_update" => {
            PlayClientboundPacket::PlayerInfoUpdate(player_info::read(&mut reader)?)
        }
        "minecraft:player_info_remove" => PlayClientboundPacket::PlayerInfoRemove(Box::new(
            player_info_remove::codec::read(&mut reader)?,
        )),
        "minecraft:player_look_at" => {
            PlayClientboundPacket::PlayerLookAt(combat_look_codec::read_look(&mut reader)?)
        }
        "minecraft:player_position" => {
            PlayClientboundPacket::PlayerPosition(entry_codec::read_position(&mut reader)?)
        }
        "minecraft:player_rotation" => PlayClientboundPacket::PlayerRotation(PlayerRotation {
            yaw: reader.read_f32()?,
            relative_yaw: reader.read_bool()?,
            pitch: reader.read_f32()?,
            relative_pitch: reader.read_bool()?,
        }),
        "minecraft:projectile_power" => PlayClientboundPacket::ProjectilePower(
            entity_motion_codec::read_projectile(&mut reader)?,
        ),
        "minecraft:recipe_book_add" => {
            PlayClientboundPacket::RecipeBookAdd(recipe::read_book_add(&mut reader, context)?)
        }
        "minecraft:recipe_book_remove" => {
            PlayClientboundPacket::RecipeBookRemove(recipe::book::codec::read_remove(&mut reader)?)
        }
        "minecraft:recipe_book_settings" => {
            PlayClientboundPacket::RecipeBookSettings(recipe::read_book_settings(&mut reader)?)
        }
        "minecraft:respawn" => {
            PlayClientboundPacket::Respawn(session::read(&mut reader, context.registries)?)
        }
        "minecraft:reset_score" => {
            PlayClientboundPacket::ResetScore(scoreboard_codec::read_reset(&mut reader)?)
        }
        "minecraft:remove_entities" => {
            PlayClientboundPacket::RemoveEntities(entity_spawn_codec::read_remove(&mut reader)?)
        }
        "minecraft:remove_mob_effect" => PlayClientboundPacket::RemoveMobEffect(
            entity_effects_codec::read_remove(&mut reader, context.registries)?,
        ),
        "minecraft:rotate_head" => {
            PlayClientboundPacket::RotateHead(entity_motion_codec::read_head(&mut reader)?)
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
        "minecraft:set_display_objective" => {
            PlayClientboundPacket::SetDisplayObjective(scoreboard_codec::read_display(&mut reader)?)
        }
        "minecraft:set_cursor_item" => PlayClientboundPacket::SetCursorItem(
            container_codec::read_cursor(&mut reader, context)?,
        ),
        "minecraft:set_camera" => {
            PlayClientboundPacket::SetCamera(entity_session_codec::read_camera(&mut reader)?)
        }
        "minecraft:set_entity_data" => PlayClientboundPacket::SetEntityData(
            entity_state_codec::read_data(&mut reader, context)?,
        ),
        "minecraft:set_entity_link" => {
            PlayClientboundPacket::SetEntityLink(entity_state_codec::read_link(&mut reader)?)
        }
        "minecraft:set_entity_motion" => {
            PlayClientboundPacket::SetEntityMotion(entity_motion_codec::read_motion(&mut reader)?)
        }
        "minecraft:set_equipment" => PlayClientboundPacket::SetEquipment(
            entity_state_codec::read_equipment(&mut reader, context)?,
        ),
        "minecraft:set_experience" => PlayClientboundPacket::SetExperience(
            player_projection_codec::read_experience(&mut reader)?,
        ),
        "minecraft:set_health" => {
            PlayClientboundPacket::SetHealth(player_projection_codec::read_health(&mut reader)?)
        }
        "minecraft:set_held_slot" => PlayClientboundPacket::SetHeldSlot(reader.read_var_i32()?),
        "minecraft:set_objective" => PlayClientboundPacket::SetObjective(
            scoreboard_codec::read_objective(&mut reader, context.registries)?,
        ),
        "minecraft:set_player_inventory" => PlayClientboundPacket::SetPlayerInventory(
            container_codec::read_player_inventory(&mut reader, context)?,
        ),
        "minecraft:set_player_team" => {
            PlayClientboundPacket::SetPlayerTeam(scoreboard_codec::read_team(&mut reader)?)
        }
        "minecraft:set_score" => PlayClientboundPacket::SetScore(scoreboard_codec::read_score(
            &mut reader,
            context.registries,
        )?),
        "minecraft:set_passengers" => {
            PlayClientboundPacket::SetPassengers(entity_state_codec::read_passengers(&mut reader)?)
        }
        "minecraft:set_time" => {
            PlayClientboundPacket::SetTime(entry_codec::read_time(&mut reader, context.registries)?)
        }
        "minecraft:sound_entity" => PlayClientboundPacket::SoundAtEntity(sound_codec::read_entity(
            &mut reader,
            context.registries,
        )?),
        "minecraft:sound" => PlayClientboundPacket::SoundAtPosition(sound_codec::read_position(
            &mut reader,
            context.registries,
        )?),
        "minecraft:stop_sound" => {
            PlayClientboundPacket::StopSound(sound_codec::read_stop(&mut reader)?)
        }
        "minecraft:system_chat" => {
            PlayClientboundPacket::SystemChat(chat_codec::read_system(&mut reader)?)
        }
        "minecraft:tag_query" => {
            PlayClientboundPacket::TagQuery(inventory_codec::read_tag_query(&mut reader)?)
        }
        "minecraft:take_item_entity" => {
            PlayClientboundPacket::TakeItemEntity(entity_session_codec::read_take(&mut reader)?)
        }
        "minecraft:teleport_entity" => {
            PlayClientboundPacket::TeleportEntity(entity_motion_codec::read_teleport(&mut reader)?)
        }
        "minecraft:ticking_state" => PlayClientboundPacket::TickingState(TickingState {
            tick_rate: reader.read_f32()?,
            frozen: reader.read_bool()?,
        }),
        "minecraft:ticking_step" => PlayClientboundPacket::TickingStep(reader.read_var_i32()?),
        "minecraft:update_advancements" => PlayClientboundPacket::UpdateAdvancements(
            inventory_codec::read_advancements(&mut reader, context)?,
        ),
        "minecraft:update_attributes" => PlayClientboundPacket::UpdateAttributes(
            entity_state_codec::read_attributes(&mut reader, context)?,
        ),
        "minecraft:update_mob_effect" => PlayClientboundPacket::UpdateMobEffect(
            entity_effects_codec::read_update(&mut reader, context.registries)?,
        ),
        "minecraft:update_recipes" => {
            PlayClientboundPacket::UpdateRecipes(recipe::read_projection(&mut reader, context)?)
        }
        "minecraft:waypoint" => {
            PlayClientboundPacket::Waypoint(boss_waypoint_codec::read_waypoint(&mut reader)?)
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
        PlayClientboundPacket::AddEntity(packet) => {
            entity_spawn_codec::write_add(&mut writer, packet)?;
        }
        PlayClientboundPacket::Animate(packet) => {
            entity_session_codec::write_animate(&mut writer, *packet)?;
        }
        PlayClientboundPacket::AwardStats(packet) => {
            player_projection_codec::write_stats(&mut writer, packet, registries)?;
        }
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
        PlayClientboundPacket::BossEvent(packet) => {
            boss_waypoint_codec::write_boss(&mut writer, packet)?;
        }
        PlayClientboundPacket::ChangeDifficulty(packet) => {
            writer.write_var_i32(packet.raw_difficulty)?;
            writer.write_bool(packet.locked)?;
        }
        PlayClientboundPacket::CommandSuggestions(packet) => {
            completion_codec::write_command(&mut writer, packet)?;
        }
        PlayClientboundPacket::Commands(tree) => command::write(&mut writer, tree, registries)?,
        PlayClientboundPacket::ContainerClose(packet) => {
            container_codec::write_close(&mut writer, *packet)?;
        }
        PlayClientboundPacket::ContainerSetContent(packet) => {
            container_codec::write_content(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::ContainerSetData(packet) => {
            container_codec::write_data(&mut writer, *packet)?;
        }
        PlayClientboundPacket::ContainerSetSlot(packet) => {
            container_codec::write_slot(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::Cooldown(packet) => {
            player_projection_codec::write_cooldown(&mut writer, packet)?;
        }
        PlayClientboundPacket::CustomChatCompletions(packet) => {
            completion_codec::write_custom(&mut writer, packet)?;
        }
        PlayClientboundPacket::DamageEvent(packet) => {
            entity_session_codec::write_damage(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::DeleteChat(packet) => {
            chat_codec::write_delete(&mut writer, packet)?;
        }
        PlayClientboundPacket::Disconnect(reason) => reason.network_nbt().write(&mut writer)?,
        PlayClientboundPacket::DisguisedChat(packet) => {
            chat_codec::write_disguised(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::EntityEvent(packet) => {
            writer.write_i32(packet.entity_id)?;
            writer.write_i8(packet.event)?;
        }
        PlayClientboundPacket::EntityPositionSync(packet) => {
            entity_motion_codec::write_position_sync(&mut writer, *packet)?;
        }
        PlayClientboundPacket::Explosion(packet) => {
            entity_effects_codec::write_explosion(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::GameEvent(packet) => {
            writer.write_u8(packet.event)?;
            writer.write_f32(packet.parameter)?;
        }
        PlayClientboundPacket::HurtAnimation(packet) => {
            entity_session_codec::write_hurt(&mut writer, *packet)?;
        }
        PlayClientboundPacket::InitializeBorder(border) => {
            entry_codec::write_border(&mut writer, border)?;
        }
        PlayClientboundPacket::KeepAlive(packet) => writer.write_i64(packet.challenge)?,
        PlayClientboundPacket::Login(login) => {
            entry_codec::write_login(&mut writer, login, registries)?;
        }
        PlayClientboundPacket::LevelParticles(packet) => {
            particle_codec::write(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::MapItemData(packet) => {
            inventory_codec::write_map(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::MerchantOffers(packet) => {
            merchant_codec::write(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::MountScreenOpen(packet) => {
            special_screen_codec::write_mount(&mut writer, *packet)?;
        }
        PlayClientboundPacket::MoveEntityPosition(packet) => {
            entity_motion_codec::write_position(&mut writer, *packet)?;
        }
        PlayClientboundPacket::MoveEntityPositionRotation(packet) => {
            entity_motion_codec::write_position_rotation(&mut writer, *packet)?;
        }
        PlayClientboundPacket::MoveEntityRotation(packet) => {
            entity_motion_codec::write_rotation(&mut writer, *packet)?;
        }
        PlayClientboundPacket::MoveMinecartAlongTrack(packet) => {
            entity_motion_codec::write_minecart(&mut writer, packet)?;
        }
        PlayClientboundPacket::MoveVehicle(packet) => {
            entry_codec::write_vector(&mut writer, packet.position)?;
            writer.write_f32(packet.yaw)?;
            writer.write_f32(packet.pitch)?;
        }
        PlayClientboundPacket::OpenScreen(packet) => {
            container_codec::write_open(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::OpenBook(hand) => {
            special_screen_codec::write_hand(&mut writer, *hand)?;
        }
        PlayClientboundPacket::OpenSignEditor(packet) => {
            special_screen_codec::write_sign(&mut writer, *packet)?;
        }
        PlayClientboundPacket::Ping(packet) => writer.write_i32(packet.payload)?,
        PlayClientboundPacket::PlayerAbilities(abilities) => {
            writer.write_u8(abilities.flags)?;
            writer.write_f32(abilities.flying_speed)?;
            writer.write_f32(abilities.walking_speed)?;
        }
        PlayClientboundPacket::PlayerChat(packet) => {
            chat_codec::write_player(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::PlayerCombatEnd(packet) => {
            combat_look_codec::write_end(&mut writer, *packet)?;
        }
        PlayClientboundPacket::PlayerCombatEnter => {}
        PlayClientboundPacket::PlayerCombatKill(packet) => {
            combat_look_codec::write_kill(&mut writer, packet)?;
        }
        PlayClientboundPacket::PlayerInfoUpdate(update) => {
            player_info::write(&mut writer, update)?;
        }
        PlayClientboundPacket::PlayerInfoRemove(packet) => {
            player_info_remove::codec::write(&mut writer, packet)?;
        }
        PlayClientboundPacket::PlayerLookAt(packet) => {
            combat_look_codec::write_look(&mut writer, *packet)?;
        }
        PlayClientboundPacket::PlayerPosition(position) => {
            entry_codec::write_position(&mut writer, position)?;
        }
        PlayClientboundPacket::PlayerRotation(rotation) => {
            writer.write_f32(rotation.yaw)?;
            writer.write_bool(rotation.relative_yaw)?;
            writer.write_f32(rotation.pitch)?;
            writer.write_bool(rotation.relative_pitch)?;
        }
        PlayClientboundPacket::ProjectilePower(packet) => {
            entity_motion_codec::write_projectile(&mut writer, *packet)?;
        }
        PlayClientboundPacket::PlaceGhostRecipe(packet) => {
            recipe::book::codec::write_ghost(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::RecipeBookAdd(packet) => {
            recipe::write_book_add(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::RecipeBookRemove(packet) => {
            recipe::book::codec::write_remove(&mut writer, packet)?;
        }
        PlayClientboundPacket::RecipeBookSettings(settings) => {
            recipe::write_book_settings(&mut writer, *settings)?;
        }
        PlayClientboundPacket::Respawn(packet) => {
            session::write(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::ResetScore(packet) => {
            scoreboard_codec::write_reset(&mut writer, packet)?;
        }
        PlayClientboundPacket::RemoveEntities(packet) => {
            entity_spawn_codec::write_remove(&mut writer, packet)?;
        }
        PlayClientboundPacket::RemoveMobEffect(packet) => {
            entity_effects_codec::write_remove(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::RotateHead(packet) => {
            entity_motion_codec::write_head(&mut writer, *packet)?;
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
        PlayClientboundPacket::SetDisplayObjective(packet) => {
            scoreboard_codec::write_display(&mut writer, packet)?;
        }
        PlayClientboundPacket::SetCursorItem(packet) => {
            container_codec::write_cursor(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::SetCamera(packet) => {
            entity_session_codec::write_camera(&mut writer, *packet)?;
        }
        PlayClientboundPacket::SetEntityData(packet) => {
            entity_state_codec::write_data(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::SetEntityLink(packet) => {
            entity_state_codec::write_link(&mut writer, *packet)?;
        }
        PlayClientboundPacket::SetEntityMotion(packet) => {
            entity_motion_codec::write_motion(&mut writer, *packet)?;
        }
        PlayClientboundPacket::SetEquipment(packet) => {
            entity_state_codec::write_equipment(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::SetExperience(packet) => {
            player_projection_codec::write_experience(&mut writer, *packet)?;
        }
        PlayClientboundPacket::SetHealth(packet) => {
            player_projection_codec::write_health(&mut writer, *packet)?;
        }
        PlayClientboundPacket::SetHeldSlot(slot) => writer.write_var_i32(*slot)?,
        PlayClientboundPacket::SetObjective(packet) => {
            scoreboard_codec::write_objective(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::SetPlayerInventory(packet) => {
            container_codec::write_player_inventory(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::SetPlayerTeam(packet) => {
            scoreboard_codec::write_team(&mut writer, packet)?;
        }
        PlayClientboundPacket::SetScore(packet) => {
            scoreboard_codec::write_score(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::SetPassengers(packet) => {
            entity_state_codec::write_passengers(&mut writer, packet)?;
        }
        PlayClientboundPacket::SetTime(time) => {
            entry_codec::write_time(&mut writer, time, registries)?;
        }
        PlayClientboundPacket::SoundAtEntity(packet) => {
            sound_codec::write_entity(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::SoundAtPosition(packet) => {
            sound_codec::write_position(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::StopSound(packet) => {
            sound_codec::write_stop(&mut writer, packet)?;
        }
        PlayClientboundPacket::SystemChat(packet) => {
            chat_codec::write_system(&mut writer, packet)?;
        }
        PlayClientboundPacket::TagQuery(packet) => {
            inventory_codec::write_tag_query(&mut writer, packet)?;
        }
        PlayClientboundPacket::TakeItemEntity(packet) => {
            entity_session_codec::write_take(&mut writer, *packet)?;
        }
        PlayClientboundPacket::TeleportEntity(packet) => {
            entity_motion_codec::write_teleport(&mut writer, *packet)?;
        }
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
        PlayClientboundPacket::UpdateAdvancements(packet) => {
            inventory_codec::write_advancements(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::UpdateAttributes(packet) => {
            entity_state_codec::write_attributes(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::UpdateMobEffect(packet) => {
            entity_effects_codec::write_update(&mut writer, packet, registries)?;
        }
        PlayClientboundPacket::UpdateRecipes(projection) => {
            recipe::write_projection(&mut writer, projection, registries)?;
        }
        PlayClientboundPacket::Waypoint(packet) => {
            boss_waypoint_codec::write_waypoint(&mut writer, packet)?;
        }
    }
    Ok(writer.into_inner())
}

pub(crate) fn packet_identity(packet: &PlayClientboundPacket) -> &'static str {
    match packet {
        PlayClientboundPacket::AddEntity(_) => "minecraft:add_entity",
        PlayClientboundPacket::Animate(_) => "minecraft:animate",
        PlayClientboundPacket::AwardStats(_) => "minecraft:award_stats",
        PlayClientboundPacket::BlockChangedAck(_) => "minecraft:block_changed_ack",
        PlayClientboundPacket::BlockDestruction(_) => "minecraft:block_destruction",
        PlayClientboundPacket::BlockEntityData(_) => "minecraft:block_entity_data",
        PlayClientboundPacket::BlockEvent(_) => "minecraft:block_event",
        PlayClientboundPacket::BlockUpdate(_) => "minecraft:block_update",
        PlayClientboundPacket::BossEvent(_) => "minecraft:boss_event",
        PlayClientboundPacket::ChangeDifficulty(_) => "minecraft:change_difficulty",
        PlayClientboundPacket::CommandSuggestions(_) => "minecraft:command_suggestions",
        PlayClientboundPacket::Commands(_) => "minecraft:commands",
        PlayClientboundPacket::ContainerClose(_) => "minecraft:container_close",
        PlayClientboundPacket::ContainerSetContent(_) => "minecraft:container_set_content",
        PlayClientboundPacket::ContainerSetData(_) => "minecraft:container_set_data",
        PlayClientboundPacket::ContainerSetSlot(_) => "minecraft:container_set_slot",
        PlayClientboundPacket::Cooldown(_) => "minecraft:cooldown",
        PlayClientboundPacket::CustomChatCompletions(_) => "minecraft:custom_chat_completions",
        PlayClientboundPacket::DamageEvent(_) => "minecraft:damage_event",
        PlayClientboundPacket::DeleteChat(_) => "minecraft:delete_chat",
        PlayClientboundPacket::Disconnect(_) => "minecraft:disconnect",
        PlayClientboundPacket::DisguisedChat(_) => "minecraft:disguised_chat",
        PlayClientboundPacket::EntityEvent(_) => "minecraft:entity_event",
        PlayClientboundPacket::EntityPositionSync(_) => "minecraft:entity_position_sync",
        PlayClientboundPacket::Explosion(_) => "minecraft:explode",
        PlayClientboundPacket::GameEvent(_) => "minecraft:game_event",
        PlayClientboundPacket::HurtAnimation(_) => "minecraft:hurt_animation",
        PlayClientboundPacket::InitializeBorder(_) => "minecraft:initialize_border",
        PlayClientboundPacket::KeepAlive(_) => "minecraft:keep_alive",
        PlayClientboundPacket::Login(_) => "minecraft:login",
        PlayClientboundPacket::LevelParticles(_) => "minecraft:level_particles",
        PlayClientboundPacket::MapItemData(_) => "minecraft:map_item_data",
        PlayClientboundPacket::MerchantOffers(_) => "minecraft:merchant_offers",
        PlayClientboundPacket::MountScreenOpen(_) => "minecraft:mount_screen_open",
        PlayClientboundPacket::MoveEntityPosition(_) => "minecraft:move_entity_pos",
        PlayClientboundPacket::MoveEntityPositionRotation(_) => "minecraft:move_entity_pos_rot",
        PlayClientboundPacket::MoveEntityRotation(_) => "minecraft:move_entity_rot",
        PlayClientboundPacket::MoveMinecartAlongTrack(_) => "minecraft:move_minecart_along_track",
        PlayClientboundPacket::MoveVehicle(_) => "minecraft:move_vehicle",
        PlayClientboundPacket::OpenBook(_) => "minecraft:open_book",
        PlayClientboundPacket::OpenScreen(_) => "minecraft:open_screen",
        PlayClientboundPacket::OpenSignEditor(_) => "minecraft:open_sign_editor",
        PlayClientboundPacket::Ping(_) => "minecraft:ping",
        PlayClientboundPacket::PlayerAbilities(_) => "minecraft:player_abilities",
        PlayClientboundPacket::PlayerChat(_) => "minecraft:player_chat",
        PlayClientboundPacket::PlayerCombatEnd(_) => "minecraft:player_combat_end",
        PlayClientboundPacket::PlayerCombatEnter => "minecraft:player_combat_enter",
        PlayClientboundPacket::PlayerCombatKill(_) => "minecraft:player_combat_kill",
        PlayClientboundPacket::PlayerInfoUpdate(_) => "minecraft:player_info_update",
        PlayClientboundPacket::PlayerInfoRemove(_) => "minecraft:player_info_remove",
        PlayClientboundPacket::PlayerLookAt(_) => "minecraft:player_look_at",
        PlayClientboundPacket::PlayerPosition(_) => "minecraft:player_position",
        PlayClientboundPacket::PlayerRotation(_) => "minecraft:player_rotation",
        PlayClientboundPacket::ProjectilePower(_) => "minecraft:projectile_power",
        PlayClientboundPacket::PlaceGhostRecipe(_) => "minecraft:place_ghost_recipe",
        PlayClientboundPacket::RecipeBookAdd(_) => "minecraft:recipe_book_add",
        PlayClientboundPacket::RecipeBookRemove(_) => "minecraft:recipe_book_remove",
        PlayClientboundPacket::RecipeBookSettings(_) => "minecraft:recipe_book_settings",
        PlayClientboundPacket::Respawn(_) => "minecraft:respawn",
        PlayClientboundPacket::ResetScore(_) => "minecraft:reset_score",
        PlayClientboundPacket::RemoveEntities(_) => "minecraft:remove_entities",
        PlayClientboundPacket::RemoveMobEffect(_) => "minecraft:remove_mob_effect",
        PlayClientboundPacket::RotateHead(_) => "minecraft:rotate_head",
        PlayClientboundPacket::ServerData(_) => "minecraft:server_data",
        PlayClientboundPacket::SectionBlocksUpdate(_) => "minecraft:section_blocks_update",
        PlayClientboundPacket::SetDefaultSpawnPosition(_) => "minecraft:set_default_spawn_position",
        PlayClientboundPacket::SetDisplayObjective(_) => "minecraft:set_display_objective",
        PlayClientboundPacket::SetCursorItem(_) => "minecraft:set_cursor_item",
        PlayClientboundPacket::SetCamera(_) => "minecraft:set_camera",
        PlayClientboundPacket::SetEntityData(_) => "minecraft:set_entity_data",
        PlayClientboundPacket::SetEntityLink(_) => "minecraft:set_entity_link",
        PlayClientboundPacket::SetEntityMotion(_) => "minecraft:set_entity_motion",
        PlayClientboundPacket::SetEquipment(_) => "minecraft:set_equipment",
        PlayClientboundPacket::SetExperience(_) => "minecraft:set_experience",
        PlayClientboundPacket::SetHealth(_) => "minecraft:set_health",
        PlayClientboundPacket::SetHeldSlot(_) => "minecraft:set_held_slot",
        PlayClientboundPacket::SetObjective(_) => "minecraft:set_objective",
        PlayClientboundPacket::SetPlayerInventory(_) => "minecraft:set_player_inventory",
        PlayClientboundPacket::SetPlayerTeam(_) => "minecraft:set_player_team",
        PlayClientboundPacket::SetScore(_) => "minecraft:set_score",
        PlayClientboundPacket::SetPassengers(_) => "minecraft:set_passengers",
        PlayClientboundPacket::SetTime(_) => "minecraft:set_time",
        PlayClientboundPacket::SoundAtEntity(_) => "minecraft:sound_entity",
        PlayClientboundPacket::SoundAtPosition(_) => "minecraft:sound",
        PlayClientboundPacket::StopSound(_) => "minecraft:stop_sound",
        PlayClientboundPacket::SystemChat(_) => "minecraft:system_chat",
        PlayClientboundPacket::TagQuery(_) => "minecraft:tag_query",
        PlayClientboundPacket::TakeItemEntity(_) => "minecraft:take_item_entity",
        PlayClientboundPacket::TeleportEntity(_) => "minecraft:teleport_entity",
        PlayClientboundPacket::Terrain(packet) => terrain_codec::identity(packet),
        PlayClientboundPacket::TickingState(_) => "minecraft:ticking_state",
        PlayClientboundPacket::TickingStep(_) => "minecraft:ticking_step",
        PlayClientboundPacket::UpdateAdvancements(_) => "minecraft:update_advancements",
        PlayClientboundPacket::UpdateAttributes(_) => "minecraft:update_attributes",
        PlayClientboundPacket::UpdateMobEffect(_) => "minecraft:update_mob_effect",
        PlayClientboundPacket::UpdateRecipes(_) => "minecraft:update_recipes",
        PlayClientboundPacket::Waypoint(_) => "minecraft:waypoint",
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

pub(super) fn read_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Identifier, PlayClientboundCodecError> {
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
