use ferrite_gameplay::mob::runtime::mob_001::hostile::{
    CreakingProtectorAttempt, Difficulty, EndermiteAttempt, NaturalCategory, PortalPiglinInput,
    ReinforcementCandidateFailure, category_survives_hostile_cache, creaking_protector_attempt,
    custom_spawner_policy, endermite_attempt, portal_piglin_attempt, refresh_hostile_policy,
    reinforcement_admission, reinforcement_candidate, reinforcement_offset, reinforcement_success,
};
use ferrite_gameplay::mob::runtime::mob_001::natural::{
    CATEGORY_ORDER, CHUNK_GENERATION_ATTEMPT, CandidateDistanceFailure, ConstructionDisposition,
    Heightmap, PackStop, PositionStartInput, REGISTERED_PLACEMENT_COUNT,
    SUCCESSFUL_SPAWN_ACCOUNTING, SnapshotInput, UNREGISTERED_PLACEMENT, below_global_cap,
    candidate_distance, chunk_center_candidate, chunk_generation_group_admitted,
    construction_disposition, creature_cadence, filtered_category, global_cap, hard_distance,
    local_cap_allows, pack_offset, pack_stop, position_start, potential_allows,
    provisional_attempts, selected_group_count, snapshot_accounting, spawn_list_selection,
};
use ferrite_gameplay::mob::runtime::mob_001::patrol::{
    PATROL_LEADER, PATROL_MEMBER_COMMIT, PatrolAttemptFailure, PatrolAttemptInput,
    PatrolMemberFailure, continue_group, group_count, leader_target_offset, member_walk,
    patrol_attempt, patrol_member, patrol_tick, player_offset, selected_player_index,
};
use ferrite_gameplay::mob::runtime::mob_001::phantom::{
    PHANTOM_MEMBER, difficulty_trial, insomnia_trial, level_sky_allows, phantom_candidate,
    phantom_group_count, phantom_tick, player_sky_gate,
};
use ferrite_gameplay::mob::runtime::mob_001::trader::{
    LLAMA_COMMIT, TRADER_CANDIDATE_ATTEMPTS, TRADER_COLLISION_CELLS, TRADER_COMMIT,
    TRADER_SEARCH_RADIUS, TraderCandidateFailure, TraderSavedState, chance_after_spawn,
    encounter_first_meeting_or_player, sampled_offset, trader_biome_allows, trader_candidate,
    trader_player_index, trader_player_selection, trader_saved_tick, trader_space_empty,
    trader_tick,
};
use ferrite_gameplay::mob::runtime::mob_001::warden::{
    Attribution, DARKNESS_APPLICATION, ExistingDarkness, GameMode, ReplySound, WARDEN_FINALIZATION,
    WARDEN_SEARCH, WarningAdmission, WarningTracker, attributed_server_player, can_respond,
    darkness_admitted, player_within_warning_radius, reply_offset, scheduled_shriek, shriek_commit,
    try_warn, vibration_admission, warden_horizontal_offset, warden_inside_suppression_box,
    warden_response,
};

#[test]
fn hostile_rule_conjunction_refreshes_cache_and_only_filters_monsters() {
    let enabled = refresh_hostile_policy(true, true);
    assert!(enabled.live && enabled.replace_chunk_cache);
    assert!(!refresh_hostile_policy(false, true).live);
    assert!(!refresh_hostile_policy(true, false).live);
    assert!(!category_survives_hostile_cache(
        NaturalCategory::Monster,
        false
    ));
    assert!(category_survives_hostile_cache(
        NaturalCategory::Creature,
        false
    ));

    let custom = custom_spawner_policy(false, false);
    assert!(!custom.patrol && !custom.phantom);
    assert!(custom.village_siege_done && custom.clear_village_siege_setup);
    assert!(custom.cat && custom.wandering_trader);
}

#[test]
fn endermite_chance_precedes_live_rule_and_never_stops_pearl_completion() {
    let equality = endermite_attempt(true, true, 0.05, true, Difficulty::Hard);
    assert!(equality.chance_draw_consumed && !equality.construct);
    assert!(equality.continue_pearl_transaction);
    let gated = endermite_attempt(true, true, 0.0, false, Difficulty::Hard);
    assert!(gated.chance_draw_consumed && !gated.construct);
    let success = endermite_attempt(true, true, 0.049, true, Difficulty::Easy);
    assert_eq!(
        success,
        EndermiteAttempt {
            chance_draw_consumed: true,
            construct: true,
            copy_owner_position_and_rotation: true,
            reason_triggered: true,
            retry_on_failure: false,
            continue_pearl_transaction: true,
        }
    );
}

#[test]
fn reinforcement_admission_offsets_and_candidate_rejections_are_ordered() {
    let not_hard = reinforcement_admission(true, true, Difficulty::Normal, 0.0, 1.0, true);
    assert!(!not_hard.chance_draw_consumed && !not_hard.construct);
    let rule_gated = reinforcement_admission(true, true, Difficulty::Hard, 0.0, 1.0, false);
    assert!(rule_gated.chance_draw_consumed && !rule_gated.construct);
    let admitted = reinforcement_admission(true, true, Difficulty::Hard, 0.2, 0.3, true);
    assert!(admitted.construct);
    assert_eq!((admitted.attempts, admitted.draws_per_attempt), (50, 6));
    assert_eq!(reinforcement_offset(0, 0), -7);
    assert_eq!(reinforcement_offset(33, 1), 0);
    assert_eq!(reinforcement_offset(33, 2), 40);

    assert_eq!(
        reinforcement_candidate(false, false, true, false, false, false, true),
        Err(ReinforcementCandidateFailure::PlacementPosition)
    );
    assert_eq!(
        reinforcement_candidate(true, true, true, true, true, false, false),
        Err(ReinforcementCandidateFailure::NearbyAlivePlayer)
    );
    assert!(reinforcement_candidate(true, true, false, true, true, false, false).is_ok());
    let commit = reinforcement_success(None);
    assert_eq!(commit.caller_permanent_add, -0.05);
    assert!(!commit.rollback_modifiers_on_insert_failure);
}

#[test]
fn creaking_and_portal_gates_keep_rng_on_the_source_side_of_each_rule() {
    let heart = creaking_protector_attempt(4, false, false, true, true, Difficulty::Normal, true);
    assert_eq!(
        heart,
        CreakingProtectorAttempt {
            ticker_draw_consumed: true,
            ticker: 24,
            query_nearest_player: true,
            call_spawn_util: true,
            spawn_attempts: 5,
            horizontal_range: 16,
            vertical_range: 8,
        }
    );
    assert!(
        !creaking_protector_attempt(0, false, true, true, false, Difficulty::Hard, true)
            .query_nearest_player
    );

    let portal = portal_piglin_attempt(PortalPiglinInput {
        live_hostile_policy: true,
        difficulty: Difficulty::Normal,
        environment_attribute: true,
        chance_draw_below_2000: 1,
        player_close_enough: true,
        valid_ground: true,
        creation_succeeded: true,
    });
    assert!(portal.chance_draw_consumed && portal.construct);
    assert!(portal.set_entity_and_vehicle_cooldown);
    assert!(
        !portal_piglin_attempt(PortalPiglinInput {
            environment_attribute: false,
            ..PortalPiglinInput {
                live_hostile_policy: true,
                difficulty: Difficulty::Hard,
                environment_attribute: true,
                chance_draw_below_2000: 0,
                player_close_enough: true,
                valid_ground: true,
                creation_succeeded: true,
            }
        })
        .chance_draw_consumed
    );
}

#[test]
fn natural_category_order_caps_and_filters_use_the_tracker_union_denominator() {
    assert_eq!(CATEGORY_ORDER[0], NaturalCategory::Monster);
    assert_eq!(CATEGORY_ORDER[6], NaturalCategory::WaterAmbient);
    assert_eq!(global_cap(NaturalCategory::Monster, 0), 0);
    assert_eq!(global_cap(NaturalCategory::Monster, 288), 69);
    assert_eq!(global_cap(NaturalCategory::Monster, 289), 70);
    assert!(below_global_cap(NaturalCategory::Monster, 289, 69));
    assert!(!below_global_cap(NaturalCategory::Monster, 289, 70));
    assert!(!filtered_category(NaturalCategory::Monster, false, true));
    assert!(!filtered_category(NaturalCategory::Creature, true, false));
    assert!(filtered_category(NaturalCategory::Ambient, false, false));
    assert!(creature_cadence(400));
    assert!(!creature_cadence(399));
    assert_eq!(hard_distance(NaturalCategory::WaterAmbient), 64);
    assert_eq!(hard_distance(NaturalCategory::Monster), 128);
}

#[test]
fn snapshot_and_local_counts_distinguish_persistent_mobs_and_nonmobs() {
    let persistent = snapshot_accounting(SnapshotInput {
        misc_category: false,
        mob: true,
        persistence_required: true,
        custom_persistence: false,
        containing_chunk_queryable: true,
        spawn_cost_defined: true,
    });
    assert!(!persistent.count_global && !persistent.count_for_nearby_players);
    let nonmob = snapshot_accounting(SnapshotInput {
        misc_category: false,
        mob: false,
        persistence_required: false,
        custom_persistence: false,
        containing_chunk_queryable: true,
        spawn_cost_defined: true,
    });
    assert!(nonmob.count_global && nonmob.add_potential_charge);
    assert!(!nonmob.count_for_nearby_players);
    assert!(!local_cap_allows(NaturalCategory::Creature, &[]));
    assert!(local_cap_allows(
        NaturalCategory::Creature,
        &[Some(10), None]
    ));
    assert!(!local_cap_allows(
        NaturalCategory::Creature,
        &[Some(10), Some(11)]
    ));
}

#[test]
fn natural_chunk_player_and_potential_boundaries_are_strict() {
    assert!(chunk_center_candidate(16_383.0, true));
    assert!(!chunk_center_candidate(16_384.0, true));
    assert!(!chunk_center_candidate(0.0, false));
    assert!(potential_allows(2.0, 3.0, 6.0));
    assert!(!potential_allows(2.0, 3.01, 6.0));
    assert_eq!(
        candidate_distance(true, 576.0, false, 0.0, false, true),
        Err(CandidateDistanceFailure::PlayerWithinTwentyFour)
    );
    assert_eq!(
        candidate_distance(true, 577.0, true, 576.0, false, true),
        Err(CandidateDistanceFailure::RespawnWithinTwentyFour)
    );
    assert!(candidate_distance(true, 577.0, false, 0.0, true, true).is_ok());
}

#[test]
fn natural_position_and_pack_random_formulas_preserve_zero_attempts() {
    let start = position_start(PositionStartInput {
        chunk_min_x: 32,
        chunk_min_z: -16,
        x_draw: 15,
        z_draw: 0,
        min_y: -64,
        surface_plus_one: 10,
        y_draw: 1,
        starting_block_conductor: false,
    });
    assert_eq!((start.x, start.z), (47, -16));
    assert!(start.admitted);
    assert_eq!(provisional_attempts(0.0), 0);
    assert_eq!(provisional_attempts(0.01), 1);
    assert_eq!(provisional_attempts(1.0), 4);
    assert_eq!(pack_offset(0, 5), -5);
    assert_eq!(pack_offset(5, 0), 5);
    assert_eq!(selected_group_count(2, 4, 2), 4);
}

#[test]
fn spawn_list_selection_construction_and_accounting_keep_failure_boundaries() {
    let reduced = spawn_list_selection(
        NaturalCategory::WaterAmbient,
        true,
        0.979,
        false,
        false,
        true,
    );
    assert!(reduced.reduced_water_draw_consumed && reduced.end_group);
    let equality = spawn_list_selection(
        NaturalCategory::WaterAmbient,
        true,
        0.98,
        false,
        false,
        true,
    );
    assert!(equality.select_weighted_entry && !equality.end_group);
    assert!(
        spawn_list_selection(NaturalCategory::Monster, false, 0.0, true, true, true)
            .use_fortress_list
    );
    assert_eq!(
        construction_disposition(true, false, false, false),
        ConstructionDisposition::EndCategoryPosition
    );
    assert_eq!(
        construction_disposition(true, true, true, true),
        ConstructionDisposition::FinalizeAndAccount
    );
    assert_eq!(
        (
            SUCCESSFUL_SPAWN_ACCOUNTING.account_after_insertion_call,
            SUCCESSFUL_SPAWN_ACCOUNTING.rollback_on_insertion_failure
        ),
        (true, false)
    );
    assert_eq!(pack_stop(4, 4, 1, true), PackStop::EndAllWalks);
    assert_eq!(pack_stop(3, 4, 2, true), PackStop::EndCurrentWalk);
}

#[test]
fn placement_and_chunk_generation_contracts_keep_registered_and_fallback_shapes() {
    assert_eq!(REGISTERED_PLACEMENT_COUNT, 83);
    assert_eq!(
        (
            UNREGISTERED_PLACEMENT.predicate,
            UNREGISTERED_PLACEMENT.heightmap
        ),
        (true, Heightmap::MotionBlockingNoLeaves)
    );
    assert_eq!(
        (
            CHUNK_GENERATION_ATTEMPT.attempts_per_member,
            CHUNK_GENERATION_ATTEMPT.horizontal_walk_draws,
            CHUNK_GENERATION_ATTEMPT.continue_after_null_or_exception
        ),
        (4, 4, true)
    );
    assert!(chunk_generation_group_admitted(true, 0.2, 0.3));
    assert!(!chunk_generation_group_admitted(false, 0.0, 1.0));
}

#[test]
fn patrol_timer_pauses_and_distinguishes_fresh_from_later_expiry() {
    let paused = patrol_tick(7, false, true, 0);
    assert_eq!(paused.next_tick, 7);
    assert!(!paused.timer_changed && !paused.schedule_draw_consumed);
    let fresh = patrol_tick(0, true, true, 0);
    assert_eq!(fresh.next_tick, 11_999);
    assert!(fresh.attempt_due && fresh.schedule_draw_consumed);
    assert_eq!(patrol_tick(1, true, true, 0).next_tick, 12_000);
    assert_eq!(patrol_tick(0, true, true, 1_199).next_tick, 13_198);
}

#[test]
fn patrol_attempt_selects_one_player_and_commits_late_failures() {
    let base = PatrolAttemptInput {
        bright_outside: true,
        chance_draw_below_five: 0,
        player_count: 2,
        selected_spectator: false,
        close_to_village: false,
        initial_square_loaded: true,
        environment_attribute: true,
    };
    assert!(patrol_attempt(base).is_ok());
    assert_eq!(
        patrol_attempt(PatrolAttemptInput {
            selected_spectator: true,
            ..base
        }),
        Err(PatrolAttemptFailure::SelectedSpectator)
    );
    assert_eq!(selected_player_index(3, 2), Some(1));
    assert_eq!(selected_player_index(0, 0), None);
    assert_eq!(player_offset(0, false), -24);
    assert_eq!(player_offset(23, true), 47);
}

#[test]
fn patrol_group_member_and_leader_boundaries_are_source_ordered() {
    assert_eq!(group_count(0.0), 1);
    assert_eq!(group_count(1.01), 3);
    assert_eq!(member_walk(4, 0), 4);
    assert_eq!(
        patrol_member(true, 9, true, true),
        Err(PatrolMemberFailure::BlockLight)
    );
    assert!(patrol_member(true, 8, true, true).is_ok());
    assert!(!continue_group(0, false));
    assert!(continue_group(1, false));
    assert_eq!(
        (
            PATROL_LEADER.target_before_placement,
            PATROL_LEADER.target_draws_from_entity_rng
        ),
        (true, 2)
    );
    assert_eq!(leader_target_offset(0), -500);
    assert_eq!(leader_target_offset(999), 499);
    assert_eq!(
        (
            PATROL_MEMBER_COMMIT.observe_insertion_result,
            PATROL_MEMBER_COMMIT.insert_with_passengers
        ),
        (false, true)
    );
}

#[test]
fn phantom_timer_sky_and_player_gates_preserve_draw_order() {
    assert_eq!(phantom_tick(0, true, true, 0).next_tick, 1_199);
    assert_eq!(phantom_tick(1, true, true, 59).next_tick, 2_380);
    assert_eq!(phantom_tick(8, false, true, 0).next_tick, 8);
    assert!(!level_sky_allows(true, 4));
    assert!(level_sky_allows(true, 5));
    assert!(level_sky_allows(false, 0));
    assert!(!player_sky_gate(true, true, 100, 63, true).eligible);
    assert!(!player_sky_gate(false, true, 62, 63, true).consume_difficulty_draw);
    assert!(player_sky_gate(false, false, 0, 63, false).eligible);
}

#[test]
fn phantom_difficulty_and_insomnia_trials_cross_strict_endpoints() {
    assert!(!difficulty_trial(1.5, 0.5));
    assert!(difficulty_trial(1.500_001, 0.5));
    assert!(!insomnia_trial(72_000, 71_999));
    assert!(insomnia_trial(72_001, 72_000));
    assert!(!insomnia_trial(0, 0));
    let candidate = phantom_candidate(14, 0, 20);
    assert_eq!(
        (candidate.x_offset, candidate.y_offset, candidate.z_offset),
        (-10, 34, 10)
    );
    assert_eq!(phantom_group_count(Difficulty::Easy, 1), 2);
    assert_eq!(phantom_group_count(Difficulty::Hard, 3), 4);
    assert_eq!(
        (
            PHANTOM_MEMBER.anchor_y_offset,
            PHANTOM_MEMBER.insertion_result_ignored
        ),
        (5, true)
    );
}

#[test]
fn trader_outer_timer_pauses_and_saved_timer_defaults_are_exact() {
    assert_eq!(TraderSavedState::default().spawn_delay, 24_000);
    assert_eq!(TraderSavedState::default().spawn_chance, 25);
    let paused = trader_tick(1_200, false);
    assert_eq!(paused.tick_delay, 1_200);
    assert!(!paused.timer_changed);
    assert_eq!(trader_tick(1_200, true).tick_delay, 1_199);
    let due = trader_tick(1, true);
    assert_eq!(due.tick_delay, 1_200);
    assert!(due.load_saved_state);
}

#[test]
fn trader_saved_chance_is_inclusive_escalating_and_nonrollback() {
    let wait = trader_saved_tick(
        TraderSavedState {
            spawn_delay: 2_400,
            spawn_chance: 25,
        },
        0,
    );
    assert_eq!(wait.state.spawn_delay, 1_200);
    assert!(!wait.consume_chance_draw);
    let due = trader_saved_tick(
        TraderSavedState {
            spawn_delay: 1_200,
            spawn_chance: 25,
        },
        25,
    );
    assert!(due.attempt_spawn);
    assert_eq!(
        (due.state.spawn_delay, due.state.spawn_chance),
        (24_000, 50)
    );
    assert!(
        !trader_saved_tick(
            TraderSavedState {
                spawn_delay: 0,
                spawn_chance: 25,
            },
            26
        )
        .attempt_spawn
    );
    assert_eq!(chance_after_spawn(false, 75), 75);
    assert_eq!(chance_after_spawn(true, 75), 25);
}

#[test]
fn trader_player_meeting_and_candidate_selection_keep_quirks() {
    let empty = trader_player_selection(0, 9);
    assert!(empty.spawn_returns_true && empty.reset_chance);
    assert!(!empty.consume_one_in_ten_draw);
    assert!(trader_player_selection(1, 0).proceed);
    assert!(!trader_player_selection(1, 1).proceed);
    assert_eq!(trader_player_index(5, 2), Some(1));
    assert_eq!(
        encounter_first_meeting_or_player(Some((1, 2, 3)), (9, 9, 9)),
        (1, 2, 3)
    );
    assert_eq!(sampled_offset(48, 0), -48);
    assert_eq!(sampled_offset(48, 95), 47);
    assert_eq!(
        trader_candidate(false, true, true, true),
        Err(TraderCandidateFailure::WorldBorder)
    );
    assert!(trader_candidate(true, true, true, true).is_ok());
}

#[test]
fn trader_space_llamas_and_post_insert_state_are_fixed() {
    assert!(!trader_space_empty(&[true; 11]));
    assert!(trader_space_empty(&[true; 12]));
    let mut blocked = [true; 12];
    blocked[7] = false;
    assert!(!trader_space_empty(&blocked));
    assert!(trader_biome_allows(false));
    assert!(!trader_biome_allows(true));
    assert_eq!(TRADER_CANDIDATE_ATTEMPTS, 10);
    assert_eq!(TRADER_SEARCH_RADIUS, 48);
    assert_eq!(TRADER_COLLISION_CELLS, 12);
    assert_eq!(
        (
            TRADER_COMMIT.llama_calls,
            TRADER_COMMIT.insertion_result_ignored
        ),
        (2, true)
    );
    assert_eq!(TRADER_COMMIT.trader_despawn_delay, 48_000);
    assert_eq!(LLAMA_COMMIT.constructor_despawn_delay, 47_999);
    assert_eq!(
        (
            LLAMA_COMMIT.use_trader_heightmap_and_placement,
            LLAMA_COMMIT.leash_with_broadcast
        ),
        (true, true)
    );
}

#[test]
fn warden_attribution_vibration_and_response_gates_are_independent_of_spawn_mobs() {
    assert!(attributed_server_player(Attribution::DirectServerPlayer));
    assert!(attributed_server_player(Attribution::ControllingPassenger));
    assert!(attributed_server_player(Attribution::ProjectileOwner));
    assert!(attributed_server_player(Attribution::ItemOwner));
    assert!(!attributed_server_player(Attribution::None));
    assert!(can_respond(true, Difficulty::Hard, true));
    assert!(!can_respond(true, Difficulty::Peaceful, true));
    let vibration = vibration_admission(true, true, false, true);
    assert!(vibration.listen && vibration.prefer_projectile_owner);
    assert_eq!(vibration.radius, 8);
    assert!(!vibration_admission(false, true, false, true).listen);
}

#[test]
fn warden_warning_radius_suppression_and_tracker_sync_use_strict_boundaries() {
    assert!(player_within_warning_radius(255.999));
    assert!(!player_within_warning_radius(256.0));
    assert!(warden_inside_suppression_box(24.0, -24.0, 0.0));
    assert!(!warden_inside_suppression_box(24.001, 0.0, 0.0));
    let mut trackers = [
        WarningTracker {
            warning_level: 1,
            ..WarningTracker::default()
        },
        WarningTracker {
            warning_level: 3,
            ..WarningTracker::default()
        },
    ];
    assert_eq!(
        try_warn(false, &mut trackers),
        WarningAdmission::Admitted { warning_level: 4 }
    );
    assert_eq!(trackers[0], trackers[1]);
    assert_eq!(
        (trackers[0].warning_level, trackers[0].cooldown_ticks),
        (4, 200)
    );
    assert_eq!(try_warn(false, &mut trackers), WarningAdmission::Cooldown);
    assert_eq!(
        try_warn(true, &mut trackers),
        WarningAdmission::NearbyWarden
    );
}

#[test]
fn warden_tracker_tick_decays_on_the_12001st_tick_then_cools_down() {
    let mut tracker = WarningTracker {
        ticks_since_last_warning: 11_999,
        warning_level: 2,
        cooldown_ticks: 1,
    };
    tracker.tick();
    assert_eq!(
        tracker,
        WarningTracker {
            ticks_since_last_warning: 12_000,
            warning_level: 2,
            cooldown_ticks: 0,
        }
    );
    tracker.tick();
    assert_eq!(
        (tracker.ticks_since_last_warning, tracker.warning_level),
        (0, 1)
    );
}

#[test]
fn shriek_and_delayed_response_keep_warning_and_sound_rng_boundaries() {
    let gated = shriek_commit(None);
    assert_eq!(gated.local_warning_level, 0);
    assert_eq!((gated.set_shrieking_flags, gated.schedule_delay), (2, 90));
    assert_eq!(gated.level_event, 3007);
    let scheduled = scheduled_shriek(true);
    assert!(scheduled.clear_shrieking && scheduled.try_respond_after_clear);
    assert_eq!(scheduled.clear_flags, 3);

    let warning_three = warden_response(true, 3, false);
    assert!(!warning_three.attempt_spawn);
    assert_eq!(warning_three.reply_sound, Some(ReplySound::NearbyClosest));
    assert_eq!(warning_three.reply_offset_draws, 3);
    assert!(warning_three.apply_darkness);
    let success = warden_response(true, 4, true);
    assert!(success.attempt_spawn && success.apply_darkness);
    assert_eq!(success.reply_sound, None);
    assert_eq!(success.reply_offset_draws, 0);
    assert!(!warden_response(false, 4, true).apply_darkness);
}

#[test]
fn warden_search_finalization_and_reply_offsets_are_exact() {
    assert_eq!(reply_offset(0), -10);
    assert_eq!(reply_offset(20), 10);
    assert_eq!(warden_horizontal_offset(0), -5);
    assert_eq!(warden_horizontal_offset(10), 5);
    assert_eq!(
        (
            WARDEN_SEARCH.attempts,
            WARDEN_SEARCH.precreation_collision_check
        ),
        (20, false)
    );
    assert_eq!(WARDEN_SEARCH.ground_cells_per_attempt, 13);
    assert_eq!(WARDEN_FINALIZATION.dig_cooldown_ticks, 1_200);
    assert_eq!(WARDEN_FINALIZATION.emerging_memory_ticks, 134);
    assert_eq!(
        (
            WARDEN_FINALIZATION.play_agitated_before_superclass,
            WARDEN_FINALIZATION.discard_failed_constructed_candidate,
            WARDEN_FINALIZATION.insertion_result_ignored
        ),
        (true, true, true)
    );
}

#[test]
fn darkness_targets_modes_strict_radius_and_finite_refresh_endpoint() {
    assert!(darkness_admitted(GameMode::Survival, 1_599.0, None));
    assert!(darkness_admitted(
        GameMode::Adventure,
        0.0,
        Some(ExistingDarkness {
            amplifier: 4,
            duration: Some(199),
        })
    ));
    assert!(!darkness_admitted(
        GameMode::Survival,
        0.0,
        Some(ExistingDarkness {
            amplifier: 0,
            duration: Some(200),
        })
    ));
    assert!(!darkness_admitted(
        GameMode::Survival,
        0.0,
        Some(ExistingDarkness {
            amplifier: 0,
            duration: None,
        })
    ));
    assert!(!darkness_admitted(GameMode::Creative, 0.0, None));
    assert!(!darkness_admitted(GameMode::Spectator, 0.0, None));
    assert!(!darkness_admitted(GameMode::Survival, 1_600.0, None));
    assert_eq!(
        (
            DARKNESS_APPLICATION.duration,
            DARKNESS_APPLICATION.amplifier
        ),
        (260, 0)
    );
    assert_eq!(
        (DARKNESS_APPLICATION.ambient, DARKNESS_APPLICATION.particles),
        (false, false)
    );
}
