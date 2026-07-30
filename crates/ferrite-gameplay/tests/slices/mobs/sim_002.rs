use ferrite_gameplay::mob::runtime::mob_001::hostile::Difficulty;
use ferrite_gameplay::mob::runtime::sim_002::manager::{
    PERSISTENCE_FACTS, STOP_RAID, manager_raid_tick, raider_reattach, unique_id,
};
use ferrite_gameplay::mob::runtime::sim_002::omen::{
    BlockPosition, CreateAdmission, RAIDS_DEFAULT, absorb_omen, bad_omen_conversion,
    create_admission, ordinary_groups, raid_center, raid_omen_tick, reuse_nearest_active,
};
use ferrite_gameplay::mob::runtime::sim_002::raid::{
    LostVillage, PostRaid, RaiderCleanupInput, WaveAdmission, active_admission, active_counter,
    celebration_stops, cooldown_tick, hero_reward, lost_village, post_raid_tick,
    remove_tracked_raider, wave_admission, wave_name_suffix,
};
use ferrite_gameplay::mob::runtime::sim_002::waves::{
    FIXED_COUNTS, MEMBER_COMMIT, RaiderType, completed_group_count, extra_draw_bound, fixed_count,
    has_bonus_group, horn_recipient, random_extra_bound, ravager_rider, spawn_candidate,
    spawn_probe,
};

#[test]
fn omen_conversion_and_expiry_preserve_the_exact_admission_gates() {
    assert!(std::hint::black_box(RAIDS_DEFAULT));
    let conversion = bad_omen_conversion(true, Difficulty::Normal, true);
    assert!(conversion.convert);
    assert_eq!(conversion.raid_omen_duration, 600);
    assert!(conversion.preserve_amplifier && conversion.snapshot_player_position);
    assert!(!bad_omen_conversion(false, Difficulty::Normal, true).convert);
    assert!(!bad_omen_conversion(true, Difficulty::Peaceful, true).convert);
    assert!(!bad_omen_conversion(true, Difficulty::Normal, false).convert);

    assert!(!raid_omen_tick(2).remove_effect);
    let expiry = raid_omen_tick(1);
    assert!(expiry.call_create_or_extend);
    assert!(expiry.clear_saved_position && expiry.remove_effect);
}

#[test]
fn creation_orders_spectator_rule_and_attribute_gates() {
    assert_eq!(
        create_admission(true, false, false),
        CreateAdmission::Spectator
    );
    assert_eq!(
        create_admission(false, false, false),
        CreateAdmission::RuleDisabled
    );
    assert_eq!(
        create_admission(false, true, false),
        CreateAdmission::AttributeDisabled
    );
    assert_eq!(
        create_admission(false, true, true),
        CreateAdmission::Admitted
    );
}

#[test]
fn raid_center_floors_component_means_and_empty_input_uses_snapshot() {
    let saved = BlockPosition { x: 9, y: 8, z: 7 };
    assert_eq!(raid_center(saved, &[]), saved);
    assert_eq!(
        raid_center(
            saved,
            &[
                BlockPosition { x: -2, y: 1, z: 5 },
                BlockPosition { x: -1, y: 2, z: 6 },
            ],
        ),
        BlockPosition { x: -2, y: 1, z: 5 }
    );
}

#[test]
fn reuse_radius_is_strict_and_manager_ids_preincrement_from_one() {
    assert_eq!(reuse_nearest_active(&[9_216.0, 4.0, 4.0]), Some(1));
    assert_eq!(reuse_nearest_active(&[9_216.0]), None);
    assert_eq!(unique_id(1), (2, 2));
    assert_eq!(
        (
            ordinary_groups(Difficulty::Peaceful),
            ordinary_groups(Difficulty::Easy),
            ordinary_groups(Difficulty::Normal),
            ordinary_groups(Difficulty::Hard),
        ),
        (0, 3, 5, 7)
    );
}

#[test]
fn omen_absorption_clamps_level_skips_started_level_five_and_always_dirties() {
    let absorbed = absorb_omen(false, 4, true, 9, 0);
    assert!(absorbed.call_absorb && absorbed.award_trigger_if_no_wave);
    assert_eq!(absorbed.new_level, 5);
    assert!(absorbed.mark_manager_dirty);

    let skipped = absorb_omen(true, 5, true, 1, 0);
    assert!(!skipped.call_absorb);
    assert_eq!(skipped.new_level, 5);
    assert!(!skipped.award_trigger_if_no_wave);
    assert!(skipped.mark_manager_dirty);
}

#[test]
fn manager_rereads_rule_before_raid_tick_and_marks_periodic_dirty() {
    let retired = manager_raid_tick(18, false, false);
    assert_eq!(retired.manager_tick, 19);
    assert!(retired.stop_raid && retired.remove_entry && retired.mark_dirty);
    assert!(!retired.tick_raid && !retired.remove_raiders);
    assert_eq!(
        (
            STOP_RAID.active,
            STOP_RAID.clear_bossbar_players,
            STOP_RAID.status_stopped
        ),
        (false, true, true)
    );

    let periodic = manager_raid_tick(199, true, false);
    assert!(periodic.tick_raid && periodic.mark_dirty);
    assert!(!periodic.remove_entry);
}

#[test]
fn active_admission_peaceful_and_lost_village_outcomes_are_explicit() {
    let loaded = active_admission(false, true, Difficulty::Normal);
    assert!(loaded.active && loaded.update_bossbar_visibility && loaded.continue_tick);
    let peaceful = active_admission(true, true, Difficulty::Peaceful);
    assert!(peaceful.stop_peaceful && !peaceful.continue_tick);
    let unloaded = active_admission(true, false, Difficulty::Hard);
    assert!(!unloaded.active && unloaded.update_bossbar_visibility);

    assert_eq!(lost_village(true, false, 1), LostVillage::KeepCenter);
    assert_eq!(lost_village(false, true, 1), LostVillage::MoveCenter);
    assert_eq!(lost_village(false, false, 0), LostVillage::StopNoGroups);
    assert_eq!(lost_village(false, false, 1), LostVillage::MarkLoss);
}

#[test]
fn active_clock_cooldown_and_wave_boundaries_match_vanilla_order() {
    let timeout = active_counter(47_999);
    assert!(timeout.stop_timeout && timeout.cleanup_due);

    let missing = cooldown_tick(300, false, false);
    assert!(missing.recompute_spawn_position && missing.refresh_membership);
    assert_eq!((missing.cooldown, missing.progress_numerator), (299, 0));
    assert!(cooldown_tick(299, true, false).recompute_spawn_position);
    assert!(!cooldown_tick(299, true, true).recompute_spawn_position);

    assert_eq!(
        wave_admission(true, 0, 0),
        WaveAdmission::ResetCooldownAndTitle
    );
    assert_eq!(
        wave_admission(false, 0, 6),
        WaveAdmission::StopAfterFailedProbes
    );
    assert_eq!(wave_name_suffix(1), Some(1));
    assert_eq!(wave_name_suffix(2), Some(2));
    assert_eq!(wave_name_suffix(3), None);
}

#[test]
fn victory_waits_through_forty_and_celebration_stops_at_six_hundred() {
    assert_eq!(
        post_raid_tick(39),
        PostRaid::Wait {
            post_raid_ticks: 40
        }
    );
    assert_eq!(post_raid_tick(40), PostRaid::Victory);
    assert!(!celebration_stops(599));
    assert!(celebration_stops(600));

    let reward = hero_reward(true, true, 3);
    assert_eq!((reward.duration, reward.amplifier), (48_000, 2));
    assert!(reward.hidden_particles && reward.visible_icon);
    assert!(reward.award_player_stat_and_criterion);
}

#[test]
fn cleanup_thresholds_are_inclusive_except_no_action_time() {
    let base = RaiderCleanupInput {
        removed: false,
        another_dimension: false,
        distance_squared: 0,
        entity_age: 0,
        uuid_resolves: true,
        no_action_time: 0,
        outside_village_checks: 0,
        outside_village: false,
    };
    assert!(remove_tracked_raider(RaiderCleanupInput {
        distance_squared: 12_544,
        ..base
    }));
    assert!(remove_tracked_raider(RaiderCleanupInput {
        entity_age: 600,
        uuid_resolves: false,
        ..base
    }));
    assert!(!remove_tracked_raider(RaiderCleanupInput {
        outside_village_checks: 30,
        no_action_time: 2_400,
        outside_village: true,
        ..base
    }));
    assert!(remove_tracked_raider(RaiderCleanupInput {
        outside_village_checks: 30,
        no_action_time: 2_401,
        outside_village: true,
        ..base
    }));
}

#[test]
fn fixed_wave_table_bonus_and_random_extra_bounds_are_locked() {
    assert_eq!(
        FIXED_COUNTS,
        [
            [0, 2, 0, 1, 4, 2, 5],
            [0, 0, 0, 0, 1, 1, 2],
            [4, 3, 3, 4, 4, 4, 2],
            [0, 0, 0, 3, 0, 0, 1],
            [0, 0, 1, 0, 1, 0, 2],
        ]
    );
    assert_eq!(fixed_count(RaiderType::Pillager, 7), 2);
    assert!(has_bonus_group(2));
    assert!(!has_bonus_group(1));
    assert_eq!(
        random_extra_bound(RaiderType::Vindicator, Difficulty::Easy, 1, false, 1),
        1
    );
    assert_eq!(extra_draw_bound(Difficulty::Easy, 1), 2);
    assert_eq!(
        random_extra_bound(RaiderType::Witch, Difficulty::Hard, 4, false, 0),
        0
    );
    assert_eq!(
        random_extra_bound(RaiderType::Ravager, Difficulty::Hard, 8, true, 0),
        1
    );
    assert_eq!(
        random_extra_bound(RaiderType::Evoker, Difficulty::Hard, 8, true, 0),
        0
    );
}

#[test]
fn spawn_probe_counts_factor_and_candidate_boundaries_are_exact() {
    let regular = spawn_probe(160, false);
    assert_eq!((regular.attempts, regular.angle_draws), (8, 1));
    assert_eq!(regular.jitter_draws_per_attempt, 2);
    assert!((regular.radial_factor - 1.52).abs() < f32::EPSILON);
    assert!(regular.outside_village_required);
    assert_eq!(spawn_probe(160, true).attempts, 20);

    let candidate = spawn_candidate(96, true, true, true, true);
    assert!(candidate.admitted && candidate.vertical_within_ninety_six);
    assert_eq!(candidate.loaded_margin, 10);
    assert!(!spawn_candidate(97, true, true, true, true).admitted);
}

#[test]
fn member_commit_group_increment_riders_and_horn_recipients_are_locked() {
    let commit = std::hint::black_box(MEMBER_COMMIT);
    assert!(commit.leader_if_first_capable);
    assert!(commit.add_health_before_insert);
    assert!(commit.finalize_event && commit.insertion_result_ignored);
    assert!(commit.null_ends_only_type_loop);
    assert_eq!(completed_group_count(4), 5);
    assert_eq!(ravager_rider(5, 0), Some(RaiderType::Pillager));
    assert_eq!(ravager_rider(7, 0), Some(RaiderType::Evoker));
    assert_eq!(ravager_rider(7, 1), Some(RaiderType::Vindicator));
    assert_eq!(ravager_rider(6, 0), None);
    assert!(horn_recipient(64.0, false));
    assert!(!horn_recipient(64.001, false));
    assert!(horn_recipient(128.0, true));
}

#[test]
fn persistence_rebuilds_runtime_only_when_saved_raid_still_resolves() {
    let persistence = std::hint::black_box(PERSISTENCE_FACTS);
    assert!(!persistence.runtime_groups_persisted);
    assert!(!persistence.leaders_persisted);
    assert!(!persistence.rng_persisted);
    assert!(!persistence.cached_spawn_position_persisted);
    assert!(!persistence.celebration_ticks_persisted);
    assert!(persistence.missing_partial_manager_falls_back_dirty);

    let attached = raider_reattach(true, true);
    assert!(attached.attach && attached.replace_equal_uuid && attached.restore_leader);
    assert!(!attached.count_health_again);
    assert!(!raider_reattach(false, true).attach);
}
