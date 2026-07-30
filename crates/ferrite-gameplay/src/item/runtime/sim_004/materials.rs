//! Live material roles, repair admission, and identity-specific consumers.

use crate::item::runtime::sim_004::catalog::MaterialItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialRole {
    BeaconPayment,
    TrimMaterial,
    DiamondToolMaterial,
    DiamondArmorRepair,
    LeatherArmorRepair,
    IgnoredByPiglinBabies,
    FrogFood,
    SulfurCubeFood,
    TurtleHelmetRepair,
}

pub const fn has_default_role(item: MaterialItem, role: MaterialRole) -> bool {
    match role {
        MaterialRole::BeaconPayment => {
            matches!(item, MaterialItem::Diamond | MaterialItem::Emerald)
        }
        MaterialRole::TrimMaterial => matches!(
            item,
            MaterialItem::Diamond
                | MaterialItem::Emerald
                | MaterialItem::LapisLazuli
                | MaterialItem::Quartz
                | MaterialItem::Redstone
        ),
        MaterialRole::DiamondToolMaterial | MaterialRole::DiamondArmorRepair => {
            matches!(item, MaterialItem::Diamond)
        }
        MaterialRole::LeatherArmorRepair | MaterialRole::IgnoredByPiglinBabies => {
            matches!(item, MaterialItem::Leather)
        }
        MaterialRole::FrogFood | MaterialRole::SulfurCubeFood => {
            matches!(item, MaterialItem::SlimeBall)
        }
        MaterialRole::TurtleHelmetRepair => matches!(item, MaterialItem::TurtleScute),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimProfile {
    pub holder: &'static str,
    pub rgb: u32,
    pub asset: &'static str,
    pub self_equipment_asset: Option<&'static str>,
}

pub const fn trim_profile(item: MaterialItem) -> Option<TrimProfile> {
    match item {
        MaterialItem::Diamond => Some(TrimProfile {
            holder: "minecraft:diamond",
            rgb: 0x006e_ecd2,
            asset: "diamond",
            self_equipment_asset: Some("diamond_darker"),
        }),
        MaterialItem::Emerald => Some(TrimProfile {
            holder: "minecraft:emerald",
            rgb: 0x0011_a036,
            asset: "emerald",
            self_equipment_asset: None,
        }),
        MaterialItem::LapisLazuli => Some(TrimProfile {
            holder: "minecraft:lapis",
            rgb: 0x0041_6e97,
            asset: "lapis",
            self_equipment_asset: None,
        }),
        MaterialItem::Quartz => Some(TrimProfile {
            holder: "minecraft:quartz",
            rgb: 0x00e3_d4c4,
            asset: "quartz",
            self_equipment_asset: None,
        }),
        MaterialItem::Redstone => Some(TrimProfile {
            holder: "minecraft:redstone",
            rgb: 0x0097_1607,
            asset: "redstone",
            self_equipment_asset: None,
        }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairTarget {
    DiamondTool,
    DiamondHumanoidArmor,
    DiamondHorseArmor,
    DiamondNautilusArmor,
    LeatherHumanoidArmor,
    LeatherHorseArmor,
    TurtleHelmet,
}

pub const fn repairs(
    item: MaterialItem,
    target: RepairTarget,
    live_tag_contains_item: bool,
) -> bool {
    if !live_tag_contains_item {
        return false;
    }
    matches!(
        (item, target),
        (
            MaterialItem::Diamond,
            RepairTarget::DiamondTool | RepairTarget::DiamondHumanoidArmor
        ) | (MaterialItem::Leather, RepairTarget::LeatherHumanoidArmor)
            | (MaterialItem::TurtleScute, RepairTarget::TurtleHelmet)
    )
}

pub const fn lapis_enchantment_consumption(
    option_index: u8,
    creative: bool,
    available_lapis: u8,
    input_valid: bool,
    displayed_cost_positive: bool,
    experience_sufficient: bool,
) -> Option<u8> {
    if option_index > 2 || !input_valid || !displayed_cost_positive {
        return None;
    }
    if creative {
        return Some(0);
    }
    if !experience_sufficient {
        return None;
    }
    let required = option_index + 1;
    if available_lapis < required {
        None
    } else {
        Some(required)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlimeBallFoodTarget {
    Frog,
    SulfurCube { baby: bool },
}

pub const fn slime_ball_feeding_admitted(
    target: SlimeBallFoodTarget,
    live_food_tag_contains_ball: bool,
) -> bool {
    live_food_tag_contains_ball
        && match target {
            SlimeBallFoodTarget::Frog => true,
            SlimeBallFoodTarget::SulfurCube { baby } => baby,
        }
}

pub const fn leather_pickup_admitted_by_age(
    piglin_is_baby: bool,
    ignored_by_babies_tag_contains_leather: bool,
) -> bool {
    !(piglin_is_baby && ignored_by_babies_tag_contains_leather)
}

pub fn cat_feather_probability(gift_chance: f32) -> Option<f32> {
    if gift_chance.is_finite() && (0.0..=1.0).contains(&gift_chance) {
        Some(gift_chance * 5.0 / 31.0)
    } else {
        None
    }
}

pub const fn fishing_junk_denominator(jungle_bamboo_admitted: bool) -> u8 {
    if jungle_bamboo_admitted { 110 } else { 100 }
}

pub const fn furnace_burn_ticks(item: MaterialItem) -> u32 {
    match item {
        MaterialItem::Stick => 100,
        _ => 0,
    }
}
