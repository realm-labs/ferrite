use ferrite_gameplay::mob::runtime::mob_004::anger::{
    NeutralAngerInput, NeutralKind, PiglinTarget, ResolveTargetInput, ResolvedAttackTarget,
    Retaliation, RevengeAdmission, group_alert_query, guarded_container_target,
    guarded_piglin_admitted, neutral_is_angry_at, piglin_anger_write, reset_commit,
    reset_goal_admission, reset_registration, resolve_attack_target, retaliation,
    revenge_admission, suppressed_event_remains_unconsumed,
};
use ferrite_gameplay::mob::runtime::mob_004::brain::{
    BRAIN_TICK_ORDER, BehaviorState, BehaviorStatus, MemorySlot, MemoryTick, MemoryWrite,
    activity_transition, behavior_duration, behavior_stops, first_valid_activity,
    initial_sensor_delay, schedule_refresh_due, sensor_tick, sight_query, tick_memory,
    try_start_behavior, write_memory,
};
use ferrite_gameplay::mob::runtime::mob_004::controls::{
    MoveOperation, MoveToInput, jump_control_tick, jumping_continues, look_control_tick,
    move_to_control, strafe_control,
};
use ferrite_gameplay::mob::runtime::mob_004::navigation::{
    MoveToPath, NAVIGATION_TICK_ORDER, NODE_CHANGE, PathAlternative, PathCreation,
    choose_alternative, corner_cut_allowed, displacement_stuck, expansion_allowed,
    expected_node_ticks, move_to_path, node_timed_out, path_budgets, path_creation, recompute,
    search_neighbor, target_reached, waypoint_reached,
};
use ferrite_gameplay::mob::runtime::mob_004::selector::{
    AiStep, GoalState, JUMP, LOOK, MOVE, SelectorPhase, adjusted_tick_delay, ai_step,
    passenger_disabled_flags, selector_phase, tick_selector,
};

fn goal(priority: i32, flags: u8) -> GoalState {
    GoalState {
        priority,
        flags,
        running: false,
        interruptible: true,
        requires_every_tick: false,
        can_continue: true,
        can_use: true,
    }
}

#[test]
fn ai_phase_order_and_entity_id_parity_are_exact() {
    assert_eq!(selector_phase(1, 0), SelectorPhase::Full);
    assert_eq!(selector_phase(2, 1), SelectorPhase::Reduced);
    assert_eq!(selector_phase(2, 2), SelectorPhase::Full);
    assert_eq!(
        ai_step(5, 1, true),
        Some(AiStep {
            increment_no_action_time: true,
            clear_sensing_cache: true,
            target_selector: SelectorPhase::Full,
            goal_selector: SelectorPhase::Full,
            tick_navigation_before_custom_ai: true,
            controls_order_move_look_jump: true,
            refresh_passenger_flags: true,
        })
    );
    assert_eq!(ai_step(5, 1, false), None);
    assert_eq!(passenger_disabled_flags(true, false), MOVE | LOOK | JUMP);
    assert_eq!(passenger_disabled_flags(false, true), JUMP);
}

#[test]
fn adjusted_goal_delay_keeps_every_tick_and_positive_ceiling_rules() {
    assert_eq!(adjusted_tick_delay(7, true), 7);
    assert_eq!(adjusted_tick_delay(7, false), 4);
    assert_eq!(adjusted_tick_delay(0, false), 1);
}

#[test]
fn full_selector_cleans_then_strictly_lower_interruptible_goal_preempts() {
    let mut goals = [
        GoalState {
            running: true,
            ..goal(5, MOVE)
        },
        goal(5, MOVE),
        goal(4, MOVE),
    ];
    let outcome = tick_selector(&mut goals, 0, SelectorPhase::Full);
    assert_eq!(outcome.started_in_order, vec![2]);
    assert_eq!(outcome.stopped_in_order, vec![0]);
    assert!(!goals[0].running && !goals[1].running && goals[2].running);
    assert_eq!(outcome.ticked_in_order, vec![2]);
}

#[test]
fn disabled_disjoint_empty_and_reduced_goal_paths_do_not_overreach() {
    let mut goals = [
        GoalState {
            running: true,
            can_continue: false,
            ..goal(1, MOVE)
        },
        goal(1, LOOK),
        goal(1, 0),
    ];
    let full = tick_selector(&mut goals, MOVE, SelectorPhase::Full);
    assert_eq!(full.stopped_in_order, vec![0]);
    assert_eq!(full.started_in_order, vec![1, 2]);

    goals[1].can_continue = false;
    goals[1].requires_every_tick = false;
    goals[2].requires_every_tick = true;
    let reduced = tick_selector(&mut goals, LOOK, SelectorPhase::Reduced);
    assert!(reduced.stopped_in_order.is_empty());
    assert_eq!(reduced.ticked_in_order, vec![2]);
    assert!(goals[1].running);
}

#[test]
fn memory_ttl_zero_expires_at_next_brain_tick_and_writes_fail_closed() {
    let mut slot = MemorySlot {
        registered: true,
        populated: true,
        ttl: Some(1),
    };
    assert_eq!(tick_memory(&mut slot), MemoryTick::Retained);
    assert_eq!(slot.ttl, Some(0));
    assert_eq!(tick_memory(&mut slot), MemoryTick::Expired);
    assert!(!slot.populated);

    let mut unregistered = MemorySlot {
        registered: false,
        populated: false,
        ttl: None,
    };
    assert_eq!(
        write_memory(&mut unregistered, true, Some(9)),
        MemoryWrite::IgnoredUnregistered
    );
    assert!(!unregistered.populated);
    slot.registered = true;
    assert_eq!(write_memory(&mut slot, false, None), MemoryWrite::Cleared);
}

#[test]
fn behavior_duration_is_inclusive_and_end_timestamp_equality_is_active() {
    assert_eq!(behavior_duration(20, 39, 0), 20);
    assert_eq!(behavior_duration(20, 39, 19), 39);
    let mut behavior = BehaviorState {
        status: BehaviorStatus::Stopped,
        end_timestamp: 0,
    };
    assert!(try_start_behavior(&mut behavior, true, true, 100, 20));
    assert_eq!(behavior.end_timestamp, 120);
    assert!(!behavior_stops(behavior, 120, true));
    assert!(behavior_stops(behavior, 121, true));
    assert!(behavior_stops(behavior, 110, false));
}

#[test]
fn schedule_sensor_and_sight_caches_cross_their_strict_cadences() {
    assert!(!schedule_refresh_due(120, 100));
    assert!(schedule_refresh_due(121, 100));
    assert_eq!(initial_sensor_delay(20, 19), 19);
    let waiting = sensor_tick(2, 20);
    assert_eq!((waiting.next_time_to_tick, waiting.run), (1, false));
    let due = sensor_tick(1, 20);
    assert_eq!((due.next_time_to_tick, due.run), (20, true));
    assert!(due.rewrite_ranges_from_follow_range);
    assert!(sight_query(false, false).perform_clip);
    assert!(sight_query(true, false).use_cached_result);
    let fallback = activity_transition(false);
    assert!(fallback.use_default_activity && fallback.include_all_core_activities);
    assert_eq!(first_valid_activity(&[false, true, true]), Some(1));
    assert_eq!(
        (
            BRAIN_TICK_ORDER.expire_memories_first,
            BRAIN_TICK_ORDER.sensors_second,
            BRAIN_TICK_ORDER.tick_or_stop_running_last
        ),
        (true, true, true)
    );
}

#[test]
fn path_creation_and_visit_budgets_distinguish_initial_and_reset_values() {
    assert_eq!(
        path_creation(true, false, true, false),
        PathCreation::RejectEmptyTargets
    );
    assert_eq!(
        path_creation(false, false, true, true),
        PathCreation::ReuseLivePath
    );
    let initial = path_budgets(10.0, 16.0, 1.5, false);
    assert_eq!(
        (
            initial.max_path_length,
            initial.base_visit_budget,
            initial.adjusted_visit_budget
        ),
        (16.0, 160, 240)
    );
    let reset = path_budgets(10.0, 16.0, 1.0, true);
    assert_eq!(reset.base_visit_budget, 256);
    assert_eq!(move_to_path(false, false, false), MoveToPath::ClearAndFail);
    assert_eq!(
        move_to_path(true, true, false),
        MoveToPath::TrimCauldronsAndStart
    );
    assert_eq!(
        (
            NAVIGATION_TICK_ORDER.retry_delayed_recompute_first,
            NAVIGATION_TICK_ORDER.write_waypoint_to_move_control_last
        ),
        (true, true)
    );
}

#[test]
fn path_search_uses_strict_length_budget_and_reach_boundaries() {
    let neighbor = search_neighbor(2.0, 3.0, 4.0, 10.0, 15.999, 15.999, 16.0);
    assert_eq!((neighbor.g, neighbor.h), (9.0, 15.0));
    assert!(neighbor.expandable);
    assert!(!search_neighbor(0.0, 1.0, 0.0, 1.0, 16.0, 1.0, 16.0).expandable);
    assert!(expansion_allowed(9, 10));
    assert!(!expansion_allowed(10, 10));
    assert!(target_reached(4, 4));
}

#[test]
fn reached_paths_choose_node_count_and_fallbacks_choose_distance_then_count() {
    let alternatives = [
        PathAlternative {
            distance_to_target: 8.0,
            node_count: 10,
            reached: true,
        },
        PathAlternative {
            distance_to_target: 2.0,
            node_count: 5,
            reached: true,
        },
    ];
    assert_eq!(choose_alternative(&alternatives), Some(1));
    let unreached = [
        PathAlternative {
            distance_to_target: 2.0,
            node_count: 9,
            reached: false,
        },
        PathAlternative {
            distance_to_target: 2.0,
            node_count: 4,
            reached: false,
        },
    ];
    assert_eq!(choose_alternative(&unreached), Some(1));
}

#[test]
fn navigation_recompute_waypoint_stuck_and_timeout_endpoints_are_strict() {
    assert!(!recompute(120, 100, true).recompute_now);
    assert!(recompute(121, 100, true).recompute_now);
    assert!(recompute(121, 100, false).mark_delayed);
    assert!(waypoint_reached(0.49, -0.49, 0.99, 0.5, 1.0));
    assert!(!waypoint_reached(0.5, 0.0, 0.0, 0.5, 1.0));
    assert!(!displacement_stuck(100, 0.0, 1.0));
    assert!(displacement_stuck(101, 24.999, 1.0));
    assert_eq!(expected_node_ticks(2.0, 0.0), None);
    let expected = expected_node_ticks(2.0, 0.5);
    assert!(!node_timed_out(240.0, expected));
    assert!(node_timed_out(240.001, expected));
    assert_eq!(
        (
            NODE_CHANGE.recompute_expected_limit,
            NODE_CHANGE.reset_accumulated_timeout
        ),
        (true, false)
    );
    assert!(!corner_cut_allowed(true, false, false, true));
}

#[test]
fn base_move_look_and_jump_controls_consume_one_shot_requests() {
    let strafe = strafe_control(3.0, 4.0, 0.8, true);
    assert!((strafe.forward - 0.6).abs() < f32::EPSILON);
    assert!((strafe.sideways - 0.8).abs() < f32::EPSILON);
    assert_eq!(strafe.next_operation, MoveOperation::Wait);
    let blocked = strafe_control(0.0, 1.0, 1.0, false);
    assert_eq!((blocked.forward, blocked.sideways), (1.0, 0.0));
    let move_to = move_to_control(MoveToInput {
        distance_squared: 1.0,
        speed_modifier: 2.0,
        movement_speed: 0.25,
        high_close_target: false,
        obstructing_shape: true,
        door: false,
        fence: false,
    });
    assert_eq!(move_to.next_operation, MoveOperation::Jumping);
    assert_eq!(move_to.speed, 0.5);
    assert!(!jumping_continues(true, false));
    let look = look_control_tick(0, true);
    assert_eq!(look.turn_head_to_body_degrees, 10);
    let jump = jump_control_tick(true);
    assert!(jump.entity_jumping && !jump.stored_request_after_tick);
}

#[test]
fn revenge_suppression_keeps_the_hurt_event_for_a_later_rule_toggle() {
    assert_eq!(
        revenge_admission(10, 10, true, true, true),
        RevengeAdmission::NoNewHurt
    );
    let suppressed = revenge_admission(11, 10, true, true, true);
    assert_eq!(suppressed, RevengeAdmission::SuppressedUniversalPlayer);
    assert!(suppressed_event_remains_unconsumed(suppressed));
    assert_eq!(
        revenge_admission(11, 10, true, true, false),
        RevengeAdmission::CheckIgnoreClassesAndCombatTargeting
    );
}

#[test]
fn classic_neutral_universal_matching_is_targetless_live_and_strictly_timed() {
    let base = NeutralAngerInput {
        can_attack: true,
        candidate_player: true,
        creative_or_spectator: false,
        peaceful: false,
        universal_anger: true,
        anger_end_time: 101,
        game_time: 100,
        persistent_target_present: false,
        persistent_target_matches: false,
    };
    assert!(neutral_is_angry_at(base));
    assert!(!neutral_is_angry_at(NeutralAngerInput {
        anger_end_time: 100,
        ..base
    }));
    assert!(!neutral_is_angry_at(NeutralAngerInput {
        universal_anger: false,
        ..base
    }));
    assert!(neutral_is_angry_at(NeutralAngerInput {
        candidate_player: false,
        universal_anger: false,
        persistent_target_present: true,
        persistent_target_matches: true,
        ..base
    }));
}

#[test]
fn classic_reset_registrations_and_group_timer_draws_are_exact() {
    let admission = reset_goal_admission(true, true, 11, 10);
    assert!(admission.admitted && admission.consume_hurt_event_on_start_only);
    assert!(!reset_goal_admission(true, true, 10, 10).admitted);
    assert_eq!(reset_commit(0).new_targetless_duration, 400);
    assert_eq!(reset_commit(380).new_targetless_duration, 780);
    assert_eq!(
        (
            reset_registration(NeutralKind::Bee).priority,
            reset_registration(NeutralKind::Bee).group_alert
        ),
        (3, true)
    );
    assert_eq!(reset_registration(NeutralKind::IronGolem).priority, 4);
    assert_eq!(reset_registration(NeutralKind::PolarBear).priority, 5);
    assert_eq!(reset_registration(NeutralKind::Wolf).priority, 8);
    assert_eq!(reset_registration(NeutralKind::Enderman).priority, 4);
    assert!(reset_registration(NeutralKind::ZombifiedPiglin).group_alert);
    let query = group_alert_query(32.0);
    assert_eq!(
        (query.horizontal_inflation, query.vertical_inflation),
        (32.0, 10.0)
    );
    assert!(query.reset_every_returned_peer_in_order);
}

#[test]
fn piglin_container_selection_and_common_setter_reread_live_rule() {
    assert!(guarded_piglin_admitted(true, true, true));
    assert!(!guarded_piglin_admitted(true, true, false));
    assert!(!guarded_piglin_admitted(false, false, true));
    assert_eq!(
        guarded_container_target(false, true),
        PiglinTarget::TriggeringPlayer
    );
    assert_eq!(
        guarded_container_target(true, true),
        PiglinTarget::NearestVisibleAttackablePlayer
    );
    let write = piglin_anger_write(true, true, true).unwrap();
    assert!(write.erase_cant_reach_since && write.write_universal_anger);
    assert_eq!((write.angry_at_ttl, write.universal_anger_ttl), (600, 600));
    assert!(
        !piglin_anger_write(true, true, false)
            .unwrap()
            .write_universal_anger
    );
    assert_eq!(piglin_anger_write(false, true, true), None);
}

#[test]
fn piglin_retaliation_and_later_target_resolution_use_distinct_precedence() {
    assert_eq!(
        retaliation(true, true, false, true, true),
        Retaliation::RejectAvoid
    );
    assert_eq!(
        retaliation(false, true, false, true, true),
        Retaliation::UniversalPlayer
    );
    assert_eq!(
        retaliation(false, true, false, false, true),
        Retaliation::ExactAttacker
    );
    let base = ResolveTargetInput {
        near_zombified: false,
        angry_at_resolves_attackable: true,
        universal_memory_present: true,
        nearest_visible_attackable_player_present: true,
        nearest_nemesis_present: true,
        nearest_non_gold_player_attackable: true,
    };
    assert_eq!(resolve_attack_target(base), ResolvedAttackTarget::AngryAt);
    assert_eq!(
        resolve_attack_target(ResolveTargetInput {
            angry_at_resolves_attackable: false,
            ..base
        }),
        ResolvedAttackTarget::UniversalNearestPlayer
    );
    assert_eq!(
        resolve_attack_target(ResolveTargetInput {
            near_zombified: true,
            ..base
        }),
        ResolvedAttackTarget::NoneNearZombified
    );
}
