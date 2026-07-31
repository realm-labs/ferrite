use std::collections::{BTreeMap, BTreeSet};

use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, GameMode, PlayClientboundPacket, PlayLogin,
};
use ferrite_protocol::java_26_2::play::clientbound::player_projection::codec::PlayerProjectionCodecError;
use ferrite_protocol::java_26_2::play::clientbound::player_projection::packet::{
    AwardStats, Cooldown, SetExperience, SetHealth, StatisticKey,
};
use ferrite_protocol::java_26_2::play::clientbound::player_projection::projection::{
    HealthReaction, PlayerProjection,
};
use ferrite_protocol::java_26_2::play::clientbound::player_projection::publication::{
    PlayerProjectionDelivery, PlayerProjectionPublisher,
};
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{
    BLOCK, CUSTOM_STAT, ENTITY_TYPE, ITEM, PlayRegistries, PlayRegistryError, STAT_TYPE,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::wire::error::WireError;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn stat(kind: &str, value: &str) -> StatisticKey {
    StatisticKey {
        statistic_type: id(kind),
        value: id(value),
    }
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(STAT_TYPE),
        [
            "minecraft:mined",
            "minecraft:crafted",
            "minecraft:used",
            "minecraft:broken",
            "minecraft:picked_up",
            "minecraft:dropped",
            "minecraft:killed",
            "minecraft:killed_by",
            "minecraft:custom",
        ]
        .map(id)
        .to_vec(),
    );
    registries.insert(id(BLOCK), vec![id("minecraft:air"), id("minecraft:stone")]);
    registries.insert(id(ITEM), vec![id("minecraft:air"), id("minecraft:diamond")]);
    registries.insert(
        id(ENTITY_TYPE),
        vec![id("minecraft:pig"), id("minecraft:cow")],
    );
    registries.insert(id(CUSTOM_STAT), vec![id("minecraft:jump")]);
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
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

#[test]
fn c3_gold_clientbound_player_projection_locks_all_four_packet_bodies() {
    let registries = registries();
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::AwardStats(AwardStats {
                values: BTreeMap::new(),
            }),
            &registries,
        )
        .unwrap(),
        [3, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::Cooldown(Cooldown {
                group: id("minecraft:test"),
                duration_ticks: -1,
            }),
            &registries,
        )
        .unwrap(),
        [
            22, 14, b'm', b'i', b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b't', b'e', b's',
            b't', 0xff, 0xff, 0xff, 0xff, 0x0f,
        ]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::SetExperience(SetExperience {
                progress: 1.0,
                level: -1,
                total_experience: 2,
            }),
            &registries,
        )
        .unwrap(),
        [103, 0x3f, 0x80, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x0f, 2]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::SetHealth(SetHealth {
                health: -0.0,
                food: -1,
                saturation: f32::INFINITY,
            }),
            &registries,
        )
        .unwrap(),
        [
            104, 0x80, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x7f, 0x80, 0, 0,
        ]
    );
}

#[test]
fn c3_player_projection_codecs_preserve_ieee_and_signed_domains() {
    let registries = registries();
    for packet in [
        PlayClientboundPacket::SetExperience(SetExperience {
            progress: f32::from_bits(1),
            level: i32::MIN,
            total_experience: i32::MAX,
        }),
        PlayClientboundPacket::Cooldown(Cooldown {
            group: id("ferrite:group/path"),
            duration_ticks: i32::MIN,
        }),
    ] {
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
    let packet = PlayClientboundPacket::SetHealth(SetHealth {
        health: f32::from_bits(0x7fc0_0001),
        food: i32::MIN,
        saturation: f32::NEG_INFINITY,
    });
    let encoded = encode_packet(&packet, &registries).unwrap();
    let PlayClientboundPacket::SetHealth(decoded) =
        decode_packet(&encoded, context(&registries)).unwrap()
    else {
        panic!("set-health identity must decode to the typed packet");
    };
    assert_eq!(decoded.health.to_bits(), 0x7fc0_0001);
    assert_eq!(decoded.food, i32::MIN);
    assert_eq!(decoded.saturation, f32::NEG_INFINITY);
}

#[test]
fn c3_stat_mapping_replaces_duplicates_and_applies_only_defaulted_backings() {
    let registries = registries();
    let duplicate = [3, 2, 8, 0, 1, 8, 0, 2];
    let PlayClientboundPacket::AwardStats(decoded) =
        decode_packet(&duplicate, context(&registries)).unwrap()
    else {
        panic!("award-stats identity must decode to the typed packet");
    };
    assert_eq!(
        decoded.values,
        BTreeMap::from([(stat("minecraft:custom", "minecraft:jump"), 2)])
    );

    let defaulted = [3, 3, 0, 99, 5, 1, 99, 6, 6, 0xff, 0xff, 0xff, 0xff, 0x0f, 7];
    let PlayClientboundPacket::AwardStats(decoded) =
        decode_packet(&defaulted, context(&registries)).unwrap()
    else {
        panic!("award-stats identity must decode to the typed packet");
    };
    assert_eq!(decoded.values[&stat("minecraft:mined", "minecraft:air")], 5);
    assert_eq!(
        decoded.values[&stat("minecraft:crafted", "minecraft:air")],
        6
    );
    assert_eq!(
        decoded.values[&stat("minecraft:killed", "minecraft:pig")],
        7
    );

    assert!(matches!(
        decode_packet(&[3, 1, 9, 0, 0], context(&registries)),
        Err(PlayClientboundCodecError::PlayerProjection(
            PlayerProjectionCodecError::Registry(PlayRegistryError::UnknownRawId {
                registry: STAT_TYPE,
                raw_id: 9,
            })
        ))
    ));
    assert!(matches!(
        decode_packet(&[3, 1, 8, 1, 0], context(&registries)),
        Err(PlayClientboundCodecError::PlayerProjection(
            PlayerProjectionCodecError::Registry(PlayRegistryError::UnknownRawId {
                registry: CUSTOM_STAT,
                raw_id: 1,
            })
        ))
    ));
}

#[test]
fn c3_player_projection_malformed_fields_and_residual_bytes_fault() {
    let registries = registries();
    assert!(matches!(
        decode_packet(&[3, 0xff, 0xff, 0xff, 0xff, 0x0f], context(&registries)),
        Err(PlayClientboundCodecError::PlayerProjection(
            PlayerProjectionCodecError::Wire(WireError::NegativeLength {
                field: "statistics",
                value: -1,
            })
        ))
    ));
    assert!(decode_packet(&[3, 1], context(&registries)).is_err());
    assert!(decode_packet(&[22, 1, b'A', 0], context(&registries)).is_err());
    assert!(decode_packet(&[103], context(&registries)).is_err());
    assert!(decode_packet(&[104], context(&registries)).is_err());
    assert!(decode_packet(&[3, 0, 0], context(&registries)).is_err());
}

#[test]
fn c3_health_food_projection_locks_flash_clamp_and_raw_food_rules() {
    let mut projection = PlayerProjection::default();
    assert_eq!(
        projection.apply_health(SetHealth {
            health: 15.0,
            food: -7,
            saturation: f32::INFINITY,
        }),
        HealthReaction::FirstValue
    );
    assert_eq!(projection.health().food, -7);
    assert_eq!(projection.health().saturation, f32::INFINITY);
    assert_eq!(
        projection.apply_health(SetHealth {
            health: 10.0,
            food: 1,
            saturation: 2.0,
        }),
        HealthReaction::Hurt { amount: 5.0 }
    );
    assert_eq!(projection.health().invulnerable_ticks, 20);
    assert_eq!(projection.health().hurt_time, 10);
    assert_eq!(
        projection.apply_health(SetHealth {
            health: 10.0,
            food: 1,
            saturation: 2.0,
        }),
        HealthReaction::Equal
    );
    assert_eq!(projection.health().invulnerable_ticks, 20);
    assert_eq!(
        projection.apply_health(SetHealth {
            health: 12.0,
            food: 1,
            saturation: 2.0,
        }),
        HealthReaction::Increase
    );
    assert_eq!(projection.health().invulnerable_ticks, 10);

    projection.apply_health(SetHealth {
        health: -0.0,
        food: i32::MIN,
        saturation: f32::NEG_INFINITY,
    });
    assert!(projection.health().current.is_sign_negative());
    assert_eq!(
        projection.apply_health(SetHealth {
            health: f32::NAN,
            food: i32::MAX,
            saturation: f32::NAN,
        }),
        HealthReaction::NonFiniteNondamage
    );
    assert_eq!(projection.health().current, 0.0);
    assert_eq!(projection.health().invulnerable_ticks, 10);
    projection.apply_health(SetHealth {
        health: f32::INFINITY,
        food: 0,
        saturation: 0.0,
    });
    assert_eq!(projection.health().current, 20.0);
}

#[test]
fn c3_experience_and_cooldown_projection_replace_without_generation_guards() {
    let mut projection = PlayerProjection::default();
    projection.apply_experience(SetExperience {
        progress: f32::NAN,
        level: i32::MIN,
        total_experience: -1,
    });
    assert_eq!(projection.experience().display_start_tick, Some(0));
    projection.tick_cooldowns();
    projection.apply_experience(SetExperience {
        progress: f32::NAN,
        level: i32::MAX,
        total_experience: i32::MIN,
    });
    assert_eq!(projection.experience().display_start_tick, Some(1));

    let group = id("minecraft:test");
    projection.apply_cooldown(Cooldown {
        group: group.clone(),
        duration_ticks: 10,
    });
    assert_eq!(projection.cooldown_percentage(&group, 0.0), Some(1.0));
    projection.apply_cooldown(Cooldown {
        group: group.clone(),
        duration_ticks: 20,
    });
    projection.apply_cooldown(Cooldown {
        group: group.clone(),
        duration_ticks: 0,
    });
    assert!(!projection.cooldowns().contains_key(&group));

    projection.apply_cooldown(Cooldown {
        group: group.clone(),
        duration_ticks: -1,
    });
    assert!(projection.cooldowns().contains_key(&group));
    assert_eq!(projection.tick_cooldowns(), std::slice::from_ref(&group));
    assert!(!projection.cooldowns().contains_key(&group));
}

#[test]
fn c3_statistics_application_is_delta_replacement_with_one_optional_callback() {
    let mut projection = PlayerProjection::default();
    let jump = stat("minecraft:custom", "minecraft:jump");
    let mined = stat("minecraft:mined", "minecraft:stone");
    projection.apply_statistics(
        AwardStats {
            values: BTreeMap::from([(jump.clone(), 1), (mined.clone(), 2)]),
        },
        false,
    );
    let application = projection.apply_statistics(
        AwardStats {
            values: BTreeMap::from([(jump.clone(), -9)]),
        },
        true,
    );
    assert_eq!(application.updated, 1);
    assert!(application.screen_callback);
    assert_eq!(projection.statistics()[&jump], -9);
    assert_eq!(projection.statistics()[&mined], 2);
    assert_eq!(projection.stats_screen_callbacks(), 1);
    let empty = projection.apply_statistics(
        AwardStats {
            values: BTreeMap::new(),
        },
        true,
    );
    assert_eq!(empty.updated, 0);
    assert!(empty.screen_callback);
    assert_eq!(projection.stats_screen_callbacks(), 2);
}

#[test]
fn c3_server_publication_orders_expiry_health_and_experience_and_locks_markers() {
    let mut publisher = PlayerProjectionPublisher::default();
    assert_eq!(
        publisher.publish_tick(),
        [
            PlayerProjectionDelivery::Health(SetHealth {
                health: 20.0,
                food: 20,
                saturation: 5.0,
            }),
            PlayerProjectionDelivery::Experience(SetExperience {
                progress: 0.0,
                level: 0,
                total_experience: 0,
            }),
        ]
    );
    assert!(publisher.publish_tick().is_empty());
    publisher.set_vitals(20.0, 20, 6.0);
    assert!(publisher.publish_tick().is_empty());
    publisher.set_vitals(20.0, 19, 7.0);
    assert_eq!(
        publisher.publish_tick(),
        [PlayerProjectionDelivery::Health(SetHealth {
            health: 20.0,
            food: 19,
            saturation: 7.0,
        })]
    );
    publisher.set_vitals(f32::NAN, 19, -0.0);
    assert!(matches!(
        publisher.publish_tick().as_slice(),
        [PlayerProjectionDelivery::Health(packet)]
            if packet.health.is_nan() && packet.saturation.is_sign_negative()
    ));
    assert!(matches!(
        publisher.publish_tick().as_slice(),
        [PlayerProjectionDelivery::Health(packet)] if packet.health.is_nan()
    ));

    let group = id("minecraft:test");
    assert_eq!(publisher.start_cooldown(group.clone(), 1).duration_ticks, 1);
    assert_eq!(
        publisher.publish_tick().first(),
        Some(&PlayerProjectionDelivery::Cooldown(Cooldown {
            group,
            duration_ticks: 0,
        }))
    );

    publisher.set_experience(SetExperience {
        progress: 0.5,
        level: 7,
        total_experience: -1,
    });
    publisher.set_experience_progress(0.75);
    assert!(
        publisher
            .publish_tick()
            .iter()
            .all(|delivery| !matches!(delivery, PlayerProjectionDelivery::Experience(_)))
    );
}

#[test]
fn c3_statistics_request_drains_dirty_values_and_uses_locked_increment_arithmetic() {
    let mut publisher = PlayerProjectionPublisher::default();
    let jump = stat("minecraft:custom", "minecraft:jump");
    let mined = stat("minecraft:mined", "minecraft:stone");
    publisher.set_statistic(jump.clone(), i32::MAX - 1);
    publisher.increment_statistic(jump.clone(), 10);
    publisher.set_statistic(mined.clone(), i32::MIN);
    publisher.increment_statistic(mined.clone(), -1);
    assert_eq!(
        publisher.request_statistics().values,
        BTreeMap::from([(jump.clone(), i32::MAX), (mined.clone(), i32::MAX)])
    );
    assert!(publisher.request_statistics().values.is_empty());
    publisher.mark_all_statistics_dirty();
    assert_eq!(publisher.request_statistics().values.len(), 2);
}

#[test]
fn c3_player_projection_requires_a_level_and_then_applies_connection_locally() {
    assert_eq!(
        PlayEntryProjection::default().apply(PlayClientboundPacket::SetHealth(SetHealth {
            health: 1.0,
            food: 2,
            saturation: 3.0,
        })),
        Err(PlayProjectionError::LevelNotInstalled)
    );
    let mut projection = PlayEntryProjection::default();
    projection.apply(login()).unwrap();
    projection
        .apply(PlayClientboundPacket::SetHealth(SetHealth {
            health: 1.0,
            food: -2,
            saturation: f32::INFINITY,
        }))
        .unwrap();
    projection
        .apply(PlayClientboundPacket::SetExperience(SetExperience {
            progress: f32::NEG_INFINITY,
            level: -3,
            total_experience: -4,
        }))
        .unwrap();
    assert_eq!(projection.player_projection().health().current, 1.0);
    assert_eq!(projection.player_projection().health().food, -2);
    assert_eq!(projection.player_projection().experience().level, -3);
}
