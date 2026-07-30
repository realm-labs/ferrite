//! Chiseled-bookshelf interaction, storage, and state-divergence rules.

use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::direction::Direction;

pub const CHISELED_BOOKSHELF_SLOTS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookshelfUse {
    Pass,
    TryWithEmptyHand,
    Consume,
    Insert { slot: usize },
    Remove { slot: usize },
}

pub fn use_with_item(
    held: &ItemStack,
    facing: Direction,
    hit_face: Direction,
    relative: [f32; 3],
    captured_occupancy: [bool; CHISELED_BOOKSHELF_SLOTS],
) -> BookshelfUse {
    if !is_bookshelf_book(held) {
        return BookshelfUse::TryWithEmptyHand;
    }
    let Some(slot) = hit_slot(facing, hit_face, relative) else {
        return BookshelfUse::Pass;
    };
    if captured_occupancy[slot] {
        BookshelfUse::TryWithEmptyHand
    } else {
        BookshelfUse::Insert { slot }
    }
}

pub fn use_empty_hand(
    facing: Direction,
    hit_face: Direction,
    relative: [f32; 3],
    captured_occupancy: [bool; CHISELED_BOOKSHELF_SLOTS],
) -> BookshelfUse {
    let Some(slot) = hit_slot(facing, hit_face, relative) else {
        return BookshelfUse::Pass;
    };
    if captured_occupancy[slot] {
        BookshelfUse::Remove { slot }
    } else {
        BookshelfUse::Consume
    }
}

pub fn hit_slot(facing: Direction, hit_face: Direction, relative: [f32; 3]) -> Option<usize> {
    if hit_face != facing || relative.iter().any(|value| !value.is_finite()) {
        return None;
    }
    let horizontal = match facing {
        Direction::North => 1.0_f32 - relative[0],
        Direction::South => relative[0],
        Direction::West => relative[2],
        Direction::East => 1.0_f32 - relative[2],
        Direction::Down | Direction::Up => return None,
    };
    let column = section(horizontal, 3);
    let row = section(1.0_f32 - relative[1], 2);
    Some(row * 3 + column)
}

fn section(coordinate: f32, sections: usize) -> usize {
    ((coordinate * 16.0_f32 / (16.0_f32 / sections as f32)).floor() as isize)
        .clamp(0, sections as isize - 1) as usize
}

pub fn is_bookshelf_book(stack: &ItemStack) -> bool {
    stack.item.as_ref().is_some_and(|item| {
        item.namespace() == "minecraft"
            && matches!(
                item.path(),
                "book" | "written_book" | "enchanted_book" | "writable_book" | "knowledge_book"
            )
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShelfMutation {
    pub accepted: bool,
    pub offered_state: Option<[bool; CHISELED_BOOKSHELF_SLOTS]>,
    pub state_changed: bool,
    pub state_write_succeeded: bool,
    pub unsourced_block_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChiseledBookshelf {
    slots: [ItemStack; CHISELED_BOOKSHELF_SLOTS],
    occupied_state: [bool; CHISELED_BOOKSHELF_SLOTS],
    last_interacted_slot: i32,
}

impl ChiseledBookshelf {
    pub fn empty() -> Self {
        Self {
            slots: std::array::from_fn(|_| ItemStack::empty()),
            occupied_state: [false; CHISELED_BOOKSHELF_SLOTS],
            last_interacted_slot: -1,
        }
    }

    pub const fn slots(&self) -> &[ItemStack; CHISELED_BOOKSHELF_SLOTS] {
        &self.slots
    }

    pub const fn occupied_state(&self) -> [bool; CHISELED_BOOKSHELF_SLOTS] {
        self.occupied_state
    }

    pub const fn last_interacted_slot(&self) -> i32 {
        self.last_interacted_slot
    }

    pub const fn comparator_output(&self, server_side: bool) -> i32 {
        if server_side {
            self.last_interacted_slot + 1
        } else {
            0
        }
    }

    pub fn can_place_item(&self, slot: usize, stack: &ItemStack) -> bool {
        self.slots.get(slot).is_some_and(ItemStack::is_empty) && is_bookshelf_book(stack)
    }

    pub fn can_take_to_destination(destination_has_empty_slot: bool) -> bool {
        destination_has_empty_slot
    }

    pub fn set_item(
        &mut self,
        slot: usize,
        stack: ItemStack,
        state_write_succeeds: bool,
    ) -> ShelfMutation {
        if slot >= CHISELED_BOOKSHELF_SLOTS || (!stack.is_empty() && !is_bookshelf_book(&stack)) {
            return rejected_mutation();
        }
        if stack.is_empty() {
            return self.remove_item(slot, state_write_succeeds).1;
        }
        self.slots[slot] = stack;
        self.update_state(slot, state_write_succeeds)
    }

    pub fn remove_item(
        &mut self,
        slot: usize,
        state_write_succeeds: bool,
    ) -> (ItemStack, ShelfMutation) {
        let Some(stored) = self.slots.get_mut(slot) else {
            return (ItemStack::empty(), rejected_mutation());
        };
        if stored.is_empty() {
            return (ItemStack::empty(), rejected_mutation());
        }
        let removed = std::mem::replace(stored, ItemStack::empty());
        let mutation = self.update_state(slot, state_write_succeeds);
        (removed, mutation)
    }

    pub fn remove_item_no_update(&mut self, slot: usize) -> ItemStack {
        let Some(stored) = self.slots.get_mut(slot) else {
            return ItemStack::empty();
        };
        if stored.is_empty() {
            return ItemStack::empty();
        }
        if stored.count == 1 {
            return std::mem::replace(stored, ItemStack::empty());
        }
        stored.count -= 1;
        let mut removed = stored.clone();
        removed.count = 1;
        removed
    }

    pub fn clear_content(&mut self) {
        self.slots = std::array::from_fn(|_| ItemStack::empty());
    }

    pub fn load_raw(
        &mut self,
        slots: [ItemStack; CHISELED_BOOKSHELF_SLOTS],
        last_interacted_slot: i32,
    ) {
        self.slots = slots;
        self.last_interacted_slot = last_interacted_slot;
    }

    pub fn replace_and_plan_drops(
        &mut self,
        suppress_side_effects: bool,
        bounded_draws: &[u8],
    ) -> Result<BookshelfDropPlan, BookshelfDropError> {
        if suppress_side_effects {
            return Ok(BookshelfDropPlan::default());
        }
        let mut draw_index = 0;
        let mut chunks = Vec::new();
        for slot in &mut self.slots {
            while !slot.is_empty() {
                let Some(&draw) = bounded_draws.get(draw_index) else {
                    return Err(BookshelfDropError::MissingBoundedDraw);
                };
                if draw > 20 {
                    return Err(BookshelfDropError::BoundedDrawOutOfRange(draw));
                }
                draw_index += 1;
                let count = slot.count.min(i32::from(10 + draw));
                let mut chunk = slot.clone();
                chunk.count = count;
                chunks.push(chunk);
                slot.count -= count;
                if slot.count == 0 {
                    *slot = ItemStack::empty();
                }
            }
        }
        Ok(BookshelfDropPlan {
            chunks,
            position_double_draws: 18,
            bounded_integer_draws: draw_index,
            velocity_double_draws: draw_index * 6,
        })
    }

    fn update_state(&mut self, slot: usize, state_write_succeeds: bool) -> ShelfMutation {
        self.last_interacted_slot = slot as i32;
        let offered_state = std::array::from_fn(|index| !self.slots[index].is_empty());
        let state_changed = offered_state != self.occupied_state;
        if state_write_succeeds {
            self.occupied_state = offered_state;
        }
        ShelfMutation {
            accepted: true,
            offered_state: Some(offered_state),
            state_changed,
            state_write_succeeded: state_write_succeeds,
            unsourced_block_change: true,
        }
    }
}

fn rejected_mutation() -> ShelfMutation {
    ShelfMutation {
        accepted: false,
        offered_state: None,
        state_changed: false,
        state_write_succeeded: false,
        unsourced_block_change: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BookshelfDropPlan {
    pub chunks: Vec<ItemStack>,
    pub position_double_draws: usize,
    pub bounded_integer_draws: usize,
    pub velocity_double_draws: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookshelfDropError {
    MissingBoundedDraw,
    BoundedDrawOutOfRange(u8),
}
