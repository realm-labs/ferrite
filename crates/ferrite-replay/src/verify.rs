//! Replay execution verification with first-divergence diagnostics.

use crate::codec::{CanonicalEncode, Encoder};
use crate::envelope::{CommandEnvelope, EventEnvelope, TickNumber};
use crate::hash::{RegionHashRecord, StateHash};
use crate::log::{ReplayFrame, ReplayHeader, ReplayLog, ReplayLogError};
use ferrite_foundation::region::SimulationRegionKey;
use std::fmt::Display;

pub trait ReplayTarget {
    type Error: Display;

    fn begin(&mut self, header: &ReplayHeader) -> Result<(), Self::Error>;

    fn execute(
        &mut self,
        tick: TickNumber,
        commands: &[CommandEnvelope],
    ) -> Result<ObservedFrame, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedFrame {
    frame: ReplayFrame,
}

impl ObservedFrame {
    pub fn new(
        tick: TickNumber,
        events: Vec<EventEnvelope>,
        region_hashes: Vec<RegionHashRecord>,
        world_hash: StateHash,
    ) -> Result<Self, ReplayLogError> {
        Ok(Self {
            frame: ReplayFrame::new(tick, Vec::new(), events, region_hashes, world_hash)?,
        })
    }

    pub const fn tick(&self) -> TickNumber {
        self.frame.tick()
    }

    pub fn events(&self) -> &[EventEnvelope] {
        self.frame.event_slice()
    }

    pub fn region_hashes(&self) -> &[RegionHashRecord] {
        self.frame.region_hash_slice()
    }

    pub const fn world_hash(&self) -> StateHash {
        self.frame.world_hash()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationReport {
    Converged {
        frames: usize,
        final_world_hash: Option<StateHash>,
    },
    Diverged(DivergenceReport),
}

impl VerificationReport {
    pub const fn is_converged(&self) -> bool {
        matches!(self, Self::Converged { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivergenceReport {
    frame_index: usize,
    tick: Option<TickNumber>,
    kind: DivergenceKind,
}

impl DivergenceReport {
    pub const fn frame_index(&self) -> usize {
        self.frame_index
    }

    pub const fn tick(&self) -> Option<TickNumber> {
        self.tick
    }

    pub const fn kind(&self) -> &DivergenceKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivergenceKind {
    BeginFailed {
        message: String,
    },
    ExecutionFailed {
        message: String,
    },
    WrongTick {
        expected: TickNumber,
        actual: TickNumber,
    },
    EventCount {
        expected: usize,
        actual: usize,
    },
    Event {
        index: usize,
        expected: StateHash,
        actual: StateHash,
    },
    RegionCount {
        expected: usize,
        actual: usize,
    },
    RegionIdentity {
        index: usize,
        expected: SimulationRegionKey,
        actual: SimulationRegionKey,
    },
    RegionHash {
        region: SimulationRegionKey,
        expected: StateHash,
        actual: StateHash,
    },
    WorldHash {
        expected: StateHash,
        actual: StateHash,
    },
}

pub fn verify_replay<T: ReplayTarget>(log: &ReplayLog, target: &mut T) -> VerificationReport {
    if let Err(error) = target.begin(log.header()) {
        return diverged(
            0,
            None,
            DivergenceKind::BeginFailed {
                message: error.to_string(),
            },
        );
    }

    let mut final_world_hash = None;
    for (frame_index, expected) in log.frames().enumerate() {
        let observed = match target.execute(expected.tick(), expected.command_slice()) {
            Ok(observed) => observed,
            Err(error) => {
                return diverged(
                    frame_index,
                    Some(expected.tick()),
                    DivergenceKind::ExecutionFailed {
                        message: error.to_string(),
                    },
                );
            }
        };
        if observed.tick() != expected.tick() {
            return diverged(
                frame_index,
                Some(expected.tick()),
                DivergenceKind::WrongTick {
                    expected: expected.tick(),
                    actual: observed.tick(),
                },
            );
        }
        if let Some(kind) = compare_events(expected.event_slice(), observed.events()) {
            return diverged(frame_index, Some(expected.tick()), kind);
        }
        if let Some(kind) = compare_regions(expected.region_hash_slice(), observed.region_hashes())
        {
            return diverged(frame_index, Some(expected.tick()), kind);
        }
        if expected.world_hash() != observed.world_hash() {
            return diverged(
                frame_index,
                Some(expected.tick()),
                DivergenceKind::WorldHash {
                    expected: expected.world_hash(),
                    actual: observed.world_hash(),
                },
            );
        }
        final_world_hash = Some(observed.world_hash());
    }
    VerificationReport::Converged {
        frames: log.frames().len(),
        final_world_hash,
    }
}

fn compare_events(expected: &[EventEnvelope], actual: &[EventEnvelope]) -> Option<DivergenceKind> {
    if expected.len() != actual.len() {
        return Some(DivergenceKind::EventCount {
            expected: expected.len(),
            actual: actual.len(),
        });
    }
    for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        if expected != actual {
            return Some(DivergenceKind::Event {
                index,
                expected: diagnostic_hash(expected),
                actual: diagnostic_hash(actual),
            });
        }
    }
    None
}

fn compare_regions(
    expected: &[RegionHashRecord],
    actual: &[RegionHashRecord],
) -> Option<DivergenceKind> {
    if expected.len() != actual.len() {
        return Some(DivergenceKind::RegionCount {
            expected: expected.len(),
            actual: actual.len(),
        });
    }
    for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        if expected.region() != actual.region() {
            return Some(DivergenceKind::RegionIdentity {
                index,
                expected: expected.region().clone(),
                actual: actual.region().clone(),
            });
        }
        if expected.hash() != actual.hash() {
            return Some(DivergenceKind::RegionHash {
                region: expected.region().clone(),
                expected: expected.hash(),
                actual: actual.hash(),
            });
        }
    }
    None
}

fn diagnostic_hash<T: CanonicalEncode>(value: &T) -> StateHash {
    let mut encoder = Encoder::new();
    value
        .encode(&mut encoder)
        .expect("validated replay records must remain canonically encodable");
    StateHash::from_bytes(*blake3::hash(encoder.as_slice()).as_bytes())
}

fn diverged(
    frame_index: usize,
    tick: Option<TickNumber>,
    kind: DivergenceKind,
) -> VerificationReport {
    VerificationReport::Diverged(DivergenceReport {
        frame_index,
        tick,
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::{EnvelopePayload, SequenceNumber};
    use crate::log::ReplayHeader;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
    use ferrite_foundation::resource::ResourceId;
    use ferrite_simulation::random::RandomAlgorithm;
    use std::convert::Infallible;

    #[derive(Clone)]
    struct FixedTarget {
        observed: ObservedFrame,
    }

    impl ReplayTarget for FixedTarget {
        type Error = Infallible;

        fn begin(&mut self, _header: &ReplayHeader) -> Result<(), Self::Error> {
            Ok(())
        }

        fn execute(
            &mut self,
            _tick: TickNumber,
            _commands: &[CommandEnvelope],
        ) -> Result<ObservedFrame, Self::Error> {
            Ok(self.observed.clone())
        }
    }

    fn fixture(event_payload: u8) -> (ReplayLog, ObservedFrame) {
        let world = WorldId::new(1).unwrap();
        let region = SimulationRegionKey::new(
            world,
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        );
        let event = EventEnvelope::new(
            TickNumber::new(1),
            SequenceNumber::new(1),
            region.clone(),
            ResourceId::new("ferrite", "event/test").unwrap(),
            EnvelopePayload::new(vec![event_payload]).unwrap(),
        );
        let region_hash = RegionHashRecord::new(region, StateHash::from_bytes([2; 32]));
        let world_hash = StateHash::from_bytes([3; 32]);
        let frame = ReplayFrame::new(
            TickNumber::new(1),
            Vec::new(),
            vec![event.clone()],
            vec![region_hash.clone()],
            world_hash,
        )
        .unwrap();
        let log = ReplayLog::new(
            ReplayHeader::new(
                ResourceId::new("ferrite", "test").unwrap(),
                world,
                StateHash::from_bytes([1; 32]),
                RegionMappingVersion::V1,
                RandomAlgorithm::Xoshiro256StarStarV1,
                TickNumber::new(0),
            ),
            vec![frame],
        )
        .unwrap();
        let observed = ObservedFrame::new(
            TickNumber::new(1),
            vec![event],
            vec![region_hash],
            world_hash,
        )
        .unwrap();
        (log, observed)
    }

    #[test]
    fn matching_execution_converges() {
        let (log, observed) = fixture(7);
        let report = verify_replay(&log, &mut FixedTarget { observed });
        assert!(report.is_converged());
    }

    #[test]
    fn first_event_divergence_reports_frame_and_digests() {
        let (log, _) = fixture(7);
        let (_, observed) = fixture(8);
        let report = verify_replay(&log, &mut FixedTarget { observed });
        let VerificationReport::Diverged(report) = report else {
            panic!("expected divergence");
        };
        assert_eq!(report.frame_index(), 0);
        assert!(matches!(
            report.kind(),
            DivergenceKind::Event { index: 0, .. }
        ));
    }
}
