//! Recovery and graceful-handoff activation fencing.

use crate::snapshot::{PersistenceRevision, RegionRecoveryPoint, SnapshotError};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionHandoffState {
    point: RegionRecoveryPoint,
    digest: [u8; 32],
    target_generation: ActivationGeneration,
}

impl RegionHandoffState {
    pub fn prepare(
        point: RegionRecoveryPoint,
        target_generation: ActivationGeneration,
    ) -> Result<Self, RecoveryError> {
        ensure_newer_generation(point.snapshot().generation(), target_generation)?;
        let digest = point.digest()?;
        Ok(Self {
            point,
            digest,
            target_generation,
        })
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        self.point.snapshot().key()
    }

    pub const fn source_generation(&self) -> ActivationGeneration {
        self.point.snapshot().generation()
    }

    pub const fn target_generation(&self) -> ActivationGeneration {
        self.target_generation
    }

    pub fn committed_tick(&self) -> u64 {
        self.point.committed_tick()
    }

    pub const fn persistence_revision(&self) -> PersistenceRevision {
        self.point.persistence_revision()
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub fn install(
        self,
        expected_key: &SimulationRegionKey,
        expected_digest: [u8; 32],
    ) -> Result<RecoveredRegion, RecoveryError> {
        if self.key() != expected_key {
            return Err(RecoveryError::WrongRegion);
        }
        if self.digest != expected_digest || self.point.digest()? != expected_digest {
            return Err(RecoveryError::DigestMismatch);
        }
        Ok(RecoveredRegion {
            point: self.point,
            generation: self.target_generation,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredRegion {
    point: RegionRecoveryPoint,
    generation: ActivationGeneration,
}

impl RecoveredRegion {
    pub const fn key(&self) -> &SimulationRegionKey {
        self.point.snapshot().key()
    }

    pub const fn generation(&self) -> ActivationGeneration {
        self.generation
    }

    pub fn committed_tick(&self) -> u64 {
        self.point.committed_tick()
    }

    pub const fn persistence_revision(&self) -> PersistenceRevision {
        self.point.persistence_revision()
    }

    pub const fn recovery_point(&self) -> &RegionRecoveryPoint {
        &self.point
    }

    pub fn into_recovery_point(self) -> RegionRecoveryPoint {
        self.point
    }
}

fn ensure_newer_generation(
    source_generation: ActivationGeneration,
    target: ActivationGeneration,
) -> Result<(), RecoveryError> {
    if target <= source_generation {
        Err(RecoveryError::GenerationNotNewer {
            source_generation,
            target,
        })
    } else {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error(
        "target activation generation {target:?} is not newer than source {source_generation:?}"
    )]
    GenerationNotNewer {
        source_generation: ActivationGeneration,
        target: ActivationGeneration,
    },
    #[error("handoff state belongs to another Region")]
    WrongRegion,
    #[error("handoff recovery-point digest does not match")]
    DigestMismatch,
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{
        RegionCommitSnapshot, RegionSnapshotHeader, SnapshotRecord, SnapshotRecordKind,
    };
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};
    use ferrite_foundation::resource::ResourceId;

    fn point() -> RegionRecoveryPoint {
        let key = SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        );
        RegionRecoveryPoint::new(
            RegionCommitSnapshot::new(
                RegionSnapshotHeader {
                    key,
                    generation: ActivationGeneration::INITIAL,
                    committed_tick: 8,
                    persistence_revision: PersistenceRevision::INITIAL,
                    region_side_chunks: 8,
                    content_manifest: [1; 32],
                    state_hash: [2; 32],
                },
                vec![
                    SnapshotRecord::new(
                        SnapshotRecordKind::Entity,
                        ResourceId::new("ferrite", "entity/v1").unwrap(),
                        vec![1],
                        vec![2],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn handoff_requires_a_newer_generation_and_matching_digest() {
        assert!(RegionHandoffState::prepare(point(), ActivationGeneration::INITIAL).is_err());
        let target = ActivationGeneration::new(2).unwrap();
        let handoff = RegionHandoffState::prepare(point(), target).unwrap();
        let digest = *handoff.digest();
        let key = handoff.key().clone();
        let recovered = handoff.install(&key, digest).unwrap();
        assert_eq!(recovered.generation(), target);
        assert_eq!(recovered.committed_tick(), 8);

        let handoff = RegionHandoffState::prepare(point(), target).unwrap();
        assert!(handoff.install(&key, [0; 32]).is_err());
    }
}
