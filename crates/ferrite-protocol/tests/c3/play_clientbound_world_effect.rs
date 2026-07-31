use std::collections::BTreeSet;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::world_effect::packet::LevelEvent;
use ferrite_protocol::java_26_2::play::clientbound::world_effect::projection::{
    Axis, BlockStateSelection, Direction, ExtinguishKind, FlameKind, GlobalLevelEffect,
    LOCAL_EVENT_IDENTITIES, LevelEventData, LevelEventFault, LevelEventProjection, local_event_id,
    local_identity, project_level_event,
};
use ferrite_protocol::java_26_2::play::clientbound::world_effect::publication::{
    GlobalLevelEventRequest, LevelEventViewer, LocalLevelEventRequest, publish_global_level_event,
    publish_local_level_event,
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

fn event(event_type: i32, data: i32, global: bool) -> LevelEvent {
    LevelEvent {
        event_type,
        position: BlockPos::new(0, 0, 0),
        data,
        global,
    }
}

fn project(event_type: i32, data: i32) -> LevelEventProjection {
    project_level_event(&event(event_type, data, false), &[])
}

fn position(x: f64, y: f64, z: f64) -> Vector3 {
    Vector3 { x, y, z }
}

#[test]
fn c3_gold_clientbound_level_event_locks_the_fixed_zero_body() {
    let registries = PlayRegistries::default();
    let packet = PlayClientboundPacket::LevelEvent(event(0, 0, false));
    let mut expected = vec![46];
    expected.extend([0; 17]);
    assert_eq!(encode_packet(&packet, &registries).unwrap(), expected);
    assert_eq!(
        decode_packet(&expected, context(&registries)).unwrap(),
        packet
    );
}

#[test]
fn c3_level_event_codec_preserves_signed_domains_positions_and_nonzero_boolean() {
    let registries = PlayRegistries::default();
    for packet in [
        LevelEvent {
            event_type: i32::MIN,
            position: BlockPos::new(-33_554_432, -2_048, -33_554_432),
            data: i32::MAX,
            global: false,
        },
        LevelEvent {
            event_type: i32::MAX,
            position: BlockPos::new(33_554_431, 2_047, 33_554_431),
            data: i32::MIN,
            global: true,
        },
    ] {
        let wrapped = PlayClientboundPacket::LevelEvent(packet);
        let encoded = encode_packet(&wrapped, &registries).unwrap();
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            wrapped
        );
    }

    let mut noncanonical_true = encode_packet(
        &PlayClientboundPacket::LevelEvent(event(1023, 0, true)),
        &registries,
    )
    .unwrap();
    *noncanonical_true.last_mut().unwrap() = 0xff;
    assert_eq!(
        decode_packet(&noncanonical_true, context(&registries)).unwrap(),
        PlayClientboundPacket::LevelEvent(event(1023, 0, true))
    );
}

#[test]
fn c3_level_event_codec_faults_truncation_and_residual_bytes() {
    let registries = PlayRegistries::default();
    let encoded = encode_packet(
        &PlayClientboundPacket::LevelEvent(event(1000, 0, false)),
        &registries,
    )
    .unwrap();
    for length in 1..encoded.len() {
        assert!(decode_packet(&encoded[..length], context(&registries)).is_err());
    }
    let mut trailing = encoded;
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn c3_level_event_dispatch_tables_are_complete_distinct_and_no_op_for_unknowns() {
    assert_eq!(LOCAL_EVENT_IDENTITIES.len(), 80);
    let ids = LOCAL_EVENT_IDENTITIES
        .iter()
        .map(|(event_type, _)| *event_type)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 80);
    for (event_type, identity) in LOCAL_EVENT_IDENTITIES {
        assert_eq!(local_identity(event_type), Some(identity));
        assert_eq!(local_event_id(&id(identity)), Some(event_type));
        assert!(matches!(
            project_level_event(&event(event_type, 0, false), &[]),
            LevelEventProjection::Local { .. }
                | LevelEventProjection::NoOp
                | LevelEventProjection::HandlerFault { .. }
        ));
        assert_eq!(
            project_level_event(&event(event_type, 0, true), &[]),
            LevelEventProjection::NoOp
        );
    }
    for global_only in [1023, 1028, 1038] {
        assert_eq!(
            project(global_only, 0),
            LevelEventProjection::NoOp,
            "global-only event {global_only} must not fall through to the local table"
        );
    }
    assert_eq!(project(i32::MIN, i32::MAX), LevelEventProjection::NoOp);
    assert_eq!(local_event_id(&id("ferrite:block_destroy")), None);
}

#[test]
fn c3_global_level_event_table_has_exactly_three_entries() {
    for (event_type, effect) in [
        (1023, GlobalLevelEffect::WitherSpawn),
        (1028, GlobalLevelEffect::EnderDragonDeath),
        (1038, GlobalLevelEffect::EndPortalSpawn),
    ] {
        assert_eq!(
            project_level_event(&event(event_type, i32::MIN, true), &[]),
            LevelEventProjection::Global(effect)
        );
        assert!(effect.identity().starts_with("minecraft:"));
    }
    for event_type in [1000, 1022, 1024, 2001, 3021, i32::MAX] {
        assert_eq!(
            project_level_event(&event(event_type, 0, true), &[]),
            LevelEventProjection::NoOp
        );
    }
}

#[test]
fn c3_jukebox_data_resolves_only_present_nonnegative_dynamic_raw_ids() {
    let songs = [id("minecraft:cat"), id("minecraft:chirp")];
    assert_eq!(
        project_level_event(&event(1010, 1, false), &songs),
        LevelEventProjection::Local {
            identity: "minecraft:jukebox_play",
            data: LevelEventData::JukeboxSong(id("minecraft:chirp")),
        }
    );
    for invalid in [-1, 2, i32::MAX] {
        assert_eq!(
            project_level_event(&event(1010, invalid, false), &songs),
            LevelEventProjection::NoOp
        );
    }
    assert_eq!(
        project(1011, i32::MIN),
        LevelEventProjection::Local {
            identity: "minecraft:jukebox_stop",
            data: LevelEventData::Ignored,
        }
    );
}

#[test]
fn c3_local_level_event_data_rules_normalize_exact_signed_inputs() {
    assert!(matches!(
        project(1009, 0),
        LevelEventProjection::Local {
            data: LevelEventData::Extinguish(ExtinguishKind::BlockFire),
            ..
        }
    ));
    assert!(matches!(
        project(1009, 1),
        LevelEventProjection::Local {
            data: LevelEventData::Extinguish(ExtinguishKind::EntityFire),
            ..
        }
    ));
    assert_eq!(project(1009, 2), LevelEventProjection::NoOp);
    assert!(matches!(
        project(1500, -1),
        LevelEventProjection::Local {
            data: LevelEventData::Composter { successful: false },
            ..
        }
    ));
    assert!(matches!(
        project(1500, 1),
        LevelEventProjection::Local {
            data: LevelEventData::Composter { successful: true },
            ..
        }
    ));
    assert!(matches!(
        project(2000, -5),
        LevelEventProjection::Local {
            data: LevelEventData::DirectionalSmoke(Direction::East),
            ..
        }
    ));
    assert!(matches!(
        project(2001, 32_365),
        LevelEventProjection::Local {
            data: LevelEventData::BlockState(BlockStateSelection::RawId(32_365)),
            ..
        }
    ));
    assert!(matches!(
        project(3008, 32_366),
        LevelEventProjection::Local {
            data: LevelEventData::BlockState(BlockStateSelection::AirFallback),
            ..
        }
    ));
    assert!(matches!(
        project(2002, 0x12_34_56),
        LevelEventProjection::Local {
            data: LevelEventData::PotionColor([0x12, 0x34, 0x56]),
            ..
        }
    ));
    assert!(matches!(
        project(2006, 1),
        LevelEventProjection::Local {
            data: LevelEventData::DragonBreath {
                play_explosion_sound: true
            },
            ..
        }
    ));
}

#[test]
fn c3_sculk_electric_growth_and_detection_data_keep_java_arithmetic() {
    assert!(matches!(
        project(3002, 2),
        LevelEventProjection::Local {
            data: LevelEventData::ElectricSpark {
                axis: Some(Axis::Z)
            },
            ..
        }
    ));
    assert!(matches!(
        project(3002, 3),
        LevelEventProjection::Local {
            data: LevelEventData::ElectricSpark { axis: None },
            ..
        }
    ));
    assert!(matches!(
        project(3006, (7 << 6) | 0b10_0101),
        LevelEventProjection::Local {
            data: LevelEventData::SculkCharge {
                count: 7,
                face_mask: 0b10_0101
            },
            ..
        }
    ));
    assert!(matches!(
        project(3006, -1),
        LevelEventProjection::Local {
            data: LevelEventData::SculkCharge {
                count: -1,
                face_mask: 0
            },
            ..
        }
    ));
    assert!(matches!(
        project(2011, i32::MIN),
        LevelEventProjection::Local {
            data: LevelEventData::GrowthParticles { count: i32::MIN },
            ..
        }
    ));
    let expected = 30_i32.wrapping_add(i32::MIN.wrapping_mul(5));
    assert!(matches!(
        project(3013, i32::MIN),
        LevelEventProjection::Local {
            data: LevelEventData::DetectionParticles { loop_bound },
            ..
        } if loop_bound == expected
    ));
}

#[test]
fn c3_trial_and_vault_flame_decoding_preserves_the_irregular_fault_boundary() {
    for data in [i32::MIN, -1, 0, 3, i32::MAX] {
        assert!(matches!(
            project(3011, data),
            LevelEventProjection::Local {
                data: LevelEventData::TrialFlame(FlameKind::Flame),
                ..
            }
        ));
    }
    assert!(matches!(
        project(3021, 1),
        LevelEventProjection::Local {
            data: LevelEventData::TrialFlame(FlameKind::SoulFireFlame),
            ..
        }
    ));
    for (event_type, retained_prefix) in [(3011, false), (3012, true), (3021, true)] {
        assert!(matches!(
            project(event_type, 2),
            LevelEventProjection::HandlerFault {
                fault: LevelEventFault::TrialFlameIndexOutOfBounds,
                retained_prefix: actual,
                ..
            } if actual == retained_prefix
        ));
    }
    assert!(matches!(
        project(3015, -1),
        LevelEventProjection::Local {
            data: LevelEventData::VaultFlame(FlameKind::SoulFireFlame),
            ..
        }
    ));
    assert!(matches!(
        project(3020, 0),
        LevelEventProjection::Local {
            data: LevelEventData::OminousActivation { volume: 0.3 },
            ..
        }
    ));
}

#[test]
fn c3_local_publication_uses_list_order_source_dimension_and_strict_radius() {
    let overworld = id("minecraft:overworld");
    let nether = id("minecraft:the_nether");
    let viewers = [
        LevelEventViewer {
            player_id: 3,
            dimension: overworld.clone(),
            position: position(63.999, 0.0, 0.0),
            block_position: BlockPos::new(63, 0, 0),
        },
        LevelEventViewer {
            player_id: 1,
            dimension: overworld.clone(),
            position: position(1.0, 0.0, 0.0),
            block_position: BlockPos::new(1, 0, 0),
        },
        LevelEventViewer {
            player_id: 4,
            dimension: overworld.clone(),
            position: position(64.0, 0.0, 0.0),
            block_position: BlockPos::new(64, 0, 0),
        },
        LevelEventViewer {
            player_id: 2,
            dimension: nether,
            position: position(0.0, 0.0, 0.0),
            block_position: BlockPos::new(0, 0, 0),
        },
    ];
    let deliveries = publish_local_level_event(
        &viewers,
        LocalLevelEventRequest {
            excluded_source_player: Some(1),
            dimension: &overworld,
            effect: &id("minecraft:block_destroy"),
            position: BlockPos::new(0, 0, 0),
            data: 7,
        },
    )
    .unwrap();
    assert_eq!(
        deliveries
            .iter()
            .map(|delivery| delivery.recipient)
            .collect::<Vec<_>>(),
        [3]
    );
    assert_eq!(deliveries[0].packet, event(2001, 7, false));
}

#[test]
fn c3_global_publication_projects_near_far_cross_dimension_and_rule_fallback() {
    let overworld = id("minecraft:overworld");
    let viewers = [
        LevelEventViewer {
            player_id: 1,
            dimension: overworld.clone(),
            position: position(0.5, 0.5, 0.5),
            block_position: BlockPos::new(0, 0, 0),
        },
        LevelEventViewer {
            player_id: 2,
            dimension: overworld.clone(),
            position: position(100.5, 0.5, 0.5),
            block_position: BlockPos::new(100, 0, 0),
        },
        LevelEventViewer {
            player_id: 3,
            dimension: id("minecraft:the_nether"),
            position: position(7.5, 8.5, 9.5),
            block_position: BlockPos::new(7, 8, 9),
        },
    ];
    let request = GlobalLevelEventRequest {
        dimension: &overworld,
        effect: GlobalLevelEffect::WitherSpawn,
        position: BlockPos::new(0, 0, 0),
        data: i32::MIN,
        global_sound_events: true,
    };
    let deliveries = publish_global_level_event(&viewers, request);
    assert_eq!(
        deliveries
            .iter()
            .map(|delivery| (
                delivery.recipient,
                delivery.packet.position,
                delivery.packet.global
            ))
            .collect::<Vec<_>>(),
        [
            (1, BlockPos::new(0, 0, 0), true),
            (2, BlockPos::new(68, 0, 0), true),
            (3, BlockPos::new(7, 8, 9), true),
        ]
    );

    let fallback = publish_global_level_event(
        &viewers,
        GlobalLevelEventRequest {
            global_sound_events: false,
            ..request
        },
    );
    assert_eq!(fallback.len(), 1);
    assert_eq!(
        (fallback[0].recipient, fallback[0].packet.global),
        (1, false)
    );
}

#[test]
fn c3_level_event_decode_projection_and_play_stage_join_end_to_end() {
    let registries = PlayRegistries::default();
    let packet = PlayClientboundPacket::LevelEvent(LevelEvent {
        event_type: 3006,
        position: BlockPos::new(-12, 34, 56),
        data: (9 << 6) | 0b11_0001,
        global: false,
    });
    let encoded = encode_packet(&packet, &registries).unwrap();
    let decoded = decode_packet(&encoded, context(&registries)).unwrap();
    let PlayClientboundPacket::LevelEvent(decoded_event) = decoded else {
        panic!("decoded packet must retain its identity");
    };
    assert!(matches!(
        project_level_event(&decoded_event, &[]),
        LevelEventProjection::Local {
            data: LevelEventData::SculkCharge {
                count: 9,
                face_mask: 0b11_0001
            },
            ..
        }
    ));
    assert_eq!(
        PlayEntryProjection::default().apply(packet),
        Err(PlayProjectionError::LevelNotInstalled)
    );
}
