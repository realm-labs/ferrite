//! Prismarine shard and crystal acquisition and recipe profiles.

use crate::item::runtime::catalog::ItemKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InclusiveCount {
    pub minimum: u8,
    pub maximum: u8,
}

impl InclusiveCount {
    pub const fn new(minimum: u8, maximum: u8) -> Self {
        Self { minimum, maximum }
    }
}

const fn count(minimum: u8, maximum: u8) -> InclusiveCount {
    InclusiveCount::new(minimum, maximum)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardianKind {
    Guardian,
    ElderGuardian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardianLootProfile {
    pub shard_base: InclusiveCount,
    pub shard_bonus_per_looting_level: InclusiveCount,
    pub secondary_crystal_weight: u8,
    pub secondary_total_weight: u8,
    pub secondary_bonus_per_looting_level: InclusiveCount,
}

pub const fn guardian_loot(kind: GuardianKind) -> GuardianLootProfile {
    GuardianLootProfile {
        shard_base: count(0, 2),
        shard_bonus_per_looting_level: count(0, 1),
        secondary_crystal_weight: 2,
        secondary_total_weight: match kind {
            GuardianKind::Guardian => 5,
            GuardianKind::ElderGuardian => 6,
        },
        secondary_bonus_per_looting_level: count(0, 1),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuriedTreasureProfile {
    pub pool_rolls: InclusiveCount,
    pub crystal_weight: u8,
    pub total_weight: u8,
    pub crystal_count: InclusiveCount,
}

pub const BURIED_TREASURE: BuriedTreasureProfile = BuriedTreasureProfile {
    pool_rolls: count(1, 3),
    crystal_weight: 5,
    total_weight: 15,
    crystal_count: count(1, 5),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeaLanternDrop {
    SeaLantern,
    Crystals(CrystalDropProfile),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrystalDropProfile {
    pub base: InclusiveCount,
    pub fortune_bonus_per_level: InclusiveCount,
    pub capped: InclusiveCount,
    pub explosion_decay: bool,
}

pub const fn sea_lantern_drop(silk_touch_level: u8) -> SeaLanternDrop {
    if silk_touch_level > 0 {
        SeaLanternDrop::SeaLantern
    } else {
        SeaLanternDrop::Crystals(CrystalDropProfile {
            base: count(2, 3),
            fortune_bonus_per_level: count(0, 1),
            capped: count(1, 5),
            explosion_decay: true,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftingKind {
    Shaped,
    Shapeless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrismarineRecipe {
    pub output: &'static str,
    pub kind: CraftingKind,
    pub pattern: &'static [&'static str],
    pub shards: u8,
    pub crystals: u8,
    pub black_dye: u8,
    pub unlock_item: ItemKind,
}

const SQUARE: [&str; 2] = ["##", "##"];
const DARK: [&str; 3] = ["SSS", "SIS", "SSS"];
const LANTERN: [&str; 3] = ["SCS", "CCC", "SCS"];

pub const RECIPES: [PrismarineRecipe; 4] = [
    PrismarineRecipe {
        output: "prismarine",
        kind: CraftingKind::Shaped,
        pattern: &SQUARE,
        shards: 4,
        crystals: 0,
        black_dye: 0,
        unlock_item: ItemKind::PrismarineShard,
    },
    PrismarineRecipe {
        output: "prismarine_bricks",
        kind: CraftingKind::Shapeless,
        pattern: &[],
        shards: 9,
        crystals: 0,
        black_dye: 0,
        unlock_item: ItemKind::PrismarineShard,
    },
    PrismarineRecipe {
        output: "dark_prismarine",
        kind: CraftingKind::Shaped,
        pattern: &DARK,
        shards: 8,
        crystals: 0,
        black_dye: 1,
        unlock_item: ItemKind::PrismarineShard,
    },
    PrismarineRecipe {
        output: "sea_lantern",
        kind: CraftingKind::Shaped,
        pattern: &LANTERN,
        shards: 4,
        crystals: 5,
        black_dye: 0,
        unlock_item: ItemKind::PrismarineCrystals,
    },
];
