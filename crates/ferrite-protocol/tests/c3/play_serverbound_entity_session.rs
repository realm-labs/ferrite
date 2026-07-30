use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::serverbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::serverbound::entity_session::low_precision;
use ferrite_protocol::java_26_2::play::serverbound::entity_session::model::{
    AttackRangeProjection, EntitySessionAction, EntitySessionDisposition, EntitySessionPlayer,
    InteractionResultProjection, ItemInteractionProjection, PlayerMode, SessionEntityKind,
    SessionEntityProjection, SessionItemStack, SessionLevelProjection, SessionPosition,
    SwingSource,
};
use ferrite_protocol::java_26_2::play::serverbound::entity_session::packet::{
    Attack, ClientCommand, ClientCommandKind, Interact, LowPrecisionVector, PickItemFromEntity,
    SpectatorAction, TeleportToEntity,
};
use ferrite_protocol::java_26_2::play::serverbound::entity_session::projection::EntitySessionProjection;
use ferrite_protocol::java_26_2::play::serverbound::packet::{Hand, PlayServerboundEntryPacket};
use ferrite_protocol::java_26_2::wire::primitive::{WireReader, WireWriter};
use ferrite_protocol::java_26_2::wire::varint::encode_i32;

fn position(x: f64) -> SessionPosition {
    SessionPosition {
        x,
        y: 64.0,
        z: 0.0,
        yaw: 30.0,
        pitch: 10.0,
    }
}

fn entity(id: i32, uuid: u128, kind: SessionEntityKind) -> SessionEntityProjection {
    let mut entity = SessionEntityProjection::new(id, uuid, kind);
    entity.position = position(f64::from(id));
    entity
}

fn projection(
    player: EntitySessionPlayer,
    entities: Vec<SessionEntityProjection>,
) -> EntitySessionProjection {
    let mut level = SessionLevelProjection::new("minecraft:overworld");
    level.entities = entities;
    EntitySessionProjection::new(player, vec![level]).unwrap()
}

fn interact(target: i32) -> Interact {
    Interact {
        target_entity_id: target,
        hand: Hand::Main,
        location: LowPrecisionVector::ZERO,
        secondary_action: false,
    }
}

#[test]
fn c3_gold_serverbound_entity_session_locks_all_six_packets() {
    let packets = [
        (
            PlayServerboundEntryPacket::Attack(Attack {
                target_entity_id: 1,
            }),
            vec![0x01, 0x01],
        ),
        (
            PlayServerboundEntryPacket::ClientCommand(ClientCommand {
                action: ClientCommandKind::PerformRespawn,
            }),
            vec![0x0c, 0x00],
        ),
        (
            PlayServerboundEntryPacket::Interact(interact(1)),
            vec![0x1a, 0x01, 0x00, 0x00, 0x00],
        ),
        (
            PlayServerboundEntryPacket::PickItemFromEntity(PickItemFromEntity {
                target_entity_id: 1,
                include_data: false,
            }),
            vec![0x25, 0x01, 0x00],
        ),
        (
            PlayServerboundEntryPacket::SpectatorAction(SpectatorAction {
                target_entity_id: None,
            }),
            vec![0x3e, 0x00],
        ),
        (
            PlayServerboundEntryPacket::TeleportToEntity(TeleportToEntity { target_uuid: 0 }),
            vec![0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
    ];
    for (packet, body) in packets {
        assert_eq!(encode_packet(packet.clone()).unwrap(), body);
        assert_eq!(decode_packet(&body).unwrap(), packet);
    }
}

#[test]
fn c3_entity_ingress_codecs_lock_fallback_bias_boolean_uuid_and_faults() {
    for target_entity_id in [i32::MIN, -1, 0, i32::MAX] {
        for packet in [
            PlayServerboundEntryPacket::Attack(Attack { target_entity_id }),
            PlayServerboundEntryPacket::PickItemFromEntity(PickItemFromEntity {
                target_entity_id,
                include_data: true,
            }),
        ] {
            assert_eq!(
                decode_packet(&encode_packet(packet.clone()).unwrap()).unwrap(),
                packet
            );
        }
    }
    let mut invalid_hand = vec![0x1a, 0x01];
    invalid_hand.extend(encode_i32(i32::MIN).as_slice());
    invalid_hand.extend([0, 0xff]);
    assert_eq!(
        decode_packet(&invalid_hand).unwrap(),
        PlayServerboundEntryPacket::Interact(Interact {
            target_entity_id: 1,
            hand: Hand::Main,
            location: LowPrecisionVector::ZERO,
            secondary_action: true,
        })
    );
    for (ordinal, expected) in [
        (i32::MIN, Hand::Main),
        (-1, Hand::Main),
        (0, Hand::Main),
        (1, Hand::Off),
        (2, Hand::Main),
        (i32::MAX, Hand::Main),
    ] {
        let mut body = vec![0x1a, 0x01];
        body.extend(encode_i32(ordinal).as_slice());
        body.extend([0, 0]);
        let PlayServerboundEntryPacket::Interact(packet) = decode_packet(&body).unwrap() else {
            panic!("interact identity must decode");
        };
        assert_eq!(packet.hand, expected);
    }
    for boolean in 0_u8..=u8::MAX {
        let PlayServerboundEntryPacket::PickItemFromEntity(packet) =
            decode_packet(&[0x25, 1, boolean]).unwrap()
        else {
            panic!("pick identity must decode");
        };
        assert_eq!(packet.include_data, boolean != 0);
    }

    let wrapped = PlayServerboundEntryPacket::SpectatorAction(SpectatorAction {
        target_entity_id: Some(i32::MAX),
    });
    assert_eq!(
        decode_packet(&encode_packet(wrapped.clone()).unwrap()).unwrap(),
        wrapped
    );
    let mut negative_optional = vec![0x3e];
    negative_optional.extend(encode_i32(i32::MIN).as_slice());
    assert_eq!(
        decode_packet(&negative_optional).unwrap(),
        PlayServerboundEntryPacket::SpectatorAction(SpectatorAction {
            target_entity_id: Some(i32::MAX),
        })
    );

    let uuid = u128::MAX - 7;
    let teleport =
        PlayServerboundEntryPacket::TeleportToEntity(TeleportToEntity { target_uuid: uuid });
    assert_eq!(
        decode_packet(&encode_packet(teleport.clone()).unwrap()).unwrap(),
        teleport
    );
    for ordinal in [-1, 3, i32::MAX] {
        let mut body = vec![0x0c];
        body.extend(encode_i32(ordinal).as_slice());
        assert!(decode_packet(&body).is_err());
    }
    assert!(decode_packet(&[0x01, 0x80]).is_err());
    assert!(decode_packet(&[0x40, 0]).is_err());
    assert!(decode_packet(&[0x01, 0x01, 0]).is_err());
}

#[test]
fn c3_low_precision_vector_locks_canonical_and_noncanonical_forms() {
    let mut writer = WireWriter::new(64);
    low_precision::write(
        &mut writer,
        LowPrecisionVector {
            x: f64::NAN,
            y: 0.0,
            z: -0.0,
        },
    )
    .unwrap();
    assert_eq!(writer.into_inner(), vec![0]);

    let mut endpoint = WireReader::new(&[0xf9, 0xff, 0, 0, 0, 3]);
    let endpoint = low_precision::read(&mut endpoint).unwrap();
    assert_eq!(endpoint.x, 1.0);
    assert_eq!((endpoint.y, endpoint.z), (-1.0, -1.0));

    let mut zero_scale = WireReader::new(&[8, 0, 0, 0, 0, 0]);
    let zero_scale = low_precision::read(&mut zero_scale).unwrap();
    assert!(zero_scale.x.is_finite());
    assert!(zero_scale.y.is_finite());
    assert!(zero_scale.z.is_finite());
    assert_eq!(zero_scale, LowPrecisionVector::ZERO);

    for vector in [
        LowPrecisionVector {
            x: 1.25,
            y: -2.5,
            z: 3.75,
        },
        LowPrecisionVector {
            x: f64::INFINITY,
            y: f64::NEG_INFINITY,
            z: 1.0,
        },
    ] {
        let mut writer = WireWriter::new(64);
        low_precision::write(&mut writer, vector).unwrap();
        let bytes = writer.into_inner();
        let mut reader = WireReader::new(&bytes);
        let decoded = low_precision::read(&mut reader).unwrap();
        reader.finish().unwrap();
        assert!(decoded.x.is_finite());
        assert!(decoded.y.is_finite());
        assert!(decoded.z.is_finite());
        assert!(decoded.x.abs() <= low_precision::MAX_COMPONENT);
        assert!(decoded.y.abs() <= low_precision::MAX_COMPONENT);
        assert!(decoded.z.abs() <= low_precision::MAX_COMPONENT);
    }

    let mut overlong = WireReader::new(&[5, 0, 0, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0]);
    assert!(low_precision::read(&mut overlong).is_err());
}

#[test]
fn c3_attack_admission_orders_load_idle_reach_piercing_and_disconnect() {
    let mut player = EntitySessionPlayer::new(99);
    let mut target = entity(1, 1, SessionEntityKind::Living);
    target.eye_to_aabb_distance_squared = 36.0;
    let mut state = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        state.handle_attack(Attack {
            target_entity_id: 1
        }),
        EntitySessionDisposition::Handled
    );
    assert_eq!(state.player().idle_resets, 1);
    assert_eq!(
        state.actions(),
        [EntitySessionAction::AttackExecuted {
            target_entity_id: 1
        }]
    );

    player.client_loaded = false;
    let mut unloaded = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        unloaded.handle_attack(Attack {
            target_entity_id: 1
        }),
        EntitySessionDisposition::Ignored
    );
    assert_eq!(unloaded.player().idle_resets, 0);

    player.client_loaded = true;
    target.eye_to_aabb_distance_squared = 36.000_001;
    let mut out_of_range = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        out_of_range.handle_attack(Attack {
            target_entity_id: 1
        }),
        EntitySessionDisposition::Ignored
    );
    assert_eq!(out_of_range.player().idle_resets, 1);

    target.kind = SessionEntityKind::Item;
    target.eye_to_aabb_distance_squared = 1.0;
    let mut invalid = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        invalid.handle_attack(Attack {
            target_entity_id: 1
        }),
        EntitySessionDisposition::DisconnectInvalidAttack
    );
    player.main_hand = SessionItemStack::item("minecraft:trident", 1);
    player.main_hand.piercing_weapon = true;
    let mut piercing = projection(player, vec![target]);
    assert_eq!(
        piercing.handle_attack(Attack {
            target_entity_id: 1
        }),
        EntitySessionDisposition::Ignored
    );
    assert!(piercing.actions().is_empty());
}

#[test]
fn c3_attack_custom_range_uses_closed_creative_and_mob_factored_endpoints() {
    let mut player = EntitySessionPlayer::new(99);
    player.main_hand = SessionItemStack::item("minecraft:stick", 1);
    player.main_hand.attack_range = Some(AttackRangeProjection {
        minimum: 2.0,
        maximum: 4.0,
        creative_minimum: 5.0,
        creative_maximum: 6.0,
        hitbox_margin: 1.0,
        mob_factor: 0.5,
    });
    let mut target = entity(1, 1, SessionEntityKind::Living);
    target.eye_to_aabb_distance_squared = 36.0;
    let mut survival = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        survival.handle_attack(Attack {
            target_entity_id: 1
        }),
        EntitySessionDisposition::Handled
    );

    player.mode = PlayerMode::Creative;
    target.eye_to_aabb_distance_squared = 100.0;
    let mut creative = projection(player, vec![target]);
    assert_eq!(
        creative.handle_attack(Attack {
            target_entity_id: 1
        }),
        EntitySessionDisposition::Handled
    );
}

#[test]
fn c3_interact_admission_retains_early_mutations_and_orders_item_success() {
    let mut player = EntitySessionPlayer::new(99);
    player.main_hand = SessionItemStack::item("minecraft:apple", 3);
    let mut missing = projection(player.clone(), Vec::new());
    let mut packet = interact(7);
    packet.secondary_action = true;
    assert_eq!(
        missing.handle_interact(packet),
        EntitySessionDisposition::Ignored
    );
    assert_eq!(
        (
            missing.player().idle_resets,
            missing.player().shift_key_down
        ),
        (1, true)
    );

    let mut target = entity(1, 1, SessionEntityKind::Living);
    target.eye_to_aabb_distance_squared = 36.0;
    let mut boundary = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        boundary.handle_interact(interact(1)),
        EntitySessionDisposition::Ignored
    );

    target.eye_to_aabb_distance_squared = 35.0;
    target.item_interaction = Some(ItemInteractionProjection {
        result: InteractionResultProjection::Success {
            swing: SwingSource::Server,
        },
        resulting_stack: SessionItemStack::default(),
    });
    let mut successful = projection(player, vec![target]);
    assert_eq!(
        successful.handle_interact(interact(1)),
        EntitySessionDisposition::Handled
    );
    assert!(successful.player().main_hand.is_empty());
    assert_eq!(
        successful.actions(),
        [
            EntitySessionAction::TargetInteraction {
                target_entity_id: 1,
                hand: Hand::Main,
                location: LowPrecisionVector::ZERO,
            },
            EntitySessionAction::ItemInteraction {
                target_entity_id: 1,
                hand: Hand::Main,
            },
            EntitySessionAction::EntityInteractGameEvent {
                target_entity_id: 1,
            },
            EntitySessionAction::InteractionCriterion {
                target_entity_id: 1,
                stack: SessionItemStack::item("minecraft:apple", 3),
            },
            EntitySessionAction::SwingPublished {
                hand: Hand::Main,
                include_self: true,
            },
        ]
    );
}

#[test]
fn c3_interact_spectator_menu_and_creative_item_restoration_are_distinct() {
    let mut target = entity(1, 1, SessionEntityKind::Living);
    target.menu_provider = true;
    let mut spectator_player = EntitySessionPlayer::new(99);
    spectator_player.mode = PlayerMode::Spectator;
    let mut spectator = projection(spectator_player, vec![target.clone()]);
    assert_eq!(
        spectator.handle_interact(interact(1)),
        EntitySessionDisposition::Handled
    );
    assert_eq!(
        spectator.actions(),
        [EntitySessionAction::SpectatorMenuOpened {
            target_entity_id: 1
        }]
    );

    let mut creative_player = EntitySessionPlayer::new(99);
    creative_player.infinite_materials = true;
    creative_player.main_hand = SessionItemStack::item("minecraft:bone_meal", 9);
    target.item_interaction = Some(ItemInteractionProjection {
        result: InteractionResultProjection::Consume,
        resulting_stack: SessionItemStack::item("minecraft:bone_meal", 2),
    });
    let mut creative = projection(creative_player, vec![target]);
    assert_eq!(
        creative.handle_interact(interact(1)),
        EntitySessionDisposition::Handled
    );
    assert_eq!(creative.player().main_hand.count, 9);
}

#[test]
fn c3_pick_entity_has_no_load_border_or_idle_gate_and_converges_inventory() {
    let mut player = EntitySessionPlayer::new(99);
    player.client_loaded = false;
    player.inventory = vec![SessionItemStack::item("minecraft:stone", 32)];
    let mut target = entity(1, 1, SessionEntityKind::Ordinary);
    target.inside_world_border = false;
    target.pick_result = Some(SessionItemStack::item("minecraft:stone", 1));
    let mut state = projection(player, vec![target]);
    assert_eq!(
        state.handle_pick(PickItemFromEntity {
            target_entity_id: 1,
            include_data: false,
        }),
        EntitySessionDisposition::Handled
    );
    assert_eq!(state.player().idle_resets, 0);
    assert_eq!(
        state.actions(),
        [
            EntitySessionAction::HeldSlotConvergence { slot: 0 },
            EntitySessionAction::InventoryMenuConvergence,
        ]
    );
    assert_eq!(
        state.player().hotbar[0].item.as_deref(),
        Some("minecraft:stone")
    );
}

#[test]
fn c3_pick_profile_data_is_independent_of_pick_result() {
    let mut player = EntitySessionPlayer::new(99);
    player.can_use_game_master_blocks = true;
    let target = entity(1, 1, SessionEntityKind::Avatar);
    let mut state = projection(player, vec![target]);
    assert_eq!(
        state.handle_pick(PickItemFromEntity {
            target_entity_id: 1,
            include_data: true,
        }),
        EntitySessionDisposition::Handled
    );
    assert_eq!(
        state.actions(),
        [EntitySessionAction::AvatarProfilePrinted {
            target_entity_id: 1
        }]
    );
}

#[test]
fn c3_pick_entity_rejects_removed_boundary_and_disabled_results() {
    let player = EntitySessionPlayer::new(99);
    let mut target = entity(1, 1, SessionEntityKind::Ordinary);
    target.removed = true;
    target.pick_result = Some(SessionItemStack::item("minecraft:stone", 1));
    let mut removed = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        removed.handle_pick(PickItemFromEntity {
            target_entity_id: 1,
            include_data: false,
        }),
        EntitySessionDisposition::Ignored
    );

    target.removed = false;
    target.eye_to_aabb_distance_squared = 36.0;
    let mut boundary = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        boundary.handle_pick(PickItemFromEntity {
            target_entity_id: 1,
            include_data: false,
        }),
        EntitySessionDisposition::Ignored
    );

    target.eye_to_aabb_distance_squared = 1.0;
    target.pick_result.as_mut().unwrap().feature_enabled = false;
    let mut disabled = projection(player, vec![target]);
    assert_eq!(
        disabled.handle_pick(PickItemFromEntity {
            target_entity_id: 1,
            include_data: false,
        }),
        EntitySessionDisposition::Ignored
    );
    assert!(disabled.actions().is_empty());
}

#[test]
fn c3_spectator_camera_applies_mode_load_range_and_publication_order() {
    let mut player = EntitySessionPlayer::new(99);
    player.mode = PlayerMode::Spectator;
    let mut target = entity(1, 1, SessionEntityKind::Living);
    target.position = position(40.0);
    let mut state = projection(player, vec![target]);
    assert_eq!(
        state.handle_spectator_action(SpectatorAction {
            target_entity_id: None
        }),
        EntitySessionDisposition::Ignored
    );
    assert_eq!(state.player().idle_resets, 1);
    assert_eq!(
        state.handle_spectator_action(SpectatorAction {
            target_entity_id: Some(1)
        }),
        EntitySessionDisposition::Handled
    );
    assert_eq!(state.player().position, position(40.0));
    assert_eq!(
        state.actions(),
        [
            EntitySessionAction::CameraTargetRelocated {
                target_entity_id: 1,
            },
            EntitySessionAction::CameraPublished {
                target_entity_id: 1,
            },
            EntitySessionAction::KnownPositionReset,
        ]
    );
}

#[test]
fn c3_spectator_camera_rejects_before_idle_or_at_strict_boundary() {
    let mut player = EntitySessionPlayer::new(99);
    player.mode = PlayerMode::Spectator;
    player.client_loaded = false;
    let mut target = entity(1, 1, SessionEntityKind::Living);
    let mut unloaded = projection(player.clone(), vec![target.clone()]);
    assert_eq!(
        unloaded.handle_spectator_action(SpectatorAction {
            target_entity_id: Some(1)
        }),
        EntitySessionDisposition::Ignored
    );
    assert_eq!(unloaded.player().idle_resets, 0);

    player.client_loaded = true;
    target.eye_to_aabb_distance_squared = 36.0;
    let mut boundary = projection(player, vec![target]);
    assert_eq!(
        boundary.handle_spectator_action(SpectatorAction {
            target_entity_id: Some(1)
        }),
        EntitySessionDisposition::Ignored
    );
    assert_eq!(boundary.player().idle_resets, 1);
}

#[test]
fn c3_uuid_teleport_requires_only_spectator_and_orders_cross_level_flow() {
    let mut player = EntitySessionPlayer::new(99);
    player.mode = PlayerMode::Spectator;
    player.client_loaded = false;
    player.camera_entity_id = 7;
    let overworld = SessionLevelProjection::new("minecraft:overworld");
    let mut nether = SessionLevelProjection::new("minecraft:the_nether");
    let mut target = entity(2, 44, SessionEntityKind::Living);
    target.position = position(120.0);
    nether.entities.push(target);
    let mut state = EntitySessionProjection::new(player, vec![overworld, nether]).unwrap();
    assert_eq!(
        state.handle_teleport_to_entity(TeleportToEntity { target_uuid: 44 }),
        EntitySessionDisposition::Handled
    );
    assert_eq!(state.player().current_level, 1);
    assert_eq!(state.player().idle_resets, 0);
    assert_eq!(
        state.actions(),
        [
            EntitySessionAction::CameraResetToSelf,
            EntitySessionAction::CameraPublished {
                target_entity_id: 99,
            },
            EntitySessionAction::CrossDimensionRespawn { keep_mask: 3 },
            EntitySessionAction::PositionChallenge,
            EntitySessionAction::LevelReprojection,
        ]
    );
}

#[test]
fn c3_client_command_respawn_stats_and_gamerules_follow_independent_rules() {
    let mut player = EntitySessionPlayer::new(99);
    player.stats_dirty.insert("minecraft:jump".into(), 3);
    player
        .gamerules
        .insert("minecraft:keep_inventory".into(), "false".into());
    let mut state = projection(player, Vec::new());
    assert_eq!(
        state.handle_client_command(ClientCommand {
            action: ClientCommandKind::PerformRespawn,
        }),
        EntitySessionDisposition::Ignored
    );
    assert_eq!(state.player().idle_resets, 1);

    state.player_mut().health = 0.0;
    state.player_mut().hardcore = true;
    assert_eq!(
        state.handle_client_command(ClientCommand {
            action: ClientCommandKind::PerformRespawn,
        }),
        EntitySessionDisposition::Handled
    );
    assert_eq!(state.player().mode, PlayerMode::Spectator);
    assert!(!state.player().client_loaded);
    assert_eq!(state.player().load_grace_ticks, 60);

    state.take_actions();
    assert_eq!(
        state.handle_client_command(ClientCommand {
            action: ClientCommandKind::RequestStats,
        }),
        EntitySessionDisposition::Handled
    );
    let mut stats = BTreeMap::new();
    stats.insert("minecraft:jump".into(), 3);
    assert_eq!(
        state.take_actions(),
        [EntitySessionAction::StatsPublished { values: stats }]
    );
    state.handle_client_command(ClientCommand {
        action: ClientCommandKind::RequestStats,
    });
    assert_eq!(
        state.take_actions(),
        [EntitySessionAction::StatsPublished {
            values: BTreeMap::new()
        }]
    );

    assert_eq!(
        state.handle_client_command(ClientCommand {
            action: ClientCommandKind::RequestGameruleValues,
        }),
        EntitySessionDisposition::Ignored
    );
    assert_eq!(
        state.take_actions(),
        [EntitySessionAction::GameruleRequestDenied]
    );
    state.player_mut().gamerule_permission = true;
    state.handle_client_command(ClientCommand {
        action: ClientCommandKind::RequestGameruleValues,
    });
    assert!(matches!(
        state.take_actions().as_slice(),
        [EntitySessionAction::GamerulesPublished { .. }]
    ));
}

#[test]
fn c3_won_game_respawn_retains_data_and_records_end_return_after_reprojection() {
    let mut player = EntitySessionPlayer::new(99);
    player.won_game = true;
    let mut state = projection(player, Vec::new());
    assert_eq!(
        state.handle_client_command(ClientCommand {
            action: ClientCommandKind::PerformRespawn,
        }),
        EntitySessionDisposition::Handled
    );
    assert!(!state.player().won_game);
    assert_eq!(
        state.actions(),
        [
            EntitySessionAction::PlayerRespawned {
                retain_player_data: true,
            },
            EntitySessionAction::KnownPositionReset,
            EntitySessionAction::ClientLoadGraceRestarted { ticks: 60 },
            EntitySessionAction::RespawnPublished,
            EntitySessionAction::PositionChallenge,
            EntitySessionAction::LevelReprojection,
            EntitySessionAction::EndToOverworldCriterion,
        ]
    );
}

#[test]
fn c3_entity_session_end_to_end_keeps_raw_lookups_at_adapter_boundary() {
    let mut player = EntitySessionPlayer::new(99);
    player.main_hand = SessionItemStack::item("minecraft:stick", 1);
    player.mode = PlayerMode::Spectator;
    let mut target = entity(1, 0xfeed, SessionEntityKind::Living);
    target.target_interaction = InteractionResultProjection::Success {
        swing: SwingSource::Client,
    };
    let mut state = projection(player, vec![target]);

    state.player_mut().mode = PlayerMode::Survival;
    assert_eq!(
        state
            .handle(PlayServerboundEntryPacket::Interact(interact(1)))
            .unwrap(),
        EntitySessionDisposition::Handled
    );
    assert_eq!(
        state
            .handle(PlayServerboundEntryPacket::Attack(Attack {
                target_entity_id: 1,
            }))
            .unwrap(),
        EntitySessionDisposition::Handled
    );
    state.player_mut().mode = PlayerMode::Spectator;
    assert_eq!(
        state
            .handle(PlayServerboundEntryPacket::SpectatorAction(
                SpectatorAction {
                    target_entity_id: Some(1),
                },
            ))
            .unwrap(),
        EntitySessionDisposition::Handled
    );
    assert_eq!(
        state
            .handle(PlayServerboundEntryPacket::TeleportToEntity(
                TeleportToEntity {
                    target_uuid: 0xfeed,
                },
            ))
            .unwrap(),
        EntitySessionDisposition::Handled
    );
    assert!(state.actions().iter().any(|action| matches!(
        action,
        EntitySessionAction::AttackExecuted {
            target_entity_id: 1
        }
    )));
    assert_eq!(state.player().camera_entity_id, state.player().entity_id);
}
