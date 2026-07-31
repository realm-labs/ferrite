//! Locked fortress records consumed by placement, spawning, and loot joins.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::structure::fortress_graph::FortressPiece;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FortressSpawnEntry {
    pub entity: &'static str,
    pub weight: u32,
    pub minimum: u32,
    pub maximum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherComplexEntry {
    pub structure: &'static str,
    pub weight: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FortressLootEntry {
    pub item: &'static str,
    pub weight: u32,
    pub minimum: u32,
    pub maximum: u32,
}

pub const FORTRESS_BIOME_TAG: &str = "#minecraft:has_structure/nether_fortress";
pub const FORTRESS_STEP: &str = "underground_decoration";
pub const FORTRESS_TERRAIN_ADAPTATION: &str = "none";
pub const FORTRESS_LOOT_TABLE: &str = "minecraft:chests/nether_bridge";

pub const FORTRESS_BIOMES: [&str; 5] = [
    "minecraft:nether_wastes",
    "minecraft:soul_sand_valley",
    "minecraft:crimson_forest",
    "minecraft:warped_forest",
    "minecraft:basalt_deltas",
];

pub const FORTRESS_MONSTERS: [FortressSpawnEntry; 5] = [
    FortressSpawnEntry {
        entity: "minecraft:blaze",
        weight: 10,
        minimum: 2,
        maximum: 3,
    },
    FortressSpawnEntry {
        entity: "minecraft:zombified_piglin",
        weight: 5,
        minimum: 4,
        maximum: 4,
    },
    FortressSpawnEntry {
        entity: "minecraft:wither_skeleton",
        weight: 8,
        minimum: 5,
        maximum: 5,
    },
    FortressSpawnEntry {
        entity: "minecraft:skeleton",
        weight: 2,
        minimum: 5,
        maximum: 5,
    },
    FortressSpawnEntry {
        entity: "minecraft:magma_cube",
        weight: 3,
        minimum: 4,
        maximum: 4,
    },
];

pub const NETHER_COMPLEXES: [NetherComplexEntry; 2] = [
    NetherComplexEntry {
        structure: "minecraft:fortress",
        weight: 2,
    },
    NetherComplexEntry {
        structure: "minecraft:bastion_remnant",
        weight: 3,
    },
];
pub const NETHER_COMPLEXES_SPACING: u32 = 27;
pub const NETHER_COMPLEXES_SEPARATION: u32 = 4;
pub const NETHER_COMPLEXES_SALT: u32 = 30_084_232;

pub const FORTRESS_PRIMARY_LOOT_ROLLS: (u32, u32) = (2, 4);
pub const FORTRESS_PRIMARY_LOOT: [FortressLootEntry; 13] = [
    FortressLootEntry {
        item: "minecraft:diamond",
        weight: 5,
        minimum: 1,
        maximum: 3,
    },
    FortressLootEntry {
        item: "minecraft:iron_ingot",
        weight: 5,
        minimum: 1,
        maximum: 5,
    },
    FortressLootEntry {
        item: "minecraft:gold_ingot",
        weight: 15,
        minimum: 1,
        maximum: 3,
    },
    single("minecraft:golden_sword", 5),
    single("minecraft:golden_chestplate", 5),
    single("minecraft:flint_and_steel", 5),
    FortressLootEntry {
        item: "minecraft:nether_wart",
        weight: 5,
        minimum: 3,
        maximum: 7,
    },
    single("minecraft:saddle", 10),
    single("minecraft:golden_horse_armor", 8),
    single("minecraft:copper_horse_armor", 5),
    single("minecraft:iron_horse_armor", 5),
    single("minecraft:diamond_horse_armor", 3),
    FortressLootEntry {
        item: "minecraft:obsidian",
        weight: 2,
        minimum: 2,
        maximum: 4,
    },
];
pub const FORTRESS_TRIM_EMPTY_WEIGHT: u32 = 14;
pub const FORTRESS_TRIM_TEMPLATE: &str = "minecraft:rib_armor_trim_smithing_template";
pub const FORTRESS_TRIM_TEMPLATE_WEIGHT: u32 = 1;

pub fn fortress_monster_spawns_at(
    pieces: &[FortressPiece],
    position: BlockPos,
) -> Option<&'static [FortressSpawnEntry]> {
    pieces
        .iter()
        .any(|piece| piece.bounding_box.contains(position))
        .then_some(FORTRESS_MONSTERS.as_slice())
}

const fn single(item: &'static str, weight: u32) -> FortressLootEntry {
    FortressLootEntry {
        item,
        weight,
        minimum: 1,
        maximum: 1,
    }
}
