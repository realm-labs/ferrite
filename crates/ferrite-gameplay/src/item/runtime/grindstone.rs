//! Grindstone merge, curse retention, repair-cost reset, and XP sampling.

use crate::item::runtime::item_enchantment::{AppliedEnchantment, EnchantedStack};
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;

pub fn grindstone_result(
    first: Option<&EnchantedStack>,
    second: Option<&EnchantedStack>,
) -> Option<EnchantedStack> {
    let first = first.filter(|stack| !stack.stack.is_empty())?;
    if first.stack.count > 1 || second.is_some_and(|stack| stack.stack.count > 1) {
        return None;
    }
    let mut output = first.clone();
    output.stack.count = 1;
    if let Some(second) = second.filter(|stack| !stack.stack.is_empty()) {
        if first.stack.item != second.stack.item {
            return None;
        }
        if first.is_damageable() {
            let maximum = first.maximum_damage.max(second.maximum_damage);
            let remaining = first.maximum_damage.saturating_sub(first.damage)
                + second.maximum_damage.saturating_sub(second.damage)
                + maximum * 5 / 100;
            output.maximum_damage = maximum;
            output.damage = maximum.saturating_sub(remaining.min(maximum));
        } else {
            if first.stack.maximum < 2 || !first.stack.equal_stack(&second.stack) {
                return None;
            }
            output.stack.count = 2;
        }
        for enchantment in second
            .enchantments
            .iter()
            .filter(|enchantment| enchantment.curse)
        {
            if output
                .enchantments
                .iter()
                .all(|existing| existing.key != enchantment.key)
            {
                output.enchantments.push(enchantment.clone());
            }
        }
    } else if first.enchantments.is_empty() {
        return None;
    }

    output.enchantments.retain(|enchantment| enchantment.curse);
    output.repair_cost = output
        .enchantments
        .iter()
        .fold(0_u32, |cost, _| cost.saturating_mul(2).saturating_add(1));
    if output.stored_enchantments && output.enchantments.is_empty() {
        output.stack.item = Some(minecraft("book"));
        output.stored_enchantments = false;
    }
    Some(output)
}

pub fn grindstone_experience(
    first: Option<&EnchantedStack>,
    second: Option<&EnchantedStack>,
    bounded_draw: Option<u32>,
) -> Result<u32, GrindstoneXpError> {
    let sum = first
        .into_iter()
        .chain(second)
        .flat_map(|stack| &stack.enchantments)
        .filter(|enchantment| !enchantment.curse)
        .map(|enchantment| enchantment.minimum_cost)
        .sum::<u32>();
    if sum == 0 {
        return Ok(0);
    }
    let bound = sum.div_ceil(2);
    let draw = bounded_draw.ok_or(GrindstoneXpError::MissingDraw)?;
    if draw >= bound {
        return Err(GrindstoneXpError::DrawOutOfRange { draw, bound });
    }
    Ok(bound + draw)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrindstoneXpError {
    MissingDraw,
    DrawOutOfRange { draw: u32, bound: u32 },
}

pub fn enchantment(path: &str, level: u8, curse: bool, minimum_cost: u32) -> AppliedEnchantment {
    AppliedEnchantment {
        key: minecraft(path),
        level,
        curse,
        minimum_cost,
        anvil_cost: 1,
    }
}

pub fn plain_enchanted_stack(stack: ItemStack) -> EnchantedStack {
    EnchantedStack {
        stack,
        maximum_damage: 0,
        damage: 0,
        enchantments: Vec::new(),
        stored_enchantments: false,
        repair_cost: 0,
        custom_name: None,
    }
}

fn minecraft(path: &str) -> ResourceId {
    ResourceId::minecraft(path).expect("locked enchantment identifier")
}
