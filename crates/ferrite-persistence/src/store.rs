//! Append-and-repoint filesystem store with committed-transaction recovery.

use crate::codec::{CodecError, Decoder, Encoder};
use crate::snapshot::{
    PersistenceRevision, RegionRecoveryPoint, SnapshotError, decode_region_key, encode_region_key,
};
use ferrite_foundation::region::SimulationRegionKey;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

const DATA_MAGIC: &[u8; 4] = b"FRDT";
const INDEX_MAGIC: &[u8; 4] = b"FRIX";
const JOURNAL_MAGIC: &[u8; 4] = b"FRJR";
const FRAME_HEADER_BYTES: usize = 4 + 8 + 32;
pub const MAX_DURABLE_FRAME_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_STORE_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;

pub struct RegionFileStore {
    root: PathBuf,
}

impl RegionFileStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, StoreError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).map_err(|source| StoreError::Io {
            operation: "create store directory",
            source,
        })?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn commit(&mut self, point: &RegionRecoveryPoint) -> Result<CommitReceipt, StoreError> {
        repair_truncated_tail(&self.data_path(), DATA_MAGIC)?;
        repair_truncated_tail(&self.index_path(), INDEX_MAGIC)?;
        repair_truncated_tail(&self.journal_path(), JOURNAL_MAGIC)?;
        let current = self.load(point.snapshot().key())?;
        validate_next_point(current.as_ref(), point)?;
        let journal_frames = scan_file(&self.journal_path(), JOURNAL_MAGIC)?;
        let transaction = next_transaction_id(&journal_frames)?;
        let digest = point.digest()?;
        let revision = point.persistence_revision();

        let intent = encode_intent(transaction, point.snapshot().key(), revision, digest)?;
        append_frame(&self.journal_path(), JOURNAL_MAGIC, &intent)?;

        let encoded = point.encode()?;
        let data_location = append_frame(&self.data_path(), DATA_MAGIC, &encoded)?;

        let index = encode_index(IndexRecord {
            transaction,
            key: point.snapshot().key().clone(),
            revision,
            digest,
            data_offset: data_location.offset,
            data_length: data_location.length,
        })?;
        append_frame(&self.index_path(), INDEX_MAGIC, &index)?;

        let commit = encode_commit(transaction);
        append_frame(&self.journal_path(), JOURNAL_MAGIC, &commit)?;
        Ok(CommitReceipt {
            transaction,
            revision,
            committed_tick: point.committed_tick(),
            digest,
        })
    }

    pub fn load(
        &self,
        key: &SimulationRegionKey,
    ) -> Result<Option<RegionRecoveryPoint>, StoreError> {
        let journal = decode_journal(scan_file(&self.journal_path(), JOURNAL_MAGIC)?)?;
        let indexes = decode_indexes(scan_file(&self.index_path(), INDEX_MAGIC)?)?;
        let data = scan_file(&self.data_path(), DATA_MAGIC)?;
        let data_by_offset = data
            .into_iter()
            .map(|frame| (frame.offset, frame))
            .collect::<BTreeMap<_, _>>();

        let mut selected: Option<(PersistenceRevision, RegionRecoveryPoint)> = None;
        for index in indexes {
            if &index.key != key || !journal.committed.contains(&index.transaction) {
                continue;
            }
            let intent = journal
                .intents
                .get(&index.transaction)
                .ok_or(StoreError::CommittedWithoutIntent(index.transaction))?;
            if intent.key != index.key
                || intent.revision != index.revision
                || intent.digest != index.digest
            {
                return Err(StoreError::TransactionMetadataMismatch(index.transaction));
            }
            let frame = data_by_offset
                .get(&index.data_offset)
                .ok_or(StoreError::MissingDataFrame(index.transaction))?;
            if frame.length != index.data_length {
                return Err(StoreError::DataLengthMismatch(index.transaction));
            }
            let actual_digest = *blake3::hash(&frame.payload).as_bytes();
            if actual_digest != index.digest {
                return Err(StoreError::DataDigestMismatch(index.transaction));
            }
            let point = RegionRecoveryPoint::decode(&frame.payload)?;
            if point.snapshot().key() != key || point.persistence_revision() != index.revision {
                return Err(StoreError::DataIdentityMismatch(index.transaction));
            }
            if let Some((revision, _)) = &selected
                && index.revision <= *revision
            {
                return Err(StoreError::NonMonotonicIndex);
            }
            selected = Some((index.revision, point));
        }
        Ok(selected.map(|(_, point)| point))
    }

    fn data_path(&self) -> PathBuf {
        self.root.join("region-data.log")
    }

    fn index_path(&self) -> PathBuf {
        self.root.join("region-index.log")
    }

    fn journal_path(&self) -> PathBuf {
        self.root.join("region-journal.log")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitReceipt {
    transaction: u64,
    revision: PersistenceRevision,
    committed_tick: u64,
    digest: [u8; 32],
}

impl CommitReceipt {
    pub const fn transaction(self) -> u64 {
        self.transaction
    }

    pub const fn revision(self) -> PersistenceRevision {
        self.revision
    }

    pub const fn committed_tick(self) -> u64 {
        self.committed_tick
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

fn validate_next_point(
    current: Option<&RegionRecoveryPoint>,
    next: &RegionRecoveryPoint,
) -> Result<(), StoreError> {
    match current {
        None if next.persistence_revision() != PersistenceRevision::INITIAL => {
            Err(StoreError::UnexpectedInitialRevision)
        }
        None => Ok(()),
        Some(current) => {
            let expected = current.persistence_revision().checked_next()?;
            if next.persistence_revision() != expected {
                return Err(StoreError::UnexpectedRevision {
                    expected,
                    actual: next.persistence_revision(),
                });
            }
            if next.committed_tick() < current.committed_tick() {
                return Err(StoreError::CommittedTickRegressed);
            }
            if next.snapshot().generation() < current.snapshot().generation() {
                return Err(StoreError::GenerationRegressed);
            }
            Ok(())
        }
    }
}

#[derive(Debug)]
struct Frame {
    offset: u64,
    length: u64,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct FrameLocation {
    offset: u64,
    length: u64,
}

fn append_frame(path: &Path, magic: &[u8; 4], payload: &[u8]) -> Result<FrameLocation, StoreError> {
    if payload.len() > MAX_DURABLE_FRAME_BYTES {
        return Err(StoreError::FrameTooLarge {
            actual: payload.len(),
            maximum: MAX_DURABLE_FRAME_BYTES,
        });
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(path)
        .map_err(|source| StoreError::Io {
            operation: "open append log",
            source,
        })?;
    let offset = file
        .seek(SeekFrom::End(0))
        .map_err(|source| StoreError::Io {
            operation: "seek append log",
            source,
        })?;
    let length = payload.len() as u64;
    file.write_all(magic)
        .and_then(|_| file.write_all(&length.to_le_bytes()))
        .and_then(|_| file.write_all(blake3::hash(payload).as_bytes()))
        .and_then(|_| file.write_all(payload))
        .map_err(|source| StoreError::Io {
            operation: "append durable frame",
            source,
        })?;
    file.sync_data().map_err(|source| StoreError::Io {
        operation: "sync durable frame",
        source,
    })?;
    Ok(FrameLocation {
        offset,
        length: length + FRAME_HEADER_BYTES as u64,
    })
}

fn scan_file(path: &Path, magic: &[u8; 4]) -> Result<Vec<Frame>, StoreError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(StoreError::Io {
                operation: "open durable log",
                source,
            });
        }
    };
    let length = file
        .metadata()
        .map_err(|source| StoreError::Io {
            operation: "stat durable log",
            source,
        })?
        .len();
    if length > MAX_STORE_FILE_BYTES {
        return Err(StoreError::StoreFileTooLarge {
            actual: length,
            maximum: MAX_STORE_FILE_BYTES,
        });
    }
    let mut bytes = Vec::with_capacity(length as usize);
    file.read_to_end(&mut bytes)
        .map_err(|source| StoreError::Io {
            operation: "read durable log",
            source,
        })?;
    let mut frames = Vec::new();
    let mut offset = 0_usize;
    while offset < bytes.len() {
        if bytes.len() - offset < FRAME_HEADER_BYTES {
            break;
        }
        if &bytes[offset..offset + 4] != magic {
            return Err(StoreError::CorruptFrame(offset as u64));
        }
        let payload_length = u64::from_le_bytes(
            bytes[offset + 4..offset + 12]
                .try_into()
                .map_err(|_| StoreError::CorruptFrame(offset as u64))?,
        );
        let payload_length =
            usize::try_from(payload_length).map_err(|_| StoreError::FrameTooLarge {
                actual: usize::MAX,
                maximum: MAX_DURABLE_FRAME_BYTES,
            })?;
        if payload_length > MAX_DURABLE_FRAME_BYTES {
            return Err(StoreError::FrameTooLarge {
                actual: payload_length,
                maximum: MAX_DURABLE_FRAME_BYTES,
            });
        }
        let end = offset
            .checked_add(FRAME_HEADER_BYTES)
            .and_then(|start| start.checked_add(payload_length))
            .ok_or(StoreError::CorruptFrame(offset as u64))?;
        if end > bytes.len() {
            break;
        }
        let expected = &bytes[offset + 12..offset + FRAME_HEADER_BYTES];
        let payload = &bytes[offset + FRAME_HEADER_BYTES..end];
        if blake3::hash(payload).as_bytes() != expected {
            return Err(StoreError::ChecksumMismatch(offset as u64));
        }
        frames.push(Frame {
            offset: offset as u64,
            length: (end - offset) as u64,
            payload: payload.to_vec(),
        });
        offset = end;
    }
    Ok(frames)
}

fn repair_truncated_tail(path: &Path, magic: &[u8; 4]) -> Result<(), StoreError> {
    let frames = scan_file(path, magic)?;
    let valid_length = frames.last().map_or(0, |frame| frame.offset + frame.length);
    let actual_length = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(StoreError::Io {
                operation: "stat durable log for repair",
                source,
            });
        }
    };
    if valid_length < actual_length {
        let file = OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|source| StoreError::Io {
                operation: "open durable log for repair",
                source,
            })?;
        file.set_len(valid_length)
            .and_then(|_| file.sync_data())
            .map_err(|source| StoreError::Io {
                operation: "repair truncated durable tail",
                source,
            })?;
    }
    Ok(())
}

#[derive(Debug)]
struct IntentRecord {
    key: SimulationRegionKey,
    revision: PersistenceRevision,
    digest: [u8; 32],
}

#[derive(Debug, Default)]
struct JournalState {
    intents: BTreeMap<u64, IntentRecord>,
    committed: BTreeSet<u64>,
}

fn encode_intent(
    transaction: u64,
    key: &SimulationRegionKey,
    revision: PersistenceRevision,
    digest: [u8; 32],
) -> Result<Vec<u8>, StoreError> {
    let mut encoder = Encoder::new();
    encoder.u8(0);
    encoder.u64(transaction);
    encode_region_key(&mut encoder, key)?;
    encoder.u64(revision.get());
    encoder.fixed(&digest);
    Ok(encoder.into_bytes())
}

fn encode_commit(transaction: u64) -> Vec<u8> {
    let mut encoder = Encoder::new();
    encoder.u8(1);
    encoder.u64(transaction);
    encoder.into_bytes()
}

fn decode_journal(frames: Vec<Frame>) -> Result<JournalState, StoreError> {
    let mut state = JournalState::default();
    for frame in frames {
        let mut decoder = Decoder::new(&frame.payload);
        let tag = decoder.u8()?;
        let transaction = decoder.u64()?;
        match tag {
            0 => {
                let intent = IntentRecord {
                    key: decode_region_key(&mut decoder)?,
                    revision: PersistenceRevision::new(decoder.u64()?)?,
                    digest: decoder.fixed()?,
                };
                if state.intents.insert(transaction, intent).is_some() {
                    return Err(StoreError::DuplicateTransaction(transaction));
                }
            }
            1 => {
                if !state.committed.insert(transaction) {
                    return Err(StoreError::DuplicateCommit(transaction));
                }
            }
            _ => return Err(StoreError::UnknownJournalTag(tag)),
        }
        decoder.finish()?;
    }
    for transaction in &state.committed {
        if !state.intents.contains_key(transaction) {
            return Err(StoreError::CommittedWithoutIntent(*transaction));
        }
    }
    Ok(state)
}

#[derive(Debug)]
struct IndexRecord {
    transaction: u64,
    key: SimulationRegionKey,
    revision: PersistenceRevision,
    digest: [u8; 32],
    data_offset: u64,
    data_length: u64,
}

fn encode_index(record: IndexRecord) -> Result<Vec<u8>, StoreError> {
    let mut encoder = Encoder::new();
    encoder.u64(record.transaction);
    encode_region_key(&mut encoder, &record.key)?;
    encoder.u64(record.revision.get());
    encoder.fixed(&record.digest);
    encoder.u64(record.data_offset);
    encoder.u64(record.data_length);
    Ok(encoder.into_bytes())
}

fn decode_indexes(frames: Vec<Frame>) -> Result<Vec<IndexRecord>, StoreError> {
    frames
        .into_iter()
        .map(|frame| {
            let mut decoder = Decoder::new(&frame.payload);
            let record = IndexRecord {
                transaction: decoder.u64()?,
                key: decode_region_key(&mut decoder)?,
                revision: PersistenceRevision::new(decoder.u64()?)?,
                digest: decoder.fixed()?,
                data_offset: decoder.u64()?,
                data_length: decoder.u64()?,
            };
            decoder.finish()?;
            Ok(record)
        })
        .collect()
}

fn next_transaction_id(frames: &[Frame]) -> Result<u64, StoreError> {
    let state = decode_journal(
        frames
            .iter()
            .map(|frame| Frame {
                offset: frame.offset,
                length: frame.length,
                payload: frame.payload.clone(),
            })
            .collect(),
    )?;
    let maximum = state
        .intents
        .keys()
        .chain(state.committed.iter())
        .copied()
        .max()
        .unwrap_or(0);
    maximum
        .checked_add(1)
        .ok_or(StoreError::TransactionExhausted)
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("{operation}: {source}")]
    Io {
        operation: &'static str,
        source: std::io::Error,
    },
    #[error("durable frame has {actual} bytes, exceeding limit {maximum}")]
    FrameTooLarge { actual: usize, maximum: usize },
    #[error("durable store file has {actual} bytes, exceeding limit {maximum}")]
    StoreFileTooLarge { actual: u64, maximum: u64 },
    #[error("durable frame at offset {0} is corrupt")]
    CorruptFrame(u64),
    #[error("durable frame at offset {0} has a checksum mismatch")]
    ChecksumMismatch(u64),
    #[error("journal tag {0} is unknown")]
    UnknownJournalTag(u8),
    #[error("transaction {0} has duplicate intents")]
    DuplicateTransaction(u64),
    #[error("transaction {0} has duplicate commit records")]
    DuplicateCommit(u64),
    #[error("transaction {0} is committed without an intent")]
    CommittedWithoutIntent(u64),
    #[error("transaction {0} metadata differs between journal and index")]
    TransactionMetadataMismatch(u64),
    #[error("transaction {0} points to missing data")]
    MissingDataFrame(u64),
    #[error("transaction {0} data-frame length differs from its index")]
    DataLengthMismatch(u64),
    #[error("transaction {0} data digest differs from its index")]
    DataDigestMismatch(u64),
    #[error("transaction {0} data identity differs from its index")]
    DataIdentityMismatch(u64),
    #[error("Region index revisions are not strictly monotonic")]
    NonMonotonicIndex,
    #[error("first Region recovery point must use the initial persistence revision")]
    UnexpectedInitialRevision,
    #[error("expected persistence revision {expected:?}, got {actual:?}")]
    UnexpectedRevision {
        expected: PersistenceRevision,
        actual: PersistenceRevision,
    },
    #[error("committed tick regressed")]
    CommittedTickRegressed,
    #[error("activation generation regressed")]
    GenerationRegressed,
    #[error("persistence transaction identity is exhausted")]
    TransactionExhausted,
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{
        RegionCommitSnapshot, RegionSnapshotHeader, SnapshotRecord, SnapshotRecordKind,
    };
    use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};
    use ferrite_foundation::resource::ResourceId;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(std::env::temp_dir().join(format!(
                "ferrite-persistence-{}-{name}-{nonce}",
                std::process::id()
            )))
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            if self.0.exists() {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
    }

    fn key() -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        )
    }

    fn point(revision: u64, tick: u64) -> RegionRecoveryPoint {
        let record = SnapshotRecord::new(
            SnapshotRecordKind::Entity,
            ResourceId::new("ferrite", "entity/v1").unwrap(),
            vec![1],
            vec![tick as u8],
        )
        .unwrap();
        RegionRecoveryPoint::new(
            RegionCommitSnapshot::new(
                RegionSnapshotHeader {
                    key: key(),
                    generation: ActivationGeneration::INITIAL,
                    committed_tick: tick,
                    persistence_revision: PersistenceRevision::new(revision).unwrap(),
                    region_side_chunks: 8,
                    content_manifest: [2; 32],
                    state_hash: [3; 32],
                },
                vec![record],
            )
            .unwrap(),
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn committed_index_repoints_and_truncated_tails_are_repaired() {
        let directory = TestDirectory::new("repoint");
        let mut store = RegionFileStore::open(&directory.0).unwrap();
        store.commit(&point(1, 4)).unwrap();
        let second = point(2, 5);
        let receipt = store.commit(&second).unwrap();
        assert_eq!(receipt.revision().get(), 2);
        assert_eq!(store.load(&key()).unwrap(), Some(second));

        let journal = store.journal_path();
        let mut file = OpenOptions::new().append(true).open(&journal).unwrap();
        file.write_all(b"FR").unwrap();
        file.sync_data().unwrap();
        assert_eq!(store.load(&key()).unwrap().unwrap().committed_tick(), 5);
        let third = point(3, 6);
        store.commit(&third).unwrap();
        assert_eq!(store.load(&key()).unwrap(), Some(third));
    }

    #[test]
    fn checksum_corruption_is_rejected_instead_of_falling_back() {
        let directory = TestDirectory::new("corrupt");
        let mut store = RegionFileStore::open(&directory.0).unwrap();
        store.commit(&point(1, 4)).unwrap();
        let path = store.data_path();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        file.seek(SeekFrom::Start(FRAME_HEADER_BYTES as u64 + 4))
            .unwrap();
        file.write_all(&[0xff]).unwrap();
        file.sync_data().unwrap();
        assert!(matches!(
            store.load(&key()),
            Err(StoreError::ChecksumMismatch(_))
        ));
    }

    #[test]
    fn index_without_commit_marker_does_not_advance_recovery() {
        let directory = TestDirectory::new("uncommitted");
        let mut store = RegionFileStore::open(&directory.0).unwrap();
        let first = point(1, 4);
        store.commit(&first).unwrap();

        let second = point(2, 5);
        let transaction = 2;
        let digest = second.digest().unwrap();
        let intent = encode_intent(
            transaction,
            second.snapshot().key(),
            second.persistence_revision(),
            digest,
        )
        .unwrap();
        append_frame(&store.journal_path(), JOURNAL_MAGIC, &intent).unwrap();
        let location =
            append_frame(&store.data_path(), DATA_MAGIC, &second.encode().unwrap()).unwrap();
        let index = encode_index(IndexRecord {
            transaction,
            key: key(),
            revision: second.persistence_revision(),
            digest,
            data_offset: location.offset,
            data_length: location.length,
        })
        .unwrap();
        append_frame(&store.index_path(), INDEX_MAGIC, &index).unwrap();
        assert_eq!(store.load(&key()).unwrap(), Some(first));

        append_frame(
            &store.journal_path(),
            JOURNAL_MAGIC,
            &encode_commit(transaction),
        )
        .unwrap();
        assert_eq!(store.load(&key()).unwrap(), Some(second));
    }

    #[test]
    fn revisions_and_ticks_cannot_regress() {
        let directory = TestDirectory::new("revision");
        let mut store = RegionFileStore::open(&directory.0).unwrap();
        store.commit(&point(1, 4)).unwrap();
        assert!(store.commit(&point(1, 5)).is_err());
        assert!(store.commit(&point(2, 3)).is_err());
        assert_eq!(store.load(&key()).unwrap().unwrap().committed_tick(), 4);
    }
}
