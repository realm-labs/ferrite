use ferrite_gameplay::item::runtime::books::{
    BookRole, BookSound, bookshelf_sound, has_book_role, signed_generation,
};
use ferrite_gameplay::item::runtime::catalog::{
    BLK_003_OWNERS, ItemKind, OwnedItemCoverage, Rarity, verify_blk_003_families,
};
use ferrite_gameplay::item::runtime::consumption::{EffectKind, food_profile};
use ferrite_gameplay::item::runtime::food_family::{
    COOKING, compost_chance, is_piglin_food, parrot_poison, wolf_healing,
};
use ferrite_registry::bundle::ContentBundle;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const EXPECTED_SLICES: [&str; 9] = [
    "ITM-BAKED-POTATO-RUNTIME-001",
    "ITM-BEEF-RUNTIME-001",
    "ITM-BOOK-FAMILY-RUNTIME-001",
    "ITM-CHICKEN-RUNTIME-001",
    "ITM-COOKIE-RUNTIME-001",
    "ITM-MUTTON-RUNTIME-001",
    "ITM-PORKCHOP-RUNTIME-001",
    "ITM-PUMPKIN-PIE-RUNTIME-001",
    "ITM-RABBIT-MATERIAL-RUNTIME-001",
];

#[test]
fn blk_003_item_identities_and_imported_families_are_closed() {
    assert_eq!(
        BLK_003_OWNERS
            .iter()
            .map(|owner| owner.slice)
            .collect::<BTreeSet<_>>(),
        EXPECTED_SLICES.into_iter().collect()
    );
    assert_eq!(
        BLK_003_OWNERS
            .iter()
            .map(|owner| owner.expected_items)
            .sum::<usize>(),
        ItemKind::BLK_003.len()
    );
    assert_eq!(
        ItemKind::BLK_003.map(|item| (item.path(), item.raw_id())),
        [
            ("baked_potato", 1259),
            ("beef", 1139),
            ("cooked_beef", 1140),
            ("book", 1058),
            ("enchanted_book", 1274),
            ("writable_book", 1250),
            ("written_book", 1251),
            ("chicken", 1141),
            ("cooked_chicken", 1142),
            ("cookie", 1131),
            ("mutton", 1294),
            ("cooked_mutton", 1295),
            ("porkchop", 1011),
            ("cooked_porkchop", 1012),
            ("pumpkin_pie", 1271),
            ("rabbit", 1279),
            ("cooked_rabbit", 1280),
            ("rabbit_hide", 1283),
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
        verify_blk_003_families(registry).unwrap(),
        OwnedItemCoverage {
            families: 9,
            items: 18,
        }
    );
}

#[test]
fn food_defaults_match_all_thirteen_edible_identities() {
    let expected = [
        (ItemKind::BakedPotato, 5, 6.0),
        (ItemKind::Beef, 3, 1.800_000_1),
        (ItemKind::CookedBeef, 8, 12.8),
        (ItemKind::Chicken, 2, 1.2),
        (ItemKind::CookedChicken, 6, 7.200_000_3),
        (ItemKind::Cookie, 2, 0.4),
        (ItemKind::Mutton, 2, 1.2),
        (ItemKind::CookedMutton, 6, 9.6),
        (ItemKind::Porkchop, 3, 1.800_000_1),
        (ItemKind::CookedPorkchop, 8, 12.8),
        (ItemKind::PumpkinPie, 8, 4.8),
        (ItemKind::Rabbit, 3, 1.800_000_1),
        (ItemKind::CookedRabbit, 5, 6.0),
    ];
    for (item, nutrition, saturation) in expected {
        let profile = food_profile(item).unwrap();
        assert_eq!(
            (profile.nutrition, profile.saturation),
            (nutrition, saturation)
        );
        assert!(!profile.always_edible);
    }
    let chicken = food_profile(ItemKind::Chicken).unwrap();
    assert_eq!(chicken.effects.len(), 1);
    assert_eq!(
        (
            chicken.effects[0].kind,
            chicken.effects[0].duration_ticks,
            chicken.effects[0].amplifier,
            chicken.effects[0].probability,
        ),
        (EffectKind::Hunger, 600, 0, 0.3)
    );
    assert!(food_profile(ItemKind::RabbitHide).is_none());
}

#[test]
fn cooking_wolf_piglin_and_composter_profiles_are_exact() {
    assert_eq!(COOKING.len(), 6);
    for profile in COOKING {
        assert_eq!(
            (
                profile.furnace_ticks,
                profile.smoker_ticks,
                profile.campfire_ticks,
                profile.furnace_experience,
            ),
            (200, 100, 600, 0.35)
        );
    }
    assert_eq!(wolf_healing(ItemKind::Beef), Some(6));
    assert_eq!(wolf_healing(ItemKind::CookedBeef), Some(16));
    assert_eq!(wolf_healing(ItemKind::Chicken), Some(4));
    assert_eq!(wolf_healing(ItemKind::CookedChicken), Some(12));
    assert_eq!(wolf_healing(ItemKind::Mutton), Some(4));
    assert_eq!(wolf_healing(ItemKind::CookedMutton), Some(12));
    assert_eq!(wolf_healing(ItemKind::Porkchop), Some(6));
    assert_eq!(wolf_healing(ItemKind::CookedPorkchop), Some(16));
    assert_eq!(wolf_healing(ItemKind::Rabbit), Some(6));
    assert_eq!(wolf_healing(ItemKind::CookedRabbit), Some(10));
    assert!(is_piglin_food(ItemKind::Porkchop));
    assert!(is_piglin_food(ItemKind::CookedPorkchop));
    assert!(!is_piglin_food(ItemKind::CookedBeef));
    assert_eq!(compost_chance(ItemKind::BakedPotato), Some(0.85));
    assert_eq!(compost_chance(ItemKind::Cookie), Some(0.85));
    assert_eq!(compost_chance(ItemKind::PumpkinPie), Some(1.0));
}

#[test]
fn cookie_parrot_path_bypasses_the_ordinary_food_transaction() {
    let player = parrot_poison(ItemKind::Cookie, true).unwrap();
    assert_eq!(player.poison_ticks, 900);
    assert_eq!(player.requested_damage, Some(f32::MAX));
    assert!(!player.uses_food_transaction);
    let dispenser = parrot_poison(ItemKind::Cookie, false).unwrap();
    assert_eq!(dispenser.requested_damage, None);
    assert!(parrot_poison(ItemKind::PumpkinPie, true).is_none());
}

#[test]
fn book_defaults_roles_sounds_and_generation_are_distinct() {
    assert_eq!(ItemKind::Book.maximum_stack(), 64);
    assert_eq!(ItemKind::EnchantedBook.maximum_stack(), 1);
    assert_eq!(ItemKind::WritableBook.maximum_stack(), 1);
    assert_eq!(ItemKind::WrittenBook.maximum_stack(), 16);
    assert_eq!(ItemKind::EnchantedBook.rarity(), Rarity::Rare);
    assert!(ItemKind::EnchantedBook.forced_glint());
    assert!(ItemKind::WrittenBook.forced_glint());
    assert!(has_book_role(ItemKind::Book, BookRole::TableEnchantable));
    assert!(has_book_role(ItemKind::WrittenBook, BookRole::Lectern));
    assert!(!has_book_role(ItemKind::EnchantedBook, BookRole::Lectern));
    for item in [
        ItemKind::Book,
        ItemKind::EnchantedBook,
        ItemKind::WritableBook,
        ItemKind::WrittenBook,
    ] {
        assert!(has_book_role(item, BookRole::Bookshelf));
    }
    assert_eq!(
        bookshelf_sound(ItemKind::EnchantedBook, true),
        Some(BookSound::InsertEnchanted)
    );
    assert_eq!(
        bookshelf_sound(ItemKind::WrittenBook, false),
        Some(BookSound::Remove)
    );
    assert_eq!(signed_generation(0), 1);
    assert_eq!(signed_generation(1), 2);
    assert_eq!(signed_generation(2), 2);
    assert_eq!(signed_generation(u8::MAX), 2);
}
