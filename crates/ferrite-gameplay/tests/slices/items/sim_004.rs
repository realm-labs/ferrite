use ferrite_gameplay::item::runtime::sim_004::brewing::{
    MaterialBrewingIngredient, PotionContainer, PotionKind, gunpowder_container_mix,
    owned_edge_count, potion_mix,
};
use ferrite_gameplay::item::runtime::sim_004::catalog::{
    MaterialCatalogCoverage, MaterialItem, verify_families,
};
use ferrite_gameplay::item::runtime::sim_004::dried_kelp::{
    AirUse, BLOCK_BURN_ODDS, BLOCK_FUEL_TICKS, BLOCK_IGNITE_ODDS, BUTCHER_TRADE, CONSUME_TICKS,
    COOKING_RECORDS, DRIED_KELP_BLOCK_ID, DRIED_KELP_BLOCK_ITEM_ID, DRIED_KELP_BLOCK_STATE_ID,
    DRIED_KELP_ITEM_ID, NUTRITION, SATURATION, air_use, composter_probability, composter_succeeds,
};
use ferrite_gameplay::item::runtime::sim_004::firework::{
    DEFAULT_STAR_TINT, ExplosionShape, FireworkExplosion, FireworkRecipeError, FireworkStar,
    Ingredient, craft_base_star, craft_faded_star, craft_rockets, rocket_damage, rocket_lifetime,
    star_tint,
};
use ferrite_gameplay::item::runtime::sim_004::joins::{
    ARMOR_TRIM_MODEL_COUNT, ARMOR_TRIM_RECIPE_COUNT, FLINT_TRADES, STRUCTURE_TEMPLATE_COUNT,
    TURTLE_SCUTE_TRADES, profile,
};
use ferrite_gameplay::item::runtime::sim_004::loot::{
    BirdKind, DeadBushDrop, GlowstoneDrop, GravelDrop, LeatherSource, LootInputError, OreBreak,
    OreBreakInput, OreOutput, RedstoneOreContact, ToolTier, bird_feather_drop, break_ore,
    contact_redstone_ore, dead_bush_drop, glowstone_drop, gravel_drop, leaf_stick_drop,
    leather_drop, ore_profile, panda_sneeze_emits_slime_ball, redstone_random_tick,
    slime_ball_drop,
};
use ferrite_gameplay::item::runtime::sim_004::materials::{
    MaterialRole, RepairTarget, SlimeBallFoodTarget, cat_feather_probability,
    fishing_junk_denominator, furnace_burn_ticks, has_default_role, lapis_enchantment_consumption,
    leather_pickup_admitted_by_age, repairs, slime_ball_feeding_admitted, trim_profile,
};
use ferrite_gameplay::item::runtime::sim_004::turtle::{
    AdulthoodInput, BABY_START_AGE, HELMET_ARMOR, HELMET_ENCHANTABILITY, HELMET_MAXIMUM_DAMAGE,
    WATER_BREATHING_TICKS, adulthood, helmet_refresh, repair_per_scute, scutes_to_repair,
    seagrass_acceleration,
};
use ferrite_registry::bundle::ContentBundle;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn sim_004_catalog_closes_fifteen_single_identity_families() {
    assert_eq!(MaterialItem::ALL.len(), 15);
    assert_eq!(
        MaterialItem::ALL
            .into_iter()
            .map(MaterialItem::slice)
            .collect::<BTreeSet<_>>()
            .len(),
        15
    );
    assert_eq!(
        MaterialItem::ALL.map(|item| (item.path(), item.raw_id())),
        [
            ("diamond", 926),
            ("dried_kelp", 1_136),
            ("emerald", 927),
            ("feather", 977),
            ("firework_star", 1_273),
            ("flint", 1_010),
            ("glowstone_dust", 1_085),
            ("gunpowder", 978),
            ("lapis_lazuli", 928),
            ("leather", 1_045),
            ("quartz", 929),
            ("redstone", 745),
            ("slime_ball", 1_059),
            ("stick", 974),
            ("turtle_scute", 916),
        ]
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
        MaterialCatalogCoverage {
            families: 15,
            slices: 15,
            items: 15,
        }
    );
}

#[test]
fn source_and_sink_cardinalities_remain_closed_for_every_slice() {
    assert_eq!(
        (STRUCTURE_TEMPLATE_COUNT, ARMOR_TRIM_RECIPE_COUNT),
        (1_212, 18)
    );
    assert_eq!(ARMOR_TRIM_MODEL_COUNT, 29);
    assert_eq!(
        MaterialItem::ALL.map(|item| {
            let joins = profile(item);
            (
                joins.recipes,
                joins.advancements,
                joins.direct_unlocks,
                joins.non_block_acquisition_tables,
            )
        }),
        [
            (56, 55, 12, 17),
            (5, 5, 5, 0),
            (24, 24, 1, 32),
            (4, 3, 1, 6),
            (3, 0, 0, 0),
            (3, 3, 3, 3),
            (3, 2, 2, 2),
            (5, 2, 2, 8),
            (25, 25, 2, 4),
            (26, 26, 7, 19),
            (26, 25, 7, 2),
            (46, 46, 8, 8),
            (4, 4, 2, 2),
            (111, 111, 8, 20),
            (1, 1, 1, 1),
        ]
    );
    assert_eq!(profile(MaterialItem::Emerald).reloadable_trade_records, 469);
    assert_eq!(profile(MaterialItem::Emerald).stored_template_offers, 2);
    assert_eq!(profile(MaterialItem::Emerald).exact_template_matches, 2);
    for item in MaterialItem::ALL {
        let joins = profile(item);
        assert_eq!(joins.trim_recipes > 0, trim_profile(item).is_some());
    }
}

fn ore_input(material: MaterialItem) -> OreBreakInput {
    OreBreakInput {
        material,
        variant_index: 0,
        correct_tool: true,
        silk_touch: false,
        base_count: 1,
        fortune_level: 0,
        fortune_draw: 0,
        explosion_survivors: None,
        experience_draw: ore_profile(material).unwrap().experience_minimum,
    }
}

#[test]
fn ore_profiles_and_fortune_algorithms_preserve_material_asymmetry() {
    let diamond = ore_profile(MaterialItem::Diamond).unwrap();
    assert_eq!(diamond.minimum_tier, ToolTier::Iron);
    assert_eq!(
        diamond
            .variants
            .iter()
            .map(|ore| (ore.block_id, ore.item_id, ore.first_state_id))
            .collect::<Vec<_>>(),
        [(203, 105, 5_307), (204, 106, 5_308)]
    );
    assert_eq!(
        ore_profile(MaterialItem::Emerald).unwrap().variants[1].last_state_id,
        9_574
    );
    assert_eq!(
        ore_profile(MaterialItem::LapisLazuli).unwrap().minimum_tier,
        ToolTier::Stone
    );
    assert_eq!(
        ore_profile(MaterialItem::Quartz).unwrap().minimum_tier,
        ToolTier::None
    );

    let mut diamond_input = ore_input(MaterialItem::Diamond);
    diamond_input.fortune_level = 3;
    diamond_input.fortune_draw = 4;
    assert_eq!(
        break_ore(diamond_input).unwrap().output,
        OreOutput::Material {
            item: MaterialItem::Diamond,
            count: 4,
        }
    );

    let mut lapis = ore_input(MaterialItem::LapisLazuli);
    lapis.base_count = 9;
    lapis.fortune_level = 2;
    lapis.fortune_draw = 3;
    lapis.experience_draw = 5;
    assert_eq!(
        break_ore(lapis).unwrap(),
        OreBreak {
            output: OreOutput::Material {
                item: MaterialItem::LapisLazuli,
                count: 27,
            },
            experience: 5,
        }
    );

    let mut redstone = ore_input(MaterialItem::Redstone);
    redstone.base_count = 5;
    redstone.fortune_level = 3;
    redstone.fortune_draw = 3;
    assert_eq!(
        break_ore(redstone).unwrap().output,
        OreOutput::Material {
            item: MaterialItem::Redstone,
            count: 8,
        }
    );
}

#[test]
fn ore_rejection_silk_explosion_and_redstone_contact_order_are_exact() {
    let mut silk = ore_input(MaterialItem::Diamond);
    silk.variant_index = 1;
    silk.silk_touch = true;
    silk.explosion_survivors = Some(0);
    assert_eq!(
        break_ore(silk).unwrap(),
        OreBreak {
            output: OreOutput::OreBlock { item_id: 106 },
            experience: 0,
        }
    );
    let mut wrong_tool = ore_input(MaterialItem::Emerald);
    wrong_tool.correct_tool = false;
    assert_eq!(break_ore(wrong_tool).unwrap().output, OreOutput::None);
    let mut invalid = ore_input(MaterialItem::Redstone);
    invalid.base_count = 4;
    invalid.fortune_draw = 1;
    assert_eq!(break_ore(invalid), Err(LootInputError::FortuneDraw(1)));

    assert_eq!(
        contact_redstone_ore(true, false),
        RedstoneOreContact {
            spawn_exposed_face_particles: true,
            write_lit_state: true,
            write_flags: 3,
        }
    );
    assert!(!contact_redstone_ore(false, false).write_lit_state);
    assert!(!contact_redstone_ore(true, true).write_lit_state);
    assert!(!redstone_random_tick(true));
}

#[test]
fn gravel_and_glowstone_keep_distinct_silk_fortune_and_explosion_order() {
    assert_eq!(
        gravel_drop(true, false, 0, 0.99).unwrap(),
        GravelDrop::Gravel
    );
    assert_eq!(
        gravel_drop(false, false, 3, 0.0).unwrap(),
        GravelDrop::Nothing
    );
    assert_eq!(
        gravel_drop(false, true, 0, 0.1).unwrap(),
        GravelDrop::Gravel
    );
    assert_eq!(
        gravel_drop(false, true, 3, 0.999).unwrap(),
        GravelDrop::Flint
    );

    assert_eq!(
        glowstone_drop(true, 0, 0, 0, Some(0)).unwrap(),
        GlowstoneDrop::Glowstone
    );
    assert_eq!(
        glowstone_drop(false, 4, 3, 3, None).unwrap(),
        GlowstoneDrop::Dust(4)
    );
    assert_eq!(
        glowstone_drop(false, 2, 3, 3, Some(2)).unwrap(),
        GlowstoneDrop::Dust(2)
    );
}

#[test]
fn animal_material_loot_preserves_age_source_and_looting_branches() {
    assert_eq!(
        bird_feather_drop(BirdKind::Chicken, false, 0, true, 3, 0.5).unwrap(),
        0
    );
    assert_eq!(
        bird_feather_drop(BirdKind::Chicken, true, 0, true, 3, 0.5).unwrap(),
        2
    );
    assert_eq!(
        bird_feather_drop(BirdKind::Parrot, true, 1, false, 3, f32::NAN).unwrap(),
        1
    );
    assert_eq!(
        leather_drop(LeatherSource::Cow, 2, true, 2, 0.75).unwrap(),
        4
    );
    assert_eq!(
        leather_drop(LeatherSource::Hoglin, 2, false, 0, 0.0),
        Err(LootInputError::BaseCount(2))
    );
    assert_eq!(slime_ball_drop(2, true, 0, true, 3, 0.9).unwrap(), 0);
    assert_eq!(slime_ball_drop(1, true, 9, true, 9, f32::NAN).unwrap(), 1);
    assert_eq!(slime_ball_drop(1, false, 0, true, 2, 0.75).unwrap(), 2);
    assert_eq!(panda_sneeze_emits_slime_ball(0), Some(true));
    assert_eq!(panda_sneeze_emits_slime_ball(699), Some(false));
    assert_eq!(panda_sneeze_emits_slime_ball(700), None);
}

#[test]
fn foliage_dead_bush_and_fishing_boundaries_are_exact() {
    assert_eq!(
        leaf_stick_drop(true, false, 4, f32::NAN, 9, None).unwrap(),
        0
    );
    assert_eq!(leaf_stick_drop(false, false, 0, 0.02, 2, None).unwrap(), 0);
    assert_eq!(
        leaf_stick_drop(false, false, 4, 0.099, 2, Some(1)).unwrap(),
        1
    );
    assert_eq!(
        dead_bush_drop(true, 9, Some(9)).unwrap(),
        DeadBushDrop::DeadBush
    );
    assert_eq!(
        dead_bush_drop(false, 2, Some(0)).unwrap(),
        DeadBushDrop::Sticks(0)
    );
    assert_eq!(fishing_junk_denominator(false), 100);
    assert_eq!(fishing_junk_denominator(true), 110);
    assert_eq!(cat_feather_probability(0.7).unwrap(), 7.0 / 62.0);
}

#[test]
fn material_tags_trim_repair_enchanting_and_fuel_are_identity_exact() {
    assert!(has_default_role(
        MaterialItem::Diamond,
        MaterialRole::BeaconPayment
    ));
    assert!(has_default_role(
        MaterialItem::Emerald,
        MaterialRole::TrimMaterial
    ));
    assert_eq!(
        trim_profile(MaterialItem::Diamond).unwrap().rgb,
        0x006e_ecd2
    );
    assert_eq!(
        trim_profile(MaterialItem::Diamond)
            .unwrap()
            .self_equipment_asset,
        Some("diamond_darker")
    );
    assert_eq!(trim_profile(MaterialItem::Quartz).unwrap().rgb, 0x00e3_d4c4);
    assert!(!repairs(
        MaterialItem::Diamond,
        RepairTarget::DiamondHorseArmor,
        true
    ));
    assert!(repairs(
        MaterialItem::Leather,
        RepairTarget::LeatherHumanoidArmor,
        true
    ));
    assert!(!repairs(
        MaterialItem::Leather,
        RepairTarget::LeatherHumanoidArmor,
        false
    ));
    assert_eq!(
        lapis_enchantment_consumption(2, false, 3, true, true, true),
        Some(3)
    );
    assert_eq!(
        lapis_enchantment_consumption(2, true, 0, true, true, false),
        Some(0)
    );
    assert_eq!(
        lapis_enchantment_consumption(1, false, 1, true, true, true),
        None
    );
    assert_eq!(furnace_burn_ticks(MaterialItem::Stick), 100);
    assert_eq!(furnace_burn_ticks(MaterialItem::Gunpowder), 0);
}

#[test]
fn leather_and_slime_food_age_gates_use_live_tags() {
    assert!(!leather_pickup_admitted_by_age(true, true));
    assert!(leather_pickup_admitted_by_age(false, true));
    assert!(slime_ball_feeding_admitted(SlimeBallFoodTarget::Frog, true));
    assert!(slime_ball_feeding_admitted(
        SlimeBallFoodTarget::SulfurCube { baby: true },
        true
    ));
    assert!(!slime_ball_feeding_admitted(
        SlimeBallFoodTarget::SulfurCube { baby: false },
        true
    ));
    assert!(!slime_ball_feeding_admitted(
        SlimeBallFoodTarget::Frog,
        false
    ));
}

#[test]
fn dried_kelp_keeps_fast_food_composter_and_4001_tick_fuel_asymmetry() {
    assert_eq!(
        (
            DRIED_KELP_ITEM_ID,
            DRIED_KELP_BLOCK_ID,
            DRIED_KELP_BLOCK_ITEM_ID,
            DRIED_KELP_BLOCK_STATE_ID,
        ),
        (1_136, 744, 1_056, 15_089)
    );
    assert_eq!((NUTRITION, SATURATION, CONSUME_TICKS), (1, 0.6, 16));
    assert_eq!(air_use(true, true, true), AirUse::Fail);
    assert_eq!(air_use(false, true, true), AirUse::BeginConsumption);
    assert_eq!(air_use(true, false, false), AirUse::Pass);
    assert_eq!(composter_probability(false), 0.3);
    assert_eq!(composter_probability(true), 0.5);
    assert_eq!(composter_succeeds(0, false, f64::NAN), Some(true));
    assert_eq!(composter_succeeds(3, false, 0.3), Some(false));
    assert_eq!(composter_succeeds(3, true, 0.3), Some(true));
    assert_eq!(composter_succeeds(7, true, f64::NAN), Some(false));
    assert_eq!(
        (BLOCK_FUEL_TICKS, BLOCK_IGNITE_ODDS, BLOCK_BURN_ODDS),
        (4_001, 30, 60)
    );
    assert_eq!(
        COOKING_RECORDS.map(|record| (record.id, record.ticks, record.experience)),
        [
            ("dried_kelp_from_smelting", 200, 0.1),
            ("dried_kelp_from_smoking", 100, 0.1),
            ("dried_kelp_from_campfire_cooking", 600, 0.1),
        ]
    );
    assert_eq!(
        (
            BUTCHER_TRADE.block_cost,
            BUTCHER_TRADE.maximum_uses,
            BUTCHER_TRADE.experience,
        ),
        (10, 12, 30)
    );
}

fn dye(color: u32) -> Ingredient {
    Ingredient::Dye {
        firework_color: color,
    }
}

#[test]
fn base_firework_star_classifies_exact_modifiers_and_preserves_color_order() {
    let star = craft_base_star(&[
        dye(0x11_22_33),
        Ingredient::Gunpowder,
        Ingredient::Feather,
        dye(0x44_55_66),
        Ingredient::Diamond,
        Ingredient::GlowstoneDust,
    ])
    .unwrap();
    let explosion = star.explosion.unwrap();
    assert_eq!(explosion.shape, ExplosionShape::Burst);
    assert_eq!(explosion.primary_colors, [0x11_22_33, 0x44_55_66]);
    assert!(explosion.has_trail);
    assert!(explosion.has_twinkle);
    assert_eq!(ExplosionShape::Creeper.stream_id(), 3);
    assert_eq!(
        ExplosionShape::from_stream_id(4),
        Some(ExplosionShape::Burst)
    );
    assert_eq!(ExplosionShape::from_stream_id(5), None);
    assert_eq!(
        craft_base_star(&[Ingredient::Gunpowder, dye(0), Ingredient::Gunpowder]),
        Err(FireworkRecipeError::DuplicateFuel)
    );
    assert_eq!(
        craft_base_star(&[
            Ingredient::Gunpowder,
            dye(0),
            Ingredient::Feather,
            Ingredient::GoldNugget,
        ]),
        Err(FireworkRecipeError::DuplicateShape)
    );
    assert_eq!(
        craft_base_star(&[Ingredient::Gunpowder, Ingredient::TaggedDyeWithoutComponent]),
        Err(FireworkRecipeError::DyeWithoutComponent)
    );
}

#[test]
fn fade_preserves_patches_replaces_colors_and_synthesizes_default_explosion() {
    let original = FireworkStar {
        explosion: Some(FireworkExplosion {
            shape: ExplosionShape::Star,
            primary_colors: vec![0x01_02_03],
            fade_colors: vec![0xaa_bb_cc],
            has_trail: true,
            has_twinkle: false,
        }),
        unrelated_patch_fingerprint: 99,
    };
    let faded = craft_faded_star(&[
        dye(0x10_20_30),
        Ingredient::FireworkStar(original),
        dye(0x40_50_60),
    ])
    .unwrap();
    assert_eq!(faded.unrelated_patch_fingerprint, 99);
    let explosion = faded.explosion.unwrap();
    assert_eq!(explosion.shape, ExplosionShape::Star);
    assert_eq!(explosion.primary_colors, [0x01_02_03]);
    assert_eq!(explosion.fade_colors, [0x10_20_30, 0x40_50_60]);
    assert!(explosion.has_trail);

    let synthesized = craft_faded_star(&[
        Ingredient::FireworkStar(FireworkStar::componentless()),
        dye(0xff_00_00),
    ])
    .unwrap();
    assert_eq!(
        synthesized.explosion.unwrap(),
        FireworkExplosion {
            shape: ExplosionShape::SmallBall,
            primary_colors: vec![],
            fade_colors: vec![0xff_00_00],
            has_trail: false,
            has_twinkle: false,
        }
    );
}

#[test]
fn rocket_crafting_copies_only_present_explosions_in_row_major_order() {
    let first = FireworkStar {
        explosion: Some(FireworkExplosion {
            primary_colors: vec![1],
            ..FireworkExplosion::default()
        }),
        unrelated_patch_fingerprint: 1,
    };
    let second = FireworkStar {
        explosion: Some(FireworkExplosion {
            primary_colors: vec![2],
            ..FireworkExplosion::default()
        }),
        unrelated_patch_fingerprint: 2,
    };
    let rockets = craft_rockets(&[
        Ingredient::Paper,
        Ingredient::FireworkStar(first),
        Ingredient::Gunpowder,
        Ingredient::FireworkStar(FireworkStar::componentless()),
        Ingredient::Gunpowder,
        Ingredient::FireworkStar(second),
    ])
    .unwrap();
    assert_eq!((rockets.count, rockets.flight_duration), (3, 2));
    assert_eq!(
        rockets
            .explosions
            .iter()
            .map(|explosion| explosion.primary_colors[0])
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(rocket_damage(rockets.explosions.len()), Some(9));
    assert_eq!(rocket_damage(0), None);
    assert_eq!(rocket_lifetime(2, 5, 6), Some(41));
    assert_eq!(rocket_lifetime(4, 0, 0), None);
    assert_eq!(
        craft_rockets(&[
            Ingredient::Paper,
            Ingredient::Gunpowder,
            Ingredient::Gunpowder,
            Ingredient::Gunpowder,
            Ingredient::Gunpowder,
        ]),
        Err(FireworkRecipeError::TooMuchFuel)
    );
}

#[test]
fn firework_star_tint_uses_only_primary_colors_and_integer_average() {
    assert_eq!(star_tint(&FireworkStar::componentless()), DEFAULT_STAR_TINT);
    let fade_only = FireworkStar {
        explosion: Some(FireworkExplosion {
            fade_colors: vec![0xff_00_00],
            ..FireworkExplosion::default()
        }),
        unrelated_patch_fingerprint: 0,
    };
    assert_eq!(star_tint(&fade_only), DEFAULT_STAR_TINT);
    let colored = FireworkStar {
        explosion: Some(FireworkExplosion {
            primary_colors: vec![0xff_00_00, 0x00_00_ff],
            ..FireworkExplosion::default()
        }),
        unrelated_patch_fingerprint: 0,
    };
    assert_eq!(star_tint(&colored), 0xff7f_007f);
}

#[test]
fn material_brewing_edges_are_closed_and_container_conversion_is_one_way() {
    assert_eq!(
        (
            owned_edge_count(MaterialBrewingIngredient::GlowstoneDust),
            owned_edge_count(MaterialBrewingIngredient::RedstoneDust),
            owned_edge_count(MaterialBrewingIngredient::Gunpowder),
            owned_edge_count(MaterialBrewingIngredient::TurtleHelmet),
        ),
        (10, 14, 1, 1)
    );
    assert_eq!(
        potion_mix(MaterialBrewingIngredient::GlowstoneDust, PotionKind::Water),
        Some(PotionKind::Thick)
    );
    assert_eq!(
        potion_mix(
            MaterialBrewingIngredient::GlowstoneDust,
            PotionKind::Strength
        ),
        Some(PotionKind::StrongStrength)
    );
    assert_eq!(
        potion_mix(
            MaterialBrewingIngredient::GlowstoneDust,
            PotionKind::LongStrength
        ),
        None
    );
    assert_eq!(
        potion_mix(
            MaterialBrewingIngredient::RedstoneDust,
            PotionKind::TurtleMaster
        ),
        Some(PotionKind::LongTurtleMaster)
    );
    assert_eq!(
        potion_mix(MaterialBrewingIngredient::TurtleHelmet, PotionKind::Awkward),
        Some(PotionKind::TurtleMaster)
    );
    assert_eq!(
        potion_mix(MaterialBrewingIngredient::TurtleHelmet, PotionKind::Water),
        None
    );
    assert_eq!(
        gunpowder_container_mix(PotionContainer::Potion),
        Some(PotionContainer::SplashPotion)
    );
    assert_eq!(gunpowder_container_mix(PotionContainer::SplashPotion), None);
}

#[test]
fn turtle_adulthood_is_one_shot_and_helmet_boundaries_are_exact() {
    assert_eq!(BABY_START_AGE, -24_000);
    let emitted = adulthood(AdulthoodInput {
        old_age: -1,
        new_age: 0,
        server_side: true,
        mob_drops_enabled: true,
        growth_table_emits: true,
    });
    assert!(emitted.crossed_to_adult);
    assert!(emitted.attempted_growth_table);
    assert_eq!(emitted.scutes_emitted, 1);
    assert_eq!(
        adulthood(AdulthoodInput {
            old_age: -1,
            new_age: 0,
            server_side: true,
            mob_drops_enabled: false,
            growth_table_emits: true,
        })
        .scutes_emitted,
        0
    );
    assert!(
        !adulthood(AdulthoodInput {
            old_age: 0,
            new_age: 1,
            server_side: true,
            mob_drops_enabled: true,
            growth_table_emits: true,
        })
        .attempted_growth_table
    );
    assert_eq!(seagrass_acceleration(199), 0);
    assert_eq!(seagrass_acceleration(200), 20);
    assert_eq!(
        (
            HELMET_MAXIMUM_DAMAGE,
            HELMET_ARMOR,
            HELMET_ENCHANTABILITY,
            repair_per_scute(HELMET_MAXIMUM_DAMAGE),
            scutes_to_repair(275, 275),
        ),
        (275, 2, 9, 68, Some(5))
    );
    let refresh = helmet_refresh(true, false).unwrap();
    assert_eq!(refresh.duration_ticks, WATER_BREATHING_TICKS);
    assert!(!refresh.show_particles);
    assert!(refresh.show_icon);
    assert_eq!(helmet_refresh(true, true), None);
    assert_eq!(helmet_refresh(false, false), None);
}

#[test]
fn flint_and_turtle_trade_sets_preserve_selection_probabilities() {
    assert_eq!(
        FLINT_TRADES.map(|trade| (
            trade.profession,
            trade.level,
            trade.first_cost,
            trade.second_cost,
            trade.inclusion_probability,
        )),
        [
            ("fletcher", 1, 10, 1, 2.0 / 3.0),
            ("fletcher", 2, 26, 0, 1.0),
            ("leatherworker", 2, 26, 0, 2.0 / 3.0),
            ("toolsmith", 3, 30, 0, 2.0 / 5.0),
            ("weaponsmith", 3, 24, 0, 1.0),
        ]
    );
    assert_eq!(
        TURTLE_SCUTE_TRADES.map(|trade| (
            trade.profession,
            trade.first_cost,
            trade.inclusion_probability,
        )),
        [("leatherworker", 4, 1.0), ("cleric", 4, 2.0 / 3.0),]
    );
}
