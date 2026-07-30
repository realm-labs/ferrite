use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::entity_motion::packet::{
    EntityPositionSync, MinecartStep, MoveMinecartAlongTrack, PositionMoveRotation,
    ProjectilePower, RelativePosition, RelativePositionRotation, RelativeRotation, RotateHead,
    SetEntityMotion, TeleportEntity, decode_rotation, encode_rotation,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_motion::projection::{
    EntityMotionAction, EntityMotionClientProjection, InterpolationMode, InterpolationTarget,
    LocalPlayerMotionState, MinecartProjectionKind, TrackedMotionEntity,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_motion::publication::{
    PosePublication, TrackerPublicationInput, TrackerPublicationStep, riding_teleport_publication,
    should_publish_velocity, tracker_publication_plan,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn change() -> PositionMoveRotation {
    PositionMoveRotation {
        position: Vector3 {
            x: 1.0,
            y: -2.0,
            z: 3.0,
        },
        motion: Vector3 {
            x: 0.25,
            y: -0.5,
            z: 0.75,
        },
        yaw: 90.0,
        pitch: -45.0,
    }
}

fn zero_change() -> PositionMoveRotation {
    PositionMoveRotation {
        position: Vector3::default(),
        motion: Vector3::default(),
        yaw: 0.0,
        pitch: 0.0,
    }
}

fn step(weight: f32) -> MinecartStep {
    MinecartStep {
        position: Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        motion: Vector3 {
            x: -1.0,
            y: -2.0,
            z: -3.0,
        },
        yaw: i8::MIN,
        pitch: i8::MAX,
        weight,
    }
}

fn assert_roundtrip(packet: PlayClientboundPacket) {
    let registries = PlayRegistries::default();
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );
}

#[test]
fn c3_gold_entity_motion_locks_all_nine_packet_bodies() {
    let registries = PlayRegistries::default();
    let mut position_sync = vec![0x23, 0x00];
    position_sync.extend_from_slice(&[0; 56]);
    position_sync.push(0);
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::EntityPositionSync(EntityPositionSync {
                entity_id: 0,
                change: zero_change(),
                on_ground: false,
            }),
            &registries,
        )
        .unwrap(),
        position_sync
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::MoveEntityPosition(RelativePosition {
                entity_id: 0,
                delta_x: 0,
                delta_y: 0,
                delta_z: 0,
                on_ground: false,
            }),
            &registries,
        )
        .unwrap(),
        [0x35, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::MoveEntityPositionRotation(RelativePositionRotation {
                entity_id: 0,
                delta_x: 0,
                delta_y: 0,
                delta_z: 0,
                yaw: 0,
                pitch: 0,
                on_ground: false,
            }),
            &registries,
        )
        .unwrap(),
        [0x36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::MoveMinecartAlongTrack(MoveMinecartAlongTrack {
                entity_id: 0,
                steps: Vec::new(),
            }),
            &registries,
        )
        .unwrap(),
        [0x37, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::MoveEntityRotation(RelativeRotation {
                entity_id: 0,
                yaw: 0,
                pitch: 0,
                on_ground: false,
            }),
            &registries,
        )
        .unwrap(),
        [0x38, 0, 0, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::RotateHead(RotateHead {
                entity_id: 0,
                head_yaw: 0,
            }),
            &registries,
        )
        .unwrap(),
        [0x53, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::SetEntityMotion(SetEntityMotion {
                entity_id: 0,
                motion: Vector3::default(),
            }),
            &registries,
        )
        .unwrap(),
        [0x65, 0, 0]
    );
    let mut teleport = vec![0x7d, 0];
    teleport.extend_from_slice(&[0; 56]);
    teleport.extend_from_slice(&[0; 5]);
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::TeleportEntity(TeleportEntity {
                entity_id: 0,
                change: zero_change(),
                relative_flags: 0,
                on_ground: false,
            }),
            &registries,
        )
        .unwrap(),
        teleport
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::ProjectilePower(ProjectilePower {
                entity_id: 0,
                acceleration_power: 0.0,
            }),
            &registries,
        )
        .unwrap(),
        [0x87, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn c3_entity_motion_codecs_preserve_signed_ieee_mask_rotation_and_boolean_domains() {
    for packet in [
        PlayClientboundPacket::EntityPositionSync(EntityPositionSync {
            entity_id: i32::MIN,
            change: PositionMoveRotation {
                position: Vector3 {
                    x: f64::from_bits(1),
                    y: f64::INFINITY,
                    z: f64::NEG_INFINITY,
                },
                motion: Vector3 {
                    x: f64::MAX,
                    y: f64::MIN,
                    z: -0.0,
                },
                yaw: f32::INFINITY,
                pitch: f32::NEG_INFINITY,
            },
            on_ground: true,
        }),
        PlayClientboundPacket::MoveEntityPositionRotation(RelativePositionRotation {
            entity_id: i32::MAX,
            delta_x: i16::MIN,
            delta_y: 0,
            delta_z: i16::MAX,
            yaw: i8::MIN,
            pitch: i8::MAX,
            on_ground: true,
        }),
        PlayClientboundPacket::TeleportEntity(TeleportEntity {
            entity_id: -1,
            change: change(),
            relative_flags: u32::MAX,
            on_ground: true,
        }),
        PlayClientboundPacket::ProjectilePower(ProjectilePower {
            entity_id: i32::MIN,
            acceleration_power: f64::INFINITY,
        }),
    ] {
        assert_roundtrip(packet);
    }

    assert_eq!(decode_rotation(i8::MIN), -180.0);
    assert_eq!(decode_rotation(64), 90.0);
    assert_eq!(encode_rotation(90.9), 64);
    assert_eq!(encode_rotation(-180.0), i8::MIN);

    let registries = PlayRegistries::default();
    let decoded = decode_packet(&[0x38, 0, 0, 0, 0x7f], context(&registries)).unwrap();
    let encoded = encode_packet(&decoded, &registries).unwrap();
    assert_eq!(encoded[4], 1);
}

#[test]
fn c3_lp_vec_canonicalizes_inputs_and_accepts_noncanonical_finite_forms() {
    let registries = PlayRegistries::default();
    let packet = PlayClientboundPacket::SetEntityMotion(SetEntityMotion {
        entity_id: 3,
        motion: Vector3 {
            x: f64::NAN,
            y: f64::INFINITY,
            z: f64::NEG_INFINITY,
        },
    });
    let encoded = encode_packet(&packet, &registries).unwrap();
    let PlayClientboundPacket::SetEntityMotion(decoded) =
        decode_packet(&encoded, context(&registries)).unwrap()
    else {
        panic!("expected motion");
    };
    assert!(decoded.motion.x.is_finite());
    assert!(decoded.motion.y.is_finite());
    assert!(decoded.motion.z.is_finite());
    assert_eq!(decoded.motion.x, 0.0);
    assert!(decoded.motion.y > 17_000_000_000.0);
    assert!(decoded.motion.z < -17_000_000_000.0);

    let noncanonical_zero_scale = [0x65, 0, 0x08, 0, 0, 0, 0, 0];
    let PlayClientboundPacket::SetEntityMotion(decoded) =
        decode_packet(&noncanonical_zero_scale, context(&registries)).unwrap()
    else {
        panic!("expected motion");
    };
    assert_eq!(decoded.motion, Vector3::default());

    let near_zero = PlayClientboundPacket::SetEntityMotion(SetEntityMotion {
        entity_id: 0,
        motion: Vector3 {
            x: 1.0e-6,
            y: -1.0e-6,
            z: 0.0,
        },
    });
    assert_eq!(
        encode_packet(&near_zero, &registries).unwrap(),
        [0x65, 0, 0]
    );

    let continued_scale = PlayClientboundPacket::SetEntityMotion(SetEntityMotion {
        entity_id: 0,
        motion: Vector3 {
            x: 4.0,
            y: 0.0,
            z: -4.0,
        },
    });
    let encoded = encode_packet(&continued_scale, &registries).unwrap();
    assert_ne!(encoded[2] & 0x04, 0);
    let PlayClientboundPacket::SetEntityMotion(decoded) =
        decode_packet(&encoded, context(&registries)).unwrap()
    else {
        panic!("expected motion");
    };
    assert_eq!(
        decoded.motion,
        Vector3 {
            x: 4.0,
            y: 0.0,
            z: -4.0,
        }
    );
}

#[test]
fn c3_minecart_codec_accepts_raw_steps_and_rejects_counts_and_framing() {
    assert_roundtrip(PlayClientboundPacket::MoveMinecartAlongTrack(
        MoveMinecartAlongTrack {
            entity_id: -1,
            steps: vec![step(f32::INFINITY), step(f32::NEG_INFINITY)],
        },
    ));
    let registries = PlayRegistries::default();
    for malformed in [
        vec![0x23],
        vec![0x35, 0],
        vec![0x37, 0, 0xff, 0xff, 0xff, 0xff, 0x0f],
        vec![0x38, 0],
        vec![0x53, 0],
        vec![0x65, 0, 0x04],
        vec![0x7d, 0],
        vec![0x87, 0x01, 0],
        vec![0x53, 0, 0, 0],
    ] {
        assert!(decode_packet(&malformed, context(&registries)).is_err());
    }
}

#[test]
fn c3_relative_projection_updates_base_before_local_authority_gate() {
    let mut client = EntityMotionClientProjection::default();
    client.track_entity(
        1,
        TrackedMotionEntity {
            packet_position_base: Vector3 {
                x: f64::INFINITY,
                y: -1.5 / 4_096.0,
                z: f64::NEG_INFINITY,
            },
            locally_authoritative: true,
            on_ground: false,
            ..TrackedMotionEntity::default()
        },
    );
    let zero = PlayClientboundPacket::MoveEntityPositionRotation(RelativePositionRotation {
        entity_id: 1,
        delta_x: 0,
        delta_y: 0,
        delta_z: 0,
        yaw: 64,
        pitch: -64,
        on_ground: true,
    });
    assert_eq!(client.apply(&zero), EntityMotionAction::PacketBaseOnly);
    let entity = client.entity(1).unwrap();
    assert!(entity.packet_position_base.x.is_infinite());
    assert_eq!(
        entity.packet_position_base.y.to_bits(),
        (-1.5_f64 / 4_096.0).to_bits()
    );
    assert!(!entity.on_ground);
    assert_eq!((entity.yaw, entity.pitch), (0.0, 0.0));

    let delta = PlayClientboundPacket::MoveEntityPosition(RelativePosition {
        entity_id: 1,
        delta_x: 1,
        delta_y: 1,
        delta_z: -1,
        on_ground: true,
    });
    client.apply(&delta);
    let base = client.entity(1).unwrap().packet_position_base;
    assert!(base.x.is_finite());
    assert_eq!(base.y, 0.0);
    assert!(base.z.is_finite());
}

#[test]
fn c3_relative_projection_interpolates_three_ticks_and_does_not_reset_same_target() {
    let mut client = EntityMotionClientProjection::default();
    client.track_entity(2, TrackedMotionEntity::default());
    let packet = PlayClientboundPacket::MoveEntityPositionRotation(RelativePositionRotation {
        entity_id: 2,
        delta_x: 4_096,
        delta_y: 0,
        delta_z: 0,
        yaw: 64,
        pitch: -64,
        on_ground: true,
    });
    assert_eq!(client.apply(&packet), EntityMotionAction::Interpolated);
    assert!(client.tick_interpolation(2));
    assert_eq!(client.entity(2).unwrap().position.x, 1.0 / 3.0);
    assert_eq!(
        client
            .entity(2)
            .unwrap()
            .interpolation_target
            .unwrap()
            .remaining_steps,
        2
    );
    let same_target = PlayClientboundPacket::EntityPositionSync(EntityPositionSync {
        entity_id: 2,
        change: PositionMoveRotation {
            position: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            motion: Vector3::default(),
            yaw: 90.0,
            pitch: -90.0,
        },
        on_ground: true,
    });
    client.apply(&same_target);
    assert_eq!(
        client
            .entity(2)
            .unwrap()
            .interpolation_target
            .unwrap()
            .remaining_steps,
        2
    );
    client.tick_interpolation(2);
    client.tick_interpolation(2);
    let entity = client.entity(2).unwrap();
    assert_eq!(entity.position.x, 1.0);
    assert_eq!((entity.yaw, entity.pitch), (90.0, -90.0));
    assert!(entity.on_ground);
}

#[test]
fn c3_absolute_sync_ignores_velocity_and_obeys_snap_local_and_rider_branches() {
    let mut client = EntityMotionClientProjection::default();
    client.track_entity(
        3,
        TrackedMotionEntity {
            motion: Vector3 {
                x: 9.0,
                y: 8.0,
                z: 7.0,
            },
            ..TrackedMotionEntity::default()
        },
    );
    let sync = PlayClientboundPacket::EntityPositionSync(EntityPositionSync {
        entity_id: 3,
        change: PositionMoveRotation {
            motion: Vector3 {
                x: -1.0,
                y: -2.0,
                z: -3.0,
            },
            ..change()
        },
        on_ground: true,
    });
    assert_eq!(client.apply(&sync), EntityMotionAction::Interpolated);
    assert_eq!(
        client.entity(3).unwrap().motion,
        Vector3 {
            x: 9.0,
            y: 8.0,
            z: 7.0,
        }
    );

    client.track_entity(
        4,
        TrackedMotionEntity {
            locally_authoritative: true,
            ..TrackedMotionEntity::default()
        },
    );
    let local = PlayClientboundPacket::EntityPositionSync(EntityPositionSync {
        entity_id: 4,
        change: change(),
        on_ground: true,
    });
    assert_eq!(client.apply(&local), EntityMotionAction::PacketBaseOnly);
    assert_eq!(client.entity(4).unwrap().position, Vector3::default());
    assert_eq!(
        client.entity(4).unwrap().packet_position_base,
        change().position
    );

    client.track_entity(
        5,
        TrackedMotionEntity {
            position: Vector3 {
                x: 100.0,
                y: 0.0,
                z: 0.0,
            },
            noninterpolating_vehicle: true,
            carries_local_player: true,
            ..TrackedMotionEntity::default()
        },
    );
    let rider = PlayClientboundPacket::EntityPositionSync(EntityPositionSync {
        entity_id: 5,
        change: zero_change(),
        on_ground: false,
    });
    assert_eq!(client.apply(&rider), EntityMotionAction::RiderRepositioned);
    assert_eq!(client.entity(5).unwrap().position, Vector3::default());
}

#[test]
fn c3_immediate_handler_and_active_teleport_target_follow_specialized_paths() {
    let mut client = EntityMotionClientProjection::default();
    client.track_entity(
        30,
        TrackedMotionEntity {
            interpolation_mode: InterpolationMode::Immediate,
            ..TrackedMotionEntity::default()
        },
    );
    let sync = PlayClientboundPacket::EntityPositionSync(EntityPositionSync {
        entity_id: 30,
        change: PositionMoveRotation {
            yaw: 450.0,
            pitch: -450.0,
            ..change()
        },
        on_ground: true,
    });
    assert_eq!(client.apply(&sync), EntityMotionAction::Applied);
    assert_eq!(
        (
            client.entity(30).unwrap().yaw,
            client.entity(30).unwrap().pitch
        ),
        (90.0, -90.0)
    );

    client.track_entity(
        31,
        TrackedMotionEntity {
            interpolation_target: Some(InterpolationTarget {
                position: Vector3 {
                    x: 10.0,
                    y: 20.0,
                    z: 30.0,
                },
                yaw: 40.0,
                pitch: 50.0,
                remaining_steps: 2,
            }),
            locally_authoritative: true,
            ticking: false,
            ..TrackedMotionEntity::default()
        },
    );
    let teleport = PlayClientboundPacket::TeleportEntity(TeleportEntity {
        entity_id: 31,
        change: PositionMoveRotation {
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            motion: Vector3::default(),
            yaw: 5.0,
            pitch: 6.0,
        },
        relative_flags: 0x1f,
        on_ground: false,
    });
    assert_eq!(client.apply(&teleport), EntityMotionAction::Interpolated);
    let target = client.entity(31).unwrap().interpolation_target.unwrap();
    assert_eq!(
        target.position,
        Vector3 {
            x: 11.0,
            y: 22.0,
            z: 33.0,
        }
    );
    assert_eq!((target.yaw, target.pitch), (45.0, 56.0));
}

#[test]
fn c3_teleport_uses_interpolation_source_relative_mask_and_direct_vehicle_echo() {
    let mut client = EntityMotionClientProjection::default();
    client.track_entity(
        6,
        TrackedMotionEntity {
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            motion: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            yaw: 90.0,
            pitch: 0.0,
            locally_authoritative: true,
            ticking: false,
            carries_local_player: true,
            ..TrackedMotionEntity::default()
        },
    );
    let teleport = PlayClientboundPacket::TeleportEntity(TeleportEntity {
        entity_id: 6,
        change: PositionMoveRotation {
            position: Vector3 {
                x: 100.0,
                y: 4.0,
                z: 6.0,
            },
            motion: Vector3 {
                x: 0.5,
                y: 0.0,
                z: 0.0,
            },
            yaw: 90.0,
            pitch: 100.0,
        },
        relative_flags: (1 << 0) | (1 << 3) | (1 << 5) | (1 << 8) | (1 << 31),
        on_ground: true,
    });
    assert!(matches!(
        client.apply(&teleport),
        EntityMotionAction::EchoVehicle { .. }
    ));
    let entity = client.entity(6).unwrap();
    assert_eq!(
        entity.position,
        Vector3 {
            x: 101.0,
            y: 4.0,
            z: 6.0,
        }
    );
    assert_eq!((entity.yaw, entity.pitch), (180.0, 90.0));
    assert!(entity.on_ground);
    assert!((entity.motion.x - 0.5).abs() < 1.0e-6);
}

#[test]
fn c3_missing_retained_vehicle_teleport_moves_player_without_clearing_marker() {
    let mut client = EntityMotionClientProjection::default();
    client.set_local_player(LocalPlayerMotionState {
        position: Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        motion: Vector3::default(),
        yaw: 10.0,
        pitch: 20.0,
    });
    client.retain_removed_player_vehicle(99);
    let packet = PlayClientboundPacket::TeleportEntity(TeleportEntity {
        entity_id: 99,
        change: PositionMoveRotation {
            position: Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            motion: Vector3::default(),
            yaw: 5.0,
            pitch: -100.0,
        },
        relative_flags: 0x1f,
        on_ground: true,
    });
    assert_eq!(
        client.apply(&packet),
        EntityMotionAction::EchoPlayerPositionRotation {
            position: Vector3 {
                x: 2.0,
                y: 3.0,
                z: 4.0,
            },
            yaw: 15.0,
            pitch: -80.0,
            on_ground: false,
            horizontal_collision: false,
        }
    );
    assert_eq!(client.retained_vehicle_id(), Some(99));
    client.clear_retained_vehicle_on_add(99);
    assert_eq!(client.retained_vehicle_id(), None);
}

#[test]
fn c3_velocity_head_projectile_and_minecart_runtime_gates_are_exact() {
    let mut client = EntityMotionClientProjection::default();
    client.track_entity(
        7,
        TrackedMotionEntity {
            living: true,
            hurting_projectile: true,
            minecart_kind: MinecartProjectionKind::NewBehaviorEnabled,
            ..TrackedMotionEntity::default()
        },
    );
    assert_eq!(
        client.apply(&PlayClientboundPacket::SetEntityMotion(SetEntityMotion {
            entity_id: 7,
            motion: change().motion,
        })),
        EntityMotionAction::Applied
    );
    assert_eq!(
        client.apply(&PlayClientboundPacket::RotateHead(RotateHead {
            entity_id: 7,
            head_yaw: -64,
        })),
        EntityMotionAction::Interpolated
    );
    for _ in 0..3 {
        assert!(client.tick_head_interpolation(7));
    }
    assert_eq!(client.entity(7).unwrap().head_yaw, -90.0);
    assert_eq!(
        client.apply(&PlayClientboundPacket::ProjectilePower(ProjectilePower {
            entity_id: 7,
            acceleration_power: f64::NAN,
        })),
        EntityMotionAction::Applied
    );
    assert!(client.entity(7).unwrap().acceleration_power.is_nan());

    let minecart = PlayClientboundPacket::MoveMinecartAlongTrack(MoveMinecartAlongTrack {
        entity_id: 7,
        steps: vec![step(-1.0), step(2.0), step(f32::NAN)],
    });
    assert_eq!(client.apply(&minecart), EntityMotionAction::Applied);
    assert!(client.activate_minecart_window(7).unwrap().is_nan());
    assert_eq!(client.select_minecart_step(7, f64::NAN), Some(2));

    client.track_entity(
        8,
        TrackedMotionEntity {
            minecart_kind: MinecartProjectionKind::OldBehavior,
            ..TrackedMotionEntity::default()
        },
    );
    assert_eq!(
        client.apply(&PlayClientboundPacket::MoveMinecartAlongTrack(
            MoveMinecartAlongTrack {
                entity_id: 8,
                steps: vec![step(1.0)],
            },
        )),
        EntityMotionAction::Ignored
    );
    let old_motion = PlayClientboundPacket::SetEntityMotion(SetEntityMotion {
        entity_id: 8,
        motion: change().motion,
    });
    client.apply(&old_motion);
    assert_eq!(
        client.entity(8).unwrap().old_minecart_motion_target,
        Some(change().motion)
    );
}

fn publication_input() -> TrackerPublicationInput {
    TrackerPublicationInput {
        passenger: false,
        new_behavior_minecart: false,
        minecart_has_recorded_steps: false,
        velocity_changed: false,
        hurting_projectile: false,
        position_changed: false,
        sixty_tick_refresh: false,
        rotation_changed: false,
        abstract_arrow: false,
        precise_position_required: false,
        delta_x: 0,
        delta_y: 0,
        delta_z: 0,
        ordinary_passes_since_absolute: 0,
        just_stopped_riding: false,
        on_ground_changed: false,
        dirty_state: false,
        head_rotation_changed: false,
        hurt_marked: false,
    }
}

#[test]
fn c3_tracker_publication_locks_pose_choice_and_global_order() {
    let plan = tracker_publication_plan(TrackerPublicationInput {
        velocity_changed: true,
        hurting_projectile: true,
        position_changed: true,
        rotation_changed: true,
        dirty_state: true,
        head_rotation_changed: true,
        hurt_marked: true,
        ..publication_input()
    });
    assert_eq!(
        plan,
        vec![
            TrackerPublicationStep::Motion,
            TrackerPublicationStep::ProjectilePower,
            TrackerPublicationStep::Pose(PosePublication::RelativePositionRotation),
            TrackerPublicationStep::DirtyState,
            TrackerPublicationStep::HeadRotation,
            TrackerPublicationStep::HurtMotionToTrackersAndSelf,
        ]
    );
    for input in [
        TrackerPublicationInput {
            precise_position_required: true,
            ..publication_input()
        },
        TrackerPublicationInput {
            delta_x: i64::from(i16::MAX) + 1,
            ..publication_input()
        },
        TrackerPublicationInput {
            ordinary_passes_since_absolute: 401,
            ..publication_input()
        },
        TrackerPublicationInput {
            just_stopped_riding: true,
            ..publication_input()
        },
        TrackerPublicationInput {
            on_ground_changed: true,
            ..publication_input()
        },
    ] {
        assert_eq!(
            tracker_publication_plan(input),
            vec![TrackerPublicationStep::Pose(PosePublication::PositionSync)]
        );
    }
}

#[test]
fn c3_tracker_passenger_minecart_velocity_and_riding_teleport_branches_are_exact() {
    assert_eq!(
        tracker_publication_plan(TrackerPublicationInput {
            passenger: true,
            rotation_changed: true,
            dirty_state: true,
            ..publication_input()
        }),
        vec![
            TrackerPublicationStep::Pose(PosePublication::RelativeRotation),
            TrackerPublicationStep::ResetPacketPositionBase,
            TrackerPublicationStep::DirtyState,
        ]
    );
    assert_eq!(
        tracker_publication_plan(TrackerPublicationInput {
            new_behavior_minecart: true,
            minecart_has_recorded_steps: false,
            dirty_state: true,
            ..publication_input()
        }),
        vec![
            TrackerPublicationStep::MinecartSteps {
                current_snapshot: true,
            },
            TrackerPublicationStep::ResetPacketPositionBase,
            TrackerPublicationStep::DirtyState,
        ]
    );
    assert!(!should_publish_velocity(
        Vector3::default(),
        Vector3::default(),
        true
    ));
    assert!(should_publish_velocity(
        Vector3 {
            x: 1.0e-5,
            y: 0.0,
            z: 0.0,
        },
        Vector3::default(),
        true
    ));
    assert!(!should_publish_velocity(
        Vector3::default(),
        Vector3 {
            x: 1.0e-5,
            y: 0.0,
            z: 0.0,
        },
        true
    ));

    let publication = riding_teleport_publication(change(), zero_change(), u32::MAX);
    assert_eq!(publication.controller_relative_flags, u32::MAX);
    assert_eq!(publication.other_passenger, zero_change());
    assert_eq!(publication.other_passenger_relative_flags, 0);
}
