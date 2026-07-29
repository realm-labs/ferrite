//! Revision tokens for asynchronous snapshot acknowledgement.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirtyRevision(u64);

impl DirtyRevision {
    pub const INITIAL: Self = Self(1);

    pub const fn get(self) -> u64 {
        self.0
    }

    fn checked_next(self) -> Result<Self, DirtyTrackerError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(DirtyTrackerError::RevisionExhausted)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyCapture {
    revision: DirtyRevision,
}

impl DirtyCapture {
    pub const fn revision(self) -> DirtyRevision {
        self.revision
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcknowledgeResult {
    Cleared,
    Stale,
    AlreadyClean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyTracker {
    revision: DirtyRevision,
    dirty: bool,
}

impl DirtyTracker {
    pub const fn new_clean() -> Self {
        Self {
            revision: DirtyRevision::INITIAL,
            dirty: false,
        }
    }

    pub const fn revision(self) -> DirtyRevision {
        self.revision
    }

    pub const fn is_dirty(self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) -> Result<DirtyRevision, DirtyTrackerError> {
        self.revision = self.revision.checked_next()?;
        self.dirty = true;
        Ok(self.revision)
    }

    pub const fn capture(self) -> Option<DirtyCapture> {
        if self.dirty {
            Some(DirtyCapture {
                revision: self.revision,
            })
        } else {
            None
        }
    }

    pub fn acknowledge(&mut self, capture: DirtyCapture) -> AcknowledgeResult {
        if !self.dirty {
            AcknowledgeResult::AlreadyClean
        } else if capture.revision != self.revision {
            AcknowledgeResult::Stale
        } else {
            self.dirty = false;
            AcknowledgeResult::Cleared
        }
    }
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new_clean()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DirtyTrackerError {
    #[error("dirty revision is exhausted")]
    RevisionExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_unchanged_capture_clears_dirty_state() {
        let mut tracker = DirtyTracker::new_clean();
        tracker.mark_dirty().unwrap();
        let stale = tracker.capture().unwrap();
        tracker.mark_dirty().unwrap();
        assert_eq!(tracker.acknowledge(stale), AcknowledgeResult::Stale);
        assert!(tracker.is_dirty());
        let current = tracker.capture().unwrap();
        assert_eq!(tracker.acknowledge(current), AcknowledgeResult::Cleared);
        assert!(!tracker.is_dirty());
        assert_eq!(
            tracker.acknowledge(current),
            AcknowledgeResult::AlreadyClean
        );
    }
}
