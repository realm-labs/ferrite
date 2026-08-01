use std::collections::BTreeMap;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::recovery::RegionHandoffState;
use ferrite_persistence::snapshot::{PersistenceRevision, SnapshotRecord, SnapshotRecordKind};
use ferrite_persistence::store::RegionFileStore;
use ferrite_server_runtime::phase8::inspection::inspect_recovery_point;
use ferrite_server_runtime::phase8::lifecycle::{
    LevelLifecycleState, PrepareOutcome, WorldLifecycleEvent, WorldLifecycleRuntime,
    WorldLifecycleState,
};
use ferrite_server_runtime::phase8::model::{
    ChunkActivity, ChunkEventKind, GenerationOutcome, Phase8RuntimeConfig, TicketOutcome,
};
use ferrite_server_runtime::phase8::runtime::{Phase8RegionRuntime, Phase8RuntimeError};
use ferrite_world::chunk::{ChunkColumn, ChunkLayout, VerticalSectionRange};
use ferrite_world::durable::{decode_chunk, encode_chunk};
use ferrite_world::generation::status::ChunkStatus;
use ferrite_world::id::{BiomeId, BlockStateId};

fn key() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

fn layout() -> ChunkLayout {
    ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).unwrap(),
        BlockStateId::new(0),
        BiomeId::new(1),
    )
}

fn config(event_capacity: usize) -> Phase8RuntimeConfig {
    Phase8RuntimeConfig {
        mapping: RegionMapping::V1,
        layout: layout(),
        region_side_chunks: 8,
        chunk_capacity: 64,
        event_capacity,
        content_manifest: [7; 32],
    }
}

fn runtime_with_capacity(event_capacity: usize) -> Phase8RegionRuntime {
    Phase8RegionRuntime::new(key(), ActivationGeneration::INITIAL, config(event_capacity)).unwrap()
}

fn advance_to_full(runtime: &mut Phase8RegionRuntime, position: ChunkPos) {
    for target in ChunkStatus::ALL.into_iter().skip(1) {
        let request = runtime.begin_generation(position, target).unwrap();
        let generated = request.source.clone();
        assert!(matches!(
            runtime
                .apply_generated(request.complete(generated))
                .unwrap(),
            GenerationOutcome::Published { .. }
        ));
    }
}

#[test]
fn durable_chunk_round_trip_preserves_sparse_values_entities_and_revisions() {
    let mut chunk = ChunkColumn::new(ChunkPos::new(-1, 2), layout());
    chunk
        .set_uniform_section(-1, BlockStateId::new(4), BiomeId::new(9))
        .unwrap();
    chunk
        .set_block(BlockPos::new(-1, -1, 32), BlockStateId::new(12))
        .unwrap();
    chunk
        .insert_block_entity(
            BlockPos::new(-2, -2, 33),
            ResourceId::minecraft("chest").unwrap(),
        )
        .unwrap();
    let encoded = encode_chunk(&chunk).unwrap();
    assert_eq!(decode_chunk(&encoded).unwrap(), chunk);
    assert!(decode_chunk(&encoded[..encoded.len() - 1]).is_err());
    let mut trailing = encoded;
    trailing.push(0);
    assert!(decode_chunk(&trailing).is_err());
}

#[test]
fn asynchronous_generation_is_fenced_by_region_generation_revision_and_content() {
    let position = ChunkPos::new(0, 0);
    let mut runtime = runtime_with_capacity(64);
    assert_eq!(
        runtime.demand_chunk(position).unwrap(),
        TicketOutcome::Loaded
    );
    let request = runtime
        .begin_generation(position, ChunkStatus::StructureStarts)
        .unwrap();
    let mut stale_result = request.clone().complete(request.source.clone());
    let expected = runtime.chunk(position).unwrap().revision();
    runtime
        .set_block(
            &key(),
            ActivationGeneration::INITIAL,
            expected,
            BlockPos::new(0, 0, 0),
            BlockStateId::new(8),
        )
        .unwrap();
    assert_eq!(
        runtime.apply_generated(stale_result.clone()).unwrap(),
        GenerationOutcome::StaleRevision {
            expected: 0,
            actual: 1,
        }
    );
    assert_eq!(
        runtime.lifecycle(position).unwrap().status,
        ChunkStatus::Empty
    );

    let request = runtime
        .begin_generation(position, ChunkStatus::StructureStarts)
        .unwrap();
    stale_result = request.clone().complete(request.source.clone());
    stale_result.content_manifest = [9; 32];
    assert!(matches!(
        runtime.apply_generated(stale_result),
        Err(Phase8RuntimeError::ContentManifestMismatch)
    ));
    let mut wrong_generation = request.clone().complete(request.source.clone());
    wrong_generation.generation = ActivationGeneration::new(2).unwrap();
    assert!(matches!(
        runtime.apply_generated(wrong_generation),
        Err(Phase8RuntimeError::StaleGeneration)
    ));
    assert!(matches!(
        runtime
            .apply_generated(request.clone().complete(request.source.clone()))
            .unwrap(),
        GenerationOutcome::Published { revision: 1 }
    ));
}

#[test]
fn full_publication_unpacks_ticks_before_block_ticking_and_backpressure_is_atomic() {
    let position = ChunkPos::new(0, 0);
    let mut runtime = runtime_with_capacity(64);
    runtime.demand_chunk(position).unwrap();
    advance_to_full(&mut runtime, position);
    runtime.take_events(usize::MAX);
    runtime
        .promote(position, ChunkActivity::Accessible)
        .unwrap();
    runtime
        .promote(position, ChunkActivity::BlockTicking)
        .unwrap();
    runtime
        .promote(position, ChunkActivity::EntityTicking)
        .unwrap();
    let events = runtime.take_events(usize::MAX);
    assert_eq!(
        events.iter().map(|event| event.kind).collect::<Vec<_>>(),
        [
            ChunkEventKind::Accessible,
            ChunkEventKind::PersistedTicksUnpacked,
            ChunkEventKind::BlockTicking,
            ChunkEventKind::EntityTicking,
        ]
    );

    let mut constrained = runtime_with_capacity(2);
    constrained.demand_chunk(position).unwrap();
    for target in ChunkStatus::ALL.into_iter().skip(1) {
        let request = constrained.begin_generation(position, target).unwrap();
        let generated = request.source.clone();
        constrained
            .apply_generated(request.complete(generated))
            .unwrap();
        constrained.take_events(usize::MAX);
    }
    constrained
        .promote(position, ChunkActivity::Accessible)
        .unwrap();
    assert!(matches!(
        constrained.promote(position, ChunkActivity::BlockTicking),
        Err(Phase8RuntimeError::EventCapacity)
    ));
    assert_eq!(
        constrained.lifecycle(position).unwrap().activity,
        ChunkActivity::Accessible
    );
}

#[test]
fn demand_cancels_identity_matched_unload_and_committed_save_tears_down_after_receipt() {
    let position = ChunkPos::new(0, 0);
    let mut runtime = runtime_with_capacity(64);
    runtime.demand_chunk(position).unwrap();
    let first_token = runtime.schedule_unload(position).unwrap();
    let first = runtime
        .prepare_save(10, PersistenceRevision::INITIAL)
        .unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(temporary.path()).unwrap();
    let first_receipt = store.commit(first.recovery_point()).unwrap();
    assert_eq!(
        runtime.demand_chunk(position).unwrap(),
        TicketOutcome::CancelledUnload { token: first_token }
    );
    assert_eq!(runtime.apply_save_receipt(first, first_receipt).unwrap(), 0);
    assert!(runtime.chunk(position).is_some());

    let second_token = runtime.schedule_unload(position).unwrap();
    let second = runtime
        .prepare_save(11, PersistenceRevision::new(2).unwrap())
        .unwrap();
    let second_receipt = store.commit(second.recovery_point()).unwrap();
    assert_eq!(
        runtime.apply_save_receipt(second, second_receipt).unwrap(),
        1
    );
    assert!(runtime.chunk(position).is_none());
    let events = runtime.take_events(usize::MAX);
    assert!(
        events
            .iter()
            .any(|event| { event.kind == ChunkEventKind::UnloadCancelled { token: first_token } })
    );
    assert!(events.iter().any(|event| {
        event.kind
            == ChunkEventKind::Saved {
                token: second_token,
            }
    }));
    assert!(events.iter().any(|event| {
        event.kind
            == ChunkEventKind::Unloaded {
                token: second_token,
            }
    }));
}

#[test]
fn recovery_handoff_preserves_chunks_auxiliary_records_and_inspector_truth() {
    let position = ChunkPos::new(1, 1);
    let mut runtime = runtime_with_capacity(64);
    runtime.demand_chunk(position).unwrap();
    let expected = runtime.chunk(position).unwrap().revision();
    runtime
        .set_block(
            &key(),
            ActivationGeneration::INITIAL,
            expected,
            BlockPos::new(16, 4, 16),
            BlockStateId::new(44),
        )
        .unwrap();
    runtime
        .replace_auxiliary_records(vec![
            SnapshotRecord::new(
                SnapshotRecordKind::Extension,
                ResourceId::new("ferrite", "phase5/runtime_v1").unwrap(),
                vec![1],
                vec![2],
            )
            .unwrap(),
        ])
        .unwrap();
    let prepared = runtime
        .prepare_save(20, PersistenceRevision::INITIAL)
        .unwrap();
    let inspection = inspect_recovery_point(prepared.recovery_point()).unwrap();
    assert!(inspection.snapshot_state_hash_matches);
    assert_eq!(inspection.auxiliary_records, 1);
    assert_eq!(inspection.chunks.len(), 1);
    assert_eq!(inspection.chunks[0].revision, 1);

    let directory = tempfile::tempdir().unwrap();
    let mut store = RegionFileStore::open(directory.path()).unwrap();
    store.commit(prepared.recovery_point()).unwrap();
    let loaded = store.load(&key()).unwrap().unwrap();
    let target_generation = ActivationGeneration::new(2).unwrap();
    let handoff = RegionHandoffState::prepare(loaded, target_generation).unwrap();
    let digest = *handoff.digest();
    let recovered = handoff.install(&key(), digest).unwrap();
    let restored = Phase8RegionRuntime::restore_recovered(recovered, config(64)).unwrap();
    assert_eq!(restored.generation(), target_generation);
    assert_eq!(
        restored
            .chunk(position)
            .unwrap()
            .block_state(BlockPos::new(16, 4, 16))
            .unwrap(),
        BlockStateId::new(44)
    );
    assert_eq!(
        restored
            .prepare_save(21, PersistenceRevision::new(2).unwrap())
            .unwrap()
            .records()
            .len(),
        2
    );

    let mut wrong_content = config(64);
    wrong_content.content_manifest = [8; 32];
    assert!(matches!(
        Phase8RegionRuntime::restore(
            key(),
            ActivationGeneration::new(3).unwrap(),
            prepared.recovery_point(),
            wrong_content,
        ),
        Err(Phase8RuntimeError::ContentManifestMismatch)
    ));
}

#[test]
fn ownership_capacity_status_and_receipt_failures_do_not_mutate_world_state() {
    let mut limited_config = config(1);
    limited_config.chunk_capacity = 1;
    let mut runtime =
        Phase8RegionRuntime::new(key(), ActivationGeneration::INITIAL, limited_config).unwrap();
    runtime.demand_chunk(ChunkPos::new(0, 0)).unwrap();
    assert!(matches!(
        runtime.demand_chunk(ChunkPos::new(1, 0)),
        Err(Phase8RuntimeError::ChunkCapacity)
    ));
    assert!(matches!(
        runtime.demand_chunk(ChunkPos::new(8, 0)),
        Err(Phase8RuntimeError::WrongChunkOwner(_))
    ));
    assert!(matches!(
        runtime.begin_generation(ChunkPos::new(0, 0), ChunkStatus::Noise),
        Err(Phase8RuntimeError::NonSequentialStatus { .. })
    ));
    assert_eq!(runtime.chunks().count(), 1);
}

#[test]
fn world_bootstrap_is_overworld_first_and_level_globals_are_control_region_owned() {
    let overworld = DimensionId::new(ResourceId::minecraft("overworld").unwrap());
    let nether = DimensionId::new(ResourceId::minecraft("the_nether").unwrap());
    let end = DimensionId::new(ResourceId::minecraft("the_end").unwrap());
    let mut lifecycle = WorldLifecycleRuntime::bootstrap(
        WorldId::new(1).unwrap(),
        RegionMappingVersion::V1,
        overworld.clone(),
        [nether.clone(), end.clone()],
        ActivationGeneration::INITIAL,
        [7; 32],
        64,
    )
    .unwrap();
    assert_eq!(
        lifecycle.dimensions(),
        [overworld.clone(), nether, end.clone()]
    );
    lifecycle.set_pending_work(&end, 2).unwrap();
    assert_eq!(
        lifecycle.prepare_levels().unwrap(),
        PrepareOutcome::Waiting { pending_work: 2 }
    );
    assert_eq!(lifecycle.state(), WorldLifecycleState::Bootstrapping);
    lifecycle.set_pending_work(&end, 0).unwrap();
    assert_eq!(lifecycle.prepare_levels().unwrap(), PrepareOutcome::Ready);
    assert_eq!(lifecycle.state(), WorldLifecycleState::Running);

    let control = lifecycle.level(&overworld).unwrap().control_region.clone();
    lifecycle
        .border_mut(&control, ActivationGeneration::INITIAL)
        .unwrap()
        .set_size(100.0);
    lifecycle
        .set_no_save(&control, ActivationGeneration::INITIAL, true)
        .unwrap();
    let control_record = lifecycle
        .level_record(&control, ActivationGeneration::INITIAL)
        .unwrap();
    let mut control_runtime = runtime_with_capacity(8);
    control_runtime
        .replace_auxiliary_records(vec![control_record])
        .unwrap();
    let control_save = control_runtime
        .prepare_save(1, PersistenceRevision::INITIAL)
        .unwrap();
    assert_eq!(
        inspect_recovery_point(control_save.recovery_point())
            .unwrap()
            .auxiliary_records,
        1
    );
    let records = lifecycle.level_records().unwrap();
    let mut restored = WorldLifecycleRuntime::bootstrap(
        WorldId::new(1).unwrap(),
        RegionMappingVersion::V1,
        overworld.clone(),
        [
            DimensionId::new(ResourceId::minecraft("the_nether").unwrap()),
            end,
        ],
        ActivationGeneration::new(2).unwrap(),
        [7; 32],
        64,
    )
    .unwrap();
    restored.apply_level_records(&records).unwrap();
    assert_eq!(restored.level(&overworld).unwrap().border.get_size(), 100.0);
    assert!(restored.level(&overworld).unwrap().no_save);
    assert!(
        restored
            .border_mut(&control, ActivationGeneration::INITIAL)
            .is_err()
    );
}

#[test]
fn shutdown_drains_before_flush_and_continues_after_independent_level_close_failure() {
    let overworld = DimensionId::new(ResourceId::minecraft("overworld").unwrap());
    let nether = DimensionId::new(ResourceId::minecraft("the_nether").unwrap());
    let mut lifecycle = WorldLifecycleRuntime::bootstrap(
        WorldId::new(1).unwrap(),
        RegionMappingVersion::V1,
        overworld.clone(),
        [nether.clone()],
        ActivationGeneration::INITIAL,
        [7; 32],
        64,
    )
    .unwrap();
    lifecycle.prepare_levels().unwrap();
    lifecycle.take_events(usize::MAX);
    lifecycle.set_pending_work(&nether, 1).unwrap();
    lifecycle.begin_shutdown(3).unwrap();
    assert!(
        lifecycle
            .finish_shutdown(&BTreeMap::from([
                (overworld.clone(), true),
                (nether.clone(), false),
            ]))
            .is_err()
    );
    assert_eq!(lifecycle.state(), WorldLifecycleState::Closing);
    lifecycle.set_pending_work(&nether, 0).unwrap();
    lifecycle
        .finish_shutdown(&BTreeMap::from([
            (overworld.clone(), true),
            (nether.clone(), false),
        ]))
        .unwrap();
    assert_eq!(lifecycle.state(), WorldLifecycleState::Closed);
    assert_eq!(
        lifecycle.level(&overworld).unwrap().lifecycle,
        LevelLifecycleState::Closed
    );
    let events = lifecycle.take_events(usize::MAX);
    let work = events
        .iter()
        .position(|event| *event == WorldLifecycleEvent::WorkDrained)
        .unwrap();
    let flush = events
        .iter()
        .position(|event| *event == WorldLifecycleEvent::LevelsFlushed)
        .unwrap();
    let failed_close = events
        .iter()
        .position(|event| {
            *event
                == WorldLifecycleEvent::LevelClosed {
                    dimension: nether.clone(),
                    succeeded: false,
                }
        })
        .unwrap();
    let resources = events
        .iter()
        .position(|event| *event == WorldLifecycleEvent::ResourcesClosed)
        .unwrap();
    assert!(work < flush && flush < failed_close && failed_close < resources);
}
