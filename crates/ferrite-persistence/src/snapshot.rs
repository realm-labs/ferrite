//! Versioned stable Region recovery points.

use crate::codec::{CodecError, Decoder, Encoder};
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use std::num::NonZeroU64;
use thiserror::Error;

const SNAPSHOT_MAGIC: &[u8; 4] = b"FRSN";
const SNAPSHOT_SCHEMA_V1: u16 = 1;
const MAX_RESOURCE_ID_BYTES: usize = 32 * 1024;
pub const MAX_SNAPSHOT_KEY_BYTES: usize = 64 * 1024;
pub const MAX_SNAPSHOT_VALUE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_SNAPSHOT_RECORDS: usize = 1024 * 1024;
pub const MAX_JOURNAL_TAIL_FRAMES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersistenceRevision(NonZeroU64);

impl PersistenceRevision {
    pub const INITIAL: Self = Self(NonZeroU64::MIN);

    pub const fn new(value: u64) -> Result<Self, SnapshotError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(SnapshotError::ZeroPersistenceRevision),
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }

    pub const fn checked_next(self) -> Result<Self, SnapshotError> {
        match self.get().checked_add(1) {
            Some(value) => Self::new(value),
            None => Err(SnapshotError::PersistenceRevisionExhausted),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SnapshotRecordKind {
    Chunk = 0,
    Entity = 1,
    ScheduledWork = 2,
    RandomStream = 3,
    AppliedBoundary = 4,
    Extension = 5,
}

impl SnapshotRecordKind {
    fn from_tag(tag: u8) -> Result<Self, SnapshotError> {
        match tag {
            0 => Ok(Self::Chunk),
            1 => Ok(Self::Entity),
            2 => Ok(Self::ScheduledWork),
            3 => Ok(Self::RandomStream),
            4 => Ok(Self::AppliedBoundary),
            5 => Ok(Self::Extension),
            _ => Err(SnapshotError::UnknownRecordKind(tag)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRecord {
    kind: SnapshotRecordKind,
    domain: ResourceId,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl SnapshotRecord {
    pub fn new(
        kind: SnapshotRecordKind,
        domain: ResourceId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Self, SnapshotError> {
        enforce_length("snapshot key", key.len(), MAX_SNAPSHOT_KEY_BYTES)?;
        enforce_length("snapshot value", value.len(), MAX_SNAPSHOT_VALUE_BYTES)?;
        Ok(Self {
            kind,
            domain,
            key,
            value,
        })
    }

    pub const fn kind(&self) -> SnapshotRecordKind {
        self.kind
    }

    pub const fn domain(&self) -> &ResourceId {
        &self.domain
    }

    pub fn key(&self) -> &[u8] {
        &self.key
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionSnapshotHeader {
    pub key: SimulationRegionKey,
    pub generation: ActivationGeneration,
    pub committed_tick: u64,
    pub persistence_revision: PersistenceRevision,
    pub region_side_chunks: u16,
    pub content_manifest: [u8; 32],
    pub state_hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionCommitSnapshot {
    header: RegionSnapshotHeader,
    records: Box<[SnapshotRecord]>,
}

impl RegionCommitSnapshot {
    pub fn new(
        header: RegionSnapshotHeader,
        mut records: Vec<SnapshotRecord>,
    ) -> Result<Self, SnapshotError> {
        if header.region_side_chunks == 0 {
            return Err(SnapshotError::ZeroRegionSide);
        }
        sort_and_validate_records(&mut records)?;
        Ok(Self {
            header,
            records: records.into_boxed_slice(),
        })
    }

    pub const fn header(&self) -> &RegionSnapshotHeader {
        &self.header
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        &self.header.key
    }

    pub const fn generation(&self) -> ActivationGeneration {
        self.header.generation
    }

    pub const fn committed_tick(&self) -> u64 {
        self.header.committed_tick
    }

    pub const fn persistence_revision(&self) -> PersistenceRevision {
        self.header.persistence_revision
    }

    pub fn records(&self) -> &[SnapshotRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalTailFrame {
    tick: u64,
    records: Box<[SnapshotRecord]>,
}

impl JournalTailFrame {
    pub fn new(tick: u64, mut records: Vec<SnapshotRecord>) -> Result<Self, SnapshotError> {
        sort_and_validate_records(&mut records)?;
        Ok(Self {
            tick,
            records: records.into_boxed_slice(),
        })
    }

    pub const fn tick(&self) -> u64 {
        self.tick
    }

    pub fn records(&self) -> &[SnapshotRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionRecoveryPoint {
    snapshot: RegionCommitSnapshot,
    journal_tail: Box<[JournalTailFrame]>,
}

impl RegionRecoveryPoint {
    pub fn new(
        snapshot: RegionCommitSnapshot,
        journal_tail: Vec<JournalTailFrame>,
    ) -> Result<Self, SnapshotError> {
        enforce_count("journal tail", journal_tail.len(), MAX_JOURNAL_TAIL_FRAMES)?;
        let mut expected = snapshot
            .committed_tick()
            .checked_add(1)
            .ok_or(SnapshotError::TickExhausted)?;
        for frame in &journal_tail {
            if frame.tick != expected {
                return Err(SnapshotError::NonContiguousJournalTail {
                    expected,
                    actual: frame.tick,
                });
            }
            expected = expected
                .checked_add(1)
                .ok_or(SnapshotError::TickExhausted)?;
        }
        Ok(Self {
            snapshot,
            journal_tail: journal_tail.into_boxed_slice(),
        })
    }

    pub const fn snapshot(&self) -> &RegionCommitSnapshot {
        &self.snapshot
    }

    pub fn journal_tail(&self) -> &[JournalTailFrame] {
        &self.journal_tail
    }

    pub fn committed_tick(&self) -> u64 {
        self.journal_tail
            .last()
            .map_or_else(|| self.snapshot.committed_tick(), JournalTailFrame::tick)
    }

    pub const fn persistence_revision(&self) -> PersistenceRevision {
        self.snapshot.persistence_revision()
    }

    pub fn encode(&self) -> Result<Vec<u8>, SnapshotError> {
        let mut encoder = Encoder::new();
        encoder.fixed(SNAPSHOT_MAGIC);
        encoder.u16(SNAPSHOT_SCHEMA_V1);
        encode_header(&mut encoder, self.snapshot.header())?;
        encode_records(&mut encoder, self.snapshot.records())?;
        encoder.var_u64(self.journal_tail.len() as u64);
        for frame in &self.journal_tail {
            encoder.u64(frame.tick);
            encode_records(&mut encoder, frame.records())?;
        }
        Ok(encoder.into_bytes())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, SnapshotError> {
        let mut decoder = Decoder::new(bytes);
        decoder.expect(SNAPSHOT_MAGIC)?;
        let schema = decoder.u16()?;
        if schema != SNAPSHOT_SCHEMA_V1 {
            return Err(SnapshotError::UnsupportedSchema(schema));
        }
        let header = decode_header(&mut decoder)?;
        let snapshot = RegionCommitSnapshot::new(header, decode_records(&mut decoder)?)?;
        let tail_count = decoder.length(MAX_JOURNAL_TAIL_FRAMES)?;
        // The count is untrusted. Do not reserve from it before the first frame proves bytes exist.
        let mut tail = Vec::new();
        for _ in 0..tail_count {
            tail.push(JournalTailFrame::new(
                decoder.u64()?,
                decode_records(&mut decoder)?,
            )?);
        }
        decoder.finish()?;
        Self::new(snapshot, tail)
    }

    pub fn digest(&self) -> Result<[u8; 32], SnapshotError> {
        Ok(*blake3::hash(&self.encode()?).as_bytes())
    }
}

fn encode_header(
    encoder: &mut Encoder,
    header: &RegionSnapshotHeader,
) -> Result<(), SnapshotError> {
    encode_region_key(encoder, &header.key)?;
    encoder.u64(header.generation.get());
    encoder.u64(header.committed_tick);
    encoder.u64(header.persistence_revision.get());
    encoder.u16(header.region_side_chunks);
    encoder.fixed(&header.content_manifest);
    encoder.fixed(&header.state_hash);
    Ok(())
}

fn decode_header(decoder: &mut Decoder<'_>) -> Result<RegionSnapshotHeader, SnapshotError> {
    Ok(RegionSnapshotHeader {
        key: decode_region_key(decoder)?,
        generation: ActivationGeneration::new(decoder.u64()?)
            .map_err(|_| SnapshotError::InvalidGeneration)?,
        committed_tick: decoder.u64()?,
        persistence_revision: PersistenceRevision::new(decoder.u64()?)?,
        region_side_chunks: decoder.u16()?,
        content_manifest: decoder.fixed()?,
        state_hash: decoder.fixed()?,
    })
}

pub(crate) fn encode_region_key(
    encoder: &mut Encoder,
    key: &SimulationRegionKey,
) -> Result<(), SnapshotError> {
    encoder.u128(key.world().get());
    encoder.string(
        &key.dimension().resource().to_string(),
        MAX_RESOURCE_ID_BYTES,
    )?;
    encoder.i32(key.coordinate().x());
    encoder.i32(key.coordinate().z());
    encoder.u16(key.mapping_version().get());
    Ok(())
}

pub(crate) fn decode_region_key(
    decoder: &mut Decoder<'_>,
) -> Result<SimulationRegionKey, SnapshotError> {
    let world = WorldId::new(decoder.u128()?).map_err(|_| SnapshotError::InvalidWorldIdentity)?;
    let dimension = decoder
        .string(MAX_RESOURCE_ID_BYTES)?
        .parse::<ResourceId>()
        .map(DimensionId::new)
        .map_err(|_| SnapshotError::InvalidResourceIdentity)?;
    let coordinate = RegionCoord::new(decoder.i32()?, decoder.i32()?);
    let mapping = RegionMappingVersion::new(decoder.u16()?)
        .map_err(|_| SnapshotError::InvalidMappingVersion)?;
    Ok(SimulationRegionKey::new(
        world, dimension, coordinate, mapping,
    ))
}

fn encode_records(encoder: &mut Encoder, records: &[SnapshotRecord]) -> Result<(), SnapshotError> {
    encoder.var_u64(records.len() as u64);
    for record in records {
        encoder.u8(record.kind as u8);
        encoder.string(&record.domain.to_string(), MAX_RESOURCE_ID_BYTES)?;
        encoder.bytes(&record.key, MAX_SNAPSHOT_KEY_BYTES)?;
        encoder.bytes(&record.value, MAX_SNAPSHOT_VALUE_BYTES)?;
    }
    Ok(())
}

fn decode_records(decoder: &mut Decoder<'_>) -> Result<Vec<SnapshotRecord>, SnapshotError> {
    let count = decoder.length(MAX_SNAPSHOT_RECORDS)?;
    // A truncated record set must fail without allocating its attacker-controlled declared size.
    let mut records = Vec::new();
    for _ in 0..count {
        let kind = SnapshotRecordKind::from_tag(decoder.u8()?)?;
        let domain = decoder
            .string(MAX_RESOURCE_ID_BYTES)?
            .parse::<ResourceId>()
            .map_err(|_| SnapshotError::InvalidResourceIdentity)?;
        let key = decoder.bytes(MAX_SNAPSHOT_KEY_BYTES)?.to_vec();
        let value = decoder.bytes(MAX_SNAPSHOT_VALUE_BYTES)?.to_vec();
        records.push(SnapshotRecord::new(kind, domain, key, value)?);
    }
    Ok(records)
}

fn sort_and_validate_records(records: &mut [SnapshotRecord]) -> Result<(), SnapshotError> {
    enforce_count("snapshot", records.len(), MAX_SNAPSHOT_RECORDS)?;
    records.sort_by(|left, right| {
        (&left.kind, &left.domain, &left.key).cmp(&(&right.kind, &right.domain, &right.key))
    });
    if records.windows(2).any(|pair| {
        pair[0].kind == pair[1].kind
            && pair[0].domain == pair[1].domain
            && pair[0].key == pair[1].key
    }) {
        return Err(SnapshotError::DuplicateRecord);
    }
    Ok(())
}

fn enforce_count(kind: &'static str, actual: usize, maximum: usize) -> Result<(), SnapshotError> {
    if actual > maximum {
        Err(SnapshotError::TooManyRecords {
            kind,
            actual,
            maximum,
        })
    } else {
        Ok(())
    }
}

fn enforce_length(kind: &'static str, actual: usize, maximum: usize) -> Result<(), SnapshotError> {
    if actual > maximum {
        Err(SnapshotError::PayloadTooLarge {
            kind,
            actual,
            maximum,
        })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SnapshotError {
    #[error("persistence revision cannot be zero")]
    ZeroPersistenceRevision,
    #[error("persistence revision is exhausted")]
    PersistenceRevisionExhausted,
    #[error("snapshot Region side cannot be zero")]
    ZeroRegionSide,
    #[error("snapshot schema {0} is unsupported")]
    UnsupportedSchema(u16),
    #[error("snapshot record kind tag {0} is unknown")]
    UnknownRecordKind(u8),
    #[error("snapshot contains a duplicate kind/domain/key record")]
    DuplicateRecord,
    #[error("{kind} has {actual} records, exceeding limit {maximum}")]
    TooManyRecords {
        kind: &'static str,
        actual: usize,
        maximum: usize,
    },
    #[error("{kind} has {actual} bytes, exceeding limit {maximum}")]
    PayloadTooLarge {
        kind: &'static str,
        actual: usize,
        maximum: usize,
    },
    #[error("journal tail expected tick {expected}, got {actual}")]
    NonContiguousJournalTail { expected: u64, actual: u64 },
    #[error("snapshot tick arithmetic is exhausted")]
    TickExhausted,
    #[error("snapshot contains an invalid activation generation")]
    InvalidGeneration,
    #[error("snapshot contains an invalid world identity")]
    InvalidWorldIdentity,
    #[error("snapshot contains an invalid mapping version")]
    InvalidMappingVersion,
    #[error("snapshot contains an invalid resource identity")]
    InvalidResourceIdentity,
    #[error(transparent)]
    Codec(#[from] CodecError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(-1, 2),
            RegionMappingVersion::V1,
        )
    }

    fn record(value: u8) -> SnapshotRecord {
        SnapshotRecord::new(
            SnapshotRecordKind::Chunk,
            ResourceId::new("ferrite", "chunk/v1").unwrap(),
            vec![value],
            vec![value + 1],
        )
        .unwrap()
    }

    fn point(reversed: bool) -> RegionRecoveryPoint {
        let mut records = vec![record(2), record(1)];
        if reversed {
            records.reverse();
        }
        RegionRecoveryPoint::new(
            RegionCommitSnapshot::new(
                RegionSnapshotHeader {
                    key: key(),
                    generation: ActivationGeneration::INITIAL,
                    committed_tick: 4,
                    persistence_revision: PersistenceRevision::INITIAL,
                    region_side_chunks: 8,
                    content_manifest: [3; 32],
                    state_hash: [4; 32],
                },
                records,
            )
            .unwrap(),
            vec![JournalTailFrame::new(5, vec![record(3)]).unwrap()],
        )
        .unwrap()
    }

    #[test]
    fn recovery_point_round_trips_and_has_locked_bytes() {
        let recovery_point = point(false);
        let bytes = recovery_point.encode().unwrap();
        assert_eq!(RegionRecoveryPoint::decode(&bytes).unwrap(), recovery_point);
        assert_eq!(
            point(false).digest().unwrap(),
            point(true).digest().unwrap()
        );
        assert_eq!(
            blake3::Hash::from_bytes(recovery_point.digest().unwrap())
                .to_hex()
                .as_str(),
            "5dc371a8e67c332f585198c2dc34eddefd3be6a335da2ff0297ce1bf61cf256e"
        );
        assert_eq!(recovery_point.committed_tick(), 5);
    }

    #[test]
    fn duplicate_and_noncontiguous_records_fail_closed() {
        let duplicate = record(1);
        assert!(
            RegionCommitSnapshot::new(
                RegionSnapshotHeader {
                    key: key(),
                    generation: ActivationGeneration::INITIAL,
                    committed_tick: 1,
                    persistence_revision: PersistenceRevision::INITIAL,
                    region_side_chunks: 8,
                    content_manifest: [0; 32],
                    state_hash: [0; 32],
                },
                vec![duplicate.clone(), duplicate],
            )
            .is_err()
        );
        let snapshot = point(false).snapshot().clone();
        assert!(
            RegionRecoveryPoint::new(snapshot, vec![JournalTailFrame::new(9, vec![]).unwrap()])
                .is_err()
        );
    }

    #[test]
    fn truncated_declared_maximum_counts_fail_before_bulk_allocation() {
        let header = point(false).snapshot().header().clone();

        let mut records = Encoder::new();
        records.fixed(SNAPSHOT_MAGIC);
        records.u16(SNAPSHOT_SCHEMA_V1);
        encode_header(&mut records, &header).unwrap();
        records.var_u64(MAX_SNAPSHOT_RECORDS as u64);
        assert!(RegionRecoveryPoint::decode(&records.into_bytes()).is_err());

        let mut tail = Encoder::new();
        tail.fixed(SNAPSHOT_MAGIC);
        tail.u16(SNAPSHOT_SCHEMA_V1);
        encode_header(&mut tail, &header).unwrap();
        encode_records(&mut tail, &[]).unwrap();
        tail.var_u64(MAX_JOURNAL_TAIL_FRAMES as u64);
        assert!(RegionRecoveryPoint::decode(&tail.into_bytes()).is_err());
    }
}
