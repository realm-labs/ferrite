use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::packet::{
    BorderInitialization, ChangeDifficulty, DefaultSpawnPosition, EntityEvent, GameEvent, GameMode,
    GlobalBlockPosition, PlayClientboundPacket, PlayerAbilities, PlayerPosition, ServerData,
    SetTime, TickingState, Vector3,
};
use ferrite_protocol::java_26_2::play::clientbound::player_info::{
    AddedProfile, PlayerInfoActions, PlayerInfoEntry, PlayerInfoUpdate,
};
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    BorderSize, Difficulty, LocalPlayerState, PlayClientAction, PlayEntryProjection,
    PlayEntryStage, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::{RecipeBookAdd, RecipeBookSettings};
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;

use super::fixtures::{empty_commands, empty_recipes, id, login};

fn apply_core(projection: &mut PlayEntryProjection) {
    for packet in [
        login(),
        PlayClientboundPacket::ChangeDifficulty(ChangeDifficulty {
            raw_difficulty: -1,
            locked: true,
        }),
        PlayClientboundPacket::PlayerAbilities(PlayerAbilities {
            flags: u8::MAX,
            flying_speed: 0.05,
            walking_speed: 0.1,
        }),
        PlayClientboundPacket::SetHeldSlot(99),
        PlayClientboundPacket::UpdateRecipes(empty_recipes()),
        PlayClientboundPacket::EntityEvent(EntityEvent {
            entity_id: 1,
            event: 27,
        }),
        PlayClientboundPacket::Commands(empty_commands()),
        PlayClientboundPacket::RecipeBookSettings(RecipeBookSettings::default()),
        PlayClientboundPacket::RecipeBookAdd(RecipeBookAdd {
            entries: Vec::new(),
            replace: true,
        }),
    ] {
        assert_eq!(projection.apply(packet).unwrap(), PlayClientAction::None);
    }
}

fn initial_state() -> LocalPlayerState {
    LocalPlayerState {
        position: Vector3 {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        },
        motion: Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        yaw: 30.0,
        pitch: 80.0,
    }
}

fn position_packet() -> PlayClientboundPacket {
    PlayClientboundPacket::PlayerPosition(PlayerPosition {
        teleport_id: 7,
        position: Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        motion: Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        yaw: 10.0,
        pitch: 20.0,
        relative_flags: (1 << 0) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 20),
    })
}

#[test]
fn locked_entry_trace_reaches_terrain_only_after_tick_projection() {
    let mut projection = PlayEntryProjection::new(initial_state(), false, true);
    assert_eq!(
        projection.apply(PlayClientboundPacket::TickingStep(0)),
        Err(PlayProjectionError::LevelNotInstalled)
    );
    apply_core(&mut projection);
    assert_eq!(projection.difficulty(), Difficulty::Hard);
    assert!(projection.difficulty_locked());
    assert_eq!(projection.held_slot(), 0);
    assert_eq!(projection.permission_tier(), Some(3));
    assert!(projection.abilities().invulnerable);
    assert!(projection.abilities().flying);
    assert!(projection.abilities().can_fly);
    assert!(projection.abilities().instant_build);

    let action = projection.apply(position_packet()).unwrap();
    let PlayClientAction::AcknowledgeTeleportThenEchoMovement {
        teleport_id,
        state,
        reset_block_prediction,
    } = action
    else {
        panic!("expected ordered teleport response");
    };
    assert_eq!(teleport_id, 7);
    assert!(reset_block_prediction);
    assert_eq!(
        state,
        LocalPlayerState {
            position: Vector3 {
                x: 11.0,
                y: 2.0,
                z: 33.0,
            },
            motion: Vector3 {
                x: 5.0,
                y: 5.0,
                z: 6.0,
            },
            yaw: 40.0,
            pitch: 90.0,
        }
    );

    projection
        .apply(PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
            actions: PlayerInfoActions::all(),
            entries: Vec::new(),
        }))
        .unwrap();
    projection
        .apply(PlayClientboundPacket::InitializeBorder(
            BorderInitialization {
                center_x: 1.0,
                center_z: 2.0,
                old_size: 3.0,
                new_size: 4.0,
                lerp_millis: -1,
                absolute_maximum: 5,
                warning_blocks: 6,
                warning_time: 7,
            },
        ))
        .unwrap();
    assert_eq!(
        projection.border().unwrap().size,
        BorderSize::Immediate(4.0)
    );
    projection
        .apply(PlayClientboundPacket::SetTime(SetTime {
            game_time: -9,
            clocks: BTreeMap::new(),
        }))
        .unwrap();
    projection
        .apply(PlayClientboundPacket::SetDefaultSpawnPosition(
            DefaultSpawnPosition {
                position: GlobalBlockPosition {
                    dimension: id("minecraft:overworld"),
                    packed_position: 64,
                },
                yaw: 0.0,
                pitch: 0.0,
            },
        ))
        .unwrap();
    projection
        .apply(PlayClientboundPacket::GameEvent(GameEvent {
            event: 7,
            parameter: 0.5,
        }))
        .unwrap();
    projection
        .apply(PlayClientboundPacket::GameEvent(GameEvent {
            event: 13,
            parameter: 0.0,
        }))
        .unwrap();
    assert!(projection.level().unwrap().terrain_load_started);
    projection
        .apply(PlayClientboundPacket::TickingState(TickingState {
            tick_rate: f32::INFINITY,
            frozen: true,
        }))
        .unwrap();
    projection
        .apply(PlayClientboundPacket::TickingStep(-1))
        .unwrap();
    assert_eq!(projection.stage(), PlayEntryStage::ReadyForTerrain);
    assert_eq!(projection.game_time(), -9);
    assert_eq!(projection.ticking_steps(), -1);
}

#[test]
fn riding_keeps_local_state_but_still_emits_both_teleport_responses() {
    let mut projection = PlayEntryProjection::new(initial_state(), true, false);
    apply_core(&mut projection);
    assert_eq!(
        projection.apply(position_packet()).unwrap(),
        PlayClientAction::AcknowledgeTeleportThenEchoMovement {
            teleport_id: 7,
            state: initial_state(),
            reset_block_prediction: true,
        }
    );
    assert_eq!(projection.local_player(), initial_state());
}

#[test]
fn player_info_adds_before_updates_and_ignores_unknown_update_only_entries() {
    let mut projection = PlayEntryProjection::default();
    apply_core(&mut projection);
    projection.apply(position_packet()).unwrap();

    projection
        .apply(PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
            actions: PlayerInfoActions::from_bits(PlayerInfoActions::UPDATE_GAME_MODE),
            entries: vec![PlayerInfoEntry {
                profile_id: 9,
                added_profile: None,
                chat_session: None,
                game_mode: Some(GameMode::Spectator),
                listed: None,
                latency_millis: None,
                display_name: None,
                list_order: None,
                show_hat: None,
            }],
        }))
        .unwrap();
    assert!(projection.players().is_empty());

    projection
        .apply(PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
            actions: PlayerInfoActions::from_bits(
                PlayerInfoActions::ADD_PLAYER
                    | PlayerInfoActions::UPDATE_GAME_MODE
                    | PlayerInfoActions::UPDATE_LISTED,
            ),
            entries: vec![PlayerInfoEntry {
                profile_id: 9,
                added_profile: Some(AddedProfile {
                    name: "Other".to_owned(),
                    properties: Vec::new(),
                }),
                chat_session: None,
                game_mode: Some(GameMode::Creative),
                listed: Some(true),
                latency_millis: None,
                display_name: None,
                list_order: None,
                show_hat: None,
            }],
        }))
        .unwrap();
    let player = projection.players().get(&9).unwrap();
    assert_eq!(player.game_mode, GameMode::Creative);
    assert!(player.listed);
}

#[test]
fn server_data_requires_a_list_record_and_drops_an_obviously_invalid_icon() {
    let data = ServerData {
        motd: TextComponentNbt::literal("Ferrite").unwrap(),
        icon: Some(vec![1, 2, 3]),
    };
    let mut projection = PlayEntryProjection::new(initial_state(), false, true);
    apply_core(&mut projection);
    projection.apply(position_packet()).unwrap();
    projection
        .apply(PlayClientboundPacket::ServerData(data.clone()))
        .unwrap();
    assert!(projection.server_data().unwrap().icon.is_none());

    let mut transferred = PlayEntryProjection::new(initial_state(), false, false);
    apply_core(&mut transferred);
    transferred.apply(position_packet()).unwrap();
    transferred
        .apply(PlayClientboundPacket::ServerData(data))
        .unwrap();
    assert!(transferred.server_data().is_none());
}

#[test]
fn duplicate_login_and_out_of_order_core_packets_fail_without_partial_advance() {
    let mut projection = PlayEntryProjection::default();
    projection.apply(login()).unwrap();
    assert_eq!(
        projection.apply(login()),
        Err(PlayProjectionError::DuplicateLogin)
    );

    let mut projection = PlayEntryProjection::default();
    projection.apply(login()).unwrap();
    assert!(matches!(
        projection.apply(PlayClientboundPacket::SetHeldSlot(0)),
        Err(PlayProjectionError::UnexpectedOrder { .. })
    ));
    assert_eq!(projection.stage(), PlayEntryStage::AwaitingDifficulty);
}
