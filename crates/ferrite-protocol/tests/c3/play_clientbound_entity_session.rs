use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_session::packet::{
    Animate, DamageEvent, HurtAnimation, SetCamera, TakeItemEntity,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_session::projection::{
    EntitySessionAction, EntitySessionClientProjection, EntitySessionProjectionError,
    LevelWaitReason, ProjectedAttribute, RespawnPlayerProjection, RespawnSessionProjection,
    SessionEntityKind, SessionEntityProjection,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_session::publication::{
    AnimationPublicationAudience, CAMERA_PUBLICATION_ORDER, CROSS_DIMENSION_ORDER,
    CameraPublicationStep, CrossDimensionPublicationStep, DEATH_RESPAWN_ORDER, PICKUP_AUDIENCE,
    RespawnPublicationStep, animation_audience, publish_damage_event,
    publish_hurt_animation_to_damaged_player,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, GameMode, PlayClientboundPacket, Vector3,
};
use ferrite_protocol::java_26_2::play::clientbound::session::Respawn;
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{DAMAGE_TYPE, DIMENSION_TYPE, PlayRegistries};
use ferrite_protocol::java_26_2::value::identifier::Identifier;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(DAMAGE_TYPE),
        vec![id("minecraft:generic"), id("minecraft:in_fire")],
    );
    registries.insert(
        id(DIMENSION_TYPE),
        vec![id("minecraft:overworld"), id("minecraft:the_nether")],
    );
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn spawn(dimension: &str) -> CommonSpawnInfo {
    CommonSpawnInfo {
        dimension_type: if dimension == "minecraft:the_nether" {
            id("minecraft:the_nether")
        } else {
            id("minecraft:overworld")
        },
        dimension: id(dimension),
        obfuscated_seed: 0,
        game_mode: GameMode::Survival,
        previous_game_mode: None,
        is_debug: false,
        is_flat: false,
        last_death: None,
        portal_cooldown: 0,
        sea_level: 63,
    }
}

fn session_entity(kind: SessionEntityKind) -> SessionEntityProjection {
    SessionEntityProjection::new(kind)
}

fn player_projection() -> RespawnPlayerProjection {
    RespawnPlayerProjection {
        entity_id: 1,
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
        yaw: 40.0,
        pitch: 50.0,
        last_input: 0xa5,
        sprinting: true,
        nondefault_entity_data: BTreeMap::from([(1, 7)]),
        attributes: BTreeMap::from([(
            id("minecraft:generic.max_health"),
            ProjectedAttribute {
                base: 20.0,
                value: 30.0,
                modifiers: vec![id("test:boost")],
            },
        )]),
        stats_generation: 8,
        recipe_book_generation: 9,
    }
}

fn respawn_session() -> RespawnSessionProjection {
    RespawnSessionProjection {
        spawn: spawn("minecraft:overworld"),
        player: player_projection(),
        camera_entity_id: Some(99),
        container_open: true,
        client_loaded: true,
        level_generation: 4,
        debug_subscriptions_installed: true,
        wait_reason: None,
    }
}

fn assert_roundtrip(packet: PlayClientboundPacket, registries: &PlayRegistries) {
    let encoded = encode_packet(&packet, registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(registries)).unwrap(),
        packet
    );
}

#[test]
fn c3_gold_entity_session_locks_all_six_packet_bodies() {
    let registries = registries();
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::Animate(Animate {
                entity_id: 0,
                action: 0,
            }),
            &registries,
        )
        .unwrap(),
        [0x02, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::DamageEvent(DamageEvent {
                entity_id: 0,
                damage_type: id("minecraft:generic"),
                cause_entity_id: -1,
                direct_entity_id: -1,
                source_position: None,
            }),
            &registries,
        )
        .unwrap(),
        [0x19, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::HurtAnimation(HurtAnimation {
                entity_id: 0,
                yaw: 0.0,
            }),
            &registries,
        )
        .unwrap(),
        [0x2a, 0, 0, 0, 0, 0]
    );

    let respawn = PlayClientboundPacket::Respawn(Respawn {
        spawn: spawn("minecraft:overworld"),
        data_to_keep: 0,
    });
    let mut expected = vec![0x52, 0, 0x13];
    expected.extend_from_slice(b"minecraft:overworld");
    expected.extend_from_slice(&[0; 8]);
    expected.extend_from_slice(&[0, 0xff, 0, 0, 0, 0, 0x3f, 0]);
    assert_eq!(encode_packet(&respawn, &registries).unwrap(), expected);

    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::SetCamera(SetCamera { entity_id: 0 }),
            &registries,
        )
        .unwrap(),
        [0x5d, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::TakeItemEntity(TakeItemEntity {
                source_entity_id: 0,
                collector_entity_id: 0,
                amount: 0,
            }),
            &registries,
        )
        .unwrap(),
        [0x7c, 0, 0, 0]
    );
}

#[test]
fn c3_entity_session_codecs_preserve_wrapping_ids_signed_values_and_ieee_position() {
    let registries = registries();
    for packet in [
        PlayClientboundPacket::Animate(Animate {
            entity_id: i32::MIN,
            action: u8::MAX,
        }),
        PlayClientboundPacket::HurtAnimation(HurtAnimation {
            entity_id: i32::MAX,
            yaw: f32::INFINITY,
        }),
        PlayClientboundPacket::SetCamera(SetCamera {
            entity_id: i32::MIN,
        }),
        PlayClientboundPacket::TakeItemEntity(TakeItemEntity {
            source_entity_id: i32::MIN,
            collector_entity_id: i32::MAX,
            amount: i32::MIN,
        }),
    ] {
        assert_roundtrip(packet, &registries);
    }
    let damage = PlayClientboundPacket::DamageEvent(DamageEvent {
        entity_id: -1,
        damage_type: id("minecraft:in_fire"),
        cause_entity_id: i32::MAX,
        direct_entity_id: i32::MIN,
        source_position: Some(Vector3 {
            x: f64::from_bits(1),
            y: f64::INFINITY,
            z: f64::NEG_INFINITY,
        }),
    });
    assert_roundtrip(damage, &registries);

    let mut noncanonical = vec![0x19, 0, 0, 0, 0, 0x7f];
    noncanonical.extend_from_slice(&[0; 24]);
    let decoded = decode_packet(&noncanonical, context(&registries)).unwrap();
    assert_eq!(encode_packet(&decoded, &registries).unwrap()[5], 1);
}

#[test]
fn c3_entity_session_rejects_unknown_holders_malformed_and_residual_bodies() {
    let registries = registries();
    let unknown = PlayClientboundPacket::DamageEvent(DamageEvent {
        entity_id: 1,
        damage_type: id("minecraft:unknown"),
        cause_entity_id: -1,
        direct_entity_id: -1,
        source_position: None,
    });
    assert!(matches!(
        encode_packet(&unknown, &registries),
        Err(PlayClientboundCodecError::EntitySession(_))
    ));
    for body in [
        vec![0x02],
        vec![0x19, 0],
        vec![0x2a, 0],
        vec![0x52],
        vec![0x5d],
        vec![0x7c, 0],
        vec![0x02, 0, 0, 0],
    ] {
        assert!(decode_packet(&body, context(&registries)).is_err());
    }
}

#[test]
fn c3_animation_actions_apply_exact_casts_and_unknown_action_ignores() {
    let mut client = EntitySessionClientProjection::new(1);
    client.track_entity(1, session_entity(SessionEntityKind::Player));
    client.track_entity(2, session_entity(SessionEntityKind::Living));
    client.track_entity(3, session_entity(SessionEntityKind::Generic));

    for (entity_id, action) in [(1, 0), (2, 3), (1, 2), (3, 4), (3, 5)] {
        assert_eq!(
            client
                .apply(&PlayClientboundPacket::Animate(Animate {
                    entity_id,
                    action,
                }))
                .unwrap(),
            EntitySessionAction::Applied
        );
    }
    assert_eq!(client.entity(1).unwrap().main_hand_swings, 1);
    assert_eq!(client.entity(2).unwrap().off_hand_swings, 1);
    assert_eq!(client.entity(1).unwrap().wakes, 1);
    assert_eq!(client.entity(3).unwrap().critical_particles, 1);
    assert_eq!(client.entity(3).unwrap().enchanted_particles, 1);

    assert_eq!(
        client
            .apply(&PlayClientboundPacket::Animate(Animate {
                entity_id: 3,
                action: 99,
            }))
            .unwrap(),
        EntitySessionAction::Ignored
    );
    assert_eq!(
        client.apply(&PlayClientboundPacket::Animate(Animate {
            entity_id: 99,
            action: 0,
        })),
        Ok(EntitySessionAction::Ignored)
    );
    assert!(matches!(
        client.apply(&PlayClientboundPacket::Animate(Animate {
            entity_id: 3,
            action: 0,
        })),
        Err(EntitySessionProjectionError::WrongEntityType { .. })
    ));
    assert!(matches!(
        client.apply(&PlayClientboundPacket::Animate(Animate {
            entity_id: 2,
            action: 2,
        })),
        Err(EntitySessionProjectionError::WrongEntityType { .. })
    ));
}

#[test]
fn c3_damage_position_precedes_entity_lookup_and_living_state_updates() {
    let mut client = EntitySessionClientProjection::new(1);
    client.track_entity(1, session_entity(SessionEntityKind::Player));
    client.track_entity(2, session_entity(SessionEntityKind::Generic));
    client.track_entity(3, session_entity(SessionEntityKind::Living));
    client.set_game_time(123);

    let positioned = PlayClientboundPacket::DamageEvent(DamageEvent {
        entity_id: 1,
        damage_type: id("minecraft:generic"),
        cause_entity_id: 2,
        direct_entity_id: 3,
        source_position: Some(Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        }),
    });
    assert_eq!(
        client.apply(&positioned).unwrap(),
        EntitySessionAction::Applied
    );
    let target = client.entity(1).unwrap();
    assert_eq!(
        (
            target.walk_animation_speed,
            target.invulnerable_time,
            target.hurt_time,
            target.hurt_duration,
        ),
        (1.5, 20, 10, 10)
    );
    let damage = target.last_damage.as_ref().unwrap();
    assert_eq!(
        damage.source_position,
        Some(Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        })
    );
    assert!(!damage.cause_resolved);
    assert!(!damage.direct_resolved);
    assert_eq!(target.last_damage_game_time, Some(123));

    let lookup = PlayClientboundPacket::DamageEvent(DamageEvent {
        entity_id: 3,
        damage_type: id("minecraft:in_fire"),
        cause_entity_id: 2,
        direct_entity_id: 99,
        source_position: None,
    });
    client.apply(&lookup).unwrap();
    let damage = client.entity(3).unwrap().last_damage.as_ref().unwrap();
    assert!(damage.cause_resolved);
    assert!(!damage.direct_resolved);
    assert_eq!(
        client
            .apply(&PlayClientboundPacket::DamageEvent(DamageEvent {
                entity_id: 99,
                damage_type: id("minecraft:generic"),
                cause_entity_id: -1,
                direct_entity_id: -1,
                source_position: None,
            }))
            .unwrap(),
        EntitySessionAction::Ignored
    );
}

#[test]
fn c3_hurt_and_camera_use_current_entity_lookup_without_acknowledgement() {
    let mut client = EntitySessionClientProjection::new(1);
    client.track_entity(1, session_entity(SessionEntityKind::Player));
    client.track_entity(2, session_entity(SessionEntityKind::Generic));
    let hurt = PlayClientboundPacket::HurtAnimation(HurtAnimation {
        entity_id: 2,
        yaw: f32::NAN,
    });
    assert_eq!(client.apply(&hurt).unwrap(), EntitySessionAction::Applied);
    assert!(client.entity(2).unwrap().hurt_yaw.unwrap().is_nan());

    assert_eq!(
        client
            .apply(&PlayClientboundPacket::SetCamera(SetCamera {
                entity_id: 99,
            }))
            .unwrap(),
        EntitySessionAction::Ignored
    );
    assert_eq!(client.camera_entity_id(), Some(1));
    assert_eq!(
        client
            .apply(&PlayClientboundPacket::SetCamera(SetCamera {
                entity_id: 2,
            }))
            .unwrap(),
        EntitySessionAction::CameraChanged
    );
    assert_eq!(client.camera_entity_id(), Some(2));
}

#[test]
fn c3_pickup_resolves_collector_first_and_applies_source_specific_removal() {
    let mut client = EntitySessionClientProjection::new(1);
    client.track_entity(1, session_entity(SessionEntityKind::Player));
    client.track_entity(2, SessionEntityProjection::item(5));
    client.track_entity(3, session_entity(SessionEntityKind::ExperienceOrb));
    client.track_entity(4, session_entity(SessionEntityKind::Generic));
    client.track_entity(5, session_entity(SessionEntityKind::Generic));

    assert!(matches!(
        client.apply(&PlayClientboundPacket::TakeItemEntity(TakeItemEntity {
            source_entity_id: 99,
            collector_entity_id: 5,
            amount: 1,
        })),
        Err(EntitySessionProjectionError::WrongEntityType { .. })
    ));
    assert_eq!(
        client
            .apply(&PlayClientboundPacket::TakeItemEntity(TakeItemEntity {
                source_entity_id: 2,
                collector_entity_id: 99,
                amount: -2,
            }))
            .unwrap(),
        EntitySessionAction::PickupProjected
    );
    assert_eq!(client.entity(2).unwrap().item_count, 7);
    assert_eq!(
        client
            .apply(&PlayClientboundPacket::TakeItemEntity(TakeItemEntity {
                source_entity_id: 2,
                collector_entity_id: 1,
                amount: 7,
            }))
            .unwrap(),
        EntitySessionAction::SourceRemoved
    );
    assert!(client.entity(2).unwrap().removed);

    assert_eq!(
        client
            .apply(&PlayClientboundPacket::TakeItemEntity(TakeItemEntity {
                source_entity_id: 3,
                collector_entity_id: 1,
                amount: i32::MAX,
            }))
            .unwrap(),
        EntitySessionAction::PickupProjected
    );
    assert!(!client.entity(3).unwrap().removed);
    assert_eq!(
        client
            .apply(&PlayClientboundPacket::TakeItemEntity(TakeItemEntity {
                source_entity_id: 4,
                collector_entity_id: 1,
                amount: 0,
            }))
            .unwrap(),
        EntitySessionAction::SourceRemoved
    );
}

#[test]
fn c3_respawn_replaces_session_and_applies_independent_keep_bits() {
    let mut client = EntitySessionClientProjection::new(1);
    client.install_respawn_session(respawn_session());
    let packet = PlayClientboundPacket::Respawn(Respawn {
        spawn: spawn("minecraft:overworld"),
        data_to_keep: 0x7e,
    });
    assert_eq!(
        client.apply(&packet).unwrap(),
        EntitySessionAction::Respawned {
            dimension_changed: false,
        }
    );
    let session = client.respawn_session().unwrap();
    assert_eq!(session.player.entity_id, 1);
    assert_eq!(session.player.position, player_projection().position);
    assert_eq!(session.player.motion, player_projection().motion);
    assert_eq!((session.player.yaw, session.player.pitch), (40.0, 50.0));
    assert_eq!(session.player.last_input, 0xa5);
    assert!(session.player.sprinting);
    let attribute = &session.player.attributes[&id("minecraft:generic.max_health")];
    assert_eq!((attribute.base, attribute.value), (20.0, 20.0));
    assert!(attribute.modifiers.is_empty());
    assert_eq!(
        (
            session.player.stats_generation,
            session.player.recipe_book_generation,
        ),
        (8, 9)
    );
    assert!(!session.container_open);
    assert!(!session.client_loaded);
    assert_eq!(session.camera_entity_id, Some(1));
    assert_eq!(session.wait_reason, Some(LevelWaitReason::Respawn));
}

#[test]
fn c3_dimension_respawn_replaces_level_and_attribute_keep_is_independent() {
    let mut client = EntitySessionClientProjection::new(1);
    client.install_respawn_session(respawn_session());
    let packet = PlayClientboundPacket::Respawn(Respawn {
        spawn: spawn("minecraft:the_nether"),
        data_to_keep: 0x01,
    });
    assert_eq!(
        client.apply(&packet).unwrap(),
        EntitySessionAction::Respawned {
            dimension_changed: true,
        }
    );
    let session = client.respawn_session().unwrap();
    assert_eq!(session.level_generation, 5);
    assert!(!session.debug_subscriptions_installed);
    assert_eq!(session.player.position, Vector3::default());
    assert_eq!(session.player.motion, Vector3::default());
    assert_eq!((session.player.yaw, session.player.pitch), (-180.0, 0.0));
    let attribute = &session.player.attributes[&id("minecraft:generic.max_health")];
    assert_eq!((attribute.base, attribute.value), (20.0, 30.0));
    assert_eq!(attribute.modifiers, vec![id("test:boost")]);
    assert_eq!(session.wait_reason, Some(LevelWaitReason::DimensionChange));

    assert_eq!(
        client.apply(&packet).unwrap(),
        EntitySessionAction::Respawned {
            dimension_changed: false,
        }
    );
}

#[test]
fn c3_entity_session_publication_locks_audiences_gates_and_order() {
    assert_eq!(
        animation_audience(0, false),
        AnimationPublicationAudience::Trackers
    );
    assert_eq!(
        animation_audience(3, true),
        AnimationPublicationAudience::TrackersAndSelf
    );
    for action in [2, 4, 5] {
        assert_eq!(
            animation_audience(action, false),
            AnimationPublicationAudience::TrackersAndSelf
        );
    }
    assert!(publish_damage_event(true, false));
    assert!(!publish_damage_event(false, false));
    assert!(!publish_damage_event(true, true));
    assert!(publish_hurt_animation_to_damaged_player(false));
    assert!(!publish_hurt_animation_to_damaged_player(true));
    assert_eq!(
        CAMERA_PUBLICATION_ORDER,
        [
            CameraPublicationStep::ChangeAuthoritativeCamera,
            CameraPublicationStep::RelocatePlayer,
            CameraPublicationStep::UpdateChunkTracking,
            CameraPublicationStep::SendCamera,
            CameraPublicationStep::ResetKnownPosition,
        ]
    );
    assert_eq!(
        (
            PICKUP_AUDIENCE.tracking_source,
            PICKUP_AUDIENCE.include_source_when_player,
        ),
        (true, false)
    );
    assert_eq!(DEATH_RESPAWN_ORDER[0], RespawnPublicationStep::Respawn);
    assert_eq!(
        DEATH_RESPAWN_ORDER[1],
        RespawnPublicationStep::PositionChallenge
    );
    assert_eq!(
        CROSS_DIMENSION_ORDER,
        [
            CrossDimensionPublicationStep::Respawn,
            CrossDimensionPublicationStep::Difficulty,
            CrossDimensionPublicationStep::Permission,
            CrossDimensionPublicationStep::TransferLevel,
            CrossDimensionPublicationStep::PositionChallenge,
            CrossDimensionPublicationStep::Abilities,
            CrossDimensionPublicationStep::NewLevelProjection,
        ]
    );
}
