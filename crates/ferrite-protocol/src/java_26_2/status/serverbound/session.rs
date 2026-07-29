use thiserror::Error;

use crate::java_26_2::status::clientbound::packet::{ServerStatus, StatusClientboundPacket};
use crate::java_26_2::status::serverbound::packet::StatusServerboundPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusServerStage {
    Open,
    PongPending,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusServerAction {
    Send(StatusClientboundPacket),
    CloseRequestHandled,
}

/// One status connection bound to the snapshot selected at handshake time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusServerSession {
    cached_status: ServerStatus,
    request_handled: bool,
    stage: StatusServerStage,
}

impl StatusServerSession {
    #[must_use]
    pub const fn new(cached_status: ServerStatus) -> Self {
        Self {
            cached_status,
            request_handled: false,
            stage: StatusServerStage::Open,
        }
    }

    #[must_use]
    pub const fn stage(&self) -> StatusServerStage {
        self.stage
    }

    #[must_use]
    pub const fn request_handled(&self) -> bool {
        self.request_handled
    }

    #[must_use]
    pub const fn cached_status(&self) -> &ServerStatus {
        &self.cached_status
    }

    pub fn apply(
        &mut self,
        packet: StatusServerboundPacket,
    ) -> Result<StatusServerAction, StatusServerSessionError> {
        if self.stage != StatusServerStage::Open {
            return Err(StatusServerSessionError::NotOpen { stage: self.stage });
        }
        match packet {
            StatusServerboundPacket::Request if !self.request_handled => {
                self.request_handled = true;
                Ok(StatusServerAction::Send(StatusClientboundPacket::Response(
                    self.cached_status.clone(),
                )))
            }
            StatusServerboundPacket::Request => {
                self.stage = StatusServerStage::Closed;
                Ok(StatusServerAction::CloseRequestHandled)
            }
            StatusServerboundPacket::Ping(token) => {
                self.stage = StatusServerStage::PongPending;
                Ok(StatusServerAction::Send(StatusClientboundPacket::Pong(
                    token,
                )))
            }
        }
    }

    pub fn pong_sent(&mut self) -> Result<StatusServerAction, StatusServerSessionError> {
        if self.stage != StatusServerStage::PongPending {
            return Err(StatusServerSessionError::PongNotPending { stage: self.stage });
        }
        self.stage = StatusServerStage::Closed;
        Ok(StatusServerAction::CloseRequestHandled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum StatusServerSessionError {
    #[error("status session is not open in stage {stage:?}")]
    NotOpen { stage: StatusServerStage },
    #[error("status pong completion callback is invalid in stage {stage:?}")]
    PongNotPending { stage: StatusServerStage },
}
