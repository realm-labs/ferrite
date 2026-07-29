use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::liveness::{
    DeferredKeepAliveEchoes, DeferredKeepAliveError,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, GameMode, KeepAlive, Ping, PlayClientboundPacket, PlayLogin, PlayerRotation,
    Vector3, VehiclePosition,
};
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    LocalPlayerState, PlayClientAction, PlayEntryProjection, PlayProjectionError,
    RootVehicleProjection, VehicleMovementState,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::play::serverbound::codec as serverbound;
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    MovePlayerRotation, MoveVehicle, PlayServerboundEntryPacket, Pong,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::{NbtQuota, NetworkNbt, TextComponentNbt};

static REJECT_COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &REJECT_COMPONENTS,
        dimension_section_count: 24,
    }
}

fn login() -> PlayClientboundPacket {
    PlayClientboundPacket::Login(PlayLogin {
        player_entity_id: 1,
        hardcore: false,
        levels: BTreeSet::from([id("minecraft:overworld")]),
        max_players: 20,
        chunk_radius: 2,
        simulation_distance: 2,
        reduced_debug_info: false,
        show_death_screen: true,
        limited_crafting: false,
        spawn: CommonSpawnInfo {
            dimension_type: id("minecraft:overworld"),
            dimension: id("minecraft:overworld"),
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
    })
}

fn initial_player() -> LocalPlayerState {
    LocalPlayerState {
        position: Vector3 {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        },
        motion: Vector3::default(),
        yaw: 30.0,
        pitch: 40.0,
    }
}

fn projection(riding: bool) -> PlayEntryProjection {
    let mut projection = PlayEntryProjection::new(initial_player(), riding, false);
    projection.apply(login()).unwrap();
    projection
}

fn empty_reason() -> TextComponentNbt {
    TextComponentNbt::literal("").unwrap()
}

#[test]
fn five_packet_goldens_have_locked_ids_and_zero_bodies() {
    let registries = PlayRegistries::default();
    let mut move_vehicle = vec![57];
    move_vehicle.extend_from_slice(&[0; 32]);
    let packets = [
        (
            PlayClientboundPacket::Disconnect(empty_reason()),
            vec![32, 8, 0, 0],
        ),
        (
            PlayClientboundPacket::KeepAlive(KeepAlive { challenge: 0 }),
            vec![44, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::MoveVehicle(VehiclePosition {
                position: Vector3::default(),
                yaw: 0.0,
                pitch: 0.0,
            }),
            move_vehicle,
        ),
        (
            PlayClientboundPacket::Ping(Ping { payload: 0 }),
            vec![61, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::PlayerRotation(PlayerRotation {
                yaw: 0.0,
                relative_yaw: false,
                pitch: 0.0,
                relative_pitch: false,
            }),
            vec![73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
    ];
    for (packet, expected) in packets {
        let body = encode_packet(&packet, &registries).unwrap();
        assert_eq!(body, expected);
        assert_eq!(decode_packet(&body, context(&registries)).unwrap(), packet);
    }
}

#[test]
fn exceptional_float_bits_and_signed_payload_extrema_round_trip() {
    let registries = PlayRegistries::default();
    let packets = [
        PlayClientboundPacket::KeepAlive(KeepAlive {
            challenge: i64::MIN,
        }),
        PlayClientboundPacket::Ping(Ping { payload: i32::MIN }),
        PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: Vector3 {
                x: f64::NAN,
                y: f64::INFINITY,
                z: f64::NEG_INFINITY,
            },
            yaw: f32::NAN,
            pitch: f32::INFINITY,
        }),
        PlayClientboundPacket::PlayerRotation(PlayerRotation {
            yaw: f32::NEG_INFINITY,
            relative_yaw: true,
            pitch: f32::NAN,
            relative_pitch: false,
        }),
    ];
    for packet in packets {
        let body = encode_packet(&packet, &registries).unwrap();
        let decoded = decode_packet(&body, context(&registries)).unwrap();
        assert_eq!(encode_packet(&decoded, &registries).unwrap(), body);
    }
}

#[test]
fn malformed_reason_primitives_and_trailing_bytes_fail_closed() {
    let registries = PlayRegistries::default();
    let invalid_component = NetworkNbt::from_bytes(vec![1, 0], NbtQuota::Trusted)
        .unwrap()
        .as_bytes()
        .to_vec();
    let mut invalid_reason = vec![32];
    invalid_reason.extend_from_slice(&invalid_component);
    assert!(decode_packet(&invalid_reason, context(&registries)).is_err());

    for body in [vec![44], vec![57], vec![61, 0, 0, 0], vec![73, 0]] {
        assert!(decode_packet(&body, context(&registries)).is_err());
    }
    let mut trailing = encode_packet(
        &PlayClientboundPacket::Ping(Ping { payload: 1 }),
        &registries,
    )
    .unwrap();
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn disconnect_reason_uses_trusted_not_default_nbt_quota() {
    let registries = PlayRegistries::default();
    let payload_len = 2_097_153_i32;
    let mut bytes = vec![10, 7, 0, 0];
    bytes.extend_from_slice(&payload_len.to_be_bytes());
    bytes.resize(bytes.len() + payload_len as usize, 0x4a);
    bytes.push(0);
    let reason = TextComponentNbt::from_network_nbt(
        NetworkNbt::from_bytes(bytes, NbtQuota::Trusted).unwrap(),
    )
    .unwrap();
    let packet = PlayClientboundPacket::Disconnect(reason);
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );
}

#[test]
fn disconnect_requires_an_installed_level_and_has_no_response() {
    let reason = TextComponentNbt::literal("bye").unwrap();
    let mut before_play = PlayEntryProjection::new(initial_player(), false, false);
    assert_eq!(
        before_play.apply(PlayClientboundPacket::Disconnect(reason.clone())),
        Err(PlayProjectionError::LevelNotInstalled)
    );

    let mut ready = projection(false);
    let action = ready
        .apply(PlayClientboundPacket::Disconnect(reason.clone()))
        .unwrap();
    assert_eq!(action, PlayClientAction::Disconnect(reason));
    assert_eq!(action.response_packet(), None);
}

#[test]
fn keep_alive_and_ping_echo_distinct_signed_domains() {
    let mut projection = projection(false);
    let keep_alive = projection
        .apply(PlayClientboundPacket::KeepAlive(KeepAlive {
            challenge: i64::MIN,
        }))
        .unwrap();
    let ping = projection
        .apply(PlayClientboundPacket::Ping(Ping { payload: i32::MIN }))
        .unwrap();

    let keep_alive_packet = keep_alive.response_packet().unwrap();
    let pong_packet = ping.response_packet().unwrap();
    assert!(matches!(
        keep_alive_packet,
        PlayServerboundEntryPacket::KeepAlive(packet) if packet.challenge == i64::MIN
    ));
    assert_eq!(
        pong_packet,
        PlayServerboundEntryPacket::Pong(Pong { payload: i32::MIN })
    );
    assert_eq!(
        serverbound::encode_packet(keep_alive_packet).unwrap()[0],
        28
    );
    assert_eq!(
        serverbound::encode_packet(pong_packet).unwrap(),
        [45, 128, 0, 0, 0]
    );
}

#[test]
fn frozen_keep_alive_defers_unfreezes_and_expires_at_one_minute() {
    let mut deferred = DeferredKeepAliveEchoes::new(2).unwrap();
    assert_eq!(deferred.receive(10, 1_000, true).unwrap(), None);
    assert_eq!(deferred.poll(60_999, true), Vec::new());
    assert_eq!(deferred.pending_count(), 1);
    assert_eq!(
        deferred.poll(60_999, false),
        [PlayClientAction::EchoKeepAlive(10)]
    );

    deferred.receive(20, 1_000, true).unwrap();
    assert_eq!(deferred.poll(61_000, true), Vec::new());
    assert_eq!(deferred.pending_count(), 0);

    deferred.receive(30, 1_000, true).unwrap();
    assert_eq!(
        deferred.poll(61_000, false),
        [PlayClientAction::EchoKeepAlive(30)]
    );
}

#[test]
fn deferred_keep_alive_queue_is_bounded_and_checks_deadlines() {
    assert_eq!(
        DeferredKeepAliveEchoes::new(0),
        Err(DeferredKeepAliveError::ZeroCapacity)
    );
    let mut deferred = DeferredKeepAliveEchoes::new(1).unwrap();
    deferred.receive(1, 0, true).unwrap();
    assert_eq!(
        deferred.receive(2, 0, true),
        Err(DeferredKeepAliveError::Full { capacity: 1 })
    );
    deferred.poll(0, false);
    assert_eq!(
        deferred.receive(3, i64::MAX, true),
        Err(DeferredKeepAliveError::DeadlineOverflow)
    );
}

#[test]
fn rotation_applies_independent_relativity_clamps_and_echoes_false_flags() {
    let mut projection = projection(false);
    let action = projection
        .apply(PlayClientboundPacket::PlayerRotation(PlayerRotation {
            yaw: 15.0,
            relative_yaw: true,
            pitch: 100.0,
            relative_pitch: true,
        }))
        .unwrap();
    assert_eq!(projection.local_player().yaw, 45.0);
    assert_eq!(projection.local_player().pitch, 90.0);
    assert_eq!(projection.render_rotation().old_yaw, 45.0);
    assert_eq!(projection.render_rotation().old_pitch, 90.0);
    let PlayServerboundEntryPacket::MovePlayerRotation(MovePlayerRotation { rotation, flags }) =
        action.response_packet().unwrap()
    else {
        panic!("rotation response identity changed");
    };
    assert_eq!((rotation.yaw, rotation.pitch), (45.0, 90.0));
    assert!(!flags.on_ground);
    assert!(!flags.horizontal_collision);
    assert_eq!(
        serverbound::encode_packet(action.response_packet().unwrap()).unwrap()[0],
        32
    );
}

#[test]
fn rotation_preserves_nonfinite_handler_boundary() {
    let mut projection = projection(false);
    let action = projection
        .apply(PlayClientboundPacket::PlayerRotation(PlayerRotation {
            yaw: f32::NEG_INFINITY,
            relative_yaw: false,
            pitch: f32::NAN,
            relative_pitch: false,
        }))
        .unwrap();
    assert_eq!(projection.local_player().yaw, f32::NEG_INFINITY);
    assert!(projection.local_player().pitch.is_nan());
    let PlayServerboundEntryPacket::MovePlayerRotation(packet) = action.response_packet().unwrap()
    else {
        panic!("rotation response identity changed");
    };
    assert_eq!(packet.rotation.yaw, f32::NEG_INFINITY);
    assert!(packet.rotation.pitch.is_nan());
}

#[test]
fn vehicle_correction_ignores_absent_and_non_authoritative_roots() {
    let packet = PlayClientboundPacket::MoveVehicle(VehiclePosition {
        position: Vector3::default(),
        yaw: 1.0,
        pitch: 2.0,
    });
    assert_eq!(
        projection(false).apply(packet.clone()).unwrap(),
        PlayClientAction::None
    );

    let mut nonlocal = projection(true);
    let mut vehicle = nonlocal.root_vehicle().unwrap();
    vehicle.locally_authoritative = false;
    nonlocal.set_root_vehicle(Some(vehicle));
    assert_eq!(nonlocal.apply(packet).unwrap(), PlayClientAction::None);
}

#[test]
fn vehicle_uses_interpolation_target_and_ignores_rotation_only_changes() {
    let mut projection = projection(true);
    projection.set_root_vehicle(Some(RootVehicleProjection {
        movement: VehicleMovementState {
            position: Vector3::default(),
            yaw: 10.0,
            pitch: 20.0,
            on_ground: true,
        },
        locally_authoritative: true,
        interpolation_target: Some(Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
    }));
    let action = projection
        .apply(PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            yaw: 90.0,
            pitch: 80.0,
        }))
        .unwrap();
    let PlayClientAction::EchoVehicle(echo) = action else {
        panic!("vehicle response identity changed");
    };
    assert_eq!(echo.position, Vector3::default());
    assert_eq!((echo.yaw, echo.pitch), (10.0, 20.0));
    assert_eq!(
        projection.root_vehicle().unwrap().interpolation_target,
        Some(Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        })
    );
    let PlayServerboundEntryPacket::MoveVehicle(MoveVehicle { on_ground, .. }) =
        action.response_packet().unwrap()
    else {
        panic!("vehicle echo packet changed");
    };
    assert!(on_ground);
    assert_eq!(
        serverbound::encode_packet(action.response_packet().unwrap()).unwrap()[0],
        34
    );
}

#[test]
fn vehicle_distance_threshold_cancels_and_snaps_only_when_exceeded() {
    let threshold = f64::from(1.0e-5_f32);
    let base = RootVehicleProjection {
        movement: VehicleMovementState {
            position: Vector3::default(),
            yaw: 1.0,
            pitch: 2.0,
            on_ground: false,
        },
        locally_authoritative: true,
        interpolation_target: Some(Vector3::default()),
    };

    let mut equal = projection(true);
    equal.set_root_vehicle(Some(base));
    equal
        .apply(PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: Vector3 {
                x: threshold,
                y: 0.0,
                z: 0.0,
            },
            yaw: 3.0,
            pitch: 4.0,
        }))
        .unwrap();
    assert_eq!(equal.root_vehicle().unwrap(), base);

    let mut above = projection(true);
    above.set_root_vehicle(Some(base));
    above
        .apply(PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: Vector3 {
                x: threshold * 1.000_001,
                y: 0.0,
                z: 0.0,
            },
            yaw: 3.0,
            pitch: 4.0,
        }))
        .unwrap();
    let snapped = above.root_vehicle().unwrap();
    assert_eq!(snapped.interpolation_target, None);
    assert_eq!((snapped.movement.yaw, snapped.movement.pitch), (3.0, 4.0));
}

#[test]
fn vehicle_nan_does_not_snap_but_infinity_does() {
    let mut projection = projection(true);
    let before = projection.root_vehicle().unwrap();
    projection
        .apply(PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: Vector3 {
                x: f64::NAN,
                y: 0.0,
                z: 0.0,
            },
            yaw: 1.0,
            pitch: 2.0,
        }))
        .unwrap();
    assert_eq!(projection.root_vehicle(), Some(before));

    projection
        .apply(PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: Vector3 {
                x: f64::INFINITY,
                y: 0.0,
                z: 0.0,
            },
            yaw: 5.0,
            pitch: 6.0,
        }))
        .unwrap();
    let after = projection.root_vehicle().unwrap();
    assert_eq!(after.movement.position.x, f64::INFINITY);
    assert_eq!((after.movement.yaw, after.movement.pitch), (5.0, 6.0));
}
