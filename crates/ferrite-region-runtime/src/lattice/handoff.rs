//! Ferrite recovery-point movement coordinated by Lattice authority.

use crate::lattice::authority::{
    RegionAuthorityAction, RegionAuthorityAdapter, RegionAuthorityError,
};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_persistence::recovery::{RecoveredRegion, RecoveryError, RegionHandoffState};
use ferrite_persistence::snapshot::{RegionRecoveryPoint, SnapshotError};
use thiserror::Error;

pub const MAX_HANDOFF_PAYLOAD_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatticeHandoffEnvelope {
    key: SimulationRegionKey,
    source_generation: ActivationGeneration,
    target_generation: ActivationGeneration,
    digest: [u8; 32],
    payload: Box<[u8]>,
}

impl LatticeHandoffEnvelope {
    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub const fn source_generation(&self) -> ActivationGeneration {
        self.source_generation
    }

    pub const fn target_generation(&self) -> ActivationGeneration {
        self.target_generation
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn recover(
        self,
        expected_key: &SimulationRegionKey,
    ) -> Result<RecoveredRegion, LatticeHandoffError> {
        if &self.key != expected_key {
            return Err(LatticeHandoffError::WrongRegion);
        }
        let point = RegionRecoveryPoint::decode(&self.payload)?;
        if point.snapshot().key() != &self.key
            || point.snapshot().generation() != self.source_generation
        {
            return Err(LatticeHandoffError::IdentityMismatch);
        }
        let state = RegionHandoffState::prepare(point, self.target_generation)?;
        Ok(state.install(expected_key, self.digest)?)
    }
}

pub fn prepare_handoff(
    authority: &mut RegionAuthorityAdapter,
    point: &RegionRecoveryPoint,
    target_generation: ActivationGeneration,
) -> Result<LatticeHandoffEnvelope, LatticeHandoffError> {
    if authority.generation() != Some(point.snapshot().generation()) {
        return Err(LatticeHandoffError::GenerationMismatch);
    }
    let state = RegionHandoffState::prepare(point.clone(), target_generation)?;
    let outcome = authority.begin_drain()?;
    if !outcome.contains(RegionAuthorityAction::FenceAdmission)
        || !outcome.contains(RegionAuthorityAction::DrainRegion)
    {
        return Err(LatticeHandoffError::DrainDidNotFence);
    }
    let payload = point.encode()?;
    if payload.len() > MAX_HANDOFF_PAYLOAD_BYTES {
        return Err(LatticeHandoffError::PayloadTooLarge {
            actual: payload.len(),
            maximum: MAX_HANDOFF_PAYLOAD_BYTES,
        });
    }
    Ok(LatticeHandoffEnvelope {
        key: point.snapshot().key().clone(),
        source_generation: point.snapshot().generation(),
        target_generation,
        digest: *state.digest(),
        payload: payload.into_boxed_slice(),
    })
}

#[derive(Debug, Error)]
pub enum LatticeHandoffError {
    #[error("authority generation does not match the durable recovery point")]
    GenerationMismatch,
    #[error("Lattice drain did not fence admission")]
    DrainDidNotFence,
    #[error("handoff envelope belongs to another Region")]
    WrongRegion,
    #[error("handoff envelope identity differs from its recovery point")]
    IdentityMismatch,
    #[error("handoff payload has {actual} bytes, exceeding limit {maximum}")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error(transparent)]
    Authority(#[from] RegionAuthorityError),
    #[error(transparent)]
    Recovery(#[from] RecoveryError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}
