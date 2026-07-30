use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_simulation::command_limit::chain::{ChainCell, ChainVisitOutcome, execute_chain};
use ferrite_simulation::command_limit::context::{
    AutomaticCost, ContextAdmission, ContextLimits, DEFAULT_COMMAND_LIMIT, DEFAULT_FORK_LIMIT,
    DrainStop, ExecutionContext, MAX_QUEUE_DEPTH, OUTER_CONTEXT_LIFECYCLE, QueueAdmission,
    QueuedAction, context_admission, queue_admission,
};
use ferrite_simulation::command_limit::redirect::{
    CUSTOM_REDIRECT_PLAN, CustomRedirectPlan, ErrorSource, ModifierResult, RedirectErrorKind,
    evaluate_standard_redirect,
};
use ferrite_simulation::server_tick::pacing::{
    DeadlineClock, OVERLOAD_BASE_THRESHOLD_NANOS, OVERLOAD_THRESHOLD_TICKS, TimeBudget,
};
use ferrite_simulation::server_tick::pause::{
    DedicatedPauseDecision, DedicatedPauseState, IntegratedPauseDecision, IntegratedPauseState,
};
use ferrite_simulation::server_tick::phases::{
    BlockEntityTickOutcome, EntityTickInput, EntityTickOutcome, LEVEL_EMPTY_ACTIVITY_CUTOFF,
    LEVEL_SCHEDULED_TICK_LIMIT, LevelPhaseInput, LevelStage, ServerChildStage,
    SleepTransitionInput, advance_base_tick_bookkeeping, block_entity_tick_outcome,
    clock_manager_plan, entity_tick_plan, increment_shared_game_time, level_phase_plan,
    server_child_order, sleep_transition,
};
use ferrite_simulation::server_tick::rate::{
    DEFAULT_NANOS_PER_TICK, DEFAULT_TICK_RATE, MAX_COMMAND_TICK_RATE, MIN_AUTOSAVE_TICKS,
    MIN_TICK_RATE, ServerTickRateState, SprintCheck, TickRateState,
    apply_changed_autosave_interval, compute_next_autosave_interval, smooth_tick_time,
};

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}

fn action(id: u8, cost: AutomaticCost) -> QueuedAction<u8> {
    QueuedAction {
        frame_depth: i32::from(id),
        automatic_cost: cost,
        payload: id,
    }
}

fn chain_cell(next_facing: Direction) -> ChainCell {
    ChainCell {
        is_chain_command_block: true,
        has_command_block_entity: true,
        sequence_mode: true,
        powered: true,
        automatic: false,
        condition_met: true,
        conditional: false,
        command_succeeded: true,
        next_facing,
    }
}

#[test]
fn tick_rate_defaults_command_range_and_interval_truncation_are_exact() {
    let mut rate = TickRateState::default();
    assert_eq!(rate.tick_rate(), DEFAULT_TICK_RATE);
    assert_eq!(rate.nanoseconds_per_tick(), DEFAULT_NANOS_PER_TICK);
    rate.set_command_tick_rate(3.0).unwrap();
    assert_eq!(rate.nanoseconds_per_tick(), 333_333_333);
    rate.set_command_tick_rate(MAX_COMMAND_TICK_RATE).unwrap();
    assert_eq!(rate.nanoseconds_per_tick(), 100_000);
    assert!(rate.set_command_tick_rate(0.999).is_err());
    assert!(rate.set_command_tick_rate(10_000.1).is_err());
    assert!(rate.set_command_tick_rate(f32::NAN).is_err());

    rate.set_tick_rate(-20.0);
    assert_eq!(rate.tick_rate(), MIN_TICK_RATE);
    rate.set_tick_rate(f32::NAN);
    assert!(rate.tick_rate().is_nan());
    assert_eq!(rate.nanoseconds_per_tick(), 0);
}

#[test]
fn freeze_snapshot_consumes_positive_steps_even_while_unfrozen() {
    let mut rate = TickRateState::default();
    rate.set_frozen(true);
    assert!(!rate.tick().run_game_elements);
    rate.set_frozen_ticks_to_run(3);
    assert_eq!(rate.tick().remaining_steps, 2);
    assert!(rate.runs_normally());
    assert_eq!(rate.tick().remaining_steps, 1);
    assert_eq!(rate.tick().remaining_steps, 0);
    assert!(!rate.tick().run_game_elements);

    rate.set_frozen(false);
    rate.set_frozen_ticks_to_run(2);
    let admission = rate.tick();
    assert!(admission.run_game_elements);
    assert!(admission.consumed_step);
    assert_eq!(admission.remaining_steps, 1);
}

#[test]
fn frozen_entity_exemption_requires_player_or_player_passenger_tree() {
    let mut rate = TickRateState::default();
    rate.set_frozen(true);
    rate.tick();
    assert!(rate.entity_is_frozen(false, 0));
    assert!(!rate.entity_is_frozen(true, 0));
    assert!(!rate.entity_is_frozen(false, 1));
}

#[test]
fn sprint_waits_for_run_snapshot_then_finishes_on_the_following_check() {
    let mut state = ServerTickRateState::default();
    state.rate_mut().set_frozen(true);
    state.rate_mut().tick();
    assert!(!state.request_sprint(2));
    assert_eq!(
        state.check_should_sprint_this_tick(100),
        SprintCheck::WaitingForRunElements
    );
    state.rate_mut().tick();
    assert_eq!(
        state.check_should_sprint_this_tick(200),
        SprintCheck::AdmitSprintTick { remaining_ticks: 1 }
    );
    state.end_sprint_tick_work(1_000_200);
    assert_eq!(
        state.check_should_sprint_this_tick(300),
        SprintCheck::AdmitSprintTick { remaining_ticks: 0 }
    );
    state.end_sprint_tick_work(2_000_300);
    let SprintCheck::Finished(report) = state.check_should_sprint_this_tick(400) else {
        panic!("sprint must finish after its final admitted tick");
    };
    assert_eq!(report.completed_ticks, 2);
    assert_eq!(report.elapsed_milliseconds, 3.0);
    assert!(report.restored_frozen);
    assert!(state.rate().is_frozen());
}

#[test]
fn replacing_a_sprint_records_the_already_unfrozen_state() {
    let mut state = ServerTickRateState::default();
    state.rate_mut().set_frozen(true);
    assert!(!state.request_sprint(3));
    assert!(state.request_sprint(1));
    state.rate_mut().tick();
    assert!(matches!(
        state.check_should_sprint_this_tick(10),
        SprintCheck::AdmitSprintTick { remaining_ticks: 0 }
    ));
    state.end_sprint_tick_work(1_000_010);
    let SprintCheck::Finished(report) = state.check_should_sprint_this_tick(20) else {
        panic!("replacement sprint must finish");
    };
    assert!(!report.restored_frozen);
    assert!(!state.rate().is_frozen());
}

#[test]
fn freeze_command_stops_active_sprint_and_pending_steps_before_freezing() {
    let mut state = ServerTickRateState::default();
    state.rate_mut().set_frozen(true);
    assert!(state.step_game_if_paused(4));
    assert!(!state.request_sprint(5));
    let result = state.apply_freeze_command(true);
    assert_eq!(result.sprint_report.unwrap().completed_ticks, 0);
    assert!(result.stepping_stopped);
    assert!(state.rate().is_frozen());
    assert_eq!(state.rate().frozen_ticks_to_run(), 0);
    assert!(!state.is_sprinting());
}

#[test]
fn ordinary_deadlines_drop_missed_intervals_without_catch_up_ticks() {
    let interval = DEFAULT_NANOS_PER_TICK;
    let next = 10_000_000_000;
    let mut clock = DeadlineClock::with_state(next, -10_000_000_000);
    let now =
        next + OVERLOAD_BASE_THRESHOLD_NANOS + OVERLOAD_THRESHOLD_TICKS * interval + 100_000_000;
    let plan = clock.plan_iteration(now, interval, false).unwrap();
    assert!(plan.overload_warning);
    assert_eq!(plan.missed_intervals, 42);
    assert_eq!(plan.next_tick_time_nanos, now + interval);
    assert_eq!(plan.time_budget, TimeBudget::RemainingUntilDeadline);

    let delayed = clock.delayed_tasks_deadline(now + 1, interval);
    assert_eq!(delayed, now + interval + 1);
}

#[test]
fn overload_threshold_is_strict_and_sprint_resets_both_deadlines() {
    let interval = DEFAULT_NANOS_PER_TICK;
    let threshold = OVERLOAD_BASE_THRESHOLD_NANOS + OVERLOAD_THRESHOLD_TICKS * interval;
    let mut ordinary = DeadlineClock::with_state(20_000_000_000, 0);
    let plan = ordinary
        .plan_iteration(20_000_000_000 + threshold, interval, false)
        .unwrap();
    assert!(!plan.overload_warning);
    assert_eq!(plan.missed_intervals, 0);

    let mut sprint = DeadlineClock::with_state(5, 1);
    let plan = sprint.plan_iteration(99, interval, true).unwrap();
    assert_eq!(plan.interval_nanos, 0);
    assert_eq!(plan.next_tick_time_nanos, 99);
    assert_eq!(plan.last_overload_warning_nanos, 99);
    assert_eq!(plan.time_budget, TimeBudget::AlwaysFalse);
}

#[test]
fn dedicated_empty_pause_counts_loop_admissions_at_fixed_twenty_multiplier() {
    let mut pause = DedicatedPauseState::default();
    for expected in 1..1_200 {
        assert_eq!(
            pause.evaluate(60, 0, false),
            DedicatedPauseDecision::AdmitBaseTick
        );
        assert_eq!(pause.empty_ticks(), expected);
    }
    assert_eq!(
        pause.evaluate(60, 0, false),
        DedicatedPauseDecision::Pause {
            first_paused_iteration: true,
            auto_save: true,
            tick_connections: true,
        }
    );
    assert_eq!(
        pause.evaluate(60, 0, false),
        DedicatedPauseDecision::Pause {
            first_paused_iteration: false,
            auto_save: false,
            tick_connections: true,
        }
    );
    assert_eq!(
        pause.evaluate(60, 1, false),
        DedicatedPauseDecision::AdmitBaseTick
    );
    assert_eq!(pause.empty_ticks(), 0);
}

#[test]
fn disabled_dedicated_pause_preserves_counter_and_sprint_resets_it() {
    let mut pause = DedicatedPauseState::default();
    pause.evaluate(1, 0, false);
    assert_eq!(pause.empty_ticks(), 1);
    assert_eq!(
        pause.evaluate(0, 0, false),
        DedicatedPauseDecision::AdmitBaseTick
    );
    assert_eq!(pause.empty_ticks(), 1);
    pause.evaluate(1, 0, true);
    assert_eq!(pause.empty_ticks(), 0);
}

#[test]
fn integrated_pause_saves_once_maintains_connections_and_syncs_on_resume() {
    let mut pause = IntegratedPauseState::default();
    assert_eq!(
        pause.evaluate(true, 1),
        IntegratedPauseDecision::SaveThenPause {
            tick_connections: true,
            player_stat_awards: 1,
        }
    );
    assert_eq!(
        pause.evaluate(true, 1),
        IntegratedPauseDecision::ContinuePaused {
            tick_connections: true,
            player_stat_awards: 1,
        }
    );
    assert_eq!(
        pause.evaluate(false, 1),
        IntegratedPauseDecision::SynchronizeTimeThenAdmit
    );
    assert_eq!(
        pause.evaluate(false, 1),
        IntegratedPauseDecision::AdmitBaseTick
    );
}

#[test]
fn child_server_order_keeps_functions_levels_and_network_freeze_exempt() {
    let frozen = server_child_order(false, 20, 2);
    assert_eq!(
        frozen,
        [
            ServerChildStage::SuspendPlayerFlushing,
            ServerChildStage::TickCommandFunctions,
            ServerChildStage::SynchronizeTime,
            ServerChildStage::RefreshEffectiveRespawn,
            ServerChildStage::TickLevel(0),
            ServerChildStage::TickLevel(1),
            ServerChildStage::TickConnections,
            ServerChildStage::TickPlayerList,
            ServerChildStage::TickDebugSubscribers,
            ServerChildStage::TickGui,
            ServerChildStage::SendChunksAndResumeFlushing,
            ServerChildStage::TickActivityMonitor,
        ]
    );
    let normal = server_child_order(true, 21, 1);
    assert!(normal.contains(&ServerChildStage::TickClockManager));
    assert!(normal.contains(&ServerChildStage::TickGameTests));
    assert!(!normal.contains(&ServerChildStage::SynchronizeTime));
}

#[test]
fn level_plan_preserves_run_debug_time_owner_and_empty_cutoff_gates() {
    let normal = level_phase_plan(LevelPhaseInput {
        run_game_elements: true,
        debug_level: false,
        owns_game_time_increment: true,
        empty_time: 298,
        has_active_chunk_tickets: false,
    });
    assert_eq!(normal.next_empty_time, 299);
    assert!(normal.stages.contains(&LevelStage::TickTime {
        owns_game_time_increment: true
    }));
    assert!(normal.stages.contains(&LevelStage::DrainScheduledBlocks {
        maximum: LEVEL_SCHEDULED_TICK_LIMIT
    }));
    assert!(normal.stages.contains(&LevelStage::TickDragonFight));

    let frozen = level_phase_plan(LevelPhaseInput {
        run_game_elements: false,
        debug_level: false,
        owns_game_time_increment: false,
        empty_time: 299,
        has_active_chunk_tickets: false,
    });
    assert_eq!(frozen.next_empty_time, 299);
    assert!(!frozen.stages.contains(&LevelStage::TickWorldBorder));
    assert!(frozen.stages.contains(&LevelStage::TickChunkSource));
    assert!(
        frozen
            .stages
            .contains(&LevelStage::TraverseEligibleEntities)
    );
    assert!(
        frozen
            .stages
            .contains(&LevelStage::ProcessBlockEntityTickers)
    );

    let cutoff = level_phase_plan(LevelPhaseInput {
        run_game_elements: true,
        debug_level: true,
        owns_game_time_increment: false,
        empty_time: LEVEL_EMPTY_ACTIVITY_CUTOFF - 1,
        has_active_chunk_tickets: false,
    });
    assert_eq!(cutoff.next_empty_time, LEVEL_EMPTY_ACTIVITY_CUTOFF);
    assert!(
        !cutoff
            .stages
            .contains(&LevelStage::TraverseEligibleEntities)
    );
    assert!(
        !cutoff
            .stages
            .iter()
            .any(|stage| matches!(stage, LevelStage::DrainScheduledBlocks { .. }))
    );
    assert!(cutoff.stages.contains(&LevelStage::TickTime {
        owns_game_time_increment: false
    }));

    let active = level_phase_plan(LevelPhaseInput {
        run_game_elements: true,
        debug_level: false,
        owns_game_time_increment: false,
        empty_time: 900,
        has_active_chunk_tickets: true,
    });
    assert_eq!(active.next_empty_time, 1);
    assert!(
        active
            .stages
            .contains(&LevelStage::TraverseEligibleEntities)
    );
}

#[test]
fn entity_and_block_entity_tick_decisions_preserve_source_gate_order() {
    let ordinary = EntityTickInput {
        removed: false,
        is_player: false,
        player_passengers: 0,
        in_entity_ticking_range: true,
        has_vehicle: true,
        vehicle_link_valid: true,
    };
    assert_eq!(
        entity_tick_plan(false, ordinary).outcome,
        EntityTickOutcome::SkipFrozen
    );
    assert_eq!(
        entity_tick_plan(true, ordinary).outcome,
        EntityTickOutcome::LeaveForVehicleTraversal
    );
    assert_eq!(
        entity_tick_plan(
            false,
            EntityTickInput {
                player_passengers: 1,
                vehicle_link_valid: false,
                ..ordinary
            }
        )
        .outcome,
        EntityTickOutcome::DetachVehicleAndTick
    );
    assert_eq!(
        entity_tick_plan(
            true,
            EntityTickInput {
                in_entity_ticking_range: false,
                ..ordinary
            }
        )
        .outcome,
        EntityTickOutcome::SkipOutOfRange
    );
    assert_eq!(
        block_entity_tick_outcome(true, false, false),
        BlockEntityTickOutcome::RemoveInvalidTicker
    );
    assert_eq!(
        block_entity_tick_outcome(false, false, true),
        BlockEntityTickOutcome::KeepWithoutCallback
    );
    assert_eq!(
        block_entity_tick_outcome(false, true, true),
        BlockEntityTickOutcome::InvokeCallback
    );
    assert!(
        entity_tick_plan(
            true,
            EntityTickInput {
                in_entity_ticking_range: false,
                ..ordinary
            }
        )
        .check_despawn
    );
}

#[test]
fn sleep_clock_and_shared_game_time_gates_remain_independent_during_freeze() {
    let transition = sleep_transition(SleepTransitionInput {
        enough_sleeping: true,
        enough_deep_sleeping: true,
        advance_time: false,
        default_clock_present: true,
        advance_weather: true,
        raining: true,
    });
    assert!(!transition.move_default_clock_to_wake_marker);
    assert!(transition.wake_all_players);
    assert!(transition.reset_weather_cycle);
    assert!(!clock_manager_plan(false, true).invoke_manager);
    assert!(clock_manager_plan(true, true).advance_registered_clocks);
    assert!(!clock_manager_plan(true, false).advance_registered_clocks);
    assert_eq!(increment_shared_game_time(i64::MAX, true), i64::MIN);
    assert_eq!(increment_shared_game_time(8, false), 8);
}

#[test]
fn base_bookkeeping_wraps_ticks_and_keeps_status_autosave_and_smoothing_exact() {
    let result = advance_base_tick_bookkeeping(i32::MAX, 1, 6_000_000_000, 0);
    assert_eq!(result.tick_count, i32::MIN);
    assert!(result.status_refresh);
    assert!(result.auto_save);
    assert_eq!(result.ring_index, i32::MIN % 100);
    assert_eq!(smooth_tick_time(10.0, 20_000_000), 12.0);
    assert_eq!(compute_next_autosave_interval(20.0, false, 0), 6_000);
    assert_eq!(
        compute_next_autosave_interval(20.0, true, 19_999_999),
        15_000
    );
    assert_eq!(
        compute_next_autosave_interval(1.0, false, 0),
        300.max(MIN_AUTOSAVE_TICKS)
    );
    assert_eq!(apply_changed_autosave_interval(6_000, 3_000), 3_000);
    assert_eq!(apply_changed_autosave_interval(3_000, 6_000), 3_000);
}

#[test]
fn command_context_snapshots_floor_sequence_and_nested_calls_reuse_it() {
    assert_eq!(DEFAULT_COMMAND_LIMIT, 65_536);
    assert_eq!(DEFAULT_FORK_LIMIT, 65_536);
    assert_eq!(
        context_admission(false, 0, 7),
        ContextAdmission::CreateOuter(ContextLimits {
            command_limit: 1,
            fork_limit: 7,
        })
    );
    assert_eq!(
        context_admission(true, 999, 999),
        ContextAdmission::ReuseExisting
    );
    assert_eq!(OUTER_CONTEXT_LIFECYCLE.len(), 5);
}

#[test]
fn newly_queued_actions_precede_older_work_without_reversing_their_order() {
    let mut context = ExecutionContext::new(ContextLimits::snapshot(20, 20));
    context.queue_next(action(1, AutomaticCost::None));
    context.queue_next(action(4, AutomaticCost::None));
    let mut order = Vec::new();
    let report = context.run(|id, context| {
        order.push(id);
        if id == 1 {
            context.queue_next(action(2, AutomaticCost::None));
            context.queue_next(action(3, AutomaticCost::None));
        }
    });
    assert_eq!(order, [1, 2, 3, 4]);
    assert_eq!(report.stop, DrainStop::QueueEmpty);
    assert_eq!(context.current_frame_depth(), 4);
}

#[test]
fn last_charged_action_completes_and_the_next_poll_abandons_pending_work() {
    let mut context = ExecutionContext::new(ContextLimits::snapshot(1, 0));
    context.queue_next(action(1, AutomaticCost::RedirectModifier));
    context.queue_next(action(2, AutomaticCost::ExecuteCommand));
    let mut order = Vec::new();
    let report = context.run(|id, _| order.push(id));
    assert_eq!(order, [1]);
    assert_eq!(report.stop, DrainStop::CommandLimit);
    assert_eq!(report.executed_actions, 1);
    assert_eq!(report.charged_actions, 1);
    assert_eq!(report.abandoned_actions, 1);
    assert_eq!(context.current_frame_depth(), 0);
}

#[test]
fn only_the_three_generic_action_sites_consume_command_quota() {
    let mut context = ExecutionContext::new(ContextLimits::snapshot(4, 0));
    for (id, cost) in [
        (1, AutomaticCost::None),
        (2, AutomaticCost::RedirectModifier),
        (3, AutomaticCost::CallFunction),
        (4, AutomaticCost::ExecuteCommand),
        (5, AutomaticCost::None),
    ] {
        context.queue_next(action(id, cost));
    }
    let report = context.run(|_, _| {});
    assert_eq!(report.stop, DrainStop::QueueEmpty);
    assert_eq!(report.executed_actions, 5);
    assert_eq!(report.charged_actions, 3);
    assert_eq!(context.remaining_quota(), 1);
}

#[test]
fn multiple_redirect_stages_can_drive_quota_below_zero_inside_one_action() {
    let mut context = ExecutionContext::new(ContextLimits::snapshot(1, 10));
    context.queue_next(action(1, AutomaticCost::None));
    let mut order = Vec::new();
    let report = context.run(|id, context| {
        order.push(id);
        context.increment_cost();
        context.increment_cost();
        context.queue_next(action(2, AutomaticCost::ExecuteCommand));
    });
    assert_eq!(order, [1]);
    assert_eq!(context.remaining_quota(), -1);
    assert_eq!(report.stop, DrainStop::CommandLimit);
    assert_eq!(report.charged_actions, 2);
    assert_eq!(report.abandoned_actions, 1);
}

#[test]
fn defensive_queue_boundary_admits_size_ten_million_then_overflows() {
    assert_eq!(
        queue_admission(MAX_QUEUE_DEPTH, 0, false),
        QueueAdmission::Admit
    );
    assert_eq!(
        queue_admission(MAX_QUEUE_DEPTH + 1, 0, false),
        QueueAdmission::OverflowAndClear
    );
    assert_eq!(
        queue_admission(0, 0, true),
        QueueAdmission::AlreadyOverflowed
    );
}

#[test]
fn fork_limit_is_strict_discards_prior_outputs_and_routes_against_original() {
    let plan = evaluate_standard_redirect(
        3,
        false,
        [
            ModifierResult::Outputs(vec![1, 2]),
            ModifierResult::Outputs(vec![3]),
        ],
    );
    assert!(plan.aborted);
    assert!(plan.outputs.is_empty());
    assert_eq!(plan.automatic_cost, 1);
    assert_eq!(
        plan.errors[0].kind,
        RedirectErrorKind::ForkLimit { limit: 3 }
    );
    assert_eq!(plan.errors[0].source, ErrorSource::OriginalCommandSource);
    assert!(plan.errors[0].tracer_receives_error);
    assert!(plan.errors[0].user_facing_failure);

    let zero = evaluate_standard_redirect(0, false, [ModifierResult::<u8>::Outputs(Vec::new())]);
    assert!(zero.aborted);
    assert_eq!(
        CUSTOM_REDIRECT_PLAN,
        CustomRedirectPlan {
            automatic_cost: 0,
            generic_fork_limit_checked: false,
            returns_from_generic_stage: true,
        }
    );
}

#[test]
fn forked_modifier_errors_continue_and_suppress_only_user_failure() {
    let plan = evaluate_standard_redirect(
        4,
        true,
        [
            ModifierResult::SyntaxError,
            ModifierResult::Outputs(vec![7]),
        ],
    );
    assert!(!plan.aborted);
    assert_eq!(plan.outputs, [7]);
    assert_eq!(
        plan.errors[0].source,
        ErrorSource::CurrentSource { index: 0 }
    );
    assert!(plan.errors[0].tracer_receives_error);
    assert!(!plan.errors[0].user_facing_failure);
    assert!(plan.executable_scheduled);
}

#[test]
fn chain_zero_warns_without_lookup_and_final_terminator_still_warns() {
    let mut inspections = 0;
    let zero = execute_chain(
        pos(0, 0, 0),
        Direction::East,
        0,
        |_, _| {
            inspections += 1;
            chain_cell(Direction::East)
        },
        9,
    );
    assert_eq!(inspections, 0);
    assert_eq!(zero.remaining_counter, -1);
    assert_eq!(zero.warning_limit, Some(9));
    assert!(!zero.initiator_counted);

    let terminating = execute_chain(
        pos(0, 0, 0),
        Direction::East,
        1,
        |_, _| ChainCell::terminating(Direction::North),
        4,
    );
    assert_eq!(terminating.visits.len(), 1);
    assert_eq!(
        terminating.visits[0].outcome,
        ChainVisitOutcome::TerminatingPosition
    );
    assert_eq!(terminating.warning_limit, Some(4));
}

#[test]
fn chain_steps_turn_skip_reset_execute_and_stop_without_pooling_context_cost() {
    let cells = [
        ChainCell {
            powered: false,
            automatic: false,
            next_facing: Direction::South,
            ..chain_cell(Direction::South)
        },
        ChainCell {
            condition_met: false,
            conditional: true,
            next_facing: Direction::West,
            ..chain_cell(Direction::West)
        },
        chain_cell(Direction::North),
        ChainCell {
            command_succeeded: false,
            ..chain_cell(Direction::East)
        },
    ];
    let mut index = 0;
    let plan = execute_chain(
        pos(0, 0, 0),
        Direction::East,
        8,
        |_, _| {
            let cell = cells[index];
            index += 1;
            cell
        },
        -2,
    );
    assert_eq!(
        plan.visits
            .iter()
            .map(|visit| visit.position)
            .collect::<Vec<_>>(),
        [pos(1, 0, 0), pos(1, 0, 1), pos(0, 0, 1), pos(0, 0, 0),]
    );
    assert_eq!(
        plan.visits
            .iter()
            .map(|visit| visit.outcome)
            .collect::<Vec<_>>(),
        [
            ChainVisitOutcome::SkippedUnpowered,
            ChainVisitOutcome::ConditionFailed {
                success_count_reset: true
            },
            ChainVisitOutcome::Executed {
                comparator_updated: true
            },
            ChainVisitOutcome::CommandReturnedFalse,
        ]
    );
    assert_eq!(plan.remaining_counter, 4);
    assert_eq!(plan.warning_limit, None);
}
