//! Executable Phase 8 root-surface conformance.

use std::collections::BTreeMap;

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, StableEntityId, WorldId};
use ferrite_foundation::region::RegionMappingVersion;
use ferrite_persistence::snapshot::PersistenceRevision;
use ferrite_persistence::store::RegionFileStore;
use ferrite_server_runtime::phase8::lifecycle::{
    PrepareOutcome, WorldLifecycleEvent, WorldLifecycleRuntime, WorldLifecycleState,
};
use ferrite_server_runtime::phase8::runtime::{Phase8RegionRuntime, Phase8RuntimeError};
use ferrite_server_runtime::player_service::model::PlayerPersistentState;
use ferrite_server_runtime::player_service::runtime::PlayerServiceRegionRuntime;
use ferrite_server_runtime::simulation::continuity::SimulationContinuity;
use ferrite_world::generation::status::ChunkStatus;
use ferrite_world::generation::worldgen_catalog::{WorldgenCatalog, WorldgenRecordKind};

use crate::phase8::fixtures::{
    bundle, config, content_manifest, dimension, generate_full, region, runtime,
};

const CONTENT_CASES: usize = 32;
const LIFECYCLE_CASES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDispatchReport {
    pub catalog_records: usize,
    pub catalog_families: usize,
    pub deterministic_cases: usize,
    pub manifest_fences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceReloadReport {
    pub chunk_records: usize,
    pub auxiliary_records: usize,
    pub restored_players: usize,
    pub restored_schedulers: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLifecycleReport {
    pub golden_digest: String,
    pub property_cases: usize,
    pub dimensions: usize,
    pub bootstrap_events: usize,
    pub shutdown_events: usize,
}

#[must_use]
pub fn run_content_dispatch_surface() -> ContentDispatchReport {
    let bundle = bundle();
    let catalog = WorldgenCatalog::from_bundle(&bundle).unwrap();
    catalog.validate_wgen_001_inventory().unwrap();
    catalog.validate_wgen_003_inventory().unwrap();
    let kinds = WorldgenRecordKind::ALL_WGEN_001
        .into_iter()
        .chain(WorldgenRecordKind::ALL_WGEN_003)
        .collect::<Vec<_>>();
    let manifest = *bundle.content_manifest().unwrap().digest().as_bytes();
    for case in 0..CONTENT_CASES {
        let chunk = ChunkPos::new(case as i32 % 8, case as i32 / 8);
        let mut first = runtime(0, manifest);
        let mut second = runtime(0, manifest);
        generate_full(&mut first, chunk, case as u64);
        generate_full(&mut second, chunk, case as u64);
        assert_eq!(
            first
                .prepare_save(1, PersistenceRevision::INITIAL)
                .unwrap()
                .records(),
            second
                .prepare_save(1, PersistenceRevision::INITIAL)
                .unwrap()
                .records()
        );
    }

    let mut fenced = runtime(0, manifest);
    let chunk = ChunkPos::new(0, 0);
    fenced.demand_chunk(chunk).unwrap();
    let request = fenced
        .begin_generation(chunk, ChunkStatus::StructureStarts)
        .unwrap();
    let mut wrong_result = request.clone().complete(request.source.clone());
    wrong_result.content_manifest = [0xff; 32];
    assert!(matches!(
        fenced.apply_generated(wrong_result),
        Err(Phase8RuntimeError::ContentManifestMismatch)
    ));
    let save = fenced.prepare_save(1, PersistenceRevision::INITIAL);
    assert!(save.is_err(), "in-flight dispatch must not become durable");

    ContentDispatchReport {
        catalog_records: kinds.iter().map(|kind| kind.locked_count()).sum(),
        catalog_families: kinds.len(),
        deterministic_cases: CONTENT_CASES,
        manifest_fences: 2,
    }
}

#[must_use]
pub fn run_persistence_reload_surface() -> PersistenceReloadReport {
    let manifest = content_manifest();
    let chunk = ChunkPos::new(0, 0);
    let mut world = runtime(0, manifest);
    generate_full(&mut world, chunk, 26);

    let player = StableEntityId::new(1).unwrap();
    let mut players =
        PlayerServiceRegionRuntime::new(region(0), ActivationGeneration::INITIAL, 8, 8).unwrap();
    players
        .join(player, PlayerPersistentState::default())
        .unwrap();
    let player_records = players.capture_continuity().unwrap();
    let scheduler_records = crate::simulation::fixtures::simulation_runtime(0)
        .capture_continuity()
        .unwrap()
        .to_records()
        .unwrap();
    let mut auxiliary = scheduler_records.clone();
    auxiliary.extend(player_records.clone());
    world.replace_auxiliary_records(auxiliary).unwrap();

    let prepared = world
        .prepare_save(20, PersistenceRevision::INITIAL)
        .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(directory.path()).unwrap();
    store.commit(prepared.recovery_point()).unwrap();
    let loaded = store.load(world.key()).unwrap().unwrap();
    let restored = Phase8RegionRuntime::restore(
        world.key().clone(),
        ActivationGeneration::new(2).unwrap(),
        &loaded,
        config(manifest, 256),
    )
    .unwrap();
    let restored_records = restored
        .prepare_save(21, PersistenceRevision::new(2).unwrap())
        .unwrap()
        .records()
        .to_vec();
    let restored_players = PlayerServiceRegionRuntime::restore(
        region(0),
        ActivationGeneration::new(2).unwrap(),
        8,
        8,
        &restored_records,
    )
    .unwrap();
    assert_eq!(
        restored_players.state(player),
        Some(PlayerPersistentState {
            last_session_epoch: 2,
            ..PlayerPersistentState::default()
        })
    );
    let scheduler = SimulationContinuity::from_records(&restored_records).unwrap();
    assert_eq!(
        scheduler,
        SimulationContinuity::from_records(&scheduler_records).unwrap()
    );

    PersistenceReloadReport {
        chunk_records: 1,
        auxiliary_records: player_records.len() + scheduler_records.len(),
        restored_players: 1,
        restored_schedulers: 1,
    }
}

#[must_use]
pub fn run_world_lifecycle_surface() -> WorldLifecycleReport {
    let manifest = content_manifest();
    let mut golden = Vec::new();
    for case in 0..LIFECYCLE_CASES {
        let dimensions = [dimension("the_nether"), dimension("the_end")];
        let mut lifecycle = WorldLifecycleRuntime::bootstrap(
            WorldId::new(case as u128 + 1).unwrap(),
            RegionMappingVersion::V1,
            dimension("overworld"),
            dimensions.clone(),
            ActivationGeneration::INITIAL,
            manifest,
            64,
        )
        .unwrap();
        assert_eq!(
            lifecycle.dimensions(),
            [
                dimension("overworld"),
                dimensions[0].clone(),
                dimensions[1].clone()
            ]
        );
        lifecycle.set_pending_work(&dimensions[0], 1).unwrap();
        assert_eq!(
            lifecycle.prepare_levels().unwrap(),
            PrepareOutcome::Waiting { pending_work: 1 }
        );
        lifecycle.set_pending_work(&dimensions[0], 0).unwrap();
        assert_eq!(lifecycle.prepare_levels().unwrap(), PrepareOutcome::Ready);
        let bootstrap = lifecycle.take_events(usize::MAX);
        lifecycle.begin_shutdown(case).unwrap();
        let first_shutdown = lifecycle.take_events(usize::MAX);
        let results = lifecycle
            .dimensions()
            .iter()
            .cloned()
            .map(|dimension| (dimension, case & 1 == 0))
            .collect::<BTreeMap<_, _>>();
        lifecycle.finish_shutdown(&results).unwrap();
        let second_shutdown = lifecycle.take_events(usize::MAX);
        assert_eq!(lifecycle.state(), WorldLifecycleState::Closed);
        if case == 0 {
            golden.extend(bootstrap.iter().map(event_tag));
            golden.extend(first_shutdown.iter().map(event_tag));
            golden.extend(second_shutdown.iter().map(event_tag));
        }
    }
    WorldLifecycleReport {
        golden_digest: blake3::hash(&golden).to_hex().to_string(),
        property_cases: LIFECYCLE_CASES,
        dimensions: 3,
        bootstrap_events: 9,
        shutdown_events: 18,
    }
}

const fn event_tag(event: &WorldLifecycleEvent) -> u8 {
    match event {
        WorldLifecycleEvent::LevelConstructed { .. } => 0,
        WorldLifecycleEvent::TicketsReactivated { .. } => 1,
        WorldLifecycleEvent::LevelReady { .. } => 2,
        WorldLifecycleEvent::NetworkAdmissionClosed => 3,
        WorldLifecycleEvent::PlayersSaved { .. } => 4,
        WorldLifecycleEvent::LevelsSaved => 5,
        WorldLifecycleEvent::PlayersRemoved { .. } => 6,
        WorldLifecycleEvent::NoSaveCleared { .. } => 7,
        WorldLifecycleEvent::ClosingTicketsDeactivated { .. } => 8,
        WorldLifecycleEvent::WorkDrained => 9,
        WorldLifecycleEvent::LevelsFlushed => 10,
        WorldLifecycleEvent::LevelClosed { .. } => 11,
        WorldLifecycleEvent::SavedDataClosed => 12,
        WorldLifecycleEvent::ResourcesClosed => 13,
        WorldLifecycleEvent::StorageLockClosed => 14,
    }
}
