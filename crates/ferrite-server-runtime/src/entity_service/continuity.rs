use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{
    ActivationGeneration, DimensionId, StableEntityId, StableIdError, WorldId,
};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{SnapshotError, SnapshotRecord, SnapshotRecordKind};
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::continuity::identity::{ContinuityDomain, ContinuityGeneration, domain_id};
use crate::entity_service::model::{
    EntityLifecycleState, EntityPayload, EntityPayloadError, EntityPersistentState,
    OutboundEntityTransfer,
};
use crate::entity_service::transfer::AppliedTransferKey;

const ENTITY_MAGIC: &[u8; 4] = b"F7E1";
const MAX_IDENTITY_BYTES: usize = u16::MAX as usize;
#[must_use]
pub fn entity_domain() -> ResourceId {
    domain_id(ContinuityDomain::Entity, ContinuityGeneration::Current)
}

#[must_use]
pub fn receipt_domain() -> ResourceId {
    domain_id(
        ContinuityDomain::EntityTransferReceipt,
        ContinuityGeneration::Current,
    )
}

pub fn encode_entity(
    entity: StableEntityId,
    state: &EntityPersistentState,
) -> Result<SnapshotRecord, EntityServiceContinuityError> {
    SnapshotRecord::new(
        SnapshotRecordKind::Entity,
        entity_domain(),
        entity.to_be_bytes().to_vec(),
        encode_state(state)?,
    )
    .map_err(Into::into)
}

pub fn decode_entity(
    record: &SnapshotRecord,
) -> Result<Option<(StableEntityId, EntityPersistentState)>, EntityServiceContinuityError> {
    if record.kind() != SnapshotRecordKind::Entity || record.domain() != &entity_domain() {
        return Ok(None);
    }
    let bytes: [u8; 16] = record
        .key()
        .try_into()
        .map_err(|_| EntityServiceContinuityError::InvalidEntityKey)?;
    let entity = StableEntityId::new(u128::from_be_bytes(bytes))?;
    Ok(Some((entity, decode_state(record.value())?)))
}

pub fn encode_receipt(
    receipt: &AppliedTransferKey,
) -> Result<SnapshotRecord, EntityServiceContinuityError> {
    let mut key = Vec::new();
    key.extend_from_slice(&receipt.tick.get().to_be_bytes());
    encode_region(&mut key, &receipt.source)?;
    key.extend_from_slice(&receipt.source_generation.get().to_be_bytes());
    key.extend_from_slice(&receipt.target_generation.get().to_be_bytes());
    key.extend_from_slice(&receipt.source_sequence.to_be_bytes());
    key.extend_from_slice(&receipt.entity.to_be_bytes());
    SnapshotRecord::new(
        SnapshotRecordKind::AppliedBoundary,
        receipt_domain(),
        key,
        Vec::new(),
    )
    .map_err(Into::into)
}

pub fn decode_receipt(
    record: &SnapshotRecord,
) -> Result<Option<AppliedTransferKey>, EntityServiceContinuityError> {
    if record.kind() != SnapshotRecordKind::AppliedBoundary || record.domain() != &receipt_domain()
    {
        return Ok(None);
    }
    if !record.value().is_empty() {
        return Err(EntityServiceContinuityError::InvalidReceipt);
    }
    let mut cursor = Cursor::new(record.key());
    let key = AppliedTransferKey {
        tick: GameTick::new(cursor.u64()?),
        source: cursor.region()?,
        source_generation: cursor.generation()?,
        target_generation: cursor.generation()?,
        source_sequence: cursor.u64()?,
        entity: cursor.stable_id()?,
    };
    cursor.finish()?;
    Ok(Some(key))
}

pub fn encode_transfer_state(
    state: &EntityPersistentState,
    pending: &OutboundEntityTransfer,
) -> Result<Vec<u8>, EntityServiceContinuityError> {
    encode_state(&EntityPersistentState {
        kind: state.kind.clone(),
        chunk: pending.candidate_chunk,
        revision: pending.candidate_revision,
        last_command_sequence: pending.source_sequence,
        payload: pending.candidate_payload.clone(),
        lifecycle: EntityLifecycleState::Active,
    })
}

pub fn decode_transfer_state(
    bytes: &[u8],
) -> Result<EntityPersistentState, EntityServiceContinuityError> {
    let state = decode_state(bytes)?;
    if state.lifecycle != EntityLifecycleState::Active {
        return Err(EntityServiceContinuityError::TransferStateNotActive);
    }
    Ok(state)
}

fn encode_state(state: &EntityPersistentState) -> Result<Vec<u8>, EntityServiceContinuityError> {
    let mut value = Vec::new();
    value.extend_from_slice(ENTITY_MAGIC);
    encode_identity(&mut value, &state.kind)?;
    encode_chunk(&mut value, state.chunk);
    value.extend_from_slice(&state.revision.to_be_bytes());
    value.extend_from_slice(&state.last_command_sequence.to_be_bytes());
    encode_payload(&mut value, &state.payload)?;
    match &state.lifecycle {
        EntityLifecycleState::Active => value.push(0),
        EntityLifecycleState::Inactive => value.push(1),
        EntityLifecycleState::OutboundPending(pending) => {
            value.push(2);
            value.extend_from_slice(&pending.tick.get().to_be_bytes());
            encode_region(&mut value, &pending.target)?;
            value.extend_from_slice(&pending.target_generation.get().to_be_bytes());
            value.extend_from_slice(&pending.source_sequence.to_be_bytes());
            encode_chunk(&mut value, pending.candidate_chunk);
            value.extend_from_slice(&pending.candidate_revision.to_be_bytes());
            encode_payload(&mut value, &pending.candidate_payload)?;
        }
    }
    Ok(value)
}

fn decode_state(bytes: &[u8]) -> Result<EntityPersistentState, EntityServiceContinuityError> {
    let mut cursor = Cursor::new(bytes);
    cursor.expect(ENTITY_MAGIC)?;
    let kind = cursor.identity()?;
    let chunk = cursor.chunk()?;
    let revision = cursor.u64()?;
    let last_command_sequence = cursor.u64()?;
    let payload = cursor.payload()?;
    let lifecycle = match cursor.u8()? {
        0 => EntityLifecycleState::Active,
        1 => EntityLifecycleState::Inactive,
        2 => EntityLifecycleState::OutboundPending(OutboundEntityTransfer {
            tick: GameTick::new(cursor.u64()?),
            target: cursor.region()?,
            target_generation: cursor.generation()?,
            source_sequence: cursor.u64()?,
            candidate_chunk: cursor.chunk()?,
            candidate_revision: cursor.u64()?,
            candidate_payload: cursor.payload()?,
        }),
        tag => return Err(EntityServiceContinuityError::InvalidLifecycleTag(tag)),
    };
    cursor.finish()?;
    Ok(EntityPersistentState {
        kind,
        chunk,
        revision,
        last_command_sequence,
        payload,
        lifecycle,
    })
}

fn encode_payload(
    output: &mut Vec<u8>,
    payload: &EntityPayload,
) -> Result<(), EntityServiceContinuityError> {
    let length = u32::try_from(payload.bytes().len())
        .map_err(|_| EntityServiceContinuityError::PayloadLengthOverflow)?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(payload.bytes());
    Ok(())
}

fn encode_identity(
    output: &mut Vec<u8>,
    identity: &ResourceId,
) -> Result<(), EntityServiceContinuityError> {
    let value = identity.to_string();
    let length =
        u16::try_from(value.len()).map_err(|_| EntityServiceContinuityError::IdentityTooLong {
            actual: value.len(),
            maximum: MAX_IDENTITY_BYTES,
        })?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_chunk(output: &mut Vec<u8>, chunk: ChunkPos) {
    output.extend_from_slice(&chunk.x.to_be_bytes());
    output.extend_from_slice(&chunk.z.to_be_bytes());
}

fn encode_region(
    output: &mut Vec<u8>,
    region: &SimulationRegionKey,
) -> Result<(), EntityServiceContinuityError> {
    output.extend_from_slice(&region.world().get().to_be_bytes());
    encode_identity(output, region.dimension().resource())?;
    output.extend_from_slice(&region.coordinate().x().to_be_bytes());
    output.extend_from_slice(&region.coordinate().z().to_be_bytes());
    output.extend_from_slice(&region.mapping_version().get().to_be_bytes());
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn expect(&mut self, expected: &[u8]) -> Result<(), EntityServiceContinuityError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(EntityServiceContinuityError::WrongMagic)
        }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], EntityServiceContinuityError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(EntityServiceContinuityError::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(EntityServiceContinuityError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], EntityServiceContinuityError> {
        self.take(N)?
            .try_into()
            .map_err(|_| EntityServiceContinuityError::Truncated)
    }

    fn u8(&mut self) -> Result<u8, EntityServiceContinuityError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, EntityServiceContinuityError> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    fn u64(&mut self) -> Result<u64, EntityServiceContinuityError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    fn chunk(&mut self) -> Result<ChunkPos, EntityServiceContinuityError> {
        Ok(ChunkPos::new(
            i32::from_be_bytes(self.fixed()?),
            i32::from_be_bytes(self.fixed()?),
        ))
    }

    fn identity(&mut self) -> Result<ResourceId, EntityServiceContinuityError> {
        let length = usize::from(self.u16()?);
        std::str::from_utf8(self.take(length)?)
            .map_err(|_| EntityServiceContinuityError::InvalidIdentity)?
            .parse()
            .map_err(|_| EntityServiceContinuityError::InvalidIdentity)
    }

    fn generation(&mut self) -> Result<ActivationGeneration, EntityServiceContinuityError> {
        ActivationGeneration::new(self.u64()?)
            .map_err(|_| EntityServiceContinuityError::InvalidGeneration)
    }

    fn stable_id(&mut self) -> Result<StableEntityId, EntityServiceContinuityError> {
        StableEntityId::new(u128::from_be_bytes(self.fixed()?)).map_err(Into::into)
    }

    fn payload(&mut self) -> Result<EntityPayload, EntityServiceContinuityError> {
        let length = u32::from_be_bytes(self.fixed()?) as usize;
        EntityPayload::new(self.take(length)?.to_vec()).map_err(Into::into)
    }

    fn region(&mut self) -> Result<SimulationRegionKey, EntityServiceContinuityError> {
        let world = WorldId::new(u128::from_be_bytes(self.fixed()?))
            .map_err(|_| EntityServiceContinuityError::InvalidWorld)?;
        let dimension = DimensionId::new(self.identity()?);
        let coordinate = RegionCoord::new(
            i32::from_be_bytes(self.fixed()?),
            i32::from_be_bytes(self.fixed()?),
        );
        let mapping = RegionMappingVersion::new(self.u16()?)
            .map_err(|_| EntityServiceContinuityError::InvalidMapping)?;
        Ok(SimulationRegionKey::new(
            world, dimension, coordinate, mapping,
        ))
    }

    fn finish(self) -> Result<(), EntityServiceContinuityError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(EntityServiceContinuityError::TrailingBytes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EntityServiceContinuityError {
    #[error("entity-service entity continuity has the wrong magic")]
    WrongMagic,
    #[error("entity-service entity continuity is truncated")]
    Truncated,
    #[error("entity-service entity continuity has trailing bytes")]
    TrailingBytes,
    #[error("entity-service entity record has an invalid stable key")]
    InvalidEntityKey,
    #[error("entity-service continuity contains an invalid resource identity")]
    InvalidIdentity,
    #[error("entity-service identity has {actual} bytes, exceeding {maximum}")]
    IdentityTooLong { actual: usize, maximum: usize },
    #[error("entity-service payload length exceeds the encoded integer range")]
    PayloadLengthOverflow,
    #[error("entity-service lifecycle tag {0} is invalid")]
    InvalidLifecycleTag(u8),
    #[error("entity-service transfer state is not active")]
    TransferStateNotActive,
    #[error("entity-service transfer receipt has a nonempty value")]
    InvalidReceipt,
    #[error("entity-service continuity contains a zero activation generation")]
    InvalidGeneration,
    #[error("entity-service continuity contains an invalid world")]
    InvalidWorld,
    #[error("entity-service continuity contains an invalid mapping version")]
    InvalidMapping,
    #[error(transparent)]
    StableId(#[from] StableIdError),
    #[error(transparent)]
    Payload(#[from] EntityPayloadError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}
