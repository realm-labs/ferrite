//! Manual crafting preview/take and redstone Crafter delivery.

use crate::item::runtime::container_lifecycle::dispose_stack;
use crate::item::runtime::inventory::{Inventory, move_item_stack_to};
use crate::item::runtime::recipe::{PositionedCraftInput, RecipeRecord};
use crate::item::runtime::stack::ItemStack;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftingGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<ItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftingPreview {
    pub result: ItemStack,
    pub stored_recipe: Option<String>,
    pub state_id: u16,
}

impl CraftingPreview {
    pub fn empty() -> Self {
        Self {
            result: ItemStack::empty(),
            stored_recipe: None,
            state_id: 0,
        }
    }

    pub fn recompute(
        &mut self,
        recipe: Option<&RecipeRecord>,
        limited_crafting: bool,
        unlocked: bool,
        result_enabled: bool,
    ) {
        let Some(recipe) = recipe else {
            self.clear();
            return;
        };
        if (!recipe.special && limited_crafting && !unlocked) || !result_enabled {
            self.clear();
            return;
        }
        self.result = recipe.result.clone();
        self.stored_recipe = Some(recipe.key.clone());
        self.state_id = self.state_id.wrapping_add(1) & 32_767;
    }

    fn clear(&mut self) {
        self.result = ItemStack::empty();
        self.stored_recipe = None;
        self.state_id = self.state_id.wrapping_add(1) & 32_767;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftTakeOutcome {
    pub credited_recipe: Option<String>,
    pub dropped_remainders: Vec<ItemStack>,
    pub consumed_cells: usize,
}

pub fn take_crafting_result(
    preview: &mut CraftingPreview,
    grid: &mut CraftingGrid,
    current_input: &PositionedCraftInput,
    remainders: &[ItemStack],
    player_inventory: &mut Inventory,
) -> CraftTakeOutcome {
    let credited_recipe = preview.stored_recipe.take();
    preview.result = ItemStack::empty();
    let mut dropped_remainders = Vec::new();
    let mut consumed_cells = 0;

    for row in 0..current_input.height {
        for column in 0..current_input.width {
            let cropped_index = row * current_input.width + column;
            let grid_index = (current_input.top + row) * grid.width + current_input.left + column;
            let Some(cell) = grid.cells.get_mut(grid_index) else {
                continue;
            };
            if !cell.is_empty() {
                cell.shrink(1);
                consumed_cells += 1;
            }
            let Some(remainder) = remainders.get(cropped_index) else {
                continue;
            };
            if remainder.is_empty() {
                continue;
            }
            let mut remainder = remainder.clone();
            if cell.is_empty() {
                *cell = remainder;
            } else if cell.compatible_with(&remainder) {
                remainder.grow(cell.count);
                *cell = remainder;
            } else {
                dispose_stack(
                    &mut remainder,
                    player_inventory,
                    true,
                    &mut dropped_remainders,
                );
            }
        }
    }
    CraftTakeOutcome {
        credited_recipe,
        dropped_remainders,
        consumed_cells,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrafterTrigger {
    pub schedule_after: Option<u32>,
    pub triggered: bool,
    pub crafting: bool,
}

pub const fn crafter_power_transition(powered: bool, triggered: bool) -> CrafterTrigger {
    if powered && !triggered {
        CrafterTrigger {
            schedule_after: Some(4),
            triggered: true,
            crafting: false,
        }
    } else if !powered {
        CrafterTrigger {
            schedule_after: None,
            triggered: false,
            crafting: false,
        }
    } else {
        CrafterTrigger {
            schedule_after: None,
            triggered,
            crafting: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crafter {
    pub slots: [ItemStack; 9],
    pub disabled: [bool; 9],
    pub animation_ticks: u8,
    pub crafting_state: bool,
}

impl Crafter {
    pub fn empty() -> Self {
        Self {
            slots: std::array::from_fn(|_| ItemStack::empty()),
            disabled: [false; 9],
            animation_ticks: 0,
            crafting_state: false,
        }
    }

    pub fn can_place_item(&self, slot: usize, incoming: &ItemStack) -> bool {
        if slot >= self.slots.len()
            || self.disabled[slot]
            || self.slots[slot].count >= self.slots[slot].maximum
        {
            return false;
        }
        for later in slot + 1..self.slots.len() {
            if self.disabled[later] {
                continue;
            }
            if self.slots[later].is_empty()
                || (self.slots[later].compatible_with(incoming)
                    && self.slots[later].count < self.slots[slot].count)
            {
                return false;
            }
        }
        true
    }

    pub fn craft(
        &mut self,
        result: ItemStack,
        remainders: &[ItemStack],
        destination: &mut Inventory,
        destination_is_crafter: bool,
    ) -> CrafterOutcome {
        if result.is_empty() {
            return CrafterOutcome {
                failed: true,
                ..CrafterOutcome::default()
            };
        }
        self.animation_ticks = 6;
        self.crafting_state = true;
        let mut deliveries = Vec::with_capacity(1 + remainders.len());
        deliveries.push(result);
        deliveries.extend(remainders.iter().filter(|stack| !stack.is_empty()).cloned());
        let mut residue = Vec::new();
        for mut delivery in deliveries {
            if destination_is_crafter
                || delivery.count
                    > destination
                        .slots
                        .iter()
                        .map(|slot| slot.maximum_for(&delivery))
                        .max()
                        .unwrap_or(0)
            {
                while !delivery.is_empty() {
                    let mut one = delivery.split(1, delivery.identity);
                    let slot_count = destination.slots.len();
                    move_item_stack_to(&mut one, &mut destination.slots, 0..slot_count, false);
                    if !one.is_empty() {
                        residue.push(one);
                    }
                }
            } else {
                let slot_count = destination.slots.len();
                move_item_stack_to(&mut delivery, &mut destination.slots, 0..slot_count, false);
                if !delivery.is_empty() {
                    residue.push(delivery);
                }
            }
        }
        for slot in &mut self.slots {
            if !slot.is_empty() {
                slot.shrink(1);
            }
        }
        CrafterOutcome {
            failed: false,
            emitted_residue_events: !residue.is_empty(),
            residue,
        }
    }

    pub fn tick_animation(&mut self) {
        self.animation_ticks = self.animation_ticks.saturating_sub(1);
        if self.animation_ticks == 0 {
            self.crafting_state = false;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CrafterOutcome {
    pub failed: bool,
    pub emitted_residue_events: bool,
    pub residue: Vec<ItemStack>,
}
