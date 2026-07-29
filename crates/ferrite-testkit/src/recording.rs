//! Minimal deterministic target for validating the scenario harness itself.

use crate::scenario::ScenarioTarget;
use crate::seed::TestSeed;
use crate::snapshot::{MAX_SNAPSHOT_BYTES, Snapshot, SnapshotError};
use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Default)]
pub struct RecordingTarget {
    tick: u64,
    state: Vec<u8>,
}

impl ScenarioTarget for RecordingTarget {
    type Error = SnapshotError;

    fn reset(&mut self, seed: TestSeed) -> Result<(), Self::Error> {
        self.tick = 0;
        self.state = seed.get().to_le_bytes().to_vec();
        Ok(())
    }

    fn advance_to(&mut self, tick: u64) -> Result<(), Self::Error> {
        self.tick = tick;
        Ok(())
    }

    fn apply(&mut self, _kind: &ResourceId, payload: &[u8]) -> Result<(), Self::Error> {
        let next_length =
            self.state
                .len()
                .checked_add(payload.len())
                .ok_or(SnapshotError::TooLarge {
                    actual: usize::MAX,
                    maximum: MAX_SNAPSHOT_BYTES,
                })?;
        if next_length > MAX_SNAPSHOT_BYTES {
            return Err(SnapshotError::TooLarge {
                actual: next_length,
                maximum: MAX_SNAPSHOT_BYTES,
            });
        }
        self.state.extend_from_slice(payload);
        Ok(())
    }

    fn snapshot(&mut self) -> Result<Snapshot, Self::Error> {
        Snapshot::new(self.state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulated_recording_state_remains_bounded() {
        let mut target = RecordingTarget {
            tick: 0,
            state: vec![0; MAX_SNAPSHOT_BYTES],
        };
        let kind = ResourceId::new("ferrite", "test/append").unwrap();
        assert!(target.apply(&kind, &[1]).is_err());
        assert_eq!(target.state.len(), MAX_SNAPSHOT_BYTES);
    }
}
