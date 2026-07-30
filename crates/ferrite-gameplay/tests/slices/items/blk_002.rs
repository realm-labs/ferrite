use ferrite_gameplay::item::runtime::catalog::{
    ItemKind, OwnedItemCoverage, PRISMARINE_OWNER, Rarity, verify_prismarine_family,
};
use ferrite_gameplay::item::runtime::prismarine::{
    BURIED_TREASURE, CraftingKind, GuardianKind, InclusiveCount, RECIPES, SeaLanternDrop,
    guardian_loot, sea_lantern_drop,
};
use ferrite_registry::bundle::ContentBundle;
use std::fs;
use std::path::Path;

#[test]
fn prismarine_identity_and_imported_family_are_closed() {
    assert_eq!(PRISMARINE_OWNER.len(), 1);
    assert_eq!(
        PRISMARINE_OWNER[0].slice,
        "ITM-PRISMARINE-MATERIAL-RUNTIME-001"
    );
    assert_eq!(
        ItemKind::PRISMARINE.map(|item| (
            item.path(),
            item.raw_id(),
            item.maximum_stack(),
            item.rarity()
        )),
        [
            ("prismarine_shard", 1_277, 64, Rarity::Common),
            ("prismarine_crystals", 1_278, 64, Rarity::Common),
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
        verify_prismarine_family(registry).unwrap(),
        OwnedItemCoverage {
            families: 1,
            items: 2,
        }
    );
    for item in ItemKind::PRISMARINE {
        let expected_id = format!("minecraft:{}", item.path());
        let entry = registry
            .entries()
            .find(|entry| entry.persistent_id().to_string() == expected_id)
            .unwrap();
        let components = entry.value()["components"].as_object().unwrap();
        assert_eq!(components["minecraft:max_stack_size"], 64);
        assert_eq!(components["minecraft:rarity"], "common");
        assert!(!components.contains_key("minecraft:food"));
        assert!(!components.contains_key("minecraft:consumable"));
    }
}

#[test]
fn guardian_profiles_preserve_looting_and_weight_asymmetry() {
    let guardian = guardian_loot(GuardianKind::Guardian);
    let elder = guardian_loot(GuardianKind::ElderGuardian);
    for profile in [guardian, elder] {
        assert_eq!(profile.shard_base, InclusiveCount::new(0, 2));
        assert_eq!(
            profile.shard_bonus_per_looting_level,
            InclusiveCount::new(0, 1)
        );
        assert_eq!(
            profile.secondary_bonus_per_looting_level,
            InclusiveCount::new(0, 1)
        );
        assert_eq!(profile.secondary_crystal_weight, 2);
    }
    assert_eq!(guardian.secondary_total_weight, 5);
    assert_eq!(elder.secondary_total_weight, 6);
}

#[test]
fn buried_treasure_and_sea_lantern_profiles_are_exact() {
    assert_eq!(BURIED_TREASURE.pool_rolls, InclusiveCount::new(1, 3));
    assert_eq!(
        (BURIED_TREASURE.crystal_weight, BURIED_TREASURE.total_weight),
        (5, 15)
    );
    assert_eq!(BURIED_TREASURE.crystal_count, InclusiveCount::new(1, 5));
    assert_eq!(sea_lantern_drop(1), SeaLanternDrop::SeaLantern);
    assert_eq!(sea_lantern_drop(u8::MAX), SeaLanternDrop::SeaLantern);
    let SeaLanternDrop::Crystals(profile) = sea_lantern_drop(0) else {
        panic!("no-Silk branch must emit crystals");
    };
    assert_eq!(profile.base, InclusiveCount::new(2, 3));
    assert_eq!(profile.fortune_bonus_per_level, InclusiveCount::new(0, 1));
    assert_eq!(profile.capped, InclusiveCount::new(1, 5));
    assert!(profile.explosion_decay);
}

#[test]
fn four_building_recipes_keep_shapes_counts_and_asymmetric_unlocks() {
    assert_eq!(RECIPES.len(), 4);
    assert_eq!(
        RECIPES.map(|recipe| (
            recipe.output,
            recipe.kind,
            recipe.shards,
            recipe.crystals,
            recipe.black_dye,
            recipe.unlock_item,
        )),
        [
            (
                "prismarine",
                CraftingKind::Shaped,
                4,
                0,
                0,
                ItemKind::PrismarineShard,
            ),
            (
                "prismarine_bricks",
                CraftingKind::Shapeless,
                9,
                0,
                0,
                ItemKind::PrismarineShard,
            ),
            (
                "dark_prismarine",
                CraftingKind::Shaped,
                8,
                0,
                1,
                ItemKind::PrismarineShard,
            ),
            (
                "sea_lantern",
                CraftingKind::Shaped,
                4,
                5,
                0,
                ItemKind::PrismarineCrystals,
            ),
        ]
    );
    assert_eq!(RECIPES[0].pattern, ["##", "##"]);
    assert!(RECIPES[1].pattern.is_empty());
    assert_eq!(RECIPES[2].pattern, ["SSS", "SIS", "SSS"]);
    assert_eq!(RECIPES[3].pattern, ["SCS", "CCC", "SCS"]);
}
