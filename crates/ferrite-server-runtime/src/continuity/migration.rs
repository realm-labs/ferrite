use std::collections::BTreeSet;

use ferrite_foundation::region::SimulationRegionKey;
use ferrite_persistence::snapshot::{
    JournalTailFrame, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
    SnapshotError, SnapshotRecord, SnapshotRecordKind,
};
use ferrite_persistence::store::{CommitReceipt, RegionFileStore, StoreError};
use thiserror::Error;

use crate::continuity::identity::{
    ContinuityGeneration, classify_domain, domain_id, is_reserved_continuity_id,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedRecords {
    generation: Option<ContinuityGeneration>,
    records: Vec<SnapshotRecord>,
}

impl NormalizedRecords {
    pub const fn generation(&self) -> Option<ContinuityGeneration> {
        self.generation
    }

    pub fn records(&self) -> &[SnapshotRecord] {
        &self.records
    }

    pub fn into_records(self) -> Vec<SnapshotRecord> {
        self.records
    }

    pub const fn was_migrated(&self) -> bool {
        matches!(self.generation, Some(ContinuityGeneration::Legacy))
    }
}

pub fn normalize_records(
    records: &[SnapshotRecord],
) -> Result<NormalizedRecords, ContinuityMigrationError> {
    let generation = detect_generation(records.iter())?;
    let mut identities = BTreeSet::new();
    let records = records
        .iter()
        .map(|record| {
            let Some(classified) = classify_domain(record.domain()) else {
                if is_reserved_continuity_id(record.domain()) {
                    return Err(ContinuityMigrationError::UnsupportedIdentity(
                        record.domain().to_string(),
                    ));
                }
                return Ok(record.clone());
            };
            if record.kind() != classified.domain.record_kind() {
                return Err(ContinuityMigrationError::WrongRecordKind {
                    domain: record.domain().to_string(),
                    expected: classified.domain.record_kind(),
                    actual: record.kind(),
                });
            }
            let normalized_domain = domain_id(classified.domain, ContinuityGeneration::Current);
            if !identities.insert((
                record.kind(),
                normalized_domain.clone(),
                record.key().to_vec(),
            )) {
                return Err(ContinuityMigrationError::DuplicateIdentity {
                    kind: record.kind(),
                    domain: normalized_domain.to_string(),
                    key: record.key().to_vec(),
                });
            }
            SnapshotRecord::new(
                record.kind(),
                normalized_domain,
                record.key().to_vec(),
                record.value().to_vec(),
            )
            .map_err(Into::into)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NormalizedRecords {
        generation,
        records,
    })
}

pub fn normalize_recovery_point(
    point: &RegionRecoveryPoint,
) -> Result<RegionRecoveryPoint, ContinuityMigrationError> {
    let all_records = point.snapshot().records().iter().chain(
        point
            .journal_tail()
            .iter()
            .flat_map(|frame| frame.records().iter()),
    );
    detect_generation(all_records)?;
    validate_snapshot_hash(point)?;
    let snapshot_records = normalize_records(point.snapshot().records())?.into_records();
    let header = point.snapshot().header();
    let snapshot = RegionCommitSnapshot::new(
        RegionSnapshotHeader {
            key: header.key.clone(),
            generation: header.generation,
            committed_tick: header.committed_tick,
            persistence_revision: header.persistence_revision,
            region_side_chunks: header.region_side_chunks,
            content_manifest: header.content_manifest,
            state_hash: canonical_record_hash(&snapshot_records),
        },
        snapshot_records,
    )?;
    let journal_tail = point
        .journal_tail()
        .iter()
        .map(|frame| {
            JournalTailFrame::new(
                frame.tick(),
                normalize_records(frame.records()).map(NormalizedRecords::into_records)?,
            )
            .map_err(Into::into)
        })
        .collect::<Result<Vec<_>, ContinuityMigrationError>>()?;
    RegionRecoveryPoint::new(snapshot, journal_tail).map_err(Into::into)
}

fn detect_generation<'a>(
    records: impl Iterator<Item = &'a SnapshotRecord>,
) -> Result<Option<ContinuityGeneration>, ContinuityMigrationError> {
    let mut generation = None;
    for record in records {
        if let Some(classified) = classify_domain(record.domain()) {
            match generation {
                Some(existing) if existing != classified.generation => {
                    return Err(ContinuityMigrationError::MixedGenerations);
                }
                None => generation = Some(classified.generation),
                _ => {}
            }
        } else if is_reserved_continuity_id(record.domain()) {
            return Err(ContinuityMigrationError::UnsupportedIdentity(
                record.domain().to_string(),
            ));
        }
    }
    Ok(generation)
}

fn validate_snapshot_hash(point: &RegionRecoveryPoint) -> Result<(), ContinuityMigrationError> {
    let expected = point.snapshot().header().state_hash;
    let actual = canonical_record_hash(point.snapshot().records());
    if actual == expected {
        Ok(())
    } else {
        Err(ContinuityMigrationError::StateHashMismatch { expected, actual })
    }
}

#[must_use]
pub fn canonical_record_hash(records: &[SnapshotRecord]) -> [u8; 32] {
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

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContinuityMigrationError {
    #[error("continuity records mix legacy and current identity generations")]
    MixedGenerations,
    #[error("continuity identity {0} is reserved but unsupported")]
    UnsupportedIdentity(String),
    #[error("continuity identity {domain} requires {expected:?}, not {actual:?}")]
    WrongRecordKind {
        domain: String,
        expected: SnapshotRecordKind,
        actual: SnapshotRecordKind,
    },
    #[error("continuity identity is duplicated: {kind:?} {domain} key {key:?}")]
    DuplicateIdentity {
        kind: SnapshotRecordKind,
        domain: String,
        key: Vec<u8>,
    },
    #[error("continuity snapshot state hash mismatch")]
    StateHashMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

pub enum PreparedStoreMigration {
    AlreadyCurrent(RegionRecoveryPoint),
    Prepared(StoreMigrationPlan),
}

pub struct StoreMigrationPlan {
    key: SimulationRegionKey,
    source_digest: [u8; 32],
    candidate: RegionRecoveryPoint,
}

impl StoreMigrationPlan {
    pub const fn candidate(&self) -> &RegionRecoveryPoint {
        &self.candidate
    }

    pub fn commit(self, store: &mut RegionFileStore) -> Result<CommitReceipt, StoreMigrationError> {
        let current = store
            .load(&self.key)?
            .ok_or(StoreMigrationError::SourceDisappeared)?;
        if current.digest()? != self.source_digest {
            return Err(StoreMigrationError::SourceChanged);
        }
        commit_current_point(store, &self.candidate)
    }
}

pub fn prepare_store_migration(
    store: &RegionFileStore,
    key: &SimulationRegionKey,
) -> Result<PreparedStoreMigration, StoreMigrationError> {
    let source = store.load(key)?.ok_or(StoreMigrationError::SourceMissing)?;
    let generation = detect_generation(
        source.snapshot().records().iter().chain(
            source
                .journal_tail()
                .iter()
                .flat_map(|frame| frame.records()),
        ),
    )?;
    let normalized = normalize_recovery_point(&source)?;
    if generation != Some(ContinuityGeneration::Legacy) {
        return Ok(PreparedStoreMigration::AlreadyCurrent(normalized));
    }
    let header = normalized.snapshot().header();
    let snapshot = RegionCommitSnapshot::new(
        RegionSnapshotHeader {
            key: header.key.clone(),
            generation: header.generation,
            committed_tick: header.committed_tick,
            persistence_revision: header.persistence_revision.checked_next()?,
            region_side_chunks: header.region_side_chunks,
            content_manifest: header.content_manifest,
            state_hash: header.state_hash,
        },
        normalized.snapshot().records().to_vec(),
    )?;
    let candidate = RegionRecoveryPoint::new(snapshot, normalized.journal_tail().to_vec())?;
    Ok(PreparedStoreMigration::Prepared(StoreMigrationPlan {
        key: key.clone(),
        source_digest: source.digest()?,
        candidate,
    }))
}

pub fn commit_current_point(
    store: &mut RegionFileStore,
    point: &RegionRecoveryPoint,
) -> Result<CommitReceipt, StoreMigrationError> {
    match detect_generation(
        point.snapshot().records().iter().chain(
            point
                .journal_tail()
                .iter()
                .flat_map(|frame| frame.records()),
        ),
    )? {
        Some(ContinuityGeneration::Legacy) => Err(StoreMigrationError::LegacyWriteDenied),
        _ => Ok(store.commit(point)?),
    }
}

#[derive(Debug, Error)]
pub enum StoreMigrationError {
    #[error("continuity migration source is absent")]
    SourceMissing,
    #[error("continuity migration source disappeared before commit")]
    SourceDisappeared,
    #[error("continuity migration source changed before commit")]
    SourceChanged,
    #[error("legacy continuity identities are read-only")]
    LegacyWriteDenied,
    #[error(transparent)]
    Migration(#[from] ContinuityMigrationError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Store(#[from] StoreError),
}
