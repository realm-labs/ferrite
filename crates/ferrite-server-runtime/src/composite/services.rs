use std::collections::BTreeMap;

mod codec;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use ferrite_region_runtime::transfer::EntityTransfer;
use ferrite_simulation::scheduled_tick::level::ScheduleOutcome;
use ferrite_simulation::scheduled_tick::record::TickPriority;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::ChunkRevision;
use ferrite_world::id::BlockStateId;
use thiserror::Error;

use codec::{
    encode_action, encode_block_projection, encode_entity_projection, encode_player_projection,
};

use crate::composite::model::{
    CommittedCompositeContinuity, CompositeCommand, CompositeCommitReceipt, CompositeEvent,
    CompositeOwner, CompositeProjection, CompositeStage,
};
use crate::composite::runtime::{
    CompositeRegionRuntime, CompositeRuntimeConfig, CompositeRuntimeError,
};
use crate::continuity::identity::{ContinuityDomain, classify_domain};
use crate::continuity::migration::normalize_recovery_point;
use crate::entity_service::model::{
    EntityCommandHeader, EntityLifecycleState, EntityMutation, EntityPersistentState,
    EntityTransferRequest, LifecycleOutcome, ObserverOutcome,
};
use crate::entity_service::runtime::{
    EntityServiceRegionRuntime, EntityServiceRuntimeError, EntityServiceRuntimeLimits,
};
use crate::entity_service::transfer::{EntityTransferReceipt, TransferAcceptance};
use crate::player::block::replication::AuthoritativeBlockUpdate;
use crate::player_service::model::{
    ActionOutcome, PlayerActionHeader, PlayerMutation, PlayerPersistentState,
};
use crate::player_service::runtime::{PlayerServiceRegionRuntime, PlayerServiceRuntimeError};
use crate::simulation::boundary::MechanicBoundaryTransaction;
use crate::simulation::continuity::ScheduledQueueKind;
use crate::simulation::continuity::SimulationContinuity;
use crate::simulation::runtime::{
    BoundaryApplyOutcome, DeferredMechanicEffect, SimulationRegionRuntime, SimulationRuntimeConfig,
    SimulationRuntimeError,
};
use crate::world_service::model::{TicketOutcome, WorldServiceRuntimeConfig};
use crate::world_service::runtime::{WorldServiceRegionRuntime, WorldServiceRuntimeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeServiceCommand {
    tick: GameTick,
    sequence: u64,
    action: CompositeServiceAction,
}

impl CompositeServiceCommand {
    #[must_use]
    pub const fn new(tick: GameTick, sequence: u64, action: CompositeServiceAction) -> Self {
        Self {
            tick,
            sequence,
            action,
        }
    }

    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn action(&self) -> &CompositeServiceAction {
        &self.action
    }

    pub const fn owner(&self) -> CompositeOwner {
        match self.action {
            CompositeServiceAction::JoinPlayer { .. }
            | CompositeServiceAction::LeavePlayer { .. }
            | CompositeServiceAction::ApplyPlayerAction { .. }
            | CompositeServiceAction::OpenMenu { .. }
            | CompositeServiceAction::CloseMenu { .. } => CompositeOwner::PlayerService,
            CompositeServiceAction::ScheduleSimulation { .. } => CompositeOwner::Simulation,
            CompositeServiceAction::InsertEntity { .. }
            | CompositeServiceAction::AddEntityObserver { .. }
            | CompositeServiceAction::MutateEntity { .. } => CompositeOwner::EntityService,
            CompositeServiceAction::DemandChunk { .. }
            | CompositeServiceAction::SetWorldBlock { .. } => CompositeOwner::WorldService,
            CompositeServiceAction::ApplyBoundaryTransaction { .. }
            | CompositeServiceAction::PrepareEntityTransfer { .. }
            | CompositeServiceAction::AcceptEntityTransfer { .. }
            | CompositeServiceAction::CommitEntityTransfer { .. } => CompositeOwner::Reconciliation,
        }
    }

    fn metadata(&self) -> CompositeCommand {
        CompositeCommand::new(
            self.tick,
            self.owner(),
            self.sequence,
            self.kind(),
            encode_action(&self.action),
        )
    }

    fn kind(&self) -> ResourceId {
        let path = match self.action {
            CompositeServiceAction::JoinPlayer { .. } => "composite/player/join_v1",
            CompositeServiceAction::LeavePlayer { .. } => "composite/player/leave_v1",
            CompositeServiceAction::ApplyPlayerAction { .. } => "composite/player/action_v1",
            CompositeServiceAction::OpenMenu { .. } => "composite/player/open_menu_v1",
            CompositeServiceAction::CloseMenu { .. } => "composite/player/close_menu_v1",
            CompositeServiceAction::ScheduleSimulation { .. } => "composite/simulation/schedule_v1",
            CompositeServiceAction::InsertEntity { .. } => "composite/entity/insert_v1",
            CompositeServiceAction::AddEntityObserver { .. } => "composite/entity/add_observer_v1",
            CompositeServiceAction::MutateEntity { .. } => "composite/entity/mutate_v1",
            CompositeServiceAction::DemandChunk { .. } => "composite/world/demand_chunk_v1",
            CompositeServiceAction::SetWorldBlock { .. } => "composite/world/set_block_v1",
            CompositeServiceAction::ApplyBoundaryTransaction { .. } => {
                "composite/reconciliation/boundary_v1"
            }
            CompositeServiceAction::PrepareEntityTransfer { .. } => {
                "composite/reconciliation/prepare_transfer_v1"
            }
            CompositeServiceAction::AcceptEntityTransfer { .. } => {
                "composite/reconciliation/accept_transfer_v1"
            }
            CompositeServiceAction::CommitEntityTransfer { .. } => {
                "composite/reconciliation/commit_transfer_v1"
            }
        };
        ResourceId::new("ferrite", path).expect("static composite service command is valid")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeServiceAction {
    JoinPlayer {
        player: StableEntityId,
        state: PlayerPersistentState,
    },
    LeavePlayer {
        player: StableEntityId,
    },
    ApplyPlayerAction {
        header: PlayerActionHeader,
        mutation: PlayerMutation,
    },
    OpenMenu {
        header: PlayerActionHeader,
        container_id: u8,
    },
    CloseMenu {
        header: PlayerActionHeader,
    },
    ScheduleSimulation {
        kind: ScheduledQueueKind,
        type_identity: ResourceId,
        position: BlockPos,
        delay: i32,
        priority: TickPriority,
    },
    InsertEntity {
        entity: StableEntityId,
        state: EntityPersistentState,
    },
    AddEntityObserver {
        observer: StableEntityId,
    },
    MutateEntity {
        header: EntityCommandHeader,
        mutation: EntityMutation,
    },
    DemandChunk {
        position: ChunkPos,
    },
    SetWorldBlock {
        expected_revision: ChunkRevision,
        position: BlockPos,
        state: BlockStateId,
    },
    ApplyBoundaryTransaction {
        transaction: MechanicBoundaryTransaction,
    },
    PrepareEntityTransfer {
        request: EntityTransferRequest,
    },
    AcceptEntityTransfer {
        transfer: EntityTransfer,
    },
    CommitEntityTransfer {
        receipt: EntityTransferReceipt,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeServiceOutcome {
    PlayerJoined {
        sequence: u64,
        player: StableEntityId,
        session_epoch: u64,
    },
    PlayerLeft {
        sequence: u64,
        player: StableEntityId,
        state: PlayerPersistentState,
    },
    PlayerAction {
        sequence: u64,
        player: StableEntityId,
        outcome: ActionOutcome,
    },
    MenuOpened {
        sequence: u64,
        player: StableEntityId,
    },
    MenuClosed {
        sequence: u64,
        player: StableEntityId,
    },
    SimulationScheduled {
        sequence: u64,
        outcome: ScheduleOutcome,
    },
    EntityInserted {
        sequence: u64,
        entity: StableEntityId,
    },
    EntityObserverAdded {
        sequence: u64,
        observer: StableEntityId,
        outcome: ObserverOutcome,
    },
    EntityMutated {
        sequence: u64,
        entity: StableEntityId,
        outcome: LifecycleOutcome,
    },
    ChunkDemanded {
        sequence: u64,
        position: ChunkPos,
        outcome: TicketOutcome,
    },
    WorldBlockSet {
        sequence: u64,
        position: BlockPos,
        revision: ChunkRevision,
    },
    BoundaryApplied {
        sequence: u64,
        outcome: BoundaryApplyOutcome,
    },
    SimulationEffects {
        sequence: u64,
        effects: Vec<DeferredMechanicEffect>,
    },
    EntityTransferPrepared {
        sequence: u64,
        transfer: EntityTransfer,
    },
    EntityTransferAccepted {
        sequence: u64,
        acceptance: TransferAcceptance,
    },
    EntityTransferCommitted {
        sequence: u64,
        entity: StableEntityId,
    },
}

#[derive(Debug)]
pub struct CompositeServiceTickReport {
    pub commit: CompositeCommitReceipt,
    pub outcomes: Vec<CompositeServiceOutcome>,
    pub events: Vec<CompositeEvent>,
    pub continuity: CommittedCompositeContinuity,
    pub projections: Vec<CompositeProjection>,
}

#[derive(Debug)]
pub struct CompositeProductionRegionRuntime {
    coordinator: CompositeRegionRuntime,
    simulation: SimulationRegionRuntime,
    players: PlayerServiceRegionRuntime,
    entities: EntityServiceRegionRuntime,
    world: WorldServiceRegionRuntime,
    commands: BTreeMap<(GameTick, CompositeOwner, u64), CompositeServiceCommand>,
    poisoned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeProductionRuntimeConfig {
    pub coordinator: CompositeRuntimeConfig,
    pub simulation: SimulationRuntimeConfig,
    pub entities: EntityServiceRuntimeLimits,
    pub world: WorldServiceRuntimeConfig,
    pub player_capacity: usize,
    pub projection_capacity_per_player: usize,
}

impl CompositeProductionRegionRuntime {
    pub fn new(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        committed_tick: GameTick,
        game_time: i64,
        chunks: impl IntoIterator<Item = ChunkPos>,
        config: CompositeProductionRuntimeConfig,
    ) -> Result<Self, CompositeServiceRuntimeError> {
        let chunks = chunks.into_iter().collect::<Vec<_>>();
        let coordinator = CompositeRegionRuntime::new(
            key.clone(),
            generation,
            committed_tick,
            config.coordinator,
        )?;
        let simulation = SimulationRegionRuntime::new(
            key.clone(),
            generation,
            committed_tick,
            game_time,
            chunks.iter().copied(),
            config.simulation,
        )?;
        let players = PlayerServiceRegionRuntime::new(
            key.clone(),
            generation,
            config.player_capacity,
            config.projection_capacity_per_player,
        )?;
        let entities = EntityServiceRegionRuntime::new(
            key.clone(),
            generation,
            config.world.mapping,
            config.entities,
        )?;
        let mut world = WorldServiceRegionRuntime::new(key, generation, config.world)?;
        for chunk in chunks {
            world.demand_chunk(chunk)?;
        }
        Ok(Self {
            coordinator,
            simulation,
            players,
            entities,
            world,
            commands: BTreeMap::new(),
            poisoned: false,
        })
    }

    pub fn restore(
        point: &ferrite_persistence::snapshot::RegionRecoveryPoint,
        generation: ActivationGeneration,
        config: CompositeProductionRuntimeConfig,
    ) -> Result<Self, CompositeServiceRuntimeError> {
        let point = normalize_recovery_point(point)?;
        let records = crate::world_service::continuity::materialized_records(&point);
        let header = point.snapshot().header();
        if generation <= header.generation {
            return Err(CompositeServiceRuntimeError::RecoveryGenerationNotNewer);
        }
        let tick = GameTick::new(point.committed_tick());
        let game_time = i64::try_from(tick.get()).unwrap_or(i64::MAX);
        let bootstrap_only = records.iter().all(|record| {
            classify_domain(record.domain()).is_some_and(|classified| {
                matches!(
                    classified.domain,
                    ContinuityDomain::WorldLevel | ContinuityDomain::WorldMetadata
                )
            })
        });
        let coordinator =
            CompositeRegionRuntime::new(header.key.clone(), generation, tick, config.coordinator)?;
        let (simulation, players, entities) = if bootstrap_only {
            (
                SimulationRegionRuntime::new(
                    header.key.clone(),
                    generation,
                    tick,
                    game_time,
                    [],
                    config.simulation,
                )?,
                PlayerServiceRegionRuntime::new(
                    header.key.clone(),
                    generation,
                    config.player_capacity,
                    config.projection_capacity_per_player,
                )?,
                EntityServiceRegionRuntime::new(
                    header.key.clone(),
                    generation,
                    config.world.mapping,
                    config.entities,
                )?,
            )
        } else {
            (
                SimulationRegionRuntime::restore(
                    header.key.clone(),
                    generation,
                    tick,
                    game_time,
                    SimulationContinuity::from_records(&records)?,
                    config.simulation,
                )?,
                PlayerServiceRegionRuntime::restore(
                    header.key.clone(),
                    generation,
                    config.player_capacity,
                    config.projection_capacity_per_player,
                    &records,
                )?,
                EntityServiceRegionRuntime::restore(
                    header.key.clone(),
                    generation,
                    config.world.mapping,
                    config.entities,
                    &records,
                )?,
            )
        };
        let mut world = WorldServiceRegionRuntime::restore(
            header.key.clone(),
            generation,
            &point,
            config.world,
        )?;
        world.replace_auxiliary_records(world_auxiliary_records(&records))?;
        Ok(Self {
            coordinator,
            simulation,
            players,
            entities,
            world,
            commands: BTreeMap::new(),
            poisoned: false,
        })
    }

    pub const fn coordinator(&self) -> &CompositeRegionRuntime {
        &self.coordinator
    }

    pub const fn simulation(&self) -> &SimulationRegionRuntime {
        &self.simulation
    }

    pub const fn players(&self) -> &PlayerServiceRegionRuntime {
        &self.players
    }

    pub const fn entities(&self) -> &EntityServiceRegionRuntime {
        &self.entities
    }

    pub const fn world(&self) -> &WorldServiceRegionRuntime {
        &self.world
    }

    pub const fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    pub fn replace_world_auxiliary_records(
        &mut self,
        records: Vec<ferrite_persistence::snapshot::SnapshotRecord>,
    ) -> Result<(), CompositeServiceRuntimeError> {
        self.world.replace_auxiliary_records(records)?;
        Ok(())
    }

    pub fn admit_command(
        &mut self,
        command: CompositeServiceCommand,
    ) -> Result<(), CompositeServiceRuntimeError> {
        self.ensure_healthy()?;
        self.coordinator.admit_command(command.metadata())?;
        let identity = (command.tick(), command.owner(), command.sequence());
        let replaced = self.commands.insert(identity, command);
        debug_assert!(
            replaced.is_none(),
            "coordinator rejected duplicate identity"
        );
        Ok(())
    }

    pub fn run_tick(
        &mut self,
        tick: GameTick,
        game_time: i64,
        maximum_projections: usize,
    ) -> Result<CompositeServiceTickReport, CompositeServiceRuntimeError> {
        self.ensure_healthy()?;
        let result = self.run_tick_inner(tick, game_time, maximum_projections);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }

    fn run_tick_inner(
        &mut self,
        tick: GameTick,
        game_time: i64,
        maximum_projections: usize,
    ) -> Result<CompositeServiceTickReport, CompositeServiceRuntimeError> {
        self.coordinator.begin_tick(tick)?;
        let mut outcomes = Vec::new();
        let mut commit = None;
        let mut projections = Vec::new();
        let mut continuity = None;
        for stage in CompositeStage::ALL {
            self.coordinator.enter_stage(stage)?;
            match stage {
                CompositeStage::PlayerService => {
                    self.execute_player_commands(tick, &mut outcomes)?
                }
                CompositeStage::Simulation => {
                    self.execute_simulation_commands(tick, &mut outcomes)?
                }
                CompositeStage::EntityService => {
                    self.execute_entity_commands(tick, &mut outcomes)?
                }
                CompositeStage::WorldService => self.execute_world_commands(tick, &mut outcomes)?,
                CompositeStage::Reconciliation => {
                    self.execute_reconciliation_commands(tick, &mut outcomes)?
                }
                CompositeStage::Continuity => self.prepare_continuity()?,
                CompositeStage::Projection => {
                    projections = self.coordinator.drain_projections(maximum_projections)?;
                    continuity = self.coordinator.take_committed_continuity();
                }
                _ => {}
            }
            let receipt = self.coordinator.complete_stage()?;
            if let Some(receipt) = receipt {
                self.simulation.advance_commit(tick, game_time)?;
                commit = Some(receipt);
            }
        }
        self.commands
            .retain(|(command_tick, _, _), _| *command_tick > tick);
        Ok(CompositeServiceTickReport {
            commit: commit.ok_or(CompositeServiceRuntimeError::MissingCommit)?,
            outcomes,
            events: self.coordinator.take_events(usize::MAX),
            continuity: continuity.ok_or(CompositeServiceRuntimeError::MissingContinuity)?,
            projections,
        })
    }

    fn execute_player_commands(
        &mut self,
        tick: GameTick,
        outcomes: &mut Vec<CompositeServiceOutcome>,
    ) -> Result<(), CompositeServiceRuntimeError> {
        let commands = self.commands_for(tick, CompositeOwner::PlayerService);
        let projection_upper_bound = commands
            .iter()
            .filter(|command| {
                matches!(
                    command.action(),
                    CompositeServiceAction::JoinPlayer { .. }
                        | CompositeServiceAction::ApplyPlayerAction { .. }
                )
            })
            .count();
        if projection_upper_bound > self.coordinator.projection_remaining() {
            return Err(CompositeServiceRuntimeError::ProjectionBackpressure {
                required: projection_upper_bound,
                remaining: self.coordinator.projection_remaining(),
            });
        }
        for command in commands {
            let player = match command.action {
                CompositeServiceAction::JoinPlayer { player, state } => {
                    let session_epoch = self.players.join(player, state)?;
                    outcomes.push(CompositeServiceOutcome::PlayerJoined {
                        sequence: command.sequence,
                        player,
                        session_epoch,
                    });
                    Some(player)
                }
                CompositeServiceAction::LeavePlayer { player } => {
                    let state = self.players.leave(player)?;
                    outcomes.push(CompositeServiceOutcome::PlayerLeft {
                        sequence: command.sequence,
                        player,
                        state,
                    });
                    None
                }
                CompositeServiceAction::ApplyPlayerAction { header, mutation } => {
                    let player = header.player;
                    let outcome = self.players.apply_player_action(&header, mutation)?;
                    outcomes.push(CompositeServiceOutcome::PlayerAction {
                        sequence: command.sequence,
                        player,
                        outcome,
                    });
                    Some(player)
                }
                CompositeServiceAction::OpenMenu {
                    header,
                    container_id,
                } => {
                    let player = header.player;
                    self.players.open_menu(&header, container_id)?;
                    outcomes.push(CompositeServiceOutcome::MenuOpened {
                        sequence: command.sequence,
                        player,
                    });
                    None
                }
                CompositeServiceAction::CloseMenu { header } => {
                    let player = header.player;
                    self.players.close_menu(&header)?;
                    outcomes.push(CompositeServiceOutcome::MenuClosed {
                        sequence: command.sequence,
                        player,
                    });
                    None
                }
                _ => {
                    return Err(CompositeServiceRuntimeError::WrongCommandOwner);
                }
            };
            if let Some(player) = player {
                for projection in self.players.drain_projections(player, usize::MAX)? {
                    self.coordinator
                        .queue_projection(encode_player_projection(projection))?;
                }
            }
        }
        Ok(())
    }

    fn execute_simulation_commands(
        &mut self,
        tick: GameTick,
        outcomes: &mut Vec<CompositeServiceOutcome>,
    ) -> Result<(), CompositeServiceRuntimeError> {
        for command in self.commands_for(tick, CompositeOwner::Simulation) {
            let CompositeServiceAction::ScheduleSimulation {
                kind,
                type_identity,
                position,
                delay,
                priority,
            } = command.action
            else {
                return Err(CompositeServiceRuntimeError::WrongCommandOwner);
            };
            let outcome =
                self.simulation
                    .schedule_local(kind, type_identity, position, delay, priority)?;
            outcomes.push(CompositeServiceOutcome::SimulationScheduled {
                sequence: command.sequence,
                outcome,
            });
        }
        Ok(())
    }

    fn execute_entity_commands(
        &mut self,
        tick: GameTick,
        outcomes: &mut Vec<CompositeServiceOutcome>,
    ) -> Result<(), CompositeServiceRuntimeError> {
        for command in self.commands_for(tick, CompositeOwner::EntityService) {
            let required = match &command.action {
                CompositeServiceAction::InsertEntity { state, .. }
                    if state.lifecycle == EntityLifecycleState::Active =>
                {
                    self.entities.observer_count()
                }
                CompositeServiceAction::AddEntityObserver { .. } => self.entities.entity_count(),
                CompositeServiceAction::MutateEntity { .. } => self.entities.observer_count(),
                _ => 0,
            };
            self.require_projection_capacity(required)?;
            match command.action {
                CompositeServiceAction::InsertEntity { entity, state } => {
                    self.entities.insert(entity, state)?;
                    outcomes.push(CompositeServiceOutcome::EntityInserted {
                        sequence: command.sequence,
                        entity,
                    });
                }
                CompositeServiceAction::AddEntityObserver { observer } => {
                    let outcome = self.entities.add_observer(observer)?;
                    outcomes.push(CompositeServiceOutcome::EntityObserverAdded {
                        sequence: command.sequence,
                        observer,
                        outcome,
                    });
                }
                CompositeServiceAction::MutateEntity { header, mutation } => {
                    let entity = header.entity;
                    let outcome = self.entities.apply_mutation(&header, mutation)?;
                    outcomes.push(CompositeServiceOutcome::EntityMutated {
                        sequence: command.sequence,
                        entity,
                        outcome,
                    });
                }
                _ => return Err(CompositeServiceRuntimeError::WrongCommandOwner),
            }
            self.collect_entity_projections()?;
        }
        Ok(())
    }

    fn execute_world_commands(
        &mut self,
        tick: GameTick,
        outcomes: &mut Vec<CompositeServiceOutcome>,
    ) -> Result<(), CompositeServiceRuntimeError> {
        for command in self.commands_for(tick, CompositeOwner::WorldService) {
            match command.action {
                CompositeServiceAction::DemandChunk { position } => {
                    let outcome = self.world.demand_chunk(position)?;
                    outcomes.push(CompositeServiceOutcome::ChunkDemanded {
                        sequence: command.sequence,
                        position,
                        outcome,
                    });
                }
                CompositeServiceAction::SetWorldBlock {
                    expected_revision,
                    position,
                    state,
                } => {
                    self.require_projection_capacity(1)?;
                    let revision = self.world.set_block(
                        self.coordinator.key(),
                        self.coordinator.generation(),
                        expected_revision,
                        position,
                        state,
                    )?;
                    self.coordinator.queue_projection(encode_block_projection(
                        command.sequence,
                        AuthoritativeBlockUpdate { position, state },
                    ))?;
                    outcomes.push(CompositeServiceOutcome::WorldBlockSet {
                        sequence: command.sequence,
                        position,
                        revision,
                    });
                }
                _ => return Err(CompositeServiceRuntimeError::WrongCommandOwner),
            }
        }
        Ok(())
    }

    fn execute_reconciliation_commands(
        &mut self,
        tick: GameTick,
        outcomes: &mut Vec<CompositeServiceOutcome>,
    ) -> Result<(), CompositeServiceRuntimeError> {
        for command in self.commands_for(tick, CompositeOwner::Reconciliation) {
            match command.action {
                CompositeServiceAction::ApplyBoundaryTransaction { transaction } => {
                    self.require_projection_capacity(transaction.mutations().len())?;
                    let outcome = self
                        .simulation
                        .apply_transaction(self.world.voxels_mut(), &transaction)?;
                    outcomes.push(CompositeServiceOutcome::BoundaryApplied {
                        sequence: command.sequence,
                        outcome,
                    });
                    let mut effects = Vec::new();
                    for kind in [
                        crate::simulation::budget::SimulationQueueKind::ImmediateNeighbors,
                        crate::simulation::budget::SimulationQueueKind::Fluids,
                        crate::simulation::budget::SimulationQueueKind::Redstone,
                        crate::simulation::budget::SimulationQueueKind::Lighting,
                    ] {
                        effects.extend(self.simulation.drain_effects(kind, usize::MAX)?);
                    }
                    outcomes.push(CompositeServiceOutcome::SimulationEffects {
                        sequence: command.sequence,
                        effects,
                    });
                    for update in self.simulation.take_projection_updates()? {
                        self.coordinator
                            .queue_projection(encode_block_projection(command.sequence, update))?;
                    }
                }
                CompositeServiceAction::PrepareEntityTransfer { request } => {
                    self.require_projection_capacity(self.entities.observer_count())?;
                    let transfer = self.entities.prepare_transfer(request)?;
                    outcomes.push(CompositeServiceOutcome::EntityTransferPrepared {
                        sequence: command.sequence,
                        transfer,
                    });
                    self.collect_entity_projections()?;
                }
                CompositeServiceAction::AcceptEntityTransfer { transfer } => {
                    self.require_projection_capacity(self.entities.observer_count())?;
                    let acceptance = self.entities.accept_transfer(&transfer)?;
                    outcomes.push(CompositeServiceOutcome::EntityTransferAccepted {
                        sequence: command.sequence,
                        acceptance,
                    });
                    self.collect_entity_projections()?;
                }
                CompositeServiceAction::CommitEntityTransfer { receipt } => {
                    let entity = receipt.entity;
                    self.entities.commit_transfer(&receipt)?;
                    outcomes.push(CompositeServiceOutcome::EntityTransferCommitted {
                        sequence: command.sequence,
                        entity,
                    });
                }
                _ => return Err(CompositeServiceRuntimeError::WrongCommandOwner),
            }
        }
        Ok(())
    }

    fn collect_entity_projections(&mut self) -> Result<(), CompositeServiceRuntimeError> {
        let observers = self.entities.observers().collect::<Vec<_>>();
        for observer in observers {
            for projection in self.entities.drain_projections(observer, usize::MAX)? {
                self.coordinator
                    .queue_projection(encode_entity_projection(projection))?;
            }
        }
        Ok(())
    }

    fn require_projection_capacity(
        &self,
        required: usize,
    ) -> Result<(), CompositeServiceRuntimeError> {
        let remaining = self.coordinator.projection_remaining();
        if required > remaining {
            Err(CompositeServiceRuntimeError::ProjectionBackpressure {
                required,
                remaining,
            })
        } else {
            Ok(())
        }
    }

    fn prepare_continuity(&mut self) -> Result<(), CompositeServiceRuntimeError> {
        let mut records = self.simulation.capture_continuity()?.to_records()?;
        records.extend(self.players.capture_continuity()?);
        records.extend(self.entities.snapshot_records()?);
        records.extend(self.world.snapshot_records()?);
        self.coordinator.prepare_continuity(records)?;
        Ok(())
    }

    fn commands_for(&self, tick: GameTick, owner: CompositeOwner) -> Vec<CompositeServiceCommand> {
        self.commands
            .range((tick, owner, 0)..=(tick, owner, u64::MAX))
            .map(|(_, command)| command.clone())
            .collect()
    }

    fn ensure_healthy(&self) -> Result<(), CompositeServiceRuntimeError> {
        if self.poisoned {
            Err(CompositeServiceRuntimeError::Poisoned)
        } else {
            Ok(())
        }
    }
}

fn world_auxiliary_records(
    records: &[ferrite_persistence::snapshot::SnapshotRecord],
) -> Vec<ferrite_persistence::snapshot::SnapshotRecord> {
    records
        .iter()
        .filter(|record| {
            classify_domain(record.domain()).is_none_or(|classified| {
                matches!(
                    classified.domain,
                    ContinuityDomain::WorldLevel | ContinuityDomain::WorldMetadata
                )
            })
        })
        .cloned()
        .collect()
}

#[derive(Debug, Error)]
pub enum CompositeServiceRuntimeError {
    #[error("composite production Region runtime is poisoned")]
    Poisoned,
    #[error("composite service command reached the wrong owner stage")]
    WrongCommandOwner,
    #[error("composite player projections require {required} slots but only {remaining} remain")]
    ProjectionBackpressure { required: usize, remaining: usize },
    #[error("composite tick completed without a commit receipt")]
    MissingCommit,
    #[error("composite tick completed without committed continuity")]
    MissingContinuity,
    #[error("composite recovery activation generation is not newer than durable state")]
    RecoveryGenerationNotNewer,
    #[error(transparent)]
    Coordinator(#[from] CompositeRuntimeError),
    #[error(transparent)]
    Simulation(#[from] SimulationRuntimeError),
    #[error(transparent)]
    Player(#[from] PlayerServiceRuntimeError),
    #[error(transparent)]
    Entity(#[from] EntityServiceRuntimeError),
    #[error(transparent)]
    World(#[from] WorldServiceRuntimeError),
    #[error(transparent)]
    Continuity(#[from] crate::simulation::continuity::ContinuityError),
    #[error(transparent)]
    Migration(#[from] crate::continuity::migration::ContinuityMigrationError),
}
