use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::item::runtime::ply_005::alchemy::{
    BrewingBottle, CauldronResult, DragonCloud, brew_dragon_breath, convert_to_mud,
    creative_pour_remainder, fill_dragon_breath, first_dragon_cloud, pour_water_cauldron,
};
use ferrite_gameplay::item::runtime::ply_005::brewing_graph::{
    BrewingIngredient, PotionKind, mix, owned_edge_count,
};
use ferrite_gameplay::item::runtime::ply_005::buckets::{
    CaptureStep, EmptyBucketKind, MobBucketKind, PlacementTarget, ReleaseInput, VariantTransfer,
    capture_mob, placement_target, release_mob, variant_transfer,
};
use ferrite_gameplay::item::runtime::ply_005::bundle::{
    BundleContents, BundleEntry, BundleSound, ClickAction, Fraction, bundle_in_slot_insert,
    bundle_in_slot_remove, bundle_on_cursor_remove, held_output_attempt, recolor_allowed,
};
use ferrite_gameplay::item::runtime::ply_005::catalog::{FamilyCoverage, OWNERS, verify_families};
use ferrite_gameplay::item::runtime::ply_005::consumables::{
    ConsumableKind, ConsumeEffect, PotionContents, PotionEffect, Remainder, SuspiciousStewEffect,
    effect_probability_draws, effects, ominous_level, periodic_drink_sound, profile,
    scaled_potion_effects, stew_remainder_count, suspicious_stew_default_duration,
    suspicious_stew_effects, water_transaction,
};
use ferrite_gameplay::item::runtime::ply_005::equipment::{
    NautilusArmorTier, NautilusEquipInput, ShearedNautilusSlot, SpearTier, SteeringKind,
    boost_multiplier, equip_nautilus_armor, first_nautilus_shear_slot, kinetic_contact, lunge,
    nautilus_armor_profile, spear_profile, stab_charge_admitted, use_steering_stick,
    zombie_nautilus_sun_protected,
};
use ferrite_gameplay::item::runtime::ply_005::knowledge::{
    KnowledgeBookResult, RecipeDisposition, RecipeResolution, use_knowledge_book,
};
use ferrite_gameplay::item::runtime::ply_005::materials::{
    BEACON_RECIPE, CONDUIT_RECIPE, MUSIC_DISC_FIVE_RECIPE, RECOVERY_COMPASS_RECIPE, Rarity,
    TEMPLATE_DUPLICATION, TrialKeyKind, advancement_observes_trial_key, anvil_wolf_armor_repair,
    armadillo_shed_timer, blaze_powder_brewing_uses, blaze_rod_fuel_ticks, bone_tames,
    direct_wolf_armor_repair, elytra_repair_per_membrane, nether_star_world_item_hurt,
    pottery_pattern, shulker_shell_drop_chance, smithing_template_rarity, vault_key_matches,
    wither_nether_star_age,
};
use ferrite_gameplay::item::runtime::ply_005::placements::{
    CrystalBase, EquipmentSlot, Face, HangingKind, PaintingCandidate, StandDamageKind,
    StandDamageOutcome, StandOccupancy, SurvivalCadence, UseResult, damage_armor_stand,
    damage_end_crystal, finalize_hanging_placement, frame_analog_value, frame_map_admitted,
    hanging_admitted, maximal_painting_candidate, place_armor_stand, place_end_crystal,
    quantized_stand_yaw, rotate_frame, select_stand_slot_for_empty_hand, stand_slot_usable,
    swap_stand_slot,
};
use ferrite_gameplay::item::runtime::ply_005::projectiles::{
    AmmunitionKind, EggHatchCount, PickupMode, ProjectileOwner, arrow_color, can_pick_up,
    converts_to_plain_arrow, egg_hatch, egg_hit, imbued_tipped_contents, pickup_after_owner,
    reset_laying_timer, select_player_ammunition, take_ammunition,
};
use ferrite_gameplay::item::runtime::ply_005::vehicles::{
    MinecartKind, RailShape, command_cart_may_activate, destruction_item,
    dispenser_vertical_offset, fuel_furnace_cart, hopper_enabled, interact_ordinary_cart,
    place_minecart, scatters_contents, tnt_fuse,
};
use ferrite_gameplay::item::runtime::stack::ItemStack;
use ferrite_registry::bundle::ContentBundle;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn ply_005_item_families_are_closed_over_all_forty_three_slices() {
    assert_eq!(OWNERS.len(), 44);
    assert_eq!(
        OWNERS
            .iter()
            .map(|owner| owner.slice)
            .collect::<BTreeSet<_>>()
            .len(),
        43
    );
    assert_eq!(
        OWNERS
            .iter()
            .map(|owner| owner.expected_items)
            .sum::<usize>(),
        138
    );
    let minecart = OWNERS
        .iter()
        .filter(|owner| owner.slice == "ITM-MINECART-RUNTIME-001")
        .collect::<Vec<_>>();
    assert_eq!(minecart.len(), 2);
    assert_eq!(
        minecart
            .iter()
            .map(|owner| owner.expected_items)
            .sum::<usize>(),
        6
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
        .find(|registry| registry.name().to_string() == "minecraft:item")
        .unwrap();
    assert_eq!(
        verify_families(registry).unwrap(),
        FamilyCoverage {
            families: 44,
            slices: 43,
            items: 138,
        }
    );
}

fn id(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

#[test]
fn food_drink_and_stew_profiles_keep_exact_values_and_remainders() {
    assert_eq!(
        (
            profile(ConsumableKind::Bread).nutrition,
            profile(ConsumableKind::Bread).saturation,
        ),
        (5, 6.0)
    );
    assert_eq!(
        (
            profile(ConsumableKind::CookedSalmon).nutrition,
            profile(ConsumableKind::CookedSalmon).saturation,
        ),
        (6, 9.6)
    );
    assert_eq!(
        profile(ConsumableKind::RabbitStew).remainder,
        Remainder::Bowl
    );
    assert_eq!(
        profile(ConsumableKind::HoneyBottle).remainder,
        Remainder::GlassBottle
    );
    assert_eq!(
        profile(ConsumableKind::MilkBucket).remainder,
        Remainder::Bucket
    );
    assert_eq!(profile(ConsumableKind::HoneyBottle).duration_ticks, 40);
    assert!(profile(ConsumableKind::SuspiciousStew).always_edible);
    assert_eq!(suspicious_stew_default_duration(None), 160);
    let stew_effects = [
        SuspiciousStewEffect {
            effect: id("night_vision"),
            duration: 100,
        },
        SuspiciousStewEffect {
            effect: id("poison"),
            duration: 220,
        },
    ];
    assert_eq!(suspicious_stew_effects(&stew_effects), stew_effects);
    assert_eq!(stew_remainder_count(3, 2, false), 1);
    assert_eq!(stew_remainder_count(3, 3, true), 0);
}

#[test]
fn toxic_and_clearing_listeners_preserve_probability_and_effect_order() {
    assert_eq!(
        effects(ConsumableKind::Pufferfish, 0.5, 0),
        [
            ConsumeEffect::Poison {
                duration: 1_200,
                amplifier: 1,
            },
            ConsumeEffect::Hunger {
                duration: 300,
                amplifier: 2,
            },
            ConsumeEffect::Nausea {
                duration: 300,
                amplifier: 0,
            },
        ]
    );
    assert_eq!(effects(ConsumableKind::RottenFlesh, 0.8_f32, 0), []);
    assert_eq!(
        effects(ConsumableKind::HoneyBottle, 0.0, 0),
        [ConsumeEffect::ClearPoison]
    );
    assert_eq!(
        effects(ConsumableKind::MilkBucket, 0.0, 0),
        [ConsumeEffect::ClearAllEffects]
    );
    assert_eq!(
        effects(ConsumableKind::OminousBottle, 0.0, 9),
        [ConsumeEffect::BadOmen {
            duration: 120_000,
            amplifier: 4,
        }]
    );
    assert_eq!(effect_probability_draws(ConsumableKind::Pufferfish), 1);
    assert_eq!(effect_probability_draws(ConsumableKind::SpiderEye), 1);
}

#[test]
fn potion_effects_keep_base_then_custom_order_and_scale_only_finite_duration() {
    let contents = PotionContents {
        base_effects: vec![
            PotionEffect {
                effect: id("speed"),
                duration: 100,
                amplifier: 0,
                instantaneous: false,
            },
            PotionEffect {
                effect: id("healing"),
                duration: 1,
                amplifier: 0,
                instantaneous: true,
            },
        ],
        custom_effects: vec![PotionEffect {
            effect: id("poison"),
            duration: 1,
            amplifier: 2,
            instantaneous: false,
        }],
        duration_scale_bits: 0.125_f32.to_bits(),
    };
    let scaled = scaled_potion_effects(&contents);
    assert_eq!(
        scaled
            .iter()
            .map(|effect| (effect.effect.path(), effect.duration))
            .collect::<Vec<_>>(),
        [("speed", 12), ("healing", 1), ("poison", 1)]
    );
    assert!(water_transaction(true, true));
    assert!(!water_transaction(true, false));
}

#[test]
fn drink_cadence_and_ominous_component_bounds_are_exact() {
    assert!(periodic_drink_sound(32, 24));
    assert!(periodic_drink_sound(40, 28));
    assert!(!periodic_drink_sound(40, 32));
    assert_eq!([-1, 0, 4, 5].map(ominous_level), [0_u8, 0_u8, 4_u8, 4_u8]);
}

fn stack(identity: u64, path: &str, count: i32, maximum: i32, patch: u64) -> ItemStack {
    ItemStack::new(identity, id(path), count, maximum, patch)
}

#[test]
fn bundle_capacity_is_fractional_and_newest_matching_entry_moves_to_front() {
    let mut contents = BundleContents::default();
    let mut stone = BundleEntry::ordinary(stack(1, "stone", 64, 64, 0));
    assert_eq!(contents.insert(&mut stone, true, 10), 64);
    assert!(stone.stack.is_empty());
    assert_eq!(contents.weight(), Some(Fraction::ONE));

    let removed = contents.remove().unwrap();
    assert_eq!(removed.stack.count, 64);
    let mut apples = BundleEntry::ordinary(stack(2, "apple", 8, 64, 7));
    let mut sticks = BundleEntry::ordinary(stack(3, "stick", 8, 64, 0));
    assert_eq!(contents.insert(&mut apples, true, 11), 8);
    assert_eq!(contents.insert(&mut sticks, true, 12), 8);
    assert_eq!(
        contents.entries()[0].stack.item.as_ref().unwrap().path(),
        "stick"
    );
    let mut more_apples = BundleEntry::ordinary(stack(4, "apple", 4, 64, 7));
    assert_eq!(contents.insert(&mut more_apples, true, 13), 4);
    assert_eq!(
        contents.entries()[0].stack.item.as_ref().unwrap().path(),
        "apple"
    );
    assert_eq!(contents.entries()[0].stack.count, 12);
}

#[test]
fn bundle_bees_nested_weight_selection_and_invalid_load_follow_component_rules() {
    let mut contents = BundleContents::default();
    let mut nested = BundleEntry {
        stack: stack(1, "bundle", 1, 1, 0),
        nonempty_bees: false,
        nested_weight: Some(Fraction::ONE),
    };
    assert_eq!(contents.insert(&mut nested, true, 2), 0);
    nested.nested_weight = Some(Fraction::new(15, 16).unwrap());
    assert_eq!(contents.insert(&mut nested, true, 3), 1);

    let mut bees = BundleEntry {
        stack: stack(4, "bee_nest", 1, 64, 0),
        nonempty_bees: true,
        nested_weight: None,
    };
    assert_eq!(contents.insert(&mut bees, true, 5), 0);
    assert!(contents.set_selected(0).is_ok());
    assert_eq!(contents.selected(), 0);
    assert!(contents.set_selected(2).is_ok());
    assert_eq!(contents.selected(), -1);
    assert!(contents.set_selected(-2).is_err());

    let invalid =
        BundleContents::from_persisted(vec![BundleEntry::ordinary(stack(6, "stone", 65, 128, 0))]);
    assert_eq!(invalid.entries().len(), 1);
    let overweight = BundleContents::from_persisted(vec![BundleEntry {
        stack: stack(7, "bee_nest", 2, 64, 0),
        nonempty_bees: true,
        nested_weight: None,
    }]);
    assert_eq!(overweight.entries().len(), 1);
    let invalid = BundleContents::from_persisted(vec![BundleEntry {
        stack: ItemStack {
            identity: 8,
            item: Some(id("stone")),
            count: 1,
            maximum: 0,
            component_fingerprint: 0,
        },
        nonempty_bees: false,
        nested_weight: None,
    }]);
    assert!(invalid.entries().is_empty());
}

#[test]
fn bundle_clicks_preserve_selection_quirks_and_removal_is_whole_entry() {
    let mut contents = BundleContents::default();
    let mut apples = BundleEntry::ordinary(stack(1, "apple", 3, 64, 0));
    contents.insert(&mut apples, true, 2);
    contents.set_selected(0).unwrap();

    let mut denied = BundleEntry::ordinary(stack(3, "stone", 1, 64, 0));
    let outcome = bundle_in_slot_insert(
        &mut contents,
        ClickAction::Primary,
        &mut denied,
        false,
        true,
        4,
    );
    assert_eq!(outcome.sound, Some(BundleSound::InsertFail));
    assert_eq!(contents.selected(), 0);

    let (outcome, removed) =
        bundle_in_slot_remove(&mut contents, ClickAction::Secondary, true, true);
    assert!(outcome.handled);
    assert_eq!(outcome.transferred, 3);
    assert_eq!(removed.unwrap().stack.count, 3);
    assert_eq!(contents.selected(), -1);

    let mut contents =
        BundleContents::from_persisted(vec![BundleEntry::ordinary(stack(5, "apple", 3, 64, 0))]);
    let (partial, _) = bundle_on_cursor_remove(&mut contents, ClickAction::Secondary, 2);
    assert_eq!(partial.transferred, 2);
    assert_eq!(partial.sound, None);
    assert_eq!(contents.entries()[0].stack.count, 1);
}

#[test]
fn bundle_use_schedule_destruction_and_recolor_match_observed_boundaries() {
    assert!(held_output_attempt(200));
    assert!(!held_output_attempt(190));
    assert!(!held_output_attempt(189));
    assert!(held_output_attempt(188));
    assert!(held_output_attempt(2));
    assert!(!held_output_attempt(0));
    assert!(recolor_allowed(false, 2, 1));
    assert!(!recolor_allowed(true, 2, 1));

    let mut contents = BundleContents::from_persisted(vec![
        BundleEntry::ordinary(stack(1, "apple", 2, 64, 0)),
        BundleEntry::ordinary(stack(2, "stick", 1, 64, 0)),
    ]);
    let dropped = contents.destroy();
    assert_eq!(dropped.len(), 2);
    assert!(contents.entries().is_empty());
}

#[test]
fn arrow_selection_consumption_and_pickup_modes_preserve_identity_rules() {
    assert_eq!(
        select_player_ammunition(
            Some(AmmunitionKind::Other),
            Some(AmmunitionKind::SpectralArrow),
            &[AmmunitionKind::Arrow],
            false,
            false,
        ),
        Some(AmmunitionKind::SpectralArrow)
    );
    assert_eq!(
        select_player_ammunition(
            None,
            None,
            &[AmmunitionKind::FireworkRocket, AmmunitionKind::TippedArrow],
            true,
            false,
        ),
        Some(AmmunitionKind::TippedArrow)
    );

    let mut arrows = stack(1, "arrow", 3, 64, 7);
    let used = take_ammunition(&mut arrows, 1, false, false, true, 2);
    assert_eq!((used.source_consumed, arrows.count), (1, 2));
    let copied = take_ammunition(&mut arrows, 0, false, false, true, 3);
    assert!(copied.intangible_projectile);
    assert_eq!(arrows.count, 2);
    assert_eq!(
        pickup_after_owner(PickupMode::Disallowed, ProjectileOwner::Player),
        PickupMode::Allowed
    );
    assert_eq!(
        pickup_after_owner(PickupMode::CreativeOnly, ProjectileOwner::OminousSpawner),
        PickupMode::Disallowed
    );
    assert!(can_pick_up(PickupMode::CreativeOnly, true, 0, true));
    assert!(!can_pick_up(PickupMode::Allowed, true, 1, false));
}

#[test]
fn tipped_arrow_scaling_conversion_and_color_use_carried_potion_state() {
    let copied = imbued_tipped_contents(Some(PotionContents {
        base_effects: vec![PotionEffect {
            effect: id("slowness"),
            duration: 800,
            amplifier: 0,
            instantaneous: false,
        }],
        custom_effects: Vec::new(),
        duration_scale_bits: 1.0_f32.to_bits(),
    }));
    assert_eq!(f32::from_bits(copied.duration_scale_bits), 0.125);
    assert!(converts_to_plain_arrow(false, 600));
    assert!(!converts_to_plain_arrow(true, 600));
    assert_eq!(arrow_color(true, Some(42)), -1);
    assert_eq!(arrow_color(false, None), -13_083_194);
}

#[test]
fn egg_hatch_draws_are_conditional_and_every_hit_discards_with_event_three() {
    assert_eq!(egg_hatch(1, None), EggHatchCount::None);
    assert_eq!(egg_hatch(0, Some(1)), EggHatchCount::One);
    assert_eq!(egg_hatch(0, Some(0)), EggHatchCount::Four);
    assert_eq!(egg_hit(0, Some(0)).requested_chicks, 4);
    assert!(egg_hit(7, None).emitted_entity_event_three);
    assert!(egg_hit(7, None).discarded_projectile);
    assert_eq!(reset_laying_timer(5_999), 11_999);
}

#[test]
fn armor_stand_placement_quantizes_after_configuration_and_damage_is_two_hit() {
    let placed = place_armor_stand(Face::Up, true, true, true, true, 359.0);
    assert_eq!(placed.result, UseResult::Success);
    assert!(placed.configuration_before_final_yaw);
    assert_eq!(placed.yaw, Some(180.0));
    assert_eq!(quantized_stand_yaw(180.0), 0.0);
    assert_eq!(
        place_armor_stand(Face::Down, true, true, true, true, 0.0).consumed,
        0
    );
    assert!(!stand_slot_usable(EquipmentSlot::MainHand, false, false, 0));
    assert!(!stand_slot_usable(
        EquipmentSlot::Head,
        false,
        true,
        1 << (EquipmentSlot::Head as u32 + 16)
    ));
    let selected = select_stand_slot_for_empty_hand(
        1.0,
        false,
        StandOccupancy {
            mainhand: false,
            offhand: false,
            feet: false,
            legs: true,
            chest: true,
            head: false,
        },
        0,
    );
    assert_eq!(selected, EquipmentSlot::Chest);
    let split = swap_stand_slot(3, 0, false, true);
    assert_eq!((split.hand_count, split.slot_count), (2, 1));
    let copied = swap_stand_slot(3, 0, true, true);
    assert!(copied.copied_for_infinite_materials);
    assert_eq!((copied.hand_count, copied.slot_count), (3, 1));
    assert_eq!(
        damage_armor_stand(StandDamageKind::Break, 100, 0, false, true, false, true),
        StandDamageOutcome::FirstHit
    );
    assert_eq!(
        damage_armor_stand(StandDamageKind::Break, 105, 100, false, true, false, true),
        StandDamageOutcome::BrokenWithStandItem
    );
}

#[test]
fn end_crystal_uses_one_by_two_entity_gate_and_notifies_after_optional_explosion() {
    let placed = place_end_crystal(CrystalBase::Obsidian, true, true, true);
    assert_eq!((placed.result, placed.consumed), (UseResult::Success, 1));
    assert!(!placed.show_bottom);
    assert!(placed.respawn_check);
    assert_eq!(
        place_end_crystal(CrystalBase::Other, true, true, true).result,
        UseResult::Fail
    );
    let hit = damage_end_crystal(true, false, false, false, false);
    assert_eq!(hit.explosion_radius, Some(6));
    assert!(hit.fight_notified_after_explosion);
    let explosion_hit = damage_end_crystal(true, false, false, false, true);
    assert_eq!(explosion_hit.explosion_radius, None);
}

#[test]
fn hanging_selection_survival_cadence_and_frame_bounds_are_exact() {
    let candidates = [
        PaintingCandidate {
            area: 1,
            survives: true,
        },
        PaintingCandidate {
            area: 4,
            survives: true,
        },
        PaintingCandidate {
            area: 4,
            survives: true,
        },
    ];
    assert_eq!(maximal_painting_candidate(&candidates, 1), Some(2));
    assert!(!hanging_admitted(HangingKind::Painting, true, Face::Up));
    assert_eq!(
        finalize_hanging_placement(true, false, true).result,
        UseResult::Consume
    );

    let mut cadence = SurvivalCadence::default();
    assert!((0..100).all(|_| !cadence.tick()));
    assert!(cadence.tick());
    assert!(frame_map_admitted(Some(256)));
    assert!(!frame_map_admitted(Some(257)));
    assert_eq!(rotate_frame(7), 0);
    assert_eq!(rotate_frame(-2), -1);
    assert_eq!(frame_analog_value(false, 7), 8);
}

#[test]
fn minecart_placement_and_subtypes_keep_observed_consumption_quirks() {
    let placement = place_minecart(true, RailShape::Ascending, true, true, false, true);
    assert!(placement.success);
    assert_eq!(placement.vertical_offset, 0.5625);
    assert!(placement.spawn_reason_dispenser);
    assert!(!placement.admission_result_observed);
    assert_eq!(
        dispenser_vertical_offset(false, true, true, true, false),
        Some(-0.4)
    );

    let interaction = interact_ordinary_cart(false, true, false, true);
    assert!(interaction.passenger_installed);
    assert!(!interaction.literal_success);
    assert_eq!(interaction.start_riding_calls, 2);
    let invalid_fuel = fuel_furnace_cart(30_000, true);
    assert!(invalid_fuel.action_consumed);
    assert_eq!(invalid_fuel.stack_consumed, 0);
    assert_eq!(tnt_fuse(-1, true), 80);
    assert!(!hopper_enabled(true));
    assert!(command_cart_may_activate(14, 10, true));
    assert_eq!(
        destruction_item(MinecartKind::CommandBlock),
        MinecartKind::Ordinary
    );
    assert!(scatters_contents(MinecartKind::Chest, true));
}

#[test]
fn mob_bucket_capture_and_release_preserve_dry_water_and_creative_asymmetry() {
    let capture = capture_mob(MobBucketKind::SulfurCube, true, EmptyBucketKind::Dry);
    assert!(capture.success);
    assert_eq!(capture.consumed_input, 1);
    assert!(!capture.retained_for_infinite_materials);
    assert_eq!(capture.steps[0], CaptureStep::PickupSound);
    assert_eq!(
        variant_transfer(MobBucketKind::TropicalFish),
        VariantTransfer::TropicalPatternAndColors
    );
    assert_eq!(
        placement_target(true, true),
        PlacementTarget::ClickedLiquidContainer
    );

    let evaporated = release_mob(
        MobBucketKind::Axolotl,
        ReleaseInput {
            permissions_admitted: true,
            fluid_admitted: false,
            evaporating_dimension: true,
            server_side: true,
            factory_created: false,
            admission_accepted: false,
            infinite_materials: true,
            dispenser: false,
        },
    );
    assert!(evaporated.success);
    assert!(!evaporated.water_written);
    assert!(evaporated.entity_place_event);
    assert!(evaporated.placed_block_criterion);
    assert!(evaporated.retains_filled_bucket);

    let dry = release_mob(
        MobBucketKind::SulfurCube,
        ReleaseInput {
            permissions_admitted: true,
            fluid_admitted: false,
            evaporating_dimension: false,
            server_side: true,
            factory_created: true,
            admission_accepted: false,
            infinite_materials: false,
            dispenser: false,
        },
    );
    assert!(dry.success);
    assert!(!dry.mob_admitted);
    assert!(!dry.placed_block_criterion);
}

#[test]
fn nautilus_armor_is_nondamageable_and_mount_order_precedes_admission() {
    let netherite = nautilus_armor_profile(NautilusArmorTier::Netherite);
    assert_eq!((netherite.item_id, netherite.armor), (1_367, 19.0));
    assert!(!netherite.damageable);
    assert!(!netherite.damage_on_hurt);

    let rejected = equip_nautilus_armor(NautilusEquipInput {
        alive: true,
        adult: false,
        tamed: false,
        allowed_by_live_tag: false,
        body_empty: true,
        secondary_use: false,
        server_side: true,
    });
    assert!(rejected.persistence_marked_before_admission);
    assert!(!rejected.equipped);
    let menu = equip_nautilus_armor(NautilusEquipInput {
        alive: true,
        adult: true,
        tamed: true,
        allowed_by_live_tag: true,
        body_empty: true,
        secondary_use: true,
        server_side: true,
    });
    assert!(menu.menu_opened);
    assert!(!menu.equipped);
    assert_eq!(
        first_nautilus_shear_slot(true, true, 0, false, false, false),
        ShearedNautilusSlot::Body
    );
    assert!(zombie_nautilus_sun_protected(true));
}

#[test]
fn spear_profiles_charge_lunge_and_kinetic_thresholds_are_tier_exact() {
    let diamond = spear_profile(SpearTier::Diamond);
    assert_eq!(
        (
            diamond.durability,
            diamond.stab_ticks,
            diamond.held_delay,
            diamond.kinetic_multiplier,
        ),
        (1_561, 21, 10, 1.075)
    );
    assert!(stab_charge_admitted(15, 20.0));
    assert!(!stab_charge_admitted(14, 20.0));

    let contact = kinetic_contact(SpearTier::Iron, 50, 11.0, 4.6, 5.0, true);
    assert!(contact.damage);
    assert!(contact.knockback);
    assert!(contact.dismount);
    assert_eq!(contact.damage_amount, 9.0);
    let failed = kinetic_contact(SpearTier::Iron, 51, 11.0, 4.59, 5.0, true);
    assert!(!failed.damage);
    assert!(!failed.dismount);

    let lunge = lunge(3, false, false, false, true, false, 7);
    assert!(lunge.admitted);
    assert_eq!((lunge.exhaustion, lunge.horizontal_impulse), (12.0, 1.374));
}

#[test]
fn steering_boost_commits_before_damage_and_break_preserves_patch_not_damage() {
    let rejected = use_steering_stick(SteeringKind::Carrot, false, false, false, 0, 0, 7);
    assert!(!rejected.success);
    assert!(rejected.item_used_stat);
    assert_eq!(rejected.boost_total, None);

    let boosted = use_steering_stick(SteeringKind::Carrot, false, true, false, 840, 20, 7);
    assert!(boosted.success);
    assert_eq!(boosted.boost_total, Some(980));
    assert!(!boosted.item_used_stat);
    assert!(boosted.broken_to_fishing_rod);
    assert!(boosted.preserved_component_patch);
    assert_eq!(boosted.replacement_damage, 0);
    assert!((boost_multiplier(50, 100) - 2.15).abs() < f32::EPSILON);
}

#[test]
fn ingredient_material_constants_and_identity_sensitive_joins_are_exact() {
    assert_eq!(armadillo_shed_timer(5_999), 11_999);
    assert_eq!(direct_wolf_armor_repair(64), 8);
    assert_eq!(anvil_wolf_armor_repair(64), 16);
    assert!(bone_tames(0));
    assert!(!bone_tames(1));
    assert_eq!(blaze_rod_fuel_ticks(), 2_400);
    assert_eq!(blaze_powder_brewing_uses(), 20);
    assert_eq!(elytra_repair_per_membrane(432), 108);
    assert_eq!(shulker_shell_drop_chance(8), 1.0);
    assert_eq!(pottery_pattern(1_485), Some("flow"));
    assert_eq!(pottery_pattern(1_499), Some("snort"));
    assert_eq!(smithing_template_rarity(1_463), Some(Rarity::Rare));
    assert_eq!(smithing_template_rarity(1_472), Some(Rarity::Epic));
    assert_eq!(TEMPLATE_DUPLICATION.output_templates, 2);
    assert_eq!(RECOVERY_COMPASS_RECIPE.primary_count, 8);
    assert_eq!(MUSIC_DISC_FIVE_RECIPE.primary_count, 9);
    assert_eq!(CONDUIT_RECIPE.primary_count, 8);
    assert_eq!(
        (BEACON_RECIPE.primary_count, BEACON_RECIPE.other_input_count,),
        (5, 4)
    );
    assert_eq!(wither_nether_star_age(true, true), Some(-6_000));
    assert!(!nether_star_world_item_hurt(true, true, true));
    assert!(nether_star_world_item_hurt(true, true, false));
    assert!(vault_key_matches(
        TrialKeyKind::Normal,
        TrialKeyKind::Normal,
        7,
        7,
        1
    ));
    assert!(!vault_key_matches(
        TrialKeyKind::Normal,
        TrialKeyKind::Normal,
        7,
        8,
        1
    ));
    assert!(advancement_observes_trial_key(
        TrialKeyKind::Ominous,
        TrialKeyKind::Ominous
    ));
}

#[test]
fn dragon_breath_and_water_potion_transactions_preserve_order_and_remainders() {
    let clouds = [
        DragonCloud {
            alive: true,
            owner_is_ender_dragon: true,
            radius: 0.25,
        },
        DragonCloud {
            alive: true,
            owner_is_ender_dragon: true,
            radius: 8.0,
        },
    ];
    assert_eq!(first_dragon_cloud(&clouds), Some(0));
    let filled = fill_dragon_breath(clouds[0].radius, true, true);
    assert_eq!(filled.new_radius, 0.0);
    assert_eq!(filled.consumed_bottle, 0);
    assert!(!filled.add_default_breath);

    let brewed = brew_dragon_breath([
        BrewingBottle::SplashWithHolder(4),
        BrewingBottle::SplashWithoutHolder,
        BrewingBottle::PotionWithHolder(4),
    ]);
    assert_eq!(
        brewed.bottles,
        [
            BrewingBottle::LingeringWithHolder(4),
            BrewingBottle::SplashWithoutHolder,
            BrewingBottle::PotionWithHolder(4),
        ]
    );
    assert_eq!(brewed.ingredient_consumed, 1);

    let poured = pour_water_cauldron(2, true, true);
    assert_eq!(poured.next_level, 3);
    assert_eq!(
        pour_water_cauldron(3, true, true).result,
        CauldronResult::TryWithEmptyHand
    );
    let mud = convert_to_mud(false, true, true, true);
    assert_eq!(mud.splash_random_doubles, 10);
    assert!(mud.writes_mud);
    assert!(!mud.write_result_observed);
    assert!(creative_pour_remainder(false).attempts_bottle_insert);
}

#[test]
fn knowledge_book_consumes_before_atomic_ordered_resolution() {
    let keys = [id("first"), id("missing"), id("after")];
    let failed = use_knowledge_book(&keys, false, false, |key| RecipeResolution {
        key: key.clone(),
        disposition: if key.path() == "missing" {
            RecipeDisposition::Missing
        } else {
            RecipeDisposition::Unlockable
        },
        displays: vec![key.clone()],
    });
    assert_eq!(failed.result, KnowledgeBookResult::Fail);
    assert_eq!(failed.consumed, 1);
    assert_eq!(
        failed
            .queried
            .iter()
            .map(ResourceId::path)
            .collect::<Vec<_>>(),
        ["first", "missing"]
    );
    assert!(failed.newly_known.is_empty());
    assert!(!failed.item_used_stat);

    let duplicate = [id("first"), id("first"), id("special")];
    let success = use_knowledge_book(&duplicate, false, false, |key| RecipeResolution {
        key: key.clone(),
        disposition: if key.path() == "special" {
            RecipeDisposition::Special
        } else {
            RecipeDisposition::Unlockable
        },
        displays: vec![key.clone()],
    });
    assert_eq!(success.result, KnowledgeBookResult::Success);
    assert_eq!(success.newly_known, [id("first")]);
    assert_eq!(success.display_packet, [id("first")]);
    assert!(success.item_used_stat);
}

#[test]
fn ply_005_brewing_ingredients_close_start_direct_and_corruption_edges() {
    assert_eq!(
        mix(BrewingIngredient::BreezeRod, PotionKind::Water),
        Some(PotionKind::Mundane)
    );
    assert_eq!(
        mix(BrewingIngredient::BreezeRod, PotionKind::Awkward),
        Some(PotionKind::WindCharged)
    );
    assert_eq!(
        mix(BrewingIngredient::GoldenCarrot, PotionKind::Water),
        None
    );
    assert_eq!(
        mix(BrewingIngredient::GoldenCarrot, PotionKind::Awkward),
        Some(PotionKind::NightVision)
    );
    assert_eq!(mix(BrewingIngredient::Pufferfish, PotionKind::Water), None);
    assert_eq!(
        mix(
            BrewingIngredient::FermentedSpiderEye,
            PotionKind::LongPoison,
        ),
        Some(PotionKind::Harming)
    );
    assert_eq!(
        mix(
            BrewingIngredient::FermentedSpiderEye,
            PotionKind::StrongPoison,
        ),
        Some(PotionKind::StrongHarming)
    );
    assert_eq!(owned_edge_count(BrewingIngredient::FermentedSpiderEye), 12);
}
