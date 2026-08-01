use std::collections::{BTreeMap, VecDeque};

use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_persistence::snapshot::SnapshotRecord;
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::composite::model::{
    CompositeCommand, CompositeCommitReceipt, CompositeEvent, CompositeOwner, CompositeProjection,
    CompositeStage,
};
use crate::continuity::identity::ContinuityGeneration;
use crate::continuity::migration::{
    ContinuityMigrationError, canonical_record_hash, normalize_records,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositeRuntimeConfig {
    pub command_capacity: usize,
    pub event_capacity: usize,
    pub projection_capacity: usize,
    pub continuity_record_capacity: usize,
    pub maximum_future_ticks: u64,
    pub maximum_payload_bytes: usize,
}

impl CompositeRuntimeConfig {
    pub const fn testing() -> Self {
        Self {
            command_capacity: 32,
            event_capacity: 32,
            projection_capacity: 32,
            continuity_record_capacity: 32,
            maximum_future_ticks: 4,
            maximum_payload_bytes: 1024,
        }
    }

    fn validate(self) -> Result<(), CompositeRuntimeError> {
        for (kind, capacity) in [
            (CompositeCapacity::Commands, self.command_capacity),
            (CompositeCapacity::Events, self.event_capacity),
            (CompositeCapacity::Projections, self.projection_capacity),
            (
                CompositeCapacity::ContinuityRecords,
                self.continuity_record_capacity,
            ),
            (CompositeCapacity::PayloadBytes, self.maximum_payload_bytes),
        ] {
            if capacity == 0 {
                return Err(CompositeRuntimeError::ZeroCapacity(kind));
            }
        }
        if self.maximum_future_ticks == 0 {
            return Err(CompositeRuntimeError::ZeroFutureHorizon);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CompositeRegionRuntime {
    key: SimulationRegionKey,
    generation: ActivationGeneration,
    config: CompositeRuntimeConfig,
    committed_tick: GameTick,
    active: Option<ActiveTick>,
    commands: BTreeMap<(GameTick, CompositeOwner, u64), CompositeCommand>,
    events: VecDeque<CompositeEvent>,
    pending_projections: Vec<CompositeProjection>,
    committed_projections: VecDeque<CompositeProjection>,
    continuity_records: Option<Vec<SnapshotRecord>>,
    next_event_sequence: u64,
}

#[derive(Debug, Clone, Copy)]
struct ActiveTick {
    tick: GameTick,
    next_stage: usize,
    current_stage: Option<CompositeStage>,
}

impl CompositeRegionRuntime {
    pub fn new(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        committed_tick: GameTick,
        config: CompositeRuntimeConfig,
    ) -> Result<Self, CompositeRuntimeError> {
        config.validate()?;
        Ok(Self {
            key,
            generation,
            config,
            committed_tick,
            active: None,
            commands: BTreeMap::new(),
            events: VecDeque::new(),
            pending_projections: Vec::new(),
            committed_projections: VecDeque::new(),
            continuity_records: None,
            next_event_sequence: 1,
        })
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub const fn generation(&self) -> ActivationGeneration {
        self.generation
    }

    pub const fn committed_tick(&self) -> GameTick {
        self.committed_tick
    }

    pub fn active_tick(&self) -> Option<GameTick> {
        self.active.map(|active| active.tick)
    }

    pub fn current_stage(&self) -> Option<CompositeStage> {
        self.active.and_then(|active| active.current_stage)
    }

    pub fn projection_remaining(&self) -> usize {
        self.config.projection_capacity.saturating_sub(
            self.pending_projections
                .len()
                .saturating_add(self.committed_projections.len()),
        )
    }

    pub fn admit_command(
        &mut self,
        command: CompositeCommand,
    ) -> Result<(), CompositeRuntimeError> {
        if command.sequence() == 0 {
            return Err(CompositeRuntimeError::ZeroSequence);
        }
        if command.payload().len() > self.config.maximum_payload_bytes {
            return Err(CompositeRuntimeError::PayloadTooLarge {
                actual: command.payload().len(),
                maximum: self.config.maximum_payload_bytes,
            });
        }
        let minimum = self.committed_tick.checked_next()?;
        let maximum = self
            .committed_tick
            .get()
            .checked_add(self.config.maximum_future_ticks)
            .ok_or(CompositeRuntimeError::TickHorizonExhausted)?;
        if command.tick() < minimum {
            return Err(CompositeRuntimeError::StaleCommand(command.tick()));
        }
        if command.tick().get() > maximum {
            return Err(CompositeRuntimeError::CommandBeyondHorizon {
                tick: command.tick(),
                maximum: GameTick::new(maximum),
            });
        }
        if self.commands.len() == self.config.command_capacity {
            return Err(CompositeRuntimeError::Full {
                kind: CompositeCapacity::Commands,
                capacity: self.config.command_capacity,
            });
        }
        let identity = (command.tick(), command.owner(), command.sequence());
        if self.commands.contains_key(&identity) {
            return Err(CompositeRuntimeError::DuplicateCommand {
                tick: command.tick(),
                owner: command.owner(),
                sequence: command.sequence(),
            });
        }
        self.commands.insert(identity, command);
        Ok(())
    }

    pub fn begin_tick(&mut self, tick: GameTick) -> Result<(), CompositeRuntimeError> {
        if self.active.is_some() {
            return Err(CompositeRuntimeError::TickAlreadyActive);
        }
        let expected = self.committed_tick.checked_next()?;
        if tick != expected {
            return Err(CompositeRuntimeError::NonSequentialTick {
                expected,
                actual: tick,
            });
        }
        self.active = Some(ActiveTick {
            tick,
            next_stage: 0,
            current_stage: None,
        });
        Ok(())
    }

    pub fn enter_stage(&mut self, stage: CompositeStage) -> Result<(), CompositeRuntimeError> {
        let active = self
            .active
            .as_mut()
            .ok_or(CompositeRuntimeError::NoActiveTick)?;
        if let Some(current) = active.current_stage {
            return Err(CompositeRuntimeError::StageAlreadyActive(current));
        }
        let expected = CompositeStage::ALL
            .get(active.next_stage)
            .copied()
            .ok_or(CompositeRuntimeError::TickStagesComplete)?;
        if stage != expected {
            return Err(CompositeRuntimeError::WrongStage {
                expected,
                actual: stage,
            });
        }
        active.current_stage = Some(stage);
        Ok(())
    }

    pub fn commands(
        &self,
        owner: CompositeOwner,
    ) -> Result<Vec<&CompositeCommand>, CompositeRuntimeError> {
        let active = self.active.ok_or(CompositeRuntimeError::NoActiveTick)?;
        let stage = active
            .current_stage
            .ok_or(CompositeRuntimeError::NoActiveStage)?;
        if stage != owner_stage(owner) {
            return Err(CompositeRuntimeError::WrongCommandOwner { stage, owner });
        }
        Ok(self
            .commands
            .range((active.tick, owner, 0)..=(active.tick, owner, u64::MAX))
            .map(|(_, command)| command)
            .collect())
    }

    pub fn queue_projection(
        &mut self,
        projection: CompositeProjection,
    ) -> Result<(), CompositeRuntimeError> {
        let active = self.active.ok_or(CompositeRuntimeError::NoActiveTick)?;
        let stage = active
            .current_stage
            .ok_or(CompositeRuntimeError::NoActiveStage)?;
        if stage >= CompositeStage::Commit {
            return Err(CompositeRuntimeError::ProjectionAfterCommit);
        }
        if projection.sequence() == 0 {
            return Err(CompositeRuntimeError::ZeroSequence);
        }
        if projection.payload().len() > self.config.maximum_payload_bytes {
            return Err(CompositeRuntimeError::PayloadTooLarge {
                actual: projection.payload().len(),
                maximum: self.config.maximum_payload_bytes,
            });
        }
        let used = self
            .pending_projections
            .len()
            .saturating_add(self.committed_projections.len());
        if used == self.config.projection_capacity {
            return Err(CompositeRuntimeError::Full {
                kind: CompositeCapacity::Projections,
                capacity: self.config.projection_capacity,
            });
        }
        self.pending_projections.push(projection);
        Ok(())
    }

    pub fn prepare_continuity(
        &mut self,
        records: Vec<SnapshotRecord>,
    ) -> Result<[u8; 32], CompositeRuntimeError> {
        self.require_stage(CompositeStage::Continuity)?;
        if self.continuity_records.is_some() {
            return Err(CompositeRuntimeError::ContinuityAlreadyPrepared);
        }
        if records.len() > self.config.continuity_record_capacity {
            return Err(CompositeRuntimeError::Full {
                kind: CompositeCapacity::ContinuityRecords,
                capacity: self.config.continuity_record_capacity,
            });
        }
        let normalized = normalize_records(&records)?;
        if normalized.generation() == Some(ContinuityGeneration::Legacy) {
            return Err(CompositeRuntimeError::LegacyContinuityWrite);
        }
        let hash = canonical_record_hash(normalized.records());
        self.continuity_records = Some(normalized.into_records());
        Ok(hash)
    }

    pub fn complete_stage(
        &mut self,
    ) -> Result<Option<CompositeCommitReceipt>, CompositeRuntimeError> {
        let active = self.active.ok_or(CompositeRuntimeError::NoActiveTick)?;
        let stage = active
            .current_stage
            .ok_or(CompositeRuntimeError::NoActiveStage)?;
        if stage == CompositeStage::Continuity && self.continuity_records.is_none() {
            return Err(CompositeRuntimeError::ContinuityNotPrepared);
        }
        if self.events.len() == self.config.event_capacity {
            return Err(CompositeRuntimeError::Full {
                kind: CompositeCapacity::Events,
                capacity: self.config.event_capacity,
            });
        }
        let next_event_sequence = self
            .next_event_sequence
            .checked_add(1)
            .ok_or(CompositeRuntimeError::SequenceExhausted)?;
        let receipt = if stage == CompositeStage::Commit {
            Some(self.commit(active.tick)?)
        } else {
            None
        };
        self.events.push_back(CompositeEvent {
            sequence: self.next_event_sequence,
            tick: active.tick,
            stage,
            replay_identity: receipt.map(|value| value.replay_identity),
        });
        self.next_event_sequence = next_event_sequence;
        let active = self
            .active
            .as_mut()
            .expect("active tick remains installed during stage completion");
        active.current_stage = None;
        active.next_stage += 1;
        if stage == CompositeStage::Projection {
            self.active = None;
        }
        Ok(receipt)
    }

    pub fn drain_projections(
        &mut self,
        maximum: usize,
    ) -> Result<Vec<CompositeProjection>, CompositeRuntimeError> {
        self.require_stage(CompositeStage::Projection)?;
        let count = maximum.min(self.committed_projections.len());
        Ok(self.committed_projections.drain(..count).collect())
    }

    pub fn take_events(&mut self, maximum: usize) -> Vec<CompositeEvent> {
        let count = maximum.min(self.events.len());
        self.events.drain(..count).collect()
    }

    fn require_stage(&self, expected: CompositeStage) -> Result<(), CompositeRuntimeError> {
        let active = self.active.ok_or(CompositeRuntimeError::NoActiveTick)?;
        let actual = active
            .current_stage
            .ok_or(CompositeRuntimeError::NoActiveStage)?;
        if actual == expected {
            Ok(())
        } else {
            Err(CompositeRuntimeError::WrongStage { expected, actual })
        }
    }

    fn commit(&mut self, tick: GameTick) -> Result<CompositeCommitReceipt, CompositeRuntimeError> {
        let records = self
            .continuity_records
            .take()
            .ok_or(CompositeRuntimeError::ContinuityNotPrepared)?;
        let continuity_hash = canonical_record_hash(&records);
        self.pending_projections.sort_by(|left, right| {
            (left.owner(), left.sequence(), left.kind()).cmp(&(
                right.owner(),
                right.sequence(),
                right.kind(),
            ))
        });
        let replay_identity = self.replay_identity(tick, continuity_hash);
        let projection_count = self.pending_projections.len();
        self.committed_projections
            .extend(self.pending_projections.drain(..));
        self.commands
            .retain(|(command_tick, _, _), _| *command_tick > tick);
        self.committed_tick = tick;
        Ok(CompositeCommitReceipt {
            tick,
            replay_identity,
            continuity_hash,
            projection_count,
        })
    }

    fn replay_identity(&self, tick: GameTick, continuity_hash: [u8; 32]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.key.world().to_be_bytes());
        hash_bytes(
            &mut hasher,
            self.key.dimension().resource().to_string().as_bytes(),
        );
        hasher.update(&self.key.coordinate().x().to_be_bytes());
        hasher.update(&self.key.coordinate().z().to_be_bytes());
        hasher.update(&self.key.mapping_version().get().to_be_bytes());
        hasher.update(&self.generation.get().to_be_bytes());
        hasher.update(&tick.get().to_be_bytes());
        for ((command_tick, owner, sequence), command) in self.commands.range(
            (tick, CompositeOwner::Ingress, 0)..=(tick, CompositeOwner::WorldService, u64::MAX),
        ) {
            hasher.update(&command_tick.get().to_be_bytes());
            hasher.update(&[owner.stable_tag()]);
            hasher.update(&sequence.to_be_bytes());
            hash_bytes(&mut hasher, command.kind().to_string().as_bytes());
            hash_bytes(&mut hasher, command.payload());
        }
        for stage in CompositeStage::ALL.into_iter().take(8) {
            hasher.update(&[stage.stable_tag()]);
        }
        hasher.update(&continuity_hash);
        for projection in &self.pending_projections {
            hasher.update(&[projection.owner().stable_tag()]);
            hasher.update(&projection.sequence().to_be_bytes());
            hash_bytes(&mut hasher, projection.kind().to_string().as_bytes());
            hash_bytes(&mut hasher, projection.payload());
        }
        *hasher.finalize().as_bytes()
    }
}

const fn owner_stage(owner: CompositeOwner) -> CompositeStage {
    match owner {
        CompositeOwner::Ingress => CompositeStage::Ingress,
        CompositeOwner::PlayerService => CompositeStage::PlayerService,
        CompositeOwner::Simulation => CompositeStage::Simulation,
        CompositeOwner::EntityService => CompositeStage::EntityService,
        CompositeOwner::WorldService => CompositeStage::WorldService,
    }
}

fn hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeCapacity {
    Commands,
    Events,
    Projections,
    ContinuityRecords,
    PayloadBytes,
}

#[derive(Debug, Error)]
pub enum CompositeRuntimeError {
    #[error("composite runtime {0:?} capacity cannot be zero")]
    ZeroCapacity(CompositeCapacity),
    #[error("composite runtime future command horizon cannot be zero")]
    ZeroFutureHorizon,
    #[error("composite runtime {kind:?} capacity {capacity} is full")]
    Full {
        kind: CompositeCapacity,
        capacity: usize,
    },
    #[error("composite command or projection sequence cannot be zero")]
    ZeroSequence,
    #[error("composite payload has {actual} bytes, exceeding {maximum}")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("composite command for tick {0:?} is stale")]
    StaleCommand(GameTick),
    #[error("composite command tick {tick:?} exceeds horizon {maximum:?}")]
    CommandBeyondHorizon { tick: GameTick, maximum: GameTick },
    #[error("composite command tick horizon is exhausted")]
    TickHorizonExhausted,
    #[error("composite command {owner:?}/{sequence} is duplicated at {tick:?}")]
    DuplicateCommand {
        tick: GameTick,
        owner: CompositeOwner,
        sequence: u64,
    },
    #[error("a composite tick is already active")]
    TickAlreadyActive,
    #[error("composite tick must be {expected:?}, not {actual:?}")]
    NonSequentialTick {
        expected: GameTick,
        actual: GameTick,
    },
    #[error("no composite tick is active")]
    NoActiveTick,
    #[error("composite stage {0:?} is already active")]
    StageAlreadyActive(CompositeStage),
    #[error("composite tick completed all stages")]
    TickStagesComplete,
    #[error("composite stage must be {expected:?}, not {actual:?}")]
    WrongStage {
        expected: CompositeStage,
        actual: CompositeStage,
    },
    #[error("no composite stage is active")]
    NoActiveStage,
    #[error("composite stage {stage:?} cannot read {owner:?} commands")]
    WrongCommandOwner {
        stage: CompositeStage,
        owner: CompositeOwner,
    },
    #[error("composite projection cannot be created after commit")]
    ProjectionAfterCommit,
    #[error("composite continuity was already prepared")]
    ContinuityAlreadyPrepared,
    #[error("composite continuity must be prepared before the stage completes")]
    ContinuityNotPrepared,
    #[error("legacy continuity identities cannot enter a new composite commit")]
    LegacyContinuityWrite,
    #[error("composite event sequence is exhausted")]
    SequenceExhausted,
    #[error(transparent)]
    Tick(#[from] ferrite_simulation::tick::TickError),
    #[error(transparent)]
    Migration(#[from] ContinuityMigrationError),
}
