//! Dried Kelp food, composter, fuel, fire, recipe, and trade joins.

pub const DRIED_KELP_ITEM_ID: u32 = 1_136;
pub const DRIED_KELP_BLOCK_ID: u32 = 744;
pub const DRIED_KELP_BLOCK_ITEM_ID: u32 = 1_056;
pub const DRIED_KELP_BLOCK_STATE_ID: u32 = 15_089;
pub const CONSUME_TICKS: u32 = 16;
pub const NUTRITION: u8 = 1;
pub const SATURATION: f32 = 0.6;
pub const DRIED_KELP_COMPOST_CHANCE: f64 = 0.3;
pub const BLOCK_COMPOST_CHANCE: f64 = 0.5;
pub const BLOCK_FUEL_TICKS: u32 = 4_001;
pub const BLOCK_IGNITE_ODDS: u8 = 30;
pub const BLOCK_BURN_ODDS: u8 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AirUse {
    Fail,
    Pass,
    BeginConsumption,
}

pub const fn air_use(
    has_food_component: bool,
    has_consumable_component: bool,
    hunger_full: bool,
) -> AirUse {
    if !has_consumable_component {
        return AirUse::Pass;
    }
    if has_food_component && hunger_full {
        AirUse::Fail
    } else {
        AirUse::BeginConsumption
    }
}

pub const fn composter_probability(compacted_block: bool) -> f64 {
    if compacted_block {
        BLOCK_COMPOST_CHANCE
    } else {
        DRIED_KELP_COMPOST_CHANCE
    }
}

pub fn composter_succeeds(level: u8, compacted_block: bool, draw: f64) -> Option<bool> {
    match level {
        0 => Some(true),
        1..=6 if draw.is_finite() && (0.0..1.0).contains(&draw) => {
            Some(draw < composter_probability(compacted_block))
        }
        1..=6 => None,
        _ => Some(false),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CookingRecord {
    pub id: &'static str,
    pub ticks: u32,
    pub experience: f32,
}

pub const COOKING_RECORDS: [CookingRecord; 3] = [
    CookingRecord {
        id: "dried_kelp_from_smelting",
        ticks: 200,
        experience: 0.1,
    },
    CookingRecord {
        id: "dried_kelp_from_smoking",
        ticks: 100,
        experience: 0.1,
    },
    CookingRecord {
        id: "dried_kelp_from_campfire_cooking",
        ticks: 600,
        experience: 0.1,
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriedKelpTrade {
    pub block_cost: u8,
    pub emerald_output: u8,
    pub maximum_uses: u8,
    pub experience: u8,
    pub discount: f32,
}

pub const BUTCHER_TRADE: DriedKelpTrade = DriedKelpTrade {
    block_cost: 10,
    emerald_output: 1,
    maximum_uses: 12,
    experience: 30,
    discount: 0.05,
};
