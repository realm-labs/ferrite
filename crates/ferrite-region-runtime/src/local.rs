//! Deterministic in-process runner for a bounded Region consistency island.

use crate::immediate::{ImmediateBoundaryEffect, ImmediateEffectError, ImmediateEffectQueue};
use crate::logic::{
    ImmediateEffectContext, RegionLogic, RegionLogicError, RegionPhaseContext, RegionPhaseOutput,
};
use crate::transfer::{
    EntityTransfer, EntityTransferError, EntityTransferQueue, TransferredEntityState,
};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_simulation::boundary::{BoundaryBatch, BoundaryError, BoundaryInbox};
use ferrite_simulation::command::{CommandError, CommandInbox, RegionCommand};
use ferrite_simulation::entity::RegionEntityError;
use ferrite_simulation::journal::{CommittedTickJournal, JournalDomain};
use ferrite_simulation::pipeline::{PipelineError, RegionTickPipeline};
use ferrite_simulation::region::{RegionSimulationState, RegionSimulationView};
use ferrite_simulation::tick::{GameTick, PhaseBarrier, TickPhase};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalRunnerConfig {
    pub command_capacity: usize,
    pub boundary_capacity: usize,
    pub immediate_effect_capacity: usize,
    pub transfer_capacity: usize,
    pub journal_capacity: usize,
    pub phase_output_capacity: usize,
    pub maximum_future_command_ticks: u64,
}

impl LocalRunnerConfig {
    pub const fn testing() -> Self {
        Self {
            command_capacity: 64,
            boundary_capacity: 64,
            immediate_effect_capacity: 64,
            transfer_capacity: 64,
            journal_capacity: 256,
            phase_output_capacity: 64,
            maximum_future_command_ticks: 4,
        }
    }

    fn validate(self) -> Result<(), LocalRunnerError> {
        for (name, value) in [
            ("command", self.command_capacity),
            ("boundary", self.boundary_capacity),
            ("immediate effect", self.immediate_effect_capacity),
            ("transfer", self.transfer_capacity),
            ("journal", self.journal_capacity),
            ("phase output", self.phase_output_capacity),
        ] {
            if value == 0 {
                return Err(LocalRunnerError::ZeroCapacity(name));
            }
        }
        if self.maximum_future_command_ticks == 0 {
            return Err(LocalRunnerError::ZeroCommandHorizon);
        }
        Ok(())
    }
}

pub struct LocalRegionRunner {
    config: LocalRunnerConfig,
    regions: BTreeMap<SimulationRegionKey, LocalRegion>,
    effects: ImmediateEffectQueue,
    transfers: EntityTransferQueue,
    poisoned: bool,
}

impl LocalRegionRunner {
    pub fn new(config: LocalRunnerConfig) -> Result<Self, LocalRunnerError> {
        config.validate()?;
        Ok(Self {
            config,
            regions: BTreeMap::new(),
            effects: ImmediateEffectQueue::new(config.immediate_effect_capacity)?,
            transfers: EntityTransferQueue::new(config.transfer_capacity)?,
            poisoned: false,
        })
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub const fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    pub fn insert_region(
        &mut self,
        state: RegionSimulationState,
        generation: ActivationGeneration,
        committed_tick: GameTick,
    ) -> Result<(), LocalRunnerError> {
        self.ensure_healthy()?;
        let key = state.key().clone();
        if self.regions.contains_key(&key) {
            return Err(LocalRunnerError::DuplicateRegion(key));
        }
        let region = LocalRegion::new(state, generation, committed_tick, self.config)?;
        self.regions.insert(key, region);
        Ok(())
    }

    pub fn region(&self, key: &SimulationRegionKey) -> Option<LocalRegionView<'_>> {
        self.regions.get(key).map(LocalRegion::view)
    }

    pub fn admit_command(&mut self, command: RegionCommand) -> Result<(), LocalRunnerError> {
        self.ensure_healthy()?;
        let key = command.target().clone();
        let region = self
            .regions
            .get_mut(&key)
            .ok_or(LocalRunnerError::UnknownRegion(key))?;
        region
            .commands
            .admit(command, region.pipeline.committed_tick())?;
        Ok(())
    }

    pub fn admit_boundary(&mut self, batch: BoundaryBatch) -> Result<(), LocalRunnerError> {
        self.ensure_healthy()?;
        let source_generation = self.generation(batch.source())?;
        let target = batch.target().clone();
        let region = self
            .regions
            .get_mut(&target)
            .ok_or_else(|| LocalRunnerError::UnknownRegion(target.clone()))?;
        region
            .boundaries
            .admit(batch, source_generation, region.pipeline.committed_tick())?;
        Ok(())
    }

    pub fn admit_immediate(
        &mut self,
        effect: ImmediateBoundaryEffect,
    ) -> Result<(), LocalRunnerError> {
        self.ensure_healthy()?;
        let source_generation = self.generation(effect.source())?;
        let target_generation = self.generation(effect.target())?;
        let committed_tick = self.maximum_committed_tick(effect.source(), effect.target())?;
        self.effects
            .admit(effect, source_generation, target_generation, committed_tick)?;
        Ok(())
    }

    pub fn admit_transfer(&mut self, transfer: EntityTransfer) -> Result<(), LocalRunnerError> {
        self.ensure_healthy()?;
        let source_generation = self.generation(transfer.source())?;
        let target_generation = self.generation(transfer.target())?;
        let committed_tick = self.maximum_committed_tick(transfer.source(), transfer.target())?;
        self.transfers.admit(
            transfer,
            source_generation,
            target_generation,
            committed_tick,
        )?;
        Ok(())
    }

    pub fn run_tick(
        &mut self,
        tick: GameTick,
        logic: &mut impl RegionLogic,
    ) -> Result<LocalTickReport, LocalRunnerError> {
        self.ensure_healthy()?;
        let result = self.run_tick_inner(tick, logic);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }

    fn run_tick_inner(
        &mut self,
        tick: GameTick,
        logic: &mut impl RegionLogic,
    ) -> Result<LocalTickReport, LocalRunnerError> {
        if self.regions.is_empty() {
            return Err(LocalRunnerError::NoRegions);
        }
        let keys = self.regions.keys().cloned().collect::<Vec<_>>();
        for region in self.regions.values_mut() {
            region.pipeline.begin_tick(tick)?;
        }

        let mut immediate_effects = 0;
        let mut entity_transfers = 0;
        for phase in TickPhase::ALL {
            if phase == TickPhase::ReconcileBoundary {
                entity_transfers += self.apply_transfers(tick)?;
            }
            let mut emitted = Vec::with_capacity(keys.len());
            for key in &keys {
                let output = self.execute_region_phase(key, tick, phase, logic)?;
                emitted.push((key.clone(), output));
            }
            self.route_phase_outputs(tick, phase, emitted)?;
            immediate_effects += self.apply_immediate_effects(tick, phase, logic)?;
            self.finish_phase(phase)?;
        }
        if self.effects.has_tick_at_or_before(tick) || self.transfers.has_tick_at_or_before(tick) {
            return Err(LocalRunnerError::UndeliveredRequiredWork(tick));
        }

        for region in self.regions.values() {
            if region.boundaries.has_pending_at_or_before(tick)
                || region.commands.has_pending_at_or_before(tick)
            {
                return Err(LocalRunnerError::UndeliveredRequiredWork(tick));
            }
        }
        let mut commits = Vec::with_capacity(keys.len());
        for key in keys {
            let region = self
                .regions
                .get_mut(&key)
                .ok_or_else(|| LocalRunnerError::UnknownRegion(key.clone()))?;
            let journal = region.pipeline.commit_tick()?;
            region.commands.prune_committed(tick);
            region.boundaries.prune_committed(tick);
            commits.push(LocalRegionCommit {
                key,
                generation: region.generation,
                journal,
            });
        }
        self.effects.prune_committed(tick);
        self.transfers.prune_committed(tick);
        Ok(LocalTickReport {
            tick,
            commits: commits.into_boxed_slice(),
            immediate_effects,
            entity_transfers,
        })
    }

    fn execute_region_phase(
        &mut self,
        key: &SimulationRegionKey,
        tick: GameTick,
        phase: TickPhase,
        logic: &mut impl RegionLogic,
    ) -> Result<RegionPhaseOutput, LocalRunnerError> {
        let region = self
            .regions
            .get_mut(key)
            .ok_or_else(|| LocalRunnerError::UnknownRegion(key.clone()))?;
        if region.pipeline.current_phase() != Some(phase) {
            return Err(LocalRunnerError::PhaseDivergence(key.clone()));
        }
        let commands = if phase == TickPhase::Ingress {
            region.commands.drain_tick(tick)
        } else {
            Vec::new()
        };
        let boundaries = region.boundaries.drain(tick, phase);
        let context = RegionPhaseContext::new(
            region.generation,
            &mut region.state,
            &mut region.pipeline,
            &commands,
            &boundaries,
        );
        let mut output = RegionPhaseOutput::new(self.config.phase_output_capacity);
        logic.execute_phase(context, &mut output)?;
        Ok(output)
    }

    fn route_phase_outputs(
        &mut self,
        tick: GameTick,
        phase: TickPhase,
        outputs: Vec<(SimulationRegionKey, RegionPhaseOutput)>,
    ) -> Result<(), LocalRunnerError> {
        for (source, output) in outputs {
            if phase == TickPhase::Commit && !output.is_empty() {
                return Err(LocalRunnerError::CommitProducedWork);
            }
            let (boundaries, effects, transfers) = output.into_parts();
            for batch in boundaries {
                self.validate_output(&source, tick, batch.tick(), batch.source())?;
                if batch.phase() <= phase {
                    return Err(LocalRunnerError::BoundaryPhaseAlreadyPassed {
                        emitted: phase,
                        target: batch.phase(),
                    });
                }
                self.admit_boundary(batch)?;
            }
            for effect in effects {
                self.validate_output(&source, tick, effect.tick(), effect.source())?;
                if effect.phase() != phase {
                    return Err(LocalRunnerError::OutputPhaseMismatch {
                        emitted: phase,
                        tagged: effect.phase(),
                    });
                }
                self.admit_immediate(effect)?;
            }
            for transfer in transfers {
                self.validate_output(&source, tick, transfer.tick(), transfer.source())?;
                if phase >= TickPhase::ReconcileBoundary {
                    return Err(LocalRunnerError::TransferPhaseAlreadyPassed(phase));
                }
                self.admit_transfer(transfer)?;
            }
        }
        Ok(())
    }

    fn apply_immediate_effects(
        &mut self,
        tick: GameTick,
        phase: TickPhase,
        logic: &mut impl RegionLogic,
    ) -> Result<usize, LocalRunnerError> {
        let effects = self.effects.drain(tick, phase);
        let count = effects.len();
        for effect in &effects {
            let target = effect.target().clone();
            let region = self
                .regions
                .get_mut(&target)
                .ok_or_else(|| LocalRunnerError::UnknownRegion(target.clone()))?;
            region.pipeline.append_journal(
                JournalDomain::Mutation,
                effect.kind().clone(),
                effect.payload().to_vec(),
            )?;
            let context = ImmediateEffectContext::new(
                region.generation,
                &mut region.state,
                &mut region.pipeline,
                effect,
            );
            logic.apply_immediate_effect(context)?;
        }
        Ok(count)
    }

    fn apply_transfers(&mut self, tick: GameTick) -> Result<usize, LocalRunnerError> {
        let transfers = self.transfers.drain_tick(tick);
        let count = transfers.len();
        for transfer in transfers {
            self.apply_transfer(&transfer)?;
        }
        Ok(count)
    }

    fn apply_transfer(&mut self, transfer: &EntityTransfer) -> Result<(), LocalRunnerError> {
        let source_key = transfer.source().clone();
        let target_key = transfer.target().clone();
        let mut source = self
            .regions
            .remove(&source_key)
            .ok_or_else(|| LocalRunnerError::UnknownRegion(source_key.clone()))?;
        let result = (|| {
            let target = self
                .regions
                .get_mut(&target_key)
                .ok_or_else(|| LocalRunnerError::UnknownRegion(target_key.clone()))?;
            if source.generation != transfer.source_generation()
                || target.generation != transfer.target_generation()
            {
                return Err(LocalRunnerError::StaleTransferGeneration);
            }
            if !source
                .state
                .view()
                .entities()
                .contains(transfer.stable_id())
            {
                return Err(LocalRunnerError::MissingTransferSource(
                    transfer.stable_id(),
                ));
            }
            if target
                .state
                .view()
                .entities()
                .contains(transfer.stable_id())
            {
                return Err(LocalRunnerError::OccupiedTransferTarget(
                    transfer.stable_id(),
                ));
            }
            source.pipeline.append_journal(
                JournalDomain::Mutation,
                transfer.kind().clone(),
                transfer.state().to_vec(),
            )?;
            target.pipeline.append_journal(
                JournalDomain::Mutation,
                transfer.kind().clone(),
                transfer.state().to_vec(),
            )?;
            target.state.entities_mut().spawn(transfer.stable_id())?;
            if let Err(error) = target.state.entities_mut().insert_component(
                transfer.stable_id(),
                TransferredEntityState::from_transfer(transfer),
            ) {
                target.state.entities_mut().despawn(transfer.stable_id())?;
                return Err(error.into());
            }
            if let Err(error) = source.state.entities_mut().despawn(transfer.stable_id()) {
                target.state.entities_mut().despawn(transfer.stable_id())?;
                return Err(error.into());
            }
            Ok(())
        })();
        self.regions.insert(source_key, source);
        result
    }

    fn finish_phase(&mut self, phase: TickPhase) -> Result<(), LocalRunnerError> {
        for region in self.regions.values_mut() {
            let barrier = phase.contract().barrier();
            if !matches!(barrier, PhaseBarrier::None | PhaseBarrier::Commit) {
                region.pipeline.satisfy_current_barrier()?;
            }
        }
        if phase != TickPhase::Commit {
            for region in self.regions.values_mut() {
                region.pipeline.advance_phase()?;
            }
        }
        Ok(())
    }

    fn validate_output(
        &self,
        source: &SimulationRegionKey,
        tick: GameTick,
        output_tick: GameTick,
        output_source: &SimulationRegionKey,
    ) -> Result<(), LocalRunnerError> {
        if output_source != source {
            return Err(LocalRunnerError::OutputSourceMismatch);
        }
        if output_tick != tick {
            return Err(LocalRunnerError::OutputTickMismatch {
                running: tick,
                tagged: output_tick,
            });
        }
        Ok(())
    }

    fn generation(
        &self,
        key: &SimulationRegionKey,
    ) -> Result<ActivationGeneration, LocalRunnerError> {
        self.regions
            .get(key)
            .map(|region| region.generation)
            .ok_or_else(|| LocalRunnerError::UnknownRegion(key.clone()))
    }

    fn maximum_committed_tick(
        &self,
        left: &SimulationRegionKey,
        right: &SimulationRegionKey,
    ) -> Result<GameTick, LocalRunnerError> {
        let left = self
            .regions
            .get(left)
            .ok_or_else(|| LocalRunnerError::UnknownRegion(left.clone()))?
            .pipeline
            .committed_tick();
        let right = self
            .regions
            .get(right)
            .ok_or_else(|| LocalRunnerError::UnknownRegion(right.clone()))?
            .pipeline
            .committed_tick();
        Ok(left.max(right))
    }

    fn ensure_healthy(&self) -> Result<(), LocalRunnerError> {
        if self.poisoned {
            Err(LocalRunnerError::Poisoned)
        } else {
            Ok(())
        }
    }
}

struct LocalRegion {
    generation: ActivationGeneration,
    state: RegionSimulationState,
    pipeline: RegionTickPipeline,
    commands: CommandInbox,
    boundaries: BoundaryInbox,
}

impl LocalRegion {
    fn new(
        state: RegionSimulationState,
        generation: ActivationGeneration,
        committed_tick: GameTick,
        config: LocalRunnerConfig,
    ) -> Result<Self, LocalRunnerError> {
        let key = state.key().clone();
        Ok(Self {
            generation,
            state,
            pipeline: RegionTickPipeline::new(committed_tick, config.journal_capacity)?,
            commands: CommandInbox::new(
                key.clone(),
                config.command_capacity,
                config.maximum_future_command_ticks,
            )?,
            boundaries: BoundaryInbox::new(key, config.boundary_capacity)?,
        })
    }

    fn view(&self) -> LocalRegionView<'_> {
        LocalRegionView {
            generation: self.generation,
            committed_tick: self.pipeline.committed_tick(),
            state: self.state.view(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct LocalRegionView<'a> {
    generation: ActivationGeneration,
    committed_tick: GameTick,
    state: RegionSimulationView<'a>,
}

impl<'a> LocalRegionView<'a> {
    pub const fn generation(self) -> ActivationGeneration {
        self.generation
    }

    pub const fn committed_tick(self) -> GameTick {
        self.committed_tick
    }

    pub const fn state(self) -> RegionSimulationView<'a> {
        self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRegionCommit {
    key: SimulationRegionKey,
    generation: ActivationGeneration,
    journal: CommittedTickJournal,
}

impl LocalRegionCommit {
    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub const fn generation(&self) -> ActivationGeneration {
        self.generation
    }

    pub const fn journal(&self) -> &CommittedTickJournal {
        &self.journal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTickReport {
    tick: GameTick,
    commits: Box<[LocalRegionCommit]>,
    immediate_effects: usize,
    entity_transfers: usize,
}

impl LocalTickReport {
    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub fn commits(&self) -> &[LocalRegionCommit] {
        &self.commits
    }

    pub const fn immediate_effects(&self) -> usize {
        self.immediate_effects
    }

    pub const fn entity_transfers(&self) -> usize {
        self.entity_transfers
    }
}

#[derive(Debug, Error)]
pub enum LocalRunnerError {
    #[error("{0} capacity cannot be zero")]
    ZeroCapacity(&'static str),
    #[error("maximum future command horizon cannot be zero")]
    ZeroCommandHorizon,
    #[error("local Region runner has no Regions")]
    NoRegions,
    #[error("local Region runner is poisoned by an uncommitted tick failure")]
    Poisoned,
    #[error("Region {0:?} is already active")]
    DuplicateRegion(SimulationRegionKey),
    #[error("Region {0:?} is not active")]
    UnknownRegion(SimulationRegionKey),
    #[error("Region {0:?} diverged from the runner phase")]
    PhaseDivergence(SimulationRegionKey),
    #[error("phase output claims another source Region")]
    OutputSourceMismatch,
    #[error("phase output tick {tagged:?} does not match running tick {running:?}")]
    OutputTickMismatch { running: GameTick, tagged: GameTick },
    #[error("immediate effect phase {tagged:?} does not match emission phase {emitted:?}")]
    OutputPhaseMismatch {
        emitted: TickPhase,
        tagged: TickPhase,
    },
    #[error("boundary target phase {target:?} has passed emission phase {emitted:?}")]
    BoundaryPhaseAlreadyPassed {
        emitted: TickPhase,
        target: TickPhase,
    },
    #[error("entity transfer was emitted after reconciliation at {0:?}")]
    TransferPhaseAlreadyPassed(TickPhase),
    #[error("commit phase cannot produce new boundary work")]
    CommitProducedWork,
    #[error("required work remains undelivered through tick {0:?}")]
    UndeliveredRequiredWork(GameTick),
    #[error("entity transfer activation generation changed after admission")]
    StaleTransferGeneration,
    #[error("transfer source entity {0} does not exist")]
    MissingTransferSource(ferrite_foundation::identity::StableEntityId),
    #[error("transfer target already contains entity {0}")]
    OccupiedTransferTarget(ferrite_foundation::identity::StableEntityId),
    #[error(transparent)]
    Pipeline(#[from] PipelineError),
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Boundary(#[from] BoundaryError),
    #[error(transparent)]
    Immediate(#[from] ImmediateEffectError),
    #[error(transparent)]
    Transfer(#[from] EntityTransferError),
    #[error(transparent)]
    Entity(#[from] RegionEntityError),
    #[error(transparent)]
    Logic(#[from] RegionLogicError),
}
