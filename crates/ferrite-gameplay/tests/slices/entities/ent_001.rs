use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use ferrite_foundation::identity::StableEntityId;
use ferrite_gameplay::entity::runtime::ent_001::aquatic::{
    DOLPHIN_MAX_MOISTNESS, DolphinMoistureStep, FishKind, GlowInk, PuffState, SalmonSize,
    SchoolState, Sting, TadpoleAgeStep, TropicalSelection, TropicalVariant, dolphin_moisture_step,
    fish_air_step, glow_squid_ink, may_join_school, puff_step, pufferfish_sting, salmon_size_roll,
    school_search_due, squid_ink, tadpole_age_step, tadpole_food_acceleration, tropical_selection,
};
use ferrite_gameplay::entity::runtime::ent_001::catalog::{
    EntityCoverage, OWNERS, verify_entities,
};
use ferrite_gameplay::entity::runtime::ent_001::drops::{
    DropEffect, FallingFailure, FrameDropInput, Remover, convert_leashed_statue,
    destroy_container_vehicle, destroy_vehicle, detach_invalid_leash, drop_frame, drop_painting,
    falling_failure, split_container_slot,
};
use ferrite_gameplay::entity::runtime::ent_001::hostile::{
    BlazeAttack, BreezeShotPhase, GhastCharge, PhantomSize, SpiderEffect, bat_step,
    blaze_attack_step, breeze_shot_step, breeze_wind_charge_hit, cave_spider_poison,
    endermite_pearl_spawn, endermite_persists, ghast_attack_step, guardian_beam_step,
    guardian_thorns, phantom_size, shulker_bullet_cooldown, shulker_emergency_teleport,
    shulker_peek, slime_child_count, slime_jump_delay, slime_profile, spider_finalization,
    vex_life_step,
};
use ferrite_gameplay::entity::runtime::ent_001::lifecycle::{
    CRAMMING_DAMAGE, CrammingInput, EntityClass, EntityLifecycle, EntityRecord, LifecycleEffect,
    RemovalReason, RideAdmission, SectionKey, Visibility, cramming,
};
use ferrite_gameplay::entity::runtime::ent_001::passive::{
    GolemCrackiness, TraderUseResult, VillagerLevel, golem_crackiness, iron_golem_attack_damage,
    repair_iron_golem, snow_golem_tick, snowball_damage, village_golem_candidate, villager_level,
    villager_restock, wandering_trader_despawn, wandering_trader_interaction,
};
use ferrite_gameplay::entity::runtime::ent_001::profiles::{EntityKind, Experience, MobCategory};
use ferrite_gameplay::entity::runtime::ent_001::raider::{
    CrossbowState, EvokerSpell, VindicatorWeapon, evoker_fang_count, evoker_spell,
    evoker_vex_summon, illusioner_spells, johnny_latch, piglin_brute_conversion,
    pillager_charge_duration, pillager_crossbow_step, vindicator_break_door,
    vindicator_raid_weapon, vindicator_wave_count,
};
use ferrite_gameplay::entity::runtime::ent_001::undead::{
    ArrowEffect, BoggedShear, Difficulty, SkeletonConversion, SkeletonKind, arrow_effect,
    arrow_inaccuracy, daylight_step, parched_jockey, shear_bogged, skeleton_attack_interval,
    skeleton_powder_snow_step, slow_skeleton_attack_interval, wither_melee,
};
use ferrite_registry::bundle::ContentBundle;

const EXPECTED_SLICES: [&str; 37] = [
    "ENT-BAT-RUNTIME-001",
    "ENT-BLAZE-RUNTIME-001",
    "ENT-BOGGED-RUNTIME-001",
    "ENT-BREEZE-RUNTIME-001",
    "ENT-COD-RUNTIME-001",
    "ENT-DOLPHIN-RUNTIME-001",
    "ENT-ELDER-GUARDIAN-RUNTIME-001",
    "ENT-ENDERMITE-RUNTIME-001",
    "ENT-ENTITY-DROPS-001",
    "ENT-EVOKER-RUNTIME-001",
    "ENT-GHAST-RUNTIME-001",
    "ENT-GIANT-RUNTIME-001",
    "ENT-GLOW-SQUID-RUNTIME-001",
    "ENT-GUARDIAN-RUNTIME-001",
    "ENT-ILLUSIONER-RUNTIME-001",
    "ENT-IRON-GOLEM-RUNTIME-001",
    "ENT-LIFECYCLE-001",
    "ENT-PARCHED-RUNTIME-001",
    "ENT-PHANTOM-RUNTIME-001",
    "ENT-PIGLIN-BRUTE-RUNTIME-001",
    "ENT-PILLAGER-RUNTIME-001",
    "ENT-PUFFERFISH-RUNTIME-001",
    "ENT-SALMON-RUNTIME-001",
    "ENT-SHULKER-RUNTIME-001",
    "ENT-SKELETON-RUNTIME-001",
    "ENT-SLIME-FAMILY-RUNTIME-001",
    "ENT-SNOW-GOLEM-RUNTIME-001",
    "ENT-SPIDER-RUNTIME-001",
    "ENT-SQUID-RUNTIME-001",
    "ENT-STRAY-RUNTIME-001",
    "ENT-TADPOLE-RUNTIME-001",
    "ENT-TROPICAL-FISH-RUNTIME-001",
    "ENT-VEX-RUNTIME-001",
    "ENT-VILLAGER-RUNTIME-001",
    "ENT-VINDICATOR-RUNTIME-001",
    "ENT-WANDERING-TRADER-RUNTIME-001",
    "ENT-WITHER-SKELETON-RUNTIME-001",
];

fn id(value: u128) -> StableEntityId {
    StableEntityId::new(value).unwrap()
}

fn section(x: i32, visibility: Visibility) -> EntityRecord {
    EntityRecord::new(
        id(x.unsigned_abs().into()),
        EntityClass::Ordinary,
        SectionKey { x, y: 0, z: 0 },
        visibility,
    )
}

#[test]
fn ent_001_owns_all_thirty_seven_source_specified_slices_and_identities() {
    let mut actual = OWNERS
        .iter()
        .map(|owner| owner.slice)
        .collect::<BTreeSet<_>>();
    actual.extend(["ENT-ENTITY-DROPS-001", "ENT-LIFECYCLE-001"]);
    assert_eq!(actual, EXPECTED_SLICES.into_iter().collect());
    assert_eq!(OWNERS.len(), 37);
    assert_eq!(
        OWNERS
            .iter()
            .map(|owner| owner.path)
            .collect::<BTreeSet<_>>()
            .len(),
        37
    );
    assert_eq!(
        EntityKind::ALL
            .into_iter()
            .map(EntityKind::path)
            .collect::<BTreeSet<_>>(),
        OWNERS
            .iter()
            .map(|owner| owner.path)
            .collect::<BTreeSet<_>>()
    );

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
        .find(|registry| registry.name().to_string() == "minecraft:entity_type")
        .unwrap();
    assert_eq!(registry.entries().len(), 158);
    assert_eq!(
        verify_entities(registry).unwrap(),
        EntityCoverage {
            identities: 37,
            slices: 35,
        }
    );
}

#[test]
fn all_owned_entity_factories_have_locked_construction_profiles() {
    for kind in EntityKind::ALL {
        assert_eq!(EntityKind::from_path(kind.path()), Some(kind));
        assert_eq!(kind.profile().update_interval, 3);
        assert!(kind.profile().width > 0.0);
        assert!(kind.profile().height > 0.0);
        assert!(kind.profile().max_health > 0.0);
    }
    assert_eq!(EntityKind::from_path("not_an_entity"), None);
    let bat = EntityKind::Bat.profile();
    assert_eq!(
        (bat.width, bat.height, bat.eye_height, bat.max_health),
        (0.5, 0.9, 0.45, 6.0)
    );
    assert_eq!(bat.category, MobCategory::Ambient);
    let giant = EntityKind::Giant.profile();
    assert_eq!(
        (giant.width, giant.height, giant.eye_height),
        (3.6, 12.0, 10.44)
    );
    assert_eq!(giant.attack_damage, Some(50.0));
    let glow_squid = EntityKind::GlowSquid.profile();
    assert_eq!(
        glow_squid.experience,
        Experience::Range {
            minimum: 1,
            maximum: 3,
        }
    );
    assert_eq!(EntityKind::WitherSkeleton.profile().height, 2.4);
}

#[test]
fn lifecycle_preserves_insertion_visibility_move_and_duplicate_order() {
    let mut lifecycle = EntityLifecycle::default();
    assert_eq!(
        lifecycle
            .add(section(1, Visibility::Accessible), true)
            .unwrap(),
        [
            LifecycleEffect::KnownUuidAdded,
            LifecycleEffect::SectionAdded(SectionKey { x: 1, y: 0, z: 0 }),
            LifecycleEffect::CallbackInstalled,
            LifecycleEffect::Created,
            LifecycleEffect::TrackingStarted,
        ]
    );
    assert_eq!(
        lifecycle
            .add(section(1, Visibility::Ticking), false)
            .unwrap(),
        [LifecycleEffect::DuplicateOrdinaryRejected]
    );
    assert_eq!(
        lifecycle
            .move_section(id(1), SectionKey { x: 2, y: 0, z: 0 }, Visibility::Ticking,)
            .unwrap(),
        [
            LifecycleEffect::SectionRemoved(SectionKey { x: 1, y: 0, z: 0 }),
            LifecycleEffect::SectionAdded(SectionKey { x: 2, y: 0, z: 0 }),
            LifecycleEffect::TickingStarted,
            LifecycleEffect::DynamicListenerMoved,
            LifecycleEffect::SectionChanged,
        ]
    );
    lifecycle.remove(id(1), RemovalReason::Discarded).unwrap();
    assert_ne!(
        lifecycle
            .add(section(1, Visibility::Hidden), false)
            .unwrap(),
        [LifecycleEffect::DuplicateOrdinaryRejected]
    );
}

#[test]
fn lifecycle_mount_tick_remove_and_teleport_transactions_are_ordered() {
    let mut lifecycle = EntityLifecycle::default();
    let mut vehicle = section(1, Visibility::Ticking);
    vehicle.class = EntityClass::AlwaysTicking;
    lifecycle.add(vehicle, true).unwrap();
    let passenger = section(2, Visibility::Ticking);
    lifecycle.add(passenger, true).unwrap();
    lifecycle
        .start_riding(
            id(2),
            id(1),
            RideAdmission {
                force: false,
                shifting: false,
                vehicle_accepts: true,
                ride_admitted: true,
                server_side: true,
            },
        )
        .unwrap();
    let ticks = lifecycle
        .tick(&[SectionKey { x: 1, y: 0, z: 0 }].into_iter().collect())
        .unwrap();
    assert!(ticks.contains(&(id(1), LifecycleEffect::TickRoot)));
    assert!(ticks.contains(&(id(2), LifecycleEffect::TickPassenger)));

    assert_eq!(
        lifecycle.teleport_same_dimension(id(1), false).unwrap(),
        [
            LifecycleEffect::SameDimensionPlaced(id(2)),
            LifecycleEffect::SameDimensionPlaced(id(1)),
            LifecycleEffect::PostTransition,
        ]
    );
    let failed = lifecycle.teleport_cross_dimension(id(1), false).unwrap();
    assert!(failed.contains(&LifecycleEffect::PassengerTransferred(id(2))));
    assert!(!failed.contains(&LifecycleEffect::DestinationRootCreated));
    assert!(lifecycle.entity(id(1)).unwrap().removed.is_none());

    lifecycle
        .start_riding(
            id(2),
            id(1),
            RideAdmission {
                force: true,
                shifting: false,
                vehicle_accepts: true,
                ride_admitted: true,
                server_side: true,
            },
        )
        .unwrap();
    let success = lifecycle.teleport_cross_dimension(id(1), true).unwrap();
    assert_eq!(
        success.last(),
        Some(&LifecycleEffect::SpectatorsTransferred)
    );
    assert!(success.contains(&LifecycleEffect::PassengerRemounted(id(2))));
    assert!(lifecycle.entity(id(1)).unwrap().removed.is_none());
    assert_eq!(lifecycle.entity(id(2)).unwrap().vehicle, Some(id(1)));
}

#[test]
fn cramming_keeps_pushes_independent_from_damage() {
    let damaging = cramming(CrammingInput {
        server_side: true,
        limit: 24,
        raw_neighbors: 25,
        nonpassenger_neighbors: 24,
        draw_four: 0,
        damage_admitted: true,
    });
    assert_eq!(damaging.damage, CRAMMING_DAMAGE);
    assert_eq!(damaging.pushed_neighbors, 25);
    let disabled = cramming(CrammingInput {
        limit: 0,
        ..CrammingInput {
            server_side: true,
            limit: 24,
            raw_neighbors: 25,
            nonpassenger_neighbors: 24,
            draw_four: 0,
            damage_admitted: true,
        }
    });
    assert_eq!(disabled.damage, 0);
    assert_eq!(disabled.pushed_neighbors, 25);
}

#[test]
fn seven_entity_drop_read_sites_keep_their_distinct_boundaries() {
    assert_eq!(destroy_vehicle(false), [DropEffect::CarrierKilled]);
    assert_eq!(
        destroy_container_vehicle(true, &[false, true], true),
        [
            DropEffect::ContainerSlotVisited(0),
            DropEffect::ContainerSlotVisited(1),
            DropEffect::ContainerContentsDropped,
            DropEffect::PiglinsAngered,
        ]
    );
    assert_eq!(
        split_container_slot(45, &[0, 20, 4]).unwrap().piece_counts,
        [10, 30, 5]
    );
    assert_eq!(
        split_container_slot(0, &[]).unwrap().position_double_draws,
        3
    );
    assert_eq!(
        split_container_slot(45, &[0, 20, 4])
            .unwrap()
            .velocity_double_draws,
        18
    );
    assert!(drop_painting(false, Remover::Ordinary).is_empty());
    assert_eq!(
        drop_painting(true, Remover::InfiniteMaterials),
        [DropEffect::PaintingBreakSound]
    );
    assert_eq!(
        drop_frame(FrameDropInput {
            fixed: false,
            entity_drops: false,
            remover: Remover::None,
            drop_frame: true,
            displayed_item: true,
            displayed_map: true,
            drop_chance: 1.0,
            draw: 0.0,
        }),
        [DropEffect::FrameItemCleared, DropEffect::FrameMapRemoved]
    );
    assert_eq!(
        detach_invalid_leash(false),
        [
            DropEffect::LeashDataCleared,
            DropEffect::LeashRemovedCallback,
            DropEffect::LeashLinkPacket,
            DropEffect::LeashHolderNotified,
        ]
    );
    assert!(falling_failure(FallingFailure::PlacementWriteFailed, true, false).is_empty());
    assert_eq!(
        falling_failure(FallingFailure::TimedOut, true, true),
        [
            DropEffect::FallingEntityDiscarded,
            DropEffect::FallingBlockItemSpawned,
        ]
    );
    let statue = convert_leashed_statue(false);
    assert_eq!(statue[0], DropEffect::StatueCommitted);
    assert!(statue.contains(&DropEffect::PreservedEquipmentDropped));
    assert!(!statue.contains(&DropEffect::LeadSpawned));
}

#[test]
fn fish_school_variant_air_and_puff_boundaries_match_reference() {
    assert!(may_join_school(
        FishKind::Cod,
        SchoolState {
            school_size: 1,
            has_leader: false,
        },
        SchoolState {
            school_size: 7,
            has_leader: false,
        },
        121,
    ));
    assert!(!may_join_school(
        FishKind::Salmon,
        SchoolState {
            school_size: 1,
            has_leader: false,
        },
        SchoolState {
            school_size: 5,
            has_leader: false,
        },
        1,
    ));
    assert!(school_search_due(200));
    assert!(school_search_due(219));
    assert_eq!(fish_air_step(false, -20).damage, 2);
    assert_eq!(salmon_size_roll(29), SalmonSize::Small);
    assert_eq!(salmon_size_roll(30), SalmonSize::Medium);
    assert_eq!(salmon_size_roll(94), SalmonSize::Large);
    let rare = TropicalVariant {
        pattern: 1_281,
        base_color: 15,
        pattern_color: 3,
    };
    assert_eq!(TropicalVariant::unpack(rare.packed()), rare);
    assert_eq!(
        tropical_selection(90, 21, rare),
        TropicalSelection::Rare(rare)
    );

    let detected = puff_step(PuffState::Small, 0, 0, true);
    assert_eq!(detected.state, PuffState::Small);
    assert_eq!(detected.inflate_counter, 1);
    let mid = puff_step(PuffState::Small, detected.inflate_counter, 0, true);
    assert_eq!(mid.state, PuffState::Mid);
    let full = puff_step(PuffState::Mid, 41, 0, true);
    assert_eq!(full.state, PuffState::Full);
    let stopped = puff_step(PuffState::Full, 42, 0, false);
    assert_eq!(stopped.inflate_counter, 0);
    assert_eq!(
        puff_step(PuffState::Full, 0, 61, false).state,
        PuffState::Mid
    );
    assert_eq!(
        pufferfish_sting(2, true),
        Sting {
            damage: 3,
            poison_ticks: 120,
        }
    );
}

#[test]
fn squid_dolphin_and_tadpole_counters_keep_exact_thresholds() {
    assert_eq!(squid_ink(true, true).packets, 30);
    assert_eq!(squid_ink(true, true).position_float_draws, 90);
    assert_eq!(
        glow_squid_ink(true, true),
        GlowInk {
            packets: 30,
            server_position_draws: 3,
            client_glow_requests: 1,
        }
    );
    assert_eq!(
        dolphin_moisture_step(true, false, true, 1),
        DolphinMoistureStep {
            moisture: DOLPHIN_MAX_MOISTNESS,
            damage: 0,
            flop: false,
        }
    );
    assert_eq!(
        tadpole_age_step(23_999, false, 24_000),
        TadpoleAgeStep {
            age: 24_000,
            converts: true,
        }
    );
    assert_eq!(tadpole_food_acceleration(4_000, 24_000), 2_000);
}

#[test]
fn hostile_state_machines_preserve_reference_cadence_and_quirks() {
    assert!(bat_step(false, true, false, 1, 10).flap);
    assert_eq!(
        blaze_attack_step(1, 0, false, true, true).action,
        BlazeAttack::Fireball {
            volley_index: 1,
            level_event: 1018,
        }
    );
    let charged = breeze_shot_step(BreezeShotPhase::Charging, 1);
    assert!(charged.projectile_spawned);
    assert_eq!(breeze_wind_charge_hit().explosion_strength, 3);
    assert!(endermite_persists(false, 2_399));
    assert!(!endermite_persists(false, 2_400));
    assert!(endermite_pearl_spawn(true, true, Difficulty::Hard, 0));
    assert_eq!(
        ghast_attack_step(19, true, false, -3).phase,
        GhastCharge::Fired
    );
    assert_eq!(ghast_attack_step(19, true, false, -3).fireball_power, -3);

    let slime = slime_profile(200, true);
    assert_eq!(slime.size, 127);
    assert_eq!(slime.max_health, 16_129.0);
    assert_eq!((slime.attack_damage, slime.contact_damage), (127.0, 129.0));
    assert_eq!(slime_child_count(2, 2), 4);
    assert_eq!(slime_jump_delay(0, true, true), 13);
    assert_eq!(
        spider_finalization(0, true, 3).group_effect,
        Some(SpiderEffect::Invisibility)
    );
    assert_eq!(cave_spider_poison(Difficulty::Normal), 140);
}

#[test]
fn phantom_guardian_shulker_and_vex_boundaries_are_exact() {
    let fresh = PhantomSize {
        stored_size: 0,
        scale: 1.0,
        attack_damage: 2.0,
    };
    assert_eq!(phantom_size(fresh, 0).attack_damage, 2.0);
    assert_eq!(phantom_size(fresh, 1).attack_damage, 7.0);
    let guardian = guardian_beam_step(79, false, true);
    assert_eq!((guardian.magic_damage, guardian.melee_damage), (3, 6));
    assert_eq!(guardian_thorns(false, true, true), 2);
    assert_eq!(shulker_peek(0).armor_bonus, 20);
    assert_eq!(shulker_bullet_cooldown(9), 110);
    assert!(shulker_emergency_teleport(14, 30, true));
    assert_eq!(vex_life_step(Some(1)).unwrap().life_ticks, 0);
    assert_eq!(vex_life_step(Some(-19)).unwrap().starvation_damage, 1);
}

#[test]
fn skeleton_family_keeps_intervals_effects_and_conversion_edges() {
    assert_eq!(skeleton_attack_interval(Difficulty::Hard), 20);
    assert_eq!(slow_skeleton_attack_interval(Difficulty::Hard), 50);
    assert_eq!(arrow_inaccuracy(Difficulty::Hard), 2);
    assert_eq!(
        arrow_effect(SkeletonKind::Bogged, true),
        ArrowEffect::Poison { duration: 100 }
    );
    assert_eq!(
        arrow_effect(SkeletonKind::WitherSkeleton, false),
        ArrowEffect::Fire { duration: 2_000 }
    );
    let state = skeleton_powder_snow_step(
        SkeletonConversion {
            exposure_ticks: 139,
            conversion_ticks: 0,
            converting: false,
            converts_now: false,
        },
        true,
        true,
        true,
    );
    assert!(state.converting);
    assert_eq!(state.conversion_ticks, 300);
    assert!(
        skeleton_powder_snow_step(
            SkeletonConversion {
                exposure_ticks: 140,
                conversion_ticks: 0,
                converting: true,
                converts_now: false,
            },
            true,
            true,
            true,
        )
        .converts_now
    );
    assert!(daylight_step(true, true, false).head_item_damaged);
    assert_eq!(
        shear_bogged(true, false, 0, 3),
        BoggedShear {
            sheared: true,
            durability_spent: true,
            game_event_emitted: true,
            brown_mushrooms: 1,
            red_mushrooms: 1,
        }
    );
    assert_eq!(wither_melee(true).wither_duration, 200);
    assert!(parched_jockey(0, true).creates_camel);
}

#[test]
fn raider_spell_crossbow_conversion_and_raid_edges_are_explicit() {
    assert_eq!(evoker_spell(true, true, true), Some(EvokerSpell::SummonVex));
    assert_eq!(evoker_fang_count(8), 13);
    assert_eq!(evoker_fang_count(9), 16);
    let summon = evoker_vex_summon(2, 3);
    assert!(summon.admitted);
    assert_eq!(summon.attempts, 3);
    assert!(illusioner_spells(false, true, 3).blindness_admitted);
    assert!(piglin_brute_conversion(300, false, false).converts);
    assert_eq!(pillager_charge_duration(0), 25);
    assert_eq!(
        pillager_crossbow_step(CrossbowState::Ready, 0, 25, 0).state,
        CrossbowState::Uncharged
    );
    assert_eq!(
        vindicator_raid_weapon(6, true),
        VindicatorWeapon::SharpnessTwoAxe
    );
    assert_eq!(vindicator_raid_weapon(6, false), VindicatorWeapon::IronAxe);
    assert_eq!(
        (1..=7).map(vindicator_wave_count).collect::<Vec<_>>(),
        [0, 2, 0, 1, 4, 2, 5]
    );
    assert!(johnny_latch(false, Some("Johnny")));
    assert!(johnny_latch(true, Some("not Johnny")));
    assert!(!johnny_latch(false, None));
    assert!(vindicator_break_door(true, true, Difficulty::Normal, 240));
}

#[test]
fn golem_villager_and_trader_rules_keep_exact_thresholds() {
    assert_eq!(golem_crackiness(24, 100), GolemCrackiness::High);
    assert_eq!(golem_crackiness(25, 100), GolemCrackiness::Medium);
    assert_eq!(repair_iron_golem(99, 100).healed, 25);
    assert_eq!(iron_golem_attack_damage(15.0, 14), 21.5);
    assert!(village_golem_candidate(24_000, 599, 5));
    assert_eq!(snowball_damage(true), 3);
    assert_eq!(
        snow_golem_tick(true, true, true, 3).environmental_damage_attempts,
        2
    );
    assert_eq!(villager_level(9), VillagerLevel::Novice);
    assert_eq!(villager_level(10), VillagerLevel::Apprentice);
    assert_eq!(villager_level(250), VillagerLevel::Master);
    assert!(villager_restock(1, true, true).restock_now);
    let trader = wandering_trader_despawn(1, false);
    assert!(trader.discarded);
    assert_eq!(
        wandering_trader_interaction(false, true, false),
        TraderUseResult::Consumed
    );
}
