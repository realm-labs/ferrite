//! Canonical replay acceptance over the local and Lattice-backed playable scenario.

use crate::conformance::playable::{
    PlayableScenarioError, PlayableTopology, run_playable_scenario,
};
use ferrite_foundation::identity::{DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_replay::codec::{decode_exact, encode_to_vec};
use ferrite_replay::envelope::{
    CommandEnvelope, CommandSource, EnvelopePayload, EventEnvelope, SequenceNumber, TickNumber,
};
use ferrite_replay::hash::StateHash;
use ferrite_replay::log::{ReplayFrame, ReplayHeader, ReplayLog};
use ferrite_replay::verify::{ObservedFrame, ReplayTarget, VerificationReport, verify_replay};
use ferrite_simulation::random::RandomAlgorithm;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const REPLAY_TICK: TickNumber = TickNumber::new(7);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayableReplayEvidence {
    pub frames: usize,
    pub encoded_bytes: usize,
    pub log_digest: String,
    pub final_world_hash: String,
}

pub fn verify_playable_replay(
    topology: PlayableTopology,
) -> Result<PlayableReplayEvidence, PlayableReplayError> {
    let (log, encoded, digest) = build_log()?;
    let mut target = PlayableReplayTarget {
        topology,
        executed: false,
    };
    match verify_replay(&log, &mut target) {
        VerificationReport::Converged {
            frames,
            final_world_hash: Some(final_world_hash),
        } => Ok(PlayableReplayEvidence {
            frames,
            encoded_bytes: encoded.len(),
            log_digest: digest,
            final_world_hash: final_world_hash.to_string(),
        }),
        report => Err(PlayableReplayError::Verification(format!("{report:?}"))),
    }
}

fn build_log() -> Result<(ReplayLog, Vec<u8>, String), PlayableReplayError> {
    let expected = run_playable_scenario(PlayableTopology::Local)?;
    let header = replay_header()?;
    let command = replay_command()?;
    let observed = observed_frame(&expected)?;
    let frame = ReplayFrame::new(
        REPLAY_TICK,
        vec![command],
        observed.events().to_vec(),
        observed.region_hashes().to_vec(),
        observed.world_hash(),
    )
    .map_err(construction)?;
    let log = ReplayLog::new(header, vec![frame]).map_err(construction)?;
    let encoded = encode_to_vec(&log).map_err(construction)?;
    let decoded = decode_exact::<ReplayLog>(&encoded).map_err(construction)?;
    if decoded != log {
        return Err(PlayableReplayError::Construction(
            "canonical replay log did not round trip".to_owned(),
        ));
    }
    let digest = blake3::hash(&encoded).to_hex().to_string();
    Ok((log, encoded, digest))
}

struct PlayableReplayTarget {
    topology: PlayableTopology,
    executed: bool,
}

impl ReplayTarget for PlayableReplayTarget {
    type Error = PlayableReplayError;

    fn begin(&mut self, header: &ReplayHeader) -> Result<(), Self::Error> {
        if header != &replay_header()? {
            return Err(PlayableReplayError::Verification(
                "playable replay header drifted".to_owned(),
            ));
        }
        self.executed = false;
        Ok(())
    }

    fn execute(
        &mut self,
        tick: TickNumber,
        commands: &[CommandEnvelope],
    ) -> Result<ObservedFrame, Self::Error> {
        if self.executed || tick != REPLAY_TICK || commands != [replay_command()?] {
            return Err(PlayableReplayError::Verification(
                "playable replay command stream drifted".to_owned(),
            ));
        }
        self.executed = true;
        observed_frame(&run_playable_scenario(self.topology)?)
    }
}

fn replay_header() -> Result<ReplayHeader, PlayableReplayError> {
    Ok(ReplayHeader::new(
        ResourceId::new("ferrite", "conformance/playable-replay-v1").map_err(construction)?,
        WorldId::new(1).map_err(construction)?,
        StateHash::from_bytes([0; 32]),
        RegionMappingVersion::V1,
        RandomAlgorithm::Xoshiro256StarStarV1,
        TickNumber::new(0),
    ))
}

fn replay_command() -> Result<CommandEnvelope, PlayableReplayError> {
    Ok(CommandEnvelope::new(
        REPLAY_TICK,
        SequenceNumber::new(1),
        CommandSource::System,
        region(),
        ResourceId::new("ferrite", "conformance/run-playable-scenario").map_err(construction)?,
        EnvelopePayload::new(Vec::new()).map_err(construction)?,
    ))
}

fn observed_frame(
    evidence: &crate::conformance::playable::PlayableScenarioEvidence,
) -> Result<ObservedFrame, PlayableReplayError> {
    let world_hash = parse_hash(&evidence.committed_hash)?;
    let event = EventEnvelope::new(
        REPLAY_TICK,
        SequenceNumber::new(1),
        region(),
        ResourceId::new("ferrite", "conformance/playable-evidence").map_err(construction)?,
        EnvelopePayload::new(serde_json::to_vec(evidence).map_err(construction)?)
            .map_err(construction)?,
    );
    ObservedFrame::new(REPLAY_TICK, vec![event], Vec::new(), world_hash).map_err(construction)
}

fn region() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).expect("locked replay world is valid"),
        DimensionId::new(
            ResourceId::minecraft("overworld").expect("locked replay dimension is valid"),
        ),
        RegionCoord::new(1, 0),
        RegionMappingVersion::V1,
    )
}

fn parse_hash(value: &str) -> Result<StateHash, PlayableReplayError> {
    if value.len() != 64 {
        return Err(PlayableReplayError::Construction(
            "playable state hash length drifted".to_owned(),
        ));
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(pair).map_err(construction)?;
        bytes[index] = u8::from_str_radix(pair, 16).map_err(construction)?;
    }
    Ok(StateHash::from_bytes(bytes))
}

fn construction(error: impl std::fmt::Display) -> PlayableReplayError {
    PlayableReplayError::Construction(error.to_string())
}

#[derive(Debug, Error)]
pub enum PlayableReplayError {
    #[error(transparent)]
    Scenario(#[from] PlayableScenarioError),
    #[error("playable replay construction failed: {0}")]
    Construction(String),
    #[error("playable replay verification failed: {0}")]
    Verification(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_and_lattice_replay_targets_converge_on_the_same_canonical_log() {
        let local = verify_playable_replay(PlayableTopology::Local).unwrap();
        let lattice = verify_playable_replay(PlayableTopology::LatticeInProcess).unwrap();
        assert_eq!(local, lattice);
        assert_eq!(local.frames, 1);
        assert_eq!(local.log_digest.len(), 64);
    }
}
