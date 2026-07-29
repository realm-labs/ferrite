use ferrite_foundation::coordinate::BlockPos;
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockEvent {
    pub position: BlockPos,
    pub block_type: u32,
    pub event_id: i32,
    pub parameter: i32,
}

#[derive(Debug, Default)]
pub struct BlockEventQueue {
    queued: VecDeque<BlockEvent>,
    identities: BTreeSet<BlockEvent>,
    inactive: Vec<BlockEvent>,
    draining: bool,
}

impl BlockEventQueue {
    pub fn submit(&mut self, event: BlockEvent) -> bool {
        if !self.identities.insert(event) {
            return false;
        }
        self.queued.push_back(event);
        true
    }

    pub fn begin_drain(&mut self) {
        self.draining = true;
    }

    pub fn next_matching<F, G>(
        &mut self,
        mut is_active: F,
        mut current_block: G,
    ) -> Option<BlockEvent>
    where
        F: FnMut(BlockPos) -> bool,
        G: FnMut(BlockPos) -> u32,
    {
        assert!(self.draining, "begin_drain must precede event removal");
        while let Some(event) = self.queued.pop_front() {
            self.identities.remove(&event);
            if !is_active(event.position) {
                self.inactive.push(event);
            } else if current_block(event.position) == event.block_type {
                return Some(event);
            }
        }
        None
    }

    pub fn finish_drain(&mut self) {
        assert!(self.draining, "begin_drain must precede finish_drain");
        self.draining = false;
        for event in self.inactive.drain(..) {
            let inserted = self.identities.insert(event);
            debug_assert!(inserted);
            self.queued.push_back(event);
        }
    }

    pub fn len(&self) -> usize {
        self.queued.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queued.is_empty()
    }
}
