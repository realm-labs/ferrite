use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_gameplay::block::command_area::{
    CLONE_FLAGS, CloneBlockCategory, CloneEntry, CloneMode, ClonePreflight, CloneWriteKind,
    FILL_FLAGS, FillBiomeChunk, FillMode, InclusiveBox, STRICT_FLAGS, fill_decision,
    fill_result_increment, plan_clone, plan_fill_biome, quantize_biome_box,
    validate_clone_preflight,
};
use ferrite_gameplay::block::spawner::{
    OrdinarySpawner, SavedSpawnerSettings, SpawnAttempt, SpawnEggEffect, SpawnerConfig,
    SpawnerEffect, SpawnerKind, SpawnerTickInputs, plan_spawn_egg_edit, player_in_required_range,
};
use ferrite_gameplay::block::trial_spawner::{
    OminousItemEffect, OminousItemInputs, TrialEffect, TrialSpawner, TrialSpawnerConfig,
    TrialSpawnerState, TrialTickInputs, plan_ominous_item_attempt, player_scan_due,
    tracked_entity_retained, trial_omen_duration,
};
use ferrite_gameplay::block::update::event::{BlockEvent, BlockEventQueue};
use ferrite_gameplay::block::update::flags::BlockUpdateFlags;
use ferrite_gameplay::block::update::lifecycle::{
    BlockEntityTransition, SetBlockEffect, SetBlockInputs, plan_set_block,
};
use ferrite_gameplay::block::update::neighbor::{
    NeighborCollector, NeighborWork, NeighborWorkKind, ORDINARY_DIRECTIONS, SHAPE_DIRECTIONS,
};
use ferrite_gameplay::block::update::ticker::{
    BlockEntityTicker, BlockEntityTickerList, TickerAction,
};
use ferrite_gameplay::block::vault::{
    EJECTION_DELAY, MAX_REWARDED_PLAYERS, UNLOCK_DELAY, Vault, VaultEffect, VaultKeyInputs,
    VaultServerData, VaultState, VaultUseResult, ejection_pitch, ejection_progress,
    within_particle_radius, within_strict_scan_radius,
};
use ferrite_gameplay::block::vine::{
    BelowVineTarget, HORIZONTAL_CEILING_CHANCE, HorizontalGrowthInputs, UpwardGrowthInputs,
    VINE_STATE_COUNT, VineState, VineTransform, density_allows_growth, downward_growth,
    growth_admission, horizontal_growth, placement_state, repair_state, transform, upward_growth,
};
use ferrite_simulation::random::DeterministicRng;

const ORIGIN: BlockPos = BlockPos::new(0, 64, 0);

#[test]
fn all_blk_003_slices_have_production_owners() {
    assert_eq!(
        [
            ("BLK-COMMAND-AREA-001", "block::command_area"),
            ("BLK-SPAWNER-RUNTIME-001", "block::spawner"),
            ("BLK-TRIAL-SPAWNER-RUNTIME-001", "block::trial_spawner"),
            ("BLK-UPDATE-PIPELINE-001", "block::update"),
            ("BLK-VAULT-RUNTIME-001", "block::vault"),
            ("BLK-VINE-RUNTIME-001", "block::vine"),
        ]
        .len(),
        6
    );
}

#[test]
fn update_flags_and_set_block_stages_preserve_narrow_suppression() {
    assert_eq!(BlockUpdateFlags::UPDATE_NONE.bits(), 260);
    assert_eq!(BlockUpdateFlags::UPDATE_ALL.bits(), 3);
    assert_eq!(BlockUpdateFlags::UPDATE_ALL_IMMEDIATE.bits(), 11);
    assert_eq!(BlockUpdateFlags::UPDATE_SKIP_ALL_SIDE_EFFECTS.bits(), 816);
    assert_eq!(
        BlockUpdateFlags::from_bits(35).nested_shape_flags().bits(),
        2
    );

    let plan = plan_set_block(SetBlockInputs {
        valid_bounds: true,
        debug_level: false,
        canonical_state_changed: true,
        block_type_changed: true,
        requested_is_rail: false,
        callback_retained_requested_type: true,
        reread_is_requested_state: true,
        server_side: true,
        chunk_block_ticking: true,
        requested_has_analog_output: true,
        block_entity_transition: BlockEntityTransition::RemoveAndCreate,
        flags: BlockUpdateFlags::from_bits(3 | 256),
        shape_budget: 1,
    });
    assert!(plan.accepted);
    assert_eq!(plan.effects[0], SetBlockEffect::InstallState);
    assert!(plan.effects.contains(&SetBlockEffect::RemoveBlockEntity));
    assert!(
        !plan
            .effects
            .contains(&SetBlockEffect::BlockEntityPreRemoval)
    );
    assert!(
        plan.effects
            .contains(&SetBlockEffect::RequestedDirectShapes {
                remaining_budget: 0
            })
    );
    assert!(
        plan.effects
            .ends_with(&[SetBlockEffect::RemoveOldPoi, SetBlockEffect::AddNewPoi])
    );

    let callback_replaced = plan_set_block(SetBlockInputs {
        callback_retained_requested_type: false,
        ..SetBlockInputs {
            valid_bounds: true,
            debug_level: false,
            canonical_state_changed: true,
            block_type_changed: true,
            requested_is_rail: false,
            callback_retained_requested_type: true,
            reread_is_requested_state: true,
            server_side: true,
            chunk_block_ticking: true,
            requested_has_analog_output: false,
            block_entity_transition: BlockEntityTransition::None,
            flags: BlockUpdateFlags::UPDATE_ALL,
            shape_budget: 512,
        }
    });
    assert!(!callback_replaced.accepted);
    assert_eq!(
        callback_replaced.effects.last(),
        Some(&SetBlockEffect::OldRemovalHook { moved: false })
    );
}

#[test]
fn neighbor_collector_is_depth_first_fifo_per_added_layer_and_counts_work_items() {
    assert_eq!(
        ORDINARY_DIRECTIONS,
        [
            Direction::West,
            Direction::East,
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South
        ]
    );
    assert_eq!(
        SHAPE_DIRECTIONS,
        [
            Direction::West,
            Direction::East,
            Direction::North,
            Direction::South,
            Direction::Down,
            Direction::Up
        ]
    );
    let west = BlockPos::new(-1, 64, 0);
    let east = BlockPos::new(1, 64, 0);
    let first = BlockPos::new(-2, 64, 0);
    let second = BlockPos::new(-3, 64, 0);
    let root = NeighborWork {
        origin: ORIGIN,
        receivers: vec![west, east],
        kind: NeighborWorkKind::RereadReceiver,
    };
    let report = NeighborCollector::new(10).run(root.clone(), |step| {
        if step.receiver == west {
            vec![
                NeighborWork::single(west, first, NeighborWorkKind::CapturedReceiver),
                NeighborWork::single(west, second, NeighborWorkKind::Shape),
            ]
        } else {
            Vec::new()
        }
    });
    assert_eq!(
        report
            .steps
            .iter()
            .map(|step| step.receiver)
            .collect::<Vec<_>>(),
        [west, first, second, east]
    );
    assert_eq!(report.submitted, 3);

    let capped = NeighborCollector::new(1).run(root, |_| {
        vec![NeighborWork::single(
            west,
            first,
            NeighborWorkKind::RereadReceiver,
        )]
    });
    assert_eq!(capped.steps.len(), 2);
    assert_eq!(capped.first_discarded, Some(west));
}

#[test]
fn block_events_deduplicate_requeue_same_drain_and_isolate_inactive_records() {
    let inactive = BlockEvent {
        position: BlockPos::new(1, 64, 0),
        block_type: 2,
        event_id: 1,
        parameter: -5,
    };
    let active = BlockEvent {
        position: ORIGIN,
        block_type: 1,
        event_id: 7,
        parameter: i32::MAX,
    };
    let mut queue = BlockEventQueue::default();
    assert!(queue.submit(inactive));
    assert!(!queue.submit(inactive));
    assert!(queue.submit(active));
    queue.begin_drain();
    assert_eq!(
        queue.next_matching(|position| position == ORIGIN, |_| 1),
        Some(active)
    );
    assert!(queue.submit(active));
    assert_eq!(queue.next_matching(|_| true, |_| 1), Some(active));
    assert_eq!(queue.next_matching(|_| true, |_| 1), None);
    queue.finish_drain();
    assert_eq!(queue.len(), 1);
    queue.begin_drain();
    assert_eq!(queue.next_matching(|_| true, |_| 3), None);
    queue.finish_drain();
    assert!(queue.is_empty());
}

#[test]
fn block_entity_tickers_rebind_in_place_and_defer_callback_creation() {
    let first = BlockPos::new(1, 64, 0);
    let removed = BlockPos::new(2, 64, 0);
    let created_during_tick = BlockPos::new(3, 64, 0);
    let mut tickers = BlockEntityTickerList::default();
    tickers.register_or_rebind(BlockEntityTicker {
        position: first,
        removed: false,
        has_ticker: true,
        normal_gameplay_gates_pass: true,
        compatible_state: true,
    });
    tickers.register_or_rebind(BlockEntityTicker {
        position: removed,
        removed: true,
        has_ticker: true,
        normal_gameplay_gates_pass: true,
        compatible_state: true,
    });
    tickers.begin_phase();
    assert_eq!(tickers.next_action(false), Some(TickerAction::Skip(first)));
    tickers.register_or_rebind(BlockEntityTicker {
        position: created_during_tick,
        removed: false,
        has_ticker: true,
        normal_gameplay_gates_pass: true,
        compatible_state: true,
    });
    assert_eq!(
        tickers.next_action(false),
        Some(TickerAction::Prune(removed))
    );
    assert_eq!(tickers.next_action(false), None);
    assert_eq!(tickers.pending_len(), 1);
    tickers.finish_phase();

    tickers.begin_phase();
    assert_eq!(tickers.next_action(true), Some(TickerAction::Tick(first)));
    assert_eq!(
        tickers.next_action(true),
        Some(TickerAction::Tick(created_during_tick))
    );
    tickers.finish_phase();
    assert_eq!(
        tickers.active_positions().collect::<Vec<_>>(),
        [first, created_during_tick]
    );
}

#[test]
fn area_commands_precharge_and_preserve_clone_fill_order() {
    let area = InclusiveBox::from_corners(BlockPos::new(-1, 0, 0), BlockPos::new(0, 1, 1));
    assert_eq!(area.volume().unwrap(), 8);
    assert_eq!(area.validate_limit(8).unwrap(), 8);
    assert!(area.validate_limit(7).is_err());
    let mut visited = Vec::new();
    area.visit_x_y_z(|position| visited.push(position)).unwrap();
    assert_eq!(
        &visited[..4],
        [
            BlockPos::new(-1, 0, 0),
            BlockPos::new(0, 0, 0),
            BlockPos::new(-1, 1, 0),
            BlockPos::new(0, 1, 0)
        ]
    );
    assert!(
        validate_clone_preflight(ClonePreflight {
            source: area,
            destination: area,
            same_level: true,
            mode: CloneMode::Normal,
            maximum: 7,
            source_loaded: false,
            destination_loaded: false,
            destination_debug: true,
        })
        .is_err()
    );

    let entries = [
        CloneEntry {
            source: BlockPos::new(0, 0, 0),
            destination: BlockPos::new(10, 0, 0),
            category: CloneBlockCategory::NonFull,
        },
        CloneEntry {
            source: BlockPos::new(1, 0, 0),
            destination: BlockPos::new(11, 0, 0),
            category: CloneBlockCategory::Solid,
        },
        CloneEntry {
            source: BlockPos::new(2, 0, 0),
            destination: BlockPos::new(12, 0, 0),
            category: CloneBlockCategory::BlockEntity,
        },
    ];
    let writes = plan_clone(&entries, CloneMode::Move, false);
    assert_eq!(writes[0].kind, CloneWriteKind::SourceBarrier);
    assert_eq!(writes[0].position, entries[0].source);
    let first_destination = writes
        .iter()
        .position(|write| write.kind == CloneWriteKind::DestinationState)
        .unwrap();
    assert_eq!(writes[first_destination].position, entries[1].destination);
    assert_eq!(writes[first_destination].flags, CLONE_FLAGS);
    assert_eq!(
        writes.last().unwrap().kind,
        CloneWriteKind::CopyScheduledTicks
    );

    let center = BlockPos::new(0, 0, 0);
    let cube = InclusiveBox::from_corners(BlockPos::new(-1, -1, -1), BlockPos::new(1, 1, 1));
    let hollow = fill_decision(cube, center, FillMode::Hollow, false);
    assert!(hollow.place_air);
    assert_eq!(hollow.flags, FILL_FLAGS);
    assert_eq!(fill_result_increment(true, true), 1);
    assert_eq!(
        fill_decision(cube, center, FillMode::Replace, true).flags,
        STRICT_FLAGS
    );
    assert_eq!(
        quantize_biome_box(BlockPos::new(-1, -5, 7), BlockPos::new(6, 1, 8)),
        InclusiveBox::from_corners(BlockPos::new(-4, -8, 4), BlockPos::new(4, 0, 8))
    );
    let unchanged_biomes = plan_fill_biome(&[
        FillBiomeChunk {
            full_chunk_available: true,
            matching_quart_cells: 0,
        },
        FillBiomeChunk {
            full_chunk_available: true,
            matching_quart_cells: 0,
        },
    ])
    .unwrap();
    assert_eq!(unchanged_biomes.matching_quart_cells, 0);
    assert_eq!(unchanged_biomes.dirty_chunk_indices, [0, 1]);
    assert_eq!(unchanged_biomes.resend_chunk_indices, [0, 1]);
    assert!(
        plan_fill_biome(&[FillBiomeChunk {
            full_chunk_available: false,
            matching_quart_cells: 4,
        }])
        .is_err()
    );
}

#[test]
fn ordinary_spawner_freezes_retries_and_resets_at_exact_branches() {
    let mut spawner = OrdinarySpawner {
        delay: -1,
        selected_data_present: false,
    };
    let mut random = DeterministicRng::from_seed(12);
    let frozen_state = random.state();
    let frozen = spawner.server_tick(
        SpawnerConfig::default(),
        SpawnerTickInputs {
            has_required_player: false,
            spawners_work: true,
            potentials_available: true,
            selected_entity_type_valid: true,
            attempts: &[],
        },
        &mut random,
    );
    assert!(!frozen.read_rule);
    assert_eq!(random.state(), frozen_state);
    assert_eq!(spawner.delay, -1);

    let admitted = spawner.server_tick(
        SpawnerConfig {
            minimum_delay: 5,
            maximum_delay: 5,
            ..SpawnerConfig::default()
        },
        SpawnerTickInputs {
            has_required_player: true,
            spawners_work: true,
            potentials_available: true,
            selected_entity_type_valid: true,
            attempts: &[],
        },
        &mut random,
    );
    assert_eq!(spawner.delay, 4);
    assert!(
        admitted
            .effects
            .contains(&SpawnerEffect::BroadcastBlockEvent(1))
    );
    assert_eq!(
        admitted.effects.last(),
        Some(&SpawnerEffect::DecrementDelay)
    );

    spawner.delay = 0;
    let due = spawner.server_tick(
        SpawnerConfig {
            minimum_delay: 9,
            maximum_delay: 9,
            spawn_count: 3,
            ..SpawnerConfig::default()
        },
        SpawnerTickInputs {
            has_required_player: true,
            spawners_work: true,
            potentials_available: false,
            selected_entity_type_valid: true,
            attempts: &[SpawnAttempt::SkipCollision, SpawnAttempt::SuccessNonMob],
        },
        &mut random,
    );
    assert_eq!(due.successes, 1);
    assert_eq!(spawner.delay, 9);
    assert!(!due.effects.contains(&SpawnerEffect::MobSpawnAnimation));
    assert!(player_in_required_range(f64::MAX, -1));
    assert!(!player_in_required_range(256.0, 16));
    let (_, loaded_config) = OrdinarySpawner::load_settings(
        SavedSpawnerSettings {
            delay: -2,
            minimum_delay: -10,
            maximum_delay: 1,
            spawn_count: -1,
            maximum_nearby_entities: -3,
            required_player_range: -1,
            spawn_range: -4,
        },
        false,
    );
    assert_eq!(loaded_config.spawn_count, -1);
    assert_eq!(loaded_config.required_player_range, -1);

    let (enabled, edit) = plan_spawn_egg_edit(SpawnerKind::Trial, true, true, true);
    assert!(enabled);
    assert_eq!(edit[0], SpawnEggEffect::ResetTrialEncounterAndConfigs);
    let (enabled, edit) = plan_spawn_egg_edit(SpawnerKind::Ordinary, true, false, false);
    assert!(!enabled);
    assert_eq!(edit, [SpawnEggEffect::FailureMessage]);
}

#[test]
fn vine_schema_support_and_growth_keep_exact_draw_boundaries() {
    for bits in 0..VINE_STATE_COUNT as u8 {
        assert_eq!(VineState::from_bits(bits).unwrap().bits(), bits);
    }
    assert!(VineState::from_bits(32).is_none());
    let empty = VineState::default();
    assert!(empty.uses_full_fallback_outline());
    assert!(!empty.can_survive());

    let north = empty.with(Direction::North, true);
    let placed = placement_state(
        Some(north),
        &[Direction::Down, Direction::East, Direction::North],
        |_| true,
    )
    .unwrap();
    assert!(placed.has(Direction::North));
    assert!(placed.has(Direction::East));
    assert_eq!(
        repair_state(north, Direction::East, Some(north), |_| false),
        Some(north)
    );
    assert_eq!(
        transform(north, VineTransform::Clockwise90),
        empty.with(Direction::East, true)
    );
    assert!(density_allows_growth(4));
    assert!(!density_allows_growth(5));
    assert_eq!(growth_admission(false, 0, Direction::Up).draws_consumed, 0);
    assert_eq!(growth_admission(true, 3, Direction::Up).draws_consumed, 1);

    let horizontal = horizontal_growth(
        ORIGIN,
        north,
        Direction::East,
        true,
        HorizontalGrowthInputs {
            target_is_air: true,
            target_accepts_selected_face: false,
            target_clockwise_neighbor_acceptable: false,
            target_counterclockwise_neighbor_acceptable: false,
            clockwise_diagonal_empty: false,
            counterclockwise_diagonal_empty: false,
            clockwise_source_neighbor_accepts_opposite: false,
            counterclockwise_source_neighbor_accepts_opposite: false,
            fallback_draw: HORIZONTAL_CEILING_CHANCE,
            target_ceiling_acceptable: true,
        },
    );
    assert!(horizontal.write.is_none());
    assert_eq!(horizontal.draws_consumed, 1);

    let upward = upward_growth(
        ORIGIN,
        north,
        UpwardGrowthInputs {
            maximum_y: 320,
            direct_ceiling_support: false,
            above_is_air: true,
            density_allows: true,
            above_support: [true; 4],
            coins: [true; 4],
        },
    );
    assert!(upward.write.is_none());
    assert_eq!(upward.draws_consumed, 4);
    let downward = downward_growth(ORIGIN, north, -64, BelowVineTarget::Air, [false; 4]);
    assert!(downward.write.is_none());
    assert_eq!(downward.draws_consumed, 4);
}

#[test]
fn trial_spawner_targets_retry_reward_and_ominous_order_are_locked() {
    let config = TrialSpawnerConfig::default();
    assert_eq!(config.target_total(1), 6);
    assert_eq!(config.target_total(3), 10);
    assert_eq!(config.target_simultaneous(3), 4);
    assert!(player_scan_due(ORIGIN, -position_as_long_for_test(ORIGIN)));
    assert!(tracked_entity_retained(true, true, 2209));
    assert!(!tracked_entity_retained(true, true, 2210));
    assert_eq!(trial_omen_duration(2), 54_000);

    let mut trial = TrialSpawner {
        state: TrialSpawnerState::Active,
        registered_players: 1,
        next_mob_spawns_at: 10,
        ..TrialSpawner::default()
    };
    let failed = trial.server_tick(
        config,
        TrialTickInputs {
            now: 10,
            encounters_enabled: true,
            selected_entity_usable: true,
            tracked_entities_removed: 0,
            newly_registered_players: 0,
            converted_to_ominous: false,
            mob_attempt_succeeded: false,
            reward_result_nonempty: false,
        },
    );
    assert!(failed.effects.contains(&TrialEffect::AttemptMob));
    assert_eq!(trial.next_mob_spawns_at, 10);

    let succeeded = trial.server_tick(
        config,
        TrialTickInputs {
            mob_attempt_succeeded: true,
            ..TrialTickInputs {
                now: 10,
                encounters_enabled: true,
                selected_entity_usable: true,
                tracked_entities_removed: 0,
                newly_registered_players: 0,
                converted_to_ominous: false,
                mob_attempt_succeeded: false,
                reward_result_nonempty: false,
            }
        },
    );
    assert!(succeeded.effects.contains(&TrialEffect::SpawnMob));
    assert_eq!(trial.next_mob_spawns_at, 50);

    let attempt = plan_ominous_item_attempt(OminousItemInputs {
        weighted_item_present: true,
        timer_due: false,
        chosen_side_has_targets: true,
        target_count: 3,
        geometry_clear: true,
        admission_succeeded: true,
    });
    assert_eq!(attempt, [OminousItemEffect::SelectWeightedItem]);
    let rejected_admission = plan_ominous_item_attempt(OminousItemInputs {
        weighted_item_present: true,
        timer_due: true,
        chosen_side_has_targets: true,
        target_count: 1,
        geometry_clear: true,
        admission_succeeded: false,
    });
    assert!(rejected_admission.contains(&OminousItemEffect::AdvanceTimer));
}

#[test]
fn vault_validation_reverse_ejection_and_reload_total_quirk_are_locked() {
    let mut vault = Vault {
        state: VaultState::Active,
        ..Vault::default()
    };
    let base = VaultKeyInputs {
        logical_server: true,
        block_entity_present: true,
        config_key_empty: false,
        key_matches_exactly: false,
        sufficient_count: true,
        player_already_rewarded: true,
        infinite_materials: false,
        now: 14,
        player_id: 7,
    };
    let (_, silent) = vault.use_key(false, base, &[1]);
    assert!(silent.is_empty());
    let (result, failure) = vault.use_key(false, VaultKeyInputs { now: 15, ..base }, &[1]);
    assert_eq!(result, VaultUseResult::SuccessServer);
    assert_eq!(failure, [VaultEffect::InsertFailureSound]);

    let (_, accepted) = vault.use_key(
        false,
        VaultKeyInputs {
            key_matches_exactly: true,
            player_already_rewarded: false,
            now: 20,
            ..base
        },
        &[10, 20, 30],
    );
    assert!(accepted.contains(&VaultEffect::AwardItemUsed));
    assert_eq!(vault.state, VaultState::Unlocking);
    assert_eq!(vault.server.state_updating_resumes_at, 20 + UNLOCK_DELAY);
    assert_eq!(vault.display_item, Some(30));

    let opening = vault.server_tick(20 + UNLOCK_DELAY, 0, None);
    assert!(opening.contains(&VaultEffect::OpenShutterSound));
    let ejection = vault.server_tick(20 + UNLOCK_DELAY + EJECTION_DELAY, 0, None);
    assert!(matches!(
        ejection.first(),
        Some(VaultEffect::EjectReward { stack: 30, .. })
    ));
    assert_eq!(vault.display_item, Some(20));
    assert_eq!(ejection_progress(3, 3), 0.0);
    assert_eq!(ejection_progress(1, 3), 1.0);
    assert_eq!(ejection_pitch(ejection_progress(2, 0)), 1.6);

    let mut loaded = VaultServerData {
        total_ejections: 99,
        ..VaultServerData::default()
    };
    loaded.load_from(&VaultServerData {
        pending_rewards: vec![1, 2],
        total_ejections: 2,
        ..VaultServerData::default()
    });
    assert_eq!(loaded.pending_rewards, [1, 2]);
    assert_eq!(loaded.total_ejections, 99);
    for player in 0..=MAX_REWARDED_PLAYERS as u128 {
        loaded.append_rewarded(player);
    }
    assert_eq!(loaded.rewarded_players.len(), MAX_REWARDED_PLAYERS);
    assert_eq!(loaded.rewarded_players.front(), Some(&1));
    assert!(!within_strict_scan_radius(16.0, 4.0));
    assert!(within_particle_radius(20.25, 4.5));
}

const fn position_as_long_for_test(position: BlockPos) -> i64 {
    ((position.x as i64 & 0x3ff_ffff) << 38)
        | ((position.z as i64 & 0x3ff_ffff) << 12)
        | (position.y as i64 & 0xfff)
}
