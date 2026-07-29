//! Bounded byte snapshots with actionable deterministic comparisons.

use std::fmt::{self, Display, Formatter};
use thiserror::Error;

pub const MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    bytes: Vec<u8>,
    digest: SnapshotDigest,
}

impl Snapshot {
    pub fn new(bytes: Vec<u8>) -> Result<Self, SnapshotError> {
        if bytes.len() > MAX_SNAPSHOT_BYTES {
            return Err(SnapshotError::TooLarge {
                actual: bytes.len(),
                maximum: MAX_SNAPSHOT_BYTES,
            });
        }
        let digest = SnapshotDigest(*blake3::hash(&bytes).as_bytes());
        Ok(Self { bytes, digest })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn digest(&self) -> SnapshotDigest {
        self.digest
    }

    pub fn compare(&self, actual: &Self) -> Result<(), SnapshotMismatch> {
        if self == actual {
            return Ok(());
        }
        let first_difference = self
            .bytes
            .iter()
            .zip(&actual.bytes)
            .position(|(expected, actual)| expected != actual)
            .unwrap_or_else(|| self.bytes.len().min(actual.bytes.len()));
        Err(SnapshotMismatch {
            first_difference,
            expected_length: self.bytes.len(),
            actual_length: actual.bytes.len(),
            expected_digest: self.digest,
            actual_digest: actual.digest,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnapshotDigest([u8; 32]);

impl Display for SnapshotDigest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SnapshotError {
    #[error("snapshot has {actual} bytes, exceeding the {maximum}-byte limit")]
    TooLarge { actual: usize, maximum: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "snapshot diverged at byte {first_difference}: expected {expected_length} bytes \
     ({expected_digest}), got {actual_length} bytes ({actual_digest})"
)]
pub struct SnapshotMismatch {
    pub first_difference: usize,
    pub expected_length: usize,
    pub actual_length: usize,
    pub expected_digest: SnapshotDigest,
    pub actual_digest: SnapshotDigest,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatch_reports_first_byte_lengths_and_digests() {
        let expected = Snapshot::new(vec![1, 2, 3]).unwrap();
        let actual = Snapshot::new(vec![1, 4]).unwrap();
        let mismatch = expected.compare(&actual).unwrap_err();
        assert_eq!(mismatch.first_difference, 1);
        assert_eq!(mismatch.expected_length, 3);
        assert_eq!(mismatch.actual_length, 2);
        assert_ne!(mismatch.expected_digest, mismatch.actual_digest);
    }
}
