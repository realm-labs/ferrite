use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerInboundReconfigurationStage {
    Play,
    WaitingForAcknowledgement,
    Configuration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerInboundAcknowledgementStep {
    ValidateOldPlayListenerWaiting,
    CaptureReplacementCommonListenerCookie,
    InstallConfigurationInbound,
}

impl ServerInboundAcknowledgementStep {
    pub const ORDER: [Self; 3] = [
        Self::ValidateOldPlayListenerWaiting,
        Self::CaptureReplacementCommonListenerCookie,
        Self::InstallConfigurationInbound,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementCommonListenerCookieField {
    Profile,
    CurrentLatency,
    LatestClientInformation,
    Transferred,
}

impl ReplacementCommonListenerCookieField {
    pub const ALL: [Self; 4] = [
        Self::Profile,
        Self::CurrentLatency,
        Self::LatestClientInformation,
        Self::Transferred,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerInboundReconfigurationEffect {
    InstallConfigurationInboundAtTerminalBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerInboundReconfigurationTransition {
    stage: ServerInboundReconfigurationStage,
}

impl ServerInboundReconfigurationTransition {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stage: ServerInboundReconfigurationStage::Play,
        }
    }

    #[must_use]
    pub const fn stage(self) -> ServerInboundReconfigurationStage {
        self.stage
    }

    pub fn begin_waiting(&mut self) -> Result<(), ServerInboundReconfigurationError> {
        if !matches!(self.stage, ServerInboundReconfigurationStage::Play) {
            return Err(ServerInboundReconfigurationError::StartOutsidePlay);
        }
        self.stage = ServerInboundReconfigurationStage::WaitingForAcknowledgement;
        Ok(())
    }

    pub fn handle_acknowledgement(
        &mut self,
    ) -> Result<ServerInboundReconfigurationEffect, ServerInboundReconfigurationError> {
        if !matches!(
            self.stage,
            ServerInboundReconfigurationStage::WaitingForAcknowledgement
        ) {
            return Err(ServerInboundReconfigurationError::AcknowledgementOutsideWaiting);
        }
        self.stage = ServerInboundReconfigurationStage::Configuration;
        Ok(ServerInboundReconfigurationEffect::InstallConfigurationInboundAtTerminalBoundary)
    }
}

impl Default for ServerInboundReconfigurationTransition {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ServerInboundReconfigurationError {
    #[error("reconfiguration waiting can begin only under the old Play listener")]
    StartOutsidePlay,
    #[error("configuration acknowledgement requires the old Play listener's waiting flag")]
    AcknowledgementOutsideWaiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcknowledgementExecutionLane {
    DirectTerminalNetworkBoundary,
}

#[must_use]
pub const fn acknowledgement_execution_lane() -> AcknowledgementExecutionLane {
    AcknowledgementExecutionLane::DirectTerminalNetworkBoundary
}
