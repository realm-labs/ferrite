//! Durable checkpoint selection and save integration for the formal local world.

use std::collections::BTreeMap;

use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_persistence::snapshot::{
    PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
    SnapshotError, SnapshotRecord,
};
use ferrite_persistence::store::{CommitReceipt, RegionFileStore, StoreError};
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::composite::gateway::CompositeGatewayTickReport;
use crate::continuity::migration::{
    StoreMigrationError, canonical_record_hash, commit_current_point,
};
use crate::world_service::metadata::{
    DurableWorldMetadata, WorldMetadataError, directory_is_empty, region_store_root,
};

const REGION_SIDE_CHUNKS: u16 = 8;

pub(crate) struct FormalWorldPersistence {
    stores: BTreeMap<SimulationRegionKey, RegionStore>,
    captures: BTreeMap<SimulationRegionKey, RegionCapture>,
    staged_commits: BTreeMap<SimulationRegionKey, FormalRegionCommit>,
    control_region: SimulationRegionKey,
    content_manifest: [u8; 32],
    autosave_interval_ticks: u64,
    checkpoint_tick: GameTick,
}

struct RegionStore {
    store: RegionFileStore,
    next_revision: PersistenceRevision,
}

#[derive(Debug, Clone)]
struct RegionCapture {
    tick: GameTick,
    generation: ActivationGeneration,
    continuity_hash: [u8; 32],
    records: Vec<SnapshotRecord>,
}

pub(crate) struct FormalWorldRecovery {
    points: BTreeMap<SimulationRegionKey, RegionRecoveryPoint>,
    checkpoint_tick: GameTick,
    resume_tick: GameTick,
}

#[derive(Clone)]
pub(crate) struct FormalRegionCommit {
    region: SimulationRegionKey,
    point: RegionRecoveryPoint,
    receipt: CommitReceipt,
}

impl FormalRegionCommit {
    pub(crate) const fn region(&self) -> &SimulationRegionKey {
        &self.region
    }

    pub(crate) const fn point(&self) -> &RegionRecoveryPoint {
        &self.point
    }

    pub(crate) const fn receipt(&self) -> CommitReceipt {
        self.receipt
    }
}

impl FormalWorldRecovery {
    pub(crate) fn point(&self, key: &SimulationRegionKey) -> Option<&RegionRecoveryPoint> {
        self.points.get(key)
    }

    pub(crate) const fn checkpoint_tick(&self) -> GameTick {
        self.checkpoint_tick
    }

    pub(crate) const fn resume_tick(&self) -> GameTick {
        self.resume_tick
    }
}

impl FormalWorldPersistence {
    pub(crate) fn open(
        storage_root: &std::path::Path,
        durable: &DurableWorldMetadata,
        expected_regions: impl IntoIterator<Item = SimulationRegionKey>,
        content_manifest: [u8; 32],
        autosave_interval_ticks: u64,
    ) -> Result<(Self, FormalWorldRecovery), FormalWorldPersistenceError> {
        let control_region = durable.control_point().snapshot().key().clone();
        let checkpoint_tick = GameTick::new(durable.control_point().committed_tick());
        let mut stores = BTreeMap::new();
        let mut points = BTreeMap::new();
        let mut resume_tick = checkpoint_tick;
        for key in expected_regions {
            if stores.contains_key(&key) {
                return Err(FormalWorldPersistenceError::DuplicateRegion(key));
            }
            let root = region_store_root(storage_root, &key)?;
            let pristine = directory_is_empty(&root)?;
            let store = RegionFileStore::open(root)?;
            let latest = store.load(&key)?;
            if latest.is_none() && !pristine {
                return Err(FormalWorldPersistenceError::ExistingStoreWithoutRegion(key));
            }
            if let Some(latest) = &latest {
                validate_header(latest, &key, content_manifest)?;
                resume_tick = resume_tick.max(GameTick::new(latest.committed_tick()));
            }
            let selected = store.load_at_or_before(&key, checkpoint_tick.get())?;
            match selected {
                Some(point) => {
                    validate_header(&point, &key, content_manifest)?;
                    if point.committed_tick() != checkpoint_tick.get() {
                        return Err(FormalWorldPersistenceError::IncompleteCheckpoint {
                            region: key,
                            expected_tick: checkpoint_tick.get(),
                            actual_tick: Some(point.committed_tick()),
                        });
                    }
                    points.insert(key.clone(), point);
                }
                None if checkpoint_tick == GameTick::ZERO && key != control_region => {}
                None => {
                    return Err(FormalWorldPersistenceError::IncompleteCheckpoint {
                        region: key,
                        expected_tick: checkpoint_tick.get(),
                        actual_tick: None,
                    });
                }
            }
            let next_revision = latest
                .as_ref()
                .map_or(Ok(PersistenceRevision::INITIAL), |point| {
                    point.persistence_revision().checked_next()
                })?;
            stores.insert(
                key,
                RegionStore {
                    store,
                    next_revision,
                },
            );
        }
        if !stores.contains_key(&control_region) {
            return Err(FormalWorldPersistenceError::MissingControlRegion);
        }
        let catch_up = resume_tick.get().saturating_sub(checkpoint_tick.get());
        if catch_up > autosave_interval_ticks {
            return Err(FormalWorldPersistenceError::RecoveryCatchUpExceeded {
                checkpoint_tick: checkpoint_tick.get(),
                resume_tick: resume_tick.get(),
                maximum: autosave_interval_ticks,
            });
        }
        Ok((
            Self {
                stores,
                captures: BTreeMap::new(),
                staged_commits: BTreeMap::new(),
                control_region,
                content_manifest,
                autosave_interval_ticks,
                checkpoint_tick,
            },
            FormalWorldRecovery {
                points,
                checkpoint_tick,
                resume_tick,
            },
        ))
    }

    pub(crate) fn capture(
        &mut self,
        report: &CompositeGatewayTickReport,
        generations: &BTreeMap<SimulationRegionKey, ActivationGeneration>,
    ) -> Result<(), FormalWorldPersistenceError> {
        if !self.staged_commits.is_empty() {
            return Err(FormalWorldPersistenceError::FlushInProgress);
        }
        if report.regions().count() != self.stores.len() || generations.len() != self.stores.len() {
            return Err(FormalWorldPersistenceError::IncompleteCapture);
        }
        let mut captures = BTreeMap::new();
        for (key, region) in report.regions() {
            if !self.stores.contains_key(key) {
                return Err(FormalWorldPersistenceError::UnexpectedRegion(key.clone()));
            }
            let generation = generations
                .get(key)
                .copied()
                .ok_or_else(|| FormalWorldPersistenceError::MissingGeneration(key.clone()))?;
            if canonical_record_hash(&region.continuity.records) != region.continuity.hash
                || region.continuity.tick != region.commit.tick
            {
                return Err(FormalWorldPersistenceError::InvalidCapture(key.clone()));
            }
            captures.insert(
                key.clone(),
                RegionCapture {
                    tick: region.continuity.tick,
                    generation,
                    continuity_hash: region.continuity.hash,
                    records: region.continuity.records.clone(),
                },
            );
        }
        let tick = captures
            .values()
            .next()
            .map(|capture| capture.tick)
            .ok_or(FormalWorldPersistenceError::IncompleteCapture)?;
        if captures.values().any(|capture| capture.tick != tick) {
            return Err(FormalWorldPersistenceError::MixedCaptureTicks);
        }
        self.captures = captures;
        Ok(())
    }

    pub(crate) fn autosave_due(&self, tick: GameTick) -> bool {
        tick > self.checkpoint_tick && tick.get().is_multiple_of(self.autosave_interval_ticks)
    }

    pub(crate) fn flush(&mut self) -> Result<Vec<FormalRegionCommit>, FormalWorldPersistenceError> {
        if self.captures.is_empty() {
            return Ok(Vec::new());
        }
        if self.captures.len() != self.stores.len() {
            return Err(FormalWorldPersistenceError::IncompleteCapture);
        }
        let tick = self
            .captures
            .values()
            .next()
            .expect("nonempty capture")
            .tick;
        let mut keys = self.stores.keys().cloned().collect::<Vec<_>>();
        keys.sort_by_key(|key| key == &self.control_region);
        let mut committed = Vec::with_capacity(keys.len());
        for key in keys {
            if self.staged_commits.contains_key(&key) {
                continue;
            }
            let capture = self
                .captures
                .get(&key)
                .expect("complete capture contains every store");
            let owned = self
                .stores
                .get_mut(&key)
                .expect("ordered key came from stores");
            let snapshot = RegionCommitSnapshot::new(
                RegionSnapshotHeader {
                    key: key.clone(),
                    generation: capture.generation,
                    committed_tick: capture.tick.get(),
                    persistence_revision: owned.next_revision,
                    region_side_chunks: REGION_SIDE_CHUNKS,
                    content_manifest: self.content_manifest,
                    state_hash: capture.continuity_hash,
                },
                capture.records.clone(),
            )?;
            let point = RegionRecoveryPoint::new(snapshot, Vec::new())?;
            let receipt = commit_current_point(&mut owned.store, &point)?;
            if receipt.revision() != owned.next_revision
                || receipt.committed_tick() != capture.tick.get()
                || receipt.digest() != point.digest()?
            {
                return Err(FormalWorldPersistenceError::ReceiptMismatch(key));
            }
            owned.next_revision = owned.next_revision.checked_next()?;
            let commit = FormalRegionCommit {
                region: key,
                point,
                receipt,
            };
            self.staged_commits
                .insert(commit.region.clone(), commit.clone());
        }
        self.checkpoint_tick = tick;
        for key in self.stores.keys() {
            committed.push(
                self.staged_commits
                    .remove(key)
                    .expect("complete flush staged every Region"),
            );
        }
        self.captures.clear();
        Ok(committed)
    }

    pub(crate) fn pending_commit_count(&self) -> usize {
        self.captures
            .values()
            .next()
            .filter(|capture| capture.tick > self.checkpoint_tick)
            .map_or(0, |_| 1)
    }
}

fn validate_header(
    point: &RegionRecoveryPoint,
    expected_key: &SimulationRegionKey,
    content_manifest: [u8; 32],
) -> Result<(), FormalWorldPersistenceError> {
    let header = point.snapshot().header();
    if &header.key != expected_key {
        return Err(FormalWorldPersistenceError::WrongRegionHeader);
    }
    if header.region_side_chunks != REGION_SIDE_CHUNKS {
        return Err(FormalWorldPersistenceError::RegionSideMismatch);
    }
    if header.content_manifest != content_manifest {
        return Err(FormalWorldPersistenceError::ContentManifestMismatch);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub(crate) enum FormalWorldPersistenceError {
    #[error("formal persistence Region is duplicated: {0:?}")]
    DuplicateRegion(SimulationRegionKey),
    #[error("formal persistence control Region is missing")]
    MissingControlRegion,
    #[error("existing formal store has no committed point for {0:?}")]
    ExistingStoreWithoutRegion(SimulationRegionKey),
    #[error(
        "formal checkpoint tick {expected_tick} is incomplete for {region:?}; selected {actual_tick:?}"
    )]
    IncompleteCheckpoint {
        region: SimulationRegionKey,
        expected_tick: u64,
        actual_tick: Option<u64>,
    },
    #[error(
        "formal recovery catch-up from {checkpoint_tick} to {resume_tick} exceeds {maximum} ticks"
    )]
    RecoveryCatchUpExceeded {
        checkpoint_tick: u64,
        resume_tick: u64,
        maximum: u64,
    },
    #[error("formal persistence capture does not contain every Region")]
    IncompleteCapture,
    #[error("formal persistence capture contains unexpected Region {0:?}")]
    UnexpectedRegion(SimulationRegionKey),
    #[error("formal persistence capture lacks generation for {0:?}")]
    MissingGeneration(SimulationRegionKey),
    #[error("formal persistence capture is invalid for {0:?}")]
    InvalidCapture(SimulationRegionKey),
    #[error("formal persistence capture mixes committed ticks")]
    MixedCaptureTicks,
    #[error("formal persistence cannot replace a capture while its flush is incomplete")]
    FlushInProgress,
    #[error("formal persistence commit receipt does not match {0:?}")]
    ReceiptMismatch(SimulationRegionKey),
    #[error("formal persistence recovery point has the wrong Region header")]
    WrongRegionHeader,
    #[error("formal persistence recovery point has the wrong Region size")]
    RegionSideMismatch,
    #[error("formal persistence recovery point has the wrong content manifest")]
    ContentManifestMismatch,
    #[error(transparent)]
    Metadata(#[from] WorldMetadataError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    StoreMigration(#[from] StoreMigrationError),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};

    use crate::config::ServerConfig;
    use crate::world_service::metadata;

    #[test]
    fn unpublished_region_prefix_selects_control_checkpoint_and_bounded_resume() {
        let temporary = tempfile::tempdir().unwrap();
        let config = ServerConfig::development_node(1, 1, 30_000, temporary.path()).unwrap();
        let config = ServerConfig::from_toml(&config.to_toml().unwrap()).unwrap();
        let manifest = [4; 32];
        let durable = metadata::load_or_create(&config, manifest).unwrap();
        let control = durable.control_point().snapshot().key().clone();
        let ahead = SimulationRegionKey::new(
            control.world(),
            control.dimension().clone(),
            RegionCoord::new(1, 0),
            RegionMappingVersion::V1,
        );
        let records = Vec::new();
        let point = RegionRecoveryPoint::new(
            RegionCommitSnapshot::new(
                RegionSnapshotHeader {
                    key: ahead.clone(),
                    generation: ActivationGeneration::INITIAL,
                    committed_tick: 2,
                    persistence_revision: PersistenceRevision::INITIAL,
                    region_side_chunks: REGION_SIDE_CHUNKS,
                    content_manifest: manifest,
                    state_hash: canonical_record_hash(&records),
                },
                records,
            )
            .unwrap(),
            Vec::new(),
        )
        .unwrap();
        let root = region_store_root(&config.config().storage.root, &ahead).unwrap();
        RegionFileStore::open(root).unwrap().commit(&point).unwrap();

        let (_persistence, recovery) = FormalWorldPersistence::open(
            &config.config().storage.root,
            &durable,
            [control.clone(), ahead.clone()],
            manifest,
            2,
        )
        .unwrap();
        assert_eq!(recovery.checkpoint_tick(), GameTick::ZERO);
        assert_eq!(recovery.resume_tick(), GameTick::new(2));
        assert!(recovery.point(&control).is_some());
        assert!(recovery.point(&ahead).is_none());
    }

    #[test]
    fn partial_flush_resumes_without_recommitting_regions_that_are_already_durable() {
        let temporary = tempfile::tempdir().unwrap();
        let config = ServerConfig::development_node(1, 1, 30_000, temporary.path()).unwrap();
        let config = ServerConfig::from_toml(&config.to_toml().unwrap()).unwrap();
        let manifest = [7; 32];
        let durable = metadata::load_or_create(&config, manifest).unwrap();
        let control = durable.control_point().snapshot().key().clone();
        let ahead = SimulationRegionKey::new(
            control.world(),
            control.dimension().clone(),
            RegionCoord::new(1, 0),
            RegionMappingVersion::V1,
        );
        let (mut persistence, _) = FormalWorldPersistence::open(
            &config.config().storage.root,
            &durable,
            [control.clone(), ahead.clone()],
            manifest,
            16,
        )
        .unwrap();
        let capture = |tick| RegionCapture {
            tick,
            generation: ActivationGeneration::INITIAL,
            continuity_hash: canonical_record_hash(&[]),
            records: Vec::new(),
        };
        persistence.captures = BTreeMap::from([
            (control.clone(), capture(GameTick::new(1))),
            (ahead.clone(), capture(GameTick::new(1))),
        ]);

        let control_root = region_store_root(&config.config().storage.root, &control).unwrap();
        let data = control_root.join("region-data.log");
        let backup = control_root.join("region-data.backup");
        fs::rename(&data, &backup).unwrap();
        fs::create_dir(&data).unwrap();
        assert!(persistence.flush().is_err());
        assert_eq!(persistence.staged_commits.len(), 1);
        assert!(persistence.staged_commits.contains_key(&ahead));

        fs::remove_dir(&data).unwrap();
        fs::rename(&backup, &data).unwrap();
        let committed = persistence.flush().unwrap();
        assert_eq!(committed.len(), 2);
        assert!(persistence.staged_commits.is_empty());
        assert!(persistence.captures.is_empty());
        let ahead_point = RegionFileStore::open(
            region_store_root(&config.config().storage.root, &ahead).unwrap(),
        )
        .unwrap()
        .load(&ahead)
        .unwrap()
        .unwrap();
        assert_eq!(
            ahead_point.persistence_revision(),
            PersistenceRevision::INITIAL,
            "retry must not append the already committed Region a second time"
        );
    }
}
