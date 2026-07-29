use thiserror::Error;

use crate::java_26_2::catalog::PROTOCOL_VERSION;
use crate::java_26_2::handshake::packet::{ClientIntention, ClientIntentionPacket};

pub const OUTDATED_CLIENT_CUTOFF: i32 = 754;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandshakePolicy {
    pub status_replies_enabled: bool,
    pub cached_status_available: bool,
    pub transfers_enabled: bool,
}

impl Default for HandshakePolicy {
    fn default() -> Self {
        Self {
            status_replies_enabled: true,
            cached_status_available: true,
            transfers_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingContext {
    pub host: String,
    pub port: u16,
    pub protocol_version: i32,
    pub intention: ClientIntention,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeTransitionPlan {
    pub routing_context: RoutingContext,
    pub steps: Vec<HandshakeStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeStep {
    InstallStatusClientbound,
    InstallStatusServerbound,
    InstallLoginClientbound,
    InstallLoginServerbound { transferred: bool },
    SendLoginDisconnect(LoginRefusal),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginRefusal {
    OutdatedClient,
    IncompatibleVersion,
    TransfersDisabled,
}

/// A one-shot handshake owner. Routing a second intention is always a terminal protocol fault.
#[derive(Debug, Clone)]
pub struct HandshakeSession {
    policy: HandshakePolicy,
    terminal: bool,
}

impl HandshakeSession {
    #[must_use]
    pub const fn new(policy: HandshakePolicy) -> Self {
        Self {
            policy,
            terminal: false,
        }
    }

    pub fn route(
        &mut self,
        packet: ClientIntentionPacket,
    ) -> Result<HandshakeTransitionPlan, HandshakeTransitionError> {
        if self.terminal {
            return Err(HandshakeTransitionError::HandshakeAlreadyComplete);
        }
        self.terminal = true;
        let routing_context = RoutingContext {
            host: packet.host,
            port: packet.port,
            protocol_version: packet.protocol_version,
            intention: packet.intention,
        };
        let steps = match packet.intention {
            ClientIntention::Status => self.status_steps(),
            ClientIntention::Login => login_steps(packet.protocol_version, false),
            ClientIntention::Transfer if !self.policy.transfers_enabled => vec![
                HandshakeStep::InstallLoginClientbound,
                HandshakeStep::SendLoginDisconnect(LoginRefusal::TransfersDisabled),
                HandshakeStep::Close,
            ],
            ClientIntention::Transfer => login_steps(packet.protocol_version, true),
        };
        Ok(HandshakeTransitionPlan {
            routing_context,
            steps,
        })
    }

    fn status_steps(&self) -> Vec<HandshakeStep> {
        if self.policy.status_replies_enabled && self.policy.cached_status_available {
            vec![
                HandshakeStep::InstallStatusClientbound,
                HandshakeStep::InstallStatusServerbound,
            ]
        } else {
            vec![
                HandshakeStep::InstallStatusClientbound,
                HandshakeStep::Close,
            ]
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HandshakeTransitionError {
    #[error("the terminal handshake intention was already routed")]
    HandshakeAlreadyComplete,
}

fn login_steps(protocol_version: i32, transferred: bool) -> Vec<HandshakeStep> {
    if protocol_version == PROTOCOL_VERSION as i32 {
        vec![
            HandshakeStep::InstallLoginClientbound,
            HandshakeStep::InstallLoginServerbound { transferred },
        ]
    } else {
        let refusal = if protocol_version < OUTDATED_CLIENT_CUTOFF {
            LoginRefusal::OutdatedClient
        } else {
            LoginRefusal::IncompatibleVersion
        };
        vec![
            HandshakeStep::InstallLoginClientbound,
            HandshakeStep::SendLoginDisconnect(refusal),
            HandshakeStep::Close,
        ]
    }
}
