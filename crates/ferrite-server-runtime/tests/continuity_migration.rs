use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{
    PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
    SnapshotRecord,
};
use ferrite_persistence::store::RegionFileStore;
use ferrite_server_runtime::continuity::identity::{
    ContinuityDomain, ContinuityGeneration, classify_domain, domain_id,
};
use ferrite_server_runtime::continuity::migration::{
    ContinuityMigrationError, PreparedStoreMigration, StoreMigrationError, canonical_record_hash,
    commit_current_point, normalize_records, normalize_recovery_point, prepare_store_migration,
};

fn region_key() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(2, -3),
        RegionMappingVersion::V1,
    )
}

fn records(generation: ContinuityGeneration) -> Vec<SnapshotRecord> {
    ContinuityDomain::ALL
        .into_iter()
        .enumerate()
        .map(|(index, domain)| {
            SnapshotRecord::new(
                domain.record_kind(),
                domain_id(domain, generation),
                vec![index as u8],
                vec![0xa0 | index as u8],
            )
            .unwrap()
        })
        .collect()
}

fn point(
    continuity_generation: ContinuityGeneration,
    revision: PersistenceRevision,
) -> RegionRecoveryPoint {
    let records = records(continuity_generation);
    let snapshot = RegionCommitSnapshot::new(
        RegionSnapshotHeader {
            key: region_key(),
            generation: ActivationGeneration::INITIAL,
            committed_tick: 10,
            persistence_revision: revision,
            region_side_chunks: 8,
            content_manifest: [7; 32],
            state_hash: canonical_record_hash(&records),
        },
        records,
    )
    .unwrap();
    RegionRecoveryPoint::new(snapshot, Vec::new()).unwrap()
}

#[test]
fn clean_old_point_migrates_every_identity_and_canonical_hash() {
    let legacy = point(ContinuityGeneration::Legacy, PersistenceRevision::INITIAL);
    let migrated = normalize_recovery_point(&legacy).unwrap();

    assert_eq!(
        migrated.persistence_revision(),
        PersistenceRevision::INITIAL
    );
    assert_ne!(
        migrated.snapshot().header().state_hash,
        legacy.snapshot().header().state_hash
    );
    assert_eq!(
        migrated.snapshot().header().state_hash,
        canonical_record_hash(migrated.snapshot().records())
    );
    for after in migrated.snapshot().records() {
        let before = legacy
            .snapshot()
            .records()
            .iter()
            .find(|record| record.key() == after.key())
            .unwrap();
        assert_eq!(before.kind(), after.kind());
        assert_eq!(before.key(), after.key());
        assert_eq!(before.value(), after.value());
        assert_eq!(
            classify_domain(after.domain()).unwrap().generation,
            ContinuityGeneration::Current
        );
    }
}

#[test]
fn clean_new_point_is_idempotent() {
    let current = point(ContinuityGeneration::Current, PersistenceRevision::INITIAL);
    assert_eq!(normalize_recovery_point(&current).unwrap(), current);
    let normalized = normalize_records(current.snapshot().records()).unwrap();
    assert!(!normalized.was_migrated());
}

#[test]
fn interrupted_prepare_leaves_legacy_commit_and_retry_is_safe() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(directory.path()).unwrap();
    let legacy = point(ContinuityGeneration::Legacy, PersistenceRevision::INITIAL);
    store.commit(&legacy).unwrap();

    let prepared = prepare_store_migration(&store, &region_key()).unwrap();
    let PreparedStoreMigration::Prepared(plan) = prepared else {
        panic!("legacy point must require migration");
    };
    assert_eq!(plan.candidate().persistence_revision().get(), 2);
    drop(plan);
    assert_eq!(store.load(&region_key()).unwrap().unwrap(), legacy);

    let PreparedStoreMigration::Prepared(plan) =
        prepare_store_migration(&store, &region_key()).unwrap()
    else {
        panic!("retry must prepare the same migration");
    };
    plan.commit(&mut store).unwrap();
    let current = store.load(&region_key()).unwrap().unwrap();
    assert_eq!(current.persistence_revision().get(), 2);
    assert!(current.snapshot().records().iter().all(|record| {
        classify_domain(record.domain()).unwrap().generation == ContinuityGeneration::Current
    }));
    assert!(matches!(
        prepare_store_migration(&store, &region_key()).unwrap(),
        PreparedStoreMigration::AlreadyCurrent(point) if point == current
    ));
}

#[test]
fn mixed_and_unsupported_generations_fail_closed() {
    let mut mixed = records(ContinuityGeneration::Legacy);
    mixed[0] = records(ContinuityGeneration::Current).remove(0);
    assert!(matches!(
        normalize_records(&mixed),
        Err(ContinuityMigrationError::MixedGenerations)
    ));

    let unsupported = SnapshotRecord::new(
        ContinuityDomain::WorldChunk.record_kind(),
        ResourceId::new("ferrite", "world-service/chunk_v2").unwrap(),
        vec![1],
        vec![2],
    )
    .unwrap();
    assert!(matches!(
        normalize_records(&[unsupported]),
        Err(ContinuityMigrationError::UnsupportedIdentity(identity))
            if identity == "ferrite:world-service/chunk_v2"
    ));
}

#[test]
fn duplicate_canonical_identity_is_rejected() {
    let record = records(ContinuityGeneration::Legacy).remove(0);
    assert!(matches!(
        normalize_records(&[record.clone(), record]),
        Err(ContinuityMigrationError::DuplicateIdentity { .. })
    ));
}

#[test]
fn rollback_to_legacy_identity_is_denied_without_advancing_store() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(directory.path()).unwrap();
    let legacy = point(ContinuityGeneration::Legacy, PersistenceRevision::INITIAL);
    store.commit(&legacy).unwrap();
    let PreparedStoreMigration::Prepared(plan) =
        prepare_store_migration(&store, &region_key()).unwrap()
    else {
        panic!("legacy point must require migration");
    };
    plan.commit(&mut store).unwrap();

    let rollback = point(
        ContinuityGeneration::Legacy,
        PersistenceRevision::new(3).unwrap(),
    );
    assert!(matches!(
        commit_current_point(&mut store, &rollback),
        Err(StoreMigrationError::LegacyWriteDenied)
    ));
    assert_eq!(
        store
            .load(&region_key())
            .unwrap()
            .unwrap()
            .persistence_revision()
            .get(),
        2
    );
}
