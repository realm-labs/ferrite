use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::codec::EntityEffectsCodecError;
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::packet::SoundEventHolder;
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::sound::codec::SoundCodecError;
use ferrite_protocol::java_26_2::play::clientbound::sound::packet::{
    SoundAtEntity, SoundAtPosition, SoundSource, StopSound,
};
use ferrite_protocol::java_26_2::play::clientbound::sound::projection::{
    SoundInstanceBinding, SoundProjection, SoundProjectionAction, TrackedSoundEntity,
};
use ferrite_protocol::java_26_2::play::clientbound::sound::publication::{
    AuthoredSound, EntitySoundRequest, EntitySoundTarget, PositionSoundRequest, SoundViewer,
    publish_entity_sound, publish_position_sound, publish_stop_sound, sound_range,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{PlayRegistries, PlayRegistryError, SOUND_EVENT};
use ferrite_protocol::java_26_2::value::identifier::Identifier;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(SOUND_EVENT),
        vec![id("minecraft:bell"), id("minecraft:chime")],
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

fn registered() -> SoundEventHolder {
    SoundEventHolder::Registered(id("minecraft:bell"))
}

fn direct(name: &str, fixed_range: Option<f32>) -> SoundEventHolder {
    SoundEventHolder::Direct {
        identity: id(name),
        fixed_range,
    }
}

fn entity_packet(sound: &str, source: SoundSource, entity_id: i32) -> SoundAtEntity {
    SoundAtEntity {
        sound: direct(sound, None),
        source,
        entity_id,
        volume: 1.0,
        pitch: 1.0,
        seed: 7,
    }
}

fn position(x: f64, y: f64, z: f64) -> Vector3 {
    Vector3 { x, y, z }
}

fn assert_roundtrip(packet: PlayClientboundPacket) {
    let registries = registries();
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );
}

#[test]
fn c3_gold_clientbound_sound_locks_all_three_empty_numeric_bodies() {
    let registries = registries();
    let position_packet = PlayClientboundPacket::SoundAtPosition(SoundAtPosition {
        sound: registered(),
        source: SoundSource::Master,
        encoded_position: [0; 3],
        volume: 0.0,
        pitch: 0.0,
        seed: 0,
    });
    let mut expected_position = vec![117, 1, 0];
    expected_position.extend([0; 28]);
    assert_eq!(
        encode_packet(&position_packet, &registries).unwrap(),
        expected_position
    );

    let entity_packet = PlayClientboundPacket::SoundAtEntity(SoundAtEntity {
        sound: registered(),
        source: SoundSource::Master,
        entity_id: 0,
        volume: 0.0,
        pitch: 0.0,
        seed: 0,
    });
    let mut expected_entity = vec![116, 1, 0, 0];
    expected_entity.extend([0; 16]);
    assert_eq!(
        encode_packet(&entity_packet, &registries).unwrap(),
        expected_entity
    );

    let stop = PlayClientboundPacket::StopSound(StopSound {
        source: None,
        sound: None,
    });
    assert_eq!(encode_packet(&stop, &registries).unwrap(), [119, 0]);
}

#[test]
fn c3_sound_codecs_roundtrip_holder_sources_ieee_and_signed_domains() {
    let sources = [
        SoundSource::Master,
        SoundSource::Music,
        SoundSource::Records,
        SoundSource::Weather,
        SoundSource::Blocks,
        SoundSource::Hostile,
        SoundSource::Neutral,
        SoundSource::Players,
        SoundSource::Ambient,
        SoundSource::Voice,
        SoundSource::Ui,
    ];
    for source in sources {
        assert_roundtrip(PlayClientboundPacket::SoundAtPosition(SoundAtPosition {
            sound: direct("ferrite:test", Some(f32::from_bits(1))),
            source,
            encoded_position: [i32::MIN, -1, i32::MAX],
            volume: f32::INFINITY,
            pitch: f32::NEG_INFINITY,
            seed: i64::MIN,
        }));
        assert_roundtrip(PlayClientboundPacket::SoundAtEntity(SoundAtEntity {
            sound: SoundEventHolder::Registered(id("minecraft:chime")),
            source,
            entity_id: i32::MIN,
            volume: f32::from_bits(1),
            pitch: -0.0,
            seed: i64::MAX,
        }));
    }
}

#[test]
fn c3_stop_sound_normalizes_high_flags_and_preserves_conditional_field_order() {
    let registries = registries();
    let decoded = decode_packet(&[119, 0xfc], context(&registries)).unwrap();
    assert_eq!(
        decoded,
        PlayClientboundPacket::StopSound(StopSound {
            source: None,
            sound: None,
        })
    );
    assert_eq!(encode_packet(&decoded, &registries).unwrap(), [119, 0]);

    let both = PlayClientboundPacket::StopSound(StopSound {
        source: Some(SoundSource::Ui),
        sound: Some(id("minecraft:bell")),
    });
    let encoded = encode_packet(&both, &registries).unwrap();
    assert_eq!(&encoded[..3], &[119, 3, 10]);
    assert_eq!(decode_packet(&encoded, context(&registries)).unwrap(), both);
}

#[test]
fn c3_sound_codecs_reject_unknown_holder_source_identifier_and_residual_bytes() {
    let registries = registries();
    assert_eq!(
        decode_packet(&[117, 1, 11], context(&registries)),
        Err(PlayClientboundCodecError::Sound(
            SoundCodecError::InvalidSource { raw_id: 11 }
        ))
    );
    assert_eq!(
        decode_packet(&[117, 3], context(&registries)),
        Err(PlayClientboundCodecError::Sound(SoundCodecError::Holder(
            EntityEffectsCodecError::Registry(PlayRegistryError::UnknownRawId {
                registry: SOUND_EVENT,
                raw_id: 2,
            })
        )))
    );
    assert!(decode_packet(&[117, 0, 1, b':'], context(&registries)).is_err());
    assert!(decode_packet(&[117], context(&registries)).is_err());

    let mut trailing = encode_packet(
        &PlayClientboundPacket::StopSound(StopSound {
            source: None,
            sound: None,
        }),
        &registries,
    )
    .unwrap();
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn c3_position_sound_uses_java_saturating_constructor_and_float_decode_narrowing() {
    let packet = SoundAtPosition::new(
        registered(),
        SoundSource::Master,
        position(f64::NAN, f64::INFINITY, f64::NEG_INFINITY),
        1.0,
        1.0,
        0,
    );
    assert_eq!(packet.encoded_position, [0, i32::MAX, i32::MIN]);
    assert_eq!(
        packet.position(),
        position(0.0, 268_435_456.0, -268_435_456.0)
    );

    let truncated = SoundAtPosition::new(
        registered(),
        SoundSource::Master,
        position(1.249, -1.249, 1.125),
        1.0,
        1.0,
        0,
    );
    assert_eq!(truncated.encoded_position, [9, -9, 9]);
    assert_eq!(truncated.position(), position(1.125, -1.125, 1.125));
}

#[test]
fn c3_sound_projection_keeps_duplicate_position_instances_and_stops_current_only() {
    let packet = SoundAtPosition::new(
        direct("minecraft:bell", None),
        SoundSource::Records,
        position(1.0, 2.0, 3.0),
        2.0,
        0.5,
        9,
    );
    let mut projection = SoundProjection::default();
    assert!(matches!(
        projection.apply_position(packet.clone()),
        SoundProjectionAction::Played(_)
    ));
    projection.apply_position(packet.clone());
    assert_eq!(projection.instances().len(), 2);
    assert_eq!(
        projection.apply_stop(&StopSound {
            source: Some(SoundSource::Records),
            sound: Some(id("minecraft:bell")),
        }),
        SoundProjectionAction::Stopped { count: 2 }
    );
    assert!(projection.instances().is_empty());

    projection.apply_position(packet);
    assert_eq!(projection.instances().len(), 1);
}

#[test]
fn c3_entity_sound_lookup_is_one_shot_and_binds_exact_object_identity() {
    let packet = entity_packet("minecraft:bell", SoundSource::Players, 7);
    let mut projection = SoundProjection::default();
    let mut entities = BTreeMap::new();
    assert_eq!(
        projection.apply_entity(packet.clone(), &entities),
        SoundProjectionAction::MissingEntity
    );
    entities.insert(
        7,
        TrackedSoundEntity {
            object_token: 41,
            position: position(1.000_000_06, 2.0, 3.0),
            silent: true,
            removed: false,
        },
    );
    assert_eq!(
        projection.apply_entity(packet.clone(), &entities),
        SoundProjectionAction::SilentEntity
    );
    assert!(projection.instances().is_empty());

    entities.get_mut(&7).unwrap().silent = false;
    let SoundProjectionAction::Played(instance) = projection.apply_entity(packet, &entities) else {
        panic!("present non-silent entity must play");
    };
    assert_eq!(
        instance.position,
        position(f64::from(1.000_000_06_f64 as f32), 2.0, 3.0)
    );
    assert_eq!(
        instance.binding,
        SoundInstanceBinding::Entity {
            entity_id: 7,
            object_token: 41,
        }
    );

    entities.get_mut(&7).unwrap().position = position(8.000_001, 9.0, 10.0);
    assert_eq!(projection.tick_entity_bindings(&entities), 0);
    assert_eq!(
        projection.instances()[0].position.x,
        f64::from(8.000_001_f64 as f32)
    );
    entities.get_mut(&7).unwrap().object_token = 42;
    assert_eq!(projection.tick_entity_bindings(&entities), 1);
    assert!(projection.instances().is_empty());
}

#[test]
fn c3_stop_sound_implements_all_four_filters_without_future_suppression() {
    let mut projection = SoundProjection::default();
    for (sound, source) in [
        ("minecraft:bell", SoundSource::Music),
        ("minecraft:bell", SoundSource::Records),
        ("minecraft:chime", SoundSource::Music),
        ("minecraft:chime", SoundSource::Records),
    ] {
        projection.apply_position(SoundAtPosition::new(
            direct(sound, None),
            source,
            Vector3::default(),
            1.0,
            1.0,
            0,
        ));
    }
    assert_eq!(
        projection.apply_stop(&StopSound {
            source: Some(SoundSource::Music),
            sound: Some(id("minecraft:bell")),
        }),
        SoundProjectionAction::Stopped { count: 1 }
    );
    assert_eq!(
        projection.apply_stop(&StopSound {
            source: None,
            sound: Some(id("minecraft:chime")),
        }),
        SoundProjectionAction::Stopped { count: 2 }
    );
    assert_eq!(
        projection.apply_stop(&StopSound {
            source: Some(SoundSource::Records),
            sound: None,
        }),
        SoundProjectionAction::Stopped { count: 1 }
    );
    assert_eq!(
        projection.apply_stop(&StopSound {
            source: None,
            sound: None,
        }),
        SoundProjectionAction::Stopped { count: 0 }
    );
}

#[test]
fn c3_sound_publication_uses_strict_dimension_range_exclusion_and_list_order() {
    let dimension = id("minecraft:overworld");
    let event = AuthoredSound {
        holder: registered(),
        fixed_range: None,
    };
    let viewers = [
        SoundViewer {
            player_id: 1,
            dimension: dimension.clone(),
            position: position(15.999, 0.0, 0.0),
        },
        SoundViewer {
            player_id: 2,
            dimension: dimension.clone(),
            position: position(16.0, 0.0, 0.0),
        },
        SoundViewer {
            player_id: 3,
            dimension: id("minecraft:the_nether"),
            position: Vector3::default(),
        },
        SoundViewer {
            player_id: 4,
            dimension: dimension.clone(),
            position: Vector3::default(),
        },
    ];
    let deliveries = publish_position_sound(
        &viewers,
        PositionSoundRequest {
            excluded_source_player: Some(4),
            dimension: &dimension,
            position: Vector3::default(),
            event: &event,
            source: SoundSource::Master,
            volume: 1.0,
            pitch: 1.0,
            seed: 1,
        },
    );
    assert_eq!(
        deliveries
            .iter()
            .map(|delivery| delivery.recipient)
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(deliveries[0].packet.encoded_position, [0; 3]);

    assert_eq!(sound_range(&event, 1.0), 16.0);
    assert_eq!(sound_range(&event, 2.0), 32.0);
    assert_eq!(sound_range(&event, -2.0), 16.0);
}

#[test]
fn c3_sound_fixed_ranges_retain_negative_nan_and_infinite_audience_behavior() {
    let dimension = id("minecraft:overworld");
    let viewers = [
        SoundViewer {
            player_id: 1,
            dimension: dimension.clone(),
            position: position(3.0, 0.0, 0.0),
        },
        SoundViewer {
            player_id: 2,
            dimension: dimension.clone(),
            position: position(5.0, 0.0, 0.0),
        },
    ];
    let publish = |range| {
        let event = AuthoredSound {
            holder: registered(),
            fixed_range: Some(range),
        };
        publish_position_sound(
            &viewers,
            PositionSoundRequest {
                excluded_source_player: None,
                dimension: &dimension,
                position: Vector3::default(),
                event: &event,
                source: SoundSource::Master,
                volume: 1.0,
                pitch: 1.0,
                seed: 0,
            },
        )
    };
    assert_eq!(publish(-4.0).len(), 1);
    assert!(publish(f32::NAN).is_empty());
    assert_eq!(publish(f32::INFINITY).len(), 2);
}

#[test]
fn c3_entity_and_stop_publication_use_target_position_and_selected_players_directly() {
    let dimension = id("minecraft:overworld");
    let event = AuthoredSound {
        holder: direct("minecraft:bell", Some(8.0)),
        fixed_range: Some(8.0),
    };
    let viewers = [
        SoundViewer {
            player_id: 8,
            dimension: dimension.clone(),
            position: position(7.0, 0.0, 0.0),
        },
        SoundViewer {
            player_id: 9,
            dimension: dimension.clone(),
            position: position(8.0, 0.0, 0.0),
        },
    ];
    let deliveries = publish_entity_sound(
        &viewers,
        EntitySoundRequest {
            excluded_source_player: None,
            dimension: &dimension,
            target: EntitySoundTarget {
                entity_id: -7,
                position: Vector3::default(),
            },
            event: &event,
            source: SoundSource::Voice,
            volume: 2.0,
            pitch: 0.5,
            seed: -1,
        },
    );
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].recipient, 8);
    assert_eq!(deliveries[0].packet.entity_id, -7);

    let stop = StopSound {
        source: Some(SoundSource::Voice),
        sound: None,
    };
    let stopped = publish_stop_sound(&[9, 8, 9], stop.clone());
    assert_eq!(
        stopped
            .iter()
            .map(|delivery| delivery.recipient)
            .collect::<Vec<_>>(),
        [9, 8, 9]
    );
    assert!(stopped.iter().all(|delivery| delivery.packet == stop));
}

#[test]
fn c3_sound_projection_requires_an_installed_play_level() {
    let packets = [
        PlayClientboundPacket::SoundAtEntity(entity_packet(
            "minecraft:bell",
            SoundSource::Master,
            1,
        )),
        PlayClientboundPacket::SoundAtPosition(SoundAtPosition::new(
            registered(),
            SoundSource::Master,
            Vector3::default(),
            1.0,
            1.0,
            0,
        )),
        PlayClientboundPacket::StopSound(StopSound {
            source: None,
            sound: None,
        }),
    ];
    for packet in packets {
        assert_eq!(
            PlayEntryProjection::default().apply(packet),
            Err(PlayProjectionError::LevelNotInstalled)
        );
    }
}
