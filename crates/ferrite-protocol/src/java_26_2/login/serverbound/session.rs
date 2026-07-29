use md5::{Digest, Md5};
use thiserror::Error;

use crate::java_26_2::login::clientbound::packet::LoginFinished;
use crate::java_26_2::login::component_json::LoginDisconnectReason;
use crate::java_26_2::login::profile::GameProfile;
use crate::java_26_2::login::serverbound::packet::LoginServerboundPacket;
use crate::java_26_2::wire::compression::{CompressionMode, CompressionThreshold};

pub const LOGIN_TIMEOUT_TICKS: u32 = 600;
pub const DEFAULT_COMPRESSION_THRESHOLD: i32 = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginServerStage {
    Hello,
    Verifying,
    AwaitingDuplicateDeparture,
    CompressionSendPending,
    ProtocolSwitching,
    Accepted,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPolicy {
    pub compression_threshold: i32,
    pub memory_connection: bool,
    pub intended_profile_id: Option<u128>,
}

impl Default for LoginPolicy {
    fn default() -> Self {
        Self {
            compression_threshold: DEFAULT_COMPRESSION_THRESHOLD,
            memory_connection: false,
            intended_profile_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionSnapshot {
    pub policy_reason: Option<LoginDisconnectReason>,
    pub duplicate_active: bool,
}

impl AdmissionSnapshot {
    #[must_use]
    pub const fn allowed() -> Self {
        Self {
            policy_reason: None,
            duplicate_active: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginDisconnect {
    InvalidName,
    AdmissionPolicy(LoginDisconnectReason),
    IntendedProfileMismatch { intended: u128, normalized: u128 },
    SlowLogin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationTransitionStep {
    InstallConfigurationClientbound,
    BuildNormalizedConnectionCookie,
    InstallConfigurationServerbound,
    StartConfigurationTasks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationTransition {
    pub profile: GameProfile,
    pub steps: Vec<ConfigurationTransitionStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginServerAction {
    None,
    Disconnect(LoginDisconnect),
    DisconnectExistingAndWait { profile_id: u128 },
    SendCompressionUncompressed(CompressionThreshold),
    SendFinished(LoginFinished),
    BeginConfiguration(ConfigurationTransition),
}

/// The required offline-mode server login listener, isolated from world and ECS state.
#[derive(Debug, Clone)]
pub struct LoginServerSession {
    policy: LoginPolicy,
    stage: LoginServerStage,
    tick_counter: u32,
    profile: Option<GameProfile>,
    compression: CompressionMode,
}

impl LoginServerSession {
    #[must_use]
    pub const fn new(policy: LoginPolicy) -> Self {
        Self {
            policy,
            stage: LoginServerStage::Hello,
            tick_counter: 0,
            profile: None,
            compression: CompressionMode::Disabled,
        }
    }

    #[must_use]
    pub const fn stage(&self) -> LoginServerStage {
        self.stage
    }

    #[must_use]
    pub const fn tick_counter(&self) -> u32 {
        self.tick_counter
    }

    #[must_use]
    pub const fn compression(&self) -> CompressionMode {
        self.compression
    }

    #[must_use]
    pub fn profile(&self) -> Option<&GameProfile> {
        self.profile.as_ref()
    }

    pub fn apply(
        &mut self,
        packet: LoginServerboundPacket,
    ) -> Result<LoginServerAction, LoginServerSessionError> {
        if matches!(
            self.stage,
            LoginServerStage::Accepted | LoginServerStage::Disconnected
        ) {
            return Err(LoginServerSessionError::TerminalStage { stage: self.stage });
        }
        match packet {
            LoginServerboundPacket::Hello(hello) => {
                if self.stage != LoginServerStage::Hello {
                    return self.fail_unexpected("hello", LoginServerStage::Hello);
                }
                if !valid_player_name(&hello.name) {
                    self.stage = LoginServerStage::Disconnected;
                    return Ok(LoginServerAction::Disconnect(LoginDisconnect::InvalidName));
                }
                self.profile = Some(GameProfile {
                    id: offline_player_id(&hello.name),
                    name: hello.name,
                    properties: Vec::new(),
                });
                self.stage = LoginServerStage::Verifying;
                Ok(LoginServerAction::None)
            }
            LoginServerboundPacket::Acknowledged => {
                if self.stage != LoginServerStage::ProtocolSwitching {
                    return self.fail_unexpected(
                        "login acknowledgement",
                        LoginServerStage::ProtocolSwitching,
                    );
                }
                let profile = self
                    .profile
                    .clone()
                    .ok_or(LoginServerSessionError::MissingNormalizedProfile)?;
                self.stage = LoginServerStage::Accepted;
                Ok(LoginServerAction::BeginConfiguration(
                    ConfigurationTransition {
                        profile,
                        steps: vec![
                            ConfigurationTransitionStep::InstallConfigurationClientbound,
                            ConfigurationTransitionStep::BuildNormalizedConnectionCookie,
                            ConfigurationTransitionStep::InstallConfigurationServerbound,
                            ConfigurationTransitionStep::StartConfigurationTasks,
                        ],
                    },
                ))
            }
        }
    }

    pub fn tick(
        &mut self,
        admission: AdmissionSnapshot,
        server_session_id: u128,
    ) -> Result<LoginServerAction, LoginServerSessionError> {
        if matches!(
            self.stage,
            LoginServerStage::Accepted | LoginServerStage::Disconnected
        ) {
            return Ok(LoginServerAction::None);
        }
        let prior = self.tick_counter;
        self.tick_counter = self.tick_counter.saturating_add(1);
        if prior == LOGIN_TIMEOUT_TICKS {
            self.stage = LoginServerStage::Disconnected;
            return Ok(LoginServerAction::Disconnect(LoginDisconnect::SlowLogin));
        }
        if !matches!(
            self.stage,
            LoginServerStage::Verifying | LoginServerStage::AwaitingDuplicateDeparture
        ) {
            return Ok(LoginServerAction::None);
        }
        if let Some(reason) = admission.policy_reason {
            self.stage = LoginServerStage::Disconnected;
            return Ok(LoginServerAction::Disconnect(
                LoginDisconnect::AdmissionPolicy(reason),
            ));
        }
        let profile = self
            .profile
            .as_ref()
            .ok_or(LoginServerSessionError::MissingNormalizedProfile)?;
        if let Some(intended) = self.policy.intended_profile_id
            && intended != profile.id
        {
            self.stage = LoginServerStage::Disconnected;
            return Ok(LoginServerAction::Disconnect(
                LoginDisconnect::IntendedProfileMismatch {
                    intended,
                    normalized: profile.id,
                },
            ));
        }
        if admission.duplicate_active {
            let first_request = self.stage == LoginServerStage::Verifying;
            self.stage = LoginServerStage::AwaitingDuplicateDeparture;
            return if first_request {
                Ok(LoginServerAction::DisconnectExistingAndWait {
                    profile_id: profile.id,
                })
            } else {
                Ok(LoginServerAction::None)
            };
        }
        self.begin_finish(server_session_id)
    }

    pub fn compression_send_completed(
        &mut self,
        server_session_id: u128,
    ) -> Result<LoginServerAction, LoginServerSessionError> {
        if self.stage != LoginServerStage::CompressionSendPending {
            return self.fail_unexpected(
                "compression send callback",
                LoginServerStage::CompressionSendPending,
            );
        }
        let threshold = CompressionThreshold::new(
            usize::try_from(self.policy.compression_threshold)
                .map_err(|_| LoginServerSessionError::InvalidCompressionPolicy)?,
        )
        .map_err(|_| LoginServerSessionError::InvalidCompressionPolicy)?;
        self.compression = CompressionMode::Enabled(threshold);
        self.send_finished(server_session_id)
    }

    fn begin_finish(
        &mut self,
        server_session_id: u128,
    ) -> Result<LoginServerAction, LoginServerSessionError> {
        if self.policy.compression_threshold >= 0 && !self.policy.memory_connection {
            let threshold = CompressionThreshold::new(
                usize::try_from(self.policy.compression_threshold)
                    .map_err(|_| LoginServerSessionError::InvalidCompressionPolicy)?,
            )
            .map_err(|_| LoginServerSessionError::InvalidCompressionPolicy)?;
            self.stage = LoginServerStage::CompressionSendPending;
            Ok(LoginServerAction::SendCompressionUncompressed(threshold))
        } else {
            self.send_finished(server_session_id)
        }
    }

    fn send_finished(
        &mut self,
        server_session_id: u128,
    ) -> Result<LoginServerAction, LoginServerSessionError> {
        let profile = self
            .profile
            .clone()
            .ok_or(LoginServerSessionError::MissingNormalizedProfile)?;
        self.stage = LoginServerStage::ProtocolSwitching;
        Ok(LoginServerAction::SendFinished(LoginFinished {
            profile,
            server_session_id,
        }))
    }

    fn fail_unexpected<T>(
        &mut self,
        packet: &'static str,
        expected: LoginServerStage,
    ) -> Result<T, LoginServerSessionError> {
        let actual = self.stage;
        self.stage = LoginServerStage::Disconnected;
        Err(LoginServerSessionError::UnexpectedStage {
            packet,
            expected,
            actual,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LoginServerSessionError {
    #[error("{packet} requires stage {expected:?}, but connection is {actual:?}")]
    UnexpectedStage {
        packet: &'static str,
        expected: LoginServerStage,
        actual: LoginServerStage,
    },
    #[error("login server session is terminal in stage {stage:?}")]
    TerminalStage { stage: LoginServerStage },
    #[error("normalized login profile is missing")]
    MissingNormalizedProfile,
    #[error("configured compression threshold cannot be installed")]
    InvalidCompressionPolicy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServerSessionIdPool {
    current: Option<u128>,
    active_connections: usize,
}

impl ServerSessionIdPool {
    #[must_use]
    pub fn acquire(&mut self, random_candidate: u128) -> u128 {
        let current = *self.current.get_or_insert(random_candidate);
        self.active_connections = self.active_connections.saturating_add(1);
        current
    }

    pub fn release(&mut self) -> Result<(), ServerSessionIdPoolError> {
        if self.active_connections == 0 {
            return Err(ServerSessionIdPoolError::NoActiveConnection);
        }
        self.active_connections -= 1;
        if self.active_connections == 0 {
            self.current = None;
        }
        Ok(())
    }

    #[must_use]
    pub const fn active_connections(&self) -> usize {
        self.active_connections
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ServerSessionIdPoolError {
    #[error("cannot release a server-session UUID without an active connection")]
    NoActiveConnection,
}

#[must_use]
pub fn valid_player_name(name: &str) -> bool {
    name.encode_utf16().all(|unit| unit > 0x20 && unit < 0x7f)
}

#[must_use]
pub fn offline_player_id(name: &str) -> u128 {
    let mut digest = Md5::new();
    digest.update(b"OfflinePlayer:");
    digest.update(name.as_bytes());
    let mut bytes: [u8; 16] = digest.finalize().into();
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    u128::from_be_bytes(bytes)
}
