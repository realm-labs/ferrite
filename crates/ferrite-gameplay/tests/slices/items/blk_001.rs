use ferrite_gameplay::item::runtime::catalog::{ItemKind, OWNERS, Rarity, verify_owned_families};
use ferrite_gameplay::item::runtime::consumption::{
    DEFAULT_EAT_TICKS, EffectKind, can_start_consuming, food_profile,
};
use ferrite_gameplay::item::runtime::interaction::{
    ALLAY_DUPLICATION_COOLDOWN_TICKS, APPLE_COMPOST_CHANCE, AllayDuplication, AllayState,
    compost_chance, duplicate_allay, heals_iron_golem, horse_food, starts_zombie_villager_cure,
};
use ferrite_gameplay::item::runtime::materials::{
    ItemRole, Material, RepairTarget, furnace_burn_ticks, furnace_minecart_fuel_ticks, has_role,
    repairs,
};
use ferrite_registry::bundle::ContentBundle;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const EXPECTED_SLICES: [&str; 10] = [
    "ITM-AMETHYST-SHARD-RUNTIME-001",
    "ITM-APPLE-RUNTIME-001",
    "ITM-BRICK-RUNTIME-001",
    "ITM-COAL-RUNTIME-001",
    "ITM-COPPER-MATERIAL-RUNTIME-001",
    "ITM-ENCHANTED-GOLDEN-APPLE-RUNTIME-001",
    "ITM-GOLD-MATERIAL-RUNTIME-001",
    "ITM-GOLDEN-APPLE-RUNTIME-001",
    "ITM-IRON-MATERIAL-RUNTIME-001",
    "ITM-NETHERITE-MATERIAL-RUNTIME-001",
];

#[test]
fn all_blk_001_item_slices_have_closed_identity_ownership() {
    let actual = OWNERS
        .iter()
        .map(|owner| owner.slice)
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, EXPECTED_SLICES.into_iter().collect());
    assert_eq!(
        OWNERS
            .iter()
            .map(|owner| owner.expected_items)
            .sum::<usize>(),
        ItemKind::ALL.len()
    );

    for item in ItemKind::ALL {
        assert_eq!(ItemKind::from_path(item.path()), Some(item));
        assert_eq!(item.maximum_stack(), 64);
        assert!(EXPECTED_SLICES.contains(&item.slice()));
    }
    assert_eq!(
        ItemKind::ALL
            .into_iter()
            .map(|item| (item.path(), item.raw_id()))
            .collect::<Vec<_>>(),
        vec![
            ("amethyst_shard", 930),
            ("apple", 921),
            ("brick", 1054),
            ("coal", 924),
            ("charcoal", 925),
            ("raw_copper", 933),
            ("copper_ingot", 934),
            ("copper_nugget", 1336),
            ("enchanted_golden_apple", 1015),
            ("raw_gold", 935),
            ("gold_ingot", 936),
            ("gold_nugget", 1147),
            ("golden_apple", 1014),
            ("raw_iron", 931),
            ("iron_ingot", 932),
            ("iron_nugget", 1335),
            ("netherite_ingot", 937),
            ("netherite_scrap", 938),
        ]
    );
    assert_eq!(ItemKind::from_path("not_a_vanilla_item"), None);
}

#[test]
fn locally_imported_item_catalog_matches_locked_raw_ids_and_components() {
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
    assert_eq!(registry.entries().len(), 1_537);
    assert_eq!(
        verify_owned_families(registry).unwrap(),
        ferrite_gameplay::item::runtime::catalog::OwnedItemCoverage {
            families: 10,
            items: 18,
        }
    );

    for item in ItemKind::ALL {
        let expected_id = format!("minecraft:{}", item.path());
        let entry = registry
            .entries()
            .find(|entry| entry.persistent_id().to_string() == expected_id)
            .unwrap();
        let components = entry.value()["components"].as_object().unwrap();
        assert_eq!(components["minecraft:max_stack_size"], 64);
        let expected_rarity = match item.rarity() {
            Rarity::Common => "common",
            Rarity::Rare => "rare",
        };
        assert_eq!(components["minecraft:rarity"], expected_rarity);
        assert_eq!(
            components.contains_key("minecraft:enchantment_glint_override"),
            item.forced_glint()
        );
        assert_eq!(
            components.contains_key("minecraft:damage_resistant"),
            item.resists_fire_damage()
        );
        assert_eq!(
            components
                .get("minecraft:provides_trim_material")
                .and_then(|value| value.as_str()),
            item.trim_material()
        );
    }
}

#[test]
fn apple_consumption_profiles_preserve_admission_and_effect_order() {
    assert_eq!(DEFAULT_EAT_TICKS, 32);
    let apple = food_profile(ItemKind::Apple).unwrap();
    assert_eq!((apple.nutrition, apple.saturation), (4, 2.4));
    assert!(!apple.always_edible);
    assert!(apple.effects.is_empty());
    assert!(!can_start_consuming(ItemKind::Apple, 20));
    assert!(can_start_consuming(ItemKind::Apple, 19));

    let golden = food_profile(ItemKind::GoldenApple).unwrap();
    assert!(golden.always_edible);
    assert_eq!((golden.nutrition, golden.saturation), (4, 9.6));
    assert_eq!(
        golden
            .effects
            .iter()
            .map(|effect| (effect.kind, effect.duration_ticks, effect.amplifier))
            .collect::<Vec<_>>(),
        vec![
            (EffectKind::Regeneration, 100, 1),
            (EffectKind::Absorption, 2_400, 0),
        ]
    );

    let enchanted = food_profile(ItemKind::EnchantedGoldenApple).unwrap();
    assert!(can_start_consuming(ItemKind::EnchantedGoldenApple, 20));
    assert_eq!(
        enchanted
            .effects
            .iter()
            .map(|effect| (effect.kind, effect.duration_ticks, effect.amplifier))
            .collect::<Vec<_>>(),
        vec![
            (EffectKind::Regeneration, 400, 1),
            (EffectKind::Resistance, 6_000, 0),
            (EffectKind::FireResistance, 6_000, 0),
            (EffectKind::Absorption, 2_400, 3),
        ]
    );
    assert_eq!(food_profile(ItemKind::Brick), None);
}

#[test]
fn live_tag_roles_are_identity_exact() {
    assert_role(
        ItemRole::BeaconPayment,
        &[
            ItemKind::GoldIngot,
            ItemKind::IronIngot,
            ItemKind::NetheriteIngot,
        ],
    );
    assert_role(ItemRole::Coal, &[ItemKind::Coal, ItemKind::Charcoal]);
    assert_role(ItemRole::DuplicatesAllays, &[ItemKind::AmethystShard]);
    assert_role(
        ItemRole::FurnaceMinecartFuel,
        &[ItemKind::Coal, ItemKind::Charcoal],
    );
    assert_role(
        ItemRole::HorseFood,
        &[
            ItemKind::Apple,
            ItemKind::EnchantedGoldenApple,
            ItemKind::GoldenApple,
        ],
    );
    assert_role(
        ItemRole::HorseTempt,
        &[ItemKind::EnchantedGoldenApple, ItemKind::GoldenApple],
    );
    assert_role(
        ItemRole::MetalNugget,
        &[
            ItemKind::CopperNugget,
            ItemKind::GoldNugget,
            ItemKind::IronNugget,
        ],
    );
    assert_role(
        ItemRole::PiglinLoved,
        &[
            ItemKind::EnchantedGoldenApple,
            ItemKind::RawGold,
            ItemKind::GoldIngot,
            ItemKind::GoldenApple,
        ],
    );
    assert_role(
        ItemRole::ToolMaterial,
        &[
            ItemKind::CopperIngot,
            ItemKind::GoldIngot,
            ItemKind::IronIngot,
            ItemKind::NetheriteIngot,
        ],
    );
    assert_role(
        ItemRole::TrimMaterial,
        &[
            ItemKind::AmethystShard,
            ItemKind::CopperIngot,
            ItemKind::GoldIngot,
            ItemKind::IronIngot,
            ItemKind::NetheriteIngot,
        ],
    );
}

#[test]
fn repair_and_fuel_dispatch_rejects_nearby_materials() {
    for (item, material) in [
        (ItemKind::CopperIngot, Material::Copper),
        (ItemKind::GoldIngot, Material::Gold),
        (ItemKind::IronIngot, Material::Iron),
        (ItemKind::NetheriteIngot, Material::Netherite),
    ] {
        assert!(repairs(item, RepairTarget::Tool(material)));
        assert!(repairs(item, RepairTarget::HumanoidArmor(material)));
        assert!(!repairs(item, RepairTarget::HorseArmor(material)));
        assert!(!repairs(item, RepairTarget::NautilusArmor(material)));
    }
    assert!(repairs(ItemKind::IronIngot, RepairTarget::ChainmailArmor));
    assert!(!repairs(ItemKind::GoldIngot, RepairTarget::ChainmailArmor));
    assert!(!repairs(
        ItemKind::CopperIngot,
        RepairTarget::Tool(Material::Iron)
    ));

    for item in [ItemKind::Coal, ItemKind::Charcoal] {
        assert_eq!(furnace_burn_ticks(item), 1_600);
        assert_eq!(furnace_minecart_fuel_ticks(item), 3_600);
    }
    assert_eq!(furnace_burn_ticks(ItemKind::GoldIngot), 0);
    assert_eq!(furnace_minecart_fuel_ticks(ItemKind::Brick), 0);
}

#[test]
fn item_owned_entity_and_composter_boundaries_are_exact() {
    let ready = AllayState {
        dancing: true,
        duplication_cooldown: 0,
    };
    assert_eq!(
        duplicate_allay(ItemKind::AmethystShard, ready, true),
        AllayDuplication::Spawned {
            parent_cooldown: ALLAY_DUPLICATION_COOLDOWN_TICKS,
            child_cooldown: ALLAY_DUPLICATION_COOLDOWN_TICKS,
        }
    );
    assert_eq!(
        duplicate_allay(ItemKind::AmethystShard, ready, false),
        AllayDuplication::ConsumedWithoutSpawn
    );
    assert_eq!(
        duplicate_allay(
            ItemKind::AmethystShard,
            AllayState {
                dancing: false,
                duplication_cooldown: 0,
            },
            true
        ),
        AllayDuplication::Pass
    );
    assert_eq!(
        duplicate_allay(ItemKind::Apple, ready, true),
        AllayDuplication::Pass
    );

    let apple = horse_food(ItemKind::Apple).unwrap();
    assert_eq!((apple.heal, apple.growth_ticks, apple.temper), (3.0, 60, 3));
    assert!(!apple.induces_love);
    let golden = horse_food(ItemKind::GoldenApple).unwrap();
    assert_eq!(
        (golden.heal, golden.growth_ticks, golden.temper),
        (10.0, 240, 10)
    );
    assert!(golden.induces_love);

    assert!(starts_zombie_villager_cure(ItemKind::GoldenApple, true));
    assert!(!starts_zombie_villager_cure(
        ItemKind::EnchantedGoldenApple,
        true
    ));
    assert!(!starts_zombie_villager_cure(ItemKind::GoldenApple, false));
    assert!(heals_iron_golem(ItemKind::IronIngot, 75.0, 100.0));
    assert!(!heals_iron_golem(ItemKind::IronIngot, 100.0, 100.0));
    assert!(!heals_iron_golem(ItemKind::GoldIngot, 75.0, 100.0));
    assert_eq!(compost_chance(ItemKind::Apple), Some(APPLE_COMPOST_CHANCE));
    assert_eq!(compost_chance(ItemKind::GoldenApple), None);
}

fn assert_role(role: ItemRole, expected: &[ItemKind]) {
    let actual = ItemKind::ALL
        .into_iter()
        .filter(|item| has_role(*item, role))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
