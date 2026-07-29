use thiserror::Error;

use crate::java_26_2::status::clientbound::packet::{
    ServerStatus, StatusClientboundPacket, StatusDescription, StatusPlayers, StatusVersion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClientStage {
    AwaitingResponse,
    AwaitingPong,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClientAction {
    SendPing {
        token: i64,
        persistent_icon_changed: bool,
    },
    CloseUnrequestedResponse,
    Complete {
        latency_millis: i64,
        legacy_fallback_on_disconnect: bool,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatusPresentation {
    pub description: StatusDescription,
    pub players: Option<StatusPlayers>,
    pub version: Option<StatusVersion>,
    pub icon: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusClientSession {
    stage: StatusClientStage,
    presentation: StatusPresentation,
    ping_start_millis: i64,
    successful_response: bool,
}

impl StatusClientSession {
    #[must_use]
    pub fn new(existing_icon: Option<Vec<u8>>) -> Self {
        Self {
            stage: StatusClientStage::AwaitingResponse,
            presentation: StatusPresentation {
                icon: existing_icon,
                ..StatusPresentation::default()
            },
            ping_start_millis: 0,
            successful_response: false,
        }
    }

    #[must_use]
    pub const fn stage(&self) -> StatusClientStage {
        self.stage
    }

    #[must_use]
    pub fn presentation(&self) -> &StatusPresentation {
        &self.presentation
    }

    #[must_use]
    pub const fn successful_response(&self) -> bool {
        self.successful_response
    }

    pub fn apply(
        &mut self,
        packet: StatusClientboundPacket,
        now_millis: i64,
    ) -> Result<StatusClientAction, StatusClientError> {
        if self.stage == StatusClientStage::Closed {
            return Err(StatusClientError::Closed);
        }
        match packet {
            StatusClientboundPacket::Response(status) => self.apply_response(status, now_millis),
            StatusClientboundPacket::Pong(_) => {
                let latency_millis = now_millis.wrapping_sub(self.ping_start_millis);
                self.stage = StatusClientStage::Closed;
                Ok(StatusClientAction::Complete {
                    latency_millis,
                    legacy_fallback_on_disconnect: !self.successful_response,
                })
            }
        }
    }

    fn apply_response(
        &mut self,
        status: ServerStatus,
        now_millis: i64,
    ) -> Result<StatusClientAction, StatusClientError> {
        if self.stage == StatusClientStage::AwaitingPong {
            self.stage = StatusClientStage::Closed;
            return Ok(StatusClientAction::CloseUnrequestedResponse);
        }
        let persistent_icon_changed = self.apply_presentation(status);
        self.ping_start_millis = now_millis;
        self.successful_response = true;
        self.stage = StatusClientStage::AwaitingPong;
        Ok(StatusClientAction::SendPing {
            token: now_millis,
            persistent_icon_changed,
        })
    }

    fn apply_presentation(&mut self, status: ServerStatus) -> bool {
        self.presentation.description = status.description;
        self.presentation.players = status.players;
        self.presentation.version = status.version;
        let Some(icon) = status.favicon else {
            return false;
        };
        if self.presentation.icon.as_deref() == Some(icon.as_slice()) {
            return false;
        }
        self.presentation.icon = valid_status_icon(icon);
        true
    }
}

impl Default for StatusClientSession {
    fn default() -> Self {
        Self::new(None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum StatusClientError {
    #[error("status client session is already closed")]
    Closed,
}

fn valid_status_icon(icon: Vec<u8>) -> Option<Vec<u8>> {
    if icon.len() < 24
        || icon[..8] != [137, 80, 78, 71, 13, 10, 26, 10]
        || u32::from_be_bytes([icon[8], icon[9], icon[10], icon[11]]) != 13
        || icon[12..16] != *b"IHDR"
    {
        return None;
    }
    let width = i32::from_be_bytes([icon[16], icon[17], icon[18], icon[19]]);
    let height = i32::from_be_bytes([icon[20], icon[21], icon[22], icon[23]]);
    (width <= 1_024 && height <= 1_024).then_some(icon)
}
