use thiserror::Error;

use crate::java_26_2::configuration::serverbound::packet::{
    ClientInformation, ConfigurationServerboundPacket,
};
use crate::java_26_2::value::known_pack::KnownPack;

pub const KEEPALIVE_INTERVAL_MILLIS: i64 = 15_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationTask {
    SynchronizeRegistries,
    PrepareSpawn,
    JoinWorld,
    InstallingPlay,
    Play,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerAction {
    None,
    SendKeepAlive(i64),
    RegistrySelection(RegistrySelection),
    KeepAliveAccepted { latency_millis: i32 },
    BeginPlayInstallation(PlayInstallation),
    DisconnectTimeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrySelection {
    pub selected_packs: Vec<KnownPack>,
    pub exact_offer_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayInstallation {
    pub client_information: ClientInformation,
}

/// Server-owned state for the required configuration task and common-liveness gates.
#[derive(Debug, Clone)]
pub struct ConfigurationServerSession {
    task: ConfigurationTask,
    client_information: ClientInformation,
    offered_packs: Vec<KnownPack>,
    keepalive_time: i64,
    keepalive_pending: bool,
    keepalive_challenge: i64,
    latency_millis: i32,
}

impl ConfigurationServerSession {
    #[must_use]
    pub fn new(
        offered_packs: Vec<KnownPack>,
        initial_client_information: ClientInformation,
        now_millis: i64,
        initial_latency_millis: i32,
    ) -> Self {
        Self {
            task: ConfigurationTask::SynchronizeRegistries,
            client_information: initial_client_information,
            offered_packs,
            keepalive_time: now_millis,
            keepalive_pending: false,
            keepalive_challenge: 0,
            latency_millis: initial_latency_millis,
        }
    }

    #[must_use]
    pub const fn task(&self) -> ConfigurationTask {
        self.task
    }

    #[must_use]
    pub const fn client_information(&self) -> &ClientInformation {
        &self.client_information
    }

    #[must_use]
    pub const fn latency_millis(&self) -> i32 {
        self.latency_millis
    }

    #[must_use]
    pub const fn pending_keepalive(&self) -> Option<i64> {
        if self.keepalive_pending {
            Some(self.keepalive_challenge)
        } else {
            None
        }
    }

    pub fn apply(
        &mut self,
        packet: ConfigurationServerboundPacket,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<ServerAction, ConfigurationServerSessionError> {
        self.require_configuration()?;
        match packet {
            ConfigurationServerboundPacket::ClientInformation(information) => {
                self.client_information = information;
                Ok(ServerAction::None)
            }
            ConfigurationServerboundPacket::CustomPayload(_) => Ok(ServerAction::None),
            ConfigurationServerboundPacket::Pong(_) => Ok(ServerAction::None),
            ConfigurationServerboundPacket::KeepAlive(token) => {
                Ok(self.handle_keepalive(token, now_millis, is_singleplayer_owner))
            }
            ConfigurationServerboundPacket::SelectKnownPacks(selected_packs) => {
                if self.task != ConfigurationTask::SynchronizeRegistries {
                    return self.fail_unexpected(
                        "known-pack selection",
                        ConfigurationTask::SynchronizeRegistries,
                    );
                }
                let exact_offer_match = selected_packs == self.offered_packs;
                self.task = ConfigurationTask::PrepareSpawn;
                Ok(ServerAction::RegistrySelection(RegistrySelection {
                    selected_packs,
                    exact_offer_match,
                }))
            }
            ConfigurationServerboundPacket::FinishConfiguration => {
                if self.task != ConfigurationTask::JoinWorld {
                    return self
                        .fail_unexpected("finish configuration", ConfigurationTask::JoinWorld);
                }
                self.task = ConfigurationTask::InstallingPlay;
                Ok(ServerAction::BeginPlayInstallation(PlayInstallation {
                    client_information: self.client_information.clone(),
                }))
            }
        }
    }

    pub fn spawn_ready_and_finish_sent(&mut self) -> Result<(), ConfigurationServerSessionError> {
        if self.task != ConfigurationTask::PrepareSpawn {
            return self.fail_unexpected("spawn readiness", ConfigurationTask::PrepareSpawn);
        }
        self.task = ConfigurationTask::JoinWorld;
        Ok(())
    }

    /// Completes after clientbound play installation, admission, player creation, and serverbound
    /// play installation, before any ordinary play packet is admitted or emitted.
    pub fn play_installation_completed(&mut self) -> Result<(), ConfigurationServerSessionError> {
        if self.task != ConfigurationTask::InstallingPlay {
            return self.fail_unexpected(
                "play installation completion",
                ConfigurationTask::InstallingPlay,
            );
        }
        self.task = ConfigurationTask::Play;
        Ok(())
    }

    pub fn poll_liveness(
        &mut self,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<ServerAction, ConfigurationServerSessionError> {
        self.require_configuration()?;
        if is_singleplayer_owner
            || now_millis.wrapping_sub(self.keepalive_time) < KEEPALIVE_INTERVAL_MILLIS
        {
            return Ok(ServerAction::None);
        }
        if self.keepalive_pending {
            self.task = ConfigurationTask::Disconnected;
            Ok(ServerAction::DisconnectTimeout)
        } else {
            self.keepalive_pending = true;
            self.keepalive_time = now_millis;
            self.keepalive_challenge = now_millis;
            Ok(ServerAction::SendKeepAlive(now_millis))
        }
    }

    fn handle_keepalive(
        &mut self,
        token: i64,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> ServerAction {
        if self.keepalive_pending && token == self.keepalive_challenge {
            let elapsed = now_millis.wrapping_sub(self.keepalive_time) as i32;
            self.latency_millis = self.latency_millis.wrapping_mul(3).wrapping_add(elapsed) / 4;
            self.keepalive_pending = false;
            ServerAction::KeepAliveAccepted {
                latency_millis: self.latency_millis,
            }
        } else if is_singleplayer_owner {
            ServerAction::None
        } else {
            self.task = ConfigurationTask::Disconnected;
            ServerAction::DisconnectTimeout
        }
    }

    fn require_configuration(&self) -> Result<(), ConfigurationServerSessionError> {
        if matches!(
            self.task,
            ConfigurationTask::Play | ConfigurationTask::Disconnected
        ) {
            Err(ConfigurationServerSessionError::TerminalTask { task: self.task })
        } else {
            Ok(())
        }
    }

    fn fail_unexpected<T>(
        &mut self,
        packet: &'static str,
        expected: ConfigurationTask,
    ) -> Result<T, ConfigurationServerSessionError> {
        let actual = self.task;
        self.task = ConfigurationTask::Disconnected;
        Err(ConfigurationServerSessionError::UnexpectedTask {
            packet,
            expected,
            actual,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigurationServerSessionError {
    #[error("{packet} requires task {expected:?}, but current task is {actual:?}")]
    UnexpectedTask {
        packet: &'static str,
        expected: ConfigurationTask,
        actual: ConfigurationTask,
    },
    #[error("configuration packet is illegal in terminal task {task:?}")]
    TerminalTask { task: ConfigurationTask },
}
