use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    BorderProjection, BorderSize, PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::world_border::packet::{
    SetBorderCenter, SetBorderLerpSize, SetBorderSize, SetBorderWarningDelay,
    SetBorderWarningDistance,
};
use ferrite_protocol::java_26_2::play::clientbound::world_border::publication::{
    BorderAuthoritativeEvent, BorderDelta, BorderViewer, publish_event,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::identifier::Identifier;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn border() -> BorderProjection {
    BorderProjection {
        center_x: 1.0,
        center_z: 2.0,
        size: BorderSize::Immediate(100.0),
        absolute_maximum: 29_999_984,
        warning_blocks: 5,
        warning_time: 15,
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
fn c3_gold_clientbound_world_border_locks_all_five_zero_bodies() {
    let registries = PlayRegistries::default();
    let mut center = vec![88];
    center.extend([0; 16]);
    let mut lerp = vec![89];
    lerp.extend([0; 17]);
    let mut size = vec![90];
    size.extend([0; 8]);
    let cases = [
        (
            PlayClientboundPacket::SetBorderCenter(SetBorderCenter {
                center_x: 0.0,
                center_z: 0.0,
            }),
            center,
        ),
        (
            PlayClientboundPacket::SetBorderLerpSize(SetBorderLerpSize {
                old_size: 0.0,
                new_size: 0.0,
                duration_millis: 0,
            }),
            lerp,
        ),
        (
            PlayClientboundPacket::SetBorderSize(SetBorderSize { size: 0.0 }),
            size,
        ),
        (
            PlayClientboundPacket::SetBorderWarningDelay(SetBorderWarningDelay { warning_time: 0 }),
            vec![91, 0],
        ),
        (
            PlayClientboundPacket::SetBorderWarningDistance(SetBorderWarningDistance {
                warning_blocks: 0,
            }),
            vec![92, 0],
        ),
    ];
    for (packet, expected) in cases {
        assert_eq!(encode_packet(&packet, &registries).unwrap(), expected);
        assert_eq!(
            decode_packet(&expected, context(&registries)).unwrap(),
            packet
        );
    }
}

#[test]
fn c3_world_border_codecs_preserve_ieee_and_signed_varint_domains() {
    assert_roundtrip(PlayClientboundPacket::SetBorderCenter(SetBorderCenter {
        center_x: f64::from_bits(1),
        center_z: -0.0,
    }));
    assert_roundtrip(PlayClientboundPacket::SetBorderLerpSize(
        SetBorderLerpSize {
            old_size: f64::INFINITY,
            new_size: f64::NEG_INFINITY,
            duration_millis: i64::MIN,
        },
    ));
    let nan = f64::from_bits(0x7ff8_0000_0000_0001);
    let registries = PlayRegistries::default();
    let encoded = encode_packet(
        &PlayClientboundPacket::SetBorderSize(SetBorderSize { size: nan }),
        &registries,
    )
    .unwrap();
    let PlayClientboundPacket::SetBorderSize(decoded) =
        decode_packet(&encoded, context(&registries)).unwrap()
    else {
        panic!("size packet identity must roundtrip");
    };
    assert_eq!(decoded.size.to_bits(), nan.to_bits());
    assert_roundtrip(PlayClientboundPacket::SetBorderWarningDelay(
        SetBorderWarningDelay {
            warning_time: i32::MIN,
        },
    ));
    assert_roundtrip(PlayClientboundPacket::SetBorderWarningDistance(
        SetBorderWarningDistance {
            warning_blocks: i32::MAX,
        },
    ));
}

#[test]
fn c3_world_border_codecs_fault_truncation_overlong_varints_and_residual_bytes() {
    let registries = PlayRegistries::default();
    assert!(decode_packet(&[88], context(&registries)).is_err());
    assert!(decode_packet(&[90, 0, 0, 0], context(&registries)).is_err());

    let mut overlong_duration = vec![89];
    overlong_duration.extend([0; 16]);
    overlong_duration.extend([0x80; 10]);
    overlong_duration.push(0);
    assert!(decode_packet(&overlong_duration, context(&registries)).is_err());
    assert!(decode_packet(&[91, 0x80, 0x80, 0x80, 0x80, 0x80, 0], context(&registries)).is_err());

    let mut trailing = encode_packet(
        &PlayClientboundPacket::SetBorderSize(SetBorderSize { size: 1.0 }),
        &registries,
    )
    .unwrap();
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn c3_border_center_replaces_coordinates_without_touching_extent_or_warnings() {
    let mut projection = border();
    let old_size = projection.size;
    projection.apply_center(SetBorderCenter {
        center_x: f64::NAN,
        center_z: f64::INFINITY,
    });
    assert!(projection.center_x.is_nan());
    assert_eq!(projection.center_z, f64::INFINITY);
    assert_eq!(projection.size, old_size);
    assert_eq!(
        (projection.warning_blocks, projection.warning_time),
        (5, 15)
    );
}

#[test]
fn c3_border_immediate_size_replaces_static_or_moving_extent() {
    let mut projection = border();
    projection.apply_lerp(
        SetBorderLerpSize {
            old_size: 100.0,
            new_size: 200.0,
            duration_millis: 40,
        },
        7,
    );
    projection.apply_size(SetBorderSize { size: -0.0 });
    let BorderSize::Immediate(size) = projection.size else {
        panic!("size delta must replace motion");
    };
    assert_eq!(size.to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn c3_border_lerp_equal_endpoints_select_static_new_with_java_double_equality() {
    let mut projection = border();
    projection.apply_lerp(
        SetBorderLerpSize {
            old_size: -0.0,
            new_size: 0.0,
            duration_millis: i64::MIN,
        },
        99,
    );
    let BorderSize::Immediate(size) = projection.size else {
        panic!("equal endpoints must select a static extent");
    };
    assert_eq!(size.to_bits(), 0.0_f64.to_bits());
}

#[test]
fn c3_border_lerp_retains_nonpositive_duration_nonfinite_values_and_receive_anchor() {
    let mut projection = border();
    projection.apply_lerp(
        SetBorderLerpSize {
            old_size: f64::NAN,
            new_size: f64::INFINITY,
            duration_millis: -7,
        },
        i64::MAX,
    );
    let BorderSize::Lerp {
        old_size,
        new_size,
        duration_millis,
        begin_game_time,
    } = projection.size
    else {
        panic!("NaN endpoints compare unequal");
    };
    assert!(old_size.is_nan());
    assert_eq!(new_size, f64::INFINITY);
    assert_eq!(duration_millis, -7);
    assert_eq!(begin_game_time, i64::MAX);
}

#[test]
fn c3_border_warning_fields_replace_independently_without_geometry_changes() {
    let mut projection = border();
    let old_size = projection.size;
    projection.apply_warning_delay(SetBorderWarningDelay {
        warning_time: i32::MIN,
    });
    projection.apply_warning_distance(SetBorderWarningDistance {
        warning_blocks: i32::MAX,
    });
    assert_eq!(projection.warning_time, i32::MIN);
    assert_eq!(projection.warning_blocks, i32::MAX);
    assert_eq!(projection.size, old_size);
    assert_eq!((projection.center_x, projection.center_z), (1.0, 2.0));
}

#[test]
fn c3_border_delta_publication_is_dimension_scoped_ordered_and_not_equality_suppressed() {
    let overworld = id("minecraft:overworld");
    let viewers = [
        BorderViewer {
            player_id: 3,
            dimension: overworld.clone(),
        },
        BorderViewer {
            player_id: 2,
            dimension: id("minecraft:the_nether"),
        },
        BorderViewer {
            player_id: 1,
            dimension: overworld.clone(),
        },
    ];
    let deliveries = publish_event(
        BorderAuthoritativeEvent::Size { size: 100.0 },
        &overworld,
        &viewers,
    );
    assert_eq!(
        deliveries
            .iter()
            .map(|delivery| delivery.recipient)
            .collect::<Vec<_>>(),
        [3, 1]
    );
    assert!(
        deliveries.iter().all(|delivery| {
            delivery.packet == BorderDelta::Size(SetBorderSize { size: 100.0 })
        })
    );
}

#[test]
fn c3_border_damage_safe_zone_and_moving_ticks_publish_no_delta() {
    let dimension = id("minecraft:overworld");
    let viewer = BorderViewer {
        player_id: 1,
        dimension: dimension.clone(),
    };
    for event in [
        BorderAuthoritativeEvent::DamagePerBlock,
        BorderAuthoritativeEvent::SafeZone,
        BorderAuthoritativeEvent::MovingTick,
    ] {
        assert!(publish_event(event, &dimension, std::slice::from_ref(&viewer)).is_empty());
    }
}

#[test]
fn c3_border_delta_decode_and_projection_converge_end_to_end() {
    let registries = PlayRegistries::default();
    let packet = PlayClientboundPacket::SetBorderLerpSize(SetBorderLerpSize {
        old_size: 12.0,
        new_size: 34.0,
        duration_millis: 56,
    });
    let encoded = encode_packet(&packet, &registries).unwrap();
    let decoded = decode_packet(&encoded, context(&registries)).unwrap();
    let PlayClientboundPacket::SetBorderLerpSize(delta) = decoded else {
        panic!("decoded packet must keep its identity");
    };
    let mut projection = border();
    projection.apply_lerp(delta, 78);
    assert_eq!(
        projection.size,
        BorderSize::Lerp {
            old_size: 12.0,
            new_size: 34.0,
            duration_millis: 56,
            begin_game_time: 78,
        }
    );
}

#[test]
fn c3_world_border_deltas_require_an_installed_play_level() {
    let packets = [
        PlayClientboundPacket::SetBorderCenter(SetBorderCenter {
            center_x: 0.0,
            center_z: 0.0,
        }),
        PlayClientboundPacket::SetBorderLerpSize(SetBorderLerpSize {
            old_size: 0.0,
            new_size: 1.0,
            duration_millis: 0,
        }),
        PlayClientboundPacket::SetBorderSize(SetBorderSize { size: 0.0 }),
        PlayClientboundPacket::SetBorderWarningDelay(SetBorderWarningDelay { warning_time: 0 }),
        PlayClientboundPacket::SetBorderWarningDistance(SetBorderWarningDistance {
            warning_blocks: 0,
        }),
    ];
    for packet in packets {
        assert_eq!(
            PlayEntryProjection::default().apply(packet),
            Err(PlayProjectionError::LevelNotInstalled)
        );
    }
}
