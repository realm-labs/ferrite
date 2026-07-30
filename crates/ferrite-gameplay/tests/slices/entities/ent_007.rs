use ferrite_gameplay::entity::runtime::ent_005::knockback::Vector3;
use ferrite_gameplay::entity::runtime::ent_007::drops::{
    EQUIPMENT_ORDER, EquipmentDropInput, EquipmentOverride, EquipmentSlot, LootOwner,
    equipment_drop, loot_context, loot_gate, nether_star_age, override_ignores_mob_drops,
    player_inventory_plan, player_item_drop, spawn_at_location,
};
use ferrite_gameplay::entity::runtime::ent_007::entry::{
    CausingCallback, Difficulty, ORDINARY_DEATH_ORDER, OrdinaryDeathStage, ServerPlayerDeathInput,
    TeamDeathVisibility, causing_callback, client_death_event, dragon_killing_blow,
    nonserver_player_death_velocity, ordinary_death_admission, ordinary_death_result,
    server_player_death, wither_rose,
};
use ferrite_gameplay::entity::runtime::ent_007::experience::{
    EquipmentXpInput, ExperienceEligibility, ExperienceOwner, OrbAward, OrbCandidate, award_orb,
    experience_eligible, mob_experience_reward, next_xp_split, player_experience_reward,
};
use ferrite_gameplay::entity::runtime::ent_007::protection::{
    Hand, NONPLAYER_TOTEM_ORDER, PLAYER_TOTEM_ORDER, ProtectionStack, ProtectionStage,
    client_totem_event, select_protection, totem_protection,
};
use ferrite_gameplay::entity::runtime::ent_007::timelines::{
    RemovalReason, common_death_tick, creaking_death_tick, dragon_death_tick,
    pearl_vanishes_before_motion,
};

const EMPTY: ProtectionStack = ProtectionStack {
    present: false,
    has_death_protection: false,
};
const TOTEM: ProtectionStack = ProtectionStack {
    present: true,
    has_death_protection: true,
};

#[test]
fn protection_bypass_and_hand_scan_order_are_exact() {
    let bypassed = select_protection(true, TOTEM, TOTEM);
    assert_eq!(bypassed.hand, None);
    assert_eq!(bypassed.inspected_hands, 0);

    let main = select_protection(false, TOTEM, TOTEM);
    assert_eq!((main.hand, main.inspected_hands), (Some(Hand::Main), 1));
    assert!(main.copy_full_stack_before_shrink);
    assert_eq!(main.shrink_held_by, 1);

    let off = select_protection(false, EMPTY, TOTEM);
    assert_eq!((off.hand, off.inspected_hands), (Some(Hand::Off), 2));
    assert_eq!(select_protection(false, EMPTY, EMPTY).inspected_hands, 2);
}

#[test]
fn protection_effects_and_client_rescan_keep_source_order() {
    assert_eq!(PLAYER_TOTEM_ORDER[0], ProtectionStage::AwardItemUsed);
    assert_eq!(
        PLAYER_TOTEM_ORDER[2],
        ProtectionStage::FinishInteractionVibration
    );
    assert_eq!(NONPLAYER_TOTEM_ORDER[0], ProtectionStage::SetHealthOne);

    let result = totem_protection(select_protection(false, TOTEM, EMPTY));
    assert!(result.protected && result.clear_all_effects && result.apply_draw_consumed);
    assert_eq!(result.health, 1.0);
    assert_eq!(
        result
            .regeneration
            .map(|effect| (effect.amplifier, effect.duration)),
        Some((1, 900))
    );
    assert_eq!(
        result
            .absorption
            .map(|effect| (effect.amplifier, effect.duration)),
        Some((1, 100))
    );
    assert_eq!(
        result
            .fire_resistance
            .map(|effect| (effect.amplifier, effect.duration)),
        Some((0, 800))
    );
    assert_eq!(result.event, Some(35));

    let event = client_totem_event(true, false, true);
    assert_eq!(
        (event.emitter_ticks, event.display_hand),
        (30, Some(Hand::Off))
    );
    assert!(event.play_local_sound && event.display_activation);
    assert!(client_totem_event(true, false, false).construct_fallback_totem);
    assert!(!client_totem_event(false, false, false).display_activation);
}

#[test]
fn ordinary_death_admission_and_callback_suppression_are_explicit() {
    let admitted = ordinary_death_admission(false, false, true);
    assert!(admitted.admitted && admitted.award_kill_score_first);
    assert!(!ordinary_death_admission(true, false, true).admitted);
    assert_eq!(ORDINARY_DEATH_ORDER[0], OrdinaryDeathStage::AwardKillScore);
    assert_eq!(ORDINARY_DEATH_ORDER[11], OrdinaryDeathStage::SetDyingPose);

    let conversion = causing_callback(CausingCallback::ZombieVillager {
        difficulty: Difficulty::Normal,
        normal_skip_draw: false,
        conversion_succeeded: true,
    });
    assert!(conversion.conversion_draw_consumed);
    assert!(!conversion.continue_death);
    let suppressed = ordinary_death_result(conversion);
    assert!(!suppressed.emit_entity_die && !suppressed.run_drops);
    assert_eq!(suppressed.broadcast_event, 3);
    assert!(suppressed.set_dying_pose);
}

#[test]
fn conversion_and_charged_creeper_gates_consume_only_required_work() {
    let skipped = causing_callback(CausingCallback::ZombieVillager {
        difficulty: Difficulty::Normal,
        normal_skip_draw: true,
        conversion_succeeded: true,
    });
    assert!(skipped.continue_death && skipped.conversion_draw_consumed);
    let hard = causing_callback(CausingCallback::ZombieVillager {
        difficulty: Difficulty::Hard,
        normal_skip_draw: true,
        conversion_succeeded: true,
    });
    assert!(!hard.continue_death && !hard.conversion_draw_consumed);
    let easy = causing_callback(CausingCallback::ZombieVillager {
        difficulty: Difficulty::Easy,
        normal_skip_draw: false,
        conversion_succeeded: true,
    });
    assert!(easy.continue_death && !easy.conversion_draw_consumed);

    let creeper = causing_callback(CausingCallback::ChargedCreeper {
        loot_gate: true,
        already_dropped_skull: false,
        emitted_stacks: 1,
    });
    assert!(creeper.evaluate_charged_creeper_loot && creeper.set_dropped_skulls);
}

#[test]
fn wither_rose_and_client_death_event_preserve_fallbacks_and_draws() {
    assert_eq!(
        wither_rose(true, true, true, true),
        ferrite_gameplay::entity::runtime::ent_007::entry::WitherRose::PlaceBlockIgnoringResult {
            flags: 3
        }
    );
    assert_eq!(
        wither_rose(true, false, true, true),
        ferrite_gameplay::entity::runtime::ent_007::entry::WitherRose::SpawnItemWithoutPickupDelay
    );
    let client = client_death_event(false, 0.75, 0.25);
    assert!((client.pitch - 1.1).abs() < f32::EPSILON);
    assert!(client.play_sound && client.set_health_zero && client.enter_local_generic_death);
    assert!(!client_death_event(true, 0.0, 0.0).set_health_zero);
}

#[test]
fn server_and_nonserver_player_death_branches_stay_separate() {
    let server = server_player_death(ServerPlayerDeathInput {
        show_death_messages: true,
        team_visibility: TeamDeathVisibility::OwnTeam,
        shoulder_time: 79,
        game_time: 100,
        forgive_dead_players: true,
        spectator: false,
        kill_credit_present: true,
    });
    assert!(server.emit_entity_die && server.combat_packet_has_message);
    assert!(server.broadcast_real_message && server.try_shoulders);
    assert_eq!(server.shoulder_attempts, 2);
    assert!(server.run_items_and_xp && server.award_kill_credit_score);
    assert!(server.attempt_wither_rose && server.record_last_death_location);
    assert!(!server.set_dead && !server.set_dying_pose);
    assert!(server.mark_client_unloaded);

    let spectator = server_player_death(ServerPlayerDeathInput {
        show_death_messages: false,
        team_visibility: TeamDeathVisibility::Never,
        shoulder_time: 80,
        game_time: 100,
        forgive_dead_players: false,
        spectator: true,
        kill_credit_present: false,
    });
    assert!(!spectator.try_shoulders && !spectator.run_items_and_xp);
    assert!(!spectator.combat_packet_has_message && !spectator.attempt_wither_rose);

    assert_eq!(
        nonserver_player_death_velocity(false, 0.0, 0.0),
        Vector3::new(0.0, 0.1, 0.0)
    );
    let velocity = nonserver_player_death_velocity(true, 0.0, 0.0);
    assert!((velocity.x + 0.1).abs() < 1.0e-7);
    assert_eq!(dragon_killing_blow(false), (true, 1.0));
    assert_eq!(dragon_killing_blow(true), (false, 0.0));
}

#[test]
fn common_and_monster_loot_gates_and_context_memory_are_exact() {
    assert!(!loot_gate(LootOwner::CommonLiving, false, true));
    assert!(loot_gate(LootOwner::Monster, false, true));
    assert!(!loot_gate(LootOwner::Monster, true, false));

    let remembered = loot_context(true, false, true, true, 2.5, 91);
    assert!(remembered.this_entity && remembered.origin && remembered.damage_source);
    assert!(remembered.attacking_entity && !remembered.direct_attacking_entity);
    assert_eq!(remembered.last_damage_player_luck, Some(2.5));
    assert_eq!(remembered.seed, 91);
    assert!(!loot_context(false, false, false, true, 2.5, 0).last_damage_player);
}

#[test]
fn item_construction_uses_exact_pickup_delays_and_random_cardinality() {
    let position = Vector3::new(1.0, 2.0, 3.0);
    let item = spawn_at_location(false, position, 0.1, 0.2, 0.25, 0.75, false);
    assert!(item.admitted);
    assert_eq!((item.position, item.pickup_delay), (position, 10));
    assert_eq!(item.construction_draws, 4);
    assert!((item.velocity.x + 0.05).abs() < f64::EPSILON);
    assert!((item.velocity.y - 0.2).abs() < f64::EPSILON);
    assert!((item.velocity.z - 0.05).abs() < 1.0e-15);
    assert_eq!(
        spawn_at_location(false, position, 0.0, 0.0, 0.0, 0.0, true).pickup_delay,
        0
    );
    assert_eq!(
        spawn_at_location(true, position, 0.0, 0.0, 0.0, 0.0, false).construction_draws,
        0
    );
}

fn equipment_input() -> EquipmentDropInput {
    EquipmentDropInput {
        nonempty: true,
        base_chance: None,
        causing_living: true,
        looting_level: 2,
        looting_holder_is_player: true,
        prevent_equipment_drop: false,
        killed_by_player: true,
        chance_draw: 0.1,
        damageable: true,
        maximum_damage: 100,
        inner_damage_draw: 7,
        outer_damage_draw: 4,
    }
}

#[test]
fn equipment_drop_order_chance_and_nested_damage_draws_are_exact() {
    assert_eq!(EQUIPMENT_ORDER.len(), 8);
    assert_eq!(EQUIPMENT_ORDER[0], EquipmentSlot::MainHand);
    assert_eq!(EQUIPMENT_ORDER[7], EquipmentSlot::Saddle);
    let dropped = equipment_drop(equipment_input());
    assert!((dropped.adjusted_chance - 0.105).abs() < f32::EPSILON);
    assert!(dropped.drop && dropped.clear_slot);
    assert_eq!(dropped.new_damage, Some(96));
    assert_eq!(dropped.damage_draws_consumed, 2);

    let zero = equipment_drop(EquipmentDropInput {
        base_chance: Some(0.0),
        ..equipment_input()
    });
    assert!(!zero.chance_draw_consumed && !zero.drop);
    let preserved = equipment_drop(EquipmentDropInput {
        base_chance: Some(2.0),
        killed_by_player: false,
        ..equipment_input()
    });
    assert!(preserved.preserved && preserved.drop);
    assert_eq!(preserved.new_damage, None);
}

#[test]
fn subtype_overrides_nether_star_and_player_items_keep_special_rules() {
    assert!(override_ignores_mob_drops(
        EquipmentOverride::FoxMainHandBeforeBase
    ));
    assert!(override_ignores_mob_drops(
        EquipmentOverride::ChestedHorseChestAfterInventory
    ));
    assert!(!override_ignores_mob_drops(
        EquipmentOverride::PiglinInventoryInsideLootGate
    ));
    assert_eq!(nether_star_age(true), Some(-6_000));
    assert_eq!(nether_star_age(false), None);

    assert!(player_inventory_plan(false, false).is_some());
    assert!(player_inventory_plan(true, false).is_none());
    let item = player_item_drop(1.0, 2.0, 3.0, 0.4, 0.25);
    assert_eq!(item.pickup_delay, 40);
    assert_eq!(item.victim_draws_consumed, 2);
    assert!(!item.thrower_present);
    assert!((item.position.y - 1.699_999_988_079_071).abs() < 1.0e-12);
}

#[test]
fn experience_eligibility_and_reward_mutation_match_owner_rules() {
    let base = ExperienceEligibility {
        owner: ExperienceOwner::CommonLiving,
        skip_drop_experience: false,
        recent_player_memory: true,
        should_drop_experience: true,
        mob_drops: true,
        adult: false,
    };
    assert!(!experience_eligible(base));
    assert!(experience_eligible(ExperienceEligibility {
        owner: ExperienceOwner::Monster,
        ..base
    }));
    assert!(!experience_eligible(ExperienceEligibility {
        owner: ExperienceOwner::Tadpole,
        adult: true,
        ..base
    }));
    assert!(experience_eligible(ExperienceEligibility {
        owner: ExperienceOwner::Player,
        recent_player_memory: false,
        mob_drops: false,
        ..base
    }));
    assert_eq!(player_experience_reward(50, false, false), 100);
    assert_eq!(player_experience_reward(10, true, false), 0);

    let reward = mob_experience_reward(
        5,
        &[
            EquipmentXpInput {
                slot: EquipmentSlot::Head,
                nonempty: true,
                drop_chance: 0.5,
                draw_three: 2,
            },
            EquipmentXpInput {
                slot: EquipmentSlot::Saddle,
                nonempty: true,
                drop_chance: 0.5,
                draw_three: 2,
            },
        ],
        false,
    );
    assert_eq!((reward.reward, reward.draws_consumed), (8, 1));
    assert_eq!(mob_experience_reward(5, &[], true).reward, 5);
}

#[test]
fn experience_splitting_and_first_matching_orb_merge_are_deterministic() {
    assert_eq!(next_xp_split(2_477), 2_477);
    assert_eq!(next_xp_split(2_476), 1_237);
    assert_eq!(next_xp_split(6), 3);
    assert_eq!(next_xp_split(0), 0);

    let candidates = [
        OrbCandidate {
            id: 41,
            value: 17,
            removed: true,
        },
        OrbCandidate {
            id: 81,
            value: 17,
            removed: false,
        },
        OrbCandidate {
            id: 121,
            value: 17,
            removed: false,
        },
    ];
    assert_eq!(
        award_orb(17, 1, &candidates),
        OrbAward::Merge {
            candidate_index: 1,
            increment_count: true,
            reset_age: true
        }
    );
    assert!(matches!(
        award_orb(7, 0, &candidates),
        OrbAward::Spawn { .. }
    ));
    assert_eq!(award_orb(0, 0, &[]), OrbAward::None);
}

#[test]
fn common_and_creaking_timelines_remove_at_distinct_boundaries() {
    assert_eq!(common_death_tick(18, true).remove, None);
    let common = common_death_tick(19, true);
    assert_eq!(common.remove, Some(RemovalReason::Killed));
    assert_eq!(
        (common.broadcast_event, common.poof_particles),
        (Some(60), 20)
    );
    assert_eq!((common.gaussian_draws, common.position_draws), (60, 60));

    assert!(creaking_death_tick(false, true, 45, true).use_common_timeline);
    assert_eq!(creaking_death_tick(true, true, 44, true).remove, None);
    let creaking = creaking_death_tick(true, true, 45, true);
    assert_eq!(creaking.remove, Some(RemovalReason::Discarded));
    assert_eq!(
        (creaking.pale_oak_particles, creaking.heart_particles),
        (100, 10)
    );
    assert_eq!(
        (creaking.particle_spread_fraction, creaking.particle_speed),
        (0.8, 0.25)
    );
}

#[test]
fn dragon_timeline_accumulates_periodic_and_final_rewards_before_removal() {
    let first = dragon_death_tick(0, true, false, true, true);
    assert_eq!(first.global_event, Some(1028));
    assert_eq!(first.xp, 0);
    assert_eq!(dragon_death_tick(149, true, false, true, true).xp, 0);
    assert_eq!(dragon_death_tick(154, true, false, true, true).xp, 960);
    let particles = dragon_death_tick(179, true, false, true, true);
    assert!(particles.particle);
    assert_eq!(particles.particle_draws, 3);

    let final_tick = dragon_death_tick(199, true, false, true, true);
    assert_eq!(final_tick.xp, 3_360);
    assert!(final_tick.notify_fight && final_tick.emit_entity_die_after_remove);
    assert_eq!(final_tick.remove, Some(RemovalReason::Killed));
    assert_eq!(dragon_death_tick(199, true, false, true, false).xp, 140);
    assert_eq!(
        dragon_death_tick(199, false, false, true, true).remove,
        None
    );
}

#[test]
fn pearl_vanish_rule_requires_all_post_player_death_gates() {
    assert!(pearl_vanishes_before_motion(true, false, true, true));
    assert!(!pearl_vanishes_before_motion(true, true, true, true));
    assert!(!pearl_vanishes_before_motion(true, false, false, true));
    assert!(!pearl_vanishes_before_motion(true, false, true, false));
}
