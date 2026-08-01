//! Canonical evidence for the formal ingress-to-projection production path.

use ferrite_foundation::region::SimulationRegionKey;
use ferrite_simulation::command::CommandSource;
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::composite::gateway::CompositeGatewayTickReport;
use crate::composite::projection::{SessionProjectionError, decode_projection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionTickReplayEvidence {
    pub tick: GameTick,
    pub ingress_digest: [u8; 32],
    pub projection_digest: [u8; 32],
    pub regions: Vec<RegionReplayEvidence>,
    pub digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionReplayEvidence {
    pub region: SimulationRegionKey,
    pub replay_identity: [u8; 32],
    pub continuity_hash: [u8; 32],
    pub continuity_record_count: usize,
    pub projection_count: usize,
}

impl ProductionTickReplayEvidence {
    pub fn capture(report: &CompositeGatewayTickReport) -> Result<Self, ProductionReplayError> {
        let tick = report.local().tick();
        let ingress_digest = ingress_digest(report);
        let mut projection_hasher = blake3::Hasher::new();
        let mut regions = Vec::new();
        for (key, region) in report.regions() {
            if region.commit.tick != tick || region.continuity.tick != tick {
                return Err(ProductionReplayError::TickMismatch {
                    expected: tick,
                    commit: region.commit.tick,
                    continuity: region.continuity.tick,
                });
            }
            if region.commit.continuity_hash != region.continuity.hash
                || region.commit.continuity_record_count != region.continuity.records.len()
            {
                return Err(ProductionReplayError::ContinuityMismatch(key.clone()));
            }
            if region.commit.projection_count != region.projections.len() {
                return Err(ProductionReplayError::ProjectionCountMismatch {
                    region: key.clone(),
                    committed: region.commit.projection_count,
                    present: region.projections.len(),
                });
            }
            hash_region(&mut projection_hasher, key);
            for projection in &region.projections {
                decode_projection(projection)?;
                projection_hasher.update(&[projection.owner().stable_tag()]);
                projection_hasher.update(&projection.sequence().to_be_bytes());
                hash_bytes(
                    &mut projection_hasher,
                    projection.kind().to_string().as_bytes(),
                );
                hash_bytes(&mut projection_hasher, projection.payload());
            }
            regions.push(RegionReplayEvidence {
                region: key.clone(),
                replay_identity: region.commit.replay_identity,
                continuity_hash: region.commit.continuity_hash,
                continuity_record_count: region.commit.continuity_record_count,
                projection_count: region.commit.projection_count,
            });
        }
        let projection_digest = *projection_hasher.finalize().as_bytes();
        let mut hasher = blake3::Hasher::new();
        hasher.update(&tick.get().to_be_bytes());
        hasher.update(&ingress_digest);
        for region in &regions {
            hash_region(&mut hasher, &region.region);
            hasher.update(&region.replay_identity);
            hasher.update(&region.continuity_hash);
            hasher.update(&(region.continuity_record_count as u64).to_be_bytes());
            hasher.update(&(region.projection_count as u64).to_be_bytes());
        }
        hasher.update(&projection_digest);
        Ok(Self {
            tick,
            ingress_digest,
            projection_digest,
            regions,
            digest: *hasher.finalize().as_bytes(),
        })
    }
}

fn ingress_digest(report: &CompositeGatewayTickReport) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for command in report.local().committed_commands() {
        hash_region(&mut hasher, &command.target);
        hasher.update(&command.tick.get().to_be_bytes());
        match &command.source {
            CommandSource::System(source) => {
                hasher.update(&[0]);
                hash_bytes(&mut hasher, source.to_string().as_bytes());
            }
            CommandSource::Player(player) => {
                hasher.update(&[1]);
                hasher.update(&player.to_be_bytes());
            }
            CommandSource::Region(region) => {
                hasher.update(&[2]);
                hash_region(&mut hasher, region);
            }
        }
        hasher.update(&command.sequence.to_be_bytes());
        hash_bytes(&mut hasher, command.kind.to_string().as_bytes());
    }
    *hasher.finalize().as_bytes()
}

fn hash_region(hasher: &mut blake3::Hasher, region: &SimulationRegionKey) {
    hasher.update(&region.world().to_be_bytes());
    hash_bytes(hasher, region.dimension().resource().to_string().as_bytes());
    hasher.update(&region.coordinate().x().to_be_bytes());
    hasher.update(&region.coordinate().z().to_be_bytes());
    hasher.update(&region.mapping_version().get().to_be_bytes());
}

fn hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

#[derive(Debug, Error)]
pub enum ProductionReplayError {
    #[error(
        "production replay tick mismatch: expected {expected:?}, commit {commit:?}, continuity {continuity:?}"
    )]
    TickMismatch {
        expected: GameTick,
        commit: GameTick,
        continuity: GameTick,
    },
    #[error("production replay Region {0:?} has mismatched continuity evidence")]
    ContinuityMismatch(SimulationRegionKey),
    #[error(
        "production replay Region {region:?} committed {committed} projections but retained {present}"
    )]
    ProjectionCountMismatch {
        region: SimulationRegionKey,
        committed: usize,
        present: usize,
    },
    #[error(transparent)]
    Projection(#[from] SessionProjectionError),
}
