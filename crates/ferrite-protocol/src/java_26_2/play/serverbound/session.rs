//! Connection-local Play challenge state.

use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::serverbound::packet::KeepAlive;
use crate::java_26_2::play::serverbound::teleport::{
    TeleportAcknowledgement, TeleportChallenge, TeleportSynchronizer,
};

const KEEP_ALIVE_INTERVAL_MILLIS: i64 = 15_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaySessionAction {
    None,
    SendKeepAlive(i64),
    KeepAliveAccepted { latency_millis: i32 },
    DisconnectTimeout,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerCorrectionChallenge {
    pub teleport: TeleportChallenge,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayServerSession {
    teleport: TeleportSynchronizer,
    correction_yaw: f32,
    correction_pitch: f32,
    listener_tick: i32,
    keep_alive_baseline_millis: Option<i64>,
    keep_alive_pending: bool,
    keep_alive_challenge: i64,
    latency_millis: i32,
}

impl PlayServerSession {
    #[must_use]
    pub const fn new(initial_position: Vector3, initial_latency_millis: i32) -> Self {
        Self {
            teleport: TeleportSynchronizer::new(initial_position),
            correction_yaw: 0.0,
            correction_pitch: 0.0,
            listener_tick: 0,
            keep_alive_baseline_millis: None,
            keep_alive_pending: false,
            keep_alive_challenge: 0,
            latency_millis: initial_latency_millis,
        }
    }

    #[must_use]
    pub const fn teleport_pending(&self) -> bool {
        self.teleport.pending_position().is_some()
    }

    #[must_use]
    pub const fn latency_millis(&self) -> i32 {
        self.latency_millis
    }

    pub fn issue_correction(
        &mut self,
        authoritative_position: Vector3,
        yaw: f32,
        pitch: f32,
    ) -> PlayerCorrectionChallenge {
        self.correction_yaw = yaw;
        self.correction_pitch = pitch;
        PlayerCorrectionChallenge {
            teleport: self
                .teleport
                .issue_correction(authoritative_position, self.listener_tick),
            yaw,
            pitch,
        }
    }

    pub fn acknowledge_teleport(&mut self, challenge: i32) -> TeleportAcknowledgement {
        self.teleport.acknowledge(
            crate::java_26_2::play::serverbound::packet::AcceptTeleportation { challenge },
        )
    }

    pub fn advance_listener_tick(&mut self) -> Option<PlayerCorrectionChallenge> {
        self.listener_tick = self.listener_tick.wrapping_add(1);
        self.teleport
            .resend_if_due(self.listener_tick)
            .map(|teleport| PlayerCorrectionChallenge {
                teleport,
                yaw: self.correction_yaw,
                pitch: self.correction_pitch,
            })
    }

    pub fn poll_liveness(
        &mut self,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> PlaySessionAction {
        if is_singleplayer_owner {
            return PlaySessionAction::None;
        }
        let Some(baseline) = self.keep_alive_baseline_millis else {
            self.keep_alive_baseline_millis = Some(now_millis);
            return PlaySessionAction::None;
        };
        if now_millis.saturating_sub(baseline) < KEEP_ALIVE_INTERVAL_MILLIS {
            return PlaySessionAction::None;
        }
        if self.keep_alive_pending {
            return PlaySessionAction::DisconnectTimeout;
        }
        self.keep_alive_pending = true;
        self.keep_alive_challenge = now_millis;
        self.keep_alive_baseline_millis = Some(now_millis);
        PlaySessionAction::SendKeepAlive(now_millis)
    }

    pub fn accept_keep_alive(
        &mut self,
        packet: KeepAlive,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> PlaySessionAction {
        if is_singleplayer_owner {
            return PlaySessionAction::None;
        }
        if !self.keep_alive_pending || packet.challenge != self.keep_alive_challenge {
            return PlaySessionAction::DisconnectTimeout;
        }
        self.keep_alive_pending = false;
        let round_trip = now_millis.saturating_sub(self.keep_alive_challenge) as i32;
        self.latency_millis = self.latency_millis.wrapping_mul(3).wrapping_add(round_trip) / 4;
        PlaySessionAction::KeepAliveAccepted {
            latency_millis: self.latency_millis,
        }
    }
}

impl Default for PlayServerSession {
    fn default() -> Self {
        Self::new(Vector3::default(), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_alive_uses_two_exact_fifteen_second_windows() {
        let mut session = PlayServerSession::default();
        assert_eq!(session.poll_liveness(1_000, false), PlaySessionAction::None);
        assert_eq!(
            session.poll_liveness(15_999, false),
            PlaySessionAction::None
        );
        assert_eq!(
            session.poll_liveness(16_000, false),
            PlaySessionAction::SendKeepAlive(16_000)
        );
        assert_eq!(
            session.poll_liveness(30_999, false),
            PlaySessionAction::None
        );
        assert_eq!(
            session.poll_liveness(31_000, false),
            PlaySessionAction::DisconnectTimeout
        );
    }

    #[test]
    fn only_an_exact_echo_updates_weighted_latency() {
        let mut session = PlayServerSession::new(Vector3::default(), 20);
        session.poll_liveness(0, false);
        session.poll_liveness(15_000, false);
        assert_eq!(
            session.accept_keep_alive(KeepAlive { challenge: 14_999 }, 15_040, false,),
            PlaySessionAction::DisconnectTimeout
        );
        assert_eq!(
            session.accept_keep_alive(KeepAlive { challenge: 15_000 }, 15_040, false,),
            PlaySessionAction::KeepAliveAccepted { latency_millis: 25 }
        );
    }

    #[test]
    fn singleplayer_owner_skips_challenges_and_bad_echoes() {
        let mut session = PlayServerSession::default();
        assert_eq!(
            session.poll_liveness(i64::MAX, true),
            PlaySessionAction::None
        );
        assert_eq!(
            session.accept_keep_alive(KeepAlive { challenge: 1 }, 2, true),
            PlaySessionAction::None
        );
    }
}
