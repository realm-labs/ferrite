use ferrite_world::id::BlockStateId;
use thiserror::Error;

use crate::player::state::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientPredictionClock {
    current_sequence: i32,
    last_teleport_sequence: i32,
    predicting: bool,
}

impl Default for ClientPredictionClock {
    fn default() -> Self {
        Self {
            current_sequence: 0,
            last_teleport_sequence: -1,
            predicting: false,
        }
    }
}

impl ClientPredictionClock {
    pub fn begin(&mut self) -> i32 {
        self.current_sequence = self.current_sequence.wrapping_add(1);
        self.predicting = true;
        self.current_sequence
    }

    pub fn end(&mut self) {
        self.predicting = false;
    }

    pub const fn on_teleport(&mut self) {
        self.last_teleport_sequence = self.current_sequence;
    }

    #[must_use]
    pub const fn current_sequence(&self) -> i32 {
        self.current_sequence
    }

    #[must_use]
    pub const fn is_predicting(&self) -> bool {
        self.predicting
    }

    #[must_use]
    pub const fn supplies_captured_position(&self, acknowledgement: i32) -> bool {
        self.last_teleport_sequence < acknowledgement
    }

    #[cfg(test)]
    pub(crate) const fn with_sequence(sequence: i32) -> Self {
        Self {
            current_sequence: sequence,
            last_teleport_sequence: -1,
            predicting: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerPredictionAck {
    highest_received: i32,
}

impl Default for ServerPredictionAck {
    fn default() -> Self {
        Self {
            highest_received: -1,
        }
    }
}

impl ServerPredictionAck {
    pub fn register(&mut self, sequence: i32) -> Result<(), PredictionError> {
        if sequence < 0 {
            return Err(PredictionError::NegativeSequence(sequence));
        }
        self.highest_received = self.highest_received.max(sequence);
        Ok(())
    }

    #[must_use]
    pub const fn acknowledgement(self) -> i32 {
        self.highest_received
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetainedPrediction {
    pub sequence: i32,
    pub authoritative_state: BlockStateId,
    pub captured_player_position: Vec3,
}

impl RetainedPrediction {
    pub const fn retain_again(&mut self, sequence: i32) {
        self.sequence = sequence;
    }

    pub const fn stage_authoritative(&mut self, state: BlockStateId) {
        self.authoritative_state = state;
    }

    #[must_use]
    pub fn resolve(
        self,
        acknowledgement: i32,
        local_state: BlockStateId,
        prediction_clock: ClientPredictionClock,
        restored_state_collides: bool,
    ) -> PredictionResolution {
        if self.sequence > acknowledgement {
            return PredictionResolution::Pending;
        }
        if local_state == self.authoritative_state {
            return PredictionResolution::RemoveUnchanged;
        }
        PredictionResolution::Restore {
            state: self.authoritative_state,
            flags: 19,
            snap_to: (prediction_clock.supplies_captured_position(acknowledgement)
                && restored_state_collides)
                .then_some(self.captured_player_position),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PredictionResolution {
    Pending,
    RemoveUnchanged,
    Restore {
        state: BlockStateId,
        flags: u32,
        snap_to: Option<Vec3>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PredictionError {
    #[error("prediction sequence {0} is negative")]
    NegativeSequence(i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_wrap_and_teleport_gate_are_source_ordered() {
        let mut clock = ClientPredictionClock::with_sequence(i32::MAX);
        assert_eq!(clock.begin(), i32::MIN);
        assert!(clock.is_predicting());
        clock.end();
        assert!(!clock.is_predicting());

        let mut ordinary = ClientPredictionClock::default();
        assert_eq!(ordinary.begin(), 1);
        ordinary.on_teleport();
        assert!(!ordinary.supplies_captured_position(1));
        assert!(ordinary.supplies_captured_position(2));
    }
}
