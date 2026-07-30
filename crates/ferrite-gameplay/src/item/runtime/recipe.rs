//! Recipe domains, serializer catalog, ordered lookup, cropping, and Crafter cache.

use crate::item::runtime::stack::ItemStack;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecipeDomain {
    Crafting,
    Smelting,
    Blasting,
    Smoking,
    CampfireCooking,
    Stonecutting,
    Smithing,
}

impl RecipeDomain {
    pub const ALL: [Self; 7] = [
        Self::Crafting,
        Self::Smelting,
        Self::Blasting,
        Self::Smoking,
        Self::CampfireCooking,
        Self::Stonecutting,
        Self::Smithing,
    ];
}

pub const RECIPE_SERIALIZERS: [&str; 21] = [
    "crafting_shaped",
    "crafting_shapeless",
    "crafting_dye",
    "crafting_imbue",
    "crafting_transmute",
    "crafting_decorated_pot",
    "crafting_special_bookcloning",
    "crafting_special_mapextending",
    "crafting_special_firework_rocket",
    "crafting_special_firework_star",
    "crafting_special_firework_star_fade",
    "crafting_special_bannerduplicate",
    "crafting_special_shielddecoration",
    "crafting_special_repairitem",
    "smelting",
    "blasting",
    "smoking",
    "campfire_cooking",
    "stonecutting",
    "smithing_transform",
    "smithing_trim",
];

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeRecord {
    pub key: String,
    pub domain: RecipeDomain,
    pub result: ItemStack,
    pub special: bool,
    pub cooking_time: u32,
    pub experience: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeManager {
    pub identity: u64,
    recipes: Vec<RecipeRecord>,
}

impl RecipeManager {
    pub fn prepare(identity: u64, mut recipes: Vec<RecipeRecord>) -> Self {
        recipes.sort_by(|left, right| left.key.cmp(&right.key));
        Self { identity, recipes }
    }

    pub fn recipes(&self, domain: RecipeDomain) -> impl Iterator<Item = &RecipeRecord> {
        self.recipes
            .iter()
            .filter(move |recipe| recipe.domain == domain)
    }

    pub fn get_recipe_for<'a>(
        &'a self,
        domain: RecipeDomain,
        preferred_key: Option<&str>,
        matches: impl Fn(&RecipeRecord) -> bool,
    ) -> Option<&'a RecipeRecord> {
        if let Some(preferred) = preferred_key
            && let Some(recipe) = self
                .recipes(domain)
                .find(|recipe| recipe.key == preferred && matches(recipe))
        {
            return Some(recipe);
        }
        self.recipes(domain).find(|recipe| matches(recipe))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedCraftInput {
    pub width: usize,
    pub height: usize,
    pub left: usize,
    pub top: usize,
    pub cells: Vec<ItemStack>,
}

pub fn crop_crafting_input(
    width: usize,
    height: usize,
    cells: &[ItemStack],
) -> PositionedCraftInput {
    if width == 0 || height == 0 || cells.len() != width.saturating_mul(height) {
        return PositionedCraftInput {
            width: 0,
            height: 0,
            left: 0,
            top: 0,
            cells: Vec::new(),
        };
    }
    let mut left = width;
    let mut right = 0;
    let mut top = height;
    let mut bottom = 0;
    for y in 0..height {
        for x in 0..width {
            if cells[y * width + x].is_empty() {
                continue;
            }
            left = left.min(x);
            right = right.max(x);
            top = top.min(y);
            bottom = bottom.max(y);
        }
    }
    if left == width {
        return PositionedCraftInput {
            width: 0,
            height: 0,
            left: 0,
            top: 0,
            cells: Vec::new(),
        };
    }
    let cropped_width = right - left + 1;
    let cropped_height = bottom - top + 1;
    let mut cropped = Vec::with_capacity(cropped_width * cropped_height);
    for y in top..=bottom {
        for x in left..=right {
            cropped.push(cells[y * width + x].clone());
        }
    }
    PositionedCraftInput {
        width: cropped_width,
        height: cropped_height,
        left,
        top,
        cells: cropped,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecipeCacheKey {
    width: usize,
    height: usize,
    cells: Vec<ItemStack>,
}

impl From<&PositionedCraftInput> for RecipeCacheKey {
    fn from(input: &PositionedCraftInput) -> Self {
        let cells = input
            .cells
            .iter()
            .map(|stack| {
                let mut normalized = stack.clone();
                if !normalized.is_empty() {
                    normalized.count = 1;
                }
                normalized
            })
            .collect();
        Self {
            width: input.width,
            height: input.height,
            cells,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrafterRecipeCache {
    manager_identity: Option<u64>,
    entries: VecDeque<(RecipeCacheKey, Option<String>)>,
}

impl CrafterRecipeCache {
    pub fn new() -> Self {
        Self {
            manager_identity: None,
            entries: VecDeque::new(),
        }
    }

    pub fn get_or_insert(
        &mut self,
        manager_identity: u64,
        input: &PositionedCraftInput,
        lookup: impl FnOnce() -> Option<String>,
    ) -> Option<String> {
        if input.cells.is_empty() {
            return None;
        }
        if self.manager_identity != Some(manager_identity) {
            self.manager_identity = Some(manager_identity);
            self.entries.clear();
        }
        let key = RecipeCacheKey::from(input);
        if let Some(index) = self.entries.iter().position(|(stored, _)| stored == &key) {
            let entry = self.entries.remove(index).expect("located cache entry");
            let value = entry.1.clone();
            self.entries.push_front(entry);
            return value;
        }
        let value = lookup();
        self.entries.push_front((key, value.clone()));
        self.entries.truncate(10);
        value
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for CrafterRecipeCache {
    fn default() -> Self {
        Self::new()
    }
}
