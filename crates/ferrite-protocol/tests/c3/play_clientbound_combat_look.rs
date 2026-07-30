use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::combat_look::packet::{
    EntityAnchor, LookEntity, LookPosition, PlayerCombatEnd, PlayerCombatKill, PlayerLookAt,
};
use ferrite_protocol::java_26_2::play::clientbound::combat_look::projection::{
    CombatLookAction, CombatLookClientProjection, TrackedEntityPosition,
};
use ferrite_protocol::java_26_2::play::clientbound::combat_look::publication::{
    publish_combat_end, publish_combat_enter, publish_coordinate_look, publish_death,
    publish_entity_look,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn literal(value: &str) -> TextComponentNbt {
    TextComponentNbt::literal(value).unwrap()
}

fn position(x: f64, y: f64, z: f64, eye_height: f32, local: bool) -> TrackedEntityPosition {
    TrackedEntityPosition {
        feet: LookPosition { x, y, z },
        eye_height,
        current_local_player: local,
    }
}

fn projection(show_death_screen: bool) -> CombatLookClientProjection {
    CombatLookClientProjection::new(
        7,
        position(0.0, 10.0, 0.0, 1.625, true),
        show_death_screen,
        true,
        true,
    )
}

#[test]
fn c3_gold_clientbound_combat_look_locks_all_four_packet_bodies() {
    let registries = PlayRegistries::default();
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::PlayerCombatEnd(PlayerCombatEnd { duration: -1 }),
            &registries,
        )
        .unwrap(),
        vec![0x42, 0xff, 0xff, 0xff, 0xff, 0x0f]
    );
    assert_eq!(
        encode_packet(&PlayClientboundPacket::PlayerCombatEnter, &registries).unwrap(),
        vec![0x43]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::PlayerCombatKill(PlayerCombatKill {
                player_entity_id: -1,
                message: literal("d"),
            }),
            &registries,
        )
        .unwrap(),
        vec![0x44, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x08, 0x00, 0x01, b'd',]
    );

    let look = PlayClientboundPacket::PlayerLookAt(PlayerLookAt {
        from_anchor: EntityAnchor::Eyes,
        fallback: LookPosition {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        entity: Some(LookEntity {
            entity_id: -1,
            anchor: EntityAnchor::Feet,
        }),
    });
    let mut expected = vec![0x47, 0x01];
    expected.extend_from_slice(&[0; 24]);
    expected.extend_from_slice(&[0x01, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x00]);
    assert_eq!(encode_packet(&look, &registries).unwrap(), expected);
}

#[test]
fn c3_combat_look_codecs_preserve_signed_and_ieee_domains() {
    let registries = PlayRegistries::default();
    for packet in [
        PlayClientboundPacket::PlayerCombatEnd(PlayerCombatEnd { duration: i32::MIN }),
        PlayClientboundPacket::PlayerCombatKill(PlayerCombatKill {
            player_entity_id: i32::MAX,
            message: literal("death"),
        }),
    ] {
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
    let look = PlayClientboundPacket::PlayerLookAt(PlayerLookAt {
        from_anchor: EntityAnchor::Feet,
        fallback: LookPosition {
            x: f64::from_bits(1),
            y: f64::INFINITY,
            z: f64::from_bits(u64::MAX),
        },
        entity: Some(LookEntity {
            entity_id: i32::MIN,
            anchor: EntityAnchor::Eyes,
        }),
    });
    let encoded = encode_packet(&look, &registries).unwrap();
    let PlayClientboundPacket::PlayerLookAt(decoded) =
        decode_packet(&encoded, context(&registries)).unwrap()
    else {
        panic!("expected look packet");
    };
    assert_eq!(decoded.fallback.x.to_bits(), 1);
    assert_eq!(decoded.fallback.y.to_bits(), f64::INFINITY.to_bits());
    assert_eq!(decoded.fallback.z.to_bits(), u64::MAX);
    assert_eq!(
        decoded.entity,
        Some(LookEntity {
            entity_id: i32::MIN,
            anchor: EntityAnchor::Eyes,
        })
    );
}

#[test]
fn c3_look_boolean_normalizes_and_invalid_anchors_fail_before_use() {
    let registries = PlayRegistries::default();
    let mut noncanonical = vec![0x47, 0x00];
    noncanonical.extend_from_slice(&[0; 24]);
    noncanonical.extend_from_slice(&[0x7f, 0x01, 0x01]);
    let decoded = decode_packet(&noncanonical, context(&registries)).unwrap();
    let canonical = encode_packet(&decoded, &registries).unwrap();
    assert_eq!(canonical[26], 1);

    assert!(matches!(
        decode_packet(&[0x47, 0x02], context(&registries)),
        Err(PlayClientboundCodecError::CombatLook(_))
    ));
    let mut invalid_target = vec![0x47, 0x00];
    invalid_target.extend_from_slice(&[0; 24]);
    invalid_target.extend_from_slice(&[0x01, 0x00, 0x02]);
    assert!(matches!(
        decode_packet(&invalid_target, context(&registries)),
        Err(PlayClientboundCodecError::CombatLook(_))
    ));
}

#[test]
fn c3_combat_look_malformed_and_residual_bodies_fail_closed() {
    let registries = PlayRegistries::default();
    for body in [
        vec![0x42],
        vec![0x43, 0x00],
        vec![0x44, 0x00, 0x00],
        vec![0x47, 0x00],
    ] {
        assert!(decode_packet(&body, context(&registries)).is_err());
    }
}

#[test]
fn c3_combat_enter_and_end_are_transport_visible_but_semantically_inert() {
    let mut client = projection(true);
    assert_eq!(
        client.apply(&PlayClientboundPacket::PlayerCombatEnter),
        CombatLookAction::Ignored
    );
    assert_eq!(
        client.apply(&PlayClientboundPacket::PlayerCombatEnd(PlayerCombatEnd {
            duration: -99,
        })),
        CombatLookAction::Ignored
    );
    assert_eq!(client.rotations().yaw, 0.0);
    assert!(client.death_screen().is_none());
}

#[test]
fn c3_death_screen_requires_the_current_local_player_object() {
    let mut client = projection(true);
    let packet = PlayClientboundPacket::PlayerCombatKill(PlayerCombatKill {
        player_entity_id: 7,
        message: literal("first"),
    });
    assert!(matches!(
        client.apply(&packet),
        CombatLookAction::DeathScreenInstalled(_)
    ));
    assert_eq!(client.death_screen().unwrap().message, literal("first"));
    assert!(client.death_screen().unwrap().hardcore);

    client.track_entity(7, position(0.0, 0.0, 0.0, 0.0, false));
    assert_eq!(client.apply(&packet), CombatLookAction::Ignored);
    let wrong = PlayClientboundPacket::PlayerCombatKill(PlayerCombatKill {
        player_entity_id: 8,
        message: literal("wrong"),
    });
    client.track_entity(8, position(0.0, 0.0, 0.0, 0.0, true));
    assert_eq!(client.apply(&wrong), CombatLookAction::Ignored);
}

#[test]
fn c3_hidden_death_screen_repeats_respawn_and_key_reset_without_message_use() {
    let mut client = projection(false);
    let packet = PlayClientboundPacket::PlayerCombatKill(PlayerCombatKill {
        player_entity_id: 7,
        message: literal("unused"),
    });
    assert_eq!(
        client.apply(&packet),
        CombatLookAction::RespawnRequestedAndToggleKeysReset
    );
    assert_eq!(
        client.apply(&packet),
        CombatLookAction::RespawnRequestedAndToggleKeysReset
    );
    assert_eq!(
        (client.respawn_requests(), client.toggle_key_resets()),
        (2, 2)
    );
    assert!(client.death_screen().is_none());
}

#[test]
fn c3_coordinate_look_uses_selected_player_origin_and_java_angles() {
    let mut client = projection(true);
    let packet = publish_coordinate_look(
        EntityAnchor::Feet,
        LookPosition {
            x: 10.0,
            y: 10.0,
            z: 0.0,
        },
    );
    let CombatLookAction::Rotated(rotations) = client.apply(&packet) else {
        panic!("expected rotation");
    };
    assert_eq!((rotations.pitch, rotations.yaw), (0.0, -90.0));
    assert_eq!(rotations.head_yaw, rotations.yaw);
    assert_eq!(rotations.previous_pitch, rotations.pitch);
    assert_eq!(rotations.previous_yaw, rotations.yaw);
    assert_eq!(rotations.body_yaw, rotations.yaw);

    let eye = publish_coordinate_look(
        EntityAnchor::Eyes,
        LookPosition {
            x: 0.0,
            y: 10.0,
            z: 10.0,
        },
    );
    let CombatLookAction::Rotated(rotations) = client.apply(&eye) else {
        panic!("expected rotation");
    };
    assert!(rotations.pitch > 9.0 && rotations.pitch < 10.0);
    assert!(rotations.yaw.abs() < 0.000_01);
}

#[test]
fn c3_entity_look_resolves_handler_time_position_or_packet_fallback_once() {
    let mut client = projection(true);
    client.track_entity(9, position(0.0, 10.0, 10.0, 2.0, false));
    let packet = publish_entity_look(
        EntityAnchor::Feet,
        9,
        EntityAnchor::Eyes,
        LookPosition {
            x: 10.0,
            y: 10.0,
            z: 0.0,
        },
    );
    let CombatLookAction::Rotated(tracked) = client.apply(&packet) else {
        panic!("expected tracked rotation");
    };
    assert!(tracked.yaw.abs() < 0.000_01);
    assert!(tracked.pitch < 0.0);

    client.remove_entity(9);
    let CombatLookAction::Rotated(fallback) = client.apply(&packet) else {
        panic!("expected fallback rotation");
    };
    assert_eq!((fallback.pitch, fallback.yaw), (0.0, -90.0));
}

#[test]
fn c3_nonfinite_and_coincident_look_targets_are_not_rejected() {
    let mut client = projection(true);
    let coincident = publish_coordinate_look(
        EntityAnchor::Feet,
        LookPosition {
            x: 0.0,
            y: 10.0,
            z: 0.0,
        },
    );
    let CombatLookAction::Rotated(coincident) = client.apply(&coincident) else {
        panic!("expected rotation");
    };
    assert_eq!(coincident.pitch.to_bits(), (-0.0_f32).to_bits());

    let nonfinite = publish_coordinate_look(
        EntityAnchor::Feet,
        LookPosition {
            x: f64::INFINITY,
            y: f64::NAN,
            z: 0.0,
        },
    );
    let CombatLookAction::Rotated(nonfinite) = client.apply(&nonfinite) else {
        panic!("expected rotation");
    };
    assert!(nonfinite.pitch.is_nan());
}

#[test]
fn c3_publication_preserves_direct_tokenless_order_and_death_fallback() {
    assert_eq!(
        publish_combat_enter(),
        PlayClientboundPacket::PlayerCombatEnter
    );
    assert_eq!(
        publish_combat_end(i32::MIN),
        PlayClientboundPacket::PlayerCombatEnd(PlayerCombatEnd { duration: i32::MIN })
    );
    let death = publish_death(7, literal("primary"), literal("fallback"), true);
    assert!(matches!(
        death.primary,
        PlayClientboundPacket::PlayerCombatKill(_)
    ));
    assert!(death.fallback.is_some() && death.broadcast_public_message);

    let hidden = publish_death(7, literal("unused"), literal("unused fallback"), false);
    assert!(hidden.fallback.is_none());
    assert!(!hidden.broadcast_public_message);
    let PlayClientboundPacket::PlayerCombatKill(packet) = hidden.primary else {
        panic!("expected death packet");
    };
    assert_eq!(packet.message, literal(""));
}

#[test]
fn c3_combat_look_end_to_end_decodes_into_handler_time_projection() {
    let registries = PlayRegistries::default();
    let packet = publish_entity_look(
        EntityAnchor::Eyes,
        99,
        EntityAnchor::Feet,
        LookPosition {
            x: -10.0,
            y: 10.0,
            z: 0.0,
        },
    );
    let encoded = encode_packet(&packet, &registries).unwrap();
    let decoded = decode_packet(&encoded, context(&registries)).unwrap();
    let mut client = projection(true);
    let CombatLookAction::Rotated(rotations) = client.apply(&decoded) else {
        panic!("expected fallback rotation");
    };
    assert!((rotations.yaw - 90.0).abs() < 0.000_02);
}
