//! Furnace, blast-furnace, and smoker timers, completion, and XP rounding.

use crate::item::runtime::inventory::Inventory;
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CookingRecipe {
    pub key: String,
    pub result: ItemStack,
    pub cooking_time: u32,
    pub experience: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Furnace {
    pub inventory: Inventory,
    pub lit_remaining: u32,
    pub lit_total: u32,
    pub cook_progress: u32,
    pub cook_total: u32,
    pub used_recipes: BTreeMap<String, u32>,
}

impl Furnace {
    pub fn new() -> Self {
        Self {
            inventory: Inventory::empty(3),
            lit_remaining: 0,
            lit_total: 0,
            cook_progress: 0,
            cook_total: 200,
            used_recipes: BTreeMap::new(),
        }
    }

    pub fn tick(&mut self, input: FurnaceTickInput<'_>) -> FurnaceTickOutcome {
        let was_lit = self.lit_remaining > 0;
        self.lit_remaining = self.lit_remaining.saturating_sub(1);
        let has_fuel_and_input =
            !self.inventory.slots[0].stack.is_empty() && !self.inventory.slots[1].stack.is_empty();
        let active_branch = self.lit_remaining > 0 || has_fuel_and_input;
        let burnable = input.recipe.is_some_and(|recipe| self.can_burn(recipe));
        let mut ignited = false;
        let mut completed = false;

        if active_branch {
            if self.lit_remaining == 0 && burnable && input.fuel_duration > 0 {
                self.lit_remaining = input.fuel_duration;
                self.lit_total = input.fuel_duration;
                self.inventory.slots[1].stack.shrink(1);
                if self.inventory.slots[1].stack.is_empty()
                    && let Some(remainder) = input.fuel_remainder
                {
                    self.inventory.slots[1].stack = remainder.clone();
                }
                ignited = true;
            }
            if self.lit_remaining > 0 && burnable {
                self.cook_progress += 1;
                if self.cook_progress == self.cook_total {
                    self.cook_progress = 0;
                    let recipe = input.recipe.expect("burnable recipe");
                    self.cook_total = recipe.cooking_time;
                    self.complete(recipe, input.wet_sponge_bucket_conversion);
                    completed = true;
                }
            } else {
                self.cook_progress = 0;
            }
        } else if self.cook_progress > 0 {
            self.cook_progress = self.cook_progress.saturating_sub(2).min(self.cook_total);
        }

        if input.input_identity_changed {
            self.cook_progress = 0;
            self.cook_total = input.recipe.map_or(200, |recipe| recipe.cooking_time);
        }
        FurnaceTickOutcome {
            ignited,
            completed,
            lit_state_changed: was_lit != (self.lit_remaining > 0),
        }
    }

    fn can_burn(&self, recipe: &CookingRecipe) -> bool {
        if recipe.result.is_empty() || self.inventory.slots[0].stack.is_empty() {
            return false;
        }
        let output = &self.inventory.slots[2].stack;
        output.is_empty()
            || (output.compatible_with(&recipe.result)
                && output.count + recipe.result.count <= 99.min(recipe.result.maximum))
    }

    fn complete(&mut self, recipe: &CookingRecipe, wet_sponge_bucket_conversion: bool) {
        let output = &mut self.inventory.slots[2].stack;
        if output.is_empty() {
            *output = recipe.result.clone();
        } else {
            output.grow(recipe.result.count);
        }
        if wet_sponge_bucket_conversion {
            self.inventory.slots[1].stack = ItemStack::new(0, minecraft("water_bucket"), 1, 1, 0);
        }
        self.inventory.slots[0].stack.shrink(1);
        *self.used_recipes.entry(recipe.key.clone()).or_default() += 1;
    }
}

impl Default for Furnace {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FurnaceTickInput<'a> {
    pub recipe: Option<&'a CookingRecipe>,
    pub fuel_duration: u32,
    pub fuel_remainder: Option<&'a ItemStack>,
    pub input_identity_changed: bool,
    pub wet_sponge_bucket_conversion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FurnaceTickOutcome {
    pub ignited: bool,
    pub completed: bool,
    pub lit_state_changed: bool,
}

pub fn experience_to_drop(count: u32, experience: f32, random_float: Option<f32>) -> u32 {
    let exact = count as f32 * experience;
    let floor = exact.floor() as u32;
    let fraction = exact - floor as f32;
    if fraction > 0.0 && random_float.is_some_and(|draw| draw < fraction) {
        floor + 1
    } else {
        floor
    }
}

fn minecraft(path: &str) -> ResourceId {
    ResourceId::minecraft(path).expect("locked item identifier")
}
