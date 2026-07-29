use ferrite_protocol::java_26_2::configuration::serverbound::packet::ClientInformation;
use ferrite_protocol::java_26_2::play::serverbound::codec;
use ferrite_protocol::java_26_2::play::serverbound::movement::{
    InputDisposition, MovementControlProjection, PlayerCommandContext, PlayerCommandDisposition,
    VehicleMovementContext, VehicleMovementOutcome, VehicleMovementProjection,
};
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    ChunkBatchReceived, KeepAlive, MovePlayerPosition, MovePlayerPositionRotation,
    MovePlayerRotation, MovePlayerStatusOnly, MoveVehicle, MovementFlags, PaddleBoat,
    PlayServerboundEntryPacket, PlayerAbilities, PlayerCommand, PlayerCommandKind, PlayerInput,
    PlayerPosition, PlayerRotation, Pong,
};

fn flags(on_ground: bool, horizontal_collision: bool) -> MovementFlags {
    MovementFlags {
        on_ground,
        horizontal_collision,
    }
}

#[test]
fn all_fifteen_movement_family_identities_round_trip_locked_ids() {
    let packets = vec![
        PlayServerboundEntryPacket::ChunkBatchReceived(ChunkBatchReceived {
            desired_chunks_per_tick: 0.0,
        }),
        PlayServerboundEntryPacket::ClientTickEnd,
        PlayServerboundEntryPacket::ClientInformation(ClientInformation::default()),
        PlayServerboundEntryPacket::KeepAlive(KeepAlive { challenge: 0 }),
        PlayServerboundEntryPacket::MovePlayerPosition(MovePlayerPosition {
            position: position(),
            flags: flags(false, false),
        }),
        PlayServerboundEntryPacket::MovePlayerPositionRotation(MovePlayerPositionRotation {
            position: position(),
            rotation: rotation(),
            flags: flags(false, false),
        }),
        PlayServerboundEntryPacket::MovePlayerRotation(MovePlayerRotation {
            rotation: rotation(),
            flags: flags(false, false),
        }),
        PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
            flags: flags(false, false),
        }),
        PlayServerboundEntryPacket::MoveVehicle(MoveVehicle {
            position: position(),
            rotation: rotation(),
            on_ground: false,
        }),
        PlayServerboundEntryPacket::PaddleBoat(PaddleBoat {
            left: false,
            right: false,
        }),
        PlayServerboundEntryPacket::PlayerAbilities(PlayerAbilities { flying: false }),
        PlayServerboundEntryPacket::PlayerCommand(PlayerCommand {
            entity_id: 0,
            action: PlayerCommandKind::StopSleeping,
            data: 0,
        }),
        PlayServerboundEntryPacket::PlayerInput(PlayerInput::default()),
        PlayServerboundEntryPacket::PlayerLoaded,
        PlayServerboundEntryPacket::Pong(Pong { payload: 0 }),
    ];
    for (packet, expected_id) in packets
        .into_iter()
        .zip([11, 13, 14, 28, 30, 31, 32, 33, 34, 35, 40, 42, 43, 44, 45])
    {
        let encoded = codec::encode_packet(packet.clone()).unwrap();
        assert_eq!(encoded[0], expected_id);
        assert_eq!(codec::decode_packet(&encoded).unwrap(), packet);
    }
}

#[test]
fn ability_and_input_high_bits_are_ignored_and_reencode_canonically() {
    let mut abilities = codec::encode_packet(PlayServerboundEntryPacket::PlayerAbilities(
        PlayerAbilities { flying: false },
    ))
    .unwrap();
    abilities[1] = 0xff;
    let decoded = codec::decode_packet(&abilities).unwrap();
    assert_eq!(
        decoded,
        PlayServerboundEntryPacket::PlayerAbilities(PlayerAbilities { flying: true })
    );
    assert_eq!(codec::encode_packet(decoded).unwrap(), [40, 2]);

    let mut input = codec::encode_packet(PlayServerboundEntryPacket::PlayerInput(
        PlayerInput::default(),
    ))
    .unwrap();
    input[1] = 0xff;
    let decoded = codec::decode_packet(&input).unwrap();
    let PlayServerboundEntryPacket::PlayerInput(decoded_input) = decoded else {
        panic!("expected player input");
    };
    assert!(decoded_input.forward);
    assert!(decoded_input.backward);
    assert!(decoded_input.left);
    assert!(decoded_input.right);
    assert!(decoded_input.jump);
    assert!(decoded_input.shift);
    assert!(decoded_input.sprint);
    assert_eq!(
        codec::encode_packet(PlayServerboundEntryPacket::PlayerInput(decoded_input)).unwrap(),
        [43, 127]
    );
}

#[test]
fn all_player_command_actions_are_strict_and_preserve_ignored_entity_and_data() {
    let actions = [
        PlayerCommandKind::StopSleeping,
        PlayerCommandKind::StartSprinting,
        PlayerCommandKind::StopSprinting,
        PlayerCommandKind::StartRidingJump,
        PlayerCommandKind::StopRidingJump,
        PlayerCommandKind::OpenInventory,
        PlayerCommandKind::StartFallFlying,
    ];
    for (index, action) in actions.into_iter().enumerate() {
        let packet = PlayServerboundEntryPacket::PlayerCommand(PlayerCommand {
            entity_id: i32::MIN,
            action,
            data: i32::MAX,
        });
        assert_eq!(
            codec::decode_packet(&codec::encode_packet(packet.clone()).unwrap()).unwrap(),
            packet
        );
        let encoded = codec::encode_packet(packet).unwrap();
        assert!(encoded.windows(2).any(|bytes| bytes == [index as u8, 255]));
    }
    let mut invalid =
        codec::encode_packet(PlayServerboundEntryPacket::PlayerCommand(PlayerCommand {
            entity_id: 0,
            action: PlayerCommandKind::StopSleeping,
            data: 0,
        }))
        .unwrap();
    invalid[2] = 7;
    assert!(codec::decode_packet(&invalid).is_err());
}

#[test]
fn client_load_input_abilities_paddles_and_commands_follow_handler_gates() {
    let mut projection = MovementControlProjection::new(ClientInformation::default());
    let input = PlayerInput {
        forward: true,
        shift: true,
        ..PlayerInput::default()
    };
    assert_eq!(
        projection.update_input(input),
        InputDisposition::RetainedBeforeClientLoaded
    );
    assert_eq!(projection.input(), input);
    projection.update_abilities(PlayerAbilities { flying: true }, false);
    assert!(!projection.flying());
    projection.update_paddles(
        PaddleBoat {
            left: true,
            right: true,
        },
        false,
    );
    assert_eq!(
        projection.boat_paddles(),
        PaddleBoat {
            left: false,
            right: false
        }
    );
    assert_eq!(
        projection.apply_command(
            PlayerCommand {
                entity_id: i32::MAX,
                action: PlayerCommandKind::StartSprinting,
                data: i32::MIN,
            },
            PlayerCommandContext::default()
        ),
        PlayerCommandDisposition::IgnoredBeforeClientLoaded
    );

    projection.player_loaded();
    assert_eq!(
        projection.update_input(input),
        InputDisposition::ApplyLoadedState
    );
    projection.update_abilities(PlayerAbilities { flying: true }, true);
    assert!(projection.flying());
    projection.update_paddles(
        PaddleBoat {
            left: true,
            right: false,
        },
        true,
    );
    assert!(projection.boat_paddles().left);
    assert_eq!(
        projection.apply_command(
            PlayerCommand {
                entity_id: -1,
                action: PlayerCommandKind::StartRidingJump,
                data: 4,
            },
            PlayerCommandContext {
                controlled_vehicle_can_jump: true,
                ..PlayerCommandContext::default()
            }
        ),
        PlayerCommandDisposition::StartRidingJump(4)
    );
}

#[test]
fn client_information_hat_transition_and_chunk_feedback_boundaries_are_explicit() {
    let mut projection = MovementControlProjection::new(ClientInformation::default());
    let information = ClientInformation {
        model_customization: 0x40,
        ..ClientInformation::default()
    };
    assert!(projection.update_client_information(information.clone()));
    assert!(!projection.update_client_information(information));

    projection.acknowledge_chunk_batch(f32::NAN);
    assert_eq!(projection.chunk_flow().desired_chunks_per_tick(), 0.01);
    assert_eq!(projection.chunk_flow().unacknowledged_batches(), 0);
    assert_eq!(projection.chunk_flow().maximum_unacknowledged_batches(), 10);
    assert_eq!(projection.chunk_flow().batch_quota(), 1.0);
    projection.acknowledge_chunk_batch(f32::INFINITY);
    assert_eq!(projection.chunk_flow().desired_chunks_per_tick(), 64.0);
    projection.acknowledge_chunk_batch(f32::NEG_INFINITY);
    assert_eq!(projection.chunk_flow().desired_chunks_per_tick(), 0.01);
    assert_eq!(projection.chunk_flow().batch_quota(), 1.0);
}

#[test]
fn load_grace_expires_after_sixty_server_ticks_and_commands_use_context_not_entity_id() {
    let mut projection = MovementControlProjection::new(ClientInformation::default());
    for _ in 0..59 {
        projection.begin_server_tick();
        assert!(!projection.client_loaded());
    }
    projection.begin_server_tick();
    assert!(projection.client_loaded());

    assert_eq!(
        projection.apply_command(
            PlayerCommand {
                entity_id: i32::MIN,
                action: PlayerCommandKind::StartSprinting,
                data: i32::MIN,
            },
            PlayerCommandContext::default()
        ),
        PlayerCommandDisposition::Applied
    );
    assert!(projection.sprinting());
    assert_eq!(
        projection.apply_command(
            PlayerCommand {
                entity_id: i32::MAX,
                action: PlayerCommandKind::StartFallFlying,
                data: i32::MAX,
            },
            PlayerCommandContext::default()
        ),
        PlayerCommandDisposition::StopFallFlying
    );
    assert!(!projection.fall_flying());
    assert_eq!(
        projection.apply_command(
            PlayerCommand {
                entity_id: 0,
                action: PlayerCommandKind::OpenInventory,
                data: 0,
            },
            PlayerCommandContext {
                controlled_vehicle_has_inventory: true,
                ..PlayerCommandContext::default()
            }
        ),
        PlayerCommandDisposition::OpenVehicleInventory
    );
}

fn vehicle_context(position: PlayerPosition) -> VehicleMovementContext {
    VehicleMovementContext {
        controlled_tick_vehicle: true,
        singleplayer_owner: false,
        collision_result_position: position,
        old_box_collision_free: true,
        introduced_collision: false,
        supporting_collision_before: true,
        nearby_block_below: true,
        server_flight_allowed: false,
        vehicle_flying: false,
        vehicle_gravity_free: false,
    }
}

#[test]
fn vehicle_validation_orders_invalid_control_speed_collision_and_success() {
    let origin = PlayerPosition {
        x: 0.0,
        y: 64.0,
        z: 0.0,
    };
    let mut projection = VehicleMovementProjection::new(origin, rotation());
    let invalid = MoveVehicle {
        position: PlayerPosition {
            x: f64::NAN,
            ..origin
        },
        rotation: rotation(),
        on_ground: false,
    };
    assert_eq!(
        projection.apply(invalid, vehicle_context(origin)),
        VehicleMovementOutcome::DisconnectInvalidVehicleMovement
    );

    let target = PlayerPosition {
        x: 20.0,
        y: 64.0,
        z: 0.0,
    };
    let packet = MoveVehicle {
        position: target,
        rotation: PlayerRotation {
            yaw: 540.0,
            pitch: -540.0,
        },
        on_ground: true,
    };
    let mut uncontrolled = vehicle_context(target);
    uncontrolled.controlled_tick_vehicle = false;
    assert_eq!(
        projection.apply(packet, uncontrolled),
        VehicleMovementOutcome::Ignored
    );
    assert!(matches!(
        projection.apply(packet, vehicle_context(target)),
        VehicleMovementOutcome::Correct { .. }
    ));

    let mut singleplayer = vehicle_context(target);
    singleplayer.singleplayer_owner = true;
    assert_eq!(
        projection.apply(packet, singleplayer),
        VehicleMovementOutcome::Accepted {
            position: target,
            rotation: PlayerRotation {
                yaw: -180.0,
                pitch: -180.0
            },
            floating: false
        }
    );
}

fn position() -> PlayerPosition {
    PlayerPosition {
        x: 1.25,
        y: -2.5,
        z: 3.75,
    }
}

fn rotation() -> PlayerRotation {
    PlayerRotation {
        yaw: -180.0,
        pitch: 90.0,
    }
}

#[test]
fn four_movement_forms_preserve_omitted_fields_and_exact_wire_order() {
    let packets = [
        PlayServerboundEntryPacket::MovePlayerPosition(MovePlayerPosition {
            position: position(),
            flags: flags(true, false),
        }),
        PlayServerboundEntryPacket::MovePlayerPositionRotation(MovePlayerPositionRotation {
            position: position(),
            rotation: rotation(),
            flags: flags(false, true),
        }),
        PlayServerboundEntryPacket::MovePlayerRotation(MovePlayerRotation {
            rotation: rotation(),
            flags: flags(true, true),
        }),
        PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
            flags: flags(false, false),
        }),
    ];
    for packet in &packets {
        let body = codec::encode_packet(packet.clone()).unwrap();
        assert_eq!(codec::decode_packet(&body).unwrap(), packet.clone());
    }
    assert_eq!(codec::encode_packet(packets[0].clone()).unwrap()[0], 30);
    assert_eq!(codec::encode_packet(packets[1].clone()).unwrap()[0], 31);
    assert_eq!(codec::encode_packet(packets[2].clone()).unwrap()[0], 32);
    assert_eq!(codec::encode_packet(packets[3].clone()).unwrap()[0], 33);
}

#[test]
fn movement_high_flag_bits_decode_as_ignored_and_reencode_canonically() {
    let mut body = codec::encode_packet(PlayServerboundEntryPacket::MovePlayerStatusOnly(
        MovePlayerStatusOnly {
            flags: flags(true, true),
        },
    ))
    .unwrap();
    body[1] = 0xff;
    let decoded = codec::decode_packet(&body).unwrap();
    assert_eq!(
        decoded,
        PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
            flags: flags(true, true),
        })
    );
    assert_eq!(codec::encode_packet(decoded).unwrap(), [33, 3]);
    body.push(0);
    assert!(codec::decode_packet(&body).is_err());
}

#[test]
fn load_feedback_tick_end_and_keep_alive_have_locked_ids_and_bodies() {
    let packets = [
        PlayServerboundEntryPacket::ChunkBatchReceived(ChunkBatchReceived {
            desired_chunks_per_tick: f32::NAN,
        }),
        PlayServerboundEntryPacket::ClientTickEnd,
        PlayServerboundEntryPacket::KeepAlive(KeepAlive {
            challenge: i64::MIN,
        }),
        PlayServerboundEntryPacket::PlayerLoaded,
    ];
    let expected_ids = [11, 13, 28, 44];
    for (packet, expected_id) in packets.into_iter().zip(expected_ids) {
        let body = codec::encode_packet(packet.clone()).unwrap();
        assert_eq!(body[0], expected_id);
        let decoded = codec::decode_packet(&body).unwrap();
        if matches!(packet, PlayServerboundEntryPacket::ChunkBatchReceived(_)) {
            let PlayServerboundEntryPacket::ChunkBatchReceived(decoded) = decoded else {
                panic!("feedback identity changed");
            };
            assert!(decoded.desired_chunks_per_tick.is_nan());
        } else {
            assert_eq!(decoded, packet);
        }
    }
}
