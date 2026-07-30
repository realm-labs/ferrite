//! Live tag and component dispatch for audited material identities.

use crate::item::runtime::catalog::ItemKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRole {
    BeaconPayment,
    Coal,
    DuplicatesAllays,
    FurnaceMinecartFuel,
    HorseFood,
    HorseTempt,
    MetalNugget,
    PiglinLoved,
    ToolMaterial,
    ArmorRepair,
    TrimMaterial,
}

pub const fn has_role(item: ItemKind, role: ItemRole) -> bool {
    match role {
        ItemRole::BeaconPayment => matches!(
            item,
            ItemKind::GoldIngot | ItemKind::IronIngot | ItemKind::NetheriteIngot
        ),
        ItemRole::Coal | ItemRole::FurnaceMinecartFuel => {
            matches!(item, ItemKind::Coal | ItemKind::Charcoal)
        }
        ItemRole::DuplicatesAllays => matches!(item, ItemKind::AmethystShard),
        ItemRole::HorseFood => matches!(
            item,
            ItemKind::Apple | ItemKind::GoldenApple | ItemKind::EnchantedGoldenApple
        ),
        ItemRole::HorseTempt => {
            matches!(item, ItemKind::GoldenApple | ItemKind::EnchantedGoldenApple)
        }
        ItemRole::MetalNugget => matches!(
            item,
            ItemKind::CopperNugget | ItemKind::GoldNugget | ItemKind::IronNugget
        ),
        ItemRole::PiglinLoved => matches!(
            item,
            ItemKind::RawGold
                | ItemKind::GoldIngot
                | ItemKind::GoldenApple
                | ItemKind::EnchantedGoldenApple
        ),
        ItemRole::ToolMaterial | ItemRole::ArmorRepair | ItemRole::TrimMaterial => {
            matches!(
                item,
                ItemKind::CopperIngot
                    | ItemKind::GoldIngot
                    | ItemKind::IronIngot
                    | ItemKind::NetheriteIngot
            ) || matches!(role, ItemRole::TrimMaterial) && matches!(item, ItemKind::AmethystShard)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Copper,
    Gold,
    Iron,
    Netherite,
}

pub const fn material(item: ItemKind) -> Option<Material> {
    match item {
        ItemKind::CopperIngot => Some(Material::Copper),
        ItemKind::GoldIngot => Some(Material::Gold),
        ItemKind::IronIngot => Some(Material::Iron),
        ItemKind::NetheriteIngot => Some(Material::Netherite),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairTarget {
    Tool(Material),
    HumanoidArmor(Material),
    ChainmailArmor,
    HorseArmor(Material),
    NautilusArmor(Material),
}

pub const fn repairs(item: ItemKind, target: RepairTarget) -> bool {
    match (material(item), target) {
        (Some(source), RepairTarget::Tool(target))
        | (Some(source), RepairTarget::HumanoidArmor(target)) => same_material(source, target),
        (Some(Material::Iron), RepairTarget::ChainmailArmor) => true,
        (Some(_), RepairTarget::HorseArmor(_) | RepairTarget::NautilusArmor(_)) | (None, _) => {
            false
        }
        (
            Some(Material::Copper | Material::Gold | Material::Netherite),
            RepairTarget::ChainmailArmor,
        ) => false,
    }
}

const fn same_material(left: Material, right: Material) -> bool {
    matches!(
        (left, right),
        (Material::Copper, Material::Copper)
            | (Material::Gold, Material::Gold)
            | (Material::Iron, Material::Iron)
            | (Material::Netherite, Material::Netherite)
    )
}

pub const fn furnace_burn_ticks(item: ItemKind) -> u32 {
    match item {
        ItemKind::Coal | ItemKind::Charcoal => 1_600,
        _ => 0,
    }
}

pub const fn furnace_minecart_fuel_ticks(item: ItemKind) -> u32 {
    match item {
        ItemKind::Coal | ItemKind::Charcoal => 3_600,
        _ => 0,
    }
}
