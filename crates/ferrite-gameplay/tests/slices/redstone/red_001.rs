use ferrite_foundation::direction::Direction;
use ferrite_gameplay::redstone::comparator::{
    BLOCK_ENTITY_SETTER_MARKS_CHANGED, CLICK_VOLUME, COMPARATOR_SHAPE_HEIGHT, COMPARE_PITCH,
    ComparatorMode, ComparatorState, EXPERIMENTAL_ORIENTATION_BIAS, EXPERIMENTAL_ORIENTATION_BOUND,
    InteractionResult, ItemFrameSample, NEIGHBOR_DELAY, OUTPUT_NOTIFICATION_ORDER,
    OUTPUT_SIGNAL_KEY, OrientationBias, OutputNotification, PLACEMENT_DELAY, REFRESH_ORDER,
    RearInputProbe, RefreshStage, STATE_WRITE_FLAGS, SUBTRACT_PITCH, SoundRecipients, TickPriority,
    comparator_calculation, comparator_signal, comparator_use, forward_block_event,
    loaded_output_signal, neighbor_check, neighbor_priority, placement_schedule, rear_input,
    refresh_plan, removal_notifies_output, side_input, support_loss_plan,
};
use ferrite_gameplay::redstone::daylight_detector::{
    BLOCK_ENTITY_HAS_DATA, BLOCK_ENTITY_HAS_RENDERER, BLOCK_ENTITY_HAS_UPDATE_PACKET,
    DEGREE_TO_RADIAN, DaylightInteractionResult, DaylightState, INVERT_WRITE_FLAGS,
    POWER_WRITE_FLAGS, SHAPE_HEIGHT, SUN_SMOOTHING, TICK_PERIOD, daylight_formula, daylight_update,
    daylight_use, direct_signal, ordinary_signal, periodic_tick_admitted, ticker_installed,
};
use ferrite_gameplay::redstone::signal::{
    BEST_NEIGHBOR_ORDER, ControlSource, DIRECT_SIGNAL_ORDER, MAX_SIGNAL, SignalSample,
    WIRE_NOTIFICATION_SET_SIZE, WIRE_PLACEMENT_ORDER, WIRE_POWER_WRITE_FLAGS, WIRE_RECOMPUTE_ORDER,
    WIRE_REMOVAL_ORDER, WireConnection, WireConnectionProbe, WireEvaluator, WireLifecycleStage,
    WireRecomputeStage, WireRemovalStage, WireRoute, WireShape, aggregate_signal, combined_signal,
    control_input, default_wire_power, normalized_placement_shape, selected_evaluator,
    toggled_player_shape, wire_connection, wire_neighbor_plan, wire_power_commit,
};

fn no_route() -> WireRoute {
    WireRoute {
        same_height: None,
        neighbor_conductor: false,
        above_neighbor: None,
        below_neighbor: None,
    }
}

#[test]
fn ordinary_direct_conductor_and_control_input_matrices_are_distinct() {
    assert_eq!(
        combined_signal(SignalSample {
            ordinary: 7,
            direct_into_block: 12,
            conductor: true,
        }),
        12
    );
    assert_eq!(
        combined_signal(SignalSample {
            ordinary: 7,
            direct_into_block: 12,
            conductor: false,
        }),
        7
    );
    assert_eq!(control_input(ControlSource::RedstoneBlock, false), 15);
    assert_eq!(control_input(ControlSource::RedstoneBlock, true), 0);
    assert_eq!(control_input(ControlSource::Wire(13), false), 13);
    assert_eq!(control_input(ControlSource::Wire(13), true), 0);
    assert_eq!(control_input(ControlSource::Diode(9), true), 9);
    assert_eq!(
        control_input(ControlSource::OtherSignalSource(11), false),
        11
    );
    assert_eq!(control_input(ControlSource::Other, false), 0);
}

#[test]
fn direct_and_best_neighbor_aggregation_use_locked_order_and_early_exit() {
    assert_eq!(DIRECT_SIGNAL_ORDER, Direction::ALL);
    assert_eq!(BEST_NEIGHBOR_ORDER, Direction::ALL);
    for hit in 0..6 {
        let mut samples = [0; 6];
        samples[hit] = MAX_SIGNAL;
        let result = aggregate_signal(samples);
        assert_eq!(result.signal, 15);
        assert_eq!(result.probes, hit as u8 + 1);
    }
    assert_eq!(aggregate_signal([1, 4, 3, 8, 2, 7]).signal, 8);
    assert_eq!(aggregate_signal([1, 4, 3, 8, 2, 7]).probes, 6);
}

#[test]
fn default_wire_power_disables_self_and_routes_level_up_or_down_with_loss_one() {
    let block_fifteen = default_wire_power(15, false, [no_route(); 4]);
    assert_eq!(block_fifteen.power, 15);
    assert_eq!(block_fifteen.horizontal_routes_sampled, 0);
    assert!(block_fifteen.returned_on_block_fifteen);

    let routes = [
        WireRoute {
            same_height: Some(4),
            neighbor_conductor: true,
            above_neighbor: Some(15),
            below_neighbor: Some(2),
        },
        WireRoute {
            same_height: Some(8),
            neighbor_conductor: false,
            above_neighbor: Some(14),
            below_neighbor: Some(12),
        },
        no_route(),
        no_route(),
    ];
    assert_eq!(default_wire_power(3, false, routes).power, 14);
    assert_eq!(default_wire_power(3, true, routes).power, 11);
    assert_eq!(default_wire_power(14, true, [no_route(); 4]).power, 14);
    assert_eq!(default_wire_power(0, true, [no_route(); 4]).power, 0);
}

#[test]
fn default_wire_commit_is_exact_state_guarded_and_notifications_are_unordered() {
    assert_eq!(
        WIRE_RECOMPUTE_ORDER,
        [
            WireRecomputeStage::DisableOwnSignal,
            WireRecomputeStage::BestBlockSignal,
            WireRecomputeStage::RestoreOwnSignal,
            WireRecomputeStage::AdjacentWireSignal,
            WireRecomputeStage::GuardedPowerWrite,
            WireRecomputeStage::NeighborSetDispatch,
        ]
    );
    let changed = wire_power_commit(3, 9, true);
    assert_eq!(changed.offered_power, Some(9));
    assert_eq!(changed.write_flags, Some(WIRE_POWER_WRITE_FLAGS));
    assert_eq!(
        changed.unordered_notification_set_size,
        WIRE_NOTIFICATION_SET_SIZE
    );
    assert_eq!(wire_power_commit(3, 9, false).offered_power, None);
    assert_eq!(wire_power_commit(3, 3, true).offered_power, None);
    assert_eq!(selected_evaluator(false), WireEvaluator::Default);
    assert_eq!(selected_evaluator(true), WireEvaluator::Experimental);
}

#[test]
fn wire_connections_shapes_and_lifecycle_keep_separate_transactions() {
    let upward = WireConnectionProbe {
        top_open: true,
        neighbor_routes_up: true,
        above_neighbor_connects: true,
        neighbor_face_sturdy: true,
        neighbor_is_wire: false,
        repeater_on_axis: false,
        observer_facing_wire: false,
        neighbor_signal_source: false,
        direction_supplied: false,
        neighbor_conductor: true,
        dust_below_neighbor: false,
    };
    assert_eq!(wire_connection(upward), WireConnection::Up);
    assert_eq!(
        wire_connection(WireConnectionProbe {
            top_open: false,
            neighbor_is_wire: true,
            ..upward
        }),
        WireConnection::Side
    );
    assert_eq!(
        wire_connection(WireConnectionProbe {
            top_open: false,
            neighbor_conductor: false,
            dust_below_neighbor: true,
            ..upward
        }),
        WireConnection::Side
    );
    assert_eq!(
        wire_connection(WireConnectionProbe {
            top_open: false,
            neighbor_routes_up: false,
            above_neighbor_connects: false,
            neighbor_face_sturdy: false,
            neighbor_conductor: true,
            ..upward
        }),
        WireConnection::None
    );
    assert_eq!(normalized_placement_shape(false), WireShape::Cross);
    assert_eq!(
        toggled_player_shape(WireShape::Cross, true),
        Some(WireShape::Dot)
    );
    assert_eq!(
        toggled_player_shape(WireShape::Dot, true),
        Some(WireShape::Cross)
    );
    assert_eq!(toggled_player_shape(WireShape::Connected, true), None);
    assert_eq!(toggled_player_shape(WireShape::Dot, false), None);

    assert_eq!(
        WIRE_PLACEMENT_ORDER,
        [
            WireLifecycleStage::RecomputePower,
            WireLifecycleStage::VerticalNeighbors,
            WireLifecycleStage::HorizontalCorners,
        ]
    );
    assert_eq!(
        WIRE_REMOVAL_ORDER,
        [
            WireRemovalStage::SixNeighbors,
            WireRemovalStage::RecomputeOldStateWithoutShape,
            WireRemovalStage::HorizontalCorners,
        ]
    );
}

#[test]
fn wire_neighbor_callbacks_drop_unsupported_and_only_experimental_self_suppresses() {
    let lost = wire_neighbor_plan(false, false, WireEvaluator::Default);
    assert!(lost.drop_and_remove);
    assert!(!lost.recompute);
    let ordinary = wire_neighbor_plan(true, true, WireEvaluator::Default);
    assert!(ordinary.recompute);
    let experimental = wire_neighbor_plan(true, true, WireEvaluator::Experimental);
    assert!(!experimental.recompute);
    assert!(experimental.suppressed_experimental_self_callback);
}

#[test]
fn comparator_rear_immediate_signal_wire_and_analog_replacement_are_exact() {
    let frames = [];
    let wire = rear_input(RearInputProbe {
        facing: Direction::North,
        immediate_signal: 7,
        immediate_wire_power: Some(12),
        immediate_analog: None,
        immediate_conductor: false,
        second_analog: None,
        frames: &frames,
    });
    assert_eq!(wire.input, 12);
    assert!(wire.wire_queried);

    let analog = rear_input(RearInputProbe {
        facing: Direction::North,
        immediate_signal: 15,
        immediate_wire_power: Some(3),
        immediate_analog: Some(4),
        immediate_conductor: true,
        second_analog: Some(14),
        frames: &frames,
    });
    assert_eq!(analog.input, 4);
    assert!(!analog.wire_queried);
    assert!(analog.immediate_analog_replaced);
    assert!(!analog.second_position_queried);
}

#[test]
fn comparator_second_analog_and_exactly_one_matching_frame_replace_conductor_input() {
    let one = [ItemFrameSample {
        attachment: Direction::East,
        has_item: true,
        rotation: 9,
    }];
    let frame = rear_input(RearInputProbe {
        facing: Direction::East,
        immediate_signal: 10,
        immediate_wire_power: None,
        immediate_analog: None,
        immediate_conductor: true,
        second_analog: Some(1),
        frames: &one,
    });
    assert_eq!(frame.frame_candidate, Some(2));
    assert_eq!(frame.input, 2);

    let empty = [ItemFrameSample {
        has_item: false,
        ..one[0]
    }];
    assert_eq!(
        rear_input(RearInputProbe {
            frames: &empty,
            second_analog: None,
            ..RearInputProbe {
                facing: Direction::East,
                immediate_signal: 10,
                immediate_wire_power: None,
                immediate_analog: None,
                immediate_conductor: true,
                second_analog: None,
                frames: &empty,
            }
        })
        .input,
        0
    );

    let two = [one[0], one[0]];
    let multiple = rear_input(RearInputProbe {
        facing: Direction::East,
        immediate_signal: 10,
        immediate_wire_power: None,
        immediate_analog: None,
        immediate_conductor: true,
        second_analog: Some(6),
        frames: &two,
    });
    assert_eq!(multiple.matching_frames, 2);
    assert_eq!(multiple.frame_candidate, None);
    assert_eq!(multiple.input, 6);
}

#[test]
fn comparator_compare_subtract_and_powered_predicate_short_circuit_rear_zero() {
    let zero = comparator_calculation(ComparatorMode::Compare, 0, 15);
    assert_eq!(
        (zero.output, zero.powered, zero.side_sampled),
        (0, false, false)
    );
    let compare_less = comparator_calculation(ComparatorMode::Compare, 10, 9);
    assert_eq!((compare_less.output, compare_less.powered), (10, true));
    let compare_equal = comparator_calculation(ComparatorMode::Compare, 10, 10);
    assert_eq!((compare_equal.output, compare_equal.powered), (10, true));
    let compare_greater = comparator_calculation(ComparatorMode::Compare, 10, 11);
    assert_eq!(
        (compare_greater.output, compare_greater.powered),
        (0, false)
    );
    let subtract = comparator_calculation(ComparatorMode::Subtract, 10, 3);
    assert_eq!((subtract.output, subtract.powered), (7, true));
    let subtract_equal = comparator_calculation(ComparatorMode::Subtract, 10, 10);
    assert_eq!((subtract_equal.output, subtract_equal.powered), (0, false));
    assert_eq!(side_input(4, 13), 13);
}

#[test]
fn comparator_neighbor_schedule_membership_priority_and_placement_are_exact() {
    assert_eq!(
        neighbor_priority(Direction::North, true, Direction::West),
        TickPriority::High
    );
    assert_eq!(
        neighbor_priority(Direction::North, true, Direction::South),
        TickPriority::Normal
    );
    let running = neighbor_check(true, 8, Some(0), true, false, TickPriority::High);
    assert!(!running.calculation_performed);
    assert_eq!(running.schedule, None);

    let mismatch = neighbor_check(false, 8, Some(0), true, false, TickPriority::High);
    assert!(!mismatch.powered_resampled);
    assert_eq!(mismatch.schedule.unwrap().delay, NEIGHBOR_DELAY);
    assert_eq!(mismatch.schedule.unwrap().priority, TickPriority::High);
    let powered_mismatch = neighbor_check(false, 8, Some(8), true, false, TickPriority::Normal);
    assert!(powered_mismatch.powered_resampled);
    assert!(powered_mismatch.schedule.is_some());
    assert_eq!(
        neighbor_check(false, 8, Some(8), true, true, TickPriority::Normal).schedule,
        None
    );
    assert_eq!(placement_schedule(true).unwrap().delay, PLACEMENT_DELAY);
    assert_eq!(placement_schedule(false), None);
}

#[test]
fn comparator_refresh_writes_cache_first_and_preserves_compare_subtract_quirks() {
    assert_eq!(
        REFRESH_ORDER,
        [
            RefreshStage::CalculateOutput,
            RefreshStage::ReadOldCache,
            RefreshStage::WriteCompatibleCache,
            RefreshStage::ResamplePowered,
            RefreshStage::OfferPoweredState,
            RefreshStage::NeighborChanged,
            RefreshStage::NeighborsExceptFacing,
        ]
    );
    assert_eq!(
        OUTPUT_NOTIFICATION_ORDER,
        [
            OutputNotification::NeighborChanged,
            OutputNotification::NeighborsExceptFacing,
        ]
    );
    let compare = refresh_plan(ComparatorMode::Compare, false, 6, true, true, 6, false);
    assert_eq!(compare.cache_write, Some(6));
    assert!(!compare.cache_marked_changed);
    assert_eq!(compare.powered_state_offer, Some(true));
    assert_eq!(compare.state_write_flags, Some(STATE_WRITE_FLAGS));
    assert!(compare.notify_output);
    assert!(!compare.experimental_orientation_draw_consumed);

    let subtract = refresh_plan(ComparatorMode::Subtract, false, 6, true, true, 6, false);
    assert!(!subtract.notify_output);
    assert_eq!(subtract.powered_state_offer, None);

    let missing = refresh_plan(
        ComparatorMode::Subtract,
        false,
        6,
        true,
        false,
        i32::MIN,
        true,
    );
    assert_eq!(missing.old_cached_output, 0);
    assert_eq!(missing.cache_write, None);
    assert!(missing.notify_output);
    assert!(missing.experimental_orientation_draw_consumed);
    assert_eq!(
        missing.orientation_bound,
        Some(EXPERIMENTAL_ORIENTATION_BOUND)
    );
    assert_eq!(
        EXPERIMENTAL_ORIENTATION_BIAS,
        [
            OrientationBias::Left,
            OrientationBias::Up,
            OrientationBias::FrontOppositeFacing,
        ]
    );
}

#[test]
fn comparator_use_signals_persistence_support_and_events_keep_raw_boundaries() {
    let state = ComparatorState {
        facing: Direction::East,
        mode: ComparatorMode::Compare,
        powered: true,
    };
    assert_eq!(
        comparator_signal(state, i32::MIN, Direction::East),
        i32::MIN
    );
    assert_eq!(comparator_signal(state, i32::MAX, Direction::West), 0);
    assert_eq!(
        comparator_signal(
            ComparatorState {
                powered: false,
                ..state
            },
            15,
            Direction::East
        ),
        0
    );
    assert_eq!(loaded_output_signal(None), 0);
    assert_eq!(loaded_output_signal(Some(i32::MAX)), i32::MAX);
    assert_eq!(OUTPUT_SIGNAL_KEY, "OutputSignal");
    assert!(!std::hint::black_box(BLOCK_ENTITY_SETTER_MARKS_CHANGED));
    assert!(!forward_block_event(true, Some(false)));
    assert!(forward_block_event(false, Some(true)));
    assert!(!forward_block_event(true, None));

    let denied = comparator_use(ComparatorMode::Compare, false, false, true);
    assert_eq!(denied.result, InteractionResult::Pass);
    assert!(!denied.sound_seed_long_consumed);
    let client = comparator_use(ComparatorMode::Compare, true, true, true);
    assert_eq!(client.result, InteractionResult::Success);
    assert_eq!(
        client.sound_recipients,
        Some(SoundRecipients::LocalExceptPlayer)
    );
    assert!(!client.state_write_offered);
    assert!(!client.refresh_intended_state);
    let server = comparator_use(ComparatorMode::Compare, true, false, true);
    assert_eq!(server.intended_mode, Some(ComparatorMode::Subtract));
    assert_eq!(server.pitch, Some(SUBTRACT_PITCH));
    assert_eq!(
        server.sound_recipients,
        Some(SoundRecipients::ServerExceptPlayer)
    );
    assert!(server.state_write_offered);
    assert!(server.refresh_intended_state);
    let replaced = comparator_use(ComparatorMode::Subtract, true, false, false);
    assert_eq!(replaced.pitch, Some(COMPARE_PITCH));
    assert!(!replaced.refresh_intended_state);
    assert_eq!(
        std::hint::black_box((CLICK_VOLUME, COMPARATOR_SHAPE_HEIGHT)),
        (0.3, 0.125)
    );

    let support = support_loss_plan(false).unwrap();
    assert!(support.capture_block_entity_for_drops);
    assert!(support.remove_moving_false);
    assert!(support.update_all_six_neighbors);
    assert_eq!(support_loss_plan(true), None);
    assert!(removal_notifies_output(false));
    assert!(!removal_notifies_output(true));
}

#[test]
fn daylight_ticker_admission_period_and_empty_subtype_are_exact() {
    assert!(ticker_installed(true, true));
    assert!(!ticker_installed(false, true));
    assert!(!ticker_installed(true, false));
    assert!(!periodic_tick_admitted(19));
    assert!(periodic_tick_admitted(20));
    assert!(!periodic_tick_admitted(21));
    assert_eq!(std::hint::black_box(TICK_PERIOD), 20);
    assert!(!std::hint::black_box(BLOCK_ENTITY_HAS_DATA));
    assert!(!std::hint::black_box(BLOCK_ENTITY_HAS_UPDATE_PACKET));
    assert!(!std::hint::black_box(BLOCK_ENTITY_HAS_RENDERER));
}

#[test]
fn daylight_formula_inverted_and_nonpositive_brightness_skip_sun_angle() {
    let inverted = daylight_formula(true, 12, 2, f64::NAN);
    assert_eq!(inverted.effective_brightness, 10);
    assert_eq!(inverted.target, 5);
    assert_eq!(inverted.initial_angle, None);
    let over = daylight_formula(true, -4, 2, 0.0);
    assert_eq!(over.unclamped_target, 21);
    assert_eq!(over.target, 15);
    let dark = daylight_formula(false, 1, 3, 90.0);
    assert_eq!(dark.unclamped_target, -2);
    assert_eq!(dark.target, 0);
    assert_eq!(dark.initial_angle, None);
}

#[test]
fn daylight_formula_uses_float_degrees_smoothing_lookup_cosine_round_and_clamp() {
    let noon = daylight_formula(false, 15, 0, 0.0);
    assert_eq!(noon.target, 15);
    assert_eq!(noon.initial_angle, Some(0.0));
    assert_eq!(noon.smoothed_angle, Some(0.0));

    let morning = daylight_formula(false, 15, 0, 90.0);
    assert_eq!(morning.target, 5);
    let below = daylight_formula(false, 15, 0, 180.0 - 0.0001);
    let equal = daylight_formula(false, 15, 0, 180.0);
    let above = daylight_formula(false, 15, 0, 180.0 + 0.0001);
    assert!(below.smoothed_angle.unwrap() < std::f32::consts::PI);
    assert!(equal.smoothed_angle.unwrap() > std::f32::consts::PI);
    assert!(above.smoothed_angle.unwrap() > std::f32::consts::PI);
    assert_eq!((below.target, equal.target, above.target), (0, 0, 0));
    assert_eq!(
        std::hint::black_box((DEGREE_TO_RADIAN, SUN_SMOOTHING)),
        (std::f32::consts::PI / 180.0, 0.2)
    );
}

#[test]
fn daylight_update_use_and_signals_preserve_intended_state_after_failed_write() {
    assert_eq!(
        DaylightState::default_state(),
        DaylightState {
            inverted: false,
            power: 0
        }
    );
    assert_eq!(daylight_update(7, 7).offered_power, None);
    let update = daylight_update(7, 9);
    assert_eq!(update.offered_power, Some(9));
    assert_eq!(update.write_flags, Some(POWER_WRITE_FLAGS));
    assert!(update.write_result_ignored);

    let denied = daylight_use(DaylightState::default_state(), false, false, 15);
    assert_eq!(denied.result, DaylightInteractionResult::Pass);
    let client = daylight_use(DaylightState::default_state(), true, true, 15);
    assert_eq!(client.result, DaylightInteractionResult::Success);
    assert_eq!(client.intended_inverted, Some(true));
    assert_eq!(client.first_write_flags, None);
    let server = daylight_use(DaylightState::default_state(), true, false, 15);
    assert_eq!(server.intended_inverted, Some(true));
    assert_eq!(server.first_write_flags, Some(INVERT_WRITE_FLAGS));
    assert!(server.emit_block_change);
    assert!(server.recompute_intended_state);
    assert_eq!(server.second_power_offer, Some(15));
    assert_eq!(server.second_write_flags, Some(POWER_WRITE_FLAGS));
    let unchanged = daylight_use(
        DaylightState {
            inverted: false,
            power: 15,
        },
        true,
        false,
        15,
    );
    assert_eq!(unchanged.second_power_offer, None);

    let signal = DaylightState {
        inverted: true,
        power: 12,
    };
    for _ in Direction::ALL {
        assert_eq!(ordinary_signal(signal), 12);
    }
    assert_eq!(direct_signal(), 0);
    assert_eq!(std::hint::black_box(SHAPE_HEIGHT), 0.375);
}
