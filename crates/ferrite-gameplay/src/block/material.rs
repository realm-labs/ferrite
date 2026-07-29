//! Property-free and low-state material families owned by the SIM-004 block batch.
//!
//! These records are protocol identities and deterministic loot/physical boundaries. Recipe,
//! world-generation, and projection owners consume them without duplicating the locked values.

use ferrite_foundation::direction::Axis;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialRecord {
    pub name: &'static str,
    pub block_id: u16,
    pub state_id: u32,
    pub item_id: u16,
    pub hardness: f32,
    pub resistance: f32,
    pub correct_tool_required: bool,
}

impl MaterialRecord {
    pub fn self_drop(self, correct_tool: bool, survives_explosion: bool) -> bool {
        (!self.correct_tool_required || correct_tool) && survives_explosion
    }
}

pub const ANCIENT_DEBRIS: MaterialRecord = MaterialRecord {
    name: "ancient_debris",
    block_id: 916,
    state_id: 21_819,
    item_id: 109,
    hardness: 30.0,
    resistance: 1_200.0,
    correct_tool_required: true,
};

pub const CLAY: MaterialRecord = MaterialRecord {
    name: "clay",
    block_id: 281,
    state_id: 6_946,
    item_id: 370,
    hardness: 0.6,
    resistance: 0.6,
    correct_tool_required: false,
};

pub const DRIPSTONE_BLOCK: MaterialRecord = MaterialRecord {
    name: "dripstone_block",
    block_id: 1_132,
    state_id: 30_208,
    item_id: 53,
    hardness: 1.5,
    resistance: 1.0,
    correct_tool_required: true,
};

pub const MOSSY_COBBLESTONE: MaterialRecord = MaterialRecord {
    name: "mossy_cobblestone",
    block_id: 192,
    state_id: 3_368,
    item_id: 348,
    hardness: 2.0,
    resistance: 6.0,
    correct_tool_required: true,
};

pub const MELON: MaterialRecord = MaterialRecord {
    name: "melon",
    block_id: 361,
    state_id: 8_333,
    item_id: 437,
    hardness: 1.0,
    resistance: 1.0,
    correct_tool_required: false,
};

pub const MUD: MaterialRecord = MaterialRecord {
    name: "mud",
    block_id: 1_150,
    state_id: 30_415,
    item_id: 59,
    hardness: 0.5,
    resistance: 0.5,
    correct_tool_required: false,
};

pub const NETHERRACK: MaterialRecord = MaterialRecord {
    name: "netherrack",
    block_id: 285,
    state_id: 6_997,
    item_id: 387,
    hardness: 0.4,
    resistance: 0.4,
    correct_tool_required: true,
};

pub const PACKED_ICE: MaterialRecord = MaterialRecord {
    name: "packed_ice",
    block_id: 556,
    state_id: 12_914,
    item_id: 550,
    hardness: 0.5,
    resistance: 0.5,
    correct_tool_required: false,
};

pub const PUMPKIN: MaterialRecord = MaterialRecord {
    name: "pumpkin",
    block_id: 360,
    state_id: 8_332,
    item_id: 384,
    hardness: 1.0,
    resistance: 1.0,
    correct_tool_required: false,
};

pub const SMOOTH_STONE: MaterialRecord = MaterialRecord {
    name: "smooth_stone",
    block_id: 624,
    state_id: 13_480,
    item_id: 331,
    hardness: 2.0,
    resistance: 6.0,
    correct_tool_required: true,
};

pub const BLACKSTONE: [MaterialRecord; 6] = [
    material("blackstone", 924, 21_831, 1_416, 1.5, 6.0, true),
    material("polished_blackstone", 928, 22_242, 1_420, 2.0, 6.0, true),
    material(
        "polished_blackstone_bricks",
        929,
        22_243,
        1_424,
        1.5,
        6.0,
        true,
    ),
    material(
        "chiseled_polished_blackstone",
        931,
        22_245,
        1_423,
        1.5,
        6.0,
        true,
    ),
    material(
        "cracked_polished_blackstone_bricks",
        930,
        22_244,
        1_427,
        1.5,
        6.0,
        true,
    ),
    material("gilded_blackstone", 935, 22_656, 1_419, 1.5, 6.0, true),
];

pub const END_STONE: [MaterialRecord; 2] = [
    material("end_stone", 393, 9_477, 463, 3.0, 9.0, true),
    material("end_stone_bricks", 661, 14_796, 464, 3.0, 9.0, true),
];

pub const NETHER_BRICKS: [MaterialRecord; 3] = [
    material("nether_bricks", 381, 9_334, 452, 2.0, 6.0, true),
    material("cracked_nether_bricks", 942, 23_094, 453, 2.0, 6.0, true),
    material("chiseled_nether_bricks", 941, 23_093, 454, 2.0, 6.0, true),
];

pub const NETHER_PLANKS: [MaterialRecord; 2] = [
    material("crimson_planks", 883, 21_032, 73, 2.0, 3.0, false),
    material("warped_planks", 884, 21_033, 74, 2.0, 3.0, false),
];

pub const PRISMARINE: [MaterialRecord; 3] = [
    material("prismarine", 527, 12_631, 590, 1.5, 6.0, true),
    material("prismarine_bricks", 528, 12_632, 591, 1.5, 6.0, true),
    material("dark_prismarine", 529, 12_633, 592, 1.5, 6.0, true),
];

pub const RESIN: [MaterialRecord; 3] = [
    material("resin_block", 375, 8_921, 441, 0.0, 0.0, false),
    material("resin_bricks", 376, 8_922, 442, 1.5, 6.0, true),
    material("chiseled_resin_bricks", 380, 9_333, 446, 1.5, 6.0, true),
];

pub const TUFF: [MaterialRecord; 5] = [
    material("tuff", 984, 23_452, 12, 1.5, 6.0, true),
    material("polished_tuff", 988, 23_863, 17, 1.5, 6.0, true),
    material("tuff_bricks", 993, 24_275, 21, 1.5, 6.0, true),
    material("chiseled_tuff", 992, 24_274, 16, 1.5, 6.0, true),
    material("chiseled_tuff_bricks", 997, 24_686, 25, 1.5, 6.0, true),
];

pub const SULFUR_CINNABAR: [MaterialRecord; 8] = [
    material("sulfur", 998, 24_687, 26, 1.5, 6.0, true),
    material("polished_sulfur", 1_003, 25_103, 31, 1.5, 6.0, true),
    material("sulfur_bricks", 1_007, 25_514, 35, 1.5, 6.0, true),
    material("chiseled_sulfur", 1_011, 25_925, 39, 1.5, 6.0, true),
    material("cinnabar", 1_012, 25_926, 40, 1.5, 6.0, true),
    material("polished_cinnabar", 1_016, 26_337, 44, 1.5, 6.0, true),
    material("cinnabar_bricks", 1_020, 26_748, 48, 1.5, 6.0, true),
    material("chiseled_cinnabar", 1_024, 27_159, 52, 1.5, 6.0, true),
];

pub const WORKSTATION_TABLES: [MaterialRecord; 2] = [
    material("crafting_table", 206, 5_310, 360, 2.5, 2.5, false),
    material("fletching_table", 843, 20_771, 1_389, 2.5, 2.5, false),
];

pub fn muddy_mangrove_roots_state(axis: Axis) -> u32 {
    match axis {
        Axis::X => 165,
        Axis::Y => 166,
        Axis::Z => 167,
    }
}

const fn material(
    name: &'static str,
    block_id: u16,
    state_id: u32,
    item_id: u16,
    hardness: f32,
    resistance: f32,
    correct_tool_required: bool,
) -> MaterialRecord {
    MaterialRecord {
        name,
        block_id,
        state_id,
        item_id,
        hardness,
        resistance,
        correct_tool_required,
    }
}

pub fn gilded_blackstone_drop(
    silk_touch: bool,
    survives_explosion: bool,
    fortune_level: u8,
    bonus_draw_succeeds: bool,
    nugget_count_2_to_5: u8,
) -> GildedDrop {
    if silk_touch {
        return GildedDrop::Block;
    }
    if !survives_explosion {
        return GildedDrop::Nothing;
    }
    let automatic = fortune_level >= 3;
    if automatic || bonus_draw_succeeds {
        GildedDrop::GoldNuggets(nugget_count_2_to_5.clamp(2, 5))
    } else {
        GildedDrop::Block
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GildedDrop {
    Nothing,
    Block,
    GoldNuggets(u8),
}

pub fn clay_ball_drop(silk_touch: bool, explosion_survivals: &[bool]) -> ClayDrop {
    if silk_touch {
        ClayDrop::Block
    } else {
        ClayDrop::Balls(
            explosion_survivals
                .iter()
                .take(4)
                .filter(|survives| **survives)
                .count() as u8,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClayDrop {
    Block,
    Balls(u8),
}

pub fn ancient_debris_item_resists_fire(damage_type_index: u8) -> bool {
    damage_type_index < 8
}

pub fn crafting_table_use(server_side: bool) -> TableUse {
    TableUse {
        result_success: true,
        open_menu: server_side,
        award_stat: server_side,
        menu_type_id: 12,
        slot_count: 46,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableUse {
    pub result_success: bool,
    pub open_menu: bool,
    pub award_stat: bool,
    pub menu_type_id: u8,
    pub slot_count: u8,
}
