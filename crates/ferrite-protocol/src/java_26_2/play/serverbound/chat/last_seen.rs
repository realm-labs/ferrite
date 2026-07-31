use std::collections::VecDeque;

use crate::java_26_2::play::clientbound::chat_presentation::packet::MessageSignature;
use crate::java_26_2::play::serverbound::chat::packet::LastSeenUpdate;

pub const LAST_SEEN_WINDOW: usize = 20;
pub const MAX_TRACKED_MESSAGES: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrackedMessage {
    signature: MessageSignature,
    pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenValidator {
    entries: VecDeque<Option<TrackedMessage>>,
    last_signature: Option<MessageSignature>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingResult {
    Added,
    ConsecutiveDuplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LastSeenError {
    TooManyPending { tracked: usize },
    NegativeOffset(i32),
    OffsetTooLarge { offset: i32, maximum: usize },
    BitsOutsideWindow,
    AcknowledgedMissing { slot: usize },
    ClearedAcknowledged { slot: usize },
    Checksum { expected: i8, received: i8 },
}

impl Default for LastSeenValidator {
    fn default() -> Self {
        Self {
            entries: (0..LAST_SEEN_WINDOW).map(|_| None).collect(),
            last_signature: None,
        }
    }
}

impl LastSeenValidator {
    #[must_use]
    pub fn tracked_len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn pending_len(&self) -> usize {
        self.entries
            .iter()
            .flatten()
            .filter(|entry| entry.pending)
            .count()
    }

    pub fn add_pending(
        &mut self,
        signature: MessageSignature,
    ) -> Result<PendingResult, LastSeenError> {
        if self.last_signature.as_ref() == Some(&signature) {
            return Ok(PendingResult::ConsecutiveDuplicate);
        }
        self.last_signature = Some(signature.clone());
        self.entries.push_back(Some(TrackedMessage {
            signature,
            pending: true,
        }));
        if self.entries.len() > MAX_TRACKED_MESSAGES {
            return Err(LastSeenError::TooManyPending {
                tracked: self.entries.len(),
            });
        }
        Ok(PendingResult::Added)
    }

    pub fn apply_ack(&mut self, offset: i32) -> Result<(), LastSeenError> {
        self.apply_offset(offset)
    }

    pub fn apply_update(
        &mut self,
        update: LastSeenUpdate,
    ) -> Result<Vec<MessageSignature>, LastSeenError> {
        self.apply_offset(update.offset)?;
        if update.acknowledged[2] & 0xf0 != 0 {
            return Err(LastSeenError::BitsOutsideWindow);
        }

        let mut acknowledged = Vec::new();
        for slot in 0..LAST_SEEN_WINDOW {
            let set = update.acknowledged[slot / 8] & (1 << (slot % 8)) != 0;
            let entry = &mut self.entries[slot];
            if set {
                let tracked = entry
                    .as_mut()
                    .ok_or(LastSeenError::AcknowledgedMissing { slot })?;
                tracked.pending = false;
                acknowledged.push(tracked.signature.clone());
            } else if let Some(tracked) = entry {
                if !tracked.pending {
                    return Err(LastSeenError::ClearedAcknowledged { slot });
                }
                *entry = None;
            }
        }

        let expected = checksum(&acknowledged);
        if update.checksum != 0 && update.checksum != expected {
            return Err(LastSeenError::Checksum {
                expected,
                received: update.checksum,
            });
        }
        Ok(acknowledged)
    }

    fn apply_offset(&mut self, offset: i32) -> Result<(), LastSeenError> {
        let offset = usize::try_from(offset).map_err(|_| LastSeenError::NegativeOffset(offset))?;
        let maximum = self.entries.len() - LAST_SEEN_WINDOW;
        if offset > maximum {
            return Err(LastSeenError::OffsetTooLarge {
                offset: offset as i32,
                maximum,
            });
        }
        self.entries.drain(..offset);
        Ok(())
    }
}

#[must_use]
pub fn checksum(signatures: &[MessageSignature]) -> i8 {
    let mut hash = 1_i32;
    for signature in signatures {
        hash = hash
            .wrapping_mul(31)
            .wrapping_add(java_byte_array_hash(signature));
    }
    let narrowed = hash as i8;
    if narrowed == 0 { 1 } else { narrowed }
}

fn java_byte_array_hash(signature: &MessageSignature) -> i32 {
    signature.0.iter().fold(1_i32, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(i32::from(*byte as i8))
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenTracker {
    entries: VecDeque<Option<MessageSignature>>,
    last_signature: Option<MessageSignature>,
    offset: i32,
}

impl Default for LastSeenTracker {
    fn default() -> Self {
        Self {
            entries: (0..LAST_SEEN_WINDOW).map(|_| None).collect(),
            last_signature: None,
            offset: 0,
        }
    }
}

impl LastSeenTracker {
    pub fn add_processed(&mut self, signature: MessageSignature, displayed: bool) -> PendingResult {
        if self.last_signature.as_ref() == Some(&signature) {
            return PendingResult::ConsecutiveDuplicate;
        }
        self.last_signature = Some(signature.clone());
        self.entries.pop_front();
        self.entries.push_back(displayed.then_some(signature));
        self.offset = self.offset.wrapping_add(1);
        PendingResult::Added
    }

    pub fn take_ack_if_due(&mut self) -> Option<i32> {
        if self.offset <= 64 {
            return None;
        }
        Some(std::mem::take(&mut self.offset))
    }

    pub fn generate_update(&mut self) -> LastSeenUpdate {
        let mut acknowledged = [0_u8; 3];
        let mut signatures = Vec::new();
        for (slot, signature) in self.entries.iter().enumerate() {
            if let Some(signature) = signature {
                acknowledged[slot / 8] |= 1 << (slot % 8);
                signatures.push(signature.clone());
            }
        }
        LastSeenUpdate {
            offset: std::mem::take(&mut self.offset),
            acknowledged,
            checksum: checksum(&signatures),
        }
    }
}
