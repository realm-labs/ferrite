//! Menu close disposition and non-click control admission.

use crate::item::runtime::inventory::Inventory;
use crate::item::runtime::stack::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalState {
    Active,
    DimensionChange,
    Removed,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseOutcome {
    pub dropped: Vec<ItemStack>,
    pub cursor_cleared: bool,
    pub transient_slots_cleared: usize,
}

pub fn close_menu(
    cursor: &mut ItemStack,
    transient_inputs: &mut [ItemStack],
    inventory: &mut Inventory,
    state: RemovalState,
) -> CloseOutcome {
    let mut dropped = Vec::new();
    let return_to_inventory = matches!(state, RemovalState::Active | RemovalState::DimensionChange);
    dispose_stack(cursor, inventory, return_to_inventory, &mut dropped);
    let mut transient_slots_cleared = 0;
    for stack in transient_inputs {
        if !stack.is_empty() {
            transient_slots_cleared += 1;
        }
        dispose_stack(stack, inventory, return_to_inventory, &mut dropped);
    }
    CloseOutcome {
        dropped,
        cursor_cleared: cursor.is_empty(),
        transient_slots_cleared,
    }
}

pub fn dispose_stack(
    stack: &mut ItemStack,
    inventory: &mut Inventory,
    return_to_inventory: bool,
    dropped: &mut Vec<ItemStack>,
) {
    if stack.is_empty() {
        return;
    }
    if return_to_inventory {
        for slot in &mut inventory.slots {
            if !slot.stack.compatible_with(stack) || slot.stack.count >= slot.stack.maximum {
                continue;
            }
            let moved = (slot.stack.maximum - slot.stack.count).min(stack.count);
            slot.stack.grow(moved);
            stack.shrink(moved);
            slot.changed = true;
            if stack.is_empty() {
                return;
            }
        }
        for slot in &mut inventory.slots {
            if !slot.stack.is_empty() {
                continue;
            }
            let moved = stack.count.min(stack.maximum);
            slot.stack = stack.split(moved, stack.identity);
            slot.changed = true;
            if stack.is_empty() {
                return;
            }
        }
    }
    dropped.push(std::mem::replace(stack, ItemStack::empty()));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlAdmission {
    pub container_matches: bool,
    pub spectator: bool,
    pub still_valid: bool,
}

impl ControlAdmission {
    pub const fn generic_button(self) -> bool {
        self.container_matches && !self.spectator && self.still_valid
    }

    pub const fn crafter_slot_state(self) -> bool {
        self.container_matches && !self.spectator
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LecternControl {
    Previous,
    Next,
    TakeBook,
    SetPage(i32),
}

pub const fn lectern_control(button: i32) -> Option<LecternControl> {
    match button {
        1 => Some(LecternControl::Previous),
        2 => Some(LecternControl::Next),
        3 => Some(LecternControl::TakeBook),
        page if page >= 100 => Some(LecternControl::SetPage(page - 100)),
        _ => None,
    }
}

pub const fn enchantment_button(
    button: i32,
    item_present: bool,
    displayed_cost: i32,
    lapis: i32,
    experience_level: i32,
    infinite_materials: bool,
) -> bool {
    if button < 0 || button >= 3 || !item_present || displayed_cost <= 0 {
        return false;
    }
    infinite_materials
        || (lapis > button && experience_level > button && experience_level >= displayed_cost)
}

pub fn set_crafter_slot_state(
    slots: &[ItemStack; 9],
    disabled: &mut [bool; 9],
    slot: i32,
    enabled: bool,
) -> bool {
    let Ok(index) = usize::try_from(slot) else {
        return false;
    };
    if index >= slots.len() || !slots[index].is_empty() || disabled[index] == !enabled {
        return false;
    }
    disabled[index] = !enabled;
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSlotSnapshot {
    pub backing_container: u64,
    pub backing_slot: usize,
    pub local: ItemStack,
    pub remote_hash: u64,
}

pub fn transfer_matching_snapshots(
    closing: &[MenuSlotSnapshot],
    inventory_menu: &mut [MenuSlotSnapshot],
) -> usize {
    let mut transferred = 0;
    for target in inventory_menu {
        let Some(source) = closing.iter().find(|source| {
            source.backing_container == target.backing_container
                && source.backing_slot == target.backing_slot
        }) else {
            continue;
        };
        target.local = source.local.clone();
        target.remote_hash = source.remote_hash;
        transferred += 1;
    }
    transferred
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionControl {
    pub acknowledged: bool,
    pub selected: Option<usize>,
}

pub fn stonecutter_selection(
    current: Option<usize>,
    requested: i32,
    recipe_count: usize,
) -> SelectionControl {
    let requested = usize::try_from(requested).ok();
    if requested == current {
        return SelectionControl {
            acknowledged: false,
            selected: current,
        };
    }
    SelectionControl {
        acknowledged: true,
        selected: requested.filter(|&index| index < recipe_count),
    }
}

pub fn loom_selection(requested: i32, selectable_patterns: usize) -> SelectionControl {
    let selected = usize::try_from(requested)
        .ok()
        .filter(|&index| index < selectable_patterns);
    SelectionControl {
        acknowledged: selected.is_some(),
        selected,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameControl {
    RejectedTooLong,
    Unchanged,
    RemoveCustomName,
    SetLiteral(String),
}

pub fn anvil_rename(current: Option<&str>, filtered: &str) -> RenameControl {
    if filtered.chars().count() > 50 {
        return RenameControl::RejectedTooLong;
    }
    if current == Some(filtered) {
        return RenameControl::Unchanged;
    }
    if filtered.is_empty() {
        RenameControl::RemoveCustomName
    } else {
        RenameControl::SetLiteral(filtered.to_owned())
    }
}
