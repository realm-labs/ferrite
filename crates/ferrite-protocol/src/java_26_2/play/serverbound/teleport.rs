use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::serverbound::packet::{
    AcceptTeleportation, PlayServerboundEntryPacket,
};

const RESEND_AFTER_TICKS: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeleportChallenge {
    pub challenge: i32,
    pub authoritative_position: Vector3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PendingTeleport {
    authoritative_position: Vector3,
    sent_at_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TeleportAcknowledgement {
    IgnoredStale {
        received: i32,
        current: i32,
    },
    Accepted {
        authoritative_position: Vector3,
        completed_dimension_change: bool,
    },
    DisconnectInvalidMovement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementDisposition {
    SuppressWhileTeleportPending,
    Validate,
}

/// Connection-local challenge state. No field in this type is authoritative world state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeleportSynchronizer {
    current_challenge: i32,
    pending: Option<PendingTeleport>,
    last_good_position: Vector3,
    dimension_change_pending: bool,
}

impl TeleportSynchronizer {
    #[must_use]
    pub const fn new(last_good_position: Vector3) -> Self {
        Self {
            current_challenge: 0,
            pending: None,
            last_good_position,
            dimension_change_pending: false,
        }
    }

    #[must_use]
    pub const fn current_challenge(&self) -> i32 {
        self.current_challenge
    }

    #[must_use]
    pub const fn pending_position(&self) -> Option<Vector3> {
        match self.pending {
            Some(pending) => Some(pending.authoritative_position),
            None => None,
        }
    }

    #[must_use]
    pub const fn last_good_position(&self) -> Vector3 {
        self.last_good_position
    }

    #[must_use]
    pub const fn dimension_change_pending(&self) -> bool {
        self.dimension_change_pending
    }

    pub const fn mark_dimension_change_pending(&mut self) {
        self.dimension_change_pending = true;
    }

    pub fn issue_correction(
        &mut self,
        authoritative_position: Vector3,
        listener_tick: i32,
    ) -> TeleportChallenge {
        self.current_challenge = next_teleport_challenge(self.current_challenge);
        self.pending = Some(PendingTeleport {
            authoritative_position,
            sent_at_tick: listener_tick,
        });
        TeleportChallenge {
            challenge: self.current_challenge,
            authoritative_position,
        }
    }

    #[must_use]
    pub const fn movement_disposition(&self) -> MovementDisposition {
        if self.pending.is_some() {
            MovementDisposition::SuppressWhileTeleportPending
        } else {
            MovementDisposition::Validate
        }
    }

    pub fn acknowledge(&mut self, packet: AcceptTeleportation) -> TeleportAcknowledgement {
        if packet.challenge != self.current_challenge {
            return TeleportAcknowledgement::IgnoredStale {
                received: packet.challenge,
                current: self.current_challenge,
            };
        }
        let Some(pending) = self.pending.take() else {
            return TeleportAcknowledgement::DisconnectInvalidMovement;
        };
        self.last_good_position = pending.authoritative_position;
        let completed_dimension_change = self.dimension_change_pending;
        self.dimension_change_pending = false;
        TeleportAcknowledgement::Accepted {
            authoritative_position: pending.authoritative_position,
            completed_dimension_change,
        }
    }

    pub fn handle(&mut self, packet: PlayServerboundEntryPacket) -> TeleportAcknowledgement {
        match packet {
            PlayServerboundEntryPacket::AcceptTeleportation(packet) => self.acknowledge(packet),
        }
    }

    pub fn resend_if_due(&mut self, listener_tick: i32) -> Option<TeleportChallenge> {
        let pending = self.pending?;
        if listener_tick.wrapping_sub(pending.sent_at_tick) <= RESEND_AFTER_TICKS {
            return None;
        }
        Some(self.issue_correction(pending.authoritative_position, listener_tick))
    }
}

impl Default for TeleportSynchronizer {
    fn default() -> Self {
        Self::new(Vector3::default())
    }
}

#[must_use]
pub const fn next_teleport_challenge(current: i32) -> i32 {
    if current == i32::MAX { 0 } else { current + 1 }
}
