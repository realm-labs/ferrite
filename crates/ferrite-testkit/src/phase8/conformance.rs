//! Executable Phase 8 generation, boundary, and recovery conformance.

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_persistence::snapshot::PersistenceRevision;
use ferrite_persistence::store::RegionFileStore;
use ferrite_server_runtime::phase8::runtime::{Phase8RegionRuntime, Phase8RuntimeError};
use ferrite_world::durable::encode_chunk;
use ferrite_world::generation::worldgen_catalog::{WorldgenCatalog, WorldgenRecordKind};

use crate::phase8::fixtures::{config, content_manifest, generate_full, owner_of, region, runtime};

const GENERATION_CASES: usize = 64;
const BOUNDARY_CASES: usize = 8;
const SAVE_LOAD_CASES: usize = 16;
const CRASH_CASES: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldConformanceReport {
    pub golden_digest: String,
    pub catalog_records: usize,
    pub catalog_families: usize,
    pub generation_cases: usize,
    pub generation_statuses: usize,
    pub boundary_cases: usize,
    pub save_load_cases: usize,
    pub crash_cases: usize,
}

#[must_use]
pub fn run_world_conformance() -> WorldConformanceReport {
    let manifest = content_manifest();
    let (catalog_records, catalog_families) = validate_catalog();
    let golden_digest = architectural_generation_digest(manifest, 0x26_02);
    validate_generation_determinism(manifest);
    validate_boundaries(manifest);
    validate_save_load(manifest);
    validate_crash_recovery(manifest);
    WorldConformanceReport {
        golden_digest,
        catalog_records,
        catalog_families,
        generation_cases: GENERATION_CASES,
        generation_statuses: ferrite_world::generation::status::ChunkStatus::ALL.len(),
        boundary_cases: BOUNDARY_CASES,
        save_load_cases: SAVE_LOAD_CASES,
        crash_cases: CRASH_CASES,
    }
}

fn validate_catalog() -> (usize, usize) {
    let bundle = crate::phase8::fixtures::bundle();
    let catalog = WorldgenCatalog::from_bundle(&bundle).expect("worldgen registry is present");
    catalog
        .validate_wgen_001_inventory()
        .expect("WGEN-001 catalog inventory is exact");
    catalog
        .validate_wgen_003_inventory()
        .expect("WGEN-003 catalog inventory is exact");
    let kinds = WorldgenRecordKind::ALL_WGEN_001
        .into_iter()
        .chain(WorldgenRecordKind::ALL_WGEN_003);
    let counts = kinds
        .map(WorldgenRecordKind::locked_count)
        .collect::<Vec<_>>();
    (counts.iter().sum(), counts.len())
}

fn validate_generation_determinism(manifest: [u8; 32]) {
    for case in 0..GENERATION_CASES {
        let seed = 0xa076_1d64_78bd_642f_u64.wrapping_mul(case as u64 + 1);
        let forward = generation_records(manifest, seed, false);
        let reverse = generation_records(manifest, seed, true);
        assert_eq!(forward, reverse, "dispatch order changed case {case}");
        assert_eq!(forward, generation_records(manifest, seed, false));
    }
}

fn generation_records(manifest: [u8; 32], seed: u64, reverse: bool) -> Vec<Vec<u8>> {
    let mut runtime = runtime(0, manifest);
    let mut chunks = [
        ChunkPos::new(0, 0),
        ChunkPos::new(7, 0),
        ChunkPos::new(0, 7),
        ChunkPos::new(7, 7),
    ];
    if reverse {
        chunks.reverse();
    }
    for chunk in chunks {
        generate_full(&mut runtime, chunk, seed);
    }
    runtime
        .prepare_save(80, PersistenceRevision::INITIAL)
        .expect("deterministic generated state is saveable")
        .records()
        .iter()
        .map(|record| record.value().to_vec())
        .collect()
}

#[must_use]
pub fn architectural_generation_digest(manifest: [u8; 32], seed: u64) -> String {
    let mut hasher = blake3::Hasher::new();
    for record in generation_records(manifest, seed, false) {
        hasher.update(&(record.len() as u64).to_be_bytes());
        hasher.update(&record);
    }
    hasher.finalize().to_hex().to_string()
}

fn validate_boundaries(manifest: [u8; 32]) {
    let coordinates = [-9, -8, -1, 0, 7, 8, 15, 16];
    for (case, x) in coordinates.into_iter().enumerate() {
        let chunk = ChunkPos::new(x, 0);
        let key = owner_of(chunk);
        let mut owned_runtime = Phase8RegionRuntime::new(
            key.clone(),
            ActivationGeneration::INITIAL,
            config(manifest, 128),
        )
        .expect("boundary owner is valid");
        generate_full(&mut owned_runtime, chunk, case as u64);
        let encoded =
            encode_chunk(owned_runtime.chunk(chunk).expect("chunk remains loaded")).unwrap();
        let point = owned_runtime
            .prepare_save(case as u64 + 1, PersistenceRevision::INITIAL)
            .unwrap();
        let restored = Phase8RegionRuntime::restore(
            key,
            ActivationGeneration::new(2).unwrap(),
            point.recovery_point(),
            config(manifest, 128),
        )
        .unwrap();
        assert_eq!(
            encode_chunk(restored.chunk(chunk).expect("boundary chunk restores")).unwrap(),
            encoded
        );
        let mut wrong_owner = runtime(0, manifest);
        if owner_of(chunk) != region(0) {
            assert!(wrong_owner.demand_chunk(chunk).is_err());
        }
    }
}

fn validate_save_load(manifest: [u8; 32]) {
    for case in 0..SAVE_LOAD_CASES {
        let directory = tempfile::tempdir().unwrap();
        let chunk = ChunkPos::new(case as i32 % 8, case as i32 / 8);
        let mut source = runtime(0, manifest);
        generate_full(&mut source, chunk, case as u64);
        let prepared = source
            .prepare_save(case as u64 + 1, PersistenceRevision::INITIAL)
            .unwrap();
        let mut store = RegionFileStore::open(directory.path()).unwrap();
        store.commit(prepared.recovery_point()).unwrap();
        let loaded = store.load(source.key()).unwrap().unwrap();
        let restored = Phase8RegionRuntime::restore(
            source.key().clone(),
            ActivationGeneration::new(2).unwrap(),
            &loaded,
            config(manifest, 128),
        )
        .unwrap();
        assert_eq!(restored.lifecycle(chunk), source.lifecycle(chunk));
        assert_eq!(
            encode_chunk(restored.chunk(chunk).unwrap()).unwrap(),
            encode_chunk(source.chunk(chunk).unwrap()).unwrap()
        );
    }
}

fn validate_crash_recovery(manifest: [u8; 32]) {
    let mut source = runtime(0, manifest);
    generate_full(&mut source, ChunkPos::new(0, 0), 1);
    let prepared = source
        .prepare_save(1, PersistenceRevision::INITIAL)
        .unwrap();
    assert!(matches!(
        Phase8RegionRuntime::restore(
            source.key().clone(),
            ActivationGeneration::new(2).unwrap(),
            prepared.recovery_point(),
            config([0xff; 32], 128),
        ),
        Err(Phase8RuntimeError::ContentManifestMismatch)
    ));
    assert!(matches!(
        Phase8RegionRuntime::restore(
            source.key().clone(),
            ActivationGeneration::INITIAL,
            prepared.recovery_point(),
            config(manifest, 128),
        ),
        Err(Phase8RuntimeError::GenerationNotNewer)
    ));
    assert!(matches!(
        Phase8RegionRuntime::restore(
            region(1),
            ActivationGeneration::new(2).unwrap(),
            prepared.recovery_point(),
            config(manifest, 128),
        ),
        Err(Phase8RuntimeError::WrongRegion)
    ));

    let torn = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(torn.path()).unwrap();
    store.commit(prepared.recovery_point()).unwrap();
    OpenOptions::new()
        .append(true)
        .open(torn.path().join("region-journal.log"))
        .unwrap()
        .write_all(b"FR")
        .unwrap();
    assert_eq!(
        store.load(source.key()).unwrap().as_ref(),
        Some(prepared.recovery_point())
    );

    let corrupt = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(corrupt.path()).unwrap();
    store.commit(prepared.recovery_point()).unwrap();
    let mut data = OpenOptions::new()
        .read(true)
        .write(true)
        .open(corrupt.path().join("region-data.log"))
        .unwrap();
    data.seek(SeekFrom::End(-1)).unwrap();
    let mut last = [0];
    data.read_exact(&mut last).unwrap();
    data.seek(SeekFrom::End(-1)).unwrap();
    data.write_all(&[last[0] ^ 0xff]).unwrap();
    assert!(store.load(source.key()).is_err());

    let mismatch = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(mismatch.path()).unwrap();
    let mut other = runtime(1, manifest);
    generate_full(&mut other, ChunkPos::new(8, 0), 2);
    let other_save = other.prepare_save(1, PersistenceRevision::INITIAL).unwrap();
    let wrong_receipt = store.commit(other_save.recovery_point()).unwrap();
    assert!(matches!(
        source.apply_save_receipt(prepared, wrong_receipt),
        Err(Phase8RuntimeError::SaveReceiptMismatch)
    ));
}
