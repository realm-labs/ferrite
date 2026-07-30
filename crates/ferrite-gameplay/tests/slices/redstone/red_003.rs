use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_gameplay::redstone::delay::diode::{
    DiodeInputSample, DiodePriority, PLACEMENT_DELAY, SHAPE_HEIGHT, STATE_WRITE_FLAGS, diode_input,
    due_tick, neighbor_schedule, on_place_notifies_output,
    output_notification as diode_output_notification, placement_schedule,
    removal_notifies_output as diode_removal_notifies_output, should_prioritize, support_loss,
};
use ferrite_gameplay::redstone::delay::observer::{
    EDGE_DELAY, ObserverState, REPLACEMENT_CLEAR_FLAGS, STATE_WRITE_FLAGS as OBSERVER_WRITE_FLAGS,
    due_tick as observer_tick, output_notification as observer_output_notification,
    output_position_direction, output_signal as observer_output,
    placement_facing as observer_placement_facing, removal_notifies_output, replacement_plan,
    start_signal,
};
use ferrite_gameplay::redstone::delay::orientation::{
    ORIENTATION_BOUND, OUTPUT_NOTIFICATION_ORDER, OutputNotificationStage,
};
use ferrite_gameplay::redstone::delay::repeater::{
    OUTPUT_SIGNAL, RepeaterDelay, RepeaterState, RepeaterUseResult, USE_WRITE_FLAGS, is_locked,
    output_signal as repeater_output, placement_facing as repeater_placement_facing,
    placement_locked, shape_update, use_repeater,
};
use ferrite_gameplay::redstone::delay::torch::{
    BURNOUT_LEVEL_EVENT, BURNOUT_THRESHOLD, DEFAULT_LIT, DEFAULT_WALL_FACING, HISTORY_MAX_AGE,
    RESTART_DELAY, STATE_WRITE_FLAGS as TORCH_WRITE_FLAGS, TOGGLE_DELAY, TorchAttachment,
    TorchToggle, TorchToggleHistory, direct_signal as torch_direct_signal, due_tick as torch_tick,
    floor_shape_becomes_air, has_neighbor_signal, input_query_direction,
    neighbor_schedule as torch_neighbor_schedule, notification_plan,
    ordinary_signal as torch_ordinary_signal, placement_notifies_neighbors,
    removal_notifies_neighbors, support_direction, wall_shape_becomes_air,
};
use ferrite_gameplay::redstone::signal::{ControlSource, SignalSample};

fn sample(ordinary: u8, direct_into_block: u8, conductor: bool) -> DiodeInputSample {
    DiodeInputSample {
        block: SignalSample {
            ordinary,
            direct_into_block,
            conductor,
        },
        dust_power: None,
    }
}

#[test]
fn diode_input_combines_conductor_and_dust_but_skips_dust_after_fifteen() {
    let direct = diode_input(sample(4, 13, true));
    assert_eq!(direct.signal, 13);
    assert!(direct.dust_queried);

    let dust = diode_input(DiodeInputSample {
        dust_power: Some(14),
        ..sample(4, 13, true)
    });
    assert_eq!(dust.signal, 14);
    assert!(dust.dust_queried);

    let fifteen = diode_input(DiodeInputSample {
        dust_power: Some(1),
        ..sample(15, 0, false)
    });
    assert_eq!(fifteen.signal, 15);
    assert!(!fifteen.dust_queried);
    assert_eq!(diode_input(sample(4, 13, false)).signal, 4);
}

#[test]
fn diode_neighbor_scheduling_locks_deduplicates_and_selects_every_priority() {
    assert!(should_prioritize(Direction::North, true, Direction::West));
    assert!(!should_prioritize(Direction::North, true, Direction::South));
    assert!(!should_prioritize(Direction::North, false, Direction::West));

    let extreme = neighbor_schedule(false, 15, false, false, true, 6).unwrap();
    assert_eq!(extreme.priority, DiodePriority::ExtremelyHigh);
    assert_eq!(extreme.delay, 6);
    assert_eq!(
        neighbor_schedule(true, 0, false, false, false, 2)
            .unwrap()
            .priority,
        DiodePriority::VeryHigh
    );
    assert_eq!(
        neighbor_schedule(false, 1, false, false, false, 2)
            .unwrap()
            .priority,
        DiodePriority::High
    );
    assert_eq!(neighbor_schedule(false, 1, true, false, false, 2), None);
    assert_eq!(neighbor_schedule(false, 1, false, true, false, 2), None);
    assert_eq!(neighbor_schedule(true, 1, false, false, false, 2), None);
}

#[test]
fn diode_due_tick_resamples_lock_and_preserves_the_vanished_input_pulse() {
    let locked = due_tick(false, 15, true, 8);
    assert_eq!(locked.offered_powered, None);
    let held = due_tick(true, 15, false, 8);
    assert_eq!(held.offered_powered, None);

    let falling = due_tick(true, 0, false, 8);
    assert_eq!(falling.offered_powered, Some(false));
    assert_eq!(falling.write_flags, Some(STATE_WRITE_FLAGS));
    assert_eq!(falling.follow_up, None);

    let rising = due_tick(false, 15, false, 8);
    assert_eq!(rising.offered_powered, Some(true));
    assert_eq!(rising.follow_up, None);
    let vanished = due_tick(false, 0, false, 8);
    assert_eq!(vanished.offered_powered, Some(true));
    assert_eq!(vanished.follow_up.unwrap().delay, 8);
    assert_eq!(
        vanished.follow_up.unwrap().priority,
        DiodePriority::VeryHigh
    );
}

#[test]
fn diode_placement_and_support_loss_keep_the_one_tick_and_six_neighbor_edges() {
    let placement = placement_schedule(1).unwrap();
    assert_eq!(placement.delay, PLACEMENT_DELAY);
    assert_eq!(placement.priority, DiodePriority::Normal);
    assert_eq!(placement_schedule(0), None);
    assert_eq!(std::hint::black_box(SHAPE_HEIGHT), 0.125);

    let loss = support_loss(false).unwrap();
    assert!(loss.drop_resources);
    assert!(loss.remove_moving_false);
    assert!(loss.notify_all_six_neighbors);
    assert_eq!(support_loss(true), None);
    assert!(on_place_notifies_output());
    assert!(diode_removal_notifies_output(false));
    assert!(!diode_removal_notifies_output(true));
    let default_notification = diode_output_notification(Direction::North, false);
    assert_eq!(default_notification.output_direction, Direction::South);
    assert_eq!(default_notification.order, OUTPUT_NOTIFICATION_ORDER);
    assert!(!default_notification.orientation.draw_consumed);
    assert_eq!(default_notification.orientation.fixed_up, None);
    let experimental = diode_output_notification(Direction::East, true);
    assert_eq!(experimental.orientation.bound, Some(ORIENTATION_BOUND));
    assert_eq!(experimental.orientation.fixed_front, Some(Direction::West));
}

#[test]
fn repeater_delay_cycle_lock_shape_use_and_output_are_exact() {
    assert_eq!(
        RepeaterState::default_state(),
        RepeaterState {
            facing: Direction::North,
            delay: RepeaterDelay::One,
            locked: false,
            powered: false,
        }
    );
    assert_eq!(repeater_placement_facing(Direction::East), Direction::West);
    let settings = [
        RepeaterDelay::One,
        RepeaterDelay::Two,
        RepeaterDelay::Three,
        RepeaterDelay::Four,
    ];
    assert_eq!(settings.map(RepeaterDelay::ticks), [2, 4, 6, 8]);
    assert_eq!(
        settings.map(RepeaterDelay::cycled),
        [
            RepeaterDelay::Two,
            RepeaterDelay::Three,
            RepeaterDelay::Four,
            RepeaterDelay::One,
        ]
    );
    assert!(is_locked(ControlSource::Diode(1), ControlSource::Other));
    assert!(!is_locked(
        ControlSource::RedstoneBlock,
        ControlSource::Wire(15)
    ));

    assert!(placement_locked(
        ControlSource::Diode(1),
        ControlSource::Other
    ));
    let changed = shape_update(Direction::East, Direction::North, true, false, true, true);
    assert!(!changed.becomes_air);
    assert!(changed.intended_locked);
    assert!(changed.server_write_offered);
    assert!(
        !shape_update(Direction::East, Direction::North, true, false, true, false)
            .server_write_offered
    );
    assert!(
        !shape_update(Direction::South, Direction::North, true, false, true, true)
            .server_write_offered
    );
    assert!(shape_update(Direction::Down, Direction::North, false, false, true, true).becomes_air);

    let denied = use_repeater(RepeaterDelay::One, false);
    assert_eq!(denied.result, RepeaterUseResult::Pass);
    assert_eq!(denied.intended_delay, None);
    let used = use_repeater(RepeaterDelay::Four, true);
    assert_eq!(used.result, RepeaterUseResult::Success);
    assert_eq!(used.intended_delay, Some(RepeaterDelay::One));
    assert!(used.state_write_offered);
    assert_eq!(used.write_flags, Some(USE_WRITE_FLAGS));

    assert_eq!(
        repeater_output(true, Direction::East, Direction::East),
        OUTPUT_SIGNAL
    );
    assert_eq!(repeater_output(true, Direction::East, Direction::West), 0);
    assert_eq!(repeater_output(false, Direction::East, Direction::East), 0);
}

#[test]
fn observer_admits_only_new_unpowered_watched_face_work_on_the_server() {
    assert_eq!(
        start_signal(false, false, Direction::North, Direction::North, false).schedule_after,
        None
    );
    assert_eq!(
        start_signal(true, true, Direction::North, Direction::North, false).schedule_after,
        None
    );
    assert_eq!(
        start_signal(true, false, Direction::North, Direction::South, false).schedule_after,
        None
    );
    assert_eq!(
        start_signal(true, false, Direction::North, Direction::North, true).schedule_after,
        None
    );
    assert_eq!(
        start_signal(true, false, Direction::North, Direction::North, false).schedule_after,
        Some(EDGE_DELAY)
    );
}

#[test]
fn observer_due_edges_form_a_two_tick_high_pulse_and_notify_in_order() {
    assert_eq!(
        ObserverState::default_state(),
        ObserverState {
            facing: Direction::South,
            powered: false,
        }
    );
    assert_eq!(observer_placement_facing(Direction::Up), Direction::Up);
    let rise = observer_tick(false);
    assert!(rise.offered_powered);
    assert_eq!(rise.write_flags, OBSERVER_WRITE_FLAGS);
    assert_eq!(rise.follow_up_after, Some(EDGE_DELAY));
    assert!(rise.notify_output);
    let fall = observer_tick(true);
    assert!(!fall.offered_powered);
    assert_eq!(fall.follow_up_after, None);
    assert_eq!(
        OUTPUT_NOTIFICATION_ORDER,
        [
            OutputNotificationStage::NeighborChanged,
            OutputNotificationStage::NeighborsExceptFacing,
        ]
    );
    assert_eq!(output_position_direction(Direction::West), Direction::East);
    assert_eq!(observer_output(true, Direction::West, Direction::West), 15);
    assert_eq!(observer_output(true, Direction::West, Direction::East), 0);
    let experimental = observer_output_notification(Direction::West, true);
    assert_eq!(experimental.output_direction, Direction::East);
    assert_eq!(experimental.orientation.bound, Some(ORIENTATION_BOUND));
    assert_eq!(experimental.orientation.fixed_front, Some(Direction::East));
    assert_eq!(experimental.orientation.fixed_up, None);
}

#[test]
fn observer_replacement_and_removal_preserve_pending_tick_quirks() {
    let clear = replacement_plan(true, false, true, false);
    assert_eq!(clear.offered_powered, Some(false));
    assert_eq!(clear.write_flags, Some(REPLACEMENT_CLEAR_FLAGS));
    assert!(clear.notify_output);
    assert_eq!(
        replacement_plan(true, false, true, true).offered_powered,
        None
    );
    assert_eq!(
        replacement_plan(true, false, false, false).offered_powered,
        None
    );
    assert_eq!(
        replacement_plan(false, false, true, false).offered_powered,
        None
    );
    assert_eq!(
        replacement_plan(true, true, true, false).offered_powered,
        None
    );
    assert!(removal_notifies_output(true, true));
    assert!(!removal_notifies_output(true, false));
    assert!(!removal_notifies_output(false, true));
}

#[test]
fn torch_floor_and_wall_inputs_schedule_only_one_mismatched_edge() {
    assert_eq!(support_direction(TorchAttachment::Floor), Direction::Down);
    assert_eq!(
        input_query_direction(TorchAttachment::Floor),
        Direction::Down
    );
    let wall = TorchAttachment::Wall {
        facing: Direction::East,
    };
    assert_eq!(support_direction(wall), Direction::West);
    assert_eq!(input_query_direction(wall), Direction::West);
    assert!(has_neighbor_signal(1));
    assert!(!has_neighbor_signal(0));
    assert_eq!(
        torch_neighbor_schedule(true, true, false),
        Some(TOGGLE_DELAY)
    );
    assert_eq!(
        torch_neighbor_schedule(false, false, false),
        Some(TOGGLE_DELAY)
    );
    assert_eq!(torch_neighbor_schedule(true, false, false), None);
    assert_eq!(torch_neighbor_schedule(true, true, true), None);
    assert!(floor_shape_becomes_air(Direction::Down, false));
    assert!(!floor_shape_becomes_air(Direction::North, false));
    assert!(wall_shape_becomes_air(
        Direction::East,
        Direction::West,
        false
    ));
    assert!(!wall_shape_becomes_air(
        Direction::East,
        Direction::North,
        false
    ));
}

#[test]
fn torch_history_retains_age_sixty_purges_age_sixty_one_and_is_level_wide() {
    let position = BlockPos::new(1, 2, 3);
    let other = BlockPos::new(9, 9, 9);
    let mut history = TorchToggleHistory::from_entries([
        TorchToggle { position, when: 40 },
        TorchToggle {
            position: other,
            when: 41,
        },
        TorchToggle { position, when: 42 },
    ]);
    assert_eq!(history.purge(100), 0);
    assert_eq!(history.entries().len(), 3);
    assert_eq!(history.purge(101), 1);
    assert_eq!(history.entries().front().unwrap().position, other);
    assert_eq!(history.purge(103), 2);
    assert!(history.entries().is_empty());
    assert_eq!(std::hint::black_box(HISTORY_MAX_AGE), 60);
}

#[test]
fn eighth_torch_toggle_burns_out_after_the_unlit_write_and_schedules_restart() {
    let position = BlockPos::new(4, 5, 6);
    let mut history =
        TorchToggleHistory::from_entries((0..7).map(|when| TorchToggle { position, when }));
    let plan = torch_tick(&mut history, position, 7, true, true);
    assert_eq!(plan.purged_entries, 0);
    assert_eq!(plan.offered_lit, Some(false));
    assert_eq!(plan.write_flags, Some(TORCH_WRITE_FLAGS));
    assert!(plan.recorded_toggle);
    assert_eq!(plan.emitted_level_event, Some(BURNOUT_LEVEL_EVENT));
    assert_eq!(plan.restart_after, Some(RESTART_DELAY));
    assert!(plan.restart_targets_live_block);
    assert!(!plan.burnout_suppressed_relight);
    assert_eq!(history.entries().len(), BURNOUT_THRESHOLD);
}

#[test]
fn torch_restart_can_stay_dark_without_rescheduling_or_relight_after_expiry() {
    let position = BlockPos::new(-1, 0, 1);
    let entries = (0..8).map(|when| TorchToggle {
        position,
        when: 100 + when,
    });
    let mut retained = TorchToggleHistory::from_entries(entries.clone());
    let blocked = torch_tick(&mut retained, position, 160, false, false);
    assert_eq!(blocked.offered_lit, None);
    assert!(blocked.burnout_suppressed_relight);
    assert_eq!(blocked.restart_after, None);

    let mut expired = TorchToggleHistory::from_entries(entries);
    let relit = torch_tick(&mut expired, position, 168, false, false);
    assert_eq!(relit.purged_entries, 8);
    assert_eq!(relit.offered_lit, Some(true));
    assert_eq!(relit.write_flags, Some(TORCH_WRITE_FLAGS));
    assert!(!relit.burnout_suppressed_relight);

    let unchanged = torch_tick(&mut expired, position, 169, true, false);
    assert_eq!(unchanged.offered_lit, None);
}

#[test]
fn torch_signals_and_neighbor_orientation_distinguish_floor_wall_and_experiments() {
    let floor = TorchAttachment::Floor;
    let wall = TorchAttachment::Wall {
        facing: Direction::East,
    };
    assert!(std::hint::black_box(DEFAULT_LIT));
    assert_eq!(DEFAULT_WALL_FACING, Direction::North);
    assert_eq!(torch_ordinary_signal(floor, true, Direction::Down), 15);
    assert_eq!(torch_ordinary_signal(floor, true, Direction::Up), 0);
    assert_eq!(torch_ordinary_signal(wall, true, Direction::East), 0);
    assert_eq!(torch_ordinary_signal(wall, true, Direction::West), 15);
    assert_eq!(torch_ordinary_signal(wall, false, Direction::West), 0);
    assert_eq!(torch_direct_signal(floor, true, Direction::Down), 15);
    assert_eq!(torch_direct_signal(floor, true, Direction::North), 0);
    assert_eq!(torch_direct_signal(wall, true, Direction::Down), 15);

    let default_floor = notification_plan(floor, false);
    assert!(!default_floor.orientation.draw_consumed);
    assert_eq!(default_floor.orientation.bound, None);
    assert_eq!(default_floor.orientation.fixed_up, None);
    assert_eq!(default_floor.orientation.fixed_front, None);
    assert_eq!(default_floor.directions, Direction::ALL);
    let experimental_wall = notification_plan(wall, true);
    assert!(experimental_wall.orientation.draw_consumed);
    assert_eq!(experimental_wall.orientation.bound, Some(ORIENTATION_BOUND));
    assert_eq!(
        experimental_wall.orientation.fixed_front,
        Some(Direction::West)
    );
    assert!(placement_notifies_neighbors());
    assert!(removal_notifies_neighbors(false));
    assert!(!removal_notifies_neighbors(true));
}
