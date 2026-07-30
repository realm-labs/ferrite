//! One loaded chunk's scheduled-tick queue and deduplication index.

use crate::scheduled_tick::record::{SavedTick, ScheduledTick};
use ferrite_foundation::coordinate::BlockPos;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::hash::Hash;

#[derive(Debug)]
struct HeapTick<I>(ScheduledTick<I>);

impl<I> PartialEq for HeapTick<I> {
    fn eq(&self, other: &Self) -> bool {
        self.0.local_order(&other.0) == Ordering::Equal
    }
}

impl<I> Eq for HeapTick<I> {}

impl<I> PartialOrd for HeapTick<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<I> Ord for HeapTick<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.local_order(&self.0)
    }
}

#[derive(Debug)]
pub struct ChunkTickContainer<I> {
    queue: BinaryHeap<HeapTick<I>>,
    pending: Option<Vec<SavedTick<I>>>,
    membership: HashSet<(I, BlockPos)>,
}

impl<I> Default for ChunkTickContainer<I>
where
    I: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<I> ChunkTickContainer<I>
where
    I: Clone + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            pending: None,
            membership: HashSet::new(),
        }
    }

    pub fn from_saved(pending: Vec<SavedTick<I>>) -> Self {
        let membership = pending
            .iter()
            .map(|tick| (tick.type_identity.clone(), tick.position))
            .collect();
        Self {
            queue: BinaryHeap::new(),
            pending: Some(pending),
            membership,
        }
    }

    pub fn schedule(&mut self, tick: ScheduledTick<I>) -> bool {
        let key = (tick.type_identity.clone(), tick.position);
        if !self.membership.insert(key) {
            return false;
        }
        self.schedule_unchecked(tick);
        true
    }

    pub fn has_scheduled_tick(&self, position: BlockPos, type_identity: &I) -> bool {
        self.membership.contains(&(type_identity.clone(), position))
    }

    pub fn remove_if(&mut self, mut predicate: impl FnMut(&ScheduledTick<I>) -> bool) {
        let mut retained = BinaryHeap::with_capacity(self.queue.len());
        while let Some(entry) = self.queue.pop() {
            if predicate(&entry.0) {
                self.membership
                    .remove(&(entry.0.type_identity.clone(), entry.0.position));
            } else {
                retained.push(entry);
            }
        }
        self.queue = retained;
    }

    pub fn all_ticks(&self) -> Vec<ScheduledTick<I>> {
        self.queue.iter().map(|entry| entry.0.clone()).collect()
    }

    pub fn count(&self) -> usize {
        self.queue.len() + self.pending.as_ref().map_or(0, Vec::len)
    }

    pub fn pack(&self, current_tick: i64) -> Vec<SavedTick<I>> {
        let mut packed = self.pending.clone().unwrap_or_default();
        let mut scheduled = self.all_ticks();
        scheduled.sort_by_key(|tick| tick.sub_tick_order);
        packed.extend(scheduled.iter().map(|tick| tick.to_saved(current_tick)));
        packed
    }

    pub fn unpack(&mut self, current_tick: i64) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let mut sub_tick_order = (pending.len() as i32).wrapping_neg();
        for tick in pending {
            self.schedule_unchecked(tick.unpack(current_tick, i64::from(sub_tick_order)));
            sub_tick_order = sub_tick_order.wrapping_add(1);
        }
    }

    pub(crate) fn peek(&self) -> Option<&ScheduledTick<I>> {
        self.queue.peek().map(|entry| &entry.0)
    }

    pub(crate) fn poll(&mut self) -> Option<ScheduledTick<I>> {
        let tick = self.queue.pop()?.0;
        self.membership
            .remove(&(tick.type_identity.clone(), tick.position));
        Some(tick)
    }

    fn schedule_unchecked(&mut self, tick: ScheduledTick<I>) {
        self.queue.push(HeapTick(tick));
    }
}
