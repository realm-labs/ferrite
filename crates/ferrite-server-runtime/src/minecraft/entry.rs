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

use crate::world_service::environment::LevelEnvironment;

pub(super) fn before_position(
    profile: &GameProfile,
    admission: &PlayAdmission,
    max_players: usize,
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
            simulation_distance: 10,
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
                is_flat: true,
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
) -> Result<Vec<PlayClientboundPacket>, EntryError> {
    let spawn = BlockPos::new(
        admission.spawn.x.floor() as i32,
        admission.spawn.y.floor() as i32,
        admission.spawn.z.floor() as i32,
    );
    let mut packets = vec![
        PlayClientboundPacket::GameEvent(GameEvent {
            event: 13,
            parameter: 0.0,
        }),
        PlayClientboundPacket::InitializeBorder(BorderInitialization {
            center_x: 0.0,
            center_z: 0.0,
            old_size: 59_999_968.0,
            new_size: 59_999_968.0,
            lerp_millis: 0,
            absolute_maximum: 29_999_984,
            warning_blocks: 5,
            warning_time: 15,
        }),
        PlayClientboundPacket::SetDefaultSpawnPosition(DefaultSpawnPosition {
            position: GlobalBlockPosition {
                dimension: identifier("minecraft:overworld")?,
                packed_position: pack_block_position(spawn),
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

fn identifier(value: &str) -> Result<Identifier, EntryError> {
    Ok(Identifier::parse(value)?)
}

#[derive(Debug, thiserror::Error)]
pub(super) enum EntryError {
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
}
