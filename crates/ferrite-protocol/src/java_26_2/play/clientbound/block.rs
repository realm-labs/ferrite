//! Bounded client-side model for the C2 block convergence family.

use std::collections::BTreeMap;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::java_26_2::play::block::{pack_block_position, unpack_block_position};
use crate::java_26_2::play::clientbound::packet::{
    BlockDestruction, BlockEntityData, BlockEvent, PlayClientboundPacket, SectionBlocksUpdate,
};
use crate::java_26_2::value::nbt::NetworkNbt;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockClientProjection {
    capacity: usize,
    blocks: BTreeMap<BlockPos, i32>,
    block_kinds: BTreeMap<BlockPos, i32>,
    predictions: PredictionTable,
    destruction: BTreeMap<i32, DestructionProgress>,
    block_entities: BTreeMap<BlockPos, ProjectedBlockEntity>,
    last_teleport_sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RetainedPrediction {
    sequence: i32,
    authoritative_state: Option<i32>,
    captured_player_position: [f64; 3],
}

#[derive(Debug, Clone, PartialEq)]
struct PredictionTable {
    capacity: usize,
    values: BTreeMap<i64, RetainedPrediction>,
    layout: FastutilLayout,
}

impl PredictionTable {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            values: BTreeMap::new(),
            layout: FastutilLayout::default(),
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn get_mut(&mut self, position: BlockPos) -> Option<&mut RetainedPrediction> {
        self.values.get_mut(&pack_block_position(position))
    }

    fn insert(
        &mut self,
        position: BlockPos,
        retained: RetainedPrediction,
    ) -> Result<(), BlockProjectionError> {
        if self.values.len() == self.capacity {
            return Err(BlockProjectionError::Full {
                capacity: self.capacity,
            });
        }
        let key = pack_block_position(position);
        self.layout.insert(key, self.values.len() + 1);
        self.values.insert(key, retained);
        Ok(())
    }

    fn remove(&mut self, position: BlockPos) -> Option<RetainedPrediction> {
        let key = pack_block_position(position);
        let removed = self.values.remove(&key)?;
        self.layout.remove(key);
        Some(removed)
    }

    fn removal_order(&self, ack: i32) -> Vec<BlockPos> {
        self.layout
            .qualifying_removal_order(&self.values, ack)
            .into_iter()
            .map(unpack_block_position)
            .collect()
    }
}

const FASTUTIL_LONG_PHI: u64 = 0x9e37_79b9_7f4a_7c15;
const FASTUTIL_LOAD_NUMERATOR: usize = 3;
const FASTUTIL_LOAD_DENOMINATOR: usize = 4;

#[derive(Debug, Clone, PartialEq)]
struct FastutilLayout {
    keys: Vec<i64>,
    contains_null_key: bool,
    max_fill: usize,
}

impl Default for FastutilLayout {
    fn default() -> Self {
        let size = 32;
        Self {
            keys: vec![0; size],
            contains_null_key: false,
            max_fill: max_fill(size),
        }
    }
}

impl FastutilLayout {
    fn insert(&mut self, key: i64, size_after: usize) {
        if key == 0 {
            self.contains_null_key = true;
        } else {
            self.insert_nonzero(key);
        }
        if size_after > self.max_fill {
            self.rehash(array_size(size_after + 1));
        }
    }

    fn remove(&mut self, key: i64) {
        if key == 0 {
            self.contains_null_key = false;
            return;
        }
        let mut position = slot(key, self.keys.len() - 1);
        while self.keys[position] != 0 {
            if self.keys[position] == key {
                self.shift_keys(position);
                return;
            }
            position = (position + 1) & (self.keys.len() - 1);
        }
    }

    fn qualifying_removal_order(
        &self,
        values: &BTreeMap<i64, RetainedPrediction>,
        ack: i32,
    ) -> Vec<i64> {
        let mut layout = self.clone();
        let mut remaining = values.len();
        let mut position = layout.keys.len() as isize;
        let mut return_null = layout.contains_null_key;
        let mut wrapped = Vec::new();
        let mut removed = Vec::new();

        while remaining != 0 {
            let (key, slot_index, from_wrapped) = if return_null {
                return_null = false;
                (0, layout.keys.len(), false)
            } else {
                loop {
                    position -= 1;
                    if position >= 0 {
                        let candidate = layout.keys[position as usize];
                        if candidate != 0 {
                            break (candidate, position as usize, false);
                        }
                    } else {
                        let wrapped_index = (-position - 1) as usize;
                        let candidate = wrapped[wrapped_index];
                        let mut current = slot(candidate, layout.keys.len() - 1);
                        while layout.keys[current] != candidate {
                            current = (current + 1) & (layout.keys.len() - 1);
                        }
                        break (candidate, current, true);
                    }
                }
            };
            remaining -= 1;
            if values
                .get(&key)
                .is_some_and(|retained| retained.sequence <= ack)
            {
                removed.push(key);
                if key == 0 {
                    layout.contains_null_key = false;
                } else if from_wrapped {
                    layout.remove(key);
                } else {
                    layout.iterator_shift_keys(slot_index, &mut wrapped);
                }
            }
        }
        removed
    }

    fn insert_nonzero(&mut self, key: i64) {
        let mask = self.keys.len() - 1;
        let mut position = slot(key, mask);
        while self.keys[position] != 0 {
            position = (position + 1) & mask;
        }
        self.keys[position] = key;
    }

    fn shift_keys(&mut self, mut position: usize) {
        let mask = self.keys.len() - 1;
        loop {
            let last = position;
            position = (position + 1) & mask;
            loop {
                let current = self.keys[position];
                if current == 0 {
                    self.keys[last] = 0;
                    return;
                }
                let home = slot(current, mask);
                let movable = if last <= position {
                    last >= home || home > position
                } else {
                    last >= home && home > position
                };
                if movable {
                    break;
                }
                position = (position + 1) & mask;
            }
            self.keys[last] = self.keys[position];
        }
    }

    fn iterator_shift_keys(&mut self, mut position: usize, wrapped: &mut Vec<i64>) {
        let mask = self.keys.len() - 1;
        loop {
            let last = position;
            position = (position + 1) & mask;
            loop {
                let current = self.keys[position];
                if current == 0 {
                    self.keys[last] = 0;
                    return;
                }
                let home = slot(current, mask);
                let movable = if last <= position {
                    last >= home || home > position
                } else {
                    last >= home && home > position
                };
                if movable {
                    break;
                }
                position = (position + 1) & mask;
            }
            if position < last {
                wrapped.push(self.keys[position]);
            }
            self.keys[last] = self.keys[position];
        }
    }

    fn rehash(&mut self, new_size: usize) {
        let old_keys = std::mem::replace(&mut self.keys, vec![0; new_size]);
        for key in old_keys.into_iter().rev().filter(|key| *key != 0) {
            self.insert_nonzero(key);
        }
        self.max_fill = max_fill(new_size);
    }
}

fn slot(key: i64, mask: usize) -> usize {
    let mut hash = (key as u64).wrapping_mul(FASTUTIL_LONG_PHI);
    hash ^= hash >> 32;
    (hash ^ (hash >> 16)) as usize & mask
}

fn max_fill(size: usize) -> usize {
    ((size * FASTUTIL_LOAD_NUMERATOR).div_ceil(FASTUTIL_LOAD_DENOMINATOR)).min(size - 1)
}

fn array_size(expected: usize) -> usize {
    let required = (expected * FASTUTIL_LOAD_DENOMINATOR)
        .div_ceil(FASTUTIL_LOAD_NUMERATOR)
        .next_power_of_two();
    required.max(2)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DestructionProgress {
    pub position: BlockPos,
    pub progress: u8,
    pub updated_game_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedBlockEntity {
    pub type_raw_id: i32,
    pub update_tag: Option<NetworkNbt>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PredictionResolution {
    pub position: BlockPos,
    pub state: i32,
    pub captured_player_position: Option<[f64; 3]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockProjectionAction {
    None,
    PredictionsResolved(Vec<PredictionResolution>),
    BlockEvent {
        position: BlockPos,
        current_block_raw_id: Option<i32>,
        action: u8,
        parameter: u8,
    },
}

impl BlockClientProjection {
    pub fn new(capacity: usize) -> Result<Self, BlockProjectionError> {
        if capacity == 0 {
            return Err(BlockProjectionError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            blocks: BTreeMap::new(),
            block_kinds: BTreeMap::new(),
            predictions: PredictionTable::new(capacity),
            destruction: BTreeMap::new(),
            block_entities: BTreeMap::new(),
            last_teleport_sequence: -1,
        })
    }

    pub fn install_block(
        &mut self,
        position: BlockPos,
        state: i32,
        block_raw_id: i32,
    ) -> Result<(), BlockProjectionError> {
        ensure_insert_capacity(&self.blocks, position, self.capacity)?;
        ensure_insert_capacity(&self.block_kinds, position, self.capacity)?;
        self.blocks.insert(position, state);
        self.block_kinds.insert(position, block_raw_id);
        Ok(())
    }

    pub fn install_block_entity(
        &mut self,
        position: BlockPos,
        type_raw_id: i32,
    ) -> Result<(), BlockProjectionError> {
        ensure_insert_capacity(&self.block_entities, position, self.capacity)?;
        self.block_entities.insert(
            position,
            ProjectedBlockEntity {
                type_raw_id,
                update_tag: None,
            },
        );
        Ok(())
    }

    pub fn retain_prediction(
        &mut self,
        position: BlockPos,
        sequence: i32,
        predicted_state: i32,
        captured_player_position: [f64; 3],
    ) -> Result<(), BlockProjectionError> {
        let authoritative_state = self
            .blocks
            .get(&position)
            .copied()
            .ok_or(BlockProjectionError::MissingBlock(position))?;
        if let Some(retained) = self.predictions.get_mut(position) {
            retained.sequence = sequence;
        } else {
            self.predictions.insert(
                position,
                RetainedPrediction {
                    sequence,
                    authoritative_state: Some(authoritative_state),
                    captured_player_position,
                },
            )?;
        }
        self.blocks.insert(position, predicted_state);
        Ok(())
    }

    pub const fn record_teleport(&mut self, current_prediction_sequence: i32) {
        self.last_teleport_sequence = current_prediction_sequence;
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
        game_time: i64,
    ) -> Result<BlockProjectionAction, BlockProjectionError> {
        match packet {
            PlayClientboundPacket::BlockChangedAck(packet) => {
                self.resolve_predictions(packet.sequence)
            }
            PlayClientboundPacket::BlockDestruction(packet) => {
                self.apply_destruction(*packet, game_time)?;
                Ok(BlockProjectionAction::None)
            }
            PlayClientboundPacket::BlockEntityData(packet) => {
                self.apply_block_entity(packet);
                Ok(BlockProjectionAction::None)
            }
            PlayClientboundPacket::BlockEvent(packet) => Ok(self.apply_block_event(*packet)),
            PlayClientboundPacket::BlockUpdate(packet) => {
                self.apply_verified_state(packet.position, Some(packet.state))?;
                Ok(BlockProjectionAction::None)
            }
            PlayClientboundPacket::SectionBlocksUpdate(packet) => {
                self.apply_section(packet)?;
                Ok(BlockProjectionAction::None)
            }
            _ => Err(BlockProjectionError::WrongPacketFamily),
        }
    }

    pub fn expire_destruction(&mut self, game_time: i64) {
        if game_time % 20 != 0 {
            return;
        }
        self.destruction
            .retain(|_, entry| game_time.saturating_sub(entry.updated_game_time) <= 400);
    }

    #[must_use]
    pub fn block_state(&self, position: BlockPos) -> Option<i32> {
        self.blocks.get(&position).copied()
    }

    #[must_use]
    pub fn prediction_count(&self) -> usize {
        self.predictions.len()
    }

    #[must_use]
    pub fn destruction(&self, breaker_entity_id: i32) -> Option<DestructionProgress> {
        self.destruction.get(&breaker_entity_id).copied()
    }

    #[must_use]
    pub fn block_entity(&self, position: BlockPos) -> Option<&ProjectedBlockEntity> {
        self.block_entities.get(&position)
    }

    fn apply_verified_state(
        &mut self,
        position: BlockPos,
        state: Option<i32>,
    ) -> Result<(), BlockProjectionError> {
        if let Some(retained) = self.predictions.get_mut(position) {
            retained.authoritative_state = state;
            return Ok(());
        }
        let state = state.ok_or(BlockProjectionError::NullBlockState(position))?;
        ensure_insert_capacity(&self.blocks, position, self.capacity)?;
        self.blocks.insert(position, state);
        Ok(())
    }

    fn apply_section(&mut self, packet: &SectionBlocksUpdate) -> Result<(), BlockProjectionError> {
        for change in &packet.changes {
            let relative = change.relative_position;
            let x = i32::from((relative >> 8) & 15);
            let z = i32::from((relative >> 4) & 15);
            let y = i32::from(relative & 15);
            let position = BlockPos::new(
                packet.section.x * 16 + x,
                packet.section.y * 16 + y,
                packet.section.z * 16 + z,
            );
            self.apply_verified_state(position, change.state)?;
        }
        Ok(())
    }

    fn resolve_predictions(
        &mut self,
        ack: i32,
    ) -> Result<BlockProjectionAction, BlockProjectionError> {
        let positions = self.predictions.removal_order(ack);
        let mut resolved = Vec::new();
        for position in positions {
            let retained = self
                .predictions
                .remove(position)
                .expect("selected prediction remains present");
            let authoritative_state = retained
                .authoritative_state
                .ok_or(BlockProjectionError::NullBlockState(position))?;
            if self.blocks.get(&position).copied() == Some(authoritative_state) {
                continue;
            }
            self.blocks.insert(position, authoritative_state);
            resolved.push(PredictionResolution {
                position,
                state: authoritative_state,
                captured_player_position: (self.last_teleport_sequence < ack)
                    .then_some(retained.captured_player_position),
            });
        }
        Ok(BlockProjectionAction::PredictionsResolved(resolved))
    }

    fn apply_destruction(
        &mut self,
        packet: BlockDestruction,
        game_time: i64,
    ) -> Result<(), BlockProjectionError> {
        if packet.progress <= 9 {
            ensure_insert_capacity(&self.destruction, packet.breaker_entity_id, self.capacity)?;
            self.destruction.insert(
                packet.breaker_entity_id,
                DestructionProgress {
                    position: packet.position,
                    progress: packet.progress,
                    updated_game_time: game_time,
                },
            );
        } else {
            self.destruction.remove(&packet.breaker_entity_id);
        }
        Ok(())
    }

    fn apply_block_entity(&mut self, packet: &BlockEntityData) {
        let Some(entity) = self.block_entities.get_mut(&packet.position) else {
            return;
        };
        if entity.type_raw_id == packet.type_raw_id {
            entity.update_tag = Some(packet.update_tag.clone());
        }
    }

    fn apply_block_event(&self, packet: BlockEvent) -> BlockProjectionAction {
        BlockProjectionAction::BlockEvent {
            position: packet.position,
            current_block_raw_id: self.block_kinds.get(&packet.position).copied(),
            action: packet.action,
            parameter: packet.parameter,
        }
    }
}

fn ensure_insert_capacity<K: Ord + Copy, V>(
    values: &BTreeMap<K, V>,
    key: K,
    capacity: usize,
) -> Result<(), BlockProjectionError> {
    if values.len() == capacity && !values.contains_key(&key) {
        Err(BlockProjectionError::Full { capacity })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BlockProjectionError {
    #[error("block projection capacity cannot be zero")]
    ZeroCapacity,
    #[error("block projection reached its {capacity}-entry bound")]
    Full { capacity: usize },
    #[error("packet is outside the clientbound block family")]
    WrongPacketFamily,
    #[error("cannot retain a prediction for absent block {0:?}")]
    MissingBlock(BlockPos),
    #[error("nullable section state reached a client state write at {0:?}")]
    NullBlockState(BlockPos),
}
