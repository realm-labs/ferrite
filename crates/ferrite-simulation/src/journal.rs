//! Bounded per-tick semantic journals.

use crate::tick::{GameTick, TickPhase};
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

pub const MAX_JOURNAL_PAYLOAD_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JournalDomain {
    Command,
    Mutation,
    Event,
    Replication,
    Effect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    phase: TickPhase,
    sequence: u64,
    domain: JournalDomain,
    kind: ResourceId,
    payload: Vec<u8>,
}

impl JournalEntry {
    pub fn new(
        phase: TickPhase,
        sequence: u64,
        domain: JournalDomain,
        kind: ResourceId,
        payload: Vec<u8>,
    ) -> Result<Self, JournalError> {
        if payload.len() > MAX_JOURNAL_PAYLOAD_BYTES {
            return Err(JournalError::PayloadTooLarge {
                actual: payload.len(),
                maximum: MAX_JOURNAL_PAYLOAD_BYTES,
            });
        }
        Ok(Self {
            phase,
            sequence,
            domain,
            kind,
            payload,
        })
    }

    pub const fn phase(&self) -> TickPhase {
        self.phase
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn domain(&self) -> JournalDomain {
        self.domain
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug)]
pub struct ActiveTickJournal {
    tick: GameTick,
    capacity: usize,
    entries: Vec<JournalEntry>,
    next_sequence: u64,
}

impl ActiveTickJournal {
    pub fn new(tick: GameTick, capacity: usize) -> Result<Self, JournalError> {
        if capacity == 0 {
            return Err(JournalError::ZeroCapacity);
        }
        Ok(Self {
            tick,
            capacity,
            entries: Vec::new(),
            next_sequence: 0,
        })
    }

    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn append(
        &mut self,
        phase: TickPhase,
        domain: JournalDomain,
        kind: ResourceId,
        payload: Vec<u8>,
    ) -> Result<u64, JournalError> {
        if self.entries.len() == self.capacity {
            return Err(JournalError::Full {
                capacity: self.capacity,
            });
        }
        let sequence = self.next_sequence;
        let entry = JournalEntry::new(phase, sequence, domain, kind, payload)?;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(JournalError::SequenceExhausted)?;
        self.entries.push(entry);
        Ok(sequence)
    }

    pub fn commit(self) -> CommittedTickJournal {
        CommittedTickJournal {
            tick: self.tick,
            entries: self.entries.into_boxed_slice(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedTickJournal {
    tick: GameTick,
    entries: Box<[JournalEntry]>,
}

impl CommittedTickJournal {
    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum JournalError {
    #[error("journal capacity cannot be zero")]
    ZeroCapacity,
    #[error("journal payload has {actual} bytes, exceeding the {maximum}-byte limit")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("journal reached its {capacity}-entry bound")]
    Full { capacity: usize },
    #[error("journal sequence is exhausted")]
    SequenceExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_sequences_are_explicit_and_capacity_is_fail_closed() {
        let mut journal = ActiveTickJournal::new(GameTick::new(1), 1).unwrap();
        assert_eq!(
            journal
                .append(
                    TickPhase::PlayerIntent,
                    JournalDomain::Command,
                    ResourceId::new("ferrite", "command/test").unwrap(),
                    vec![1],
                )
                .unwrap(),
            0
        );
        assert!(
            journal
                .append(
                    TickPhase::PlayerIntent,
                    JournalDomain::Command,
                    ResourceId::new("ferrite", "command/test").unwrap(),
                    vec![],
                )
                .is_err()
        );
        let committed = journal.commit();
        assert_eq!(committed.tick(), GameTick::new(1));
        assert_eq!(committed.entries().len(), 1);
    }
}
