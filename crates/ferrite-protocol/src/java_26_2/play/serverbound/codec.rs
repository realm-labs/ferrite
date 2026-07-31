use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::configuration::serverbound::codec::{
    ConfigurationServerboundCodecError, decode_client_information_body,
    encode_client_information_body,
};
use crate::java_26_2::play::block::{
    direction_from_index, direction_from_player_action, direction_index, pack_block_position,
    unpack_block_position,
};
use crate::java_26_2::play::registry::PlayRegistries;
use crate::java_26_2::play::serverbound::anvil_beacon::codec::{
    AnvilBeaconCodecError, decode_rename, decode_set_beacon, encode_rename, encode_set_beacon,
};
use crate::java_26_2::play::serverbound::chat::codec::{
    ChatCodecError, decode_ack, decode_chat, decode_command, decode_session, decode_signed_command,
    decode_suggestion, encode_ack, encode_chat, encode_command, encode_session,
    encode_signed_command, encode_suggestion,
};
use crate::java_26_2::play::serverbound::container::codec::{
    ContainerServerboundCodecError, decode_button, decode_click, decode_close, decode_set_carried,
    decode_slot_state, encode_button, encode_click, encode_close, encode_set_carried,
    encode_slot_state,
};
use crate::java_26_2::play::serverbound::entity_session::codec::{
    EntitySessionCodecError, decode_attack, decode_client_command, decode_interact, decode_pick,
    decode_spectator_action, decode_teleport, encode_attack, encode_client_command,
    encode_interact, encode_pick, encode_spectator_action, encode_teleport,
};
use crate::java_26_2::play::serverbound::inventory_auxiliary::codec::{
    InventoryAuxiliaryCodecError, decode_bundle_selection, decode_edit_book,
    decode_seen_advancements, encode_bundle_selection, encode_edit_book, encode_seen_advancements,
};
use crate::java_26_2::play::serverbound::merchant::codec::{
    decode_select_trade, encode_select_trade,
};
use crate::java_26_2::play::serverbound::packet::{
    AcceptTeleportation, BlockHit, ChunkBatchReceived, Hand, KeepAlive, MovePlayerPosition,
    MovePlayerPositionRotation, MovePlayerRotation, MovePlayerStatusOnly, MoveVehicle,
    MovementFlags, PaddleBoat, PickItemFromBlock, PlayServerboundEntryPacket, PlayerAbilities,
    PlayerAction, PlayerActionKind, PlayerCommand, PlayerCommandKind, PlayerInput, PlayerPosition,
    PlayerRotation, Pong, Swing, UseItem, UseItemOn,
};
use crate::java_26_2::play::serverbound::recipe_book::codec::{
    RecipeBookServerboundCodecError, decode_change_settings, decode_place_recipe,
    decode_seen_recipe, encode_change_settings, encode_place_recipe, encode_seen_recipe,
};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const ACCEPT_TELEPORTATION: &str = "minecraft:accept_teleportation";
const ATTACK: &str = "minecraft:attack";
const BUNDLE_ITEM_SELECTED: &str = "minecraft:bundle_item_selected";
const CHAT_ACK: &str = "minecraft:chat_ack";
const CHAT_COMMAND: &str = "minecraft:chat_command";
const CHAT_COMMAND_SIGNED: &str = "minecraft:chat_command_signed";
const CHAT: &str = "minecraft:chat";
const CHAT_SESSION_UPDATE: &str = "minecraft:chat_session_update";
const CHUNK_BATCH_RECEIVED: &str = "minecraft:chunk_batch_received";
const CLIENT_TICK_END: &str = "minecraft:client_tick_end";
const CLIENT_INFORMATION: &str = "minecraft:client_information";
const CLIENT_COMMAND: &str = "minecraft:client_command";
const CONTAINER_BUTTON_CLICK: &str = "minecraft:container_button_click";
const CONTAINER_CLICK: &str = "minecraft:container_click";
const CONTAINER_CLOSE: &str = "minecraft:container_close";
const CONTAINER_SLOT_STATE_CHANGED: &str = "minecraft:container_slot_state_changed";
const COMMAND_SUGGESTION: &str = "minecraft:command_suggestion";
const EDIT_BOOK: &str = "minecraft:edit_book";
const KEEP_ALIVE: &str = "minecraft:keep_alive";
const INTERACT: &str = "minecraft:interact";
const MOVE_PLAYER_POS: &str = "minecraft:move_player_pos";
const MOVE_PLAYER_POS_ROT: &str = "minecraft:move_player_pos_rot";
const MOVE_PLAYER_ROT: &str = "minecraft:move_player_rot";
const MOVE_PLAYER_STATUS_ONLY: &str = "minecraft:move_player_status_only";
const MOVE_VEHICLE: &str = "minecraft:move_vehicle";
const PADDLE_BOAT: &str = "minecraft:paddle_boat";
const PICK_ITEM_FROM_BLOCK: &str = "minecraft:pick_item_from_block";
const PICK_ITEM_FROM_ENTITY: &str = "minecraft:pick_item_from_entity";
const PLAYER_ACTION: &str = "minecraft:player_action";
const PLAYER_ABILITIES: &str = "minecraft:player_abilities";
const PLAYER_COMMAND: &str = "minecraft:player_command";
const PLAYER_INPUT: &str = "minecraft:player_input";
const PLAYER_LOADED: &str = "minecraft:player_loaded";
const PONG: &str = "minecraft:pong";
const PLACE_RECIPE: &str = "minecraft:place_recipe";
const RECIPE_BOOK_CHANGE_SETTINGS: &str = "minecraft:recipe_book_change_settings";
const RECIPE_BOOK_SEEN_RECIPE: &str = "minecraft:recipe_book_seen_recipe";
const RENAME_ITEM: &str = "minecraft:rename_item";
const SEEN_ADVANCEMENTS: &str = "minecraft:seen_advancements";
const SELECT_TRADE: &str = "minecraft:select_trade";
const SET_BEACON: &str = "minecraft:set_beacon";
const SET_CARRIED_ITEM: &str = "minecraft:set_carried_item";
const SPECTATOR_ACTION: &str = "minecraft:spectator_action";
const SWING: &str = "minecraft:swing";
const TELEPORT_TO_ENTITY: &str = "minecraft:teleport_to_entity";
const USE_ITEM_ON: &str = "minecraft:use_item_on";
const USE_ITEM: &str = "minecraft:use_item";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlayServerboundEntryCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    ClientInformation(#[from] ConfigurationServerboundCodecError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error(transparent)]
    AnvilBeacon(#[from] AnvilBeaconCodecError),
    #[error(transparent)]
    Container(#[from] ContainerServerboundCodecError),
    #[error(transparent)]
    EntitySession(#[from] EntitySessionCodecError),
    #[error(transparent)]
    InventoryAuxiliary(#[from] InventoryAuxiliaryCodecError),
    #[error(transparent)]
    Chat(#[from] ChatCodecError),
    #[error(transparent)]
    RecipeBook(#[from] RecipeBookServerboundCodecError),
    #[error("play serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("play serverbound packet {identity} is outside the implemented required families")]
    UnsupportedPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing required packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
    #[error("play serverbound packet {identity} requires a configured registry snapshot")]
    MissingRegistryContext { identity: &'static str },
    #[error("{field} ordinal {value} is invalid")]
    InvalidEnum { field: &'static str, value: i32 },
}

pub fn decode_packet(
    body: &[u8],
) -> Result<PlayServerboundEntryPacket, PlayServerboundEntryCodecError> {
    decode_packet_inner(body, None)
}

pub fn decode_packet_with_registries(
    body: &[u8],
    registries: &PlayRegistries,
) -> Result<PlayServerboundEntryPacket, PlayServerboundEntryCodecError> {
    decode_packet_inner(body, Some(registries))
}

fn decode_packet_inner(
    body: &[u8],
    registries: Option<&PlayRegistries>,
) -> Result<PlayServerboundEntryPacket, PlayServerboundEntryCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor =
        PacketCatalog::by_wire_id(ConnectionState::Play, PacketDirection::Serverbound, wire_id)
            .ok_or(PlayServerboundEntryCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        ACCEPT_TELEPORTATION => {
            PlayServerboundEntryPacket::AcceptTeleportation(AcceptTeleportation {
                challenge: reader.read_var_i32()?,
            })
        }
        ATTACK => PlayServerboundEntryPacket::Attack(decode_attack(&mut reader)?),
        BUNDLE_ITEM_SELECTED => {
            PlayServerboundEntryPacket::BundleItemSelected(decode_bundle_selection(&mut reader)?)
        }
        CHAT_ACK => PlayServerboundEntryPacket::ChatAck(decode_ack(&mut reader)?),
        CHAT_COMMAND => PlayServerboundEntryPacket::ChatCommand(decode_command(&mut reader)?),
        CHAT_COMMAND_SIGNED => {
            PlayServerboundEntryPacket::ChatCommandSigned(decode_signed_command(&mut reader)?)
        }
        CHAT => PlayServerboundEntryPacket::ChatMessage(decode_chat(&mut reader)?),
        CHAT_SESSION_UPDATE => {
            PlayServerboundEntryPacket::ChatSessionUpdate(decode_session(&mut reader)?)
        }
        CHUNK_BATCH_RECEIVED => {
            PlayServerboundEntryPacket::ChunkBatchReceived(ChunkBatchReceived {
                desired_chunks_per_tick: reader.read_f32()?,
            })
        }
        CLIENT_TICK_END => PlayServerboundEntryPacket::ClientTickEnd,
        CLIENT_INFORMATION => PlayServerboundEntryPacket::ClientInformation(
            decode_client_information_body(&mut reader)?,
        ),
        CLIENT_COMMAND => {
            PlayServerboundEntryPacket::ClientCommand(decode_client_command(&mut reader)?)
        }
        CONTAINER_BUTTON_CLICK => {
            PlayServerboundEntryPacket::ContainerButtonClick(decode_button(&mut reader)?)
        }
        CONTAINER_CLICK => {
            let registries =
                registries.ok_or(PlayServerboundEntryCodecError::MissingRegistryContext {
                    identity: CONTAINER_CLICK,
                })?;
            PlayServerboundEntryPacket::ContainerClick(decode_click(&mut reader, registries)?)
        }
        CONTAINER_CLOSE => PlayServerboundEntryPacket::ContainerClose(decode_close(&mut reader)?),
        CONTAINER_SLOT_STATE_CHANGED => {
            PlayServerboundEntryPacket::ContainerSlotStateChanged(decode_slot_state(&mut reader)?)
        }
        COMMAND_SUGGESTION => {
            PlayServerboundEntryPacket::CommandSuggestion(decode_suggestion(&mut reader)?)
        }
        EDIT_BOOK => PlayServerboundEntryPacket::EditBook(decode_edit_book(&mut reader)?),
        KEEP_ALIVE => PlayServerboundEntryPacket::KeepAlive(KeepAlive {
            challenge: reader.read_i64()?,
        }),
        INTERACT => PlayServerboundEntryPacket::Interact(decode_interact(&mut reader)?),
        MOVE_PLAYER_POS => PlayServerboundEntryPacket::MovePlayerPosition(MovePlayerPosition {
            position: read_position(&mut reader)?,
            flags: MovementFlags::from_wire(reader.read_u8()?),
        }),
        MOVE_PLAYER_POS_ROT => {
            PlayServerboundEntryPacket::MovePlayerPositionRotation(MovePlayerPositionRotation {
                position: read_position(&mut reader)?,
                rotation: read_rotation(&mut reader)?,
                flags: MovementFlags::from_wire(reader.read_u8()?),
            })
        }
        MOVE_PLAYER_ROT => PlayServerboundEntryPacket::MovePlayerRotation(MovePlayerRotation {
            rotation: read_rotation(&mut reader)?,
            flags: MovementFlags::from_wire(reader.read_u8()?),
        }),
        MOVE_PLAYER_STATUS_ONLY => {
            PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
                flags: MovementFlags::from_wire(reader.read_u8()?),
            })
        }
        MOVE_VEHICLE => PlayServerboundEntryPacket::MoveVehicle(MoveVehicle {
            position: read_position(&mut reader)?,
            rotation: read_rotation(&mut reader)?,
            on_ground: reader.read_bool()?,
        }),
        PADDLE_BOAT => PlayServerboundEntryPacket::PaddleBoat(PaddleBoat {
            left: reader.read_bool()?,
            right: reader.read_bool()?,
        }),
        PICK_ITEM_FROM_BLOCK => PlayServerboundEntryPacket::PickItemFromBlock(PickItemFromBlock {
            position: unpack_block_position(reader.read_i64()?),
            include_data: reader.read_bool()?,
        }),
        PICK_ITEM_FROM_ENTITY => {
            PlayServerboundEntryPacket::PickItemFromEntity(decode_pick(&mut reader)?)
        }
        PLAYER_ACTION => PlayServerboundEntryPacket::PlayerAction(PlayerAction {
            action: read_action(&mut reader)?,
            position: unpack_block_position(reader.read_i64()?),
            direction: direction_from_player_action(reader.read_u8()?),
            sequence: reader.read_var_i32()?,
        }),
        PLAYER_ABILITIES => PlayServerboundEntryPacket::PlayerAbilities(PlayerAbilities {
            flying: reader.read_u8()? & 0x02 != 0,
        }),
        PLAYER_COMMAND => PlayServerboundEntryPacket::PlayerCommand(PlayerCommand {
            entity_id: reader.read_var_i32()?,
            action: read_player_command(&mut reader)?,
            data: reader.read_var_i32()?,
        }),
        PLAYER_INPUT => {
            PlayServerboundEntryPacket::PlayerInput(PlayerInput::from_wire(reader.read_u8()?))
        }
        PLAYER_LOADED => PlayServerboundEntryPacket::PlayerLoaded,
        PONG => PlayServerboundEntryPacket::Pong(Pong {
            payload: reader.read_i32()?,
        }),
        PLACE_RECIPE => PlayServerboundEntryPacket::PlaceRecipe(decode_place_recipe(&mut reader)?),
        RECIPE_BOOK_CHANGE_SETTINGS => PlayServerboundEntryPacket::RecipeBookChangeSettings(
            decode_change_settings(&mut reader)?,
        ),
        RECIPE_BOOK_SEEN_RECIPE => {
            PlayServerboundEntryPacket::RecipeBookSeenRecipe(decode_seen_recipe(&mut reader)?)
        }
        RENAME_ITEM => PlayServerboundEntryPacket::RenameItem(decode_rename(&mut reader)?),
        SEEN_ADVANCEMENTS => {
            PlayServerboundEntryPacket::SeenAdvancements(decode_seen_advancements(&mut reader)?)
        }
        SELECT_TRADE => PlayServerboundEntryPacket::SelectTrade(decode_select_trade(&mut reader)?),
        SET_BEACON => {
            let registries =
                registries.ok_or(PlayServerboundEntryCodecError::MissingRegistryContext {
                    identity: SET_BEACON,
                })?;
            PlayServerboundEntryPacket::SetBeacon(decode_set_beacon(&mut reader, registries)?)
        }
        SET_CARRIED_ITEM => {
            PlayServerboundEntryPacket::SetCarriedItem(decode_set_carried(&mut reader)?)
        }
        SPECTATOR_ACTION => {
            PlayServerboundEntryPacket::SpectatorAction(decode_spectator_action(&mut reader)?)
        }
        SWING => PlayServerboundEntryPacket::Swing(Swing {
            hand: read_hand(&mut reader)?,
        }),
        TELEPORT_TO_ENTITY => {
            PlayServerboundEntryPacket::TeleportToEntity(decode_teleport(&mut reader)?)
        }
        USE_ITEM_ON => PlayServerboundEntryPacket::UseItemOn(UseItemOn {
            hand: read_hand(&mut reader)?,
            hit: read_block_hit(&mut reader)?,
            sequence: reader.read_var_i32()?,
        }),
        USE_ITEM => PlayServerboundEntryPacket::UseItem(UseItem {
            hand: read_hand(&mut reader)?,
            sequence: reader.read_var_i32()?,
            yaw: reader.read_f32()?,
            pitch: reader.read_f32()?,
        }),
        identity => {
            return Err(PlayServerboundEntryCodecError::UnsupportedPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: PlayServerboundEntryPacket,
) -> Result<Vec<u8>, PlayServerboundEntryCodecError> {
    encode_packet_inner(packet, None)
}

pub fn encode_packet_with_registries(
    packet: PlayServerboundEntryPacket,
    registries: &PlayRegistries,
) -> Result<Vec<u8>, PlayServerboundEntryCodecError> {
    encode_packet_inner(packet, Some(registries))
}

fn encode_packet_inner(
    packet: PlayServerboundEntryPacket,
    registries: Option<&PlayRegistries>,
) -> Result<Vec<u8>, PlayServerboundEntryCodecError> {
    let identity = packet_identity(&packet);
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Play,
        PacketDirection::Serverbound,
        identity,
    )
    .ok_or(PlayServerboundEntryCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        PlayServerboundEntryPacket::AcceptTeleportation(packet) => {
            writer.write_var_i32(packet.challenge)?;
        }
        PlayServerboundEntryPacket::Attack(packet) => {
            encode_attack(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::BundleItemSelected(packet) => {
            encode_bundle_selection(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::ChatAck(packet) => encode_ack(&mut writer, packet)?,
        PlayServerboundEntryPacket::ChatCommand(packet) => encode_command(&mut writer, &packet)?,
        PlayServerboundEntryPacket::ChatCommandSigned(packet) => {
            encode_signed_command(&mut writer, &packet)?;
        }
        PlayServerboundEntryPacket::ChatMessage(packet) => encode_chat(&mut writer, &packet)?,
        PlayServerboundEntryPacket::ChatSessionUpdate(packet) => {
            encode_session(&mut writer, &packet)?;
        }
        PlayServerboundEntryPacket::ChunkBatchReceived(packet) => {
            writer.write_f32(packet.desired_chunks_per_tick)?;
        }
        PlayServerboundEntryPacket::ClientTickEnd | PlayServerboundEntryPacket::PlayerLoaded => {}
        PlayServerboundEntryPacket::ClientInformation(information) => {
            encode_client_information_body(&mut writer, &information)?;
        }
        PlayServerboundEntryPacket::ClientCommand(packet) => {
            encode_client_command(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::ContainerButtonClick(packet) => {
            encode_button(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::ContainerClick(packet) => {
            let registries =
                registries.ok_or(PlayServerboundEntryCodecError::MissingRegistryContext {
                    identity: CONTAINER_CLICK,
                })?;
            encode_click(&mut writer, &packet, registries)?;
        }
        PlayServerboundEntryPacket::ContainerClose(packet) => {
            encode_close(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::ContainerSlotStateChanged(packet) => {
            encode_slot_state(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::CommandSuggestion(packet) => {
            encode_suggestion(&mut writer, &packet)?;
        }
        PlayServerboundEntryPacket::EditBook(packet) => {
            encode_edit_book(&mut writer, &packet)?;
        }
        PlayServerboundEntryPacket::KeepAlive(packet) => writer.write_i64(packet.challenge)?,
        PlayServerboundEntryPacket::Interact(packet) => {
            encode_interact(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::MovePlayerPosition(packet) => {
            write_position(&mut writer, packet.position)?;
            writer.write_u8(packet.flags.to_wire())?;
        }
        PlayServerboundEntryPacket::MovePlayerPositionRotation(packet) => {
            write_position(&mut writer, packet.position)?;
            write_rotation(&mut writer, packet.rotation)?;
            writer.write_u8(packet.flags.to_wire())?;
        }
        PlayServerboundEntryPacket::MovePlayerRotation(packet) => {
            write_rotation(&mut writer, packet.rotation)?;
            writer.write_u8(packet.flags.to_wire())?;
        }
        PlayServerboundEntryPacket::MovePlayerStatusOnly(packet) => {
            writer.write_u8(packet.flags.to_wire())?;
        }
        PlayServerboundEntryPacket::MoveVehicle(packet) => {
            write_position(&mut writer, packet.position)?;
            write_rotation(&mut writer, packet.rotation)?;
            writer.write_bool(packet.on_ground)?;
        }
        PlayServerboundEntryPacket::PaddleBoat(packet) => {
            writer.write_bool(packet.left)?;
            writer.write_bool(packet.right)?;
        }
        PlayServerboundEntryPacket::PickItemFromBlock(packet) => {
            writer.write_i64(pack_block_position(packet.position))?;
            writer.write_bool(packet.include_data)?;
        }
        PlayServerboundEntryPacket::PickItemFromEntity(packet) => {
            encode_pick(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::PlayerAction(packet) => {
            writer.write_var_i32(packet.action.index())?;
            writer.write_i64(pack_block_position(packet.position))?;
            writer.write_u8(direction_index(packet.direction) as u8)?;
            writer.write_var_i32(packet.sequence)?;
        }
        PlayServerboundEntryPacket::PlayerAbilities(packet) => {
            writer.write_u8(if packet.flying { 0x02 } else { 0 })?;
        }
        PlayServerboundEntryPacket::PlayerCommand(packet) => {
            writer.write_var_i32(packet.entity_id)?;
            writer.write_var_i32(packet.action.index())?;
            writer.write_var_i32(packet.data)?;
        }
        PlayServerboundEntryPacket::PlayerInput(packet) => {
            writer.write_u8(packet.to_wire())?;
        }
        PlayServerboundEntryPacket::Pong(packet) => writer.write_i32(packet.payload)?,
        PlayServerboundEntryPacket::PlaceRecipe(packet) => {
            encode_place_recipe(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::RecipeBookChangeSettings(packet) => {
            encode_change_settings(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::RecipeBookSeenRecipe(packet) => {
            encode_seen_recipe(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::RenameItem(packet) => {
            encode_rename(&mut writer, &packet)?;
        }
        PlayServerboundEntryPacket::SeenAdvancements(packet) => {
            encode_seen_advancements(&mut writer, &packet)?;
        }
        PlayServerboundEntryPacket::SelectTrade(packet) => {
            encode_select_trade(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::SetCarriedItem(packet) => {
            encode_set_carried(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::SpectatorAction(packet) => {
            encode_spectator_action(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::SetBeacon(packet) => {
            let registries =
                registries.ok_or(PlayServerboundEntryCodecError::MissingRegistryContext {
                    identity: SET_BEACON,
                })?;
            encode_set_beacon(&mut writer, &packet, registries)?;
        }
        PlayServerboundEntryPacket::Swing(packet) => {
            writer.write_var_i32(packet.hand.index())?;
        }
        PlayServerboundEntryPacket::TeleportToEntity(packet) => {
            encode_teleport(&mut writer, packet)?;
        }
        PlayServerboundEntryPacket::UseItemOn(packet) => {
            writer.write_var_i32(packet.hand.index())?;
            write_block_hit(&mut writer, packet.hit)?;
            writer.write_var_i32(packet.sequence)?;
        }
        PlayServerboundEntryPacket::UseItem(packet) => {
            writer.write_var_i32(packet.hand.index())?;
            writer.write_var_i32(packet.sequence)?;
            writer.write_f32(packet.yaw)?;
            writer.write_f32(packet.pitch)?;
        }
    }
    Ok(writer.into_inner())
}

#[must_use]
pub const fn packet_identity(packet: &PlayServerboundEntryPacket) -> &'static str {
    match packet {
        PlayServerboundEntryPacket::AcceptTeleportation(_) => ACCEPT_TELEPORTATION,
        PlayServerboundEntryPacket::Attack(_) => ATTACK,
        PlayServerboundEntryPacket::BundleItemSelected(_) => BUNDLE_ITEM_SELECTED,
        PlayServerboundEntryPacket::ChatAck(_) => CHAT_ACK,
        PlayServerboundEntryPacket::ChatCommand(_) => CHAT_COMMAND,
        PlayServerboundEntryPacket::ChatCommandSigned(_) => CHAT_COMMAND_SIGNED,
        PlayServerboundEntryPacket::ChatMessage(_) => CHAT,
        PlayServerboundEntryPacket::ChatSessionUpdate(_) => CHAT_SESSION_UPDATE,
        PlayServerboundEntryPacket::ChunkBatchReceived(_) => CHUNK_BATCH_RECEIVED,
        PlayServerboundEntryPacket::ClientTickEnd => CLIENT_TICK_END,
        PlayServerboundEntryPacket::ClientInformation(_) => CLIENT_INFORMATION,
        PlayServerboundEntryPacket::ClientCommand(_) => CLIENT_COMMAND,
        PlayServerboundEntryPacket::ContainerButtonClick(_) => CONTAINER_BUTTON_CLICK,
        PlayServerboundEntryPacket::ContainerClick(_) => CONTAINER_CLICK,
        PlayServerboundEntryPacket::ContainerClose(_) => CONTAINER_CLOSE,
        PlayServerboundEntryPacket::ContainerSlotStateChanged(_) => CONTAINER_SLOT_STATE_CHANGED,
        PlayServerboundEntryPacket::CommandSuggestion(_) => COMMAND_SUGGESTION,
        PlayServerboundEntryPacket::EditBook(_) => EDIT_BOOK,
        PlayServerboundEntryPacket::KeepAlive(_) => KEEP_ALIVE,
        PlayServerboundEntryPacket::Interact(_) => INTERACT,
        PlayServerboundEntryPacket::MovePlayerPosition(_) => MOVE_PLAYER_POS,
        PlayServerboundEntryPacket::MovePlayerPositionRotation(_) => MOVE_PLAYER_POS_ROT,
        PlayServerboundEntryPacket::MovePlayerRotation(_) => MOVE_PLAYER_ROT,
        PlayServerboundEntryPacket::MovePlayerStatusOnly(_) => MOVE_PLAYER_STATUS_ONLY,
        PlayServerboundEntryPacket::MoveVehicle(_) => MOVE_VEHICLE,
        PlayServerboundEntryPacket::PaddleBoat(_) => PADDLE_BOAT,
        PlayServerboundEntryPacket::PickItemFromBlock(_) => PICK_ITEM_FROM_BLOCK,
        PlayServerboundEntryPacket::PickItemFromEntity(_) => PICK_ITEM_FROM_ENTITY,
        PlayServerboundEntryPacket::PlayerAction(_) => PLAYER_ACTION,
        PlayServerboundEntryPacket::PlayerAbilities(_) => PLAYER_ABILITIES,
        PlayServerboundEntryPacket::PlayerCommand(_) => PLAYER_COMMAND,
        PlayServerboundEntryPacket::PlayerInput(_) => PLAYER_INPUT,
        PlayServerboundEntryPacket::PlayerLoaded => PLAYER_LOADED,
        PlayServerboundEntryPacket::Pong(_) => PONG,
        PlayServerboundEntryPacket::PlaceRecipe(_) => PLACE_RECIPE,
        PlayServerboundEntryPacket::RecipeBookChangeSettings(_) => RECIPE_BOOK_CHANGE_SETTINGS,
        PlayServerboundEntryPacket::RecipeBookSeenRecipe(_) => RECIPE_BOOK_SEEN_RECIPE,
        PlayServerboundEntryPacket::RenameItem(_) => RENAME_ITEM,
        PlayServerboundEntryPacket::SeenAdvancements(_) => SEEN_ADVANCEMENTS,
        PlayServerboundEntryPacket::SelectTrade(_) => SELECT_TRADE,
        PlayServerboundEntryPacket::SetCarriedItem(_) => SET_CARRIED_ITEM,
        PlayServerboundEntryPacket::SpectatorAction(_) => SPECTATOR_ACTION,
        PlayServerboundEntryPacket::SetBeacon(_) => SET_BEACON,
        PlayServerboundEntryPacket::Swing(_) => SWING,
        PlayServerboundEntryPacket::TeleportToEntity(_) => TELEPORT_TO_ENTITY,
        PlayServerboundEntryPacket::UseItemOn(_) => USE_ITEM_ON,
        PlayServerboundEntryPacket::UseItem(_) => USE_ITEM,
    }
}

fn read_position(
    reader: &mut WireReader<'_>,
) -> Result<PlayerPosition, PlayServerboundEntryCodecError> {
    Ok(PlayerPosition {
        x: reader.read_f64()?,
        y: reader.read_f64()?,
        z: reader.read_f64()?,
    })
}

fn write_position(
    writer: &mut WireWriter,
    position: PlayerPosition,
) -> Result<(), PlayServerboundEntryCodecError> {
    writer.write_f64(position.x)?;
    writer.write_f64(position.y)?;
    writer.write_f64(position.z)?;
    Ok(())
}

fn read_rotation(
    reader: &mut WireReader<'_>,
) -> Result<PlayerRotation, PlayServerboundEntryCodecError> {
    Ok(PlayerRotation {
        yaw: reader.read_f32()?,
        pitch: reader.read_f32()?,
    })
}

fn write_rotation(
    writer: &mut WireWriter,
    rotation: PlayerRotation,
) -> Result<(), PlayServerboundEntryCodecError> {
    writer.write_f32(rotation.yaw)?;
    writer.write_f32(rotation.pitch)?;
    Ok(())
}

fn read_action(
    reader: &mut WireReader<'_>,
) -> Result<PlayerActionKind, PlayServerboundEntryCodecError> {
    let value = reader.read_var_i32()?;
    PlayerActionKind::from_index(value).ok_or(PlayServerboundEntryCodecError::InvalidEnum {
        field: "player action",
        value,
    })
}

fn read_player_command(
    reader: &mut WireReader<'_>,
) -> Result<PlayerCommandKind, PlayServerboundEntryCodecError> {
    let value = reader.read_var_i32()?;
    PlayerCommandKind::from_index(value).ok_or(PlayServerboundEntryCodecError::InvalidEnum {
        field: "player command",
        value,
    })
}

fn read_hand(reader: &mut WireReader<'_>) -> Result<Hand, PlayServerboundEntryCodecError> {
    let value = reader.read_var_i32()?;
    Hand::from_index(value).ok_or(PlayServerboundEntryCodecError::InvalidEnum {
        field: "hand",
        value,
    })
}

fn read_block_hit(reader: &mut WireReader<'_>) -> Result<BlockHit, PlayServerboundEntryCodecError> {
    let position = unpack_block_position(reader.read_i64()?);
    let direction_value = reader.read_var_i32()?;
    let direction = direction_from_index(direction_value).map_err(|_| {
        PlayServerboundEntryCodecError::InvalidEnum {
            field: "block hit direction",
            value: direction_value,
        }
    })?;
    Ok(BlockHit {
        position,
        direction,
        offset_x: reader.read_f32()?,
        offset_y: reader.read_f32()?,
        offset_z: reader.read_f32()?,
        inside: reader.read_bool()?,
        world_border_hit: reader.read_bool()?,
    })
}

fn write_block_hit(
    writer: &mut WireWriter,
    hit: BlockHit,
) -> Result<(), PlayServerboundEntryCodecError> {
    writer.write_i64(pack_block_position(hit.position))?;
    writer.write_var_i32(direction_index(hit.direction))?;
    writer.write_f32(hit.offset_x)?;
    writer.write_f32(hit.offset_y)?;
    writer.write_f32(hit.offset_z)?;
    writer.write_bool(hit.inside)?;
    writer.write_bool(hit.world_border_hit)?;
    Ok(())
}
