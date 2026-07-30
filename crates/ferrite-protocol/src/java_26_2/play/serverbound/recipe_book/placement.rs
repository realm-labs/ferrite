use std::collections::{BTreeMap, BTreeSet};

use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::clientbound::recipe::book::PlaceGhostRecipe;
use crate::java_26_2::play::clientbound::recipe::display::RecipeDisplay;
use crate::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStack, StackContents,
};
use crate::java_26_2::play::serverbound::recipe_book::packet::PlaceRecipe;
use crate::java_26_2::play::serverbound::recipe_book::state::ServerRecipeBook;
use crate::java_26_2::value::identifier::Identifier;

const DEFAULT_MAXIMUM_STACK: i32 = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementItem {
    pub stack: ItemStack,
    pub maximum_stack_size: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRecipePlacement {
    pub width: usize,
    pub height: usize,
    pub slots: Vec<Option<PlacementItem>>,
}

impl ResolvedRecipePlacement {
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.slots.len() == self.width.saturating_mul(self.height)
            && self
                .slots
                .iter()
                .flatten()
                .all(|item| !item.stack.is_empty() && item.maximum_stack_size > 0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipePlacementSource {
    pub parent: Identifier,
    pub display: RecipeDisplay,
    pub enabled: bool,
    pub placement: Option<ResolvedRecipePlacement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedRecipePlacement {
    pub display_id: i32,
    pub parent: Identifier,
    pub display: RecipeDisplay,
    pub placement: Option<ResolvedRecipePlacement>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecipePlacementIndex {
    entries: Vec<IndexedRecipePlacement>,
}

impl RecipePlacementIndex {
    pub fn rebuild(
        sources: impl IntoIterator<Item = RecipePlacementSource>,
    ) -> Result<Self, RecipePlacementIndexError> {
        let mut entries = Vec::new();
        for source in sources {
            if !source.enabled {
                continue;
            }
            let display_id = i32::try_from(entries.len())
                .map_err(|_| RecipePlacementIndexError::TooManyDisplays)?;
            entries.push(IndexedRecipePlacement {
                display_id,
                parent: source.parent,
                display: source.display,
                placement: source.placement,
            });
        }
        Ok(Self { entries })
    }

    #[must_use]
    pub fn resolve(&self, display_id: i32) -> Option<&IndexedRecipePlacement> {
        usize::try_from(display_id)
            .ok()
            .and_then(|index| self.entries.get(index))
    }

    #[must_use]
    pub fn entries(&self) -> &[IndexedRecipePlacement] {
        &self.entries
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipePlacementIndexError {
    TooManyDisplays,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementBracket {
    Begin,
    Finish,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeMenuTransaction {
    pub container_id: i32,
    pub still_valid: bool,
    pub recipe_book_menu: bool,
    pub grid_width: usize,
    pub grid_height: usize,
    pub grid: Vec<ItemStack>,
    pub extra_clear_targets: Vec<ItemStack>,
    pub player_inventory: Vec<ItemStack>,
    pub inventory_changed: bool,
    pub discarded_on_clear: Vec<ItemStack>,
    pub brackets: Vec<PlacementBracket>,
    maximum_stack_sizes: BTreeMap<Identifier, i32>,
}

impl RecipeMenuTransaction {
    #[must_use]
    pub fn new(
        container_id: i32,
        grid_width: usize,
        grid_height: usize,
        inventory_slots: usize,
    ) -> Self {
        Self {
            container_id,
            still_valid: true,
            recipe_book_menu: true,
            grid_width,
            grid_height,
            grid: vec![ItemStack::Empty; grid_width.saturating_mul(grid_height)],
            extra_clear_targets: Vec::new(),
            player_inventory: vec![ItemStack::Empty; inventory_slots],
            inventory_changed: false,
            discarded_on_clear: Vec::new(),
            brackets: Vec::new(),
            maximum_stack_sizes: BTreeMap::new(),
        }
    }

    pub fn set_maximum_stack_size(&mut self, item: Identifier, maximum: i32) {
        self.maximum_stack_sizes.insert(item, maximum.max(1));
    }

    fn place(
        &mut self,
        entry: &IndexedRecipePlacement,
        use_maximum_items: bool,
        creative: bool,
    ) -> PlacementMutation {
        let placement = entry
            .placement
            .as_ref()
            .expect("placement admission requires present information");
        self.brackets.push(PlacementBracket::Begin);
        let mutation = self.place_inner(entry, placement, use_maximum_items, creative);
        self.brackets.push(PlacementBracket::Finish);
        mutation
    }

    fn place_inner(
        &mut self,
        entry: &IndexedRecipePlacement,
        placement: &ResolvedRecipePlacement,
        use_maximum_items: bool,
        creative: bool,
    ) -> PlacementMutation {
        if !creative && !self.can_clear_targets() {
            return PlacementMutation::NoChange;
        }
        let biggest_craftable = self.biggest_craftable(placement);
        if biggest_craftable == 0 {
            self.clear_targets();
            self.inventory_changed = true;
            return PlacementMutation::Ghost(Box::new(PlayClientboundPacket::PlaceGhostRecipe(
                Box::new(PlaceGhostRecipe {
                    container_id: self.container_id,
                    display: entry.display.clone(),
                }),
            )));
        }

        let already_matches = self.grid_matches(placement);
        if already_matches
            && self.grid.iter().zip(&placement.slots).any(|(stack, item)| {
                item.as_ref().is_some_and(|item| {
                    !stack.is_empty()
                        && stack.count().saturating_add(1)
                            > biggest_craftable.min(item.maximum_stack_size)
                })
            })
        {
            return PlacementMutation::NoChange;
        }
        let minimum_current = self
            .grid
            .iter()
            .filter(|stack| !stack.is_empty())
            .map(ItemStack::count)
            .min()
            .unwrap_or(0);
        let mut amount = if use_maximum_items {
            biggest_craftable
        } else if already_matches {
            minimum_current.saturating_add(1)
        } else {
            1
        };
        let holder_maximum = placement
            .slots
            .iter()
            .flatten()
            .map(|item| item.maximum_stack_size)
            .min()
            .unwrap_or(1);
        amount = amount.min(holder_maximum).max(1);

        self.clear_targets();
        for (slot, required) in placement.slots.iter().enumerate() {
            let Some(required) = required else {
                continue;
            };
            let removed = remove_matching(&required.stack, amount, &mut self.player_inventory);
            if removed < amount {
                self.inventory_changed = true;
                return PlacementMutation::Placed;
            }
            self.grid[slot] = copy_with_count(&required.stack, amount);
        }
        self.inventory_changed = true;
        PlacementMutation::Placed
    }

    fn biggest_craftable(&self, placement: &ResolvedRecipePlacement) -> i32 {
        let mut requirements: Vec<(ItemStack, i32)> = Vec::new();
        for item in placement.slots.iter().flatten() {
            if let Some((_, count)) = requirements
                .iter_mut()
                .find(|(stack, _)| same_item_and_components(stack, &item.stack))
            {
                *count = count.saturating_add(1);
            } else {
                requirements.push((item.stack.clone(), 1));
            }
        }
        requirements
            .into_iter()
            .map(|(required, per_craft)| {
                self.player_inventory
                    .iter()
                    .chain(&self.grid)
                    .filter(|stack| same_item_and_components(stack, &required))
                    .map(ItemStack::count)
                    .sum::<i32>()
                    / per_craft
            })
            .min()
            .unwrap_or(0)
            .max(0)
    }

    fn grid_matches(&self, placement: &ResolvedRecipePlacement) -> bool {
        self.grid_width == placement.width
            && self.grid_height == placement.height
            && self.grid.len() == placement.slots.len()
            && self
                .grid
                .iter()
                .zip(&placement.slots)
                .all(|(stack, required)| match required {
                    Some(required) => same_item_and_components(stack, &required.stack),
                    None => stack.is_empty(),
                })
    }

    fn can_clear_targets(&self) -> bool {
        let mut inventory = self.player_inventory.clone();
        self.grid
            .iter()
            .chain(&self.extra_clear_targets)
            .all(|stack| {
                let mut stack = stack.clone();
                move_to_inventory(&mut stack, &mut inventory, &self.maximum_stack_sizes);
                stack.is_empty()
            })
    }

    fn clear_targets(&mut self) {
        for mut stack in std::mem::take(&mut self.grid) {
            move_to_inventory(
                &mut stack,
                &mut self.player_inventory,
                &self.maximum_stack_sizes,
            );
            if !stack.is_empty() {
                self.discarded_on_clear.push(stack);
            }
        }
        self.grid = vec![ItemStack::Empty; self.grid_width.saturating_mul(self.grid_height)];
        for mut stack in std::mem::take(&mut self.extra_clear_targets) {
            move_to_inventory(
                &mut stack,
                &mut self.player_inventory,
                &self.maximum_stack_sizes,
            );
            if !stack.is_empty() {
                self.discarded_on_clear.push(stack);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlacementMutation {
    NoChange,
    Placed,
    Ghost(Box<PlayClientboundPacket>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceRecipeIgnore {
    SpectatorOrWrongContainer,
    InvalidMenu,
    UnknownDisplay,
    LockedParent,
    NotRecipeMenu,
    ImpossiblePlacement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaceRecipeOutcome {
    Ignored(PlaceRecipeIgnore),
    Applied(PlacementMutation),
}

#[derive(Debug, Clone, Default)]
pub struct RecipePlacementSession {
    pub spectator: bool,
    pub creative: bool,
    pub idle_resets: u64,
    pub invalid_menu_logs: u64,
    pub impossible_placement_logs: u64,
    pub book: ServerRecipeBook,
    pub index: RecipePlacementIndex,
}

impl RecipePlacementSession {
    pub fn handle_place_recipe(
        &mut self,
        current_menu: Option<&mut RecipeMenuTransaction>,
        packet: PlaceRecipe,
    ) -> PlaceRecipeOutcome {
        self.idle_resets = self.idle_resets.wrapping_add(1);
        let Some(menu) = current_menu else {
            return PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::SpectatorOrWrongContainer);
        };
        if self.spectator || menu.container_id != packet.container_id {
            return PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::SpectatorOrWrongContainer);
        }
        if !menu.still_valid {
            self.invalid_menu_logs = self.invalid_menu_logs.wrapping_add(1);
            return PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::InvalidMenu);
        }
        let Some(entry) = self.index.resolve(packet.display_id).cloned() else {
            return PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::UnknownDisplay);
        };
        if !self.book.known.contains(&entry.parent) {
            return PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::LockedParent);
        }
        if !menu.recipe_book_menu {
            return PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::NotRecipeMenu);
        }
        if !entry.placement.as_ref().is_some_and(|placement| {
            placement.is_well_formed()
                && placement.width == menu.grid_width
                && placement.height == menu.grid_height
        }) {
            self.impossible_placement_logs = self.impossible_placement_logs.wrapping_add(1);
            return PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::ImpossiblePlacement);
        }
        PlaceRecipeOutcome::Applied(menu.place(&entry, packet.use_maximum_items, self.creative))
    }
}

fn remove_matching(required: &ItemStack, amount: i32, inventory: &mut [ItemStack]) -> i32 {
    let mut remaining = amount;
    for source in inventory {
        if remaining == 0 {
            break;
        }
        if !same_item_and_components(source, required) {
            continue;
        }
        let removed = remaining.min(source.count()).max(0);
        shrink(source, removed);
        remaining -= removed;
    }
    amount - remaining
}

fn move_to_inventory(
    source: &mut ItemStack,
    inventory: &mut [ItemStack],
    maximum_stack_sizes: &BTreeMap<Identifier, i32>,
) {
    let maximum = source
        .contents()
        .and_then(|contents| maximum_stack_sizes.get(&contents.item))
        .copied()
        .unwrap_or(DEFAULT_MAXIMUM_STACK)
        .max(1);
    for target in inventory.iter_mut() {
        if source.is_empty() {
            return;
        }
        if target.is_empty() || !same_item_and_components(source, target) {
            continue;
        }
        let moved = maximum
            .saturating_sub(target.count())
            .max(0)
            .min(source.count());
        grow(target, moved);
        shrink(source, moved);
    }
    for target in inventory {
        if source.is_empty() {
            return;
        }
        if !target.is_empty() {
            continue;
        }
        let moved = maximum.min(source.count()).max(0);
        *target = copy_with_count(source, moved);
        shrink(source, moved);
    }
}

fn same_item_and_components(left: &ItemStack, right: &ItemStack) -> bool {
    match (left.contents(), right.contents()) {
        (Some(left), Some(right)) => {
            left.item == right.item
                && normalized_patch(&left.components) == normalized_patch(&right.components)
        }
        (None, None) => true,
        _ => false,
    }
}

fn normalized_patch(
    patch: &DataComponentPatch,
) -> (BTreeMap<Identifier, Vec<u8>>, BTreeSet<Identifier>) {
    let added = patch
        .added
        .iter()
        .map(
            |EncodedComponentValue {
                 component,
                 encoded_value,
             }| (component.clone(), encoded_value.clone()),
        )
        .collect();
    let removed = patch.removed.iter().cloned().collect();
    (added, removed)
}

fn copy_with_count(source: &ItemStack, count: i32) -> ItemStack {
    let Some(contents) = source.contents() else {
        return ItemStack::Empty;
    };
    ItemStack::Present(StackContents {
        item: contents.item.clone(),
        count,
        components: contents.components.clone(),
    })
}

fn grow(stack: &mut ItemStack, amount: i32) {
    if let ItemStack::Present(contents) = stack {
        contents.count = contents.count.saturating_add(amount.max(0));
    }
}

fn shrink(stack: &mut ItemStack, amount: i32) {
    let ItemStack::Present(contents) = stack else {
        return;
    };
    contents.count = contents.count.saturating_sub(amount.max(0));
    if contents.count <= 0 {
        *stack = ItemStack::Empty;
    }
}
