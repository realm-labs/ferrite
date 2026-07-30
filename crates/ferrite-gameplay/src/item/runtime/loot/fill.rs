//! Vanilla container slot shuffle and stack splitting after one loot generation.

use crate::item::runtime::inventory::Inventory;
use crate::item::runtime::random::{GameplayRandom, GameplayRandomError, checked_int, shuffle};
use crate::item::runtime::stack::ItemStack;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerFill {
    pub written_slots: Vec<usize>,
    pub overfill: Vec<ItemStack>,
}

pub fn fill_container(
    inventory: &mut Inventory,
    generated: Vec<ItemStack>,
    random: &mut dyn GameplayRandom,
) -> Result<ContainerFill, GameplayRandomError> {
    let mut available_slots = inventory
        .slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| slot.stack.is_empty().then_some(index))
        .collect::<Vec<_>>();
    shuffle(&mut available_slots, random)?;
    let generated = shuffle_and_split(generated, available_slots.len(), random)?;
    let mut written_slots = Vec::new();
    let mut overfill = Vec::new();
    let mut generated = generated.into_iter();
    while let Some(stack) = generated.next() {
        let Some(slot) = available_slots.pop() else {
            overfill.push(stack);
            overfill.extend(generated);
            break;
        };
        inventory.slots[slot].stack = stack;
        inventory.slots[slot].changed = true;
        written_slots.push(slot);
    }
    if !written_slots.is_empty() {
        inventory.changed_calls += 1;
    }
    Ok(ContainerFill {
        written_slots,
        overfill,
    })
}

fn shuffle_and_split(
    generated: Vec<ItemStack>,
    available_slots: usize,
    random: &mut dyn GameplayRandom,
) -> Result<Vec<ItemStack>, GameplayRandomError> {
    let mut result = Vec::new();
    let mut splittable = Vec::new();
    for stack in generated {
        if stack.is_empty() {
            continue;
        }
        if stack.count > 1 {
            splittable.push(stack);
        } else {
            result.push(stack);
        }
    }
    while available_slots.saturating_sub(result.len() + splittable.len()) > 0
        && !splittable.is_empty()
    {
        let index = checked_int(random, splittable.len() as u32)? as usize;
        let mut stack = splittable.remove(index);
        let maximum_split = stack.count / 2;
        let remove = checked_int(random, maximum_split as u32)? as i32 + 1;
        let split = stack.split(remove, stack.identity);
        retain_or_finish(stack, &mut splittable, &mut result, random);
        retain_or_finish(split, &mut splittable, &mut result, random);
    }
    result.extend(splittable);
    shuffle(&mut result, random)?;
    Ok(result)
}

fn retain_or_finish(
    stack: ItemStack,
    splittable: &mut Vec<ItemStack>,
    result: &mut Vec<ItemStack>,
    random: &mut dyn GameplayRandom,
) {
    if stack.count > 1 && random.next_bool() {
        splittable.push(stack);
    } else {
        result.push(stack);
    }
}
