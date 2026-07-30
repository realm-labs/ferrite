//! Fastutil-8.5.18-compatible simulation-level storage and iteration order.

use ferrite_foundation::coordinate::ChunkPos;

pub const ABSENT_SIMULATION_LEVEL: u8 = 33;
pub const ENTITY_TICKING_LEVEL: u8 = 31;
pub const BLOCK_TICKING_LEVEL: u8 = 32;

const INITIAL_CAPACITY: usize = 32;
const MINIMUM_CAPACITY: usize = 32;
const LOAD_NUMERATOR: usize = 3;
const LOAD_DENOMINATOR: usize = 4;
const HASH_MULTIPLIER: i64 = -7_046_029_254_386_353_131;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationChunkTracker {
    keys: Vec<i64>,
    levels: Vec<u8>,
    mask: usize,
    size: usize,
    maximum_fill: usize,
    contains_zero_key: bool,
}

impl Default for SimulationChunkTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationChunkTracker {
    pub fn new() -> Self {
        Self {
            keys: vec![0; INITIAL_CAPACITY + 1],
            levels: vec![ABSENT_SIMULATION_LEVEL; INITIAL_CAPACITY + 1],
            mask: INITIAL_CAPACITY - 1,
            size: 0,
            maximum_fill: maximum_fill(INITIAL_CAPACITY),
            contains_zero_key: false,
        }
    }

    pub const fn len(&self) -> usize {
        self.size
    }

    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn level(&self, chunk: ChunkPos) -> u8 {
        self.level_by_key(pack_chunk(chunk))
    }

    pub fn level_by_key(&self, key: i64) -> u8 {
        if key == 0 {
            return if self.contains_zero_key {
                self.levels[self.mask + 1]
            } else {
                ABSENT_SIMULATION_LEVEL
            };
        }
        let mut position = mixed_slot(key, self.mask);
        loop {
            let current = self.keys[position];
            if current == 0 {
                return ABSENT_SIMULATION_LEVEL;
            }
            if current == key {
                return self.levels[position];
            }
            position = (position + 1) & self.mask;
        }
    }

    pub fn set_level(&mut self, chunk: ChunkPos, level: u8) {
        self.set_level_by_key(pack_chunk(chunk), level);
    }

    pub fn set_level_by_key(&mut self, key: i64, level: u8) {
        if level >= ABSENT_SIMULATION_LEVEL {
            self.remove(key);
        } else {
            self.put(key, level);
        }
    }

    pub fn compatibility_order(&self) -> Vec<(ChunkPos, u8)> {
        self.compatibility_key_order()
            .into_iter()
            .map(|(key, level)| (unpack_chunk(key), level))
            .collect()
    }

    pub fn compatibility_key_order(&self) -> Vec<(i64, u8)> {
        let mut ordered = Vec::with_capacity(self.size);
        if self.contains_zero_key {
            ordered.push((0, self.levels[self.mask + 1]));
        }
        for position in (0..=self.mask).rev() {
            let key = self.keys[position];
            if key != 0 {
                ordered.push((key, self.levels[position]));
            }
        }
        ordered
    }

    fn put(&mut self, key: i64, level: u8) {
        let position = if key == 0 {
            if self.contains_zero_key {
                self.levels[self.mask + 1] = level;
                return;
            }
            self.contains_zero_key = true;
            self.mask + 1
        } else {
            let mut position = mixed_slot(key, self.mask);
            loop {
                let current = self.keys[position];
                if current == 0 {
                    self.keys[position] = key;
                    break position;
                }
                if current == key {
                    self.levels[position] = level;
                    return;
                }
                position = (position + 1) & self.mask;
            }
        };
        self.levels[position] = level;
        let old_size = self.size;
        self.size += 1;
        if old_size >= self.maximum_fill {
            self.rehash(array_size(self.size + 1));
        }
    }

    fn remove(&mut self, key: i64) {
        if key == 0 {
            if !self.contains_zero_key {
                return;
            }
            self.contains_zero_key = false;
            self.size -= 1;
            self.shrink_if_needed();
            return;
        }
        let mut position = mixed_slot(key, self.mask);
        loop {
            let current = self.keys[position];
            if current == 0 {
                return;
            }
            if current == key {
                self.size -= 1;
                self.shift_keys(position);
                self.shrink_if_needed();
                return;
            }
            position = (position + 1) & self.mask;
        }
    }

    fn shift_keys(&mut self, mut position: usize) {
        loop {
            let last = position;
            position = (position + 1) & self.mask;
            loop {
                let current = self.keys[position];
                if current == 0 {
                    self.keys[last] = 0;
                    return;
                }
                let slot = mixed_slot(current, self.mask);
                let must_move = if last <= position {
                    last >= slot || slot > position
                } else {
                    last >= slot && slot > position
                };
                if must_move {
                    break;
                }
                position = (position + 1) & self.mask;
            }
            self.keys[last] = self.keys[position];
            self.levels[last] = self.levels[position];
        }
    }

    fn shrink_if_needed(&mut self) {
        let capacity = self.mask + 1;
        if capacity > MINIMUM_CAPACITY && self.size < self.maximum_fill / 4 && capacity > 16 {
            self.rehash(capacity / 2);
        }
    }

    fn rehash(&mut self, new_capacity: usize) {
        let mut new_keys = vec![0; new_capacity + 1];
        let mut new_levels = vec![ABSENT_SIMULATION_LEVEL; new_capacity + 1];
        let new_mask = new_capacity - 1;
        let mut remaining = self.size - usize::from(self.contains_zero_key);
        let mut scan = self.mask + 1;
        while remaining != 0 {
            loop {
                scan -= 1;
                if self.keys[scan] != 0 {
                    break;
                }
            }
            let key = self.keys[scan];
            let mut position = mixed_slot(key, new_mask);
            while new_keys[position] != 0 {
                position = (position + 1) & new_mask;
            }
            new_keys[position] = key;
            new_levels[position] = self.levels[scan];
            remaining -= 1;
        }
        new_levels[new_capacity] = self.levels[self.mask + 1];
        self.keys = new_keys;
        self.levels = new_levels;
        self.mask = new_mask;
        self.maximum_fill = maximum_fill(new_capacity);
    }
}

pub const fn is_entity_ticking(level: u8) -> bool {
    level <= ENTITY_TICKING_LEVEL
}

pub const fn is_block_ticking(level: u8) -> bool {
    level <= BLOCK_TICKING_LEVEL
}

pub const fn player_simulation_source_level(simulation_distance: u8) -> u8 {
    ENTITY_TICKING_LEVEL.saturating_sub(simulation_distance)
}

pub const fn pack_chunk(chunk: ChunkPos) -> i64 {
    (chunk.x as u32 as i64) | ((chunk.z as u32 as i64) << 32)
}

pub const fn unpack_chunk(key: i64) -> ChunkPos {
    ChunkPos::new(key as i32, (key >> 32) as i32)
}

fn mixed_slot(key: i64, mask: usize) -> usize {
    let mut hash = key.wrapping_mul(HASH_MULTIPLIER) as u64;
    hash ^= hash >> 32;
    hash ^= hash >> 16;
    hash as usize & mask
}

const fn maximum_fill(capacity: usize) -> usize {
    let ceiling = (capacity * LOAD_NUMERATOR).div_ceil(LOAD_DENOMINATOR);
    if ceiling < capacity {
        ceiling
    } else {
        capacity - 1
    }
}

fn array_size(expected: usize) -> usize {
    let required = expected
        .saturating_mul(LOAD_DENOMINATOR)
        .div_ceil(LOAD_NUMERATOR)
        .next_power_of_two();
    required.max(2)
}
