//! Stable semantic payloads carried by Ferrite's Lattice remoting envelope.

use crate::lattice::remoting::{
    RemoteRegionEnvelope, RemoteRegionEnvelopeHeader, RemoteRegionMessageKind,
};
use crate::transfer::{EntityTransfer, EntityTransferError, EntityTransferHeader, TransferRole};
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use ferrite_simulation::command::{
    CommandError, CommandSource, MAX_COMMAND_PAYLOAD_BYTES, RegionCommand,
};
use thiserror::Error;

const COMMAND_MAGIC: &[u8; 4] = b"FRC1";
const TRANSFER_MAGIC: &[u8; 4] = b"FRT1";
const MAX_RESOURCE_BYTES: usize = 32 * 1024;

pub fn encode_region_command(
    ingress: SimulationRegionKey,
    ingress_generation: ActivationGeneration,
    target_generation: ActivationGeneration,
    command: &RegionCommand,
) -> Result<RemoteRegionEnvelope, SemanticRemotingError> {
    let mut payload = Vec::new();
    payload.extend_from_slice(COMMAND_MAGIC);
    encode_command_source(&mut payload, command.source())?;
    encode_resource(&mut payload, command.kind())?;
    encode_bytes(&mut payload, command.payload(), MAX_COMMAND_PAYLOAD_BYTES)?;
    Ok(RemoteRegionEnvelope::new(
        RemoteRegionEnvelopeHeader {
            kind: RemoteRegionMessageKind::Command,
            tick: command.tick(),
            source: ingress,
            target: command.target().clone(),
            source_generation: ingress_generation,
            target_generation,
            source_sequence: command.sequence(),
        },
        payload,
    )?)
}

pub fn decode_region_command(
    envelope: &RemoteRegionEnvelope,
) -> Result<RegionCommand, SemanticRemotingError> {
    require_kind(envelope, RemoteRegionMessageKind::Command)?;
    let mut decoder = Decoder::new(envelope.payload());
    decoder.magic(COMMAND_MAGIC)?;
    let source = decode_command_source(&mut decoder)?;
    let kind = decode_resource(&mut decoder)?;
    let payload = decoder.bytes(MAX_COMMAND_PAYLOAD_BYTES)?.to_vec();
    decoder.finish()?;
    Ok(RegionCommand::new(
        envelope.target().clone(),
        envelope.tick(),
        source,
        envelope.source_sequence(),
        kind,
        payload,
    )?)
}

pub fn encode_entity_transfer(
    transfer: &EntityTransfer,
) -> Result<RemoteRegionEnvelope, SemanticRemotingError> {
    let mut payload = Vec::new();
    payload.extend_from_slice(TRANSFER_MAGIC);
    payload.extend_from_slice(&transfer.stable_id().to_be_bytes());
    payload.push(match transfer.role() {
        TransferRole::Entity => 0,
        TransferRole::Player => 1,
    });
    encode_resource(&mut payload, transfer.kind())?;
    encode_bytes(
        &mut payload,
        transfer.state(),
        crate::transfer::MAX_ENTITY_TRANSFER_BYTES,
    )?;
    Ok(RemoteRegionEnvelope::new(
        RemoteRegionEnvelopeHeader {
            kind: RemoteRegionMessageKind::EntityTransfer,
            tick: transfer.tick(),
            source: transfer.source().clone(),
            target: transfer.target().clone(),
            source_generation: transfer.source_generation(),
            target_generation: transfer.target_generation(),
            source_sequence: transfer.source_sequence(),
        },
        payload,
    )?)
}

pub fn decode_entity_transfer(
    envelope: &RemoteRegionEnvelope,
) -> Result<EntityTransfer, SemanticRemotingError> {
    require_kind(envelope, RemoteRegionMessageKind::EntityTransfer)?;
    let mut decoder = Decoder::new(envelope.payload());
    decoder.magic(TRANSFER_MAGIC)?;
    let stable_id =
        StableEntityId::new(decoder.u128()?).map_err(|_| SemanticRemotingError::InvalidIdentity)?;
    let role = match decoder.u8()? {
        0 => TransferRole::Entity,
        1 => TransferRole::Player,
        value => return Err(SemanticRemotingError::InvalidTransferRole(value)),
    };
    let kind = decode_resource(&mut decoder)?;
    let state = decoder
        .bytes(crate::transfer::MAX_ENTITY_TRANSFER_BYTES)?
        .to_vec();
    decoder.finish()?;
    Ok(EntityTransfer::new(
        EntityTransferHeader {
            tick: envelope.tick(),
            source: envelope.source().clone(),
            target: envelope.target().clone(),
            source_generation: envelope.source_generation(),
            target_generation: envelope.target_generation(),
            source_sequence: envelope.source_sequence(),
            stable_id,
            role,
        },
        kind,
        state,
    )?)
}

fn require_kind(
    envelope: &RemoteRegionEnvelope,
    expected: RemoteRegionMessageKind,
) -> Result<(), SemanticRemotingError> {
    if envelope.kind() == expected {
        Ok(())
    } else {
        Err(SemanticRemotingError::WrongMessageKind {
            expected,
            actual: envelope.kind(),
        })
    }
}

fn encode_command_source(
    bytes: &mut Vec<u8>,
    source: &CommandSource,
) -> Result<(), SemanticRemotingError> {
    match source {
        CommandSource::System(resource) => {
            bytes.push(0);
            encode_resource(bytes, resource)?;
        }
        CommandSource::Player(player) => {
            bytes.push(1);
            bytes.extend_from_slice(&player.to_be_bytes());
        }
        CommandSource::Region(region) => {
            bytes.push(2);
            encode_region(bytes, region)?;
        }
    }
    Ok(())
}

fn decode_command_source(
    decoder: &mut Decoder<'_>,
) -> Result<CommandSource, SemanticRemotingError> {
    match decoder.u8()? {
        0 => Ok(CommandSource::System(decode_resource(decoder)?)),
        1 => Ok(CommandSource::Player(
            StableEntityId::new(decoder.u128()?)
                .map_err(|_| SemanticRemotingError::InvalidIdentity)?,
        )),
        2 => Ok(CommandSource::Region(decode_region(decoder)?)),
        value => Err(SemanticRemotingError::InvalidCommandSource(value)),
    }
}

fn encode_region(
    bytes: &mut Vec<u8>,
    region: &SimulationRegionKey,
) -> Result<(), SemanticRemotingError> {
    bytes.extend_from_slice(&region.world().to_be_bytes());
    encode_resource(bytes, region.dimension().resource())?;
    bytes.extend_from_slice(&region.coordinate().x().to_be_bytes());
    bytes.extend_from_slice(&region.coordinate().z().to_be_bytes());
    bytes.extend_from_slice(&region.mapping_version().get().to_be_bytes());
    Ok(())
}

fn decode_region(decoder: &mut Decoder<'_>) -> Result<SimulationRegionKey, SemanticRemotingError> {
    let world =
        WorldId::new(decoder.u128()?).map_err(|_| SemanticRemotingError::InvalidIdentity)?;
    let dimension = DimensionId::new(decode_resource(decoder)?);
    let coordinate = RegionCoord::new(decoder.i32()?, decoder.i32()?);
    let mapping_version = RegionMappingVersion::new(decoder.u16()?)
        .map_err(|_| SemanticRemotingError::InvalidIdentity)?;
    Ok(SimulationRegionKey::new(
        world,
        dimension,
        coordinate,
        mapping_version,
    ))
}

fn encode_resource(
    bytes: &mut Vec<u8>,
    resource: &ResourceId,
) -> Result<(), SemanticRemotingError> {
    let value = resource.to_string();
    let length = u16::try_from(value.len()).map_err(|_| SemanticRemotingError::ResourceTooLarge)?;
    if value.len() > MAX_RESOURCE_BYTES {
        return Err(SemanticRemotingError::ResourceTooLarge);
    }
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn decode_resource(decoder: &mut Decoder<'_>) -> Result<ResourceId, SemanticRemotingError> {
    let length = usize::from(decoder.u16()?);
    if length > MAX_RESOURCE_BYTES {
        return Err(SemanticRemotingError::ResourceTooLarge);
    }
    let value = std::str::from_utf8(decoder.take(length)?)
        .map_err(|_| SemanticRemotingError::InvalidIdentity)?;
    Ok(value.parse()?)
}

fn encode_bytes(
    output: &mut Vec<u8>,
    bytes: &[u8],
    maximum: usize,
) -> Result<(), SemanticRemotingError> {
    if bytes.len() > maximum {
        return Err(SemanticRemotingError::PayloadTooLarge {
            actual: bytes.len(),
            maximum,
        });
    }
    let length =
        u32::try_from(bytes.len()).map_err(|_| SemanticRemotingError::PayloadTooLarge {
            actual: bytes.len(),
            maximum,
        })?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(bytes);
    Ok(())
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn magic(&mut self, expected: &[u8]) -> Result<(), SemanticRemotingError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(SemanticRemotingError::WrongMagic)
        }
    }

    fn u8(&mut self) -> Result<u8, SemanticRemotingError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, SemanticRemotingError> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    fn u32(&mut self) -> Result<u32, SemanticRemotingError> {
        Ok(u32::from_be_bytes(self.fixed()?))
    }

    fn i32(&mut self) -> Result<i32, SemanticRemotingError> {
        Ok(i32::from_be_bytes(self.fixed()?))
    }

    fn u128(&mut self) -> Result<u128, SemanticRemotingError> {
        Ok(u128::from_be_bytes(self.fixed()?))
    }

    fn bytes(&mut self, maximum: usize) -> Result<&'a [u8], SemanticRemotingError> {
        let length = self.u32()? as usize;
        if length > maximum {
            return Err(SemanticRemotingError::PayloadTooLarge {
                actual: length,
                maximum,
            });
        }
        self.take(length)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], SemanticRemotingError> {
        self.take(N)?
            .try_into()
            .map_err(|_| SemanticRemotingError::Truncated)
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], SemanticRemotingError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(SemanticRemotingError::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(SemanticRemotingError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn finish(self) -> Result<(), SemanticRemotingError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(SemanticRemotingError::TrailingBytes)
        }
    }
}

#[derive(Debug, Error)]
pub enum SemanticRemotingError {
    #[error(transparent)]
    Remoting(#[from] crate::lattice::remoting::RemotingAdapterError),
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Transfer(#[from] EntityTransferError),
    #[error(transparent)]
    Resource(#[from] ResourceIdError),
    #[error("semantic remoting payload has the wrong magic")]
    WrongMagic,
    #[error("semantic remoting payload is truncated")]
    Truncated,
    #[error("semantic remoting payload has trailing bytes")]
    TrailingBytes,
    #[error("semantic remoting payload contains an invalid identity")]
    InvalidIdentity,
    #[error("semantic remoting resource identity exceeds its bound")]
    ResourceTooLarge,
    #[error("semantic remoting payload has {actual} bytes, exceeding limit {maximum}")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("semantic remoting command-source tag {0} is invalid")]
    InvalidCommandSource(u8),
    #[error("semantic remoting transfer-role tag {0} is invalid")]
    InvalidTransferRole(u8),
    #[error("semantic remoting expected {expected:?}, received {actual:?}")]
    WrongMessageKind {
        expected: RemoteRegionMessageKind,
        actual: RemoteRegionMessageKind,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_simulation::tick::GameTick;

    fn region(x: i32) -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(x, 0),
            RegionMappingVersion::V1,
        )
    }

    #[test]
    fn all_command_sources_round_trip_without_transport_metadata_leaking() {
        let sources = [
            CommandSource::System(ResourceId::new("ferrite", "test").unwrap()),
            CommandSource::Player(StableEntityId::new(7).unwrap()),
            CommandSource::Region(region(1)),
        ];
        for (sequence, source) in sources.into_iter().enumerate() {
            let command = RegionCommand::new(
                region(0),
                GameTick::new(3),
                source,
                sequence as u64,
                ResourceId::new("ferrite", "playable/test").unwrap(),
                vec![1, 2, 3],
            )
            .unwrap();
            let envelope = encode_region_command(
                region(9),
                ActivationGeneration::INITIAL,
                ActivationGeneration::INITIAL,
                &command,
            )
            .unwrap();
            assert_eq!(decode_region_command(&envelope).unwrap(), command);
        }
    }

    #[test]
    fn player_transfer_round_trips_and_malformed_payloads_fail_closed() {
        let transfer = EntityTransfer::new(
            EntityTransferHeader {
                tick: GameTick::new(4),
                source: region(0),
                target: region(1),
                source_generation: ActivationGeneration::INITIAL,
                target_generation: ActivationGeneration::new(2).unwrap(),
                source_sequence: 8,
                stable_id: StableEntityId::new(7).unwrap(),
                role: TransferRole::Player,
            },
            ResourceId::minecraft("player").unwrap(),
            vec![4, 5, 6],
        )
        .unwrap();
        let envelope = encode_entity_transfer(&transfer).unwrap();
        assert_eq!(decode_entity_transfer(&envelope).unwrap(), transfer);

        let mut payload = envelope.payload().to_vec();
        payload.push(0);
        let malformed = RemoteRegionEnvelope::new(
            RemoteRegionEnvelopeHeader {
                kind: envelope.kind(),
                tick: envelope.tick(),
                source: envelope.source().clone(),
                target: envelope.target().clone(),
                source_generation: envelope.source_generation(),
                target_generation: envelope.target_generation(),
                source_sequence: envelope.source_sequence(),
            },
            payload,
        )
        .unwrap();
        assert!(matches!(
            decode_entity_transfer(&malformed),
            Err(SemanticRemotingError::TrailingBytes)
        ));
        assert!(matches!(
            decode_region_command(&envelope),
            Err(SemanticRemotingError::WrongMessageKind { .. })
        ));
    }
}
