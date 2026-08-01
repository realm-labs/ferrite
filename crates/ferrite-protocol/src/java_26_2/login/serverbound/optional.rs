use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_CUSTOM_QUERY_ANSWER: usize = 1_048_576;
const MAX_COOKIE_BYTES: usize = 5_120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalLoginServerboundPacketKind {
    Key,
    CustomQueryAnswer,
    CookieResponse,
}

impl OptionalLoginServerboundPacketKind {
    pub const ALL: [Self; 3] = [Self::Key, Self::CustomQueryAnswer, Self::CookieResponse];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::Key => 1,
            Self::CustomQueryAnswer => 2,
            Self::CookieResponse => 4,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::Key => "minecraft:key",
            Self::CustomQueryAnswer => "minecraft:custom_query_answer",
            Self::CookieResponse => "minecraft:cookie_response",
        }
    }

    #[must_use]
    pub const fn service(self) -> LoginServerboundOptionalService {
        match self {
            Self::Key => LoginServerboundOptionalService::OnlineAuthentication,
            Self::CustomQueryAnswer => LoginServerboundOptionalService::CustomQuery,
            Self::CookieResponse => LoginServerboundOptionalService::Cookies,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalLoginServerboundPacket {
    Key {
        encrypted_secret: Vec<u8>,
        encrypted_challenge: Vec<u8>,
    },
    CustomQueryAnswer {
        transaction_id: i32,
        remainder: Vec<u8>,
    },
    CookieResponse {
        key: Identifier,
        value: Option<Vec<u8>>,
    },
}

impl OptionalLoginServerboundPacket {
    #[must_use]
    pub const fn kind(&self) -> OptionalLoginServerboundPacketKind {
        match self {
            Self::Key { .. } => OptionalLoginServerboundPacketKind::Key,
            Self::CustomQueryAnswer { .. } => OptionalLoginServerboundPacketKind::CustomQueryAnswer,
            Self::CookieResponse { .. } => OptionalLoginServerboundPacketKind::CookieResponse,
        }
    }

    #[must_use]
    pub fn null_custom_query_answer(transaction_id: i32) -> Self {
        Self::CustomQueryAnswer {
            transaction_id,
            remainder: vec![0],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OptionalLoginServerboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("login serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("login serverbound packet {identity} is not part of the optional C4 family")]
    RequiredPacketIdentity { identity: &'static str },
    #[error("locked catalog is missing optional packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
}

pub fn decode_packet(
    body: &[u8],
) -> Result<OptionalLoginServerboundPacket, OptionalLoginServerboundCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Login,
        PacketDirection::Serverbound,
        wire_id,
    )
    .ok_or(OptionalLoginServerboundCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:key" => OptionalLoginServerboundPacket::Key {
            encrypted_secret: reader.read_byte_array(MAX_INFLATED_PACKET_LENGTH)?.to_vec(),
            encrypted_challenge: reader.read_byte_array(MAX_INFLATED_PACKET_LENGTH)?.to_vec(),
        },
        "minecraft:custom_query_answer" => OptionalLoginServerboundPacket::CustomQueryAnswer {
            transaction_id: reader.read_var_i32()?,
            remainder: reader
                .read_bounded_remaining("login custom query answer", MAX_CUSTOM_QUERY_ANSWER)?
                .to_vec(),
        },
        "minecraft:cookie_response" => {
            let key = read_identifier(&mut reader)?;
            let value = if reader.read_bool()? {
                Some(reader.read_byte_array(MAX_COOKIE_BYTES)?.to_vec())
            } else {
                None
            };
            OptionalLoginServerboundPacket::CookieResponse { key, value }
        }
        identity => {
            return Err(OptionalLoginServerboundCodecError::RequiredPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &OptionalLoginServerboundPacket,
) -> Result<Vec<u8>, OptionalLoginServerboundCodecError> {
    let identity = packet.kind().identity();
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Login,
        PacketDirection::Serverbound,
        identity,
    )
    .ok_or(OptionalLoginServerboundCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        OptionalLoginServerboundPacket::Key {
            encrypted_secret,
            encrypted_challenge,
        } => {
            writer.write_byte_array(encrypted_secret, MAX_INFLATED_PACKET_LENGTH)?;
            writer.write_byte_array(encrypted_challenge, MAX_INFLATED_PACKET_LENGTH)?;
        }
        OptionalLoginServerboundPacket::CustomQueryAnswer {
            transaction_id,
            remainder,
        } => {
            writer.write_var_i32(*transaction_id)?;
            if remainder.len() > MAX_CUSTOM_QUERY_ANSWER {
                return Err(WireError::LengthLimit {
                    field: "login custom query answer",
                    length: remainder.len(),
                    maximum: MAX_CUSTOM_QUERY_ANSWER,
                }
                .into());
            }
            writer.write_bytes(remainder)?;
        }
        OptionalLoginServerboundPacket::CookieResponse { key, value } => {
            key.write(&mut writer)?;
            writer.write_bool(value.is_some())?;
            if let Some(value) = value {
                writer.write_byte_array(value, MAX_COOKIE_BYTES)?;
            }
        }
    }
    Ok(writer.into_inner())
}

fn read_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Identifier, OptionalLoginServerboundCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginServerboundOptionalService {
    OnlineAuthentication,
    CustomQuery,
    Cookies,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LoginServerboundGates {
    pub online_authentication: bool,
    pub custom_query: bool,
    pub cookies: bool,
}

impl LoginServerboundGates {
    const fn enabled(self, service: LoginServerboundOptionalService) -> bool {
        match service {
            LoginServerboundOptionalService::OnlineAuthentication => self.online_authentication,
            LoginServerboundOptionalService::CustomQuery => self.custom_query,
            LoginServerboundOptionalService::Cookies => self.cookies,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalLoginServerTask {
    None,
    Key { expected_challenge: Vec<u8> },
    KeyVerificationPending,
    Authenticating,
    CustomQuery { transaction_id: i32 },
    CookieRequest { key: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalLoginServerDecision {
    DecryptAndVerifyKey {
        encrypted_secret: Vec<u8>,
        encrypted_challenge: Vec<u8>,
        expected_challenge: Vec<u8>,
    },
    InstallEncryptionThenAuthenticate,
    CustomQueryAnswer {
        remainder: Vec<u8>,
    },
    CookieResponse {
        value: Option<Vec<u8>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OptionalLoginServerGateError {
    #[error("optional login service {service:?} is disabled")]
    Disabled {
        service: LoginServerboundOptionalService,
    },
    #[error("optional login packet {packet:?} is unsolicited while task is {task:?}")]
    UnexpectedTask {
        packet: OptionalLoginServerboundPacketKind,
        task: OptionalLoginServerTask,
    },
    #[error("cookie response key {actual} does not match requested key {expected}")]
    CookieKeyMismatch {
        expected: Identifier,
        actual: Identifier,
    },
    #[error("verified key callback requires pending key verification, not {task:?}")]
    UnexpectedKeyVerification { task: OptionalLoginServerTask },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginServerboundGate {
    gates: LoginServerboundGates,
    task: OptionalLoginServerTask,
}

impl LoginServerboundGate {
    #[must_use]
    pub const fn new(gates: LoginServerboundGates, task: OptionalLoginServerTask) -> Self {
        Self { gates, task }
    }

    #[must_use]
    pub const fn task(&self) -> &OptionalLoginServerTask {
        &self.task
    }

    pub fn apply(
        &mut self,
        packet: OptionalLoginServerboundPacket,
    ) -> Result<OptionalLoginServerDecision, OptionalLoginServerGateError> {
        let kind = packet.kind();
        let service = kind.service();
        if !self.gates.enabled(service) {
            return Err(OptionalLoginServerGateError::Disabled { service });
        }
        match packet {
            OptionalLoginServerboundPacket::Key {
                encrypted_secret,
                encrypted_challenge,
            } => {
                let OptionalLoginServerTask::Key { expected_challenge } = &self.task else {
                    return self.unexpected(kind);
                };
                let expected_challenge = expected_challenge.clone();
                self.task = OptionalLoginServerTask::KeyVerificationPending;
                Ok(OptionalLoginServerDecision::DecryptAndVerifyKey {
                    encrypted_secret,
                    encrypted_challenge,
                    expected_challenge,
                })
            }
            OptionalLoginServerboundPacket::CustomQueryAnswer {
                transaction_id,
                remainder,
            } => {
                let OptionalLoginServerTask::CustomQuery {
                    transaction_id: expected,
                } = &self.task
                else {
                    return self.unexpected(kind);
                };
                if transaction_id != *expected {
                    return self.unexpected(kind);
                }
                self.task = OptionalLoginServerTask::None;
                Ok(OptionalLoginServerDecision::CustomQueryAnswer { remainder })
            }
            OptionalLoginServerboundPacket::CookieResponse { key, value } => {
                let OptionalLoginServerTask::CookieRequest { key: expected } = &self.task else {
                    return self.unexpected(kind);
                };
                if key != *expected {
                    return Err(OptionalLoginServerGateError::CookieKeyMismatch {
                        expected: expected.clone(),
                        actual: key,
                    });
                }
                self.task = OptionalLoginServerTask::None;
                Ok(OptionalLoginServerDecision::CookieResponse { value })
            }
        }
    }

    pub fn key_verified(
        &mut self,
    ) -> Result<OptionalLoginServerDecision, OptionalLoginServerGateError> {
        if self.task != OptionalLoginServerTask::KeyVerificationPending {
            return Err(OptionalLoginServerGateError::UnexpectedKeyVerification {
                task: self.task.clone(),
            });
        }
        self.task = OptionalLoginServerTask::Authenticating;
        Ok(OptionalLoginServerDecision::InstallEncryptionThenAuthenticate)
    }

    fn unexpected<T>(
        &self,
        packet: OptionalLoginServerboundPacketKind,
    ) -> Result<T, OptionalLoginServerGateError> {
        Err(OptionalLoginServerGateError::UnexpectedTask {
            packet,
            task: self.task.clone(),
        })
    }
}
