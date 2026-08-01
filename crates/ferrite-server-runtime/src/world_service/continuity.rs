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

use crate::world_service::model::{ChunkActivity, ChunkLifecycle, PendingUnload};

const LIFECYCLE_MAGIC: &[u8; 4] = b"P8C1";
// This Goal 01 identity is persisted. G03-P1-B3 owns its versioned migration.
const LEGACY_CHUNK_DOMAIN: &str = "phase8/chunk_v1";

#[must_use]
pub fn chunk_domain() -> ResourceId {
    ResourceId::new("ferrite", LEGACY_CHUNK_DOMAIN)
        .expect("static legacy world chunk domain is valid")
}

pub fn encode_chunk_record(
    chunk: &ChunkColumn,
    lifecycle: ChunkLifecycle,
) -> Result<SnapshotRecord, WorldServiceContinuityError> {
    if lifecycle.pending_generation.is_some() {
        return Err(WorldServiceContinuityError::GenerationInFlight);
    }
    let mut value = Vec::new();
    value.extend_from_slice(LIFECYCLE_MAGIC);
    value.push(lifecycle.status as u8);
    value.push(lifecycle.activity as u8);
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
    cursor.expect(LIFECYCLE_MAGIC)?;
    let status = decode_status(cursor.u8()?)?;
    let activity = decode_activity(cursor.u8()?)?;
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
    Ok(Some((
        chunk,
        ChunkLifecycle {
            status,
            activity,
            pending_generation: None,
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
    let mut ordered = records.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        (left.kind(), left.domain(), left.key()).cmp(&(right.kind(), right.domain(), right.key()))
    });
    let mut hasher = blake3::Hasher::new();
    for record in ordered {
        hasher.update(&[record.kind() as u8]);
        hash_bytes(&mut hasher, record.domain().to_string().as_bytes());
        hash_bytes(&mut hasher, record.key());
        hash_bytes(&mut hasher, record.value());
    }
    *hasher.finalize().as_bytes()
}

fn hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
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

    fn expect(&mut self, expected: &[u8]) -> Result<(), WorldServiceContinuityError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(WorldServiceContinuityError::WrongMagic)
        }
    }

    fn u8(&mut self) -> Result<u8, WorldServiceContinuityError> {
        Ok(self.take(1)?[0])
    }

    fn u64(&mut self) -> Result<u64, WorldServiceContinuityError> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| WorldServiceContinuityError::Truncated)?,
        ))
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
    #[error("world-service save cannot capture an in-flight generation task")]
    GenerationInFlight,
    #[error(transparent)]
    Chunk(#[from] DurableChunkError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}
