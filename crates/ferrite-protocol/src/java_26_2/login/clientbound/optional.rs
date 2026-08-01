use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_SERVER_ID_CODE_UNITS: usize = 20;
const MAX_CUSTOM_QUERY_PAYLOAD: usize = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalLoginClientboundPacketKind {
    EncryptionHello,
    CustomQuery,
    CookieRequest,
}

impl OptionalLoginClientboundPacketKind {
    pub const ALL: [Self; 3] = [
        Self::EncryptionHello,
        Self::CustomQuery,
        Self::CookieRequest,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::EncryptionHello => 1,
            Self::CustomQuery => 4,
            Self::CookieRequest => 5,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::EncryptionHello => "minecraft:hello",
            Self::CustomQuery => "minecraft:custom_query",
            Self::CookieRequest => "minecraft:cookie_request",
        }
    }

    #[must_use]
    pub const fn service(self) -> LoginClientboundOptionalService {
        match self {
            Self::EncryptionHello => LoginClientboundOptionalService::OnlineAuthentication,
            Self::CustomQuery => LoginClientboundOptionalService::CustomQuery,
            Self::CookieRequest => LoginClientboundOptionalService::Cookies,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalLoginClientboundPacket {
    EncryptionHello {
        server_id: String,
        public_key: Vec<u8>,
        challenge: Vec<u8>,
        authenticate: bool,
    },
    CustomQuery {
        transaction_id: i32,
        channel: Identifier,
        payload: Vec<u8>,
    },
    CookieRequest {
        key: Identifier,
    },
}

impl OptionalLoginClientboundPacket {
    #[must_use]
    pub const fn kind(&self) -> OptionalLoginClientboundPacketKind {
        match self {
            Self::EncryptionHello { .. } => OptionalLoginClientboundPacketKind::EncryptionHello,
            Self::CustomQuery { .. } => OptionalLoginClientboundPacketKind::CustomQuery,
            Self::CookieRequest { .. } => OptionalLoginClientboundPacketKind::CookieRequest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OptionalLoginClientboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("login clientbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("login clientbound packet {identity} is not part of the optional C4 family")]
    RequiredPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing optional packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
}

pub fn decode_packet(
    body: &[u8],
) -> Result<OptionalLoginClientboundPacket, OptionalLoginClientboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Login,
        PacketDirection::Clientbound,
        wire_id,
    )
    .ok_or(OptionalLoginClientboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:hello" => OptionalLoginClientboundPacket::EncryptionHello {
            server_id: reader.read_utf(MAX_SERVER_ID_CODE_UNITS)?.into_owned(),
            public_key: reader.read_byte_array(MAX_INFLATED_PACKET_LENGTH)?.to_vec(),
            challenge: reader.read_byte_array(MAX_INFLATED_PACKET_LENGTH)?.to_vec(),
            authenticate: reader.read_bool()?,
        },
        "minecraft:custom_query" => OptionalLoginClientboundPacket::CustomQuery {
            transaction_id: reader.read_var_i32()?,
            channel: read_identifier(&mut reader)?,
            payload: reader
                .read_bounded_remaining("login custom query payload", MAX_CUSTOM_QUERY_PAYLOAD)?
                .to_vec(),
        },
        "minecraft:cookie_request" => OptionalLoginClientboundPacket::CookieRequest {
            key: read_identifier(&mut reader)?,
        },
        identity => {
            return Err(OptionalLoginClientboundCodecError::RequiredPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &OptionalLoginClientboundPacket,
) -> Result<Vec<u8>, OptionalLoginClientboundCodecError> {
    let identity = packet.kind().identity();
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Login,
        PacketDirection::Clientbound,
        identity,
    )
    .ok_or(OptionalLoginClientboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        OptionalLoginClientboundPacket::EncryptionHello {
            server_id,
            public_key,
            challenge,
            authenticate,
        } => {
            writer.write_utf(server_id, MAX_SERVER_ID_CODE_UNITS)?;
            writer.write_byte_array(public_key, MAX_INFLATED_PACKET_LENGTH)?;
            writer.write_byte_array(challenge, MAX_INFLATED_PACKET_LENGTH)?;
            writer.write_bool(*authenticate)?;
        }
        OptionalLoginClientboundPacket::CustomQuery {
            transaction_id,
            channel,
            payload,
        } => {
            writer.write_var_i32(*transaction_id)?;
            channel.write(&mut writer)?;
            if payload.len() > MAX_CUSTOM_QUERY_PAYLOAD {
                return Err(WireError::LengthLimit {
                    field: "login custom query payload",
                    length: payload.len(),
                    maximum: MAX_CUSTOM_QUERY_PAYLOAD,
                }
                .into());
            }
            writer.write_bytes(payload)?;
        }
        OptionalLoginClientboundPacket::CookieRequest { key } => key.write(&mut writer)?,
    }
    Ok(writer.into_inner())
}

fn read_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Identifier, OptionalLoginClientboundCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginClientboundOptionalService {
    OnlineAuthentication,
    CustomQuery,
    Cookies,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LoginClientboundGates {
    pub online_authentication: bool,
    pub custom_query: bool,
    pub cookies: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionalLoginClientboundContext {
    pub valid_hello_received: bool,
    pub memory_connection: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalLoginClientboundEffect {
    EnterKeyStage,
    RegisterCorrelatedQuery,
    RegisterCookieRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalLoginClientboundDecision {
    OmitDisabled(LoginClientboundOptionalService),
    RefuseEncryptionBeforeValidHello,
    OmitEncryptionForMemoryConnection,
    Emit(OptionalLoginClientboundEffect),
}

impl LoginClientboundGates {
    #[must_use]
    pub const fn decide(
        self,
        packet: &OptionalLoginClientboundPacket,
        context: OptionalLoginClientboundContext,
    ) -> OptionalLoginClientboundDecision {
        let service = packet.kind().service();
        if !self.enabled(service) {
            return OptionalLoginClientboundDecision::OmitDisabled(service);
        }
        OptionalLoginClientboundDecision::Emit(match packet {
            OptionalLoginClientboundPacket::EncryptionHello { .. } => {
                if !context.valid_hello_received {
                    return OptionalLoginClientboundDecision::RefuseEncryptionBeforeValidHello;
                }
                if context.memory_connection {
                    return OptionalLoginClientboundDecision::OmitEncryptionForMemoryConnection;
                }
                OptionalLoginClientboundEffect::EnterKeyStage
            }
            OptionalLoginClientboundPacket::CustomQuery { .. } => {
                OptionalLoginClientboundEffect::RegisterCorrelatedQuery
            }
            OptionalLoginClientboundPacket::CookieRequest { .. } => {
                OptionalLoginClientboundEffect::RegisterCookieRequest
            }
        })
    }

    const fn enabled(self, service: LoginClientboundOptionalService) -> bool {
        match service {
            LoginClientboundOptionalService::OnlineAuthentication => self.online_authentication,
            LoginClientboundOptionalService::CustomQuery => self.custom_query,
            LoginClientboundOptionalService::Cookies => self.cookies,
        }
    }
}
