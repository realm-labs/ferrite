use thiserror::Error;

use crate::java_26_2::catalog::ConnectionState;
use crate::java_26_2::configuration::clientbound::codec::ConfigurationClientboundCodecError;
use crate::java_26_2::configuration::serverbound::codec::ConfigurationServerboundCodecError;
use crate::java_26_2::configuration::serverbound::session::ConfigurationServerSessionError;
use crate::java_26_2::connection::output::ServerConnectionStage;
use crate::java_26_2::handshake::codec::HandshakeCodecError;
use crate::java_26_2::handshake::transition::HandshakeTransitionError;
use crate::java_26_2::login::clientbound::codec::LoginClientboundCodecError;
use crate::java_26_2::login::serverbound::codec::LoginServerboundCodecError;
use crate::java_26_2::login::serverbound::session::LoginServerSessionError;
use crate::java_26_2::play::clientbound::codec::PlayClientboundCodecError;
use crate::java_26_2::play::serverbound::codec::PlayServerboundEntryCodecError;
use crate::java_26_2::status::clientbound::codec::StatusClientboundCodecError;
use crate::java_26_2::status::serverbound::codec::StatusServerboundCodecError;
use crate::java_26_2::status::serverbound::session::StatusServerSessionError;
use crate::java_26_2::wire::error::WireError;

#[derive(Debug, Error)]
pub enum ServerConnectionError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    HandshakeCodec(#[from] HandshakeCodecError),
    #[error(transparent)]
    HandshakeTransition(#[from] HandshakeTransitionError),
    #[error(transparent)]
    StatusServerboundCodec(#[from] StatusServerboundCodecError),
    #[error(transparent)]
    StatusClientboundCodec(#[from] StatusClientboundCodecError),
    #[error(transparent)]
    StatusSession(#[from] StatusServerSessionError),
    #[error(transparent)]
    LoginServerboundCodec(#[from] LoginServerboundCodecError),
    #[error(transparent)]
    LoginClientboundCodec(#[from] LoginClientboundCodecError),
    #[error(transparent)]
    LoginSession(#[from] LoginServerSessionError),
    #[error(transparent)]
    ConfigurationServerboundCodec(#[from] ConfigurationServerboundCodecError),
    #[error(transparent)]
    ConfigurationClientboundCodec(#[from] ConfigurationClientboundCodecError),
    #[error(transparent)]
    ConfigurationSession(#[from] ConfigurationServerSessionError),
    #[error(transparent)]
    PlayClientboundCodec(#[from] PlayClientboundCodecError),
    #[error(transparent)]
    PlayServerboundCodec(#[from] PlayServerboundEntryCodecError),
    #[error("connection stage {stage:?} cannot accept this operation")]
    TerminalStage { stage: ServerConnectionStage },
    #[error("{operation} requires stage {expected:?}, but connection is {actual:?}")]
    UnexpectedStage {
        operation: &'static str,
        expected: ServerConnectionStage,
        actual: ServerConnectionStage,
    },
    #[error("connection is missing its {0} state owner")]
    MissingStateOwner(&'static str),
    #[error("status routing selected without a cached status snapshot")]
    MissingStatusSnapshot,
    #[error("configuration transition steps differ from the locked order")]
    InvalidConfigurationTransition,
    #[error("compression negotiation is missing the selected server-session UUID")]
    MissingServerSessionId,
    #[error("compression threshold cannot be represented as a signed VarInt")]
    CompressionThresholdOutOfRange,
    #[error("configuration reached play installation without a normalized profile")]
    MissingNormalizedProfile,
    #[error("status pong callback did not produce the terminal close action")]
    InvalidStatusCompletion,
    #[error("no outbound frame is awaiting send completion")]
    NoOutboundInFlight,
    #[error("outbound completion sequence {actual} arrived before {expected}")]
    UnexpectedOutboundSequence { expected: u64, actual: u64 },
    #[error("clientbound state is {actual:?}, expected {expected:?}")]
    WrongClientboundState {
        expected: ConnectionState,
        actual: ConnectionState,
    },
    #[error("outbound sequence space is exhausted")]
    SequenceExhausted,
    #[error("bounded outbound queue reached {maximum} frames")]
    OutboundQueueFull { maximum: usize },
    #[error("bounded connection-event queue reached {maximum} events")]
    EventQueueFull { maximum: usize },
}
