use ferrite_foundation::coordinate::ChunkPos;
use ferrite_world::projection::ChunkSnapshot;
use thiserror::Error;

use crate::chunk::interest::{ClientInterest, InterestError};

const INITIAL_DESIRED_CHUNKS_PER_TICK: f32 = 9.0;
const INITIAL_MAXIMUM_UNACKNOWLEDGED: u8 = 1;
const ACKNOWLEDGED_MAXIMUM_UNACKNOWLEDGED: u8 = 10;
const MINIMUM_DESIRED_CHUNKS_PER_TICK: f32 = 0.01;
const MAXIMUM_DESIRED_CHUNKS_PER_TICK: f32 = 64.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkStreamEvent {
    SetCenter(ChunkPos),
    SetViewDistance(u16),
    SetSimulationDistance(u16),
    Forget(ChunkPos),
    BatchStart,
    Chunk(ChunkSnapshot),
    BatchFinish { chunks: usize },
}

#[derive(Debug, Clone)]
pub struct ChunkStream {
    interest: ClientInterest,
    maximum_chunks_per_batch: usize,
    desired_chunks_per_tick: f32,
    quota: f32,
    unacknowledged_batches: u8,
    maximum_unacknowledged_batches: u8,
}

impl ChunkStream {
    pub fn new(
        interest: ClientInterest,
        maximum_chunks_per_batch: usize,
    ) -> Result<Self, ChunkStreamError> {
        if maximum_chunks_per_batch == 0 {
            return Err(ChunkStreamError::ZeroBatchCapacity);
        }
        Ok(Self {
            interest,
            maximum_chunks_per_batch,
            desired_chunks_per_tick: INITIAL_DESIRED_CHUNKS_PER_TICK,
            quota: 0.0,
            unacknowledged_batches: 0,
            maximum_unacknowledged_batches: INITIAL_MAXIMUM_UNACKNOWLEDGED,
        })
    }

    #[must_use]
    pub fn initial_events(&self) -> [ChunkStreamEvent; 3] {
        [
            ChunkStreamEvent::SetCenter(self.interest.center()),
            ChunkStreamEvent::SetViewDistance(self.interest.view_distance()),
            ChunkStreamEvent::SetSimulationDistance(self.interest.simulation_distance()),
        ]
    }

    pub fn recenter(
        &mut self,
        center: ChunkPos,
        alive: bool,
    ) -> Result<Vec<ChunkStreamEvent>, ChunkStreamError> {
        let delta = self.interest.recenter(center)?;
        let mut events = Vec::with_capacity(delta.forgotten.len() + 1);
        if delta.center_changed {
            events.push(ChunkStreamEvent::SetCenter(center));
        }
        if alive {
            events.extend(delta.forgotten.into_iter().map(ChunkStreamEvent::Forget));
        }
        for position in delta.entered {
            self.interest.mark_ready(position)?;
        }
        if delta.center_changed {
            self.interest.requeue(center)?;
        }
        Ok(events)
    }

    pub fn restart(&mut self, center: ChunkPos) -> Result<[ChunkStreamEvent; 3], ChunkStreamError> {
        self.interest.restart(center)?;
        self.quota = 0.0;
        self.unacknowledged_batches = 0;
        self.maximum_unacknowledged_batches = INITIAL_MAXIMUM_UNACKNOWLEDGED;
        Ok(self.initial_events())
    }

    pub fn mark_ready(&mut self, position: ChunkPos) -> Result<bool, ChunkStreamError> {
        Ok(self.interest.mark_ready(position)?)
    }

    pub fn next_batch(
        &mut self,
        mut snapshot: impl FnMut(ChunkPos) -> Option<ChunkSnapshot>,
    ) -> Result<Vec<ChunkStreamEvent>, ChunkStreamError> {
        if self.unacknowledged_batches >= self.maximum_unacknowledged_batches {
            return Ok(Vec::new());
        }
        self.quota =
            (self.quota + self.desired_chunks_per_tick).min(self.desired_chunks_per_tick.max(1.0));
        let budget = (self.quota.floor() as usize).min(self.maximum_chunks_per_batch);
        if budget == 0 {
            return Ok(Vec::new());
        }
        let mut chunks = Vec::with_capacity(budget);
        for position in self.interest.pending_by_distance() {
            let Some(snapshot) = snapshot(position) else {
                continue;
            };
            chunks.push(snapshot);
            if chunks.len() == budget {
                break;
            }
        }
        if chunks.is_empty() {
            return Ok(Vec::new());
        }
        for chunk in &chunks {
            self.interest
                .mark_sent(chunk.position(), chunk.revision())?;
        }
        self.quota -= chunks.len() as f32;
        self.unacknowledged_batches += 1;
        let count = chunks.len();
        let mut events = Vec::with_capacity(count + 2);
        events.push(ChunkStreamEvent::BatchStart);
        events.extend(chunks.into_iter().map(ChunkStreamEvent::Chunk));
        events.push(ChunkStreamEvent::BatchFinish { chunks: count });
        Ok(events)
    }

    pub fn acknowledge_batch(&mut self, desired_chunks_per_tick: f32) {
        self.unacknowledged_batches = self.unacknowledged_batches.saturating_sub(1);
        self.desired_chunks_per_tick = if desired_chunks_per_tick.is_nan() {
            MINIMUM_DESIRED_CHUNKS_PER_TICK
        } else {
            desired_chunks_per_tick.clamp(
                MINIMUM_DESIRED_CHUNKS_PER_TICK,
                MAXIMUM_DESIRED_CHUNKS_PER_TICK,
            )
        };
        if self.unacknowledged_batches == 0 {
            self.quota = (self.quota + 1.0).min(self.desired_chunks_per_tick.max(1.0));
        }
        self.maximum_unacknowledged_batches = ACKNOWLEDGED_MAXIMUM_UNACKNOWLEDGED;
    }

    #[must_use]
    pub const fn interest(&self) -> &ClientInterest {
        &self.interest
    }

    #[must_use]
    pub const fn desired_chunks_per_tick(&self) -> f32 {
        self.desired_chunks_per_tick
    }

    #[must_use]
    pub const fn unacknowledged_batches(&self) -> u8 {
        self.unacknowledged_batches
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChunkStreamError {
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error("chunk stream batch capacity cannot be zero")]
    ZeroBatchCapacity,
}
