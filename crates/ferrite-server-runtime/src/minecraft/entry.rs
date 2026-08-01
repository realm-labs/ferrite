use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::login::profile::GameProfile;
use ferrite_protocol::java_26_2::play::block::pack_block_position;
use ferrite_protocol::java_26_2::play::clientbound::command::{
    CommandNode, CommandNodeKind, CommandTree,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    BorderInitialization, ChangeDifficulty, CommonSpawnInfo, DefaultSpawnPosition, EntityEvent,
    GameEvent, GameMode, GlobalBlockPosition, PlayClientboundPacket, PlayLogin, PlayerAbilities,
    TickingState,
};
use ferrite_protocol::java_26_2::play::clientbound::player_info::{
    AddedProfile, PlayerInfoActions, PlayerInfoEntry, PlayerInfoUpdate,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::{
    RecipeBookAdd, RecipeBookSettings, RecipeProjection,
};
use ferrite_protocol::java_26_2::value::identifier::{Identifier, IdentifierError};
use ferrite_protocol::semantic::PlayAdmission;
use ferrite_world::generation::border::state::WorldBorder;

use crate::world_service::environment::LevelEnvironment;

pub(super) fn before_position(
    profile: &GameProfile,
    admission: &PlayAdmission,
    max_players: usize,
    simulation_distance: u16,
) -> Result<Vec<PlayClientboundPacket>, EntryError> {
    let dimension = identifier("minecraft:overworld")?;
    let max_players = i32::try_from(max_players).unwrap_or(i32::MAX);
    let radius = i32::from(admission.requested_view_distance.clamp(2, 32));
    Ok(vec![
        PlayClientboundPacket::Login(PlayLogin {
            player_entity_id: 1,
            hardcore: false,
            levels: BTreeSet::from([dimension.clone()]),
            max_players,
            chunk_radius: radius,
            simulation_distance: i32::from(simulation_distance),
            reduced_debug_info: false,
            show_death_screen: true,
            limited_crafting: false,
            spawn: CommonSpawnInfo {
                dimension_type: dimension.clone(),
                dimension: dimension.clone(),
                obfuscated_seed: 0,
                game_mode: GameMode::Survival,
                previous_game_mode: None,
                is_debug: false,
                is_flat: false,
                last_death: None,
                portal_cooldown: 0,
                sea_level: 63,
            },
            online_mode: false,
            enforces_secure_chat: false,
        }),
        PlayClientboundPacket::ChangeDifficulty(ChangeDifficulty {
            raw_difficulty: 2,
            locked: false,
        }),
        PlayClientboundPacket::PlayerAbilities(PlayerAbilities {
            flags: 0,
            flying_speed: 0.05,
            walking_speed: 0.1,
        }),
        PlayClientboundPacket::SetHeldSlot(0),
        PlayClientboundPacket::UpdateRecipes(RecipeProjection {
            properties: BTreeMap::new(),
            stonecutter: Vec::new(),
        }),
        PlayClientboundPacket::EntityEvent(EntityEvent {
            entity_id: 1,
            event: 24,
        }),
        PlayClientboundPacket::Commands(CommandTree {
            nodes: vec![CommandNode {
                executable: false,
                restricted: false,
                children: Vec::new(),
                redirect: None,
                kind: CommandNodeKind::Root,
            }],
            root_index: 0,
        }),
        PlayClientboundPacket::RecipeBookSettings(RecipeBookSettings::default()),
        PlayClientboundPacket::RecipeBookAdd(RecipeBookAdd {
            entries: Vec::new(),
            replace: true,
        }),
        PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
            actions: PlayerInfoActions::all(),
            entries: vec![PlayerInfoEntry {
                profile_id: profile.id,
                added_profile: Some(AddedProfile {
                    name: profile.name.clone(),
                    properties: profile.properties.clone(),
                }),
                chat_session: Some(None),
                game_mode: Some(GameMode::Survival),
                listed: Some(true),
                latency_millis: Some(0),
                display_name: Some(None),
                list_order: Some(0),
                show_hat: Some(true),
            }],
        }),
    ])
}

pub(super) fn after_position(
    admission: &PlayAdmission,
    environment: LevelEnvironment,
    border: &WorldBorder,
    world_spawn: BlockPos,
) -> Result<Vec<PlayClientboundPacket>, EntryError> {
    let mut packets = vec![
        PlayClientboundPacket::GameEvent(GameEvent {
            event: 13,
            parameter: 0.0,
        }),
        PlayClientboundPacket::InitializeBorder(border_initialization(border)),
        PlayClientboundPacket::SetDefaultSpawnPosition(DefaultSpawnPosition {
            position: GlobalBlockPosition {
                dimension: identifier("minecraft:overworld")?,
                packed_position: pack_block_position(world_spawn),
            },
            yaw: admission.spawn.yaw,
            pitch: admission.spawn.pitch,
        }),
        PlayClientboundPacket::TickingState(TickingState {
            tick_rate: 20.0,
            frozen: false,
        }),
        PlayClientboundPacket::TickingStep(0),
    ];
    packets.extend(crate::minecraft::environment::join_packets(environment)?);
    Ok(packets)
}

fn border_initialization(border: &WorldBorder) -> BorderInitialization {
    let snapshot = border.snapshot();
    BorderInitialization {
        center_x: snapshot.center_x,
        center_z: snapshot.center_z,
        old_size: snapshot.old_size,
        new_size: snapshot.new_size,
        lerp_millis: snapshot.remaining_ticks.saturating_mul(50),
        absolute_maximum: snapshot.absolute_max,
        warning_blocks: snapshot.warning_blocks,
        warning_time: snapshot.warning_time,
    }
}

fn identifier(value: &str) -> Result<Identifier, EntryError> {
    Ok(Identifier::parse(value)?)
}

#[derive(Debug, thiserror::Error)]
pub(super) enum EntryError {
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_border_projection_uses_the_authoritative_snapshot() {
        let mut border = WorldBorder::default();
        border.set_center(12.5, -7.25);
        border.set_warning_blocks(9);
        border.set_warning_time(40);
        border.lerp_size_between(128.0, 64.0, 20, 5);
        let packet = border_initialization(&border);
        assert_eq!((packet.center_x, packet.center_z), (12.5, -7.25));
        assert_eq!((packet.old_size, packet.new_size), (128.0, 64.0));
        assert_eq!(packet.lerp_millis, 1_000);
        assert_eq!((packet.warning_blocks, packet.warning_time), (9, 40));
    }
}
