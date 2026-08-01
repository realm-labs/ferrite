//! Executable Phase 8 cross-system join conformance.

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, StableEntityId, WorldId};
use ferrite_foundation::region::RegionMappingVersion;
use ferrite_persistence::snapshot::PersistenceRevision;
use ferrite_persistence::store::RegionFileStore;
use ferrite_server_runtime::phase5::continuity::Phase5Continuity;
use ferrite_server_runtime::phase6::model::PlayerPersistentState;
use ferrite_server_runtime::phase6::runtime::Phase6RegionRuntime;
use ferrite_server_runtime::phase8::lifecycle::{
    PrepareOutcome, WorldLifecycleEvent, WorldLifecycleRuntime, WorldLifecycleState,
};
use ferrite_server_runtime::phase8::model::{ChunkActivity, ChunkEventKind};
use ferrite_server_runtime::phase8::runtime::{Phase8RegionRuntime, Phase8RuntimeError};

use crate::phase6::fixtures::join_command;
use crate::phase8::fixtures::{
    config, content_manifest, dimension, generate_full, region, runtime,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldJoinReport {
    pub checkpoints: usize,
    pub rejected_faults: usize,
}

#[must_use]
pub fn run_content_dispatch_persistence_reload() -> WorldJoinReport {
    let manifest = content_manifest();
    let chunk = ChunkPos::new(0, 0);
    let mut source = runtime(0, manifest);
    generate_full(&mut source, chunk, 1);
    let prepared = source
        .prepare_save(1, PersistenceRevision::INITIAL)
        .unwrap();
    let restored = Phase8RegionRuntime::restore(
        region(0),
        ActivationGeneration::new(2).unwrap(),
        prepared.recovery_point(),
        config(manifest, 128),
    )
    .unwrap();
    assert_eq!(restored.lifecycle(chunk), source.lifecycle(chunk));
    assert!(matches!(
        Phase8RegionRuntime::restore(
            region(0),
            ActivationGeneration::new(2).unwrap(),
            prepared.recovery_point(),
            config([0xff; 32], 128),
        ),
        Err(Phase8RuntimeError::ContentManifestMismatch)
    ));
    WorldJoinReport {
        checkpoints: 2,
        rejected_faults: 1,
    }
}

#[must_use]
pub fn run_content_dispatch_player_lifecycle() -> WorldJoinReport {
    let manifest = content_manifest();
    let chunk = ChunkPos::new(0, 0);
    let mut world = runtime(0, manifest);
    generate_full(&mut world, chunk, 2);
    assert_eq!(
        world.lifecycle(chunk).unwrap().activity,
        ChunkActivity::EntityTicking
    );
    let player = StableEntityId::new(1).unwrap();
    let mut players =
        Phase6RegionRuntime::new(region(0), ActivationGeneration::INITIAL, 1, 4).unwrap();
    players
        .join(player, PlayerPersistentState::default())
        .unwrap();
    assert!(players.state(player).is_some());
    assert!(
        players
            .join(player, PlayerPersistentState::default())
            .is_err()
    );
    WorldJoinReport {
        checkpoints: 2,
        rejected_faults: 1,
    }
}

#[must_use]
pub fn run_content_dispatch_world_lifecycle() -> WorldJoinReport {
    let manifest = content_manifest();
    let mut lifecycle = lifecycle(manifest);
    assert_eq!(lifecycle.content_manifest(), manifest);
    assert_eq!(lifecycle.prepare_levels().unwrap(), PrepareOutcome::Ready);
    let world = runtime(0, manifest);
    assert_eq!(world.key().world(), lifecycle.world());
    WorldJoinReport {
        checkpoints: 3,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_network_ingress_content_dispatch() -> WorldJoinReport {
    let manifest = content_manifest();
    let command = join_command(1, 1, 0);
    let mut world = runtime(0, manifest);
    generate_full(&mut world, ChunkPos::new(0, 0), 3);
    assert_eq!(command.target(), world.key());
    assert_eq!(
        world.lifecycle(ChunkPos::new(0, 0)).unwrap().activity,
        ChunkActivity::EntityTicking
    );
    WorldJoinReport {
        checkpoints: 2,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_network_ingress_persistence_reload() -> WorldJoinReport {
    let manifest = content_manifest();
    let mut world = runtime(0, manifest);
    generate_full(&mut world, ChunkPos::new(0, 0), 4);
    let prepared = world.prepare_save(1, PersistenceRevision::INITIAL).unwrap();
    let restored = Phase8RegionRuntime::restore(
        region(0),
        ActivationGeneration::new(2).unwrap(),
        prepared.recovery_point(),
        config(manifest, 128),
    )
    .unwrap();
    let command = join_command(1, 2, 0);
    assert_eq!(command.target(), restored.key());
    assert!(restored.chunk(ChunkPos::new(0, 0)).is_some());
    WorldJoinReport {
        checkpoints: 2,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_network_ingress_world_lifecycle() -> WorldJoinReport {
    let manifest = content_manifest();
    let mut world = lifecycle(manifest);
    let command = join_command(1, 1, 0);
    assert_eq!(world.state(), WorldLifecycleState::Bootstrapping);
    assert_eq!(world.prepare_levels().unwrap(), PrepareOutcome::Ready);
    assert_eq!(world.state(), WorldLifecycleState::Running);
    assert_eq!(command.target().world(), world.world());
    WorldJoinReport {
        checkpoints: 3,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_player_lifecycle_persistence_reload() -> WorldJoinReport {
    let report = crate::phase8::surfaces::run_persistence_reload_surface();
    assert_eq!(report.restored_players, 1);
    WorldJoinReport {
        checkpoints: report.chunk_records + report.auxiliary_records,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_player_lifecycle_world_lifecycle() -> WorldJoinReport {
    let manifest = content_manifest();
    let mut world = lifecycle(manifest);
    world.prepare_levels().unwrap();
    let player = StableEntityId::new(1).unwrap();
    let mut players =
        Phase6RegionRuntime::new(region(0), ActivationGeneration::INITIAL, 2, 4).unwrap();
    players
        .join(player, PlayerPersistentState::default())
        .unwrap();
    world.begin_shutdown(1).unwrap();
    let events = world.take_events(usize::MAX);
    let saved = events
        .iter()
        .position(|event| matches!(event, WorldLifecycleEvent::PlayersSaved { count: 1 }))
        .unwrap();
    let removed = events
        .iter()
        .position(|event| matches!(event, WorldLifecycleEvent::PlayersRemoved { count: 1 }))
        .unwrap();
    assert!(saved < removed);
    WorldJoinReport {
        checkpoints: 2,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_tick_scheduler_content_dispatch() -> WorldJoinReport {
    let manifest = content_manifest();
    let chunk = ChunkPos::new(0, 0);
    let mut world = runtime(0, manifest);
    generate_full(&mut world, chunk, 5);
    let events = world.take_events(usize::MAX);
    let unpack = events
        .iter()
        .position(|event| event.kind == ChunkEventKind::PersistedTicksUnpacked)
        .unwrap();
    let ticking = events
        .iter()
        .position(|event| event.kind == ChunkEventKind::BlockTicking)
        .unwrap();
    assert!(unpack < ticking);
    WorldJoinReport {
        checkpoints: 2,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_tick_scheduler_persistence_reload() -> WorldJoinReport {
    let manifest = content_manifest();
    let scheduler = crate::phase5::fixtures::phase5_runtime(0)
        .capture_continuity()
        .unwrap();
    let records = scheduler.to_records().unwrap();
    let mut world = runtime(0, manifest);
    generate_full(&mut world, ChunkPos::new(0, 0), 6);
    world.replace_auxiliary_records(records.clone()).unwrap();
    let prepared = world.prepare_save(1, PersistenceRevision::INITIAL).unwrap();
    let restored = Phase8RegionRuntime::restore(
        region(0),
        ActivationGeneration::new(2).unwrap(),
        prepared.recovery_point(),
        config(manifest, 128),
    )
    .unwrap();
    let materialized = restored
        .prepare_save(2, PersistenceRevision::new(2).unwrap())
        .unwrap()
        .records()
        .to_vec();
    assert_eq!(
        Phase5Continuity::from_records(&materialized).unwrap(),
        scheduler
    );
    WorldJoinReport {
        checkpoints: records.len() + 1,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_tick_scheduler_world_lifecycle() -> WorldJoinReport {
    let manifest = content_manifest();
    let mut world = lifecycle(manifest);
    let overworld = dimension("overworld");
    world.set_pending_work(&overworld, 1).unwrap();
    assert_eq!(
        world.prepare_levels().unwrap(),
        PrepareOutcome::Waiting { pending_work: 1 }
    );
    world.set_pending_work(&overworld, 0).unwrap();
    assert_eq!(world.prepare_levels().unwrap(), PrepareOutcome::Ready);
    WorldJoinReport {
        checkpoints: 2,
        rejected_faults: 0,
    }
}

#[must_use]
pub fn run_world_lifecycle_persistence_reload() -> WorldJoinReport {
    let manifest = content_manifest();
    let mut source_lifecycle = lifecycle(manifest);
    source_lifecycle.prepare_levels().unwrap();
    let control = source_lifecycle
        .level(&dimension("overworld"))
        .unwrap()
        .control_region
        .clone();
    source_lifecycle
        .set_no_save(&control, ActivationGeneration::INITIAL, true)
        .unwrap();
    let records = source_lifecycle.level_records().unwrap();

    let mut world = runtime(0, manifest);
    generate_full(&mut world, ChunkPos::new(0, 0), 7);
    world.replace_auxiliary_records(records.clone()).unwrap();
    let prepared = world.prepare_save(1, PersistenceRevision::INITIAL).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(directory.path()).unwrap();
    store.commit(prepared.recovery_point()).unwrap();
    let recovered = store.load(world.key()).unwrap().unwrap();
    let restored_world = Phase8RegionRuntime::restore(
        region(0),
        ActivationGeneration::new(2).unwrap(),
        &recovered,
        config(manifest, 128),
    )
    .unwrap();
    let restored_records = restored_world
        .prepare_save(2, PersistenceRevision::new(2).unwrap())
        .unwrap()
        .records()
        .to_vec();
    let mut restored_lifecycle = lifecycle(manifest);
    restored_lifecycle
        .apply_level_records(&restored_records)
        .unwrap();
    assert!(
        restored_lifecycle
            .level(&dimension("overworld"))
            .unwrap()
            .no_save
    );
    WorldJoinReport {
        checkpoints: records.len() + 2,
        rejected_faults: 0,
    }
}

fn lifecycle(manifest: [u8; 32]) -> WorldLifecycleRuntime {
    WorldLifecycleRuntime::bootstrap(
        WorldId::new(1).unwrap(),
        RegionMappingVersion::V1,
        dimension("overworld"),
        [dimension("the_nether"), dimension("the_end")],
        ActivationGeneration::INITIAL,
        manifest,
        64,
    )
    .unwrap()
}
