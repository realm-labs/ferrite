//! Generic seven-variant container click state machine.

use crate::item::runtime::inventory::{Slot, move_item_stack_to};
use crate::item::runtime::menu_layout::MenuLayout;
use crate::item::runtime::stack::ItemStack;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerInput {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

impl ContainerInput {
    pub const fn decode(id: i32) -> Self {
        match id {
            1 => Self::QuickMove,
            2 => Self::Swap,
            3 => Self::Clone,
            4 => Self::Throw,
            5 => Self::QuickCraft,
            6 => Self::PickupAll,
            _ => Self::Pickup,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickPlayer {
    pub inventory: Vec<ItemStack>,
    pub infinite_materials: bool,
    pub can_drop_items: bool,
}

impl ClickPlayer {
    pub fn empty() -> Self {
        Self {
            inventory: (0..41).map(|_| ItemStack::empty()).collect(),
            infinite_materials: false,
            can_drop_items: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QuickCraftState {
    pub status: u8,
    pub kind: u8,
    pub selected: BTreeSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickMenu {
    pub slots: Vec<Slot>,
    pub carried: ItemStack,
    pub quick_craft: QuickCraftState,
    pub dropped: Vec<ItemStack>,
    next_identity: u64,
}

impl ClickMenu {
    pub fn new(slots: Vec<Slot>, next_identity: u64) -> Self {
        Self {
            slots,
            carried: ItemStack::empty(),
            quick_craft: QuickCraftState::default(),
            dropped: Vec::new(),
            next_identity,
        }
    }

    pub fn is_valid_slot_index(&self, index: i32) -> bool {
        index == -1 || index == -999 || index < self.slots.len() as i32
    }

    pub fn clicked(
        &mut self,
        slot: i32,
        button: i32,
        input: ContainerInput,
        player: &mut ClickPlayer,
        layout: &MenuLayout,
    ) -> Result<(), ClickError> {
        if input != ContainerInput::QuickCraft && self.quick_craft.status != 0 {
            self.reset_quick_craft();
            return Ok(());
        }
        match input {
            ContainerInput::Pickup => self.pickup(slot, button),
            ContainerInput::QuickMove => self.quick_move(slot, button, layout),
            ContainerInput::Swap => self.swap(slot, button, player),
            ContainerInput::Clone => self.clone_stack(slot, player),
            ContainerInput::Throw => self.throw(slot, button, player),
            ContainerInput::QuickCraft => self.quick_craft(slot, button, player),
            ContainerInput::PickupAll => self.pickup_all(slot, button),
        }
    }

    fn pickup(&mut self, slot_index: i32, button: i32) -> Result<(), ClickError> {
        if !matches!(button, 0 | 1) {
            return Ok(());
        }
        if slot_index == -999 {
            let amount = if button == 0 { self.carried.count } else { 1 };
            let identity = self.next_identity();
            let dropped = self.carried.split(amount, identity);
            if !dropped.is_empty() {
                self.dropped.push(dropped);
            }
            return Ok(());
        }
        let index = self.slot_index(slot_index)?;
        let identity = self.next_identity();
        let slot = &mut self.slots[index];

        if slot.stack.is_empty() {
            let amount = if button == 0 { self.carried.count } else { 1 };
            slot.safe_insert(&mut self.carried, amount);
            return Ok(());
        }
        if !slot.policy.may_pickup {
            return Ok(());
        }
        if self.carried.is_empty() {
            let amount = if button == 0 {
                slot.stack.count
            } else {
                (slot.stack.count + 1) / 2
            };
            self.carried = slot.safe_take(amount, i32::MAX, identity);
            return Ok(());
        }
        if slot.policy.may_place {
            if slot.stack.compatible_with(&self.carried) {
                let amount = if button == 0 { self.carried.count } else { 1 };
                slot.safe_insert(&mut self.carried, amount);
            } else if self.carried.count <= slot.maximum_for(&self.carried) {
                std::mem::swap(&mut slot.stack, &mut self.carried);
                slot.changed = true;
            }
        } else if slot.stack.compatible_with(&self.carried) {
            let available = self.carried.maximum - self.carried.count;
            let pulled = slot.safe_take(available, available, identity);
            if !pulled.is_empty() {
                self.carried.grow(pulled.count);
            }
        }
        Ok(())
    }

    fn quick_move(
        &mut self,
        slot_index: i32,
        button: i32,
        layout: &MenuLayout,
    ) -> Result<(), ClickError> {
        if !matches!(button, 0 | 1) {
            return Ok(());
        }
        if slot_index == -999 {
            return self.pickup(slot_index, button);
        }
        let index = self.slot_index(slot_index)?;
        if !self.slots[index].policy.may_pickup {
            return Ok(());
        }
        let Some(target) = layout.simple_quick_move_target(index) else {
            return Ok(());
        };

        loop {
            let snapshot = self.slots[index].stack.clone();
            if snapshot.is_empty() {
                break;
            }
            let mut source = std::mem::replace(&mut self.slots[index].stack, ItemStack::empty());
            let report = move_item_stack_to(
                &mut source,
                &mut self.slots,
                target.range.clone(),
                target.reverse,
            );
            self.slots[index].stack = source;
            if report.moved == 0 {
                break;
            }
            self.slots[index].changed = true;
            if self.slots[index].stack.is_empty() || self.slots[index].stack.item != snapshot.item {
                break;
            }
        }
        Ok(())
    }

    fn swap(
        &mut self,
        slot_index: i32,
        button: i32,
        player: &mut ClickPlayer,
    ) -> Result<(), ClickError> {
        if !matches!(button, 0..=8 | 40) {
            return Ok(());
        }
        let index = self.slot_index(slot_index)?;
        let selected = player
            .inventory
            .get_mut(button as usize)
            .ok_or(ClickError::PlayerInventoryIndex(button))?;
        let slot = &mut self.slots[index];

        if selected.is_empty() {
            if slot.policy.may_pickup {
                slot.swap_craft_items += slot.stack.count;
                *selected = std::mem::replace(&mut slot.stack, ItemStack::empty());
                if !selected.is_empty() {
                    slot.take_calls += 1;
                    slot.changed = true;
                }
            }
            return Ok(());
        }
        if slot.stack.is_empty() {
            if slot.policy.may_place {
                let maximum = slot.maximum_for(selected);
                slot.stack = selected.split(maximum, selected.identity);
                slot.changed = true;
            }
            return Ok(());
        }
        if !slot.policy.may_pickup || !slot.policy.may_place {
            return Ok(());
        }
        let old = slot.stack.clone();
        let maximum = slot.maximum_for(selected);
        if selected.count > maximum {
            slot.stack = selected.split(maximum, selected.identity);
            if selected.is_empty() {
                *selected = old;
            } else {
                self.dropped.push(old);
            }
        } else {
            slot.stack = std::mem::replace(selected, old);
        }
        slot.take_calls += 1;
        slot.changed = true;
        Ok(())
    }

    fn clone_stack(&mut self, slot_index: i32, player: &ClickPlayer) -> Result<(), ClickError> {
        if !player.infinite_materials || !self.carried.is_empty() {
            return Ok(());
        }
        let index = self.slot_index(slot_index)?;
        if !self.slots[index].stack.is_empty() {
            self.carried = self.slots[index].stack.clone();
            self.carried.count = self.carried.maximum;
        }
        Ok(())
    }

    fn throw(
        &mut self,
        slot_index: i32,
        button: i32,
        player: &ClickPlayer,
    ) -> Result<(), ClickError> {
        if !self.carried.is_empty() || !player.can_drop_items {
            return Ok(());
        }
        let index = self.slot_index(slot_index)?;
        loop {
            let requested = if button == 0 {
                1
            } else {
                self.slots[index].stack.count
            };
            let identity = self.next_identity();
            let removed = self.slots[index].safe_take(requested, i32::MAX, identity);
            if removed.is_empty() {
                break;
            }
            let item = removed.item.clone();
            self.dropped.push(removed);
            if button != 1 || self.slots[index].stack.item != item {
                break;
            }
        }
        Ok(())
    }

    fn pickup_all(&mut self, slot_index: i32, button: i32) -> Result<(), ClickError> {
        let clicked = self.slot_index(slot_index)?;
        if self.carried.is_empty()
            || (!self.slots[clicked].stack.is_empty() && self.slots[clicked].policy.may_pickup)
        {
            return Ok(());
        }
        let mut indices = (0..self.slots.len()).collect::<Vec<_>>();
        if button != 0 {
            indices.reverse();
        }
        for pass in 0..2 {
            for &index in &indices {
                let slot = &self.slots[index];
                if slot.stack.is_empty()
                    || !slot.stack.compatible_with(&self.carried)
                    || !slot.policy.may_pickup
                    || (pass == 0 && slot.stack.count == slot.stack.maximum)
                {
                    continue;
                }
                let available = self.carried.maximum - self.carried.count;
                let identity = self.next_identity();
                let removed = self.slots[index].safe_take(available, available, identity);
                self.carried.grow(removed.count);
                if self.carried.count >= self.carried.maximum {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn quick_craft(
        &mut self,
        slot_index: i32,
        button: i32,
        player: &ClickPlayer,
    ) -> Result<(), ClickError> {
        let header = (button & 3) as u8;
        let kind = ((button >> 2) & 3) as u8;
        match header {
            0 if self.quick_craft.status == 0 => {
                self.quick_craft.kind = kind;
                self.quick_craft.status = 1;
                self.quick_craft.selected.clear();
                if self.carried.is_empty() || kind == 3 || (kind == 2 && !player.infinite_materials)
                {
                    self.reset_quick_craft();
                }
            }
            1 if self.quick_craft.status == 1 => {
                let index = self.slot_index(slot_index)?;
                let slot = &self.slots[index];
                if slot.policy.may_place
                    && (slot.stack.is_empty() || slot.stack.compatible_with(&self.carried))
                    && (kind == 2 || self.carried.count > self.quick_craft.selected.len() as i32)
                {
                    self.quick_craft.selected.insert(index);
                }
            }
            2 if self.quick_craft.status == 1 => self.finish_quick_craft(player)?,
            _ => self.reset_quick_craft(),
        }
        Ok(())
    }

    fn finish_quick_craft(&mut self, player: &ClickPlayer) -> Result<(), ClickError> {
        let selected = self
            .quick_craft
            .selected
            .iter()
            .copied()
            .collect::<Vec<_>>();
        if selected.len() == 1 {
            let slot = selected[0] as i32;
            let kind = i32::from(self.quick_craft.kind);
            self.reset_quick_craft();
            return self.pickup(slot, kind);
        }
        if selected.len() < 2 {
            self.reset_quick_craft();
            return Ok(());
        }
        let original_count = self.carried.count;
        let kind = self.quick_craft.kind;
        let per_slot = match kind {
            0 => (original_count as f32 / selected.len() as f32).floor() as i32,
            1 => 1,
            2 if player.infinite_materials => self.carried.maximum,
            _ => {
                self.reset_quick_craft();
                return Ok(());
            }
        };
        let mut remaining = original_count;
        for index in selected {
            let slot = &mut self.slots[index];
            if !slot.policy.may_place
                || (!slot.stack.is_empty() && !slot.stack.compatible_with(&self.carried))
            {
                continue;
            }
            let existing = if slot.stack.is_empty() {
                0
            } else {
                slot.stack.count
            };
            let target = (existing + per_slot)
                .min(slot.maximum_for(&self.carried))
                .min(self.carried.maximum);
            let added = target - existing;
            remaining -= added;
            if existing == 0 {
                slot.stack = self.carried.copy_with_identity(self.carried.identity);
            }
            slot.stack.count = target;
            slot.changed = true;
        }
        self.carried.count = remaining;
        if self.carried.count <= 0 {
            self.carried = ItemStack::empty();
        }
        self.reset_quick_craft();
        Ok(())
    }

    fn slot_index(&self, index: i32) -> Result<usize, ClickError> {
        usize::try_from(index)
            .ok()
            .filter(|&index| index < self.slots.len())
            .ok_or(ClickError::SlotIndex(index))
    }

    fn next_identity(&mut self) -> u64 {
        let identity = self.next_identity;
        self.next_identity = self.next_identity.wrapping_add(1);
        identity
    }

    fn reset_quick_craft(&mut self) {
        self.quick_craft = QuickCraftState::default();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickError {
    SlotIndex(i32),
    PlayerInventoryIndex(i32),
}
