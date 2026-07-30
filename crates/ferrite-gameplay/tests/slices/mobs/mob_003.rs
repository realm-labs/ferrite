use ferrite_gameplay::mob::runtime::mob_003::{
    CustomPersistence, DespawnCategory, DespawnExit, DespawnInput, INVOCATION_POLICY,
    RemovalPolicy, SOFT_DISTANCE_SQUARED, check_despawn, hard_distance_squared,
    no_action_time_after_ai_step, remove_when_far_away, requires_custom_persistence,
};

fn input() -> DespawnInput {
    DespawnInput {
        peaceful: false,
        type_allowed_in_peaceful: true,
        stored_persistence: false,
        custom_persistence: false,
        nearest_nonspectator_player_present: true,
        category: DespawnCategory::Other,
        distance_squared: 0.0,
        no_action_time: 0,
        random_draw_below_eight_hundred: 1,
        remove_when_far_away: true,
    }
}

#[test]
fn peaceful_discard_precedes_every_persistence_and_distance_gate() {
    let outcome = check_despawn(DespawnInput {
        peaceful: true,
        type_allowed_in_peaceful: false,
        stored_persistence: true,
        custom_persistence: true,
        nearest_nonspectator_player_present: true,
        distance_squared: 100_000.0,
        no_action_time: 601,
        random_draw_below_eight_hundred: 0,
        ..input()
    });
    assert_eq!(outcome.exit, DespawnExit::PeacefulDiscard);
    assert!(outcome.discard_peaceful);
    assert!(!outcome.reset_no_action_time && !outcome.random_draw_consumed);
    assert_eq!(outcome.remove_policy_calls, 0);
}

#[test]
fn stored_or_custom_persistence_resets_inactivity_before_player_lookup() {
    let stored = check_despawn(DespawnInput {
        stored_persistence: true,
        nearest_nonspectator_player_present: false,
        no_action_time: 99,
        ..input()
    });
    assert_eq!(stored.exit, DespawnExit::PersistentReset);
    assert!(stored.reset_no_action_time);
    let custom = check_despawn(DespawnInput {
        custom_persistence: true,
        ..input()
    });
    assert_eq!(custom.exit, DespawnExit::PersistentReset);
}

#[test]
fn absent_nonspectator_player_preserves_timer_and_rng_cursor() {
    let outcome = check_despawn(DespawnInput {
        nearest_nonspectator_player_present: false,
        distance_squared: 100_000.0,
        no_action_time: 601,
        random_draw_below_eight_hundred: 0,
        ..input()
    });
    assert_eq!(outcome.exit, DespawnExit::NoPlayer);
    assert!(!outcome.reset_no_action_time);
    assert!(!outcome.random_draw_consumed);
    assert_eq!(outcome.remove_policy_calls, 0);
}

#[test]
fn hard_distances_are_strict_and_water_ambient_uses_sixty_four() {
    assert_eq!(
        hard_distance_squared(DespawnCategory::WaterAmbient),
        4_096.0
    );
    assert_eq!(hard_distance_squared(DespawnCategory::Other), 16_384.0);
    assert!(
        !check_despawn(DespawnInput {
            distance_squared: 16_384.0,
            ..input()
        })
        .discard_hard
    );
    assert!(
        check_despawn(DespawnInput {
            distance_squared: 16_385.0,
            ..input()
        })
        .discard_hard
    );
    assert!(
        check_despawn(DespawnInput {
            category: DespawnCategory::WaterAmbient,
            distance_squared: 4_097.0,
            ..input()
        })
        .discard_hard
    );
}

#[test]
fn hard_discard_still_reaches_soft_rng_and_calls_policy_twice() {
    let outcome = check_despawn(DespawnInput {
        distance_squared: 16_385.0,
        no_action_time: 601,
        random_draw_below_eight_hundred: 0,
        ..input()
    });
    assert!(outcome.discard_hard && outcome.discard_soft);
    assert!(outcome.continue_soft_after_hard_discard);
    assert!(outcome.random_draw_consumed);
    assert_eq!(outcome.remove_policy_calls, 2);
}

#[test]
fn soft_branch_draws_before_distance_and_policy_at_strict_timer_boundary() {
    let at_six_hundred = check_despawn(DespawnInput {
        distance_squared: SOFT_DISTANCE_SQUARED + 1.0,
        no_action_time: 600,
        random_draw_below_eight_hundred: 0,
        ..input()
    });
    assert!(!at_six_hundred.random_draw_consumed && !at_six_hundred.discard_soft);
    let near = check_despawn(DespawnInput {
        distance_squared: SOFT_DISTANCE_SQUARED - 1.0,
        no_action_time: 601,
        random_draw_below_eight_hundred: 0,
        ..input()
    });
    assert!(near.random_draw_consumed && near.reset_no_action_time);
    assert_eq!(near.remove_policy_calls, 0);
    let equality = check_despawn(DespawnInput {
        distance_squared: SOFT_DISTANCE_SQUARED,
        no_action_time: 601,
        random_draw_below_eight_hundred: 0,
        ..input()
    });
    assert!(!equality.discard_soft && !equality.reset_no_action_time);
}

#[test]
fn effective_ai_alone_increments_no_action_time_before_goals() {
    assert_eq!(no_action_time_after_ai_step(600, true), 601);
    assert_eq!(no_action_time_after_ai_step(600, false), 600);
    assert_eq!(no_action_time_after_ai_step(u32::MAX, true), u32::MAX);
}

#[test]
fn custom_persistence_catalog_keeps_base_and_subtype_additions_separate() {
    assert!(requires_custom_persistence(
        true,
        false,
        CustomPersistence::Base
    ));
    assert!(requires_custom_persistence(
        false,
        true,
        CustomPersistence::Base
    ));
    assert!(requires_custom_persistence(
        false,
        false,
        CustomPersistence::Fish { from_bucket: true }
    ));
    assert!(requires_custom_persistence(
        false,
        false,
        CustomPersistence::Nautilus { tamed: true }
    ));
    assert!(requires_custom_persistence(
        false,
        false,
        CustomPersistence::SulfurCube {
            body_item: true,
            from_bucket: false
        }
    ));
    assert!(requires_custom_persistence(
        false,
        false,
        CustomPersistence::Enderman {
            carrying_block: true
        }
    ));
    assert!(requires_custom_persistence(
        false,
        false,
        CustomPersistence::Raider { current_raid: true }
    ));
    assert!(!requires_custom_persistence(
        false,
        false,
        CustomPersistence::Base
    ));
}

#[test]
fn animal_fish_and_age_based_removal_overrides_are_exact() {
    assert!(!remove_when_far_away(RemovalPolicy::Never, 99_999.0));
    assert!(remove_when_far_away(
        RemovalPolicy::Chicken {
            chicken_jockey: true
        },
        0.0
    ));
    assert!(!remove_when_far_away(
        RemovalPolicy::Cat {
            tamed: false,
            tick_count: 2_400
        },
        0.0
    ));
    assert!(remove_when_far_away(
        RemovalPolicy::Cat {
            tamed: false,
            tick_count: 2_401
        },
        0.0
    ));
    assert!(!remove_when_far_away(
        RemovalPolicy::Ocelot {
            trusting: true,
            tick_count: 9_999
        },
        0.0
    ));
    assert!(!remove_when_far_away(
        RemovalPolicy::Fish {
            from_bucket: false,
            custom_named: true
        },
        0.0
    ));
    assert!(remove_when_far_away(RemovalPolicy::Nautilus, 0.0));
}

#[test]
fn raider_patrol_piglin_and_zombie_villager_overrides_cross_endpoints() {
    assert!(!remove_when_far_away(
        RemovalPolicy::Raider { current_raid: true },
        99_999.0
    ));
    assert!(!remove_when_far_away(
        RemovalPolicy::Patrolling { patrolling: true },
        16_384.0
    ));
    assert!(remove_when_far_away(
        RemovalPolicy::Patrolling { patrolling: true },
        16_385.0
    ));
    assert!(remove_when_far_away(
        RemovalPolicy::Patrolling { patrolling: false },
        0.0
    ));
    assert!(!remove_when_far_away(
        RemovalPolicy::Piglin {
            stored_persistence: true
        },
        0.0
    ));
    assert!(remove_when_far_away(RemovalPolicy::Always, 0.0));
    assert!(!remove_when_far_away(
        RemovalPolicy::ZombieVillager {
            converting: true,
            villager_xp: 0
        },
        0.0
    ));
    assert!(!remove_when_far_away(
        RemovalPolicy::ZombieVillager {
            converting: false,
            villager_xp: 1
        },
        0.0
    ));
    assert!(remove_when_far_away(
        RemovalPolicy::ZombieVillager {
            converting: false,
            villager_xp: 0
        },
        0.0
    ));
}

#[test]
fn invocation_is_root_only_pre_chunk_and_discard_has_no_death_rewards() {
    assert_eq!(
        (
            INVOCATION_POLICY.check_before_current_chunk_ticking_admission,
            INVOCATION_POLICY.root_entities_only,
            INVOCATION_POLICY.valid_passenger_checked_independently,
            INVOCATION_POLICY.discard_without_death_loot_or_xp
        ),
        (true, true, false, true)
    );
}
