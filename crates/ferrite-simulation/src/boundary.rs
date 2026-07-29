//! Bounded cross-Region boundary batches and deterministic admission.

use crate::tick::{GameTick, TickPhase};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const MAX_BOUNDARY_EVENT_BYTES: usize = 1024 * 1024;
pub const MAX_BOUNDARY_BATCH_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryEvent {
    order: u64,
    kind: ResourceId,
    payload: Vec<u8>,
}

impl BoundaryEvent {
    pub fn new(order: u64, kind: ResourceId, payload: Vec<u8>) -> Result<Self, BoundaryError> {
        if payload.len() > MAX_BOUNDARY_EVENT_BYTES {
            return Err(BoundaryError::EventTooLarge {
                actual: payload.len(),
                maximum: MAX_BOUNDARY_EVENT_BYTES,
            });
        }
        Ok(Self {
            order,
            kind,
            payload,
        })
    }

    pub const fn order(&self) -> u64 {
        self.order
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryBatch {
    tick: GameTick,
    phase: TickPhase,
    source: SimulationRegionKey,
    target: SimulationRegionKey,
    source_generation: ActivationGeneration,
    source_sequence: u64,
    events: Box<[BoundaryEvent]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryBatchHeader {
    pub tick: GameTick,
    pub phase: TickPhase,
    pub source: SimulationRegionKey,
    pub target: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub source_sequence: u64,
}

impl BoundaryBatch {
    pub fn new(
        header: BoundaryBatchHeader,
        mut events: Vec<BoundaryEvent>,
        maximum_events: usize,
    ) -> Result<Self, BoundaryError> {
        if header.source == header.target {
            return Err(BoundaryError::SelfTarget);
        }
        if events.len() > maximum_events {
            return Err(BoundaryError::TooManyEvents {
                actual: events.len(),
                maximum: maximum_events,
            });
        }
        let total_bytes = events.iter().try_fold(0_usize, |total, event| {
            total
                .checked_add(event.payload.len())
                .ok_or(BoundaryError::BatchTooLarge {
                    actual: usize::MAX,
                    maximum: MAX_BOUNDARY_BATCH_BYTES,
                })
        })?;
        if total_bytes > MAX_BOUNDARY_BATCH_BYTES {
            return Err(BoundaryError::BatchTooLarge {
                actual: total_bytes,
                maximum: MAX_BOUNDARY_BATCH_BYTES,
            });
        }
        events.sort_by_key(BoundaryEvent::order);
        if events.windows(2).any(|pair| pair[0].order == pair[1].order) {
            return Err(BoundaryError::DuplicateEventOrder);
        }
        Ok(Self {
            tick: header.tick,
            phase: header.phase,
            source: header.source,
            target: header.target,
            source_generation: header.source_generation,
            source_sequence: header.source_sequence,
            events: events.into_boxed_slice(),
        })
    }

    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub const fn phase(&self) -> TickPhase {
        self.phase
    }

    pub const fn source(&self) -> &SimulationRegionKey {
        &self.source
    }

    pub const fn target(&self) -> &SimulationRegionKey {
        &self.target
    }

    pub const fn source_generation(&self) -> ActivationGeneration {
        self.source_generation
    }

    pub const fn source_sequence(&self) -> u64 {
        self.source_sequence
    }

    pub fn events(&self) -> &[BoundaryEvent] {
        &self.events
    }

    fn order_key(&self) -> BoundaryOrderKey {
        BoundaryOrderKey {
            tick: self.tick,
            phase: self.phase,
            source: self.source.clone(),
            source_sequence: self.source_sequence,
        }
    }
}

#[derive(Debug)]
pub struct BoundaryInbox {
    target: SimulationRegionKey,
    capacity: usize,
    pending: BTreeMap<BoundaryOrderKey, BoundaryBatch>,
    admitted: BTreeSet<BoundaryOrderKey>,
}

impl BoundaryInbox {
    pub fn new(target: SimulationRegionKey, capacity: usize) -> Result<Self, BoundaryError> {
        if capacity == 0 {
            return Err(BoundaryError::ZeroCapacity);
        }
        Ok(Self {
            target,
            capacity,
            pending: BTreeMap::new(),
            admitted: BTreeSet::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn admit(
        &mut self,
        batch: BoundaryBatch,
        expected_source_generation: ActivationGeneration,
        committed_tick: GameTick,
    ) -> Result<(), BoundaryError> {
        if batch.target != self.target {
            return Err(BoundaryError::WrongTarget);
        }
        if batch.source_generation != expected_source_generation {
            return Err(BoundaryError::StaleSourceGeneration {
                expected: expected_source_generation,
                actual: batch.source_generation,
            });
        }
        if batch.tick <= committed_tick {
            return Err(BoundaryError::AlreadyCommitted {
                batch: batch.tick,
                committed: committed_tick,
            });
        }
        let key = batch.order_key();
        if self.admitted.contains(&key) {
            return Err(BoundaryError::DuplicateBatch);
        }
        if self.pending.len() == self.capacity || self.admitted.len() == self.capacity {
            return Err(BoundaryError::Full {
                capacity: self.capacity,
            });
        }
        self.admitted.insert(key.clone());
        self.pending.insert(key, batch);
        Ok(())
    }

    pub fn drain(&mut self, tick: GameTick, phase: TickPhase) -> Vec<BoundaryBatch> {
        let keys = self
            .pending
            .keys()
            .filter(|key| key.tick == tick && key.phase == phase)
            .cloned()
            .collect::<Vec<_>>();
        keys.into_iter()
            .filter_map(|key| self.pending.remove(&key))
            .collect()
    }

    pub fn prune_committed(&mut self, committed_tick: GameTick) {
        self.admitted.retain(|key| key.tick > committed_tick);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BoundaryOrderKey {
    tick: GameTick,
    phase: TickPhase,
    source: SimulationRegionKey,
    source_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BoundaryError {
    #[error("boundary inbox capacity cannot be zero")]
    ZeroCapacity,
    #[error("boundary event has {actual} bytes, exceeding the {maximum}-byte limit")]
    EventTooLarge { actual: usize, maximum: usize },
    #[error("boundary batch payload has {actual} bytes, exceeding the {maximum}-byte limit")]
    BatchTooLarge { actual: usize, maximum: usize },
    #[error("boundary batch has {actual} events, exceeding the {maximum}-event limit")]
    TooManyEvents { actual: usize, maximum: usize },
    #[error("boundary event order is duplicated")]
    DuplicateEventOrder,
    #[error("boundary batch cannot target its source Region")]
    SelfTarget,
    #[error("boundary batch targets another Region")]
    WrongTarget,
    #[error("boundary source generation {actual:?} does not match {expected:?}")]
    StaleSourceGeneration {
        expected: ActivationGeneration,
        actual: ActivationGeneration,
    },
    #[error("boundary batch tick {batch:?} is not after committed tick {committed:?}")]
    AlreadyCommitted {
        batch: GameTick,
        committed: GameTick,
    },
    #[error("boundary batch order key is already admitted")]
    DuplicateBatch,
    #[error("boundary inbox reached its {capacity}-batch bound")]
    Full { capacity: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};

    fn region(x: i32) -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(x, 0),
            RegionMappingVersion::V1,
        )
    }

    fn batch(source: i32, sequence: u64) -> BoundaryBatch {
        BoundaryBatch::new(
            BoundaryBatchHeader {
                tick: GameTick::new(1),
                phase: TickPhase::ImmediateNeighbors,
                source: region(source),
                target: region(0),
                source_generation: ActivationGeneration::INITIAL,
                source_sequence: sequence,
            },
            vec![
                BoundaryEvent::new(
                    2,
                    ResourceId::new("ferrite", "boundary/test").unwrap(),
                    vec![2],
                )
                .unwrap(),
                BoundaryEvent::new(
                    1,
                    ResourceId::new("ferrite", "boundary/test").unwrap(),
                    vec![1],
                )
                .unwrap(),
            ],
            4,
        )
        .unwrap()
    }

    #[test]
    fn batches_and_events_are_drained_in_canonical_order() {
        let mut inbox = BoundaryInbox::new(region(0), 4).unwrap();
        inbox
            .admit(batch(2, 1), ActivationGeneration::INITIAL, GameTick::ZERO)
            .unwrap();
        inbox
            .admit(batch(1, 2), ActivationGeneration::INITIAL, GameTick::ZERO)
            .unwrap();
        let drained = inbox.drain(GameTick::new(1), TickPhase::ImmediateNeighbors);
        assert_eq!(drained[0].source().coordinate().x(), 1);
        assert_eq!(drained[0].events()[0].order(), 1);
    }

    #[test]
    fn stale_duplicate_and_full_batches_fail_without_replacement() {
        let mut inbox = BoundaryInbox::new(region(0), 1).unwrap();
        let batch = batch(1, 1);
        assert!(
            inbox
                .admit(
                    batch.clone(),
                    ActivationGeneration::new(2).unwrap(),
                    GameTick::ZERO
                )
                .is_err()
        );
        inbox
            .admit(batch.clone(), ActivationGeneration::INITIAL, GameTick::ZERO)
            .unwrap();
        assert_eq!(
            inbox
                .drain(GameTick::new(1), TickPhase::ImmediateNeighbors)
                .len(),
            1
        );
        assert!(
            inbox
                .admit(batch, ActivationGeneration::INITIAL, GameTick::ZERO)
                .is_err()
        );
        assert_eq!(inbox.len(), 0);
    }
}
