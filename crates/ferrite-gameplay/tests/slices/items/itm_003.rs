use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::item::runtime::anvil::{
    AnvilDamage, IncomingEnchantment, build_anvil_preview, damage_anvil, may_take_anvil_result,
};
use ferrite_gameplay::item::runtime::brewing::{
    BREW_DURATION, BREW_FUEL_USES, BrewingStand, MixEdge, MixKind, PotionStack,
};
use ferrite_gameplay::item::runtime::campfire::Campfire;
use ferrite_gameplay::item::runtime::crafting::{
    Crafter, CraftingGrid, CraftingPreview, crafter_power_transition, take_crafting_result,
};
use ferrite_gameplay::item::runtime::furnace::{
    CookingRecipe, Furnace, FurnaceTickInput, experience_to_drop,
};
use ferrite_gameplay::item::runtime::grindstone::{
    enchantment, grindstone_experience, grindstone_result, plain_enchanted_stack,
};
use ferrite_gameplay::item::runtime::inventory::Inventory;
use ferrite_gameplay::item::runtime::item_enchantment::{AppliedEnchantment, EnchantedStack};
use ferrite_gameplay::item::runtime::recipe::{
    CrafterRecipeCache, RECIPE_SERIALIZERS, RecipeDomain, RecipeManager, RecipeRecord,
    crop_crafting_input,
};
use ferrite_gameplay::item::runtime::stack::ItemStack;
use ferrite_gameplay::item::runtime::workstation::{
    ALL_BANNER_PATTERNS, BannerLayer, CartographyMaterial, Loom, MapData, MapPostProcess,
    NO_ITEM_BANNER_PATTERNS, Smithing, Stonecutter, apply_map_post_process, cartography_preview,
};
use std::cell::Cell;
use std::collections::BTreeSet;

fn id(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

fn stack(identity: u64, path: &str, count: i32, maximum: i32, components: u64) -> ItemStack {
    ItemStack::new(identity, id(path), count, maximum, components)
}

fn recipe(key: &str, domain: RecipeDomain, result: ItemStack) -> RecipeRecord {
    RecipeRecord {
        key: key.to_owned(),
        domain,
        result,
        special: false,
        cooking_time: 200,
        experience: 0.0,
    }
}

fn enchanted(path: &str) -> EnchantedStack {
    plain_enchanted_stack(stack(1, path, 1, 1, 0))
}

#[test]
fn recipe_catalog_and_lookup_preserve_closed_sets_and_key_order() {
    assert_eq!(RecipeDomain::ALL.len(), 7);
    assert_eq!(RECIPE_SERIALIZERS.len(), 21);
    assert_eq!(
        RECIPE_SERIALIZERS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        21
    );
    assert_eq!(RECIPE_SERIALIZERS[0], "crafting_shaped");
    assert_eq!(RECIPE_SERIALIZERS[20], "smithing_trim");

    let manager = RecipeManager::prepare(
        7,
        vec![
            recipe(
                "minecraft:z",
                RecipeDomain::Crafting,
                stack(2, "stick", 4, 64, 0),
            ),
            recipe(
                "minecraft:a",
                RecipeDomain::Crafting,
                stack(3, "stick", 2, 64, 0),
            ),
            recipe(
                "minecraft:b",
                RecipeDomain::Smelting,
                stack(4, "iron_ingot", 1, 64, 0),
            ),
        ],
    );
    let keys = manager
        .recipes(RecipeDomain::Crafting)
        .map(|entry| entry.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(keys, ["minecraft:a", "minecraft:z"]);
    assert_eq!(
        manager
            .get_recipe_for(RecipeDomain::Crafting, Some("minecraft:z"), |_| true)
            .unwrap()
            .key,
        "minecraft:z"
    );
    assert_eq!(
        manager
            .get_recipe_for(RecipeDomain::Crafting, Some("minecraft:missing"), |_| true)
            .unwrap()
            .key,
        "minecraft:a"
    );
}

#[test]
fn cropped_inputs_and_crafter_cache_follow_identity_and_lru_rules() {
    let empty = ItemStack::empty();
    let input = vec![
        empty.clone(),
        empty.clone(),
        empty.clone(),
        empty.clone(),
        stack(1, "oak_planks", 4, 64, 9),
        stack(2, "stick", 2, 64, 3),
        empty.clone(),
        empty.clone(),
        empty,
    ];
    let cropped = crop_crafting_input(3, 3, &input);
    assert_eq!(
        (cropped.width, cropped.height, cropped.left, cropped.top),
        (2, 1, 1, 1)
    );

    let calls = Cell::new(0);
    let mut cache = CrafterRecipeCache::new();
    assert_eq!(
        cache.get_or_insert(1, &cropped, || {
            calls.set(calls.get() + 1);
            Some("minecraft:first".to_owned())
        }),
        Some("minecraft:first".to_owned())
    );
    let mut count_changed = cropped.clone();
    count_changed.cells[0].count = 31;
    assert_eq!(
        cache.get_or_insert(1, &count_changed, || {
            calls.set(calls.get() + 1);
            Some("minecraft:wrong".to_owned())
        }),
        Some("minecraft:first".to_owned())
    );
    assert_eq!(calls.get(), 1);

    for component in 10..21 {
        let distinct = crop_crafting_input(1, 1, &[stack(10, "stone", 1, 64, component)]);
        cache.get_or_insert(1, &distinct, || Some(format!("minecraft:{component}")));
    }
    assert_eq!(cache.len(), 10);
    cache.get_or_insert(2, &cropped, || Some("minecraft:reload".to_owned()));
    assert_eq!(cache.len(), 1);
    assert_eq!(
        cache.get_or_insert(2, &crop_crafting_input(1, 1, &[ItemStack::empty()]), || {
            Some("minecraft:never".to_owned())
        }),
        None
    );
    assert_eq!(cache.len(), 1);
}

#[test]
fn manual_crafting_gates_preview_and_places_fresh_remainders_by_crop_offset() {
    let locked = recipe(
        "minecraft:locked",
        RecipeDomain::Crafting,
        stack(20, "cake", 1, 1, 0),
    );
    let mut preview = CraftingPreview::empty();
    preview.recompute(Some(&locked), true, false, true);
    assert!(preview.result.is_empty());
    preview.recompute(Some(&locked), true, true, true);
    assert_eq!(preview.stored_recipe.as_deref(), Some("minecraft:locked"));

    let mut grid = CraftingGrid {
        width: 3,
        height: 3,
        cells: vec![ItemStack::empty(); 9],
    };
    grid.cells[4] = stack(21, "milk_bucket", 1, 1, 0);
    grid.cells[5] = stack(22, "wheat", 2, 64, 0);
    let cropped = crop_crafting_input(grid.width, grid.height, &grid.cells);
    let remainders = [stack(23, "bucket", 1, 16, 0), stack(24, "bowl", 1, 64, 0)];
    let mut inventory = Inventory::empty(1);
    let outcome = take_crafting_result(
        &mut preview,
        &mut grid,
        &cropped,
        &remainders,
        &mut inventory,
    );
    assert_eq!(outcome.credited_recipe.as_deref(), Some("minecraft:locked"));
    assert_eq!(outcome.consumed_cells, 2);
    assert_eq!(grid.cells[4].item.as_ref(), Some(&id("bucket")));
    assert_eq!(grid.cells[5].count, 1);
    assert_eq!(inventory.slots[0].stack.item.as_ref(), Some(&id("bowl")));
    assert!(outcome.dropped_remainders.is_empty());
}

#[test]
fn crafter_balances_insertion_and_delivers_result_then_remainders() {
    let pulse = crafter_power_transition(true, false);
    assert_eq!(pulse.schedule_after, Some(4));
    assert!(pulse.triggered);
    assert!(!crafter_power_transition(false, true).triggered);

    let mut crafter = Crafter::empty();
    crafter.slots[0] = stack(30, "iron_ingot", 3, 64, 0);
    crafter.slots[1] = stack(31, "iron_ingot", 2, 64, 0);
    assert!(!crafter.can_place_item(0, &stack(32, "iron_ingot", 1, 64, 0)));
    crafter.disabled[2..].fill(true);
    assert!(crafter.can_place_item(1, &stack(32, "iron_ingot", 1, 64, 0)));

    let mut destination = Inventory::empty(1);
    destination.slots[0].policy.maximum = 1;
    let outcome = crafter.craft(
        stack(33, "iron_sword", 1, 1, 0),
        &[stack(34, "bucket", 1, 16, 0)],
        &mut destination,
        false,
    );
    assert_eq!(
        destination.slots[0].stack.item.as_ref(),
        Some(&id("iron_sword"))
    );
    assert_eq!(outcome.residue[0].item.as_ref(), Some(&id("bucket")));
    assert!(outcome.emitted_residue_events);
    assert_eq!((crafter.slots[0].count, crafter.slots[1].count), (2, 1));
    assert_eq!(crafter.animation_ticks, 6);
    for _ in 0..6 {
        crafter.tick_animation();
    }
    assert!(!crafter.crafting_state);
}

#[test]
fn furnace_renews_fire_completes_and_uses_fractional_xp_draw() {
    let cooking = CookingRecipe {
        key: "minecraft:iron".to_owned(),
        result: stack(40, "iron_ingot", 1, 64, 0),
        cooking_time: 100,
        experience: 0.7,
    };
    let mut furnace = Furnace::new();
    furnace.inventory.slots[0].stack = stack(41, "raw_iron", 1, 64, 0);
    furnace.inventory.slots[1].stack = stack(42, "coal", 1, 64, 0);
    furnace.lit_remaining = 1;
    furnace.cook_total = 1;
    let outcome = furnace.tick(FurnaceTickInput {
        recipe: Some(&cooking),
        fuel_duration: 1600,
        fuel_remainder: None,
        input_identity_changed: false,
        wet_sponge_bucket_conversion: false,
    });
    assert!(outcome.ignited);
    assert!(outcome.completed);
    assert_eq!(furnace.lit_remaining, 1600);
    assert_eq!(
        furnace.inventory.slots[2].stack.item.as_ref(),
        Some(&id("iron_ingot"))
    );
    assert_eq!(furnace.used_recipes["minecraft:iron"], 1);

    furnace.cook_progress = 7;
    furnace.lit_remaining = 0;
    furnace.inventory.slots[0].stack = ItemStack::empty();
    furnace.tick(FurnaceTickInput {
        recipe: None,
        fuel_duration: 0,
        fuel_remainder: None,
        input_identity_changed: false,
        wet_sponge_bucket_conversion: false,
    });
    assert_eq!(furnace.cook_progress, 5);
    assert_eq!(experience_to_drop(3, 0.7, Some(0.09)), 3);
    assert_eq!(experience_to_drop(3, 0.7, Some(0.11)), 2);
}

#[test]
fn brewing_refuels_starts_cancels_and_prefers_container_mix_order() {
    let blaze_powder = id("blaze_powder");
    let ingredient = id("gunpowder");
    let mut stand = BrewingStand::empty();
    stand.fuel = stack(50, "blaze_powder", 1, 64, 0);
    stand.ingredient = stack(51, "gunpowder", 2, 64, 0);
    stand.bottles[0] = PotionStack {
        stack: stack(52, "potion", 1, 1, 0),
        potion_fingerprint: Some(7),
    };
    let mixes = [
        MixEdge {
            ingredient: ingredient.clone(),
            kind: MixKind::Container {
                from_item: id("potion"),
                to_item: id("splash_potion"),
            },
        },
        MixEdge {
            ingredient: ingredient.clone(),
            kind: MixKind::Potion {
                from_potion: 7,
                to_potion: 8,
            },
        },
    ];
    let first = stand.tick(&blaze_powder, &mixes, None);
    assert!(first.fuel_refilled && first.started);
    assert_eq!(stand.fuel_uses, BREW_FUEL_USES - 1);
    assert_eq!(stand.brew_time, BREW_DURATION);

    stand.brew_time = 1;
    let completed = stand.tick(
        &blaze_powder,
        &mixes,
        Some(&stack(53, "glass_bottle", 1, 64, 0)),
    );
    assert!(completed.completed);
    assert_eq!(
        stand.bottles[0].stack.item.as_ref(),
        Some(&id("splash_potion"))
    );
    assert_eq!(stand.bottles[0].potion_fingerprint, Some(7));
    assert_eq!(stand.ingredient.count, 1);
    assert_eq!(stand.dropped_remainders.len(), 1);

    stand.brew_time = 10;
    stand.remembered_ingredient = Some(ingredient);
    stand.ingredient = stack(54, "redstone", 1, 64, 0);
    assert!(stand.tick(&blaze_powder, &mixes, None).cancelled);
    assert_eq!(stand.brew_time, 0);
}

#[test]
fn campfire_uses_four_slots_retries_disabled_outputs_and_cools_by_two() {
    let mut campfire = Campfire::empty();
    let mut food = stack(60, "cod", 5, 64, 0);
    assert_eq!(
        campfire.place_food(&mut food, true, Some(2), false),
        Some(0)
    );
    assert_eq!(food.count, 4);
    assert_eq!(campfire.place_food(&mut food, false, Some(2), false), None);

    assert!(
        campfire
            .tick_lit(
                [Some(stack(61, "cooked_cod", 1, 64, 0)), None, None, None,],
                [false, true, true, true],
            )
            .is_empty()
    );
    assert!(
        campfire
            .tick_lit(
                [Some(stack(61, "cooked_cod", 1, 64, 0)), None, None, None,],
                [false, true, true, true],
            )
            .is_empty()
    );
    let completed = campfire.tick_lit(
        [Some(stack(61, "cooked_cod", 1, 64, 0)), None, None, None],
        [true; 4],
    );
    assert_eq!(completed[0].output.item.as_ref(), Some(&id("cooked_cod")));

    campfire.slots[1] = stack(62, "salmon", 1, 64, 0);
    campfire.progress[1] = 6;
    campfire.total[1] = 10;
    campfire.tick_unlit();
    assert_eq!(campfire.progress[1], 4);
}

#[test]
fn smithing_and_stonecutter_keep_transaction_and_selection_state() {
    let mut smithing = Smithing::empty();
    smithing.inputs = [
        stack(70, "netherite_upgrade_smithing_template", 1, 64, 0),
        stack(71, "diamond_sword", 1, 1, 0),
        stack(72, "netherite_ingot", 1, 64, 0),
    ];
    smithing.recompute(None);
    assert!(smithing.recipe_error);
    smithing.recompute(Some((
        "minecraft:netherite_sword",
        stack(73, "netherite_sword", 1, 1, 0),
    )));
    assert_eq!(
        smithing.take().as_deref(),
        Some("minecraft:netherite_sword")
    );
    assert!(smithing.inputs.iter().all(ItemStack::is_empty));

    let mut input = stack(74, "stone", 2, 64, 0);
    let mut stonecutter = Stonecutter::empty();
    stonecutter.change_input(
        &input,
        vec![
            (
                "minecraft:slab".to_owned(),
                stack(75, "stone_slab", 2, 64, 0),
            ),
            (
                "minecraft:stairs".to_owned(),
                stack(76, "stone_stairs", 1, 64, 0),
            ),
        ],
    );
    assert!(stonecutter.select(1));
    assert!(!stonecutter.select(1));
    assert!(stonecutter.select(99));
    assert_eq!(stonecutter.selected, 1);
    assert!(stonecutter.take(&mut input, 10).play_sound);
    assert!(!stonecutter.take(&mut input, 10).play_sound);
}

#[test]
fn cartography_gates_operations_and_allocates_scaled_or_locked_maps() {
    let map = MapData {
        id: 5,
        center_x: -257,
        center_z: 511,
        scale: 1,
        locked: false,
    };
    assert_eq!(
        cartography_preview(map, CartographyMaterial::Paper),
        Some((MapPostProcess::Scale, 1))
    );
    let scaled = apply_map_post_process(map, MapPostProcess::Scale, 9);
    assert_eq!(scaled[0].id, 9);
    assert_eq!(scaled[0].scale, 2);
    assert_eq!((scaled[0].center_x, scaled[0].center_z), (-320, 704));

    let locked = apply_map_post_process(map, MapPostProcess::Lock, 10);
    assert!(locked[0].locked);
    assert_eq!(
        cartography_preview(locked[0], CartographyMaterial::Paper),
        None
    );
    assert_eq!(
        cartography_preview(locked[0], CartographyMaterial::EmptyMap),
        Some((MapPostProcess::Duplicate, 2))
    );
    assert_eq!(
        apply_map_post_process(map, MapPostProcess::Duplicate, 99),
        [map, map]
    );
}

#[test]
fn loom_catalog_and_selection_preserve_holder_identity_and_six_layer_cap() {
    assert_eq!(ALL_BANNER_PATTERNS.len(), 43);
    assert_eq!(NO_ITEM_BANNER_PATTERNS.len(), 32);
    assert_eq!(
        ALL_BANNER_PATTERNS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        43
    );
    assert!(
        NO_ITEM_BANNER_PATTERNS
            .iter()
            .all(|pattern| { ALL_BANNER_PATTERNS.contains(pattern) && *pattern != "base" })
    );

    let mut loom = Loom::new();
    loom.update_choices(vec![id("border"), id("circle")]);
    assert!(loom.select(1, 4));
    loom.update_choices(vec![id("circle"), id("border")]);
    assert_eq!(loom.selected, 0);
    loom.existing_layers = (0..5)
        .map(|color| BannerLayer {
            pattern: id("border"),
            dye_color: color,
        })
        .collect();
    assert!(loom.select(0, 9));
    assert_eq!(loom.result_layers.len(), 6);
    loom.existing_layers.push(BannerLayer {
        pattern: id("circle"),
        dye_color: 10,
    });
    assert!(!loom.select(0, 11));
}

#[test]
fn grindstone_retains_curses_repairs_items_and_samples_removed_costs() {
    let mut first = enchanted("diamond_sword");
    first.maximum_damage = 100;
    first.damage = 70;
    first.enchantments = vec![
        enchantment("sharpness", 3, false, 5),
        enchantment("binding_curse", 1, true, 1),
    ];
    let mut second = enchanted("diamond_sword");
    second.maximum_damage = 100;
    second.damage = 60;
    second.enchantments = vec![
        enchantment("smite", 2, false, 7),
        enchantment("vanishing_curse", 1, true, 1),
    ];
    let result = grindstone_result(Some(&first), Some(&second)).unwrap();
    assert_eq!(result.damage, 25);
    assert_eq!(result.enchantments.len(), 2);
    assert!(result.enchantments.iter().all(|entry| entry.curse));
    assert_eq!(result.repair_cost, 3);
    assert_eq!(
        grindstone_experience(Some(&first), Some(&second), Some(0)),
        Ok(6)
    );
    assert_eq!(
        grindstone_experience(Some(&first), Some(&second), Some(5)),
        Ok(11)
    );

    let mut book = enchanted("enchanted_book");
    book.stored_enchantments = true;
    book.enchantments
        .push(enchantment("efficiency", 1, false, 1));
    let plain_book = grindstone_result(Some(&book), None).unwrap();
    assert_eq!(plain_book.stack.item.as_ref(), Some(&id("book")));
}

#[test]
fn anvil_applies_material_sacrifice_rename_limits_and_damage_boundary() {
    let mut base = enchanted("diamond_pickaxe");
    base.maximum_damage = 100;
    base.damage = 80;
    let material = EnchantedStack {
        stack: stack(80, "diamond", 4, 64, 0),
        ..enchanted("diamond")
    };
    let repaired = build_anvil_preview(&base, Some(&material), true, &[], |_, _| true, None);
    assert_eq!(repaired.result.as_ref().unwrap().damage, 0);
    assert_eq!(repaired.addition_consumed, 4);
    assert_eq!(repaired.level_cost, 4);

    let mut sacrifice = enchanted("diamond_pickaxe");
    sacrifice.maximum_damage = 100;
    sacrifice.damage = 60;
    let combined = build_anvil_preview(&base, Some(&sacrifice), false, &[], |_, _| true, None);
    assert_eq!(combined.result.as_ref().unwrap().damage, 28);
    assert_eq!(combined.level_cost, 2);

    base.repair_cost = 40;
    let renamed = build_anvil_preview(&base, None, false, &[], |_, _| true, Some("workhorse"));
    assert!(renamed.rename_only);
    assert_eq!(renamed.level_cost, 39);
    assert_eq!(renamed.result.as_ref().unwrap().repair_cost, 40);
    assert!(may_take_anvil_result(&renamed, 39, false));

    let sharpness = AppliedEnchantment {
        key: id("sharpness"),
        level: 1,
        curse: false,
        minimum_cost: 1,
        anvil_cost: 2,
    };
    let incoming = [IncomingEnchantment {
        enchantment: sharpness,
        maximum_level: 5,
        can_apply: true,
    }];
    let enchanted_preview =
        build_anvil_preview(&base, Some(&sacrifice), false, &incoming, |_, _| true, None);
    assert!(enchanted_preview.result.is_none());
    assert!(enchanted_preview.level_cost >= 40);

    assert_eq!(
        damage_anvil(AnvilDamage::Unchanged, false, 0.119_999),
        AnvilDamage::Chipped
    );
    assert_eq!(
        damage_anvil(AnvilDamage::Unchanged, false, 0.12),
        AnvilDamage::Unchanged
    );
}
