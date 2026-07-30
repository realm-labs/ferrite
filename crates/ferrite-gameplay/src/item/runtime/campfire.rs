//! Four-slot campfire placement, cooking, reload fallback, and cooldown.

use crate::item::runtime::stack::ItemStack;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Campfire {
    pub slots: [ItemStack; 4],
    pub progress: [u32; 4],
    pub total: [u32; 4],
}

impl Campfire {
    pub fn empty() -> Self {
        Self {
            slots: std::array::from_fn(|_| ItemStack::empty()),
            progress: [0; 4],
            total: [0; 4],
        }
    }

    pub fn place_food(
        &mut self,
        hand: &mut ItemStack,
        synchronized_input: bool,
        current_recipe_time: Option<u32>,
        infinite_materials: bool,
    ) -> Option<usize> {
        if !synchronized_input {
            return None;
        }
        let slot = self.slots.iter().position(ItemStack::is_empty)?;
        let cooking_time = current_recipe_time?;
        let mut accepted = hand.clone();
        accepted.count = 1;
        self.slots[slot] = accepted;
        self.progress[slot] = 0;
        self.total[slot] = cooking_time;
        if !infinite_materials {
            hand.shrink(1);
        }
        Some(slot)
    }

    pub fn tick_lit(
        &mut self,
        resolved_outputs: [Option<ItemStack>; 4],
        output_enabled: [bool; 4],
    ) -> Vec<CampfireCompletion> {
        let mut completed = Vec::new();
        for index in 0..self.slots.len() {
            if self.slots[index].is_empty() {
                continue;
            }
            self.progress[index] += 1;
            if self.progress[index] < self.total[index] {
                continue;
            }
            let output = resolved_outputs[index]
                .clone()
                .unwrap_or_else(|| self.slots[index].clone());
            if !output_enabled[index] {
                continue;
            }
            completed.push(CampfireCompletion {
                slot: index,
                output,
            });
            self.slots[index] = ItemStack::empty();
        }
        completed
    }

    pub fn tick_unlit(&mut self) {
        for index in 0..self.slots.len() {
            self.progress[index] = self.progress[index]
                .saturating_sub(2)
                .min(self.total[index]);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampfireCompletion {
    pub slot: usize,
    pub output: ItemStack,
}
