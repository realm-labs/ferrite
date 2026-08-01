//! Region-owned Simulation state and atomic reconciliation.

use crate::chunk::projection::JavaTerrainRegistryMap;
use crate::player::block::replication::AuthoritativeBlockUpdate;
use crate::simulation::boundary::{BoundaryMechanic, MechanicBoundaryTransaction};
use crate::simulation::budget::{
    QueueBudgetError, QueuePressure, QueueReservation, SimulationQueueBudget, SimulationQueueKind,
};
use crate::simulation::continuity::{
    AppliedBoundaryReceipt, ScheduledQueueKind, SimulationContinuity,
};
use crate::simulation::projection::{SimulationProjectionBuffer, SimulationProjectionError};
use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_simulation::random::DeterministicRng;
use ferrite_simulation::random_tick::position::RandomPositionStream;
use ferrite_simulation::scheduled_tick::container::ChunkTickContainer;
use ferrite_simulation::scheduled_tick::level::{ScheduleOutcome, ScheduledTickQueue};
use ferrite_simulation::scheduled_tick::record::{ScheduledTick, SubTickCounter, TickPriority};
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::ChunkColumn;
use ferrite_world::id::BlockStateId;
use ferrite_world::region::{RegionVoxelError, RegionVoxelState};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

const ALL_QUEUE_KINDS: [SimulationQueueKind; 8] = [
    SimulationQueueKind::ScheduledBlocks,
    SimulationQueueKind::ScheduledFluids,
    SimulationQueueKind::BoundaryTransactions,
    SimulationQueueKind::ImmediateNeighbors,
    SimulationQueueKind::Fluids,
    SimulationQueueKind::Redstone,
    SimulationQueueKind::Lighting,
    SimulationQueueKind::ProjectionPositions,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationRuntimeConfig {
    pub mapping: RegionMapping,
    pub budget: SimulationQueueBudget,
    pub projection_capacity: usize,
    pub receipt_capacity: usize,
    pub gameplay_random_seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeferredEffectOrder {
    pub tick: GameTick,
    pub source_x: i32,
    pub source_z: i32,
    pub source_generation: ActivationGeneration,
    pub source_sequence: u64,
    pub mutation_order: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeferredMechanicEffect {
    pub order: DeferredEffectOrder,
    pub mechanic: BoundaryMechanic,
    pub position: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryApplyOutcome {
    Applied {
        mutations: usize,
        scheduled_blocks: usize,
        scheduled_fluids: usize,
        deferred_effects: usize,
        projected_positions: usize,
    },
    AlreadyApplied,
}

#[derive(Debug)]
pub struct SimulationRegionRuntime {
    key: SimulationRegionKey,
    generation: ActivationGeneration,
    tick: GameTick,
    game_time: i64,
    mapping: RegionMapping,
    blocks: ScheduledTickQueue<ResourceId>,
    fluids: ScheduledTickQueue<ResourceId>,
    sub_tick_counter: SubTickCounter,
    random_position: RandomPositionStream,
    gameplay_random: DeterministicRng,
    applied_boundaries: BTreeSet<AppliedBoundaryReceipt>,
    receipt_capacity: usize,
    effects: BTreeMap<SimulationQueueKind, VecDeque<DeferredMechanicEffect>>,
    budget: SimulationQueueBudget,
    projection: SimulationProjectionBuffer,
}

impl SimulationRegionRuntime {
    pub fn new(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        tick: GameTick,
        game_time: i64,
        chunks: impl IntoIterator<Item = ChunkPos>,
        config: SimulationRuntimeConfig,
    ) -> Result<Self, SimulationRuntimeError> {
        validate_config(&key, &config)?;
        let mut blocks = ScheduledTickQueue::new();
        let mut fluids = ScheduledTickQueue::new();
        let mut registered = BTreeSet::new();
        for chunk in chunks {
            validate_chunk_owner(&key, config.mapping, chunk)?;
            if !registered.insert(chunk) {
                return Err(SimulationRuntimeError::DuplicateRegisteredChunk(chunk));
            }
            blocks.register_container(chunk, ChunkTickContainer::new());
            fluids.register_container(chunk, ChunkTickContainer::new());
        }
        Ok(Self {
            key,
            generation,
            tick,
            game_time,
            mapping: config.mapping,
            blocks,
            fluids,
            sub_tick_counter: SubTickCounter::default(),
            random_position: RandomPositionStream::new(0),
            gameplay_random: DeterministicRng::from_seed(config.gameplay_random_seed),
            applied_boundaries: BTreeSet::new(),
            receipt_capacity: config.receipt_capacity,
            effects: BTreeMap::new(),
            budget: config.budget,
            projection: SimulationProjectionBuffer::new(config.projection_capacity)?,
        })
    }

    pub fn restore(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        tick: GameTick,
        game_time: i64,
        continuity: SimulationContinuity,
        mut config: SimulationRuntimeConfig,
    ) -> Result<Self, SimulationRuntimeError> {
        validate_config(&key, &config)?;
        if continuity.applied_boundaries.len() > config.receipt_capacity {
            return Err(SimulationRuntimeError::ReceiptCapacity {
                used: continuity.applied_boundaries.len(),
                capacity: config.receipt_capacity,
            });
        }
        let mut blocks = ScheduledTickQueue::new();
        let mut fluids = ScheduledTickQueue::new();
        let mut seen = BTreeSet::new();
        for entry in continuity.scheduled {
            validate_chunk_owner(&key, config.mapping, entry.chunk)?;
            if !seen.insert((entry.kind, entry.chunk)) {
                return Err(SimulationRuntimeError::DuplicateContinuityChunk {
                    kind: entry.kind,
                    chunk: entry.chunk,
                });
            }
            let queue = match entry.kind {
                ScheduledQueueKind::Block => &mut blocks,
                ScheduledQueueKind::Fluid => &mut fluids,
            };
            queue.register_container(entry.chunk, ChunkTickContainer::from_saved(entry.ticks));
            if !queue.unpack_container(entry.chunk, game_time) {
                return Err(SimulationRuntimeError::ContinuityRegistration);
            }
        }
        let block_count = blocks.count();
        let fluid_count = fluids.count();
        reserve_nonzero(
            &mut config.budget,
            [
                (SimulationQueueKind::ScheduledBlocks, block_count),
                (SimulationQueueKind::ScheduledFluids, fluid_count),
            ],
        )?;
        Ok(Self {
            key,
            generation,
            tick,
            game_time,
            mapping: config.mapping,
            blocks,
            fluids,
            sub_tick_counter: SubTickCounter::new(continuity.next_sub_tick),
            random_position: RandomPositionStream::new(continuity.random_position_value),
            gameplay_random: DeterministicRng::from_state(
                continuity.gameplay_random_algorithm,
                continuity.gameplay_random_state,
            ),
            applied_boundaries: continuity.applied_boundaries,
            receipt_capacity: config.receipt_capacity,
            effects: BTreeMap::new(),
            budget: config.budget,
            projection: SimulationProjectionBuffer::new(config.projection_capacity)?,
        })
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub const fn generation(&self) -> ActivationGeneration {
        self.generation
    }

    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub const fn game_time(&self) -> i64 {
        self.game_time
    }

    pub fn queue_pressure(
        &self,
        kind: SimulationQueueKind,
    ) -> Result<QueuePressure, SimulationRuntimeError> {
        Ok(self.budget.pressure(kind)?)
    }

    pub fn capture_continuity(&self) -> Result<SimulationContinuity, SimulationRuntimeError> {
        let effects = self.effects.values().map(VecDeque::len).sum();
        let projection = self.projection.len();
        if effects != 0 || projection != 0 {
            return Err(SimulationRuntimeError::TransientStateAtCommit {
                effects,
                projection,
            });
        }
        Ok(SimulationContinuity::capture(
            &self.blocks,
            &self.fluids,
            self.game_time,
            self.sub_tick_counter,
            self.random_position,
            &self.gameplay_random,
            self.applied_boundaries.clone(),
        ))
    }

    pub fn apply_transaction(
        &mut self,
        voxels: &mut RegionVoxelState,
        transaction: &MechanicBoundaryTransaction,
    ) -> Result<BoundaryApplyOutcome, SimulationRuntimeError> {
        self.validate_transaction(voxels, transaction)?;
        let receipt = transaction.receipt();
        if self.applied_boundaries.contains(&receipt) {
            return Ok(BoundaryApplyOutcome::AlreadyApplied);
        }
        if self.applied_boundaries.len() == self.receipt_capacity {
            return Err(SimulationRuntimeError::ReceiptCapacity {
                used: self.applied_boundaries.len(),
                capacity: self.receipt_capacity,
            });
        }

        let (block_schedules, fluid_schedules) = self.preflight_schedules(transaction)?;
        let (staged_chunks, updates) = preflight_mutations(voxels, transaction)?;
        let projection_positions = self.projection.additional_positions(&updates)?;
        let effect_kind = transaction.mechanic().queue_kind();
        let effect_count = transaction.mutations().len();
        let reservation = reserve_nonzero(
            &mut self.budget,
            [
                (SimulationQueueKind::BoundaryTransactions, 1),
                (SimulationQueueKind::ScheduledBlocks, block_schedules),
                (SimulationQueueKind::ScheduledFluids, fluid_schedules),
                (effect_kind, effect_count),
                (
                    SimulationQueueKind::ProjectionPositions,
                    projection_positions,
                ),
            ],
        )?;
        if let Err(error) = self.projection.enqueue(&updates) {
            self.budget.release(reservation)?;
            return Err(error.into());
        }

        self.commit_schedules(transaction);
        commit_chunks(voxels, staged_chunks);
        self.commit_effects(transaction);
        self.applied_boundaries.insert(receipt);
        self.budget
            .release_usage([(SimulationQueueKind::BoundaryTransactions, 1)])?;
        Ok(BoundaryApplyOutcome::Applied {
            mutations: transaction.mutations().len(),
            scheduled_blocks: block_schedules,
            scheduled_fluids: fluid_schedules,
            deferred_effects: effect_count,
            projected_positions: projection_positions,
        })
    }

    pub fn schedule_local(
        &mut self,
        kind: ScheduledQueueKind,
        type_identity: ResourceId,
        position: BlockPos,
        delay: i32,
        priority: TickPriority,
    ) -> Result<ScheduleOutcome, SimulationRuntimeError> {
        validate_chunk_owner(&self.key, self.mapping, position.chunk())?;
        let queue = self.queue(kind);
        if queue.container(position.chunk()).is_none() {
            return Err(SimulationRuntimeError::UnregisteredScheduledChunk {
                kind,
                chunk: position.chunk(),
            });
        }
        if queue.has_scheduled_tick(position, &type_identity) {
            return Ok(ScheduleOutcome::Duplicate);
        }
        let budget_kind = scheduled_budget_kind(kind);
        let reservation = self.budget.try_reserve([(budget_kind, 1)])?;
        let tick =
            self.sub_tick_counter
                .create(type_identity, position, self.game_time, delay, priority);
        let outcome = self.queue_mut(kind).schedule(tick);
        if outcome != ScheduleOutcome::Queued {
            self.budget.release(reservation)?;
            return Err(SimulationRuntimeError::ScheduleInvariant);
        }
        Ok(outcome)
    }

    pub fn tick_scheduled(
        &mut self,
        kind: ScheduledQueueKind,
        maximum: usize,
        mut in_ticking_range: impl FnMut(ChunkPos) -> bool,
        mut output: impl FnMut(ScheduledTick<ResourceId>),
    ) -> usize {
        let game_time = self.game_time;
        let count =
            self.queue_mut(kind)
                .tick(game_time, maximum, &mut in_ticking_range, |_, tick| {
                    output(tick)
                });
        if count > 0 {
            self.budget
                .release_usage([(scheduled_budget_kind(kind), count)])
                .expect("scheduled queue count and budget usage remain equal");
        }
        count
    }

    pub fn drain_effects(
        &mut self,
        kind: SimulationQueueKind,
        maximum: usize,
    ) -> Result<Vec<DeferredMechanicEffect>, SimulationRuntimeError> {
        if !is_effect_queue(kind) {
            return Err(SimulationRuntimeError::NotEffectQueue(kind));
        }
        let queue = self.effects.entry(kind).or_default();
        let count = maximum.min(queue.len());
        let drained = queue.drain(..count).collect::<Vec<_>>();
        if count > 0 {
            self.budget.release_usage([(kind, count)])?;
        }
        Ok(drained)
    }

    pub fn project_and_clear(
        &mut self,
        registries: &JavaTerrainRegistryMap,
    ) -> Result<Vec<PlayClientboundPacket>, SimulationRuntimeError> {
        let count = self.projection.len();
        let pressure = self
            .budget
            .pressure(SimulationQueueKind::ProjectionPositions)?;
        if pressure.used != count {
            return Err(SimulationRuntimeError::ProjectionBudgetInvariant {
                positions: count,
                reserved: pressure.used,
            });
        }
        let packets = self.projection.project_and_clear(registries)?;
        if count > 0 {
            self.budget
                .release_usage([(SimulationQueueKind::ProjectionPositions, count)])?;
        }
        Ok(packets)
    }

    pub fn next_random_position(&mut self, base: BlockPos, y_mask: i32) -> BlockPos {
        self.random_position.next(base, y_mask)
    }

    pub fn gameplay_random_mut(&mut self) -> &mut DeterministicRng {
        &mut self.gameplay_random
    }

    pub fn prune_receipts(&mut self, predicate: impl FnMut(&AppliedBoundaryReceipt) -> bool) {
        self.applied_boundaries.retain(predicate);
    }

    pub fn advance_commit(
        &mut self,
        tick: GameTick,
        game_time: i64,
    ) -> Result<(), SimulationRuntimeError> {
        if tick <= self.tick {
            return Err(SimulationRuntimeError::NonIncreasingCommit {
                current: self.tick,
                requested: tick,
            });
        }
        self.tick = tick;
        self.game_time = game_time;
        Ok(())
    }

    fn validate_transaction(
        &self,
        voxels: &RegionVoxelState,
        transaction: &MechanicBoundaryTransaction,
    ) -> Result<(), SimulationRuntimeError> {
        if voxels.key() != &self.key {
            return Err(SimulationRuntimeError::WrongVoxelRegion);
        }
        if transaction.target() != &self.key {
            return Err(SimulationRuntimeError::WrongTransactionTarget);
        }
        if transaction.target_generation() != self.generation {
            return Err(SimulationRuntimeError::StaleTargetGeneration {
                expected: self.generation,
                actual: transaction.target_generation(),
            });
        }
        if transaction.tick() != self.tick {
            return Err(SimulationRuntimeError::WrongTransactionTick {
                expected: self.tick,
                actual: transaction.tick(),
            });
        }
        Ok(())
    }

    fn preflight_schedules(
        &self,
        transaction: &MechanicBoundaryTransaction,
    ) -> Result<(usize, usize), SimulationRuntimeError> {
        let mut blocks = 0;
        let mut fluids = 0;
        for schedule in transaction.schedules() {
            let queue = self.queue(schedule.kind);
            if queue.container(schedule.position.chunk()).is_none() {
                return Err(SimulationRuntimeError::UnregisteredScheduledChunk {
                    kind: schedule.kind,
                    chunk: schedule.position.chunk(),
                });
            }
            if queue.has_scheduled_tick(schedule.position, &schedule.type_identity) {
                continue;
            }
            match schedule.kind {
                ScheduledQueueKind::Block => blocks += 1,
                ScheduledQueueKind::Fluid => fluids += 1,
            }
        }
        Ok((blocks, fluids))
    }

    fn commit_schedules(&mut self, transaction: &MechanicBoundaryTransaction) {
        for schedule in transaction.schedules() {
            if self
                .queue(schedule.kind)
                .has_scheduled_tick(schedule.position, &schedule.type_identity)
            {
                continue;
            }
            let tick = self.sub_tick_counter.create(
                schedule.type_identity.clone(),
                schedule.position,
                self.game_time,
                schedule.delay,
                schedule.priority,
            );
            let outcome = self.queue_mut(schedule.kind).schedule(tick);
            assert_eq!(
                outcome,
                ScheduleOutcome::Queued,
                "preflight guarantees scheduled work admission"
            );
        }
    }

    fn commit_effects(&mut self, transaction: &MechanicBoundaryTransaction) {
        let source = transaction.source().coordinate();
        let queue = self
            .effects
            .entry(transaction.mechanic().queue_kind())
            .or_default();
        for mutation in transaction.mutations() {
            queue.push_back(DeferredMechanicEffect {
                order: DeferredEffectOrder {
                    tick: transaction.tick(),
                    source_x: source.x(),
                    source_z: source.z(),
                    source_generation: transaction.source_generation(),
                    source_sequence: transaction.source_sequence(),
                    mutation_order: mutation.order,
                },
                mechanic: transaction.mechanic(),
                position: mutation.position,
            });
        }
        queue.make_contiguous().sort_by_key(|effect| effect.order);
    }

    fn queue(&self, kind: ScheduledQueueKind) -> &ScheduledTickQueue<ResourceId> {
        match kind {
            ScheduledQueueKind::Block => &self.blocks,
            ScheduledQueueKind::Fluid => &self.fluids,
        }
    }

    fn queue_mut(&mut self, kind: ScheduledQueueKind) -> &mut ScheduledTickQueue<ResourceId> {
        match kind {
            ScheduledQueueKind::Block => &mut self.blocks,
            ScheduledQueueKind::Fluid => &mut self.fluids,
        }
    }
}

fn validate_config(
    key: &SimulationRegionKey,
    config: &SimulationRuntimeConfig,
) -> Result<(), SimulationRuntimeError> {
    if key.mapping_version() != config.mapping.version() {
        return Err(SimulationRuntimeError::MappingVersionMismatch);
    }
    if config.receipt_capacity == 0 {
        return Err(SimulationRuntimeError::ZeroReceiptCapacity);
    }
    for kind in ALL_QUEUE_KINDS {
        config.budget.pressure(kind)?;
    }
    Ok(())
}

fn validate_chunk_owner(
    key: &SimulationRegionKey,
    mapping: RegionMapping,
    chunk: ChunkPos,
) -> Result<(), SimulationRuntimeError> {
    let actual = mapping.region_for_chunk(key.world(), key.dimension().clone(), chunk);
    if &actual == key {
        Ok(())
    } else {
        Err(SimulationRuntimeError::WrongChunkOwner { chunk })
    }
}

fn preflight_mutations(
    voxels: &RegionVoxelState,
    transaction: &MechanicBoundaryTransaction,
) -> Result<
    (
        BTreeMap<ChunkPos, ChunkColumn>,
        Vec<AuthoritativeBlockUpdate>,
    ),
    SimulationRuntimeError,
> {
    let mut chunks = BTreeMap::<ChunkPos, ChunkColumn>::new();
    let mut updates = Vec::with_capacity(transaction.mutations().len());
    for mutation in transaction.mutations() {
        let chunk = if let Some(chunk) = chunks.get_mut(&mutation.position.chunk()) {
            chunk
        } else {
            let loaded = voxels
                .view()
                .chunk(mutation.position.chunk())
                .ok_or(RegionVoxelError::ChunkNotLoaded(mutation.position.chunk()))?
                .clone();
            chunks.entry(mutation.position.chunk()).or_insert(loaded)
        };
        let actual = chunk
            .block_state(mutation.position)
            .map_err(RegionVoxelError::from)?;
        if actual != mutation.expected {
            return Err(SimulationRuntimeError::UnexpectedBlockState {
                position: mutation.position,
                expected: mutation.expected,
                actual,
            });
        }
        chunk
            .set_block(mutation.position, mutation.replacement)
            .map_err(RegionVoxelError::from)?;
        updates.push(AuthoritativeBlockUpdate {
            position: mutation.position,
            state: mutation.replacement,
        });
    }
    Ok((chunks, updates))
}

fn commit_chunks(voxels: &mut RegionVoxelState, chunks: BTreeMap<ChunkPos, ChunkColumn>) {
    for (position, chunk) in chunks {
        let removed = voxels.remove_chunk(position);
        assert!(
            removed.is_some(),
            "preflight only stages currently loaded chunks"
        );
        voxels
            .insert_chunk(chunk)
            .expect("preflight preserves owner and chunk layout");
    }
}

fn reserve_nonzero<const N: usize>(
    budget: &mut SimulationQueueBudget,
    requests: [(SimulationQueueKind, usize); N],
) -> Result<QueueReservation, QueueBudgetError> {
    budget.try_reserve(requests.into_iter().filter(|(_, amount)| *amount != 0))
}

const fn scheduled_budget_kind(kind: ScheduledQueueKind) -> SimulationQueueKind {
    match kind {
        ScheduledQueueKind::Block => SimulationQueueKind::ScheduledBlocks,
        ScheduledQueueKind::Fluid => SimulationQueueKind::ScheduledFluids,
    }
}

const fn is_effect_queue(kind: SimulationQueueKind) -> bool {
    matches!(
        kind,
        SimulationQueueKind::ImmediateNeighbors
            | SimulationQueueKind::Fluids
            | SimulationQueueKind::Redstone
            | SimulationQueueKind::Lighting
    )
}

#[derive(Debug, Error)]
pub enum SimulationRuntimeError {
    #[error("Simulation runtime mapping does not match its Region key")]
    MappingVersionMismatch,
    #[error("Simulation applied-boundary receipt capacity cannot be zero")]
    ZeroReceiptCapacity,
    #[error("chunk {0:?} is registered more than once")]
    DuplicateRegisteredChunk(ChunkPos),
    #[error("continuity has duplicate {kind:?} state for chunk {chunk:?}")]
    DuplicateContinuityChunk {
        kind: ScheduledQueueKind,
        chunk: ChunkPos,
    },
    #[error("continuity scheduled container could not be registered")]
    ContinuityRegistration,
    #[error("chunk {chunk:?} does not belong to this Simulation Region")]
    WrongChunkOwner { chunk: ChunkPos },
    #[error("voxel state does not belong to this Simulation runtime")]
    WrongVoxelRegion,
    #[error("boundary transaction targets another Region")]
    WrongTransactionTarget,
    #[error("boundary target generation is {actual:?}, expected {expected:?}")]
    StaleTargetGeneration {
        expected: ActivationGeneration,
        actual: ActivationGeneration,
    },
    #[error("boundary transaction tick is {actual:?}, expected {expected:?}")]
    WrongTransactionTick {
        expected: GameTick,
        actual: GameTick,
    },
    #[error("applied-boundary receipts use {used}/{capacity} entries")]
    ReceiptCapacity { used: usize, capacity: usize },
    #[error("scheduled {kind:?} work targets unregistered chunk {chunk:?}")]
    UnregisteredScheduledChunk {
        kind: ScheduledQueueKind,
        chunk: ChunkPos,
    },
    #[error("boundary mutation expected {expected:?} at {position:?}, found {actual:?}")]
    UnexpectedBlockState {
        position: BlockPos,
        expected: BlockStateId,
        actual: BlockStateId,
    },
    #[error("scheduled work changed between preflight and commit")]
    ScheduleInvariant,
    #[error("{0:?} is not a deferred mechanic-effect queue")]
    NotEffectQueue(SimulationQueueKind),
    #[error("projection contains {positions} positions but has {reserved} reservations")]
    ProjectionBudgetInvariant { positions: usize, reserved: usize },
    #[error("commit tick {requested:?} does not follow {current:?}")]
    NonIncreasingCommit {
        current: GameTick,
        requested: GameTick,
    },
    #[error(
        "commit continuity requires drained transient queues; {effects} effects and {projection} projection positions remain"
    )]
    TransientStateAtCommit { effects: usize, projection: usize },
    #[error(transparent)]
    Budget(#[from] QueueBudgetError),
    #[error(transparent)]
    Projection(#[from] SimulationProjectionError),
    #[error(transparent)]
    Voxel(#[from] RegionVoxelError),
}
