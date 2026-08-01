use std::collections::BTreeMap;

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{
    JournalTailFrame, RegionRecoveryPoint, SnapshotError, SnapshotRecord, SnapshotRecordKind,
};
use ferrite_world::chunk::ChunkColumn;
use ferrite_world::durable::{DurableChunkError, decode_chunk, encode_chunk};
use ferrite_world::generation::status::ChunkStatus;
use thiserror::Error;

use crate::continuity::identity::{ContinuityDomain, ContinuityGeneration, domain_id};
use crate::continuity::migration::{ContinuityMigrationError, canonical_record_hash};
use crate::world_service::model::{
    ChunkActivity, ChunkLifecycle, GENERATION_CONTINUATION_VERSION_V1, PendingGeneration,
    PendingUnload,
};

const LIFECYCLE_MAGIC_V1: [u8; 4] = *b"P8C1";
const LIFECYCLE_MAGIC_V2: [u8; 4] = *b"P8C2";
#[must_use]
pub fn chunk_domain() -> ResourceId {
    domain_id(ContinuityDomain::WorldChunk, ContinuityGeneration::Current)
}

pub fn encode_chunk_record(
    chunk: &ChunkColumn,
    lifecycle: ChunkLifecycle,
) -> Result<SnapshotRecord, WorldServiceContinuityError> {
    let mut value = Vec::new();
    value.extend_from_slice(&LIFECYCLE_MAGIC_V2);
    value.push(lifecycle.status as u8);
    value.push(lifecycle.activity as u8);
    match lifecycle.pending_generation {
        None => value.push(0),
        Some(pending) => {
            if pending.continuation_version != GENERATION_CONTINUATION_VERSION_V1 {
                return Err(WorldServiceContinuityError::UnsupportedContinuationVersion(
                    pending.continuation_version,
                ));
            }
            value.push(1);
            value.extend_from_slice(&pending.continuation_version.to_be_bytes());
            value.extend_from_slice(&pending.request_id.to_be_bytes());
            value.extend_from_slice(&pending.expected_revision.to_be_bytes());
            value.push(pending.target_status as u8);
            value.extend_from_slice(&pending.content_manifest);
        }
    }
    match lifecycle.pending_unload {
        None => value.push(0),
        Some(pending) => {
            value.push(1);
            value.extend_from_slice(&pending.token.to_be_bytes());
            value.extend_from_slice(&pending.expected_revision.to_be_bytes());
        }
    }
    let encoded_chunk = encode_chunk(chunk)?;
    value.extend_from_slice(&encoded_chunk);
    SnapshotRecord::new(
        SnapshotRecordKind::Chunk,
        chunk_domain(),
        encode_chunk_key(chunk.position()).to_vec(),
        value,
    )
    .map_err(Into::into)
}

pub fn decode_chunk_record(
    record: &SnapshotRecord,
) -> Result<Option<(ChunkColumn, ChunkLifecycle)>, WorldServiceContinuityError> {
    if record.kind() != SnapshotRecordKind::Chunk || record.domain() != &chunk_domain() {
        return Ok(None);
    }
    let position = decode_chunk_key(record.key())?;
    let mut cursor = Cursor::new(record.value());
    let has_generation_continuation = match cursor.fixed::<4>()? {
        LIFECYCLE_MAGIC_V1 => false,
        LIFECYCLE_MAGIC_V2 => true,
        _ => return Err(WorldServiceContinuityError::WrongMagic),
    };
    let status = decode_status(cursor.u8()?)?;
    let activity = decode_activity(cursor.u8()?)?;
    let pending_generation = if has_generation_continuation {
        match cursor.u8()? {
            0 => None,
            1 => {
                let continuation_version = cursor.u16()?;
                if continuation_version != GENERATION_CONTINUATION_VERSION_V1 {
                    return Err(WorldServiceContinuityError::UnsupportedContinuationVersion(
                        continuation_version,
                    ));
                }
                Some(PendingGeneration {
                    continuation_version,
                    request_id: cursor.u64()?,
                    expected_revision: cursor.u64()?,
                    target_status: decode_status(cursor.u8()?)?,
                    content_manifest: cursor.fixed()?,
                })
            }
            tag => return Err(WorldServiceContinuityError::InvalidGenerationTag(tag)),
        }
    } else {
        None
    };
    let pending_unload = match cursor.u8()? {
        0 => None,
        1 => Some(PendingUnload {
            token: cursor.u64()?,
            expected_revision: cursor.u64()?,
        }),
        tag => return Err(WorldServiceContinuityError::InvalidPendingTag(tag)),
    };
    let chunk = decode_chunk(cursor.remaining())?;
    if chunk.position() != position {
        return Err(WorldServiceContinuityError::ChunkKeyMismatch);
    }
    if pending_generation.is_some_and(|pending| {
        pending.request_id == 0
            || pending_unload.is_some()
            || pending.expected_revision != chunk.revision().get()
            || ChunkStatus::ALL.get(status as usize + 1).copied() != Some(pending.target_status)
    }) {
        return Err(WorldServiceContinuityError::InvalidGenerationContinuation);
    }
    Ok(Some((
        chunk,
        ChunkLifecycle {
            status,
            activity,
            pending_generation,
            pending_unload,
        },
    )))
}

pub fn materialized_records(point: &RegionRecoveryPoint) -> Vec<SnapshotRecord> {
    let mut records = BTreeMap::new();
    overlay_records(&mut records, point.snapshot().records());
    for frame in point.journal_tail() {
        overlay_frame(&mut records, frame);
    }
    records.into_values().collect()
}

fn overlay_frame(
    records: &mut BTreeMap<(SnapshotRecordKind, ResourceId, Vec<u8>), SnapshotRecord>,
    frame: &JournalTailFrame,
) {
    overlay_records(records, frame.records());
}

fn overlay_records(
    records: &mut BTreeMap<(SnapshotRecordKind, ResourceId, Vec<u8>), SnapshotRecord>,
    additions: &[SnapshotRecord],
) {
    for record in additions {
        records.insert(
            (
                record.kind(),
                record.domain().clone(),
                record.key().to_vec(),
            ),
            record.clone(),
        );
    }
}

pub fn canonical_state_hash(records: &[SnapshotRecord]) -> [u8; 32] {
    canonical_record_hash(records)
}

fn encode_chunk_key(position: ChunkPos) -> [u8; 8] {
    let mut key = [0; 8];
    key[..4].copy_from_slice(&position.x.to_be_bytes());
    key[4..].copy_from_slice(&position.z.to_be_bytes());
    key
}

fn decode_chunk_key(bytes: &[u8]) -> Result<ChunkPos, WorldServiceContinuityError> {
    let bytes: [u8; 8] = bytes
        .try_into()
        .map_err(|_| WorldServiceContinuityError::InvalidChunkKey)?;
    Ok(ChunkPos::new(
        i32::from_be_bytes(bytes[..4].try_into().expect("fixed slice")),
        i32::from_be_bytes(bytes[4..].try_into().expect("fixed slice")),
    ))
}

fn decode_status(tag: u8) -> Result<ChunkStatus, WorldServiceContinuityError> {
    ChunkStatus::ALL
        .get(usize::from(tag))
        .copied()
        .ok_or(WorldServiceContinuityError::InvalidStatus(tag))
}

fn decode_activity(tag: u8) -> Result<ChunkActivity, WorldServiceContinuityError> {
    match tag {
        0 => Ok(ChunkActivity::Dormant),
        1 => Ok(ChunkActivity::Accessible),
        2 => Ok(ChunkActivity::BlockTicking),
        3 => Ok(ChunkActivity::EntityTicking),
        _ => Err(WorldServiceContinuityError::InvalidActivity(tag)),
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], WorldServiceContinuityError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(WorldServiceContinuityError::Truncated)?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or(WorldServiceContinuityError::Truncated)?;
        self.offset = end;
        Ok(bytes)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], WorldServiceContinuityError> {
        self.take(N)?
            .try_into()
            .map_err(|_| WorldServiceContinuityError::Truncated)
    }

    fn u8(&mut self) -> Result<u8, WorldServiceContinuityError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, WorldServiceContinuityError> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    fn u64(&mut self) -> Result<u64, WorldServiceContinuityError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.offset..]
    }
}

#[derive(Debug, Error)]
pub enum WorldServiceContinuityError {
    #[error("world-service chunk continuity has the wrong magic")]
    WrongMagic,
    #[error("world-service chunk continuity is truncated")]
    Truncated,
    #[error("world-service chunk key is invalid")]
    InvalidChunkKey,
    #[error("world-service chunk key does not match the encoded column")]
    ChunkKeyMismatch,
    #[error("world-service status tag {0} is invalid")]
    InvalidStatus(u8),
    #[error("world-service activity tag {0} is invalid")]
    InvalidActivity(u8),
    #[error("world-service pending-unload tag {0} is invalid")]
    InvalidPendingTag(u8),
    #[error("world-service pending-generation tag {0} is invalid")]
    InvalidGenerationTag(u8),
    #[error("world-service generation continuation version {0} is unsupported")]
    UnsupportedContinuationVersion(u16),
    #[error("world-service generation continuation is inconsistent with its durable chunk")]
    InvalidGenerationContinuation,
    #[error(transparent)]
    Chunk(#[from] DurableChunkError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Migration(#[from] ContinuityMigrationError),
}

#[cfg(test)]
mod tests {
    use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
    use ferrite_world::id::{BiomeId, BlockStateId};

    use super::*;

    fn chunk() -> ChunkColumn {
        ChunkColumn::new(
            ChunkPos::new(0, 0),
            ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).unwrap(),
                BlockStateId::new(0),
                BiomeId::new(0),
            ),
        )
    }

    #[test]
    fn current_lifecycle_round_trips_generation_continuation() {
        let lifecycle = ChunkLifecycle {
            status: ChunkStatus::StructureStarts,
            activity: ChunkActivity::Dormant,
            pending_generation: Some(PendingGeneration {
                continuation_version: GENERATION_CONTINUATION_VERSION_V1,
                request_id: 7,
                expected_revision: 0,
                target_status: ChunkStatus::StructureReferences,
                content_manifest: [5; 32],
            }),
            pending_unload: None,
        };
        let record = encode_chunk_record(&chunk(), lifecycle).unwrap();
        let (_, decoded) = decode_chunk_record(&record).unwrap().unwrap();
        assert_eq!(decoded, lifecycle);
    }

    #[test]
    fn legacy_lifecycle_decodes_without_synthesizing_a_continuation() {
        let chunk = chunk();
        let mut value = Vec::new();
        value.extend_from_slice(&LIFECYCLE_MAGIC_V1);
        value.push(ChunkStatus::Empty as u8);
        value.push(ChunkActivity::Dormant as u8);
        value.push(0);
        value.extend_from_slice(&encode_chunk(&chunk).unwrap());
        let record = SnapshotRecord::new(
            SnapshotRecordKind::Chunk,
            chunk_domain(),
            encode_chunk_key(chunk.position()).to_vec(),
            value,
        )
        .unwrap();
        let (_, lifecycle) = decode_chunk_record(&record).unwrap().unwrap();
        assert_eq!(lifecycle, ChunkLifecycle::empty());
    }
}
