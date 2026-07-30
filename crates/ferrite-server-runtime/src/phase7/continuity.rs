use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{
    ActivationGeneration, DimensionId, StableEntityId, StableIdError, WorldId,
};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{SnapshotError, SnapshotRecord, SnapshotRecordKind};
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::phase7::model::{
    EntityLifecycleState, EntityPayload, EntityPayloadError, EntityPersistentState,
    OutboundEntityTransfer,
};
use crate::phase7::transfer::AppliedTransferKey;

const ENTITY_MAGIC: &[u8; 4] = b"F7E1";
const MAX_IDENTITY_BYTES: usize = u16::MAX as usize;

#[must_use]
pub fn entity_domain() -> ResourceId {
    ResourceId::new("ferrite", "phase7/entity_v1").expect("static Phase 7 entity domain is valid")
}

#[must_use]
pub fn receipt_domain() -> ResourceId {
    ResourceId::new("ferrite", "phase7/applied_transfer_v1")
        .expect("static Phase 7 receipt domain is valid")
}

pub fn encode_entity(
    entity: StableEntityId,
    state: &EntityPersistentState,
) -> Result<SnapshotRecord, Phase7ContinuityError> {
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
) -> Result<Option<(StableEntityId, EntityPersistentState)>, Phase7ContinuityError> {
    if record.kind() != SnapshotRecordKind::Entity || record.domain() != &entity_domain() {
        return Ok(None);
    }
    let bytes: [u8; 16] = record
        .key()
        .try_into()
        .map_err(|_| Phase7ContinuityError::InvalidEntityKey)?;
    let entity = StableEntityId::new(u128::from_be_bytes(bytes))?;
    Ok(Some((entity, decode_state(record.value())?)))
}

pub fn encode_receipt(
    receipt: &AppliedTransferKey,
) -> Result<SnapshotRecord, Phase7ContinuityError> {
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
) -> Result<Option<AppliedTransferKey>, Phase7ContinuityError> {
    if record.kind() != SnapshotRecordKind::AppliedBoundary || record.domain() != &receipt_domain()
    {
        return Ok(None);
    }
    if !record.value().is_empty() {
        return Err(Phase7ContinuityError::InvalidReceipt);
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
) -> Result<Vec<u8>, Phase7ContinuityError> {
    encode_state(&EntityPersistentState {
        kind: state.kind.clone(),
        chunk: pending.candidate_chunk,
        revision: pending.candidate_revision,
        last_command_sequence: pending.source_sequence,
        payload: pending.candidate_payload.clone(),
        lifecycle: EntityLifecycleState::Active,
    })
}

pub fn decode_transfer_state(bytes: &[u8]) -> Result<EntityPersistentState, Phase7ContinuityError> {
    let state = decode_state(bytes)?;
    if state.lifecycle != EntityLifecycleState::Active {
        return Err(Phase7ContinuityError::TransferStateNotActive);
    }
    Ok(state)
}

fn encode_state(state: &EntityPersistentState) -> Result<Vec<u8>, Phase7ContinuityError> {
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

fn decode_state(bytes: &[u8]) -> Result<EntityPersistentState, Phase7ContinuityError> {
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
        tag => return Err(Phase7ContinuityError::InvalidLifecycleTag(tag)),
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
) -> Result<(), Phase7ContinuityError> {
    let length = u32::try_from(payload.bytes().len())
        .map_err(|_| Phase7ContinuityError::PayloadLengthOverflow)?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(payload.bytes());
    Ok(())
}

fn encode_identity(
    output: &mut Vec<u8>,
    identity: &ResourceId,
) -> Result<(), Phase7ContinuityError> {
    let value = identity.to_string();
    let length =
        u16::try_from(value.len()).map_err(|_| Phase7ContinuityError::IdentityTooLong {
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
) -> Result<(), Phase7ContinuityError> {
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

    fn expect(&mut self, expected: &[u8]) -> Result<(), Phase7ContinuityError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(Phase7ContinuityError::WrongMagic)
        }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], Phase7ContinuityError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(Phase7ContinuityError::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(Phase7ContinuityError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], Phase7ContinuityError> {
        self.take(N)?
            .try_into()
            .map_err(|_| Phase7ContinuityError::Truncated)
    }

    fn u8(&mut self) -> Result<u8, Phase7ContinuityError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, Phase7ContinuityError> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    fn u64(&mut self) -> Result<u64, Phase7ContinuityError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    fn chunk(&mut self) -> Result<ChunkPos, Phase7ContinuityError> {
        Ok(ChunkPos::new(
            i32::from_be_bytes(self.fixed()?),
            i32::from_be_bytes(self.fixed()?),
        ))
    }

    fn identity(&mut self) -> Result<ResourceId, Phase7ContinuityError> {
        let length = usize::from(self.u16()?);
        std::str::from_utf8(self.take(length)?)
            .map_err(|_| Phase7ContinuityError::InvalidIdentity)?
            .parse()
            .map_err(|_| Phase7ContinuityError::InvalidIdentity)
    }

    fn generation(&mut self) -> Result<ActivationGeneration, Phase7ContinuityError> {
        ActivationGeneration::new(self.u64()?).map_err(|_| Phase7ContinuityError::InvalidGeneration)
    }

    fn stable_id(&mut self) -> Result<StableEntityId, Phase7ContinuityError> {
        StableEntityId::new(u128::from_be_bytes(self.fixed()?)).map_err(Into::into)
    }

    fn payload(&mut self) -> Result<EntityPayload, Phase7ContinuityError> {
        let length = u32::from_be_bytes(self.fixed()?) as usize;
        EntityPayload::new(self.take(length)?.to_vec()).map_err(Into::into)
    }

    fn region(&mut self) -> Result<SimulationRegionKey, Phase7ContinuityError> {
        let world = WorldId::new(u128::from_be_bytes(self.fixed()?))
            .map_err(|_| Phase7ContinuityError::InvalidWorld)?;
        let dimension = DimensionId::new(self.identity()?);
        let coordinate = RegionCoord::new(
            i32::from_be_bytes(self.fixed()?),
            i32::from_be_bytes(self.fixed()?),
        );
        let mapping = RegionMappingVersion::new(self.u16()?)
            .map_err(|_| Phase7ContinuityError::InvalidMapping)?;
        Ok(SimulationRegionKey::new(
            world, dimension, coordinate, mapping,
        ))
    }

    fn finish(self) -> Result<(), Phase7ContinuityError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(Phase7ContinuityError::TrailingBytes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Phase7ContinuityError {
    #[error("Phase 7 entity continuity has the wrong magic")]
    WrongMagic,
    #[error("Phase 7 entity continuity is truncated")]
    Truncated,
    #[error("Phase 7 entity continuity has trailing bytes")]
    TrailingBytes,
    #[error("Phase 7 entity record has an invalid stable key")]
    InvalidEntityKey,
    #[error("Phase 7 continuity contains an invalid resource identity")]
    InvalidIdentity,
    #[error("Phase 7 identity has {actual} bytes, exceeding {maximum}")]
    IdentityTooLong { actual: usize, maximum: usize },
    #[error("Phase 7 payload length exceeds the encoded integer range")]
    PayloadLengthOverflow,
    #[error("Phase 7 lifecycle tag {0} is invalid")]
    InvalidLifecycleTag(u8),
    #[error("Phase 7 transfer state is not active")]
    TransferStateNotActive,
    #[error("Phase 7 transfer receipt has a nonempty value")]
    InvalidReceipt,
    #[error("Phase 7 continuity contains a zero activation generation")]
    InvalidGeneration,
    #[error("Phase 7 continuity contains an invalid world")]
    InvalidWorld,
    #[error("Phase 7 continuity contains an invalid mapping version")]
    InvalidMapping,
    #[error(transparent)]
    StableId(#[from] StableIdError),
    #[error(transparent)]
    Payload(#[from] EntityPayloadError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}
