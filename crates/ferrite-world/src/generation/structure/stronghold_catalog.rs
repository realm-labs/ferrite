//! Locked stronghold structure/set, biome, and loot records.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrongholdLootEntry {
    pub item: &'static str,
    pub weight: u32,
    pub minimum: u32,
    pub maximum: u32,
    pub function: Option<&'static str>,
}

pub const STRONGHOLD_BIOME_TAG: &str = "#minecraft:has_structure/stronghold";
pub const STRONGHOLD_PREFERRED_BIOME_TAG: &str = "#minecraft:stronghold_biased_to";
pub const STRONGHOLD_STEP: &str = "surface_structures";
pub const STRONGHOLD_TERRAIN_ADAPTATION: &str = "bury";
pub const STRONGHOLDS_STRUCTURE: &str = "minecraft:stronghold";
pub const STRONGHOLDS_WEIGHT: u32 = 1;
pub const STRONGHOLDS_DISTANCE: u32 = 32;
pub const STRONGHOLDS_SPREAD: u32 = 3;
pub const STRONGHOLDS_COUNT: u32 = 128;
pub const STRONGHOLDS_SALT: u32 = 0;

pub const STRONGHOLD_BIOMES: [&str; 55] = [
    "minecraft:badlands",
    "minecraft:bamboo_jungle",
    "minecraft:beach",
    "minecraft:birch_forest",
    "minecraft:cherry_grove",
    "minecraft:cold_ocean",
    "minecraft:dark_forest",
    "minecraft:deep_cold_ocean",
    "minecraft:deep_dark",
    "minecraft:deep_frozen_ocean",
    "minecraft:deep_lukewarm_ocean",
    "minecraft:deep_ocean",
    "minecraft:desert",
    "minecraft:dripstone_caves",
    "minecraft:eroded_badlands",
    "minecraft:flower_forest",
    "minecraft:forest",
    "minecraft:frozen_ocean",
    "minecraft:frozen_peaks",
    "minecraft:frozen_river",
    "minecraft:grove",
    "minecraft:ice_spikes",
    "minecraft:jagged_peaks",
    "minecraft:jungle",
    "minecraft:lukewarm_ocean",
    "minecraft:lush_caves",
    "minecraft:mangrove_swamp",
    "minecraft:meadow",
    "minecraft:mushroom_fields",
    "minecraft:ocean",
    "minecraft:old_growth_birch_forest",
    "minecraft:old_growth_pine_taiga",
    "minecraft:old_growth_spruce_taiga",
    "minecraft:pale_garden",
    "minecraft:plains",
    "minecraft:river",
    "minecraft:savanna",
    "minecraft:savanna_plateau",
    "minecraft:snowy_beach",
    "minecraft:snowy_plains",
    "minecraft:snowy_slopes",
    "minecraft:snowy_taiga",
    "minecraft:sparse_jungle",
    "minecraft:stony_peaks",
    "minecraft:stony_shore",
    "minecraft:sulfur_caves",
    "minecraft:sunflower_plains",
    "minecraft:swamp",
    "minecraft:taiga",
    "minecraft:warm_ocean",
    "minecraft:windswept_forest",
    "minecraft:windswept_gravelly_hills",
    "minecraft:windswept_hills",
    "minecraft:windswept_savanna",
    "minecraft:wooded_badlands",
];

pub const STRONGHOLD_PREFERRED_BIOMES: [&str; 38] = [
    "minecraft:plains",
    "minecraft:sunflower_plains",
    "minecraft:snowy_plains",
    "minecraft:ice_spikes",
    "minecraft:desert",
    "minecraft:forest",
    "minecraft:flower_forest",
    "minecraft:birch_forest",
    "minecraft:dark_forest",
    "minecraft:pale_garden",
    "minecraft:old_growth_birch_forest",
    "minecraft:old_growth_pine_taiga",
    "minecraft:old_growth_spruce_taiga",
    "minecraft:taiga",
    "minecraft:snowy_taiga",
    "minecraft:savanna",
    "minecraft:savanna_plateau",
    "minecraft:windswept_hills",
    "minecraft:windswept_gravelly_hills",
    "minecraft:windswept_forest",
    "minecraft:windswept_savanna",
    "minecraft:jungle",
    "minecraft:sparse_jungle",
    "minecraft:bamboo_jungle",
    "minecraft:badlands",
    "minecraft:eroded_badlands",
    "minecraft:wooded_badlands",
    "minecraft:meadow",
    "minecraft:cherry_grove",
    "minecraft:grove",
    "minecraft:snowy_slopes",
    "minecraft:frozen_peaks",
    "minecraft:jagged_peaks",
    "minecraft:stony_peaks",
    "minecraft:mushroom_fields",
    "minecraft:dripstone_caves",
    "minecraft:lush_caves",
    "minecraft:sulfur_caves",
];

pub const STRONGHOLD_CORRIDOR_LOOT_TABLE: &str = "minecraft:chests/stronghold_corridor";
pub const STRONGHOLD_CROSSING_LOOT_TABLE: &str = "minecraft:chests/stronghold_crossing";
pub const STRONGHOLD_LIBRARY_LOOT_TABLE: &str = "minecraft:chests/stronghold_library";
pub const STRONGHOLD_CORRIDOR_ROLLS: (u32, u32) = (2, 3);
pub const STRONGHOLD_CROSSING_ROLLS: (u32, u32) = (1, 4);
pub const STRONGHOLD_LIBRARY_ROLLS: (u32, u32) = (2, 10);
pub const STRONGHOLD_CORRIDOR_TRIM_EMPTY_WEIGHT: u32 = 9;
pub const STRONGHOLD_CORRIDOR_TRIM_WEIGHT: u32 = 1;
pub const STRONGHOLD_LIBRARY_TRIM_WEIGHT: u32 = 1;
pub const STRONGHOLD_EYE_TRIM_TEMPLATE: &str = "minecraft:eye_armor_trim_smithing_template";

pub const STRONGHOLD_CROSSING_LOOT: [StrongholdLootEntry; 8] = [
    loot("minecraft:iron_ingot", 10, 1, 5),
    loot("minecraft:gold_ingot", 5, 1, 3),
    loot("minecraft:redstone", 5, 4, 9),
    loot("minecraft:coal", 10, 3, 8),
    loot("minecraft:bread", 15, 1, 3),
    loot("minecraft:apple", 15, 1, 3),
    loot("minecraft:iron_pickaxe", 1, 1, 1),
    enchanted_book(1),
];

pub const STRONGHOLD_LIBRARY_LOOT: [StrongholdLootEntry; 5] = [
    loot("minecraft:book", 20, 1, 3),
    loot("minecraft:paper", 20, 2, 7),
    loot("minecraft:map", 1, 1, 1),
    loot("minecraft:compass", 1, 1, 1),
    enchanted_book(10),
];

pub const STRONGHOLD_CORRIDOR_LOOT: [StrongholdLootEntry; 21] = [
    loot("minecraft:ender_pearl", 10, 1, 1),
    loot("minecraft:diamond", 3, 1, 3),
    loot("minecraft:iron_ingot", 10, 1, 5),
    loot("minecraft:gold_ingot", 5, 1, 3),
    loot("minecraft:redstone", 5, 4, 9),
    loot("minecraft:bread", 15, 1, 3),
    loot("minecraft:apple", 15, 1, 3),
    loot("minecraft:iron_pickaxe", 5, 1, 1),
    loot("minecraft:iron_sword", 5, 1, 1),
    loot("minecraft:iron_chestplate", 5, 1, 1),
    loot("minecraft:iron_helmet", 5, 1, 1),
    loot("minecraft:iron_leggings", 5, 1, 1),
    loot("minecraft:iron_boots", 5, 1, 1),
    loot("minecraft:golden_apple", 1, 1, 1),
    loot("minecraft:leather", 1, 1, 5),
    loot("minecraft:copper_horse_armor", 1, 1, 1),
    loot("minecraft:iron_horse_armor", 1, 1, 1),
    loot("minecraft:golden_horse_armor", 1, 1, 1),
    loot("minecraft:diamond_horse_armor", 1, 1, 1),
    loot("minecraft:music_disc_otherside", 1, 1, 1),
    enchanted_book(1),
];

const fn loot(item: &'static str, weight: u32, minimum: u32, maximum: u32) -> StrongholdLootEntry {
    StrongholdLootEntry {
        item,
        weight,
        minimum,
        maximum,
        function: None,
    }
}

const fn enchanted_book(weight: u32) -> StrongholdLootEntry {
    StrongholdLootEntry {
        item: "minecraft:book",
        weight,
        minimum: 1,
        maximum: 1,
        function: Some("minecraft:enchant_with_levels:30:#minecraft:on_random_loot"),
    }
}
