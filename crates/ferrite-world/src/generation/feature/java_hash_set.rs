//! Java `HashSet<BlockPos>` insertion, retention, and bucket iteration semantics.

use ferrite_foundation::coordinate::BlockPos;

#[derive(Debug, Clone)]
pub(crate) struct JavaBlockPosSet {
    entries: Vec<BlockPos>,
    capacity: usize,
}

impl JavaBlockPosSet {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            capacity: 0,
        }
    }

    pub(crate) fn insert(&mut self, position: BlockPos) {
        if self.entries.contains(&position) {
            return;
        }
        self.entries.push(position);
        if self.capacity == 0 {
            self.capacity = 16;
        }
        while self.entries.len() > self.capacity * 3 / 4 {
            self.capacity *= 2;
        }
    }

    pub(crate) fn retain(&mut self, mut predicate: impl FnMut(BlockPos) -> bool) {
        let ordered = self.iter().collect::<Vec<_>>();
        let retained = ordered
            .into_iter()
            .filter(|position| predicate(*position))
            .collect::<Vec<_>>();
        self.entries.retain(|position| retained.contains(position));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = BlockPos> + '_ {
        let capacity = self.capacity.max(1);
        (0..capacity).flat_map(move |bucket| {
            self.entries
                .iter()
                .copied()
                .filter(move |position| java_bucket(*position, capacity) == bucket)
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn contains(&self, position: &BlockPos) -> bool {
        self.entries.contains(position)
    }
}

fn java_bucket(position: BlockPos, capacity: usize) -> usize {
    let hash = position
        .z
        .wrapping_mul(31)
        .wrapping_add(position.y)
        .wrapping_mul(31)
        .wrapping_add(position.x) as u32;
    let spread = hash ^ (hash >> 16);
    spread as usize & (capacity - 1)
}
