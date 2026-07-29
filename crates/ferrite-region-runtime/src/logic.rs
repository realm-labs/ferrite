//! Executor-neutral Region logic interface used by the local runner.

use crate::immediate::ImmediateBoundaryEffect;
use crate::transfer::EntityTransfer;
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::boundary::BoundaryBatch;
use ferrite_simulation::command::RegionCommand;
use ferrite_simulation::journal::JournalDomain;
use ferrite_simulation::pipeline::{PipelineError, RegionTickPipeline};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::{GameTick, TickPhase};
use thiserror::Error;

pub struct RegionPhaseContext<'a> {
    generation: ActivationGeneration,
    state: &'a mut RegionSimulationState,
    pipeline: &'a mut RegionTickPipeline,
    commands: &'a [RegionCommand],
    boundaries: &'a [BoundaryBatch],
}

impl<'a> RegionPhaseContext<'a> {
    pub const fn key(&self) -> &SimulationRegionKey {
        self.state.key()
    }

    pub const fn generation(&self) -> ActivationGeneration {
        self.generation
    }

    pub fn tick(&self) -> GameTick {
        self.pipeline
            .active_tick()
            .expect("a Region phase context always has an active tick")
    }

    pub fn phase(&self) -> TickPhase {
        self.pipeline
            .current_phase()
            .expect("a Region phase context always has an active phase")
    }

    pub fn commands(&self) -> &[RegionCommand] {
        self.commands
    }

    pub fn boundaries(&self) -> &[BoundaryBatch] {
        self.boundaries
    }

    pub fn state(&self) -> &RegionSimulationState {
        self.state
    }

    pub fn state_mut(&mut self) -> &mut RegionSimulationState {
        self.state
    }

    pub fn append_journal(
        &mut self,
        domain: JournalDomain,
        kind: ResourceId,
        payload: Vec<u8>,
    ) -> Result<u64, PipelineError> {
        self.pipeline.append_journal(domain, kind, payload)
    }

    pub(crate) fn new(
        generation: ActivationGeneration,
        state: &'a mut RegionSimulationState,
        pipeline: &'a mut RegionTickPipeline,
        commands: &'a [RegionCommand],
        boundaries: &'a [BoundaryBatch],
    ) -> Self {
        Self {
            generation,
            state,
            pipeline,
            commands,
            boundaries,
        }
    }
}

pub struct ImmediateEffectContext<'a> {
    generation: ActivationGeneration,
    state: &'a mut RegionSimulationState,
    pipeline: &'a mut RegionTickPipeline,
    effect: &'a ImmediateBoundaryEffect,
}

impl<'a> ImmediateEffectContext<'a> {
    pub const fn key(&self) -> &SimulationRegionKey {
        self.state.key()
    }

    pub const fn generation(&self) -> ActivationGeneration {
        self.generation
    }

    pub const fn effect(&self) -> &ImmediateBoundaryEffect {
        self.effect
    }

    pub fn state(&self) -> &RegionSimulationState {
        self.state
    }

    pub fn state_mut(&mut self) -> &mut RegionSimulationState {
        self.state
    }

    pub fn append_journal(
        &mut self,
        domain: JournalDomain,
        kind: ResourceId,
        payload: Vec<u8>,
    ) -> Result<u64, PipelineError> {
        self.pipeline.append_journal(domain, kind, payload)
    }

    pub(crate) fn new(
        generation: ActivationGeneration,
        state: &'a mut RegionSimulationState,
        pipeline: &'a mut RegionTickPipeline,
        effect: &'a ImmediateBoundaryEffect,
    ) -> Self {
        Self {
            generation,
            state,
            pipeline,
            effect,
        }
    }
}

pub trait RegionLogic {
    fn execute_phase(
        &mut self,
        context: RegionPhaseContext<'_>,
        output: &mut RegionPhaseOutput,
    ) -> Result<(), RegionLogicError>;

    fn apply_immediate_effect(
        &mut self,
        context: ImmediateEffectContext<'_>,
    ) -> Result<(), RegionLogicError>;
}

#[derive(Debug, Default)]
pub struct RegionPhaseOutput {
    limit: usize,
    boundaries: Vec<BoundaryBatch>,
    effects: Vec<ImmediateBoundaryEffect>,
    transfers: Vec<EntityTransfer>,
}

impl RegionPhaseOutput {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            limit,
            boundaries: Vec::new(),
            effects: Vec::new(),
            transfers: Vec::new(),
        }
    }

    pub fn emit_boundary(&mut self, batch: BoundaryBatch) -> Result<(), PhaseOutputError> {
        ensure_output_capacity(self.len(), self.limit)?;
        self.boundaries.push(batch);
        Ok(())
    }

    pub fn emit_immediate(
        &mut self,
        effect: ImmediateBoundaryEffect,
    ) -> Result<(), PhaseOutputError> {
        ensure_output_capacity(self.len(), self.limit)?;
        self.effects.push(effect);
        Ok(())
    }

    pub fn emit_transfer(&mut self, transfer: EntityTransfer) -> Result<(), PhaseOutputError> {
        ensure_output_capacity(self.len(), self.limit)?;
        self.transfers.push(transfer);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.boundaries.len() + self.effects.len() + self.transfers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<BoundaryBatch>,
        Vec<ImmediateBoundaryEffect>,
        Vec<EntityTransfer>,
    ) {
        (self.boundaries, self.effects, self.transfers)
    }
}

fn ensure_output_capacity(actual: usize, limit: usize) -> Result<(), PhaseOutputError> {
    if actual == limit {
        Err(PhaseOutputError::Full { capacity: limit })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Region logic failed with semantic error {kind}")]
pub struct RegionLogicError {
    kind: ResourceId,
}

impl RegionLogicError {
    pub const fn new(kind: ResourceId) -> Self {
        Self { kind }
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PhaseOutputError {
    #[error("phase output reached its {capacity}-record bound")]
    Full { capacity: usize },
}
