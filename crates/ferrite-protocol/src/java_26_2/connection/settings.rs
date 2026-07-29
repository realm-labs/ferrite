use thiserror::Error;

use crate::java_26_2::configuration::serverbound::packet::ClientInformation;
use crate::java_26_2::connection::bootstrap::ConfigurationSnapshot;
use crate::java_26_2::handshake::transition::HandshakePolicy;
use crate::java_26_2::login::component_json::{LoginDisconnectReason, LoginDisconnectReasonError};
use crate::java_26_2::login::serverbound::session::LoginPolicy;
use crate::java_26_2::status::clientbound::packet::ServerStatus;
use crate::java_26_2::value::nbt::{NbtError, TextComponentNbt};
use crate::java_26_2::wire::frame::FrameLimits;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisconnectMessages {
    pub outdated_client: LoginDisconnectReason,
    pub incompatible_version: LoginDisconnectReason,
    pub transfers_disabled: LoginDisconnectReason,
    pub invalid_name: LoginDisconnectReason,
    pub invalid_profile: LoginDisconnectReason,
    pub slow_login: LoginDisconnectReason,
    pub configuration_timeout: TextComponentNbt,
}

impl DisconnectMessages {
    pub fn standard() -> Result<Self, DisconnectMessageError> {
        Ok(Self {
            outdated_client: LoginDisconnectReason::from_json(
                r#"{"translate":"multiplayer.disconnect.outdated_client","with":["26.2"]}"#,
            )?,
            incompatible_version: LoginDisconnectReason::from_json(
                r#"{"translate":"multiplayer.disconnect.incompatible","with":["26.2"]}"#,
            )?,
            transfers_disabled: LoginDisconnectReason::from_json(
                r#"{"translate":"multiplayer.disconnect.transfers_disabled"}"#,
            )?,
            invalid_name: LoginDisconnectReason::from_json(
                r#"{"translate":"multiplayer.disconnect.invalid_player_data"}"#,
            )?,
            invalid_profile: LoginDisconnectReason::from_json(
                r#"{"translate":"multiplayer.disconnect.invalid_player_data"}"#,
            )?,
            slow_login: LoginDisconnectReason::from_json(
                r#"{"translate":"multiplayer.disconnect.slow_login"}"#,
            )?,
            configuration_timeout: TextComponentNbt::literal("Timed out")?,
        })
    }
}

#[derive(Debug, Error)]
pub enum DisconnectMessageError {
    #[error(transparent)]
    Login(#[from] LoginDisconnectReasonError),
    #[error(transparent)]
    Configuration(#[from] NbtError),
}

#[derive(Debug, Clone)]
pub struct ServerConnectionSettings {
    pub handshake_policy: HandshakePolicy,
    pub status: Option<ServerStatus>,
    pub login_policy: LoginPolicy,
    pub configuration: ConfigurationSnapshot,
    pub initial_client_information: ClientInformation,
    pub initial_latency_millis: i32,
    pub disconnect_messages: DisconnectMessages,
    pub frame_limits: FrameLimits,
}

impl ServerConnectionSettings {
    #[must_use]
    pub fn with_required_defaults(
        status: Option<ServerStatus>,
        configuration: ConfigurationSnapshot,
        disconnect_messages: DisconnectMessages,
    ) -> Self {
        Self {
            handshake_policy: HandshakePolicy::default(),
            status,
            login_policy: LoginPolicy::default(),
            configuration,
            initial_client_information: ClientInformation::default(),
            initial_latency_millis: 0,
            disconnect_messages,
            frame_limits: FrameLimits::default(),
        }
    }
}
