use ferrite_gameplay::mob::runtime::mob_006::age::{
    DEFAULT_BABY_AGE, FoodInteraction, age_tick, age_up, feeding_growth_seconds, food_interaction,
    love_tick, toggle_age_lock,
};
use ferrite_gameplay::mob::runtime::mob_006::breeding::{
    ChildCommit, MateCandidate, SpecialBreeding, base_can_mate, breed_goal_continues,
    breed_goal_tick, generic_child_commit, nearest_mate, special_order,
};
use ferrite_gameplay::mob::runtime::mob_006::families::{
    AxolotlVariant, MooshroomVariant, Parent, ProducerFamily, RabbitVariant, axolotl_variant,
    equine_parentable, goat_screaming, horse_variant, llama_strength, mooshroom_variant,
    producer_facts, rabbit_variant, random_parent, reflected_horse_attribute,
};
use ferrite_gameplay::mob::runtime::mob_006::tame::{
    HorseTame, OWNER_TELEPORT_COMMIT, TAME_COMMIT, TameAttempt, horse_tame, ocelot_trust_admitted,
    owner_teleport_admitted, tame_attempt, teleport_candidate, teleport_offset, trust_event,
};

#[test]
fn signed_age_ticks_cross_zero_only_on_unlocked_living_server_ticks() {
    assert_eq!(DEFAULT_BABY_AGE, -24_000);
    assert_eq!(age_tick(-1, false, true).age, 0);
    assert!(age_tick(-1, false, true).crossed_zero);
    assert_eq!(age_tick(1, false, true).age, 0);
    assert_eq!(age_tick(0, false, true).age, 0);
    assert_eq!(age_tick(-1, true, true).age, -1);
    assert_eq!(age_tick(-1, false, false).age, -1);
}

#[test]
fn forced_growth_accumulates_delta_then_installs_positive_cooldown() {
    let growth = age_up(-100, 20, 10, true);
    assert_eq!(growth.forced_age, 120);
    assert_eq!(growth.age, 120);
    assert!(growth.crossed_zero);
    assert_eq!(growth.particle_timer, 40);
    assert_eq!(feeding_growth_seconds(24_000), 120);
    assert_eq!(feeding_growth_seconds(199), 0);
}

#[test]
fn golden_dandelion_lock_toggle_resets_age_and_persists_only_on_lock() {
    let locked = toggle_age_lock(true, true, false, false, -12_000);
    assert!(locked.admitted && locked.age_locked && locked.consume_item);
    assert_eq!((locked.age, locked.particle_timer), (-12_000, 40));
    assert!(locked.set_persistence_required);
    let unlocked = toggle_age_lock(true, true, false, true, -12_000);
    assert!(!unlocked.age_locked && !unlocked.set_persistence_required);
    assert!(!toggle_age_lock(true, true, true, false, -12_000).admitted);
}

#[test]
fn food_interaction_separates_love_growth_client_consume_and_delegate() {
    assert_eq!(
        food_interaction(true, true, true, 0, 0, false),
        FoodInteraction::EnterLove
    );
    assert_eq!(
        food_interaction(true, true, true, -1, 0, false),
        FoodInteraction::GrowBaby
    );
    assert_eq!(
        food_interaction(true, true, false, -1, 0, false),
        FoodInteraction::ClientConsume
    );
    assert_eq!(
        food_interaction(true, true, true, 0, 1, false),
        FoodInteraction::Delegate
    );
}

#[test]
fn love_clock_clears_on_age_or_damage_and_hearts_follow_remaining_tens() {
    assert_eq!(love_tick(600, 0, false).love_timer, 599);
    assert!(love_tick(11, 0, false).emit_heart);
    assert_eq!(love_tick(10, 0, false).love_timer, 9);
    assert!(love_tick(10, 1, false).clear);
    assert!(love_tick(10, 0, true).clear);
}

#[test]
fn base_compatibility_and_nearest_selection_keep_class_and_query_ties() {
    assert!(base_can_mate(false, true, 1, 1));
    assert!(!base_can_mate(true, true, 1, 1));
    assert!(!base_can_mate(false, false, 1, 1));
    let candidates = [
        MateCandidate {
            distance_squared: 4.0,
            can_mate: true,
            panicking: false,
        },
        MateCandidate {
            distance_squared: 4.0,
            can_mate: true,
            panicking: false,
        },
        MateCandidate {
            distance_squared: 1.0,
            can_mate: true,
            panicking: true,
        },
    ];
    assert_eq!(nearest_mate(&candidates), Some(0));
}

#[test]
fn breed_goal_uses_adjusted_thirty_ticks_and_strict_distance_nine() {
    assert!(!breed_goal_tick(28, 8.0).attempt_breeding);
    assert!(breed_goal_tick(29, 8.999).attempt_breeding);
    assert!(!breed_goal_tick(29, 9.0).attempt_breeding);
    assert!(breed_goal_continues(true, true, false, 59));
    assert!(!breed_goal_continues(true, true, false, 60));
    assert!(!breed_goal_continues(true, true, true, 1));
}

#[test]
fn null_child_retries_while_generic_commit_precedes_unchecked_insertion() {
    assert_eq!(
        generic_child_commit(false, true, 0),
        ChildCommit::NullRetryWithoutChanges
    );
    let ChildCommit::Generic(commit) = generic_child_commit(true, true, 6) else {
        panic!("expected generic child commit");
    };
    assert_eq!((commit.parent_age, commit.xp), (6_000, Some(7)));
    assert!(commit.xp_before_child_insertion && commit.insertion_result_ignored);
    assert!(commit.cause_actor_then_partner && commit.clear_both_love);
    let ChildCommit::Generic(no_xp) = generic_child_commit(true, false, 0) else {
        panic!("expected generic child commit");
    };
    assert_eq!(no_xp.xp, None);
}

#[test]
fn special_producers_preserve_fox_and_allay_insertion_before_event() {
    let fox = special_order(SpecialBreeding::Fox);
    assert!(!fox.generic_finalization && fox.child_or_item_before_event);
    assert!(fox.event_before_xp);
    let turtle = special_order(SpecialBreeding::Turtle);
    assert!(turtle.generic_finalization && !turtle.child_or_item_before_event);
    assert!(special_order(SpecialBreeding::Allay).child_or_item_before_event);
}

#[test]
fn parent_and_mooshroom_rabbit_axolotl_variant_draws_cross_exact_odds() {
    assert_eq!(random_parent(true), Parent::Actor);
    assert_eq!(random_parent(false), Parent::Partner);
    assert_eq!(
        mooshroom_variant(true, true, 0),
        MooshroomVariant::MutatedOther
    );
    assert_eq!(
        mooshroom_variant(false, false, 0),
        MooshroomVariant::Partner
    );
    assert_eq!(rabbit_variant(0, true, true), RabbitVariant::Biome);
    assert_eq!(rabbit_variant(1, true, true), RabbitVariant::Partner);
    assert_eq!(rabbit_variant(19, false, true), RabbitVariant::Actor);
    assert_eq!(axolotl_variant(0, true), AxolotlVariant::RareRegistry);
    assert_eq!(axolotl_variant(1, false), AxolotlVariant::Partner);
}

#[test]
fn goat_horse_and_llama_inheritance_use_locked_probability_partitions() {
    assert!(goat_screaming(true, 1.0));
    assert!(goat_screaming(false, 0.019));
    assert!(!goat_screaming(false, 0.02));
    assert_eq!(horse_variant(0, 0).coat_source, 0);
    assert_eq!(horse_variant(4, 2).coat_source, 1);
    assert_eq!(horse_variant(8, 4).coat_source, 2);
    assert_eq!(llama_strength(2, 4, 3, 1.0), 4);
    assert_eq!(llama_strength(5, 5, 4, 0.029), 5);
}

#[test]
fn horse_attribute_reflection_and_parentability_keep_bounds_and_state_gates() {
    let value = reflected_horse_attribute(10.0, 10.0, 5.0, 15.0, 1.0, 1.0, 1.0);
    assert!((5.0..=15.0).contains(&value));
    assert!(equine_parentable(true, true, true, true, false, false));
    assert!(!equine_parentable(true, true, true, true, true, false));
}

#[test]
fn producer_catalog_marks_persistent_and_brain_owned_families() {
    assert!(producer_facts(ProducerFamily::Axolotl).persistent_child);
    assert!(producer_facts(ProducerFamily::Hoglin).uses_brain_behavior);
    assert!(!producer_facts(ProducerFamily::Turtle).creates_immediate_child);
    assert!(producer_facts(ProducerFamily::Villager).uses_brain_behavior);
    assert!(producer_facts(ProducerFamily::Allay).persistent_child);
}

#[test]
fn tame_commit_and_species_odds_keep_authority_separate_from_events() {
    assert_eq!(
        (
            TAME_COMMIT.tame_flag_bit,
            TAME_COMMIT.sitting_pose_bit,
            TAME_COMMIT.set_generic_persistence
        ),
        (4, 1, false)
    );
    assert!(matches!(
        tame_attempt(true, 0, 3, true, true),
        TameAttempt::Success {
            event: 7,
            order_sit: true,
            ..
        }
    ));
    assert_eq!(
        tame_attempt(true, 1, 3, true, true),
        TameAttempt::Failure { event: 6 }
    );
    assert!(matches!(
        tame_attempt(true, 0, 10, false, false),
        TameAttempt::Success { .. }
    ));
}

#[test]
fn ocelot_trust_is_strictly_inside_nine_and_assigns_no_owner() {
    assert!(ocelot_trust_admitted(true, false, true, 8.999));
    assert!(!ocelot_trust_admitted(true, false, true, 9.0));
    assert!(!ocelot_trust_admitted(false, false, true, 0.0));
    assert_eq!((trust_event(true), trust_event(false)), (41, 40));
}

#[test]
fn horse_temper_requires_strict_draw_below_temper_and_failure_adds_five() {
    assert_eq!(horse_tame(1, 25, 100, 50, 0), HorseTame::NotChecked);
    assert_eq!(horse_tame(0, 25, 100, 50, 49), HorseTame::Success);
    assert_eq!(
        horse_tame(0, 25, 100, 50, 50),
        HorseTame::Failure { new_temper: 55 }
    );
}

#[test]
fn owner_teleport_uses_inclusive_distance_ten_attempts_and_offset_safety() {
    assert!(owner_teleport_admitted(144.0, false, false, false, false));
    assert!(!owner_teleport_admitted(
        143.999, false, false, false, false
    ));
    assert!(!owner_teleport_admitted(144.0, true, false, false, false));
    let offset = teleport_offset(0, 0, 6);
    assert_eq!((offset.x, offset.y, offset.z), (-3, -1, 3));
    assert!(offset.horizontal_far_enough);
    assert!(teleport_candidate(true, true, false, false, true));
    assert!(!teleport_candidate(true, true, true, false, true));
    assert!(teleport_candidate(true, true, true, true, true));
    assert_eq!(
        (
            OWNER_TELEPORT_COMMIT.attempts,
            OWNER_TELEPORT_COMMIT.snap_block_center,
            OWNER_TELEPORT_COMMIT.stop_path
        ),
        (10, true, true)
    );
}
