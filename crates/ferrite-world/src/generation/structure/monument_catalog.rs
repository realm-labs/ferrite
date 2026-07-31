//! Locked ocean-monument structure, set, biome, and spawn-override records.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::structure::BlockBox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonumentSpawnEntry {
    pub entity: &'static str,
    pub weight: u32,
    pub minimum: u32,
    pub maximum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonumentSpawnOverride {
    Monsters(&'static [MonumentSpawnEntry]),
    Empty,
}

pub const MONUMENT_BIOME_TAG: &str = "#minecraft:has_structure/ocean_monument";
pub const MONUMENT_SURROUNDING_BIOME_TAG: &str = "#minecraft:required_ocean_monument_surrounding";
pub const MONUMENT_STEP: &str = "surface_structures";
pub const MONUMENT_TERRAIN_ADAPTATION: &str = "none";
pub const MONUMENT_BIOME_RANGE: u32 = 29;

pub const MONUMENT_START_BIOMES: [&str; 4] = [
    "minecraft:deep_frozen_ocean",
    "minecraft:deep_cold_ocean",
    "minecraft:deep_ocean",
    "minecraft:deep_lukewarm_ocean",
];

pub const MONUMENT_SURROUNDING_BIOMES: [&str; 11] = [
    "minecraft:deep_frozen_ocean",
    "minecraft:deep_cold_ocean",
    "minecraft:deep_ocean",
    "minecraft:deep_lukewarm_ocean",
    "minecraft:frozen_ocean",
    "minecraft:ocean",
    "minecraft:cold_ocean",
    "minecraft:lukewarm_ocean",
    "minecraft:warm_ocean",
    "minecraft:river",
    "minecraft:frozen_river",
];

pub const OCEAN_MONUMENTS_STRUCTURE: &str = "minecraft:monument";
pub const OCEAN_MONUMENTS_WEIGHT: u32 = 1;
pub const OCEAN_MONUMENTS_PLACEMENT: &str = "random_spread";
pub const OCEAN_MONUMENTS_SPREAD_TYPE: &str = "triangular";
pub const OCEAN_MONUMENTS_SPACING: u32 = 32;
pub const OCEAN_MONUMENTS_SEPARATION: u32 = 5;
pub const OCEAN_MONUMENTS_SALT: u32 = 10_387_313;

pub const MONUMENT_MONSTERS: [MonumentSpawnEntry; 1] = [MonumentSpawnEntry {
    entity: "minecraft:guardian",
    weight: 1,
    minimum: 2,
    maximum: 4,
}];

pub fn monument_monster_spawns_at(
    building: BlockBox,
    position: BlockPos,
) -> Option<&'static [MonumentSpawnEntry]> {
    building
        .contains(position)
        .then_some(MONUMENT_MONSTERS.as_slice())
}

pub fn monument_spawn_override_at(
    building: BlockBox,
    category: &str,
    position: BlockPos,
) -> Option<MonumentSpawnOverride> {
    if !building.contains(position) {
        return None;
    }
    match category {
        "monster" => Some(MonumentSpawnOverride::Monsters(&MONUMENT_MONSTERS)),
        "axolotls" | "underground_water_creature" => Some(MonumentSpawnOverride::Empty),
        _ => None,
    }
}
