use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerReconfigurationStep {
    SetWaitingForAcknowledgement,
    SaveAndRemovePlayerFromPlay,
    SendTerminalStartConfiguration,
    InstallConfigurationOutbound,
}

impl ServerReconfigurationStep {
    pub const ORDER: [Self; 4] = [
        Self::SetWaitingForAcknowledgement,
        Self::SaveAndRemovePlayerFromPlay,
        Self::SendTerminalStartConfiguration,
        Self::InstallConfigurationOutbound,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientReconfigurationStep {
    FlushDelayedChat,
    SendPendingLastSeenAcknowledgement,
    StoreChatAndCommonState,
    ClearLevelAndShowReconfigurationScreen,
    CreateConfigurationListenerWithFreshLoadTracker,
    InstallConfigurationInbound,
    SendTerminalPlayAcknowledgement,
    InstallConfigurationOutbound,
}

impl ClientReconfigurationStep {
    pub const ORDER: [Self; 8] = [
        Self::FlushDelayedChat,
        Self::SendPendingLastSeenAcknowledgement,
        Self::StoreChatAndCommonState,
        Self::ClearLevelAndShowReconfigurationScreen,
        Self::CreateConfigurationListenerWithFreshLoadTracker,
        Self::InstallConfigurationInbound,
        Self::SendTerminalPlayAcknowledgement,
        Self::InstallConfigurationOutbound,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarriedReconfigurationState {
    Profile,
    TelemetryManager,
    Registries,
    EnabledFeatures,
    Brand,
    ServerRecord,
    PostDisconnectScreen,
    Cookies,
    ChatState,
    ReportDetails,
    ValidatedServerLinks,
    SeenPlayers,
    InsecureChatWarning,
}

impl CarriedReconfigurationState {
    pub const ALL: [Self; 13] = [
        Self::Profile,
        Self::TelemetryManager,
        Self::Registries,
        Self::EnabledFeatures,
        Self::Brand,
        Self::ServerRecord,
        Self::PostDisconnectScreen,
        Self::Cookies,
        Self::ChatState,
        Self::ReportDetails,
        Self::ValidatedServerLinks,
        Self::SeenPlayers,
        Self::InsecureChatWarning,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientReconfigurationStage {
    Play,
    Configuration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientReconfigurationEffect {
    ExecuteTerminalPlan,
    CreateFreshPlayProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientReconfigurationTransition {
    stage: ClientReconfigurationStage,
}

impl ClientReconfigurationTransition {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stage: ClientReconfigurationStage::Play,
        }
    }

    #[must_use]
    pub const fn stage(self) -> ClientReconfigurationStage {
        self.stage
    }

    pub fn handle_start(
        &mut self,
    ) -> Result<ClientReconfigurationEffect, ClientReconfigurationError> {
        if !matches!(self.stage, ClientReconfigurationStage::Play) {
            return Err(ClientReconfigurationError::StartOutsidePlay);
        }
        self.stage = ClientReconfigurationStage::Configuration;
        Ok(ClientReconfigurationEffect::ExecuteTerminalPlan)
    }

    pub fn finish_configuration(
        &mut self,
    ) -> Result<ClientReconfigurationEffect, ClientReconfigurationError> {
        if !matches!(self.stage, ClientReconfigurationStage::Configuration) {
            return Err(ClientReconfigurationError::FinishOutsideConfiguration);
        }
        self.stage = ClientReconfigurationStage::Play;
        Ok(ClientReconfigurationEffect::CreateFreshPlayProjection)
    }
}

impl Default for ClientReconfigurationTransition {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ClientReconfigurationError {
    #[error("start-configuration is legal only under the old Play listener")]
    StartOutsidePlay,
    #[error("configuration finish is legal only under the configuration listener")]
    FinishOutsideConfiguration,
}
