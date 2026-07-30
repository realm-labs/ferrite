//! String identity, acquisition, crafting, trade, and structure joins.

use ferrite_registry::bundle::BundleRegistry;
use thiserror::Error;

pub const STRING_ITEM_ID: u32 = 976;
pub const STRING_MAXIMUM_STACK: u8 = 64;
pub const DIRECT_ACQUISITION_TABLES: [&str; 17] = [
    "blocks/cobweb",
    "blocks/tripwire",
    "entities/cat",
    "entities/cave_spider",
    "entities/spider",
    "entities/strider",
    "gameplay/cat_morning_gift",
    "gameplay/piglin_bartering",
    "gameplay/fishing/junk",
    "archaeology/trail_ruins_common",
    "chests/bastion_bridge",
    "chests/bastion_hoglin_stable",
    "chests/bastion_other",
    "chests/desert_pyramid",
    "chests/pillager_outpost",
    "chests/simple_dungeon",
    "chests/woodland_mansion",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringRecipe {
    pub id: &'static str,
    pub string_count: u8,
    pub output_count: u8,
    pub unlocks_from_string: bool,
}

pub const STRING_RECIPES: [StringRecipe; 9] = [
    recipe("bow", 3, 1, true),
    recipe("bundle", 1, 1, true),
    recipe("candle", 1, 1, true),
    recipe("crossbow", 2, 1, true),
    recipe("fishing_rod", 2, 1, true),
    recipe("lead", 5, 2, true),
    recipe("loom", 2, 1, true),
    recipe("scaffolding", 1, 6, false),
    recipe("white_wool_from_string", 4, 1, true),
];

pub const STRING_UNLOCKS: [&str; 9] = [
    "bow",
    "bundle",
    "candle",
    "crossbow",
    "fishing_rod",
    "lead",
    "loom",
    "tripwire_hook",
    "white_wool_from_string",
];

const fn recipe(
    id: &'static str,
    string_count: u8,
    output_count: u8,
    unlocks_from_string: bool,
) -> StringRecipe {
    StringRecipe {
        id,
        string_count,
        output_count,
        unlocks_from_string,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StringTrade {
    pub profession: &'static str,
    pub level: u8,
    pub string_cost: u8,
    pub emerald_output: u8,
    pub inclusion_probability: f32,
    pub maximum_uses: u8,
    pub villager_experience: u8,
    pub price_multiplier: f32,
}

pub const STRING_TRADES: [StringTrade; 2] = [
    StringTrade {
        profession: "fisherman",
        level: 1,
        string_cost: 20,
        emerald_output: 1,
        inclusion_probability: 0.5,
        maximum_uses: 16,
        villager_experience: 2,
        price_multiplier: 0.05,
    },
    StringTrade {
        profession: "fletcher",
        level: 3,
        string_cost: 14,
        emerald_output: 1,
        inclusion_probability: 1.0,
        maximum_uses: 16,
        villager_experience: 20,
        price_multiplier: 0.05,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureStringRecord {
    pub decoded_templates: usize,
    pub matching_templates: usize,
    pub path: &'static str,
    pub stored_count: u8,
}

pub const STRUCTURE_STRING: StructureStringRecord = StructureStringRecord {
    decoded_templates: 1_212,
    matching_templates: 1,
    path: "trial_chambers/intersection/intersection_2",
    stored_count: 3,
};

pub const fn fishing_junk_probability_denominator(jungle_bamboo_admitted: bool) -> u8 {
    if jungle_bamboo_admitted { 22 } else { 20 }
}

pub const fn looting_bonus(looting_level: u8, uniform: f32) -> u32 {
    (looting_level as f32 * uniform).round() as u32
}

pub fn verify_string_family(registry: &BundleRegistry) -> Result<(), StringCatalogError> {
    if registry.name().to_string() != "minecraft:item" {
        return Err(StringCatalogError::WrongRegistry(
            registry.name().to_string(),
        ));
    }
    let entries = registry
        .entries()
        .filter(|entry| entry.family().as_str() == "string-runtime")
        .collect::<Vec<_>>();
    if entries.len() != 1 {
        return Err(StringCatalogError::FamilyCount(entries.len()));
    }
    let id = entries[0].persistent_id().to_string();
    if id != "minecraft:string" {
        return Err(StringCatalogError::WrongIdentity(id));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StringCatalogError {
    #[error("expected minecraft:item registry, found {0}")]
    WrongRegistry(String),
    #[error("string-runtime contains {0} entries, expected 1")]
    FamilyCount(usize),
    #[error("string-runtime contains {0}, expected minecraft:string")]
    WrongIdentity(String),
}
