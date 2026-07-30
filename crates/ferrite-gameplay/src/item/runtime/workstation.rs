//! Smithing, stonecutting, cartography, and Loom result transactions.

use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Smithing {
    pub inputs: [ItemStack; 3],
    pub result: ItemStack,
    pub stored_recipe: Option<String>,
    pub recipe_error: bool,
}

impl Smithing {
    pub fn empty() -> Self {
        Self {
            inputs: std::array::from_fn(|_| ItemStack::empty()),
            result: ItemStack::empty(),
            stored_recipe: None,
            recipe_error: false,
        }
    }

    pub fn recompute(&mut self, recipe: Option<(&str, ItemStack)>) {
        if let Some((key, result)) = recipe {
            self.stored_recipe = Some(key.to_owned());
            self.result = result;
        } else {
            self.stored_recipe = None;
            self.result = ItemStack::empty();
        }
        self.recipe_error =
            self.inputs.iter().all(|stack| !stack.is_empty()) && self.result.is_empty();
    }

    pub fn take(&mut self) -> Option<String> {
        if self.result.is_empty() {
            return None;
        }
        let credited = self.stored_recipe.take();
        self.result = ItemStack::empty();
        for input in &mut self.inputs {
            if !input.is_empty() {
                input.shrink(1);
            }
        }
        credited
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stonecutter {
    pub input_item: Option<ResourceId>,
    pub recipes: Vec<(String, ItemStack)>,
    pub selected: i32,
    pub result: ItemStack,
    pub stored_recipe: Option<String>,
    pub last_sound_game_time: Option<i64>,
}

impl Stonecutter {
    pub fn empty() -> Self {
        Self {
            input_item: None,
            recipes: Vec::new(),
            selected: -1,
            result: ItemStack::empty(),
            stored_recipe: None,
            last_sound_game_time: None,
        }
    }

    pub fn change_input(&mut self, input: &ItemStack, enabled_matches: Vec<(String, ItemStack)>) {
        if self.input_item == input.item {
            return;
        }
        self.input_item = input.item.clone();
        self.recipes = enabled_matches;
        self.selected = -1;
        self.result = ItemStack::empty();
        self.stored_recipe = None;
    }

    pub fn select(&mut self, requested: i32) -> bool {
        if requested == self.selected {
            return false;
        }
        let Ok(index) = usize::try_from(requested) else {
            return true;
        };
        let Some((key, result)) = self.recipes.get(index) else {
            return true;
        };
        self.selected = requested;
        self.stored_recipe = Some(key.clone());
        self.result = result.clone();
        true
    }

    pub fn take(&mut self, input: &mut ItemStack, game_time: i64) -> StonecutterTake {
        if self.result.is_empty() {
            return StonecutterTake {
                recipe: None,
                play_sound: false,
            };
        }
        input.shrink(1);
        let recipe = self.stored_recipe.clone();
        let play_sound = self.last_sound_game_time != Some(game_time);
        if play_sound {
            self.last_sound_game_time = Some(game_time);
        }
        if input.is_empty() {
            self.selected = -1;
            self.result = ItemStack::empty();
            self.stored_recipe = None;
        } else if let Ok(index) = usize::try_from(self.selected)
            && let Some((_, result)) = self.recipes.get(index)
        {
            self.result = result.clone();
        }
        StonecutterTake { recipe, play_sound }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StonecutterTake {
    pub recipe: Option<String>,
    pub play_sound: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapData {
    pub id: u32,
    pub center_x: i32,
    pub center_z: i32,
    pub scale: u8,
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartographyMaterial {
    Paper,
    GlassPane,
    EmptyMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapPostProcess {
    Scale,
    Lock,
    Duplicate,
}

pub fn cartography_preview(
    map: MapData,
    material: CartographyMaterial,
) -> Option<(MapPostProcess, u8)> {
    match material {
        CartographyMaterial::Paper if !map.locked && map.scale < 4 => {
            Some((MapPostProcess::Scale, 1))
        }
        CartographyMaterial::GlassPane if !map.locked => Some((MapPostProcess::Lock, 1)),
        CartographyMaterial::EmptyMap => Some((MapPostProcess::Duplicate, 2)),
        _ => None,
    }
}

pub fn apply_map_post_process(
    map: MapData,
    process: MapPostProcess,
    allocated_id: u32,
) -> Vec<MapData> {
    match process {
        MapPostProcess::Duplicate => vec![map, map],
        MapPostProcess::Lock => vec![MapData {
            id: allocated_id,
            locked: true,
            ..map
        }],
        MapPostProcess::Scale => {
            let scale = map.scale.saturating_add(1).min(4);
            let size = 128_i32 << scale;
            let center_x = (map.center_x + 64).div_euclid(size) * size + size / 2 - 64;
            let center_z = (map.center_z + 64).div_euclid(size) * size + size / 2 - 64;
            vec![MapData {
                id: allocated_id,
                center_x,
                center_z,
                scale,
                locked: false,
            }]
        }
    }
}

pub const ALL_BANNER_PATTERNS: [&str; 43] = [
    "base",
    "border",
    "bricks",
    "circle",
    "creeper",
    "cross",
    "curly_border",
    "diagonal_left",
    "diagonal_right",
    "diagonal_up_left",
    "diagonal_up_right",
    "flow",
    "flower",
    "globe",
    "gradient",
    "gradient_up",
    "guster",
    "half_horizontal",
    "half_horizontal_bottom",
    "half_vertical",
    "half_vertical_right",
    "mojang",
    "piglin",
    "rhombus",
    "skull",
    "small_stripes",
    "square_bottom_left",
    "square_bottom_right",
    "square_top_left",
    "square_top_right",
    "straight_cross",
    "stripe_bottom",
    "stripe_center",
    "stripe_downleft",
    "stripe_downright",
    "stripe_left",
    "stripe_middle",
    "stripe_right",
    "stripe_top",
    "triangle_bottom",
    "triangle_top",
    "triangles_bottom",
    "triangles_top",
];

pub const NO_ITEM_BANNER_PATTERNS: [&str; 32] = [
    "square_bottom_left",
    "square_bottom_right",
    "square_top_left",
    "square_top_right",
    "stripe_bottom",
    "stripe_top",
    "stripe_left",
    "stripe_right",
    "stripe_center",
    "stripe_middle",
    "stripe_downright",
    "stripe_downleft",
    "small_stripes",
    "cross",
    "straight_cross",
    "triangle_bottom",
    "triangle_top",
    "triangles_bottom",
    "triangles_top",
    "diagonal_left",
    "diagonal_up_right",
    "diagonal_up_left",
    "diagonal_right",
    "circle",
    "rhombus",
    "half_vertical",
    "half_horizontal",
    "half_vertical_right",
    "half_horizontal_bottom",
    "border",
    "gradient",
    "gradient_up",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerLayer {
    pub pattern: ResourceId,
    pub dye_color: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loom {
    pub selectable: Vec<ResourceId>,
    pub selected: i32,
    pub existing_layers: Vec<BannerLayer>,
    pub result_layers: Vec<BannerLayer>,
}

impl Loom {
    pub fn new() -> Self {
        Self {
            selectable: Vec::new(),
            selected: -1,
            existing_layers: Vec::new(),
            result_layers: Vec::new(),
        }
    }

    pub fn update_choices(&mut self, choices: Vec<ResourceId>) {
        let old = usize::try_from(self.selected)
            .ok()
            .and_then(|index| self.selectable.get(index))
            .cloned();
        self.selectable = choices;
        self.selected = if self.selectable.len() == 1 {
            0
        } else {
            old.and_then(|pattern| self.selectable.iter().position(|item| item == &pattern))
                .map_or(-1, |index| index as i32)
        };
        if self.existing_layers.len() >= 6 {
            self.selected = -1;
            self.result_layers.clear();
        }
    }

    pub fn select(&mut self, index: i32, dye_color: u8) -> bool {
        let Ok(index) = usize::try_from(index) else {
            return false;
        };
        let Some(pattern) = self.selectable.get(index).cloned() else {
            return false;
        };
        if self.existing_layers.len() >= 6 {
            return false;
        }
        self.selected = index as i32;
        self.result_layers = self.existing_layers.clone();
        self.result_layers.push(BannerLayer { pattern, dye_color });
        true
    }
}

impl Default for Loom {
    fn default() -> Self {
        Self::new()
    }
}
