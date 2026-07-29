//! Version-independent session ingress and egress shared with the server runtime.

use std::num::NonZeroU64;

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(NonZeroU64);

impl SessionId {
    pub const fn new(value: u64) -> Result<Self, SessionIdError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(SessionIdError::Zero),
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionIdentity {
    pub profile_id: u128,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualHost {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSettings {
    pub language: String,
    pub view_distance: i8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customization: u8,
    pub main_hand: MainHand,
    pub text_filtering: bool,
    pub allows_listing: bool,
    pub particle_status: ParticleStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainHand {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinRequest {
    pub identity: SessionIdentity,
    pub settings: ClientSettings,
    pub transferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionIngress {
    Routed(VirtualHost),
    DisconnectDuplicate { profile_id: u128 },
    ConfigurationStarted(SessionIdentity),
    LatencyUpdated { latency_millis: i32 },
    JoinRequested(JoinRequest),
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayAdmission {
    pub session: SessionId,
    pub identity: SessionIdentity,
    pub player: StableEntityId,
    pub region: SimulationRegionKey,
    pub region_mapping: RegionMapping,
    pub spawn_chunk: ChunkPos,
    pub spawn: PlayerSpawn,
    pub requested_view_distance: i8,
    pub transferred: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerSpawn {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionEgress {
    Disconnect {
        session: SessionId,
        reason: SessionDisconnectReason,
    },
    CompletePlayInstallation(PlayAdmission),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionDisconnectReason {
    AdmissionDenied(String),
    DuplicateLogin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SessionIdError {
    #[error("session ID cannot be zero")]
    Zero,
}
