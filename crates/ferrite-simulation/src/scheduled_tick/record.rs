//! Scheduled-tick ordering, creation, and persistence records.

use ferrite_foundation::coordinate::BlockPos;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[repr(i8)]
pub enum TickPriority {
    ExtremelyHigh = -3,
    VeryHigh = -2,
    High = -1,
    #[default]
    Normal = 0,
    Low = 1,
    VeryLow = 2,
    ExtremelyLow = 3,
}

impl TickPriority {
    pub const fn from_value(value: i32) -> Self {
        match value {
            i32::MIN..=-3 => Self::ExtremelyHigh,
            -2 => Self::VeryHigh,
            -1 => Self::High,
            0 => Self::Normal,
            1 => Self::Low,
            2 => Self::VeryLow,
            3..=i32::MAX => Self::ExtremelyLow,
        }
    }

    pub const fn value(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledTick<I> {
    pub type_identity: I,
    pub position: BlockPos,
    pub trigger_tick: i64,
    pub priority: TickPriority,
    pub sub_tick_order: i64,
}

impl<I> ScheduledTick<I> {
    pub const fn new(
        type_identity: I,
        position: BlockPos,
        trigger_tick: i64,
        priority: TickPriority,
        sub_tick_order: i64,
    ) -> Self {
        Self {
            type_identity,
            position,
            trigger_tick,
            priority,
            sub_tick_order,
        }
    }

    pub fn to_saved(&self, current_tick: i64) -> SavedTick<I>
    where
        I: Clone,
    {
        SavedTick {
            type_identity: self.type_identity.clone(),
            position: self.position,
            delay: self.trigger_tick.wrapping_sub(current_tick) as i32,
            priority: self.priority,
        }
    }

    pub(crate) fn local_order(&self, other: &Self) -> Ordering {
        self.trigger_tick
            .cmp(&other.trigger_tick)
            .then_with(|| self.intra_tick_order(other))
    }

    pub(crate) fn intra_tick_order(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.sub_tick_order.cmp(&other.sub_tick_order))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedTick<I> {
    pub type_identity: I,
    pub position: BlockPos,
    pub delay: i32,
    pub priority: TickPriority,
}

impl<I> SavedTick<I> {
    pub const fn new(
        type_identity: I,
        position: BlockPos,
        delay: i32,
        priority: TickPriority,
    ) -> Self {
        Self {
            type_identity,
            position,
            delay,
            priority,
        }
    }

    pub fn unpack(self, current_tick: i64, sub_tick_order: i64) -> ScheduledTick<I> {
        ScheduledTick {
            type_identity: self.type_identity,
            position: self.position,
            trigger_tick: current_tick.wrapping_add(i64::from(self.delay)),
            priority: self.priority,
            sub_tick_order,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SubTickCounter {
    next: i64,
}

impl SubTickCounter {
    pub const fn new(next: i64) -> Self {
        Self { next }
    }

    pub const fn value(self) -> i64 {
        self.next
    }

    pub fn take(&mut self) -> i64 {
        let current = self.next;
        self.next = self.next.wrapping_add(1);
        current
    }

    pub fn create<I>(
        &mut self,
        type_identity: I,
        position: BlockPos,
        game_time: i64,
        delay: i32,
        priority: TickPriority,
    ) -> ScheduledTick<I> {
        ScheduledTick::new(
            type_identity,
            position,
            game_time.wrapping_add(i64::from(delay)),
            priority,
            self.take(),
        )
    }
}
