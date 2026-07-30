use ferrite_gameplay::entity::runtime::ent_005::admission::{
    AcceptedHitInput, Attribution, BaseImmunityInput, CooldownDecision, DamageAbort, Difficulty,
    LivingImmunityInput, PlayerRuleImmunityInput, WrapperInput, accepted_hit_plan, attribution,
    base_immunity, difficulty_scale, living_immunity, player_rule_immunity, select_cooldown_amount,
    server_player_immunity, tick_hurt_timers, transform_after_block, wrapper_admission,
};
use ferrite_gameplay::entity::runtime::ent_005::blocking::{
    AttackerKind, DamageReduction, MISSING_SOURCE_ANGLE, ResolveBlockingInput, Retaliation,
    attacker_disable_seconds, block_sound_pitch, blocking_amount_admitted, blocking_stack_mature,
    disable_blocking, hoglin_throw, incidence_angle, resolve_blocking, retaliation, start_blocking,
};
use ferrite_gameplay::entity::runtime::ent_005::knockback::{
    DIRECTION_EPSILON_SQUARED, FiveArgumentGate, ProjectileDirectionKind, SulfurArchetype,
    SulfurKnockbackInput, Vector3, active_sulfur_settings, common_knockback, damage_indication,
    five_argument_admitted, positioned_source_direction, projectile_direction, sulfur_knockback,
    sulfur_settings, uses_sulfur_special,
};
use ferrite_gameplay::entity::runtime::ent_005::reduction::{
    ArmadilloAction, ArmadilloInput, ArmorOwner, EquipmentSlot, HealthReductionInput,
    MagicReductionInput, PLAYER_ARMOR_ORDER, PROTECTION_ORDER, WolfArmorCrack,
    apply_absorption_and_health, armadillo_post_damage, armor_durability, armor_reduction,
    armor_slot_takes_durability, combat_expired, magic_reduction, protection_reduction, resistance,
    subtype_hooks, witch_reduction, wolf_armor_crack, wolf_armor_outcome,
};

fn base_immunity_input() -> BaseImmunityInput {
    BaseImmunityInput {
        removed: false,
        invulnerable_flag: false,
        bypasses_invulnerability: false,
        creative_player_source: false,
        fire_source: false,
        fire_immune: false,
        fall_source: false,
        fall_damage_immune_type: false,
    }
}

fn wrapper_input() -> WrapperInput {
    WrapperInput {
        server_player: false,
        player: false,
        server_immunity: false,
        player_pvp_disallowed: false,
        arrow_owner_pvp_disallowed: false,
        ability_invulnerable: false,
        bypasses_invulnerability: false,
        dead_or_dying: false,
        scales_with_difficulty: false,
        difficulty: Difficulty::Normal,
        living_immunity: false,
        fire_source: false,
        fire_resistance: false,
        sleeping: false,
        amount: 4.0,
    }
}

fn assert_vector(actual: Vector3, expected: Vector3) {
    assert!((actual.x - expected.x).abs() < 1.0e-9);
    assert!((actual.y - expected.y).abs() < 1.0e-9);
    assert!((actual.z - expected.z).abs() < 1.0e-9);
}

#[test]
fn base_player_and_server_immunity_layers_keep_first_matching_rules() {
    let mut base = base_immunity_input();
    base.invulnerable_flag = true;
    assert!(base_immunity(base));
    base.creative_player_source = true;
    assert!(!base_immunity(base));
    assert!(living_immunity(LivingImmunityInput {
        base: base_immunity_input(),
        enchantment_immune: true,
    }));

    assert!(player_rule_immunity(PlayerRuleImmunityInput {
        living_immune: false,
        drowning_source: true,
        drowning_damage: false,
        fall_source: true,
        fall_damage: true,
        fire_source: false,
        fire_damage: true,
        freeze_source: false,
        freeze_damage: true,
    }));
    assert!(server_player_immunity(false, true, false, true));
    assert!(!server_player_immunity(false, true, true, true));
    assert!(server_player_immunity(false, false, false, false));
}

#[test]
fn wrapper_rejections_retain_only_side_effects_before_their_abort() {
    let mut input = wrapper_input();
    input.server_player = true;
    input.player = true;
    input.player_pvp_disallowed = true;
    let pvp = wrapper_admission(input);
    assert_eq!(pvp.abort, Some(DamageAbort::PlayerPvp));
    assert!(!pvp.no_action_time_reset && !pvp.shoulder_entities_removed);

    input.player_pvp_disallowed = false;
    input.living_immunity = true;
    let living = wrapper_admission(input);
    assert_eq!(living.abort, Some(DamageAbort::LivingImmunity));
    assert!(living.no_action_time_reset && living.shoulder_entities_removed);
    assert!(!living.woke_sleeping);

    input.living_immunity = false;
    input.sleeping = true;
    let admitted = wrapper_admission(input);
    assert_eq!(admitted.abort, None);
    assert!(admitted.woke_sleeping && admitted.no_action_time_reset);
}

#[test]
fn player_difficulty_preserves_zero_nan_and_float_order() {
    assert_eq!(difficulty_scale(10.0, true, Difficulty::Peaceful), 0.0);
    assert_eq!(difficulty_scale(1.0, true, Difficulty::Easy), 1.0);
    assert_eq!(difficulty_scale(10.0, true, Difficulty::Easy), 6.0);
    assert_eq!(difficulty_scale(10.0, true, Difficulty::Hard), 15.0);
    assert!(difficulty_scale(f32::NAN, true, Difficulty::Easy).is_nan());

    let mut input = wrapper_input();
    input.player = true;
    input.amount = -0.0;
    assert_eq!(
        wrapper_admission(input).abort,
        Some(DamageAbort::PlayerDifficultyZero)
    );
    input.amount = f32::NAN;
    assert_eq!(wrapper_admission(input).abort, None);
}

#[test]
fn post_block_transform_orders_freeze_helmet_and_nonfinite_sanitization() {
    let transformed = transform_after_block(10.0, 2.0, true, true, true, true);
    assert_eq!(transformed.original, 10.0);
    assert_eq!(transformed.helmet_damage_input, 40.0);
    assert_eq!(transformed.remaining, 30.0);
    assert!(transformed.blocked && transformed.damage_helmet);

    let negative = transform_after_block(-2.0, 0.0, false, false, false, false);
    assert_eq!(negative.original.to_bits(), 0.0_f32.to_bits());
    let infinite = transform_after_block(f32::INFINITY, f32::INFINITY, false, false, false, false);
    assert_eq!(infinite.remaining, f32::MAX);
    assert!(infinite.blocked);
}

#[test]
fn cooldown_strict_threshold_selects_delta_without_resetting_timers() {
    assert_eq!(
        select_cooldown_amount(4.0, 4.0, 11, false),
        CooldownDecision::Rejected
    );
    assert_eq!(
        select_cooldown_amount(5.0, 4.0, 11, false),
        CooldownDecision::Accepted {
            selected_amount: 1.0,
            last_hurt: 5.0,
            fresh: false,
            invulnerable_time: None,
            hurt_time: None,
            hurt_duration: None,
        }
    );
    assert_eq!(
        select_cooldown_amount(4.0, 9.0, 10, false),
        CooldownDecision::Accepted {
            selected_amount: 4.0,
            last_hurt: 4.0,
            fresh: true,
            invulnerable_time: Some(20),
            hurt_time: Some(10),
            hurt_duration: Some(10),
        }
    );
    assert_eq!(tick_hurt_timers(1, 1, false), (0, 0));
    assert_eq!(tick_hurt_timers(1, 1, true), (0, 1));
}

#[test]
fn accepted_hit_plan_keeps_full_block_false_result_and_criteria() {
    let plan = accepted_hit_plan(AcceptedHitInput {
        fresh: true,
        blocked: true,
        blocked_amount: 4.0,
        remaining: 0.0,
        snapshot_still_blocks: true,
        no_impact: false,
        no_knockback: false,
        dead_after_reduction: false,
        server_player_victim: true,
        causing_server_player: true,
    });
    assert!(plan.call_on_blocked && plan.call_knockback && plan.play_hurt_sounds);
    assert!(!plan.broadcast_damage_event && !plan.mark_hurt && !plan.meaningful);
    assert!(!plan.store_last_source_and_time && !plan.invoke_active_effects);
    assert_eq!(plan.shield_block_stat, Some(40));
    assert_eq!(
        (plan.knockback_amount, plan.active_effect_amount),
        (0.0, 0.0)
    );
    assert!(plan.victim_criterion && plan.attacker_criterion);

    assert_eq!(
        attribution(false, false, false, true, false),
        Attribution::Clear
    );
    assert_eq!(
        attribution(false, false, true, false, false),
        Attribution::CausingPlayer { ticks: 100 }
    );
}

#[test]
fn blocking_use_delay_angle_and_initial_nan_gate_match_jvm_behavior() {
    let start = start_blocking(false, false, true, true, true);
    assert!(start.admitted && start.set_using_flag && start.set_offhand_flag);
    assert_eq!(start.use_duration, 72_000);
    assert!(start.emit_interact_start);
    assert!(!blocking_stack_mature(true, true, 72_000, 71_996, 0.25));
    assert!(blocking_stack_mature(true, true, 72_000, 71_995, 0.25));
    assert!(!blocking_amount_admitted(-0.0));
    assert!(!blocking_amount_admitted(f32::NEG_INFINITY));
    assert!(blocking_amount_admitted(f32::NAN));
    let front = incidence_angle(Some(Vector3::new(0.0, 0.0, 1.0)), Vector3::ZERO, 0.0);
    assert_eq!(front, 0.0);
    let coincident = incidence_angle(Some(Vector3::ZERO), Vector3::ZERO, 0.0);
    assert!((coincident - std::f64::consts::FRAC_PI_2).abs() < 1.0e-12);
    assert_eq!(
        incidence_angle(None, Vector3::ZERO, 0.0),
        MISSING_SOURCE_ANGLE
    );
}

fn default_block_input<'a>(
    amount: f32,
    reductions: &'a [DamageReduction],
) -> ResolveBlockingInput<'a> {
    ResolveBlockingInput {
        amount,
        mature_stack: true,
        bypassed_by_damage_type: false,
        piercing_arrow: false,
        angle: 0.0,
        reductions,
        player_victim: true,
        durability_threshold: 3.0,
        durability_base: 1.0,
        durability_factor: 1.0,
        projectile_damage_type: false,
        living_direct_attacker: true,
    }
}

#[test]
fn ordered_block_reductions_cap_once_and_request_shield_durability() {
    let reductions = [
        DamageReduction {
            horizontal_angle_degrees: 90.0,
            damage_type_matches: true,
            base: 0.0,
            factor: 0.75,
        },
        DamageReduction {
            horizontal_angle_degrees: 90.0,
            damage_type_matches: true,
            base: 1.0,
            factor: 0.5,
        },
    ];
    let blocked = resolve_blocking(default_block_input(4.0, &reductions));
    assert_eq!(blocked.blocked_amount, 4.0);
    assert_eq!(blocked.requested_durability, 5);
    assert!(blocked.item_used_stat && blocked.retaliate);
    let below = resolve_blocking(default_block_input(2.999, &reductions[..1]));
    assert_eq!(below.requested_durability, 0);

    let nan = resolve_blocking(default_block_input(f32::NAN, &reductions[..1]));
    assert!(nan.blocked_amount.is_nan());
    assert_eq!(nan.requested_durability, 0);
    assert!(nan.item_used_stat && !nan.retaliate);
}

#[test]
fn retaliation_hoglin_ravager_and_disable_paths_keep_rng_and_order() {
    assert_eq!(
        retaliation(AttackerKind::BabyHoglin, 4.0, 0, 0.0, Vector3::ZERO),
        Retaliation::None
    );
    assert_eq!(
        retaliation(AttackerKind::AdultZoglin, 4.0, 0, 0.0, Vector3::ZERO),
        Retaliation::HoglinThrow
    );
    assert_eq!(
        retaliation(AttackerKind::Ravager, 4.0, 0, 0.49, Vector3::ZERO),
        Retaliation::RavagerStun {
            stunned_ticks: 40,
            event: 39,
            push_victim: true,
            dirty: true,
        }
    );
    let push = retaliation(
        AttackerKind::Ravager,
        4.0,
        0,
        0.5,
        Vector3::new(1.0, 0.0, 0.0),
    );
    assert!(matches!(push, Retaliation::RavagerPush { .. }));

    let no_throw = hoglin_throw(1.0, 1.0, Vector3::ZERO, 0, 0.0, 0.0);
    assert!(!no_throw.dirty);
    assert_eq!(no_throw.draws_consumed, 0);
    let thrown = hoglin_throw(2.0, 1.0, Vector3::new(1.0, 0.0, 0.0), 10, 0.0, 1.0);
    assert_vector(thrown.velocity, Vector3::new(0.2, 0.5, 0.0));
    assert_eq!(thrown.draws_consumed, 3);

    let disabled = disable_blocking(5.0, 1.0, true, true, true);
    assert_eq!(disabled.cooldown_ticks, 100);
    assert!(disabled.stop_use && disabled.emit_interact_finish);
    assert!(disabled.play_disable_sound_after_stop);
    assert_eq!(attacker_disable_seconds(true, None, false), 5.0);
    assert_eq!(attacker_disable_seconds(false, Some(5.0), false), 0.0);
    assert_eq!(attacker_disable_seconds(false, Some(5.0), true), 5.0);
    assert_eq!(block_sound_pitch(0.5), 1.0);
}

#[test]
fn armor_durability_order_and_breach_formula_use_selected_amount() {
    let armor = armor_durability(ArmorOwner::Player, 3.999);
    assert_eq!(armor.request_per_selected_slot, 1);
    assert_eq!(armor.selected_slots, &PLAYER_ARMOR_ORDER);
    assert_eq!(
        armor_durability(ArmorOwner::Horse, 8.0).selected_slots,
        &[EquipmentSlot::Body]
    );
    assert!(
        armor_durability(ArmorOwner::OrdinaryLiving, 8.0)
            .selected_slots
            .is_empty()
    );
    assert!(armor_slot_takes_durability(true, true, false));
    assert!(!armor_slot_takes_durability(true, true, true));
    assert_eq!(PROTECTION_ORDER.len(), 8);
    let without_breach = armor_reduction(10.0, 20.9, 0.0, &[]);
    assert!((without_breach - 4.0).abs() < 1.0e-6);
    assert_eq!(armor_reduction(10.0, 20.9, 0.0, &[4]), 10.0);
}

#[test]
fn resistance_protection_and_witch_reductions_keep_bypass_and_caps() {
    let resisted = resistance(10.0, 0, true, false);
    assert_eq!((resisted.amount, resisted.resisted), (8.0, 2.0));
    assert_eq!(resisted.victim_stat, Some(20));
    assert_eq!(resistance(10.0, 4, false, true).amount, 0.0);
    assert_eq!(protection_reduction(10.0, &[19.0]), 2.4);
    assert!((protection_reduction(10.0, &[19.0, 2.0]) - 2.0).abs() < 1.0e-6);

    let magic = magic_reduction(MagicReductionInput {
        amount: 10.0,
        bypasses_effects: false,
        resistance_amplifier: Some(0),
        bypasses_resistance: false,
        bypasses_enchantments: false,
        protection_contributions: &[5.0],
        victim_is_server_player: true,
        nonplayer_with_server_player_attacker: false,
    });
    assert!((magic.amount - 6.4).abs() < 1.0e-6);
    assert_eq!(witch_reduction(10.0, false, true), 1.5);
    assert_eq!(witch_reduction(10.0, true, false), 0.0);
}

#[test]
fn absorption_health_exhaustion_and_combat_boundaries_are_ordered() {
    let absorbed = apply_absorption_and_health(HealthReductionInput {
        defended_amount: 4.0,
        absorption: 4.0,
        maximum_absorption: 8.0,
        health: 20.0,
        maximum_health: 20.0,
        player_victim: true,
        ability_invulnerable: false,
        source_exhaustion: 2.0,
        current_exhaustion: 39.0,
        causing_server_player: false,
    });
    assert_eq!((absorbed.absorption, absorbed.health), (0.0, 20.0));
    assert!(!absorbed.record_combat && !absorbed.emit_entity_damage);
    assert_eq!(absorbed.absorbed_stat, Some(40));

    let hurt = apply_absorption_and_health(HealthReductionInput {
        defended_amount: 6.0,
        absorption: 2.0,
        maximum_absorption: 8.0,
        health: 20.0,
        maximum_health: 20.0,
        player_victim: true,
        ability_invulnerable: false,
        source_exhaustion: 2.0,
        current_exhaustion: 39.0,
        causing_server_player: false,
    });
    assert_eq!(
        (hurt.health_damage, hurt.health, hurt.exhaustion),
        (4.0, 16.0, 40.0)
    );
    assert!(hurt.record_combat && hurt.emit_entity_damage);
    assert!(!combat_expired(true, false, 100, 0, false));
    assert!(combat_expired(true, false, 101, 0, false));
    assert!(!combat_expired(true, true, 300, 0, false));
    assert!(combat_expired(true, true, 301, 0, false));

    let overflow = apply_absorption_and_health(HealthReductionInput {
        defended_amount: f32::INFINITY,
        absorption: 2.0,
        maximum_absorption: 8.0,
        health: 20.0,
        maximum_health: 20.0,
        player_victim: false,
        ability_invulnerable: false,
        source_exhaustion: 0.0,
        current_exhaustion: 0.0,
        causing_server_player: false,
    });
    assert!(overflow.absorption.is_nan());
    assert_eq!(overflow.health, 0.0);
}

#[test]
fn wolf_armor_and_animal_hooks_preserve_interception_and_thresholds() {
    assert_eq!(wolf_armor_crack(Some(0.95)), WolfArmorCrack::None);
    assert_eq!(wolf_armor_crack(Some(0.949)), WolfArmorCrack::Low);
    assert_eq!(wolf_armor_crack(Some(0.689)), WolfArmorCrack::Medium);
    assert_eq!(wolf_armor_crack(Some(0.319)), WolfArmorCrack::High);
    let wolf = wolf_armor_outcome(3.2, true, false, WolfArmorCrack::Low, WolfArmorCrack::High);
    assert!(wolf.intercepts_common_damage && wolf.crack_changed);
    assert_eq!((wolf.requested_durability, wolf.particles), (4, 20));

    let hooks = subtype_hooks(true, true, true);
    assert!(hooks.camel_stands && hooks.clear_animal_love_before_defense);
    assert!(hooks.copper_golem_idle_after_defense);
    assert_eq!(hooks.camel_pose_tick_offset, 53);
    assert_eq!(
        armadillo_post_damage(ArmadilloInput {
            no_ai: false,
            dead_or_dying: false,
            causing_living: true,
            panicking: false,
            in_liquid: false,
            leashed: false,
            passenger: false,
            vehicle: false,
            already_scared: false,
            environmental_panic: false,
        }),
        ArmadilloAction::RollUp { ticks: 80 }
    );
}

#[test]
fn source_direction_and_common_knockback_preserve_retry_cardinality() {
    assert_eq!(
        projectile_direction(
            ProjectileDirectionKind::Ordinary,
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, -1.0),
            Vector3::ZERO,
        ),
        (-1.0, 1.0)
    );
    assert_eq!(
        projectile_direction(
            ProjectileDirectionKind::FireworkOrPotion,
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::ZERO,
            Vector3::new(3.0, 0.0, 4.0),
        ),
        (-2.0, -3.0)
    );
    assert_eq!(
        positioned_source_direction(
            Some(Vector3::new(2.0, 0.0, 3.0)),
            Vector3::new(1.0, 0.0, 1.0)
        ),
        (1.0, 2.0)
    );
    let resisted = common_knockback(0.4_f32.into(), 1.0, 1.0, 0.0, Vector3::ZERO, true, &[]);
    assert!(!resisted.dirty && resisted.completed);
    let exhausted = common_knockback(0.4_f32.into(), 0.0, 0.0, 0.0, Vector3::ZERO, true, &[]);
    assert!(!exhausted.completed);
    let result = common_knockback(
        0.4_f32.into(),
        0.0,
        0.0,
        0.0,
        Vector3::new(0.2, -0.2, 0.4),
        true,
        &[[1.0, 0.0, 0.5, 0.0]],
    );
    assert!(result.dirty && result.completed);
    assert_eq!(result.draws_consumed, 4);
    assert!((result.velocity.y - 0.3).abs() < 1.0e-8);
    assert_eq!(DIRECTION_EPSILON_SQUARED, f64::from(1.0e-5_f32));
}

#[test]
fn five_argument_gates_and_player_indication_are_independent_from_velocity() {
    assert!(five_argument_admitted(FiveArgumentGate::Common));
    assert!(!five_argument_admitted(FiveArgumentGate::CreakingImmobile));
    assert!(!five_argument_admitted(FiveArgumentGate::DragonSitting));
    assert!(uses_sulfur_special(true, true));
    assert!(!uses_sulfur_special(true, false));
    assert_eq!(damage_indication(true, true, 1.0, 0.0, 0.0), None);
    let indication = damage_indication(false, true, 1.0, 0.0, 30.0).unwrap();
    assert_eq!(indication.hurt_direction, -30.0);
    assert!(indication.send_hurt_animation);
}

#[test]
fn sulfur_settings_last_match_and_special_velocity_keep_zero_power_side_effects() {
    assert_eq!(
        sulfur_settings(SulfurArchetype::FastFlat).horizontal,
        0.9125
    );
    let settings =
        active_sulfur_settings(&[SulfurArchetype::FastFlat, SulfurArchetype::SlowBouncy]);
    assert_eq!((settings.horizontal, settings.vertical), (0.4125, 0.24));
    assert_eq!(settings.sound_suffix, "slow_bouncy.hit");
    let result = sulfur_knockback(SulfurKnockbackInput {
        old_velocity: Vector3::new(1.0, 2.0, 3.0),
        cube_position: Vector3::new(1.0, 0.0, 0.0),
        cube_center: Vector3::new(1.0, 1.0, 0.0),
        cube_height: 2.0,
        attacker_position: Vector3::ZERO,
        attacker_eye: Vector3::new(0.0, 1.0, 0.0),
        attacker_look: Vector3::new(1.0, 0.0, 0.0),
        direction_x: 1.0,
        direction_z: 0.0,
        amount: 4.0,
        strength: 0.4,
        final_boolean: false,
        resistance: 1.0,
        settings,
    });
    assert_vector(result.velocity, Vector3::new(1.0, 2.0, 3.0));
    assert!(result.dirty);
    assert_eq!(result.sound_suffix, "slow_bouncy.hit");
}
