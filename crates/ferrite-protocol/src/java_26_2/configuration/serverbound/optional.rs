use thiserror::Error;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection, PacketIdError};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_COOKIE_BYTES: usize = 5_120;
const MAX_CUSTOM_CLICK_PREFIX_BYTES: usize = 65_536;
const MAX_CUSTOM_CLICK_NBT_BYTES: u64 = 32_768;
const MAX_CUSTOM_CLICK_NBT_DEPTH: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalConfigurationPacketKind {
    CookieResponse,
    ResourcePack,
    CustomClickAction,
    AcceptCodeOfConduct,
}

impl OptionalConfigurationPacketKind {
    pub const ALL: [Self; 4] = [
        Self::CookieResponse,
        Self::ResourcePack,
        Self::CustomClickAction,
        Self::AcceptCodeOfConduct,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::CookieResponse => 1,
            Self::ResourcePack => 6,
            Self::CustomClickAction => 8,
            Self::AcceptCodeOfConduct => 9,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::CookieResponse => "minecraft:cookie_response",
            Self::ResourcePack => "minecraft:resource_pack",
            Self::CustomClickAction => "minecraft:custom_click_action",
            Self::AcceptCodeOfConduct => "minecraft:accept_code_of_conduct",
        }
    }

    #[must_use]
    pub const fn service(self) -> ConfigurationServerboundOptionalService {
        match self {
            Self::CookieResponse => ConfigurationServerboundOptionalService::Cookies,
            Self::ResourcePack => ConfigurationServerboundOptionalService::ResourcePacks,
            Self::CustomClickAction => ConfigurationServerboundOptionalService::CustomClick,
            Self::AcceptCodeOfConduct => ConfigurationServerboundOptionalService::CodeOfConduct,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalConfigurationPacket {
    CookieResponse {
        key: Identifier,
        value: Option<Vec<u8>>,
    },
    ResourcePack {
        pack_id: u128,
        action: ResourcePackAction,
    },
    CustomClickAction {
        action: Identifier,
        payload: Option<NetworkNbt>,
    },
    AcceptCodeOfConduct,
}

impl OptionalConfigurationPacket {
    #[must_use]
    pub const fn kind(&self) -> OptionalConfigurationPacketKind {
        match self {
            Self::CookieResponse { .. } => OptionalConfigurationPacketKind::CookieResponse,
            Self::ResourcePack { .. } => OptionalConfigurationPacketKind::ResourcePack,
            Self::CustomClickAction { .. } => OptionalConfigurationPacketKind::CustomClickAction,
            Self::AcceptCodeOfConduct => OptionalConfigurationPacketKind::AcceptCodeOfConduct,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourcePackAction {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
    Downloaded,
    InvalidUrl,
    FailedReload,
    Discarded,
}

impl ResourcePackAction {
    #[must_use]
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::SuccessfullyLoaded => 0,
            Self::Declined => 1,
            Self::FailedDownload => 2,
            Self::Accepted => 3,
            Self::Downloaded => 4,
            Self::InvalidUrl => 5,
            Self::FailedReload => 6,
            Self::Discarded => 7,
        }
    }

    #[must_use]
    pub const fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::SuccessfullyLoaded),
            1 => Some(Self::Declined),
            2 => Some(Self::FailedDownload),
            3 => Some(Self::Accepted),
            4 => Some(Self::Downloaded),
            5 => Some(Self::InvalidUrl),
            6 => Some(Self::FailedReload),
            7 => Some(Self::Discarded),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::Accepted | Self::Downloaded)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OptionalConfigurationCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error(transparent)]
    InvalidPacketId(#[from] PacketIdError),
    #[error("configuration serverbound packet ID {id} is absent from the locked catalog")]
    UnknownPacketId { id: i32 },
    #[error("configuration serverbound packet {identity} is not part of the optional C4 family")]
    RequiredPacketIdentity { identity: &'static str },
    #[error("resource-pack action ordinal {ordinal} is outside 0..=7")]
    InvalidResourcePackAction { ordinal: i32 },
    #[error("locked catalog is missing optional packet identity {identity}")]
    MissingCatalogIdentity { identity: &'static str },
}

pub fn decode_packet(
    body: &[u8],
) -> Result<OptionalConfigurationPacket, OptionalConfigurationCodecError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Configuration,
        PacketDirection::Serverbound,
        wire_id,
    )
    .ok_or(OptionalConfigurationCodecError::UnknownPacketId { id: wire_id })?;
    let packet = match descriptor.identity() {
        "minecraft:cookie_response" => decode_cookie_response(&mut reader)?,
        "minecraft:resource_pack" => decode_resource_pack(&mut reader)?,
        "minecraft:custom_click_action" => decode_custom_click(&mut reader)?,
        "minecraft:accept_code_of_conduct" => OptionalConfigurationPacket::AcceptCodeOfConduct,
        identity => {
            return Err(OptionalConfigurationCodecError::RequiredPacketIdentity { identity });
        }
    };
    reader.finish()?;
    Ok(packet)
}

pub fn encode_packet(
    packet: &OptionalConfigurationPacket,
) -> Result<Vec<u8>, OptionalConfigurationCodecError> {
    let identity = packet.kind().identity();
    let descriptor = PacketCatalog::by_identity(
        ConnectionState::Configuration,
        PacketDirection::Serverbound,
        identity,
    )
    .ok_or(OptionalConfigurationCodecError::MissingCatalogIdentity { identity })?;
    let mut writer = WireWriter::new(MAX_INFLATED_PACKET_LENGTH);
    writer.write_var_i32(descriptor.id().into())?;
    match packet {
        OptionalConfigurationPacket::CookieResponse { key, value } => {
            key.write(&mut writer)?;
            writer.write_bool(value.is_some())?;
            if let Some(value) = value {
                writer.write_byte_array(value, MAX_COOKIE_BYTES)?;
            }
        }
        OptionalConfigurationPacket::ResourcePack { pack_id, action } => {
            writer.write_u128(*pack_id)?;
            writer.write_var_i32(action.ordinal())?;
        }
        OptionalConfigurationPacket::CustomClickAction { action, payload } => {
            action.write(&mut writer)?;
            let mut payload_writer = WireWriter::new(MAX_CUSTOM_CLICK_PREFIX_BYTES);
            NetworkNbt::write_nullable(payload.as_ref(), &mut payload_writer)?;
            writer.write_byte_array(&payload_writer.into_inner(), MAX_CUSTOM_CLICK_PREFIX_BYTES)?;
        }
        OptionalConfigurationPacket::AcceptCodeOfConduct => {}
    }
    Ok(writer.into_inner())
}

fn decode_cookie_response(
    reader: &mut WireReader<'_>,
) -> Result<OptionalConfigurationPacket, OptionalConfigurationCodecError> {
    let key = read_identifier(reader)?;
    let value = if reader.read_bool()? {
        Some(reader.read_byte_array(MAX_COOKIE_BYTES)?.to_vec())
    } else {
        None
    };
    Ok(OptionalConfigurationPacket::CookieResponse { key, value })
}

fn decode_resource_pack(
    reader: &mut WireReader<'_>,
) -> Result<OptionalConfigurationPacket, OptionalConfigurationCodecError> {
    let pack_id = reader.read_u128()?;
    let ordinal = reader.read_var_i32()?;
    let action = ResourcePackAction::from_ordinal(ordinal)
        .ok_or(OptionalConfigurationCodecError::InvalidResourcePackAction { ordinal })?;
    Ok(OptionalConfigurationPacket::ResourcePack { pack_id, action })
}

fn decode_custom_click(
    reader: &mut WireReader<'_>,
) -> Result<OptionalConfigurationPacket, OptionalConfigurationCodecError> {
    let action = read_identifier(reader)?;
    let bytes = reader.read_byte_array(MAX_CUSTOM_CLICK_PREFIX_BYTES)?;
    let mut payload_reader = WireReader::new(bytes);
    let payload = NetworkNbt::read_nullable(
        &mut payload_reader,
        NbtQuota::Bounded {
            maximum_bytes: MAX_CUSTOM_CLICK_NBT_BYTES,
            maximum_depth: MAX_CUSTOM_CLICK_NBT_DEPTH,
        },
    )?;
    payload_reader.finish()?;
    Ok(OptionalConfigurationPacket::CustomClickAction { action, payload })
}

fn read_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Identifier, OptionalConfigurationCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationServerboundOptionalService {
    Cookies,
    ResourcePacks,
    CustomClick,
    CodeOfConduct,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigurationServerboundGates {
    pub cookies: bool,
    pub resource_packs: bool,
    pub custom_click: bool,
    pub code_of_conduct: bool,
}

impl ConfigurationServerboundGates {
    const fn enabled(self, service: ConfigurationServerboundOptionalService) -> bool {
        match service {
            ConfigurationServerboundOptionalService::Cookies => self.cookies,
            ConfigurationServerboundOptionalService::ResourcePacks => self.resource_packs,
            ConfigurationServerboundOptionalService::CustomClick => self.custom_click,
            ConfigurationServerboundOptionalService::CodeOfConduct => self.code_of_conduct,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalConfigurationTask {
    None,
    CookieRequest { key: Identifier },
    ResourcePack { required: bool },
    CodeOfConduct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalServerboundDecision {
    CookieResponse {
        value: Option<Vec<u8>>,
    },
    AwaitResourcePackTerminal {
        action: ResourcePackAction,
    },
    ResourcePackCompleted {
        pack_id: u128,
        action: ResourcePackAction,
    },
    DisconnectRequiredPackDeclined {
        pack_id: u128,
    },
    DispatchCustomClick {
        action: Identifier,
        payload: Option<NetworkNbt>,
    },
    CodeOfConductAccepted,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OptionalConfigurationGateError {
    #[error("optional configuration service {service:?} is disabled")]
    Disabled {
        service: ConfigurationServerboundOptionalService,
    },
    #[error("optional packet {packet:?} is unsolicited while task is {task:?}")]
    UnexpectedTask {
        packet: OptionalConfigurationPacketKind,
        task: OptionalConfigurationTask,
    },
    #[error("cookie response key {actual} does not match requested key {expected}")]
    CookieKeyMismatch {
        expected: Identifier,
        actual: Identifier,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationServerboundGate {
    gates: ConfigurationServerboundGates,
    task: OptionalConfigurationTask,
}

impl ConfigurationServerboundGate {
    #[must_use]
    pub const fn new(
        gates: ConfigurationServerboundGates,
        task: OptionalConfigurationTask,
    ) -> Self {
        Self { gates, task }
    }

    #[must_use]
    pub const fn task(&self) -> &OptionalConfigurationTask {
        &self.task
    }

    pub fn apply(
        &mut self,
        packet: OptionalConfigurationPacket,
    ) -> Result<OptionalServerboundDecision, OptionalConfigurationGateError> {
        let kind = packet.kind();
        let service = kind.service();
        if !self.gates.enabled(service) {
            return Err(OptionalConfigurationGateError::Disabled { service });
        }
        match packet {
            OptionalConfigurationPacket::CookieResponse { key, value } => {
                let OptionalConfigurationTask::CookieRequest { key: expected } = &self.task else {
                    return self.unexpected(kind);
                };
                if key != *expected {
                    return Err(OptionalConfigurationGateError::CookieKeyMismatch {
                        expected: expected.clone(),
                        actual: key,
                    });
                }
                self.task = OptionalConfigurationTask::None;
                Ok(OptionalServerboundDecision::CookieResponse { value })
            }
            OptionalConfigurationPacket::ResourcePack { pack_id, action } => {
                let OptionalConfigurationTask::ResourcePack { required } = &self.task else {
                    return self.unexpected(kind);
                };
                let required = *required;
                if !action.is_terminal() {
                    return Ok(OptionalServerboundDecision::AwaitResourcePackTerminal { action });
                }
                self.task = OptionalConfigurationTask::None;
                if required && action == ResourcePackAction::Declined {
                    Ok(OptionalServerboundDecision::DisconnectRequiredPackDeclined { pack_id })
                } else {
                    Ok(OptionalServerboundDecision::ResourcePackCompleted { pack_id, action })
                }
            }
            OptionalConfigurationPacket::CustomClickAction { action, payload } => {
                Ok(OptionalServerboundDecision::DispatchCustomClick { action, payload })
            }
            OptionalConfigurationPacket::AcceptCodeOfConduct => {
                if self.task != OptionalConfigurationTask::CodeOfConduct {
                    return self.unexpected(kind);
                }
                self.task = OptionalConfigurationTask::None;
                Ok(OptionalServerboundDecision::CodeOfConductAccepted)
            }
        }
    }

    fn unexpected<T>(
        &self,
        packet: OptionalConfigurationPacketKind,
    ) -> Result<T, OptionalConfigurationGateError> {
        Err(OptionalConfigurationGateError::UnexpectedTask {
            packet,
            task: self.task.clone(),
        })
    }
}
