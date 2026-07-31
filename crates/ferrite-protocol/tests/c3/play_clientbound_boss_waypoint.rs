use ferrite_protocol::java_26_2::play::clientbound::boss_waypoint::codec::BossWaypointCodecError;
use ferrite_protocol::java_26_2::play::clientbound::boss_waypoint::packet::{
    BossColor, BossEvent, BossOperation, BossOverlay, TrackedWaypoint, WaypointIcon,
    WaypointIdentifier, WaypointLocation, WaypointOperation, WaypointPacket,
};
use ferrite_protocol::java_26_2::play::clientbound::boss_waypoint::projection::{
    BossClientProjection, BossCollectionAction, BossProjectionError, TrackedEntityEye,
    WaypointClientProjection, WaypointCollectionAction, WaypointProjectionError, WaypointViewer,
};
use ferrite_protocol::java_26_2::play::clientbound::boss_waypoint::publication::{
    BossPublisher, WaypointReceiver, WaypointSource, resolved_icon, select_waypoint,
    waypoint_transition,
};
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn literal(value: &str) -> TextComponentNbt {
    TextComponentNbt::literal(value).unwrap()
}

fn icon() -> WaypointIcon {
    WaypointIcon {
        style: id("minecraft:default"),
        color: None,
    }
}

fn waypoint(operation: WaypointOperation, location: WaypointLocation) -> PlayClientboundPacket {
    PlayClientboundPacket::Waypoint(WaypointPacket {
        operation,
        waypoint: TrackedWaypoint {
            identifier: WaypointIdentifier::Uuid(0),
            icon: icon(),
            location,
        },
    })
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
fn c3_gold_clientbound_boss_waypoint_locks_all_six_packet_bodies() {
    let registries = PlayRegistries::default();
    let add = PlayClientboundPacket::BossEvent(BossEvent {
        id: 0,
        operation: BossOperation::Add {
            name: literal(""),
            progress: 1.0,
            color: BossColor::Pink,
            overlay: BossOverlay::Progress,
            properties: 0,
        },
    });
    assert_eq!(
        encode_packet(&add, &registries).unwrap(),
        [
            vec![0x09],
            vec![0; 16],
            vec![
                0x00, 0x08, 0x00, 0x00, 0x3f, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00
            ],
        ]
        .concat()
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::BossEvent(BossEvent {
                id: 0,
                operation: BossOperation::Remove,
            }),
            &registries,
        )
        .unwrap(),
        [vec![0x09], vec![0; 16], vec![0x01]].concat()
    );

    let expected_prefix = [vec![0x8a, 0x01], vec![0x01, 0x01], vec![0; 16]].concat();
    let expected_icon = [vec![0x11], b"minecraft:default".to_vec(), vec![0x00]].concat();
    for (packet, operation, tail) in [
        (
            waypoint(WaypointOperation::Untrack, WaypointLocation::Empty),
            1,
            vec![0],
        ),
        (
            waypoint(
                WaypointOperation::Track,
                WaypointLocation::Position { x: 0, y: 0, z: 0 },
            ),
            0,
            vec![1, 0, 0, 0],
        ),
        (
            waypoint(
                WaypointOperation::Track,
                WaypointLocation::Chunk { x: 0, z: 0 },
            ),
            0,
            vec![2, 0, 0],
        ),
        (
            waypoint(
                WaypointOperation::Track,
                WaypointLocation::Azimuth { angle: 0.0 },
            ),
            0,
            vec![3, 0, 0, 0, 0],
        ),
    ] {
        let mut expected = expected_prefix.clone();
        expected[2] = operation;
        expected.extend_from_slice(&expected_icon);
        expected.extend_from_slice(&tail);
        assert_eq!(encode_packet(&packet, &registries).unwrap(), expected);
    }
}

#[test]
fn c3_boss_waypoint_require_an_installed_play_level_before_projection() {
    for packet in [
        PlayClientboundPacket::BossEvent(BossEvent {
            id: 0,
            operation: BossOperation::Remove,
        }),
        waypoint(WaypointOperation::Untrack, WaypointLocation::Empty),
    ] {
        assert!(matches!(
            PlayEntryProjection::default().apply(packet),
            Err(PlayProjectionError::LevelNotInstalled)
        ));
    }
}

#[test]
fn c3_boss_codecs_preserve_every_operation_enum_property_and_ieee_class() {
    for operation in [
        BossOperation::Add {
            name: literal("add"),
            progress: f32::INFINITY,
            color: BossColor::White,
            overlay: BossOverlay::Notched20,
            properties: 0xff,
        },
        BossOperation::Remove,
        BossOperation::UpdateProgress(f32::NEG_INFINITY),
        BossOperation::UpdateName(literal("name")),
        BossOperation::UpdateStyle {
            color: BossColor::Purple,
            overlay: BossOverlay::Notched12,
        },
        BossOperation::UpdateProperties(0xf8),
    ] {
        assert_roundtrip(PlayClientboundPacket::BossEvent(BossEvent {
            id: u128::MAX,
            operation,
        }));
    }
    for color in [
        BossColor::Pink,
        BossColor::Blue,
        BossColor::Red,
        BossColor::Green,
        BossColor::Yellow,
        BossColor::Purple,
        BossColor::White,
    ] {
        assert_roundtrip(PlayClientboundPacket::BossEvent(BossEvent {
            id: 1,
            operation: BossOperation::UpdateStyle {
                color,
                overlay: BossOverlay::Progress,
            },
        }));
    }
    for overlay in [
        BossOverlay::Progress,
        BossOverlay::Notched6,
        BossOverlay::Notched10,
        BossOverlay::Notched12,
        BossOverlay::Notched20,
    ] {
        assert_roundtrip(PlayClientboundPacket::BossEvent(BossEvent {
            id: 1,
            operation: BossOperation::UpdateStyle {
                color: BossColor::Pink,
                overlay,
            },
        }));
    }

    let nan_bits = 0x7fc1_2345;
    let nan_packet = PlayClientboundPacket::BossEvent(BossEvent {
        id: 2,
        operation: BossOperation::UpdateProgress(f32::from_bits(nan_bits)),
    });
    let registries = PlayRegistries::default();
    let encoded = encode_packet(&nan_packet, &registries).unwrap();
    let PlayClientboundPacket::BossEvent(decoded) =
        decode_packet(&encoded, context(&registries)).unwrap()
    else {
        panic!("boss event expected");
    };
    let BossOperation::UpdateProgress(progress) = decoded.operation else {
        panic!("boss progress update expected");
    };
    assert_eq!(progress.to_bits(), nan_bits);
}

#[test]
fn c3_waypoint_codec_wraps_operations_and_preserves_identifiers_icons_and_locations() {
    for location in [
        WaypointLocation::Empty,
        WaypointLocation::Position {
            x: i32::MIN,
            y: 0,
            z: i32::MAX,
        },
        WaypointLocation::Chunk {
            x: i32::MAX,
            z: i32::MIN,
        },
        WaypointLocation::Azimuth {
            angle: f32::NEG_INFINITY,
        },
    ] {
        assert_roundtrip(PlayClientboundPacket::Waypoint(WaypointPacket {
            operation: WaypointOperation::Update,
            waypoint: TrackedWaypoint {
                identifier: WaypointIdentifier::String("opaque key".to_owned()),
                icon: WaypointIcon {
                    style: id("custom:style"),
                    color: Some(0xff12_3456),
                },
                location,
            },
        }));
    }

    let registries = PlayRegistries::default();
    let canonical = encode_packet(
        &waypoint(WaypointOperation::Track, WaypointLocation::Empty),
        &registries,
    )
    .unwrap();
    for (raw, expected) in [
        (-3, WaypointOperation::Track),
        (-2, WaypointOperation::Untrack),
        (-1, WaypointOperation::Update),
        (3, WaypointOperation::Track),
        (4, WaypointOperation::Untrack),
        (5, WaypointOperation::Update),
    ] {
        let mut encoded = canonical.clone();
        encoded.splice(2..3, encode_var_i32(raw));
        let PlayClientboundPacket::Waypoint(decoded) =
            decode_packet(&encoded, context(&registries)).unwrap()
        else {
            panic!("waypoint packet expected");
        };
        assert_eq!(decoded.operation, expected);
    }
    let mut nonzero_bool = canonical;
    nonzero_bool[3] = 2;
    assert!(decode_packet(&nonzero_bool, context(&registries)).is_ok());

    let mut nonzero_color_bool = encode_packet(
        &waypoint(WaypointOperation::Track, WaypointLocation::Empty),
        &registries,
    )
    .unwrap();
    let color_presence_index = nonzero_color_bool.len() - 2;
    nonzero_color_bool[color_presence_index] = 2;
    nonzero_color_bool.splice(
        color_presence_index + 1..color_presence_index + 1,
        [1, 2, 3],
    );
    let PlayClientboundPacket::Waypoint(decoded) =
        decode_packet(&nonzero_color_bool, context(&registries)).unwrap()
    else {
        panic!("waypoint packet expected");
    };
    assert_eq!(decoded.waypoint.icon.color, Some(0xff01_0203));
}

#[test]
fn c3_boss_waypoint_malformed_values_and_trailing_data_fail_closed() {
    let registries = PlayRegistries::default();
    let mut unknown_operation = vec![0x09];
    unknown_operation.extend_from_slice(&[0; 16]);
    unknown_operation.push(6);
    assert!(matches!(
        decode_packet(&unknown_operation, context(&registries)),
        Err(PlayClientboundCodecError::BossWaypoint(
            BossWaypointCodecError::UnknownBossOperation { ordinal: 6 }
        ))
    ));

    let mut unknown_color = vec![0x09];
    unknown_color.extend_from_slice(&[0; 16]);
    unknown_color.extend_from_slice(&[4, 7, 0]);
    assert!(matches!(
        decode_packet(&unknown_color, context(&registries)),
        Err(PlayClientboundCodecError::BossWaypoint(
            BossWaypointCodecError::UnknownBossColor { ordinal: 7 }
        ))
    ));
    let mut unknown_overlay = vec![0x09];
    unknown_overlay.extend_from_slice(&[0; 16]);
    unknown_overlay.extend_from_slice(&[4, 0, 5]);
    assert!(matches!(
        decode_packet(&unknown_overlay, context(&registries)),
        Err(PlayClientboundCodecError::BossWaypoint(
            BossWaypointCodecError::UnknownBossOverlay { ordinal: 5 }
        ))
    ));

    let mut unknown_type = encode_packet(
        &waypoint(WaypointOperation::Track, WaypointLocation::Empty),
        &registries,
    )
    .unwrap();
    *unknown_type.last_mut().unwrap() = 4;
    assert!(matches!(
        decode_packet(&unknown_type, context(&registries)),
        Err(PlayClientboundCodecError::BossWaypoint(
            BossWaypointCodecError::UnknownWaypointType { ordinal: 4 }
        ))
    ));
    for malformed in [vec![0x09], vec![0x8a, 0x01], {
        let mut trailing = encode_packet(
            &waypoint(WaypointOperation::Track, WaypointLocation::Empty),
            &registries,
        )
        .unwrap();
        trailing.push(0);
        trailing
    }] {
        assert!(decode_packet(&malformed, context(&registries)).is_err());
    }
}

#[test]
fn c3_boss_collection_replaces_in_place_lerps_and_aggregates_low_property_bits() {
    let mut client = BossClientProjection::default();
    for (id, properties) in [(1, 0xf9), (2, 0x06)] {
        assert_eq!(
            client
                .apply(
                    &BossEvent {
                        id,
                        operation: BossOperation::Add {
                            name: literal("boss"),
                            progress: 0.0,
                            color: BossColor::Pink,
                            overlay: BossOverlay::Progress,
                            properties,
                        },
                    },
                    0,
                )
                .unwrap(),
            BossCollectionAction::Added { replaced: false }
        );
    }
    assert_eq!(client.ordered_ids(), &[1, 2]);
    assert_eq!(client.rendered_ids(120), vec![1, 2]);
    client
        .apply(
            &BossEvent {
                id: 1,
                operation: BossOperation::Add {
                    name: literal("replacement"),
                    progress: 0.0,
                    color: BossColor::Blue,
                    overlay: BossOverlay::Notched6,
                    properties: 1,
                },
            },
            0,
        )
        .unwrap();
    assert_eq!(client.ordered_ids(), &[1, 2]);
    client
        .apply(
            &BossEvent {
                id: 1,
                operation: BossOperation::UpdateProgress(1.0),
            },
            0,
        )
        .unwrap();
    assert_eq!(client.bar(1).unwrap().visible_progress(50), 0.5);
    client
        .apply(
            &BossEvent {
                id: 1,
                operation: BossOperation::UpdateProgress(0.0),
            },
            50,
        )
        .unwrap();
    assert_eq!(client.bar(1).unwrap().visible_progress(100), 0.25);
    client
        .apply(
            &BossEvent {
                id: 1,
                operation: BossOperation::UpdateName(literal("renamed")),
            },
            100,
        )
        .unwrap();
    client
        .apply(
            &BossEvent {
                id: 1,
                operation: BossOperation::UpdateStyle {
                    color: BossColor::Green,
                    overlay: BossOverlay::Notched10,
                },
            },
            100,
        )
        .unwrap();
    let updated = client.bar(1).unwrap();
    assert_eq!(updated.name, literal("renamed"));
    assert_eq!(updated.color, BossColor::Green);
    assert_eq!(updated.overlay, BossOverlay::Notched10);
    let aggregate = client.aggregate();
    assert!(aggregate.darken_screen && aggregate.play_music && aggregate.create_fog);
    assert_eq!(
        client.apply(
            &BossEvent {
                id: 99,
                operation: BossOperation::UpdateProperties(0),
            },
            0,
        ),
        Err(BossProjectionError::MissingBoss { id: 99 })
    );
}

#[test]
fn c3_boss_publisher_suppresses_equal_hidden_deltas_and_snapshots_on_show() {
    let mut publisher = BossPublisher::new(1, literal("boss"));
    assert_eq!(publisher.add_player(10).len(), 1);
    assert!(publisher.add_player(10).is_empty());
    assert!(publisher.set_progress(1.0).is_empty());
    assert_eq!(publisher.set_progress(f32::NAN).len(), 1);
    assert_eq!(publisher.set_progress(f32::NAN).len(), 1);
    assert_eq!(publisher.set_visible(false).len(), 1);
    assert!(publisher.set_properties(true, true, true).is_empty());
    assert_eq!(publisher.dirty_updates, 3);
    assert!(publisher.add_player(11).is_empty());
    let shown = publisher.set_visible(true);
    assert_eq!(shown.len(), 2);
    assert!(shown.iter().all(|message| matches!(
        message.packet.operation,
        BossOperation::Add { properties: 7, .. }
    )));
    let style = publisher.set_style(BossColor::Blue, BossOverlay::Notched6);
    assert_eq!(style.len(), 2);
    assert!(style.iter().all(|message| matches!(
        message.packet.operation,
        BossOperation::UpdateStyle {
            color: BossColor::Blue,
            overlay: BossOverlay::Notched6,
        }
    )));
    assert_eq!(publisher.remove_player(10).len(), 1);
}

#[test]
fn c3_waypoint_collection_updates_only_matching_content_and_keeps_track_icon() {
    let key = WaypointIdentifier::Uuid(7);
    let original = TrackedWaypoint {
        identifier: key.clone(),
        icon: icon(),
        location: WaypointLocation::Position { x: 1, y: 2, z: 3 },
    };
    let mut client = WaypointClientProjection::default();
    assert_eq!(
        client
            .apply(&WaypointPacket {
                operation: WaypointOperation::Track,
                waypoint: original.clone(),
            })
            .unwrap(),
        WaypointCollectionAction::Tracked { replaced: false }
    );
    let replacement_icon = WaypointIcon {
        style: id("custom:ignored"),
        color: Some(0xff00_ff00),
    };
    client
        .apply(&WaypointPacket {
            operation: WaypointOperation::Update,
            waypoint: TrackedWaypoint {
                identifier: key.clone(),
                icon: replacement_icon,
                location: WaypointLocation::Position { x: 4, y: 5, z: 6 },
            },
        })
        .unwrap();
    assert_eq!(client.waypoint(&key).unwrap().icon, icon());
    assert_eq!(
        client
            .apply(&WaypointPacket {
                operation: WaypointOperation::Update,
                waypoint: TrackedWaypoint {
                    identifier: key.clone(),
                    icon: icon(),
                    location: WaypointLocation::Chunk { x: 1, z: 2 },
                },
            })
            .unwrap(),
        WaypointCollectionAction::TypeMismatchWarned
    );
    assert_eq!(
        client.apply(&WaypointPacket {
            operation: WaypointOperation::Update,
            waypoint: TrackedWaypoint {
                identifier: WaypointIdentifier::String("missing".to_owned()),
                icon: icon(),
                location: WaypointLocation::Empty,
            },
        }),
        Err(WaypointProjectionError::MissingWaypoint)
    );
    assert_eq!(
        client
            .apply(&WaypointPacket {
                operation: WaypointOperation::Untrack,
                waypoint: original,
            })
            .unwrap(),
        WaypointCollectionAction::Untracked { existed: true }
    );
}

#[test]
fn c3_waypoint_projection_uses_near_uuid_eye_and_locked_location_conventions() {
    let key = WaypointIdentifier::Uuid(7);
    let mut client = WaypointClientProjection::default();
    client.track_entity(
        7,
        TrackedEntityEye {
            block_position: [2, 64, 1],
            eye_position: [2.0, 65.62, 1.0],
        },
    );
    client
        .apply(&WaypointPacket {
            operation: WaypointOperation::Track,
            waypoint: TrackedWaypoint {
                identifier: key.clone(),
                icon: icon(),
                location: WaypointLocation::Position { x: 1, y: 64, z: 1 },
            },
        })
        .unwrap();
    let viewer = WaypointViewer {
        camera_position: [0.0, 64.0, 0.0],
        block_position: [0, 64, 0],
        yaw_degrees: 0.0,
    };
    assert_eq!(
        client.project_marker(&key, viewer).unwrap().point,
        Some([2.0, 65.62, 1.0])
    );
    client
        .apply(&WaypointPacket {
            operation: WaypointOperation::Track,
            waypoint: TrackedWaypoint {
                identifier: key.clone(),
                icon: icon(),
                location: WaypointLocation::Chunk { x: 1, z: -1 },
            },
        })
        .unwrap();
    let chunk = client.project_marker(&key, viewer).unwrap();
    assert_eq!(chunk.point, Some([24.5, 64.0, -7.5]));
    client
        .apply(&WaypointPacket {
            operation: WaypointOperation::Track,
            waypoint: TrackedWaypoint {
                identifier: key.clone(),
                icon: icon(),
                location: WaypointLocation::Empty,
            },
        })
        .unwrap();
    let empty = client.project_marker(&key, viewer).unwrap();
    assert!(empty.yaw_difference.is_nan() && empty.distance_squared.is_infinite());

    let near_key = WaypointIdentifier::String("near".to_owned());
    client
        .apply(&WaypointPacket {
            operation: WaypointOperation::Track,
            waypoint: TrackedWaypoint {
                identifier: near_key.clone(),
                icon: icon(),
                location: WaypointLocation::Position { x: 1, y: 64, z: 0 },
            },
        })
        .unwrap();
    let sorted = client.markers_by_descending_distance(viewer);
    assert_eq!(sorted[0].0, &key);
    assert_eq!(sorted[1].0, &near_key);
}

#[test]
fn c3_waypoint_publication_locks_admission_representation_and_transition_thresholds() {
    let source = WaypointSource {
        uuid: 1,
        position: [0.0, 64.0, 0.0],
        block_position: [0, 64, 0],
        chunk: [0, 0],
        spectator: false,
        first_tick: false,
        transmit_range: 1_000.0,
        icon: icon(),
    };
    let receiver = WaypointReceiver {
        uuid: 2,
        position: [332.0, 64.0, 0.0],
        spectator: false,
        riding_source: false,
        receive_range: 1_000.0,
        source_chunk_visible: false,
    };
    assert!(
        select_waypoint(
            &source,
            WaypointReceiver {
                uuid: source.uuid,
                ..receiver
            },
            true,
        )
        .is_none()
    );
    assert!(
        select_waypoint(
            &WaypointSource {
                first_tick: true,
                ..source.clone()
            },
            receiver,
            true,
        )
        .is_none()
    );
    assert!(select_waypoint(&source, receiver, false).is_none());
    assert!(
        select_waypoint(
            &source,
            WaypointReceiver {
                position: [f64::NAN, 64.0, 0.0],
                ..receiver
            },
            true,
        )
        .is_none()
    );
    assert!(matches!(
        select_waypoint(&source, receiver, true).unwrap().location,
        WaypointLocation::Chunk { .. }
    ));
    assert!(matches!(
        select_waypoint(
            &source,
            WaypointReceiver {
                position: [332.001, 64.0, 0.0],
                ..receiver
            },
            true,
        )
        .unwrap()
        .location,
        WaypointLocation::Azimuth { .. }
    ));
    assert!(
        select_waypoint(
            &source,
            WaypointReceiver {
                position: [1_000.0, 64.0, 0.0],
                ..receiver
            },
            true,
        )
        .is_none()
    );
    assert!(
        select_waypoint(
            &source,
            WaypointReceiver {
                position: [1_000.0, 64.0, 0.0],
                spectator: true,
                ..receiver
            },
            true,
        )
        .is_some()
    );
    let spectator_source = WaypointSource {
        spectator: true,
        ..source.clone()
    };
    assert!(select_waypoint(&spectator_source, receiver, true).is_none());
    assert!(
        select_waypoint(
            &source,
            WaypointReceiver {
                riding_source: true,
                ..receiver
            },
            true,
        )
        .is_none()
    );
    assert!(
        select_waypoint(
            &spectator_source,
            WaypointReceiver {
                spectator: true,
                riding_source: true,
                ..receiver
            },
            true,
        )
        .is_some()
    );
    assert_eq!(
        resolved_icon(None, Some(0x12_3456), false),
        Some(0xff12_3456)
    );
    assert_eq!(resolved_icon(None, Some(0), true), Some(0xff30_3030));
    assert_eq!(
        resolved_icon(Some(0xffab_cdef), Some(0), true),
        Some(0xffab_cdef)
    );

    let previous = TrackedWaypoint {
        identifier: WaypointIdentifier::Uuid(1),
        icon: icon(),
        location: WaypointLocation::Position { x: 0, y: 64, z: 0 },
    };
    let adjacent = TrackedWaypoint {
        location: WaypointLocation::Position { x: 1, y: 64, z: 0 },
        ..previous.clone()
    };
    assert_eq!(
        waypoint_transition(
            Some(&previous),
            Some(adjacent),
            false,
            id("minecraft:default")
        )
        .unwrap()
        .operation,
        WaypointOperation::Update
    );
    let jumped = TrackedWaypoint {
        location: WaypointLocation::Position { x: 2, y: 64, z: 0 },
        ..previous.clone()
    };
    assert_eq!(
        waypoint_transition(
            Some(&previous),
            Some(jumped),
            false,
            id("minecraft:default")
        )
        .unwrap()
        .operation,
        WaypointOperation::Track
    );
    let nan_azimuth = TrackedWaypoint {
        location: WaypointLocation::Azimuth { angle: f32::NAN },
        ..previous.clone()
    };
    assert!(
        waypoint_transition(
            Some(&nan_azimuth),
            Some(nan_azimuth.clone()),
            false,
            id("minecraft:default"),
        )
        .is_none()
    );
    let chunk = TrackedWaypoint {
        location: WaypointLocation::Chunk { x: 0, z: 0 },
        ..previous.clone()
    };
    let adjacent_chunk = TrackedWaypoint {
        location: WaypointLocation::Chunk { x: 1, z: 1 },
        ..previous.clone()
    };
    assert_eq!(
        waypoint_transition(
            Some(&chunk),
            Some(adjacent_chunk.clone()),
            false,
            id("minecraft:default"),
        )
        .unwrap()
        .operation,
        WaypointOperation::Update
    );
    assert_eq!(
        waypoint_transition(
            Some(&chunk),
            Some(adjacent_chunk),
            true,
            id("minecraft:default"),
        )
        .unwrap()
        .operation,
        WaypointOperation::Track
    );
    let azimuth = TrackedWaypoint {
        location: WaypointLocation::Azimuth { angle: 0.0 },
        ..previous.clone()
    };
    let below_threshold = TrackedWaypoint {
        location: WaypointLocation::Azimuth {
            angle: 0.008_726_646,
        },
        ..previous.clone()
    };
    assert!(
        waypoint_transition(
            Some(&azimuth),
            Some(below_threshold),
            false,
            id("minecraft:default"),
        )
        .is_none()
    );
    let above_threshold = TrackedWaypoint {
        location: WaypointLocation::Azimuth { angle: 0.009 },
        ..previous.clone()
    };
    assert_eq!(
        waypoint_transition(
            Some(&azimuth),
            Some(above_threshold),
            false,
            id("minecraft:default"),
        )
        .unwrap()
        .operation,
        WaypointOperation::Update
    );
    assert_eq!(
        waypoint_transition(
            Some(&azimuth),
            Some(previous.clone()),
            false,
            id("minecraft:default"),
        )
        .unwrap()
        .operation,
        WaypointOperation::Track
    );
    assert_eq!(
        waypoint_transition(None, Some(previous.clone()), false, id("minecraft:default"),)
            .unwrap()
            .operation,
        WaypointOperation::Track
    );
    let disconnect =
        waypoint_transition(Some(&previous), None, false, id("minecraft:default")).unwrap();
    assert_eq!(disconnect.operation, WaypointOperation::Untrack);
    assert!(matches!(
        disconnect.waypoint.location,
        WaypointLocation::Empty
    ));
}

fn encode_var_i32(value: i32) -> Vec<u8> {
    let mut remaining = value as u32;
    let mut encoded = Vec::new();
    loop {
        if remaining & !0x7f == 0 {
            encoded.push(remaining as u8);
            return encoded;
        }
        encoded.push(((remaining & 0x7f) | 0x80) as u8);
        remaining >>= 7;
    }
}
