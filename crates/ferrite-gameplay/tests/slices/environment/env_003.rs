use ferrite_gameplay::environment::lighting::{
    CHECK_ORDER, CLIENT_ALL_TASK_THRESHOLD, CLIENT_IMPORT_ORDER, CLIENT_MIN_TASK_BUDGET,
    ClientLightStage, CopperOxidation, DEFERRED_EXPERIMENT, DRAIN_ORDER, EMITTING_BLOCK_COUNT,
    EmitterState, LATENCY_POLICY, LightChannel, LightWork, MAX_LIGHT, NIBBLE_SECTION_BYTES,
    SERVER_DRAIN_STAGES, SERVER_TASK_BATCH, SKY_LIGHT_ATTRIBUTE_DEFAULT, SectionStorageTransition,
    ServerLightStage, SkyEdge, TrialSpawnerLight, affected_sections, block_source_change,
    client_light_task_budget, direct_sky_source, emission, first_write_copies_layer,
    has_different_light_properties, light_packet_plan, lowest_sky_source,
    missing_block_layer_value, propagation_candidate, publish_light, queued_layer_installed,
    raw_brightness, server_task_batch, sky_darken, sky_engine_enabled, sky_query,
};

#[test]
fn lighting_constants_channels_and_deferred_policy_are_explicit() {
    assert_eq!(
        (
            MAX_LIGHT,
            NIBBLE_SECTION_BYTES,
            SERVER_TASK_BATCH,
            CLIENT_ALL_TASK_THRESHOLD,
            CLIENT_MIN_TASK_BUDGET,
            EMITTING_BLOCK_COUNT,
            SKY_LIGHT_ATTRIBUTE_DEFAULT,
        ),
        (15, 2_048, 1_000, 1_000, 10, 109, 15.0)
    );
    assert_eq!(CHECK_ORDER, [LightChannel::Block, LightChannel::Sky]);
    assert_eq!(DRAIN_ORDER, [LightWork::Decrease, LightWork::Increase]);
    assert_eq!(LATENCY_POLICY.experiment, DEFERRED_EXPERIMENT);
    assert_eq!(
        (
            LATENCY_POLICY.universal_tick_deadline,
            LATENCY_POLICY.universal_frame_deadline,
            LATENCY_POLICY.claims_vanilla_bound
        ),
        (None, None, false)
    );
}

#[test]
fn propagation_loses_at_least_one_and_requires_strict_improvement() {
    assert_eq!(
        propagation_candidate(15, 0, 0, false)
            .expect("air propagation")
            .value,
        14
    );
    assert_eq!(
        propagation_candidate(15, 4, 0, false)
            .expect("dampened")
            .value,
        11
    );
    assert!(propagation_candidate(15, 1, 14, false).is_none());
    assert!(propagation_candidate(15, 1, 13, true).is_none());
    assert!(propagation_candidate(0, 0, 0, false).is_none());
}

#[test]
fn source_loss_enqueues_decrease_before_brighter_alternative_restore() {
    let removed = block_source_change(12, 0, 13, true);
    assert_eq!(removed.stored_value, 0);
    assert_eq!(removed.decrease_value, Some(12));
    assert_eq!(removed.increase_value, Some(13));
    let no_restore = block_source_change(12, 0, 12, true);
    assert_eq!(no_restore.increase_value, None);
    let gained = block_source_change(0, 9, 0, true);
    assert_eq!(gained.decrease_value, None);
    assert_eq!(gained.increase_value, Some(9));
    let disabled = block_source_change(7, 15, 0, false);
    assert_eq!(disabled.decrease_value, Some(7));
}

#[test]
fn server_batches_999_1000_1001_and_drains_block_before_sky() {
    assert_eq!(
        server_task_batch(999),
        ferrite_gameplay::environment::lighting::ServerTaskBatch {
            selected: 999,
            remaining: 0,
            drains_engines_completely: true
        }
    );
    assert_eq!(server_task_batch(1_000).remaining, 0);
    assert_eq!(server_task_batch(1_001).remaining, 1);
    assert_eq!(
        SERVER_DRAIN_STAGES,
        [
            ServerLightStage::PreUpdate,
            ServerLightStage::BlockDecrease,
            ServerLightStage::BlockIncrease,
            ServerLightStage::SkyDecrease,
            ServerLightStage::SkyIncrease,
            ServerLightStage::ReconcileSections,
            ServerLightStage::PublishVisible,
            ServerLightStage::PostUpdate
        ]
    );
}

#[test]
fn section_lifecycle_creates_references_and_defers_removal() {
    assert_eq!(
        ferrite_gameplay::environment::lighting::section_storage_transition(false, true),
        SectionStorageTransition {
            create_layer: true,
            defer_removal: false,
            reference_delta: 27
        }
    );
    assert_eq!(
        ferrite_gameplay::environment::lighting::section_storage_transition(true, false),
        SectionStorageTransition {
            create_layer: false,
            defer_removal: true,
            reference_delta: -27
        }
    );
    assert!(queued_layer_installed(true));
    assert!(!queued_layer_installed(false));
    assert!(first_write_copies_layer(false));
    assert!(!first_write_copies_layer(true));
}

#[test]
fn publication_copies_visible_map_and_marks_all_27_affected_sections() {
    let sections = affected_sections([4, 5, 6]);
    assert_eq!(sections.len(), 27);
    assert_eq!(sections[0], [3, 4, 5]);
    assert_eq!(sections[13], [4, 5, 6]);
    assert_eq!(sections[26], [5, 6, 7]);
    let publication = publish_light(LightChannel::Sky, [4, 5, 6]);
    assert!(publication.copy_updating_to_visible);
    assert!(publication.mark_chunk_unsaved);
    assert!(publication.packet_bit_set);
    assert_eq!(publication.affected_sections, sections);
}

#[test]
fn packet_gates_require_visible_holder_tracking_player_and_nonempty_mask() {
    assert!(light_packet_plan(true, true, true).send);
    assert!(light_packet_plan(true, true, true).clear_masks_after_send);
    for inputs in [
        (false, true, true),
        (true, false, true),
        (true, true, false),
    ] {
        let plan = light_packet_plan(inputs.0, inputs.1, inputs.2);
        assert!(!plan.send);
        assert!(!plan.clear_masks_after_send);
    }
}

#[test]
fn missing_block_and_sky_layers_keep_distinct_query_rules() {
    assert_eq!(missing_block_layer_value(None), 0);
    assert_eq!(missing_block_layer_value(Some(8)), 8);
    assert_eq!(sky_query(true, true, None, None), 15);
    assert_eq!(sky_query(false, false, Some(15), Some(12)), 0);
    assert_eq!(sky_query(true, false, Some(7), Some(12)), 7);
    assert_eq!(sky_query(true, false, None, Some(12)), 12);
    assert_eq!(sky_query(true, false, None, None), 0);
    assert!(sky_engine_enabled(true));
    assert!(!sky_engine_enabled(false));
}

#[test]
fn sky_threshold_scans_first_dampening_or_occluded_vertical_edge() {
    let edges = [
        SkyEdge {
            below_y: 20,
            below_dampening: 0,
            joined_vertical_faces_occlude: false,
        },
        SkyEdge {
            below_y: 19,
            below_dampening: 0,
            joined_vertical_faces_occlude: true,
        },
        SkyEdge {
            below_y: 18,
            below_dampening: 15,
            joined_vertical_faces_occlude: false,
        },
    ];
    assert_eq!(lowest_sky_source(&edges, -64), 20);
    assert_eq!(lowest_sky_source(&[], -64), -64);
    assert_eq!(direct_sky_source(20, 20), 15);
    assert_eq!(direct_sky_source(19, 20), 0);
}

#[test]
fn raw_brightness_and_float_darken_preserve_truncation() {
    assert_eq!(raw_brightness(Some(6), Some(15), 4), 11);
    assert_eq!(raw_brightness(Some(12), Some(15), 4), 12);
    assert_eq!(raw_brightness(None, None, 0), 0);
    assert_eq!(sky_darken(15.0), 0);
    assert_eq!(sky_darken(14.9), 0);
    assert_eq!(sky_darken(14.0), 1);
    assert_eq!(sky_darken(0.0), 15);
    assert_eq!(sky_darken(-10.0), 15);
    assert_eq!(sky_darken(20.0), 0);
}

#[test]
fn light_property_change_includes_shape_only_occluders() {
    assert!(!has_different_light_properties(0, 0, 0, 0, false, false));
    assert!(has_different_light_properties(0, 1, 0, 0, false, false));
    assert!(has_different_light_properties(0, 0, 1, 2, false, false));
    assert!(has_different_light_properties(0, 0, 0, 0, true, false));
    assert!(has_different_light_properties(0, 0, 0, 0, false, true));
}

#[test]
fn client_budget_and_import_order_lock_fifo_visibility_boundary() {
    assert_eq!(client_light_task_budget(9), 10);
    assert_eq!(client_light_task_budget(10), 10);
    assert_eq!(client_light_task_budget(999), 99);
    assert_eq!(client_light_task_budget(1_000), 1_000);
    assert_eq!(
        CLIENT_IMPORT_ORDER,
        [
            ClientLightStage::ImportSky,
            ClientLightStage::ImportBlock,
            ClientLightStage::MarkSectionsDirty,
            ClientLightStage::EnableChunk,
            ClientLightStage::DrainLighting,
            ClientLightStage::RendererUpdate
        ]
    );
}

#[test]
fn candle_bulb_campfire_and_machine_emission_rules_are_exact() {
    assert_eq!(
        emission(EmitterState::Candle {
            lit: true,
            candles: 4
        }),
        12
    );
    assert_eq!(
        emission(EmitterState::Candle {
            lit: false,
            candles: 4
        }),
        0
    );
    assert_eq!(emission(EmitterState::CandleCake { lit: true }), 3);
    let expected = [
        (CopperOxidation::Unaffected, 15),
        (CopperOxidation::Exposed, 12),
        (CopperOxidation::Weathered, 8),
        (CopperOxidation::Oxidized, 4),
    ];
    for (oxidation, light) in expected {
        assert_eq!(
            emission(EmitterState::CopperBulb {
                lit: true,
                oxidation
            }),
            light
        );
    }
    assert_eq!(
        emission(EmitterState::Campfire {
            lit: true,
            soul: false
        }),
        15
    );
    assert_eq!(
        emission(EmitterState::Campfire {
            lit: true,
            soul: true
        }),
        10
    );
    assert_eq!(emission(EmitterState::FurnaceFamily { lit: true }), 13);
}

#[test]
fn redstone_vines_pickles_anchor_and_light_block_emission_are_exact() {
    assert_eq!(emission(EmitterState::RedstoneTorch { lit: true }), 7);
    assert_eq!(emission(EmitterState::RedstoneOre { lit: true }), 9);
    assert_eq!(emission(EmitterState::CaveVines { berries: true }), 14);
    assert_eq!(
        emission(EmitterState::SeaPickle {
            waterlogged: true,
            pickles: 4
        }),
        15
    );
    assert_eq!(
        emission(EmitterState::SeaPickle {
            waterlogged: false,
            pickles: 4
        }),
        0
    );
    for (charges, light) in [(0, 0), (1, 3), (2, 7), (3, 11), (4, 15)] {
        assert_eq!(emission(EmitterState::RespawnAnchor { charges }), light);
    }
    assert_eq!(emission(EmitterState::LightBlock { level: 15 }), 15);
}

#[test]
fn lichen_lamp_trial_spawner_and_vault_state_emission_are_exact() {
    assert_eq!(emission(EmitterState::GlowLichen { any_face: true }), 7);
    assert_eq!(emission(EmitterState::GlowLichen { any_face: false }), 0);
    assert_eq!(emission(EmitterState::RedstoneLamp { lit: true }), 15);
    assert_eq!(
        emission(EmitterState::TrialSpawner(TrialSpawnerLight::Inactive)),
        0
    );
    assert_eq!(
        emission(EmitterState::TrialSpawner(TrialSpawnerLight::Cooldown)),
        0
    );
    assert_eq!(
        emission(EmitterState::TrialSpawner(
            TrialSpawnerLight::WaitingForPlayers
        )),
        4
    );
    assert_eq!(
        emission(EmitterState::TrialSpawner(TrialSpawnerLight::Active)),
        8
    );
    assert_eq!(emission(EmitterState::Vault { inactive: true }), 6);
    assert_eq!(emission(EmitterState::Vault { inactive: false }), 12);
}

#[test]
fn static_emitter_families_cover_each_locked_light_level() {
    for (path, light) in [
        ("brewing_stand", 1),
        ("firefly_bush", 2),
        ("magma_block", 3),
        ("large_amethyst_bud", 4),
        ("amethyst_cluster", 5),
        ("sculk_catalyst", 6),
        ("enchanting_table", 7),
        ("soul_lantern", 10),
        ("nether_portal", 11),
        ("torch", 14),
        ("beacon", 15),
        ("waxed_oxidized_copper_lantern", 15),
        ("ochre_froglight", 15),
        ("stone", 0),
    ] {
        assert_eq!(emission(EmitterState::Static(path)), light, "{path}");
    }
}
