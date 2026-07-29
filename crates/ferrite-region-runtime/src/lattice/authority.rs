//! Lattice claim and lease fencing behind Ferrite authority types.

use crate::lattice::spatial::{LatticeSlotDescriptor, SpatialAdapterError};
use ferrite_foundation::identity::ActivationGeneration;
use lattice_core::actor_ref::{NodeAddress, NodeIncarnation, ReferenceError};
use lattice_placement::authority::{
    AuthorityEffect, AuthorityError, AuthorityEvent, PlacementAuthority,
};
use lattice_placement::types::{
    AssignmentGeneration, ClaimGrant, CoordinatorTerm, GrantSequence, MonotonicTime, NodeKey,
    PlacementSlot, PlacementSlotKey, PlacementSlotState, PlacementTypeError, PlacementVersion,
    Revision,
};
use std::collections::BTreeSet;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatticeNodeIdentity {
    node_id: String,
    host: String,
    port: u16,
    incarnation: u128,
}

impl LatticeNodeIdentity {
    pub fn generate(
        node_id: impl Into<String>,
        host: impl Into<String>,
        port: u16,
    ) -> Result<Self, RegionAuthorityError> {
        Self::new(node_id, host, port, NodeIncarnation::generate().get())
    }

    pub fn new(
        node_id: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        incarnation: u128,
    ) -> Result<Self, RegionAuthorityError> {
        let identity = Self {
            node_id: node_id.into(),
            host: host.into(),
            port,
            incarnation,
        };
        identity.to_lattice()?;
        Ok(identity)
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub const fn incarnation(&self) -> u128 {
        self.incarnation
    }

    fn to_lattice(&self) -> Result<NodeKey, RegionAuthorityError> {
        let key = NodeKey {
            node_id: self.node_id.clone(),
            address: NodeAddress::new(self.host.clone(), self.port)?,
            incarnation: NodeIncarnation::new(self.incarnation)?,
        };
        key.validate()?;
        Ok(key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionPlacementState {
    Unallocated,
    Allocating,
    Running,
    BeginHandoff,
    Stopping,
    StopFailed,
    Fenced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionPlacementObservation {
    pub generation: ActivationGeneration,
    pub coordinator_term: u64,
    pub revision: u64,
    pub state: RegionPlacementState,
    pub target: Option<LatticeNodeIdentity>,
    pub move_id: Option<u128>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionClaimGrant {
    pub generation: ActivationGeneration,
    pub coordinator_term: u64,
    pub grant_sequence: u64,
    pub ttl_millis: u64,
}

pub struct RegionAuthorityAdapter {
    local: NodeKey,
    slot: LatticeSlotDescriptor,
    authority: PlacementAuthority,
    generation: Option<ActivationGeneration>,
}

impl RegionAuthorityAdapter {
    pub(crate) fn new(
        local: LatticeNodeIdentity,
        slot: LatticeSlotDescriptor,
        safety_margin_millis: u64,
    ) -> Result<Self, RegionAuthorityError> {
        let local = local.to_lattice()?;
        let authority =
            PlacementAuthority::new(local.clone(), Duration::from_millis(safety_margin_millis))?;
        Ok(Self {
            local,
            slot,
            authority,
            generation: None,
        })
    }

    pub fn reconcile(
        &mut self,
        observation: RegionPlacementObservation,
    ) -> Result<RegionAuthorityOutcome, RegionAuthorityError> {
        let generation = AssignmentGeneration::new(observation.generation.get())?;
        let state = map_state(observation.state);
        let target = observation
            .target
            .as_ref()
            .map(LatticeNodeIdentity::to_lattice)
            .transpose()?;
        let slot = PlacementSlot {
            key: self.slot_key(),
            config_fingerprint: self.slot.fingerprint,
            owner: if state == PlacementSlotState::Unallocated {
                None
            } else {
                Some(self.local.clone())
            },
            target,
            assignment_generation: generation,
            version: PlacementVersion::new(
                self.slot.domain.clone(),
                CoordinatorTerm::new(observation.coordinator_term)?,
                Revision::new(observation.revision)?,
            ),
            state,
            active_move: observation.move_id,
            barrier_sessions: BTreeSet::new(),
        };
        let effects = self
            .authority
            .transition(AuthorityEvent::ReconcileSlot(slot))?;
        self.generation = Some(observation.generation);
        Ok(RegionAuthorityOutcome::from_lattice(effects))
    }

    pub fn install_claim(
        &mut self,
        grant: RegionClaimGrant,
        now_millis: u64,
    ) -> Result<RegionAuthorityOutcome, RegionAuthorityError> {
        if self.generation != Some(grant.generation) {
            return Err(RegionAuthorityError::GenerationMismatch);
        }
        let claim = ClaimGrant {
            domain: self.slot.domain.clone(),
            slot: self.slot_key(),
            owner: self.local.clone(),
            coordinator_term: CoordinatorTerm::new(grant.coordinator_term)?,
            assignment_generation: AssignmentGeneration::new(grant.generation.get())?,
            grant_sequence: GrantSequence::new(grant.grant_sequence)?,
            ttl: Duration::from_millis(grant.ttl_millis),
        };
        let effects = self.authority.transition(AuthorityEvent::InstallGrant {
            grant: claim,
            now: MonotonicTime::from_millis(now_millis),
        })?;
        Ok(RegionAuthorityOutcome::from_lattice(effects))
    }

    pub fn tick(
        &mut self,
        now_millis: u64,
    ) -> Result<RegionAuthorityOutcome, RegionAuthorityError> {
        let effects = self.authority.transition(AuthorityEvent::Tick {
            now: MonotonicTime::from_millis(now_millis),
        })?;
        Ok(RegionAuthorityOutcome::from_lattice(effects))
    }

    pub fn begin_drain(&mut self) -> Result<RegionAuthorityOutcome, RegionAuthorityError> {
        let effects = self.authority.transition(AuthorityEvent::BeginDrain)?;
        Ok(RegionAuthorityOutcome::from_lattice(effects))
    }

    pub fn claim_lost(&mut self) -> Result<RegionAuthorityOutcome, RegionAuthorityError> {
        let effects = self
            .authority
            .transition(AuthorityEvent::ExternalClaimLost)?;
        Ok(RegionAuthorityOutcome::from_lattice(effects))
    }

    pub fn admission_open(&self, generation: ActivationGeneration, now_millis: u64) -> bool {
        self.generation == Some(generation)
            && self
                .authority
                .admission_open_at(MonotonicTime::from_millis(now_millis))
    }

    pub const fn generation(&self) -> Option<ActivationGeneration> {
        self.generation
    }

    fn slot_key(&self) -> PlacementSlotKey {
        PlacementSlotKey::Shard {
            domain: self.slot.domain.clone(),
            entity_type: self.slot.entity_type.clone(),
            shard_id: self.slot.shard,
        }
    }
}

fn map_state(state: RegionPlacementState) -> PlacementSlotState {
    match state {
        RegionPlacementState::Unallocated => PlacementSlotState::Unallocated,
        RegionPlacementState::Allocating => PlacementSlotState::Allocating,
        RegionPlacementState::Running => PlacementSlotState::Running,
        RegionPlacementState::BeginHandoff => PlacementSlotState::BeginHandoff,
        RegionPlacementState::Stopping => PlacementSlotState::Stopping,
        RegionPlacementState::StopFailed => PlacementSlotState::StopFailed,
        RegionPlacementState::Fenced => PlacementSlotState::Fenced,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegionAuthorityAction {
    FenceAdmission,
    OpenAdmission,
    StartRegion,
    DrainRegion,
    StopRegion,
    PublishReady,
    PublishDrained,
    PublishStopFailed,
    StateLossPossible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionAuthorityOutcome {
    actions: Box<[RegionAuthorityAction]>,
}

impl RegionAuthorityOutcome {
    pub fn actions(&self) -> &[RegionAuthorityAction] {
        &self.actions
    }

    pub fn contains(&self, action: RegionAuthorityAction) -> bool {
        self.actions.contains(&action)
    }

    fn from_lattice(effects: Vec<AuthorityEffect>) -> Self {
        let mut actions = effects
            .into_iter()
            .map(|effect| match effect {
                AuthorityEffect::FenceAdmission => RegionAuthorityAction::FenceAdmission,
                AuthorityEffect::OpenAdmission => RegionAuthorityAction::OpenAdmission,
                AuthorityEffect::StartSlot => RegionAuthorityAction::StartRegion,
                AuthorityEffect::DrainSlot => RegionAuthorityAction::DrainRegion,
                AuthorityEffect::StopSlot => RegionAuthorityAction::StopRegion,
                AuthorityEffect::PublishReady => RegionAuthorityAction::PublishReady,
                AuthorityEffect::PublishDrained => RegionAuthorityAction::PublishDrained,
                AuthorityEffect::PublishStopFailed => RegionAuthorityAction::PublishStopFailed,
                AuthorityEffect::StateLossPossible => RegionAuthorityAction::StateLossPossible,
            })
            .collect::<Vec<_>>();
        actions.sort();
        actions.dedup();
        Self {
            actions: actions.into_boxed_slice(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RegionAuthorityError {
    #[error("claim generation does not match the reconciled Region generation")]
    GenerationMismatch,
    #[error(transparent)]
    Reference(#[from] ReferenceError),
    #[error(transparent)]
    Placement(#[from] PlacementTypeError),
    #[error(transparent)]
    Authority(#[from] AuthorityError),
    #[error(transparent)]
    Spatial(#[from] SpatialAdapterError),
}
