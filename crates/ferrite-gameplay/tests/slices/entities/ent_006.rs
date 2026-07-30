use std::fs;
use std::path::Path;

use ferrite_gameplay::entity::runtime::ent_006::instance::{
    EffectInstance, INFINITE_DURATION, add_effect, force_add_plan, merge, remove_all_plan,
};
use ferrite_gameplay::entity::runtime::ent_006::special::{
    Applicability, ApplicabilityEffect, InstantEffect, PeriodicAction, PeriodicEffect,
    RemovalReason, absorption_floor, absorption_keeps_ticking, absorption_on_start, bad_omen_tick,
    client_particle_denominator, effect_applicable, hunger_exhaustion, infested_hurt,
    instant_effect, oozing_attempt, oozing_plan, periodic_action, periodic_interval,
    periodic_scheduled, raid_omen_tick, saturation, weaving_removal, wind_charged_explosion,
};
use ferrite_gameplay::entity::runtime::ent_006::ticking::{
    TickRemoval, attribute_refresh, cadence_value, ordinary_removal, tick_instance, tick_pass,
};
use ferrite_registry::bundle::ContentBundle;

fn effect(duration: i32, amplifier: u16, ambient: bool) -> EffectInstance {
    EffectInstance::new(duration, amplifier, ambient, true, true)
}

#[test]
fn locked_bundle_still_supplies_all_forty_effect_identities() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("../../target/ferrite-content/26.2/content-bundle.json");
    if !path.is_file() {
        eprintln!(
            "locked local artifact bundle is absent; `cargo ferrite content verify` owns that gate"
        );
        return;
    }
    let bundle = serde_json::from_slice::<ContentBundle>(&fs::read(path).unwrap()).unwrap();
    let registry = bundle
        .registries()
        .find(|registry| registry.name().to_string() == "minecraft:mob_effect")
        .unwrap();
    assert_eq!(registry.entries().len(), 40);
}

#[test]
fn construction_clamps_amplifier_and_infinite_duration_stays_live() {
    let instance = effect(INFINITE_DURATION, 300, false);
    assert_eq!(instance.amplifier, 255);
    assert!(instance.has_duration());
    assert!(!effect(0, 0, false).has_duration());
}

#[test]
fn stronger_shorter_merge_preserves_old_visible_ahead_of_hidden_chain() {
    let mut current = effect(100, 1, true);
    current.hidden = Some(Box::new(effect(50, 0, true)));
    let incoming = effect(20, 2, false);
    let outcome = merge(&mut current, &incoming);
    assert!(outcome.changed && outcome.amplifier_or_duration_changed);
    assert_eq!((current.amplifier, current.duration), (2, 20));
    assert!(!current.ambient);
    let old = current.hidden.as_ref().unwrap();
    assert_eq!((old.amplifier, old.duration), (1, 100));
    assert_eq!(old.hidden.as_ref().unwrap().amplifier, 0);
}

#[test]
fn equal_extends_weaker_hides_and_infinite_outranks_finite() {
    let mut current = effect(20, 2, false);
    assert!(merge(&mut current, &effect(21, 2, false)).changed);
    assert_eq!(current.duration, 21);
    assert!(merge(&mut current, &effect(100, 1, false)).changed);
    assert_eq!(current.hidden.as_ref().unwrap().duration, 100);
    assert!(
        merge(
            current.hidden.as_mut().unwrap(),
            &effect(INFINITE_DURATION, 1, false)
        )
        .changed
    );
    assert_eq!(current.hidden.as_ref().unwrap().duration, INFINITE_DURATION);
}

#[test]
fn unchanged_add_still_starts_effect_but_reports_no_map_change() {
    let mut current = effect(20, 1, false);
    let incoming = effect(20, 1, false);
    let plan = add_effect(Some(&mut current), &incoming, true);
    assert!(plan.accepted && plan.effect_started_callback);
    assert!(!plan.map_changed && !plan.living_update_with_attribute_refresh);
    let first = add_effect(None, &incoming, true);
    assert!(first.map_changed && first.living_add_callback);
    assert!(first.effect_added_callback && first.effect_started_callback);
    assert!(first.notify_player_passengers);
    assert!(!add_effect(None, &incoming, false).accepted);
}

#[test]
fn force_add_and_remove_all_keep_their_distinct_callback_contracts() {
    let forced = force_add_plan(true);
    assert!(forced.replace && forced.copy_blend_state && forced.living_update_callback);
    assert!(!forced.effect_added_callback && !forced.effect_started_callback);
    assert!(!force_add_plan(false).living_update_callback);
    assert!(!remove_all_plan(false, 4).admitted);
    let removal = remove_all_plan(true, 4);
    assert!(removal.copy_then_clear_map && removal.remove_modifiers_after_clear);
    assert_eq!(ordinary_removal(true).removal_callbacks, 1);
    assert_eq!(ordinary_removal(false).removal_callbacks, 0);
}

#[test]
fn server_tick_handles_zero_callback_abort_one_tick_and_periodic_refresh() {
    let mut zero = effect(0, 0, false);
    assert_eq!(
        tick_instance(&mut zero, 0, true, true).remove,
        Some(TickRemoval::NoDuration)
    );
    let mut rejected = effect(10, 0, false);
    let rejected_tick = tick_instance(&mut rejected, 0, true, false);
    assert_eq!(rejected_tick.remove, Some(TickRemoval::CallbackRejected));
    assert_eq!(rejected.duration, 10);

    let mut one = effect(1, 0, false);
    let last = tick_instance(&mut one, 0, true, true);
    assert!(last.apply_callback);
    assert_eq!(last.remove, Some(TickRemoval::Expired));

    let mut refresh = effect(601, 0, false);
    let tick = tick_instance(&mut refresh, 0, false, true);
    assert_eq!(refresh.duration, 600);
    assert!(tick.periodic_update_without_refresh);
}

#[test]
fn hidden_duration_ticks_and_promotes_even_after_reaching_zero() {
    let mut visible = effect(2, 2, false);
    visible.hidden = Some(Box::new(effect(1, 1, false)));
    let first = tick_instance(&mut visible, 0, false, true);
    assert!(!first.promoted_hidden);
    assert_eq!(visible.hidden.as_ref().unwrap().duration, 0);
    let second = tick_instance(&mut visible, 1, false, true);
    assert!(second.promoted_hidden && second.update_with_attribute_refresh);
    assert_eq!(visible.amplifier, 1);
    assert_eq!(second.remove, Some(TickRemoval::Expired));
}

#[test]
fn infinite_cadence_attributes_and_concurrent_pass_are_explicit() {
    let infinite = effect(INFINITE_DURATION, 0, false);
    assert_eq!(cadence_value(&infinite, 42), 42);
    let refresh = attribute_refresh(2.5, 2);
    assert_eq!(refresh.permanent_amount, 7.5);
    assert!(refresh.remove_modifier_by_id_first && refresh.clamp_health_and_absorption);
    assert!(refresh.refresh_dimensions && refresh.refresh_waypoint_tracking);
    assert_eq!(tick_pass(5, Some(2)).visited, 3);
    assert_eq!(tick_pass(5, Some(2)).deferred, 2);
    assert!(tick_pass(5, Some(2)).aborted_on_concurrent_modification);
}

#[test]
fn regeneration_poison_wither_hunger_saturation_and_absorption_are_exact() {
    assert_eq!(periodic_interval(PeriodicEffect::Regeneration, 0), 50);
    assert_eq!(periodic_interval(PeriodicEffect::Poison, 0), 25);
    assert_eq!(periodic_interval(PeriodicEffect::Wither, 0), 40);
    assert!(periodic_scheduled(PeriodicEffect::Regeneration, 6, 7));
    assert_eq!(
        periodic_action(PeriodicEffect::Regeneration, 20.0),
        PeriodicAction::Heal { amount: 1 }
    );
    assert_eq!(
        periodic_action(PeriodicEffect::Poison, 1.0),
        PeriodicAction::None
    );
    assert_eq!(
        periodic_action(PeriodicEffect::Poison, 1.01),
        PeriodicAction::Damage { amount: 1 }
    );
    assert_eq!(
        periodic_action(PeriodicEffect::Wither, 1.0),
        PeriodicAction::Damage { amount: 1 }
    );
    assert!((hunger_exhaustion(1) - 0.01).abs() < f32::EPSILON);
    assert_eq!((saturation(255).food, saturation(255).modifier), (256, 1));
    assert_eq!(absorption_floor(255), 1_024);
    assert_eq!(absorption_on_start(20.0, 1), 20.0);
    assert_eq!(absorption_on_start(2.0, 1), 8.0);
    assert!(absorption_keeps_ticking(0.1));
    assert!(!absorption_keeps_ticking(0.0));
}

#[test]
fn instant_heal_harm_inversion_scaling_and_source_kind_are_exact() {
    let heal = instant_effect(InstantEffect::Heal, 1, 0.5, false, false);
    assert_eq!((heal.healing, heal.damage), (4, 6));
    assert!(!heal.indirect_magic_source);
    let inverted = instant_effect(InstantEffect::Heal, 1, 0.5, true, true);
    assert_eq!((inverted.healing, inverted.damage), (6, 4));
    assert!(inverted.indirect_magic_source);
    assert_eq!(
        instant_effect(InstantEffect::Harm, 0, 0.0, false, false).damage,
        0
    );
}

#[test]
fn applicability_and_omen_effects_keep_tick_and_removal_boundaries() {
    let tags = Applicability {
        infested: false,
        oozing: true,
        poison: false,
        regeneration: true,
    };
    assert!(!effect_applicable(ApplicabilityEffect::Infested, tags));
    assert!(effect_applicable(ApplicabilityEffect::Oozing, tags));
    assert!(effect_applicable(ApplicabilityEffect::Other, tags));

    let bad = bad_omen_tick(true, true, true, true);
    assert!(bad.remove && bad.save_position);
    assert_eq!(bad.add_raid_omen_duration, 600);
    assert!(!bad_omen_tick(true, false, true, true).remove);
    assert!(!raid_omen_tick(2).remove);
    let raid = raid_omen_tick(1);
    assert!(raid.create_or_extend_raid && raid.clear_saved_position && raid.remove);
}

#[test]
fn hurt_and_killed_hooks_consume_rng_only_inside_their_exact_gates() {
    assert_eq!(infested_hurt(0.1, 1).silverfish, 2);
    assert!(!infested_hurt(f32::from_bits(0.1_f32.to_bits() + 1), 0).trigger);
    assert_eq!(
        wind_charged_explosion(RemovalReason::Killed, 0.5),
        Some(4.0)
    );
    assert_eq!(wind_charged_explosion(RemovalReason::Discarded, 0.5), None);
    let weaving = weaving_removal(RemovalReason::Killed, false, true, 1);
    assert_eq!((weaving.attempts, weaving.samples_per_attempt), (3, 15));
    assert!(!weaving_removal(RemovalReason::Unloaded, true, true, 0).may_place);
}

#[test]
fn oozing_cramming_construction_and_yaw_cardinality_are_bounded() {
    assert_eq!(oozing_plan(RemovalReason::Killed, 0, 99).spawn_attempts, 2);
    let capped = oozing_plan(RemovalReason::Killed, 2, 1);
    assert!(capped.query_nearby);
    assert_eq!((capped.nearby_scan_limit, capped.spawn_attempts), (2, 1));
    assert_eq!(oozing_plan(RemovalReason::Killed, 24, 24).spawn_attempts, 0);
    assert_eq!(
        oozing_plan(RemovalReason::ChangedDimension, 24, 0).spawn_attempts,
        0
    );
    let failed = oozing_attempt(false, 0.75);
    assert_eq!(failed.yaw, None);
    let created = oozing_attempt(true, 0.75);
    assert_eq!(
        (created.slime_size, created.y_offset, created.yaw),
        (2, 0.5, Some(270.0))
    );
    assert!(!created.finalize_spawn && !created.rollback_failed_insertion);
    assert_eq!(client_particle_denominator(true, false), 4);
    assert_eq!(client_particle_denominator(false, true), 75);
}
