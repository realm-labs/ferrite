use crate::java_26_2::catalog::ConnectionState;
use crate::java_26_2::configuration::serverbound::packet::ClientInformation;
use crate::java_26_2::handshake::transition::{LoginRefusal, RoutingContext};
use crate::java_26_2::login::profile::GameProfile;
use crate::java_26_2::login::serverbound::session::LoginDisconnect;
use crate::java_26_2::value::known_pack::KnownPack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerConnectionStage {
    Handshake,
    Status,
    Login,
    Configuration,
    InstallingPlay,
    Play,
    Closing,
    Closed,
    Faulted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundFrame {
    pub sequence: u64,
    pub state: ConnectionState,
    pub identity: &'static str,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayInstallationRequest {
    pub profile: GameProfile,
    pub client_information: ClientInformation,
    pub transferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerConnectionEvent {
    Routed(RoutingContext),
    DisconnectExisting {
        profile_id: u128,
    },
    ConfigurationStarted {
        profile: GameProfile,
    },
    RegistrySelection {
        selected_packs: Vec<KnownPack>,
        exact_offer_match: bool,
    },
    LatencyUpdated {
        latency_millis: i32,
    },
    PlayInstallationRequested(PlayInstallationRequest),
    Closed(ConnectionCloseReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionCloseReason {
    StatusUnavailable,
    StatusRequestHandled,
    HandshakeRefused(LoginRefusal),
    LoginRejected(LoginDisconnect),
    ConfigurationTimeout,
}
