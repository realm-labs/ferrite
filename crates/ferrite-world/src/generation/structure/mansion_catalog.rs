//! Locked woodland-mansion structure, set, biome, template, and loot records.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::mansion_pieces::random_index;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MansionStart {
    pub origin: BlockPos,
    pub rotation: Rotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MansionLootEntry {
    pub item: &'static str,
    pub weight: u32,
    pub minimum: u32,
    pub maximum: u32,
    pub function: Option<&'static str>,
}

pub const MANSION_BIOME_TAG: &str = "#minecraft:has_structure/woodland_mansion";
pub const MANSION_BIOMES: [&str; 2] = ["minecraft:dark_forest", "minecraft:pale_garden"];
pub const MANSION_STEP: &str = "surface_structures";
pub const MANSION_TERRAIN_ADAPTATION: &str = "none";
pub const WOODLAND_MANSIONS_STRUCTURE: &str = "minecraft:mansion";
pub const WOODLAND_MANSIONS_WEIGHT: u32 = 1;
pub const WOODLAND_MANSIONS_PLACEMENT: &str = "random_spread";
pub const WOODLAND_MANSIONS_SPREAD_TYPE: &str = "triangular";
pub const WOODLAND_MANSIONS_SPACING: u32 = 80;
pub const WOODLAND_MANSIONS_SEPARATION: u32 = 20;
pub const WOODLAND_MANSIONS_SALT: u32 = 10_387_319;
pub const MANSION_PIECE_ID: &str = "wmp";
pub const MANSION_LOOT_TABLE: &str = "minecraft:chests/woodland_mansion";

pub const MANSION_TEMPLATES: [&str; 73] = [
    "entrance",
    "wall_flat",
    "wall_window",
    "wall_corner",
    "roof",
    "roof_front",
    "small_wall",
    "small_wall_corner",
    "roof_corner",
    "roof_inner_corner",
    "corridor_floor",
    "carpet_north",
    "carpet_east",
    "carpet_south_1",
    "carpet_west_1",
    "carpet_south_2",
    "carpet_west_2",
    "indoors_wall_1",
    "indoors_door_1",
    "indoors_wall_2",
    "indoors_door_2",
    "1x1_a1",
    "1x1_a2",
    "1x1_a3",
    "1x1_a4",
    "1x1_a5",
    "1x1_as1",
    "1x1_as2",
    "1x1_as3",
    "1x1_as4",
    "1x2_a1",
    "1x2_a2",
    "1x2_a3",
    "1x2_a4",
    "1x2_a5",
    "1x2_a6",
    "1x2_a7",
    "1x2_a8",
    "1x2_a9",
    "1x2_b1",
    "1x2_b2",
    "1x2_b3",
    "1x2_b4",
    "1x2_b5",
    "1x2_s1",
    "1x2_s2",
    "2x2_a1",
    "2x2_a2",
    "2x2_a3",
    "2x2_a4",
    "2x2_s1",
    "1x1_b1",
    "1x1_b2",
    "1x1_b3",
    "1x1_b4",
    "1x1_b5",
    "1x2_c1",
    "1x2_c2",
    "1x2_c3",
    "1x2_c4",
    "1x2_c_stairs",
    "1x2_d1",
    "1x2_d2",
    "1x2_d3",
    "1x2_d4",
    "1x2_d5",
    "1x2_d_stairs",
    "1x2_se1",
    "2x2_b1",
    "2x2_b2",
    "2x2_b3",
    "2x2_b4",
    "2x2_b5",
];

pub const MANSION_RARE_LOOT: [MansionLootEntry; 9] = [
    loot("minecraft:lead", 20, 1, 1),
    loot("minecraft:golden_apple", 15, 1, 1),
    loot("minecraft:enchanted_golden_apple", 2, 1, 1),
    loot("minecraft:music_disc_13", 15, 1, 1),
    loot("minecraft:music_disc_cat", 15, 1, 1),
    loot("minecraft:chainmail_chestplate", 10, 1, 1),
    loot("minecraft:diamond_hoe", 15, 1, 1),
    loot("minecraft:diamond_chestplate", 5, 1, 1),
    MansionLootEntry {
        item: "minecraft:book",
        weight: 10,
        minimum: 1,
        maximum: 1,
        function: Some("minecraft:enchant_randomly:#minecraft:on_random_loot"),
    },
];

pub const MANSION_SUPPLY_LOOT: [MansionLootEntry; 11] = [
    loot("minecraft:iron_ingot", 10, 1, 4),
    loot("minecraft:gold_ingot", 5, 1, 4),
    loot("minecraft:bread", 20, 1, 1),
    loot("minecraft:wheat", 20, 1, 4),
    loot("minecraft:bucket", 10, 1, 1),
    loot("minecraft:redstone", 15, 1, 4),
    loot("minecraft:coal", 15, 1, 4),
    loot("minecraft:melon_seeds", 10, 2, 4),
    loot("minecraft:pumpkin_seeds", 10, 2, 4),
    loot("minecraft:beetroot_seeds", 10, 2, 4),
    loot("minecraft:resin_clump", 50, 2, 4),
];

pub const MANSION_COMMON_LOOT: [MansionLootEntry; 4] = [
    loot("minecraft:bone", 1, 1, 8),
    loot("minecraft:gunpowder", 1, 1, 8),
    loot("minecraft:rotten_flesh", 1, 1, 8),
    loot("minecraft:string", 1, 1, 8),
];

pub const MANSION_RARE_ROLLS: (u32, u32) = (1, 3);
pub const MANSION_SUPPLY_ROLLS: (u32, u32) = (1, 4);
pub const MANSION_COMMON_ROLLS: u32 = 3;
pub const MANSION_TRIM_EMPTY_WEIGHT: u32 = 1;
pub const MANSION_TRIM_WEIGHT: u32 = 1;
pub const MANSION_TRIM_TEMPLATE: &str = "minecraft:vex_armor_trim_smithing_template";

pub fn find_mansion_start(
    chunk_x: i32,
    chunk_z: i32,
    random: &mut impl GenerationRandom,
    mut surface_height: impl FnMut(i32, i32) -> i32,
) -> Option<MansionStart> {
    let rotation = Rotation::ALL[random_index(random, Rotation::ALL.len())];
    let origin_x = chunk_x.wrapping_mul(16).wrapping_add(7);
    let origin_z = chunk_z.wrapping_mul(16).wrapping_add(7);
    let (dx, dz) = match rotation {
        Rotation::None => (5, 5),
        Rotation::Clockwise90 => (-5, 5),
        Rotation::Clockwise180 => (-5, -5),
        Rotation::CounterClockwise90 => (5, -5),
    };
    let y = [
        surface_height(origin_x, origin_z),
        surface_height(origin_x, origin_z.wrapping_add(dz)),
        surface_height(origin_x.wrapping_add(dx), origin_z),
        surface_height(origin_x.wrapping_add(dx), origin_z.wrapping_add(dz)),
    ]
    .into_iter()
    .min()
    .expect("four terrain samples have a minimum");
    (y >= 60).then_some(MansionStart {
        origin: BlockPos::new(origin_x, y, origin_z),
        rotation,
    })
}

pub fn is_mansion_biome(biome: &str) -> bool {
    MANSION_BIOMES.contains(&biome)
}

const fn loot(item: &'static str, weight: u32, minimum: u32, maximum: u32) -> MansionLootEntry {
    MansionLootEntry {
        item,
        weight,
        minimum,
        maximum,
        function: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::generation::feature::random::LegacyRandom;

    use super::*;

    #[test]
    fn template_catalog_is_unique_and_complete() {
        let unique = MANSION_TEMPLATES
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 73);
    }

    #[test]
    fn terrain_gate_uses_the_minimum_of_four_rotation_aware_samples() {
        let mut samples = Vec::new();
        let start = find_mansion_start(2, -3, &mut LegacyRandom::new(0), |x, z| {
            samples.push((x, z));
            if samples.len() == 4 { 59 } else { 80 }
        });
        assert!(start.is_none());
        assert_eq!(samples.len(), 4);
    }
}
