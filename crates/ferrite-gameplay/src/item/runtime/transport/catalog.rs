//! Closed 26.2 boat, raft, harness, recipe, fuel, and trade identities.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatGeometry {
    Boat,
    Raft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatFamily {
    pub material: &'static str,
    pub plank: &'static str,
    pub ordinary_item: &'static str,
    pub ordinary_item_id: u32,
    pub ordinary_entity: &'static str,
    pub ordinary_entity_id: u32,
    pub chest_item: &'static str,
    pub chest_item_id: u32,
    pub chest_entity: &'static str,
    pub chest_entity_id: u32,
    pub geometry: BoatGeometry,
}

pub const BOAT_FAMILIES: [BoatFamily; 10] = [
    boat_family(
        ["oak", "oak_planks", "oak_boat", "oak_chest_boat"],
        [891, 89, 892, 90],
    ),
    boat_family(
        [
            "spruce",
            "spruce_planks",
            "spruce_boat",
            "spruce_chest_boat",
        ],
        [893, 125, 894, 126],
    ),
    boat_family(
        ["birch", "birch_planks", "birch_boat", "birch_chest_boat"],
        [895, 12, 896, 13],
    ),
    boat_family(
        [
            "jungle",
            "jungle_planks",
            "jungle_boat",
            "jungle_chest_boat",
        ],
        [897, 74, 898, 75],
    ),
    boat_family(
        [
            "acacia",
            "acacia_planks",
            "acacia_boat",
            "acacia_chest_boat",
        ],
        [899, 0, 900, 1],
    ),
    boat_family(
        [
            "cherry",
            "cherry_planks",
            "cherry_boat",
            "cherry_chest_boat",
        ],
        [901, 23, 902, 24],
    ),
    boat_family(
        [
            "dark_oak",
            "dark_oak_planks",
            "dark_oak_boat",
            "dark_oak_chest_boat",
        ],
        [903, 33, 904, 34],
    ),
    boat_family(
        [
            "pale_oak",
            "pale_oak_planks",
            "pale_oak_boat",
            "pale_oak_chest_boat",
        ],
        [905, 94, 906, 95],
    ),
    boat_family(
        [
            "mangrove",
            "mangrove_planks",
            "mangrove_boat",
            "mangrove_chest_boat",
        ],
        [907, 81, 908, 82],
    ),
    BoatFamily {
        material: "bamboo",
        plank: "bamboo_planks",
        ordinary_item: "bamboo_raft",
        ordinary_item_id: 909,
        ordinary_entity: "bamboo_raft",
        ordinary_entity_id: 9,
        chest_item: "bamboo_chest_raft",
        chest_item_id: 910,
        chest_entity: "bamboo_chest_raft",
        chest_entity_id: 8,
        geometry: BoatGeometry::Raft,
    },
];

const fn boat_family(names: [&'static str; 4], ids: [u32; 4]) -> BoatFamily {
    BoatFamily {
        material: names[0],
        plank: names[1],
        ordinary_item: names[2],
        ordinary_item_id: ids[0],
        ordinary_entity: names[2],
        ordinary_entity_id: ids[1],
        chest_item: names[3],
        chest_item_id: ids[2],
        chest_entity: names[3],
        chest_entity_id: ids[3],
        geometry: BoatGeometry::Boat,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatIdentity {
    pub item: &'static str,
    pub item_id: u32,
    pub entity: &'static str,
    pub entity_id: u32,
    pub family_index: usize,
    pub chest: bool,
}

pub fn boat_identity(item: &str) -> Option<BoatIdentity> {
    BOAT_FAMILIES
        .iter()
        .enumerate()
        .find_map(|(family_index, family)| {
            if item == family.ordinary_item {
                Some(BoatIdentity {
                    item: family.ordinary_item,
                    item_id: family.ordinary_item_id,
                    entity: family.ordinary_entity,
                    entity_id: family.ordinary_entity_id,
                    family_index,
                    chest: false,
                })
            } else if item == family.chest_item {
                Some(BoatIdentity {
                    item: family.chest_item,
                    item_id: family.chest_item_id,
                    entity: family.chest_entity,
                    entity_id: family.chest_entity_id,
                    family_index,
                    chest: true,
                })
            } else {
                None
            }
        })
}

pub const BOAT_FUEL_TICKS: u32 = 1_200;
pub const BOAT_MAXIMUM_STACK: u16 = 1;
pub const HARNESS_MAXIMUM_STACK: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FishermanBoatTrade {
    pub villager_types: &'static [&'static str],
    pub boat: &'static str,
    pub emeralds: u8,
    pub boats: u8,
    pub maximum_uses: u8,
    pub villager_experience: u8,
    pub reputation_discount: f32,
}

pub const FISHERMAN_BOAT_TRADES: [FishermanBoatTrade; 5] = [
    trade(&["plains"], "oak_boat"),
    trade(&["taiga", "snow"], "spruce_boat"),
    trade(&["desert", "jungle"], "jungle_boat"),
    trade(&["savanna"], "acacia_boat"),
    trade(&["swamp"], "dark_oak_boat"),
];

const fn trade(villager_types: &'static [&'static str], boat: &'static str) -> FishermanBoatTrade {
    FishermanBoatTrade {
        villager_types,
        boat,
        emeralds: 1,
        boats: 1,
        maximum_uses: 12,
        villager_experience: 30,
        reputation_discount: 0.05,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarnessIdentity {
    pub color: &'static str,
    pub item: &'static str,
    pub item_id: u32,
    pub asset: &'static str,
}

pub const HARNESSES: [HarnessIdentity; 16] = [
    harness("white", "white_harness", 866),
    harness("orange", "orange_harness", 867),
    harness("magenta", "magenta_harness", 868),
    harness("light_blue", "light_blue_harness", 869),
    harness("yellow", "yellow_harness", 870),
    harness("lime", "lime_harness", 871),
    harness("pink", "pink_harness", 872),
    harness("gray", "gray_harness", 873),
    harness("light_gray", "light_gray_harness", 874),
    harness("cyan", "cyan_harness", 875),
    harness("purple", "purple_harness", 876),
    harness("blue", "blue_harness", 877),
    harness("brown", "brown_harness", 878),
    harness("green", "green_harness", 879),
    harness("red", "red_harness", 880),
    harness("black", "black_harness", 881),
];

const fn harness(color: &'static str, item: &'static str, item_id: u32) -> HarnessIdentity {
    HarnessIdentity {
        color,
        item,
        item_id,
        asset: item,
    }
}

pub fn harness_identity(item: &str) -> Option<HarnessIdentity> {
    HARNESSES
        .iter()
        .copied()
        .find(|identity| identity.item == item)
}
