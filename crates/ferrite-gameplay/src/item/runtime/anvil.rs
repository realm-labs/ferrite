//! Anvil material repair, sacrifice, enchantment, rename, cost, and damage.

use crate::item::runtime::item_enchantment::{AppliedEnchantment, EnchantedStack};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingEnchantment {
    pub enchantment: AppliedEnchantment,
    pub maximum_level: u8,
    pub can_apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnvilPreview {
    pub result: Option<EnchantedStack>,
    pub level_cost: u32,
    pub addition_consumed: u32,
    pub rename_only: bool,
}

pub fn build_anvil_preview(
    base: &EnchantedStack,
    addition: Option<&EnchantedStack>,
    addition_repairs_base: bool,
    incoming: &[IncomingEnchantment],
    compatible: impl Fn(&AppliedEnchantment, &AppliedEnchantment) -> bool,
    submitted_name: Option<&str>,
) -> AnvilPreview {
    if base.stack.is_empty() {
        return empty_preview();
    }
    let mut output = base.clone();
    let prior_work = base.repair_cost + addition.map_or(0, |stack| stack.repair_cost);
    let mut operation_cost = 0_u32;
    let mut consumed = 0_u32;
    let mut nonrename_operation = false;

    if let Some(addition) = addition.filter(|stack| !stack.stack.is_empty()) {
        if addition_repairs_base && base.is_damageable() {
            let repair_per_item = base.maximum_damage / 4;
            let mut available = addition.stack.count.max(0) as u32;
            if repair_per_item == 0 || output.damage == 0 {
                return empty_preview();
            }
            while output.damage > 0 && available > 0 {
                output.damage = output.damage.saturating_sub(repair_per_item);
                operation_cost += 1;
                consumed += 1;
                available -= 1;
            }
            nonrename_operation = consumed > 0;
        } else {
            let same_damageable = base.is_damageable() && base.stack.item == addition.stack.item;
            if same_damageable {
                let remaining = base.maximum_damage.saturating_sub(base.damage)
                    + addition.maximum_damage.saturating_sub(addition.damage)
                    + base.maximum_damage * 12 / 100;
                let repaired = base
                    .maximum_damage
                    .saturating_sub(remaining.min(base.maximum_damage));
                if repaired < output.damage {
                    output.damage = repaired;
                    operation_cost += 2;
                    nonrename_operation = true;
                }
            } else if !addition.stored_enchantments {
                return empty_preview();
            }
            let mut any_accepted = false;
            let mut any_incompatible = false;
            for incoming_enchantment in incoming {
                let conflicts = output.enchantments.iter().filter(|existing| {
                    existing.key != incoming_enchantment.enchantment.key
                        && !compatible(existing, &incoming_enchantment.enchantment)
                });
                let conflict_count = conflicts.count() as u32;
                if conflict_count > 0 {
                    operation_cost += conflict_count;
                    any_incompatible = true;
                    continue;
                }
                if !incoming_enchantment.can_apply {
                    continue;
                }
                let current = output
                    .enchantments
                    .iter()
                    .find(|existing| existing.key == incoming_enchantment.enchantment.key)
                    .map_or(0, |existing| existing.level);
                let target = if current == incoming_enchantment.enchantment.level {
                    current.saturating_add(1)
                } else {
                    current.max(incoming_enchantment.enchantment.level)
                }
                .min(incoming_enchantment.maximum_level);
                if let Some(existing) = output
                    .enchantments
                    .iter_mut()
                    .find(|existing| existing.key == incoming_enchantment.enchantment.key)
                {
                    existing.level = target;
                } else {
                    let mut applied = incoming_enchantment.enchantment.clone();
                    applied.level = target;
                    output.enchantments.push(applied);
                }
                operation_cost += incoming_enchantment.enchantment.anvil_cost * u32::from(target);
                any_accepted = true;
                nonrename_operation = true;
            }
            if !same_damageable && !any_accepted && any_incompatible {
                return empty_preview();
            }
            consumed = 1;
        }
    }

    let desired_name = submitted_name.filter(|name| name.chars().count() <= 50);
    let rename_changed = desired_name.map(str::to_owned) != base.custom_name;
    if rename_changed {
        output.custom_name = desired_name.map(str::to_owned);
        operation_cost += 1;
    }
    if operation_cost == 0 {
        return empty_preview();
    }
    let rename_only = rename_changed && !nonrename_operation;
    let mut level_cost = prior_work.saturating_add(operation_cost);
    if rename_only && level_cost >= 40 {
        level_cost = 39;
    }
    if !rename_only && level_cost >= 40 {
        return AnvilPreview {
            result: None,
            level_cost,
            addition_consumed: consumed,
            rename_only,
        };
    }
    if nonrename_operation {
        output.repair_cost = increased_repair_cost(
            base.repair_cost
                .max(addition.map_or(0, |stack| stack.repair_cost)),
        );
    }
    AnvilPreview {
        result: Some(output),
        level_cost,
        addition_consumed: consumed,
        rename_only,
    }
}

pub const fn may_take_anvil_result(
    preview: &AnvilPreview,
    experience_level: u32,
    infinite_materials: bool,
) -> bool {
    preview.result.is_some()
        && preview.level_cost > 0
        && (infinite_materials || experience_level >= preview.level_cost)
}

pub const fn increased_repair_cost(old: u32) -> u32 {
    old.saturating_mul(2).saturating_add(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnvilDamage {
    Unchanged,
    Chipped,
    Damaged,
    Destroyed,
}

pub fn damage_anvil(stage: AnvilDamage, infinite_materials: bool, draw: f32) -> AnvilDamage {
    if infinite_materials || draw >= 0.12 {
        return stage;
    }
    match stage {
        AnvilDamage::Unchanged => AnvilDamage::Chipped,
        AnvilDamage::Chipped => AnvilDamage::Damaged,
        AnvilDamage::Damaged | AnvilDamage::Destroyed => AnvilDamage::Destroyed,
    }
}

fn empty_preview() -> AnvilPreview {
    AnvilPreview {
        result: None,
        level_cost: 0,
        addition_consumed: 0,
        rename_only: false,
    }
}
