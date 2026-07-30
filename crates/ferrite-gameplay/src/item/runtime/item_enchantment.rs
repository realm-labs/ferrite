//! Enchanted-stack representation shared by Grindstone and Anvil transactions.

use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedEnchantment {
    pub key: ResourceId,
    pub level: u8,
    pub curse: bool,
    pub minimum_cost: u32,
    pub anvil_cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantedStack {
    pub stack: ItemStack,
    pub maximum_damage: u32,
    pub damage: u32,
    pub enchantments: Vec<AppliedEnchantment>,
    pub stored_enchantments: bool,
    pub repair_cost: u32,
    pub custom_name: Option<String>,
}

impl EnchantedStack {
    pub fn is_damageable(&self) -> bool {
        self.maximum_damage > 0
    }
}
