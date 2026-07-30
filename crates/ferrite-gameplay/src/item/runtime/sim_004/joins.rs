//! Exact source/sink cardinalities and shared trade contracts.

use crate::item::runtime::sim_004::catalog::MaterialItem;

pub const STRUCTURE_TEMPLATE_COUNT: usize = 1_212;
pub const ARMOR_TRIM_RECIPE_COUNT: u16 = 18;
pub const ARMOR_TRIM_MODEL_COUNT: u16 = 29;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinProfile {
    pub recipes: u16,
    pub advancements: u16,
    pub direct_unlocks: u16,
    pub non_block_acquisition_tables: u16,
    pub brewing_edges: u16,
    pub trim_recipes: u16,
    pub reloadable_trade_records: u16,
    pub stored_template_offers: u8,
    pub exact_template_matches: u8,
}

pub const fn profile(item: MaterialItem) -> JoinProfile {
    match item {
        MaterialItem::Diamond => joins(recipe(56, 55, 12), 17, 0, 18, 13, template(0, 0)),
        MaterialItem::DriedKelp => joins(recipe(5, 5, 5), 0, 0, 0, 1, template(0, 0)),
        MaterialItem::Emerald => joins(recipe(24, 24, 1), 32, 0, 18, 469, template(2, 2)),
        MaterialItem::Feather => joins(recipe(4, 3, 1), 6, 0, 0, 1, template(0, 0)),
        MaterialItem::FireworkStar => joins(recipe(3, 0, 0), 0, 0, 0, 0, template(0, 0)),
        MaterialItem::Flint => joins(recipe(3, 3, 3), 3, 0, 0, 5, template(0, 0)),
        MaterialItem::GlowstoneDust => joins(recipe(3, 2, 2), 2, 10, 0, 0, template(0, 0)),
        MaterialItem::Gunpowder => joins(recipe(5, 2, 2), 8, 1, 0, 1, template(0, 0)),
        MaterialItem::LapisLazuli => joins(recipe(25, 25, 2), 4, 0, 18, 1, template(0, 0)),
        MaterialItem::Leather => joins(recipe(26, 26, 7), 19, 0, 0, 1, template(0, 0)),
        MaterialItem::Quartz => joins(recipe(26, 25, 7), 2, 0, 18, 1, template(0, 0)),
        MaterialItem::Redstone => joins(recipe(46, 46, 8), 8, 14, 18, 1, template(0, 0)),
        MaterialItem::SlimeBall => joins(recipe(4, 4, 2), 2, 0, 0, 1, template(0, 0)),
        MaterialItem::Stick => joins(recipe(111, 111, 8), 20, 0, 0, 1, template(0, 0)),
        MaterialItem::TurtleScute => joins(recipe(1, 1, 1), 1, 1, 0, 2, template(0, 0)),
    }
}

#[derive(Debug, Clone, Copy)]
struct RecipeCounts {
    recipes: u16,
    advancements: u16,
    direct_unlocks: u16,
}

const fn recipe(recipes: u16, advancements: u16, direct_unlocks: u16) -> RecipeCounts {
    RecipeCounts {
        recipes,
        advancements,
        direct_unlocks,
    }
}

#[derive(Debug, Clone, Copy)]
struct TemplateCounts {
    stored_offers: u8,
    exact_matches: u8,
}

const fn template(stored_offers: u8, exact_matches: u8) -> TemplateCounts {
    TemplateCounts {
        stored_offers,
        exact_matches,
    }
}

const fn joins(
    recipe: RecipeCounts,
    non_block_acquisition_tables: u16,
    brewing_edges: u16,
    trim_recipes: u16,
    reloadable_trade_records: u16,
    template: TemplateCounts,
) -> JoinProfile {
    JoinProfile {
        recipes: recipe.recipes,
        advancements: recipe.advancements,
        direct_unlocks: recipe.direct_unlocks,
        non_block_acquisition_tables,
        brewing_edges,
        trim_recipes,
        reloadable_trade_records,
        stored_template_offers: template.stored_offers,
        exact_template_matches: template.exact_matches,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TradeProfile {
    pub profession: &'static str,
    pub level: u8,
    pub first_cost: u8,
    pub second_cost: u8,
    pub output_count: u8,
    pub inclusion_probability: f32,
    pub maximum_uses: u8,
    pub experience: u8,
    pub discount: f32,
}

pub const FLINT_TRADES: [TradeProfile; 5] = [
    trade(
        "fletcher",
        1,
        exchange(10, 1, 10),
        2.0 / 3.0,
        economy(12, 1),
    ),
    trade("fletcher", 2, exchange(26, 0, 1), 1.0, economy(12, 10)),
    trade(
        "leatherworker",
        2,
        exchange(26, 0, 1),
        2.0 / 3.0,
        economy(12, 10),
    ),
    trade(
        "toolsmith",
        3,
        exchange(30, 0, 1),
        2.0 / 5.0,
        economy(12, 20),
    ),
    trade("weaponsmith", 3, exchange(24, 0, 1), 1.0, economy(12, 20)),
];

pub const TURTLE_SCUTE_TRADES: [TradeProfile; 2] = [
    trade("leatherworker", 4, exchange(4, 0, 1), 1.0, economy(12, 30)),
    trade("cleric", 4, exchange(4, 0, 1), 2.0 / 3.0, economy(12, 30)),
];

#[derive(Debug, Clone, Copy)]
struct Exchange {
    first_cost: u8,
    second_cost: u8,
    output_count: u8,
}

const fn exchange(first_cost: u8, second_cost: u8, output_count: u8) -> Exchange {
    Exchange {
        first_cost,
        second_cost,
        output_count,
    }
}

#[derive(Debug, Clone, Copy)]
struct Economy {
    maximum_uses: u8,
    experience: u8,
}

const fn economy(maximum_uses: u8, experience: u8) -> Economy {
    Economy {
        maximum_uses,
        experience,
    }
}

const fn trade(
    profession: &'static str,
    level: u8,
    exchange: Exchange,
    inclusion_probability: f32,
    economy: Economy,
) -> TradeProfile {
    TradeProfile {
        profession,
        level,
        first_cost: exchange.first_cost,
        second_cost: exchange.second_cost,
        output_count: exchange.output_count,
        inclusion_probability,
        maximum_uses: economy.maximum_uses,
        experience: economy.experience,
        discount: 0.05,
    }
}
