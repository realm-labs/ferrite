use thiserror::Error;

use crate::java_26_2::login::clientbound::packet::{LoginClientboundPacket, LoginFinished};
use crate::java_26_2::login::component_json::LoginDisconnectReason;
use crate::java_26_2::wire::compression::{CompressionMode, CompressionThreshold};
use crate::java_26_2::wire::error::WireError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginClientStage {
    Connecting,
    LoginFinishedReceived,
    ConfigurationClientboundInstalled,
    AcknowledgementSent,
    Configuration,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginClientAction {
    InstallCompressionAfterCurrentPacket(CompressionThreshold),
    InstallConfigurationClientbound(LoginFinished),
    SendAcknowledgementUnderLogin,
    InstallConfigurationServerbound,
    Disconnect(LoginDisconnectReason),
}

/// A headless-client projection that makes login codec transition callbacks explicit.
#[derive(Debug, Clone)]
pub struct LoginClientProjection {
    stage: LoginClientStage,
    compression: CompressionMode,
    pending_compression: Option<CompressionThreshold>,
}

impl LoginClientProjection {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stage: LoginClientStage::Connecting,
            compression: CompressionMode::Disabled,
            pending_compression: None,
        }
    }

    #[must_use]
    pub const fn stage(&self) -> LoginClientStage {
        self.stage
    }

    #[must_use]
    pub const fn compression(&self) -> CompressionMode {
        self.compression
    }

    pub fn apply(
        &mut self,
        packet: LoginClientboundPacket,
    ) -> Result<LoginClientAction, LoginClientProjectionError> {
        if self.stage == LoginClientStage::Disconnected {
            return Err(LoginClientProjectionError::TerminalStage);
        }
        match packet {
            LoginClientboundPacket::Disconnect(reason) => {
                self.stage = LoginClientStage::Disconnected;
                Ok(LoginClientAction::Disconnect(reason))
            }
            LoginClientboundPacket::Compression(threshold) => {
                self.require_connecting("login compression")?;
                if self.compression != CompressionMode::Disabled
                    || self.pending_compression.is_some()
                {
                    return Err(LoginClientProjectionError::DuplicateCompression);
                }
                let threshold = usize::try_from(threshold)
                    .map_err(
                        |_| LoginClientProjectionError::NegativeCompressionThreshold { threshold },
                    )
                    .and_then(|threshold| {
                        CompressionThreshold::new(threshold)
                            .map_err(LoginClientProjectionError::InvalidCompressionThreshold)
                    })?;
                self.pending_compression = Some(threshold);
                Ok(LoginClientAction::InstallCompressionAfterCurrentPacket(
                    threshold,
                ))
            }
            LoginClientboundPacket::Finished(finished) => {
                self.require_connecting("login finished")?;
                if self.pending_compression.is_some() {
                    return Err(LoginClientProjectionError::CompressionCallbackPending);
                }
                self.stage = LoginClientStage::LoginFinishedReceived;
                Ok(LoginClientAction::InstallConfigurationClientbound(finished))
            }
        }
    }

    pub fn compression_installed(&mut self) -> Result<(), LoginClientProjectionError> {
        self.require_connecting("compression callback")?;
        let threshold = self
            .pending_compression
            .take()
            .ok_or(LoginClientProjectionError::NoCompressionCallbackPending)?;
        self.compression = CompressionMode::Enabled(threshold);
        Ok(())
    }

    pub fn configuration_clientbound_installed(
        &mut self,
    ) -> Result<LoginClientAction, LoginClientProjectionError> {
        self.require_stage(
            LoginClientStage::LoginFinishedReceived,
            "configuration clientbound install",
        )?;
        self.stage = LoginClientStage::ConfigurationClientboundInstalled;
        Ok(LoginClientAction::SendAcknowledgementUnderLogin)
    }

    pub fn acknowledgement_sent(
        &mut self,
    ) -> Result<LoginClientAction, LoginClientProjectionError> {
        self.require_stage(
            LoginClientStage::ConfigurationClientboundInstalled,
            "login acknowledgement",
        )?;
        self.stage = LoginClientStage::AcknowledgementSent;
        Ok(LoginClientAction::InstallConfigurationServerbound)
    }

    pub fn configuration_serverbound_installed(
        &mut self,
    ) -> Result<(), LoginClientProjectionError> {
        self.require_stage(
            LoginClientStage::AcknowledgementSent,
            "configuration serverbound install",
        )?;
        self.stage = LoginClientStage::Configuration;
        Ok(())
    }

    fn require_connecting(
        &self,
        operation: &'static str,
    ) -> Result<(), LoginClientProjectionError> {
        self.require_stage(LoginClientStage::Connecting, operation)
    }

    fn require_stage(
        &self,
        expected: LoginClientStage,
        operation: &'static str,
    ) -> Result<(), LoginClientProjectionError> {
        if self.stage == expected {
            Ok(())
        } else {
            Err(LoginClientProjectionError::UnexpectedStage {
                operation,
                expected,
                actual: self.stage,
            })
        }
    }
}

impl Default for LoginClientProjection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LoginClientProjectionError {
    #[error("{operation} requires stage {expected:?}, but connection is {actual:?}")]
    UnexpectedStage {
        operation: &'static str,
        expected: LoginClientStage,
        actual: LoginClientStage,
    },
    #[error("login client projection is terminal")]
    TerminalStage,
    #[error("compression threshold {threshold} is negative")]
    NegativeCompressionThreshold { threshold: i32 },
    #[error(transparent)]
    InvalidCompressionThreshold(WireError),
    #[error("login compression may only be negotiated once")]
    DuplicateCompression,
    #[error("the compression codec callback must finish before login finished is decoded")]
    CompressionCallbackPending,
    #[error("no compression codec callback is pending")]
    NoCompressionCallbackPending,
}
