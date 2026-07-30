//! Ordered container slots, transfer primitives, selection, and comparator projection.

use crate::item::runtime::stack::ItemStack;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotPolicy {
    pub may_pickup: bool,
    pub may_place: bool,
    pub allow_modification: bool,
    pub maximum: i32,
}

impl Default for SlotPolicy {
    fn default() -> Self {
        Self {
            may_pickup: true,
            may_place: true,
            allow_modification: true,
            maximum: 99,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    pub stack: ItemStack,
    pub policy: SlotPolicy,
    pub changed: bool,
    pub take_calls: u32,
    pub quick_craft_calls: u32,
    pub swap_craft_items: i32,
}

impl Slot {
    pub fn empty() -> Self {
        Self {
            stack: ItemStack::empty(),
            policy: SlotPolicy::default(),
            changed: false,
            take_calls: 0,
            quick_craft_calls: 0,
            swap_craft_items: 0,
        }
    }

    pub fn with_stack(stack: ItemStack) -> Self {
        Self {
            stack,
            ..Self::empty()
        }
    }

    pub fn maximum_for(&self, stack: &ItemStack) -> i32 {
        self.policy.maximum.min(stack.maximum)
    }

    pub fn safe_take(&mut self, requested: i32, maximum: i32, identity: u64) -> ItemStack {
        if !self.policy.may_pickup || self.stack.is_empty() {
            return ItemStack::empty();
        }
        if !self.policy.allow_modification && maximum < self.stack.count {
            return ItemStack::empty();
        }
        let removed = self.stack.split(requested.min(maximum), identity);
        if !removed.is_empty() {
            self.take_calls += 1;
            self.changed = true;
        }
        removed
    }

    pub fn safe_insert(&mut self, input: &mut ItemStack, requested: i32) -> i32 {
        if input.is_empty() || !self.policy.may_place {
            return 0;
        }
        let maximum = self.maximum_for(input);
        let available = if self.stack.is_empty() {
            maximum
        } else if self.stack.compatible_with(input) {
            maximum - self.stack.count
        } else {
            0
        };
        let moved = requested.min(input.count).min(available).max(0);
        if moved == 0 {
            return 0;
        }
        if self.stack.is_empty() {
            self.stack = input.split(moved, input.identity);
        } else {
            self.stack.grow(moved);
            input.shrink(moved);
        }
        self.changed = true;
        moved
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    pub slots: Vec<Slot>,
    pub changed_calls: u32,
}

impl Inventory {
    pub fn empty(size: usize) -> Self {
        Self {
            slots: (0..size).map(|_| Slot::empty()).collect(),
            changed_calls: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|slot| slot.stack.is_empty())
    }

    pub fn full_by_exact_maximum(&self) -> bool {
        self.slots
            .iter()
            .all(|slot| slot.stack.count == slot.stack.maximum)
    }

    pub fn comparator_output(&self, container_maximum: i32) -> u8 {
        if self.slots.is_empty() {
            return 0;
        }
        let fullness = self
            .slots
            .iter()
            .filter(|slot| !slot.stack.is_empty())
            .map(|slot| {
                slot.stack.count as f32 / container_maximum.min(slot.stack.maximum).max(1) as f32
            })
            .sum::<f32>()
            / self.slots.len() as f32;
        if fullness == 0.0 {
            0
        } else {
            (fullness * 14.0).floor() as u8 + 1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MoveReport {
    pub moved: i32,
    pub changed_slots: Vec<usize>,
}

pub fn move_item_stack_to(
    source: &mut ItemStack,
    slots: &mut [Slot],
    range: Range<usize>,
    reverse: bool,
) -> MoveReport {
    if source.is_empty() || range.start > range.end || range.end > slots.len() {
        return MoveReport::default();
    }
    let indices = ordered_indices(range, reverse);
    let mut report = MoveReport::default();

    if source.maximum > 1 {
        for &index in &indices {
            let slot = &mut slots[index];
            if slot.stack.is_empty() || !slot.stack.compatible_with(source) {
                continue;
            }
            let available = slot.maximum_for(source) - slot.stack.count;
            let moved = available.min(source.count).max(0);
            if moved > 0 {
                slot.stack.grow(moved);
                source.shrink(moved);
                slot.changed = true;
                report.moved += moved;
                report.changed_slots.push(index);
            }
            if source.is_empty() {
                return report;
            }
        }
    }

    for index in indices {
        let slot = &mut slots[index];
        if !slot.stack.is_empty() || !slot.policy.may_place {
            continue;
        }
        let moved = source.count.min(slot.maximum_for(source)).max(0);
        if moved > 0 {
            slot.stack = source.split(moved, source.identity);
            slot.changed = true;
            report.moved += moved;
            report.changed_slots.push(index);
        }
        break;
    }
    report
}

fn ordered_indices(range: Range<usize>, reverse: bool) -> Vec<usize> {
    if reverse {
        range.rev().collect()
    } else {
        range.collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionError {
    MissingDraw,
    DrawOutOfRange { draw: usize, bound: usize },
}

pub fn select_random_occupied(
    slots: &[Slot],
    draws: &[usize],
) -> Result<Option<usize>, SelectionError> {
    let mut selected = None;
    let mut occupied = 0;
    for (index, slot) in slots.iter().enumerate() {
        if slot.stack.is_empty() {
            continue;
        }
        occupied += 1;
        let Some(&draw) = draws.get(occupied - 1) else {
            return Err(SelectionError::MissingDraw);
        };
        if draw >= occupied {
            return Err(SelectionError::DrawOutOfRange {
                draw,
                bound: occupied,
            });
        }
        if draw == 0 {
            selected = Some(index);
        }
    }
    Ok(selected)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferPolicy {
    pub source_may_take: bool,
    pub destination_may_place: bool,
}

impl Default for TransferPolicy {
    fn default() -> Self {
        Self {
            source_may_take: true,
            destination_may_place: true,
        }
    }
}

pub fn transfer_one(
    source: &mut Inventory,
    source_slot: usize,
    destination: &mut Inventory,
    destination_slots: &[usize],
    policy: TransferPolicy,
    split_identity: u64,
) -> bool {
    let Some(source_entry) = source.slots.get_mut(source_slot) else {
        return false;
    };
    if source_entry.stack.is_empty() || !source_entry.policy.may_pickup || !policy.source_may_take {
        return false;
    }

    let original = source_entry.stack.clone();
    let mut incoming = source_entry.stack.split(1, split_identity);
    for &index in destination_slots {
        let Some(slot) = destination.slots.get_mut(index) else {
            continue;
        };
        if !policy.destination_may_place || !slot.policy.may_place {
            continue;
        }
        if slot.safe_insert(&mut incoming, 1) == 1 {
            destination.changed_calls += 1;
            source.changed_calls += 1;
            return true;
        }
    }

    source.slots[source_slot].stack = original;
    false
}
