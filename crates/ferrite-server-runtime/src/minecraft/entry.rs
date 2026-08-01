use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::DimensionId;
use ferrite_gameplay::player::state::PlayerPose;
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
use ferrite_protocol::java_26_2::play::clientbound::session::Respawn;
use ferrite_protocol::java_26_2::play::clientbound::world_effect::packet::LevelEvent;
use ferrite_protocol::java_26_2::value::identifier::{Identifier, IdentifierError};
use ferrite_protocol::semantic::PlayAdmission;
use ferrite_world::generation::border::state::WorldBorder;

use crate::world_service::dimension::FormalDimensionKind;
use crate::world_service::environment::LevelEnvironment;

pub(super) fn before_position(
    profile: &GameProfile,
    admission: &PlayAdmission,
    max_players: usize,
    simulation_distance: u16,
    enabled_dimensions: &[DimensionId],
) -> Result<Vec<PlayClientboundPacket>, EntryError> {
    let current_dimension = admission.region.dimension();
    let dimension = identifier(&current_dimension.to_string())?;
    let levels = enabled_dimensions
        .iter()
        .map(|dimension| identifier(&dimension.to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let max_players = i32::try_from(max_players).unwrap_or(i32::MAX);
    let radius = i32::from(admission.requested_view_distance.clamp(2, 32));
    Ok(vec![
        PlayClientboundPacket::Login(PlayLogin {
            player_entity_id: 1,
            hardcore: false,
            levels,
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
                sea_level: FormalDimensionKind::from_dimension(current_dimension)
                    .map_err(|_| EntryError::UnsupportedDimension(current_dimension.clone()))?
                    .sea_level(),
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
                dimension: identifier(&admission.region.dimension().to_string())?,
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
    packets.extend(crate::minecraft::environment::join_packets(
        admission.region.dimension(),
        environment,
    )?);
    Ok(packets)
}

pub(super) fn dimension_transition(
    dimension: &DimensionId,
    border: &WorldBorder,
    world_spawn: BlockPos,
    portal_cooldown: i32,
    player_level_event: Option<u16>,
    pose: PlayerPose,
) -> Result<Vec<PlayClientboundPacket>, EntryError> {
    let identity = identifier(&dimension.to_string())?;
    let mut packets = vec![
        PlayClientboundPacket::Respawn(Respawn {
            spawn: CommonSpawnInfo {
                dimension_type: identity.clone(),
                dimension: identity,
                obfuscated_seed: 0,
                game_mode: GameMode::Survival,
                previous_game_mode: Some(GameMode::Survival),
                is_debug: false,
                is_flat: false,
                last_death: None,
                portal_cooldown,
                sea_level: FormalDimensionKind::from_dimension(dimension)
                    .map_err(|_| EntryError::UnsupportedDimension(dimension.clone()))?
                    .sea_level(),
            },
            data_to_keep: 3,
        }),
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
            yaw: 0.0,
            pitch: 0.0,
        }),
    ];
    if let Some(event_type) = player_level_event {
        packets.push(PlayClientboundPacket::LevelEvent(LevelEvent {
            event_type: i32::from(event_type),
            position: BlockPos::new(
                pose.position.x.floor() as i32,
                pose.position.y.floor() as i32,
                pose.position.z.floor() as i32,
            ),
            data: 0,
            global: false,
        }));
    }
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
    #[error("play entry does not support dimension {0}")]
    UnsupportedDimension(DimensionId),
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::coordinate::ChunkPos;
    use ferrite_foundation::identity::{StableEntityId, WorldId};
    use ferrite_foundation::region::{
        RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
    };
    use ferrite_foundation::resource::ResourceId;
    use ferrite_protocol::semantic::{PlayerSpawn, SessionId, SessionIdentity};

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

    #[test]
    fn login_advertises_every_enabled_level_and_current_dimension_semantics() {
        let overworld = DimensionId::new(ResourceId::minecraft("overworld").unwrap());
        let nether = DimensionId::new(ResourceId::minecraft("the_nether").unwrap());
        let end = DimensionId::new(ResourceId::minecraft("the_end").unwrap());
        let admission = PlayAdmission {
            session: SessionId::new(1).unwrap(),
            identity: SessionIdentity {
                profile_id: 1,
                name: "FerriteUser".to_owned(),
            },
            player: StableEntityId::new(1).unwrap(),
            region: SimulationRegionKey::new(
                WorldId::new(1).unwrap(),
                nether.clone(),
                RegionCoord::new(0, 0),
                RegionMappingVersion::V1,
            ),
            region_mapping: RegionMapping::V1,
            spawn_chunk: ChunkPos::new(0, 0),
            spawn: PlayerSpawn {
                x: 0.5,
                y: 64.0,
                z: 0.5,
                yaw: 0.0,
                pitch: 0.0,
            },
            requested_view_distance: 10,
            transferred: false,
        };
        let profile = GameProfile {
            id: 1,
            name: "FerriteUser".to_owned(),
            properties: Vec::new(),
        };
        let packets =
            before_position(&profile, &admission, 8, 6, &[overworld, nether, end]).unwrap();
        let PlayClientboundPacket::Login(login) = &packets[0] else {
            panic!("first entry packet is not login")
        };
        assert_eq!(login.levels.len(), 3);
        assert_eq!(login.spawn.dimension.to_string(), "minecraft:the_nether");
        assert_eq!(
            login.spawn.dimension_type.to_string(),
            "minecraft:the_nether"
        );
        assert_eq!(login.spawn.sea_level, 32);
    }

    #[test]
    fn dimension_transition_orders_respawn_border_spawn_and_portal_event() {
        let nether = DimensionId::new(ResourceId::minecraft("the_nether").unwrap());
        let packets = dimension_transition(
            &nether,
            &WorldBorder::default(),
            BlockPos::new(8, 70, 8),
            10,
            Some(1032),
            ferrite_gameplay::player::state::PlayerPose::new(
                ferrite_gameplay::player::state::Vec3::new(1.5, 70.0, 2.5),
                ferrite_gameplay::player::state::Rotation {
                    yaw: 90.0,
                    pitch: 0.0,
                },
            ),
        )
        .unwrap();
        let PlayClientboundPacket::Respawn(respawn) = &packets[0] else {
            panic!("dimension transition must begin with Respawn");
        };
        assert_eq!(respawn.spawn.dimension.to_string(), "minecraft:the_nether");
        assert_eq!(respawn.spawn.portal_cooldown, 10);
        assert_eq!(respawn.spawn.sea_level, 32);
        assert_eq!(respawn.data_to_keep, 3);
        assert!(matches!(
            packets[1],
            PlayClientboundPacket::GameEvent(GameEvent {
                event: 13,
                parameter: 0.0
            })
        ));
        assert!(matches!(
            packets[2],
            PlayClientboundPacket::InitializeBorder(_)
        ));
        let PlayClientboundPacket::SetDefaultSpawnPosition(spawn) = &packets[3] else {
            panic!("dimension transition must reinstall the global spawn");
        };
        assert_eq!(spawn.position.dimension.to_string(), "minecraft:overworld");
        assert!(matches!(packets[4], PlayClientboundPacket::LevelEvent(_)));
    }
}
