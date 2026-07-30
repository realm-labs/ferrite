//! Hopper cooldown, ordered one-item transfer, and loose-item collection.

use crate::item::runtime::inventory::{
    Inventory, TransferPolicy, move_item_stack_to, transfer_one,
};
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::direction::Direction;

pub const HOPPER_SLOTS: usize = 5;
pub const HOPPER_COOLDOWN: i32 = 8;
pub const SAME_TICK_RECEIVER_COOLDOWN: i32 = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hopper {
    pub inventory: Inventory,
    pub facing: Direction,
    pub enabled: bool,
    pub transfer_cooldown: i32,
    pub last_ticked_game_time: i64,
    pub comparator_updates: u32,
}

impl Hopper {
    pub fn new(facing: Direction) -> Self {
        Self {
            inventory: Inventory::empty(HOPPER_SLOTS),
            facing,
            enabled: true,
            transfer_cooldown: -1,
            last_ticked_game_time: 0,
            comparator_updates: 0,
        }
    }

    pub fn begin_tick(&mut self, game_time: i64) -> HopperTickGate {
        self.transfer_cooldown = self.transfer_cooldown.saturating_sub(1);
        self.last_ticked_game_time = game_time;
        if self.transfer_cooldown > 0 {
            return HopperTickGate::CoolingDown;
        }
        self.transfer_cooldown = 0;
        if self.enabled {
            HopperTickGate::RunTransaction
        } else {
            HopperTickGate::Disabled
        }
    }

    pub fn finish_transaction(&mut self, pushed: bool, pulled: bool) -> bool {
        let changed = pushed | pulled;
        if changed {
            self.transfer_cooldown = HOPPER_COOLDOWN;
            self.inventory.changed_calls += 1;
            self.comparator_updates += 1;
        }
        changed
    }

    pub fn push_to_inventory(
        &mut self,
        destination: &mut Inventory,
        destination_slots: &[usize],
        destination_may_place: bool,
        split_identity: &mut u64,
    ) -> bool {
        if destination_preflight_full(destination, destination_slots) {
            return false;
        }
        for source_slot in 0..self.inventory.slots.len() {
            if self.inventory.slots[source_slot].stack.is_empty() {
                continue;
            }
            let moved = transfer_one(
                &mut self.inventory,
                source_slot,
                destination,
                destination_slots,
                TransferPolicy {
                    source_may_take: true,
                    destination_may_place,
                },
                next_identity(split_identity),
            );
            if moved {
                destination.changed_calls += 1;
                return true;
            }
        }
        false
    }

    pub fn push_to_hopper(&mut self, destination: &mut Hopper, split_identity: &mut u64) -> bool {
        let destination_was_empty = destination.inventory.is_empty();
        let source_tick = self.last_ticked_game_time;
        let moved = self.push_to_inventory(
            &mut destination.inventory,
            &(0..HOPPER_SLOTS).collect::<Vec<_>>(),
            true,
            split_identity,
        );
        if moved && destination_was_empty && destination.transfer_cooldown <= HOPPER_COOLDOWN {
            destination.transfer_cooldown = if destination.last_ticked_game_time >= source_tick {
                SAME_TICK_RECEIVER_COOLDOWN
            } else {
                HOPPER_COOLDOWN
            };
        }
        moved
    }

    pub fn pull_from_inventory(
        &mut self,
        source: &mut Inventory,
        source_slots: &[usize],
        source_may_take: bool,
        split_identity: &mut u64,
    ) -> bool {
        for &source_slot in source_slots {
            if source
                .slots
                .get(source_slot)
                .is_none_or(|slot| slot.stack.is_empty())
            {
                continue;
            }
            if transfer_one(
                source,
                source_slot,
                &mut self.inventory,
                &(0..HOPPER_SLOTS).collect::<Vec<_>>(),
                TransferPolicy {
                    source_may_take,
                    destination_may_place: true,
                },
                next_identity(split_identity),
            ) {
                return true;
            }
        }
        false
    }

    pub fn collect_loose_item(&mut self, entity: &mut LooseItem) -> LooseCollectionOutcome {
        if entity.discarded || entity.stack.is_empty() {
            return LooseCollectionOutcome::Rejected;
        }
        let before = entity.stack.count;
        let mut remainder = entity.stack.clone();
        let report = move_item_stack_to(
            &mut remainder,
            &mut self.inventory.slots,
            0..HOPPER_SLOTS,
            false,
        );
        if report.moved == 0 {
            return LooseCollectionOutcome::Rejected;
        }
        self.inventory.changed_calls += 1;
        self.comparator_updates += 1;
        entity.stack = remainder;
        if entity.stack.is_empty() {
            entity.discarded = true;
            LooseCollectionOutcome::FullyAbsorbed
        } else {
            LooseCollectionOutcome::PartiallyAbsorbed {
                moved: before - entity.stack.count,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HopperTickGate {
    CoolingDown,
    Disabled,
    RunTransaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LooseItem {
    pub stack: ItemStack,
    pub discarded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LooseCollectionOutcome {
    Rejected,
    PartiallyAbsorbed { moved: i32 },
    FullyAbsorbed,
}

pub fn destination_preflight_full(destination: &Inventory, slots: &[usize]) -> bool {
    !slots.is_empty()
        && slots.iter().all(|&index| {
            destination.slots.get(index).is_some_and(|slot| {
                !slot.stack.is_empty() && slot.stack.count >= slot.stack.maximum
            })
        })
}

pub const fn loose_item_search_allowed(
    source_container_exists: bool,
    above_full_collision: bool,
    above_does_not_block_hoppers: bool,
) -> bool {
    !source_container_exists && (!above_full_collision || above_does_not_block_hoppers)
}

fn next_identity(identity: &mut u64) -> u64 {
    let next = *identity;
    *identity = (*identity).wrapping_add(1);
    next
}
