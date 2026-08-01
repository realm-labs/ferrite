use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{SnapshotRecord, SnapshotRecordKind};
use ferrite_server_runtime::entity_service::continuity::{
    decode_entity, encode_entity, entity_domain, receipt_domain,
};
use ferrite_server_runtime::entity_service::model::{
    EntityCommandHeader, EntityLifecycleState, EntityMutation, EntityPayload,
    EntityPersistentState, EntityProjectionKind, EntityTransferRequest, LifecycleOutcome,
    MAX_ENTITY_PAYLOAD_BYTES, ObserverOutcome, RemovalReason,
};
use ferrite_server_runtime::entity_service::runtime::{
    EntityServiceRegionRuntime, EntityServiceRuntimeError, EntityServiceRuntimeLimits,
};
use ferrite_server_runtime::entity_service::transfer::TransferAcceptance;
use ferrite_simulation::tick::GameTick;

fn region(x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(x, 0),
        RegionMappingVersion::V1,
    )
}

fn id(value: u128) -> StableEntityId {
    StableEntityId::new(value).unwrap()
}

fn limits(projections: usize) -> EntityServiceRuntimeLimits {
    EntityServiceRuntimeLimits::new(8, 4, projections, 8)
}

fn runtime(region_x: i32, projections: usize) -> EntityServiceRegionRuntime {
    EntityServiceRegionRuntime::new(
        region(region_x),
        ActivationGeneration::INITIAL,
        RegionMapping::V1,
        limits(projections),
    )
    .unwrap()
}

fn payload(marker: u8) -> EntityPayload {
    EntityPayload::new(vec![marker; usize::from(marker) + 1]).unwrap()
}

fn state(chunk_x: i32, marker: u8) -> EntityPersistentState {
    EntityPersistentState::active(
        ResourceId::minecraft("zombie").unwrap(),
        ChunkPos::new(chunk_x, 0),
        payload(marker),
    )
}

fn header(
    runtime: &EntityServiceRegionRuntime,
    entity: StableEntityId,
    revision: u64,
    sequence: u64,
) -> EntityCommandHeader {
    EntityCommandHeader {
        region: runtime.key().clone(),
        generation: runtime.generation(),
        entity,
        expected_revision: revision,
        sequence,
    }
}

fn transfer_request(
    source: &EntityServiceRegionRuntime,
    target: &EntityServiceRegionRuntime,
    entity: StableEntityId,
    revision: u64,
    sequence: u64,
    marker: u8,
) -> EntityTransferRequest {
    EntityTransferRequest {
        tick: GameTick::new(10),
        source: source.key().clone(),
        source_generation: source.generation(),
        target: target.key().clone(),
        target_generation: target.generation(),
        entity,
        expected_revision: revision,
        sequence,
        candidate: EntityMutation {
            chunk: ChunkPos::new(8, 0),
            payload: payload(marker),
        },
    }
}

#[test]
fn generation_region_revision_and_sequence_fences_precede_entity_mutation() {
    let entity = id(1);
    let observer = id(100);
    let mut runtime = runtime(0, 8);
    runtime.insert(entity, state(0, 1)).unwrap();
    runtime.add_observer(observer).unwrap();
    runtime.drain_projections(observer, usize::MAX).unwrap();
    let before = runtime.state(entity).unwrap().clone();

    let mut wrong_region = header(&runtime, entity, 0, 1);
    wrong_region.region = region(1);
    assert!(matches!(
        runtime.apply_mutation(
            &wrong_region,
            EntityMutation {
                chunk: ChunkPos::new(1, 0),
                payload: payload(2),
            }
        ),
        Err(EntityServiceRuntimeError::WrongRegion)
    ));
    let mut stale_generation = header(&runtime, entity, 0, 1);
    stale_generation.generation = ActivationGeneration::new(2).unwrap();
    assert!(matches!(
        runtime.deactivate(&stale_generation),
        Err(EntityServiceRuntimeError::StaleGeneration { .. })
    ));
    assert!(matches!(
        runtime.activate(&header(&runtime, entity, 0, 2)),
        Err(EntityServiceRuntimeError::CommandSequenceGap {
            expected: 1,
            actual: 2
        })
    ));
    assert!(matches!(
        runtime.deactivate(&header(&runtime, entity, 9, 1)),
        Err(EntityServiceRuntimeError::RevisionMismatch {
            expected: 0,
            actual: 9
        })
    ));
    assert_eq!(runtime.state(entity), Some(&before));
    assert_eq!(runtime.projection_len(observer), Some(0));

    assert_eq!(
        runtime
            .apply_mutation(
                &header(&runtime, entity, 0, 1),
                EntityMutation {
                    chunk: ChunkPos::new(7, 0),
                    payload: payload(3),
                },
            )
            .unwrap(),
        LifecycleOutcome::Committed { revision: 1 }
    );
    assert_eq!(
        runtime
            .apply_mutation(
                &header(&runtime, entity, 0, 1),
                EntityMutation {
                    chunk: ChunkPos::new(1, 0),
                    payload: payload(9),
                },
            )
            .unwrap(),
        LifecycleOutcome::AlreadyApplied
    );
    let committed = runtime.state(entity).unwrap();
    assert_eq!(committed.chunk, ChunkPos::new(7, 0));
    assert_eq!(committed.payload, payload(3));
}

#[test]
fn activation_deactivation_and_despawn_publish_exact_lifecycle_edges() {
    let entity = id(1);
    let observer = id(100);
    let mut runtime = runtime(0, 8);
    let mut inactive = state(0, 1);
    inactive.lifecycle = EntityLifecycleState::Inactive;
    runtime.insert(entity, inactive).unwrap();
    runtime.add_observer(observer).unwrap();
    assert_eq!(runtime.projection_len(observer), Some(0));

    runtime.activate(&header(&runtime, entity, 0, 1)).unwrap();
    let spawn = runtime.drain_projections(observer, usize::MAX).unwrap();
    assert!(matches!(
        spawn.as_slice(),
        [projection]
            if projection.entity == entity
                && matches!(
                    projection.kind,
                    EntityProjectionKind::Spawn { revision: 1, .. }
                )
    ));

    runtime.deactivate(&header(&runtime, entity, 1, 2)).unwrap();
    let remove = runtime.drain_projections(observer, usize::MAX).unwrap();
    assert!(matches!(
        remove.as_slice(),
        [projection]
            if matches!(
                projection.kind,
                EntityProjectionKind::Remove {
                    revision: 2,
                    reason: RemovalReason::Deactivated,
                }
            )
    ));
    runtime.despawn(&header(&runtime, entity, 2, 3)).unwrap();
    assert!(runtime.state(entity).is_none());
    assert_eq!(
        runtime.projection_len(observer),
        Some(0),
        "inactive despawn does not duplicate a remove"
    );
}

#[test]
fn bounded_fanout_is_atomic_across_all_observers() {
    let entity = id(1);
    let first = id(100);
    let second = id(101);
    let mut runtime = runtime(0, 1);
    runtime.add_observer(first).unwrap();
    runtime.add_observer(second).unwrap();
    runtime.insert(entity, state(0, 1)).unwrap();
    let before = runtime.state(entity).unwrap().clone();

    assert!(matches!(
        runtime.apply_mutation(
            &header(&runtime, entity, 0, 1),
            EntityMutation {
                chunk: ChunkPos::new(1, 0),
                payload: payload(2),
            },
        ),
        Err(EntityServiceRuntimeError::ProjectionCapacity {
            observer,
            capacity: 1
        }) if observer == first
    ));
    assert_eq!(runtime.state(entity), Some(&before));
    runtime.drain_projections(first, usize::MAX).unwrap();
    assert!(matches!(
        runtime.deactivate(&header(&runtime, entity, 0, 1)),
        Err(EntityServiceRuntimeError::ProjectionCapacity { observer, .. }) if observer == second
    ));
    runtime.drain_projections(second, usize::MAX).unwrap();
    runtime
        .apply_mutation(
            &header(&runtime, entity, 0, 1),
            EntityMutation {
                chunk: ChunkPos::new(1, 0),
                payload: payload(2),
            },
        )
        .unwrap();
    assert_eq!(runtime.projection_len(first), Some(1));
    assert_eq!(runtime.projection_len(second), Some(1));
}

#[test]
fn observer_join_is_bounded_and_uses_stable_entity_order() {
    let mut runtime = runtime(0, 2);
    runtime.insert(id(2), state(0, 2)).unwrap();
    runtime.insert(id(1), state(0, 1)).unwrap();
    assert_eq!(
        runtime.add_observer(id(100)).unwrap(),
        ObserverOutcome::Added
    );
    assert_eq!(
        runtime.add_observer(id(100)).unwrap(),
        ObserverOutcome::AlreadyPresent
    );
    assert_eq!(
        runtime
            .drain_projections(id(100), usize::MAX)
            .unwrap()
            .iter()
            .map(|projection| projection.entity)
            .collect::<Vec<_>>(),
        vec![id(1), id(2)]
    );

    runtime.insert(id(3), state(0, 3)).unwrap();
    runtime.drain_projections(id(100), usize::MAX).unwrap();
    assert!(matches!(
        runtime.add_observer(id(101)),
        Err(EntityServiceRuntimeError::ProjectionCapacity {
            observer,
            capacity: 2
        }) if observer == id(101)
    ));
    assert_eq!(runtime.observer_count(), 1);
}

#[test]
fn two_phase_transfer_is_generation_fenced_idempotent_and_ordered() {
    let entity = id(1);
    let source_observer = id(100);
    let target_observer = id(101);
    let mut source = runtime(0, 8);
    let mut target = runtime(1, 8);
    source.add_observer(source_observer).unwrap();
    target.add_observer(target_observer).unwrap();
    source.insert(entity, state(7, 1)).unwrap();
    source
        .drain_projections(source_observer, usize::MAX)
        .unwrap();

    let request = transfer_request(&source, &target, entity, 0, 1, 7);
    let transfer = source.prepare_transfer(request.clone()).unwrap();
    assert_eq!(source.retry_transfer(entity).unwrap(), transfer);
    assert!(matches!(
        source.state(entity).unwrap().lifecycle,
        EntityLifecycleState::OutboundPending(_)
    ));
    assert!(matches!(
        source
            .drain_projections(source_observer, usize::MAX)
            .unwrap()
            .as_slice(),
        [projection]
            if matches!(
                projection.kind,
                EntityProjectionKind::Remove {
                    reason: RemovalReason::Transferred,
                    ..
                }
            )
    ));

    let accepted = target.accept_transfer(&transfer).unwrap();
    let TransferAcceptance::Accepted(receipt) = accepted else {
        panic!("first transfer must commit");
    };
    assert!(matches!(
        target.accept_transfer(&transfer).unwrap(),
        TransferAcceptance::AlreadyApplied(_)
    ));
    let target_state = target.state(entity).unwrap();
    assert_eq!(target_state.chunk, ChunkPos::new(8, 0));
    assert_eq!(target_state.payload, payload(7));
    assert_eq!(target.applied_transfer_count(), 1);
    assert!(matches!(
        target
            .drain_projections(target_observer, usize::MAX)
            .unwrap()
            .as_slice(),
        [projection]
            if matches!(
                projection.kind,
                EntityProjectionKind::Spawn { revision: 1, .. }
            )
    ));

    source.commit_transfer(&receipt).unwrap();
    assert!(source.state(entity).is_none());
    assert_eq!(target.entity_count(), 1);
}

#[test]
fn transfer_failure_preserves_retry_and_abort_restores_source_tracking() {
    let entity = id(1);
    let observer = id(100);
    let mut source = runtime(0, 8);
    let mut target = EntityServiceRegionRuntime::new(
        region(1),
        ActivationGeneration::new(2).unwrap(),
        RegionMapping::V1,
        limits(8),
    )
    .unwrap();
    source.add_observer(observer).unwrap();
    source.insert(entity, state(7, 1)).unwrap();
    source.drain_projections(observer, usize::MAX).unwrap();

    let mut request = transfer_request(&source, &target, entity, 0, 1, 4);
    request.target_generation = ActivationGeneration::INITIAL;
    let transfer = source.prepare_transfer(request.clone()).unwrap();
    assert!(matches!(
        target.accept_transfer(&transfer),
        Err(EntityServiceRuntimeError::StaleGeneration { .. })
    ));
    assert_eq!(target.entity_count(), 0);
    assert_eq!(source.retry_transfer(entity).unwrap(), transfer);

    let mut mismatch = request;
    mismatch.candidate.payload = payload(9);
    assert!(matches!(
        source.prepare_transfer(mismatch),
        Err(EntityServiceRuntimeError::TransferReplayMismatch)
    ));
    source.drain_projections(observer, usize::MAX).unwrap();
    source.abort_transfer(entity, 1).unwrap();
    assert_eq!(
        source.state(entity).unwrap().lifecycle,
        EntityLifecycleState::Active
    );
    assert!(matches!(
        source
            .drain_projections(observer, usize::MAX)
            .unwrap()
            .as_slice(),
        [projection]
            if matches!(projection.kind, EntityProjectionKind::Spawn { .. })
    ));
}

#[test]
fn save_restore_preserves_active_inactive_pending_and_receipt_continuity() {
    assert_eq!(
        entity_domain().to_string(),
        "ferrite:entity-service/entity_v1"
    );
    assert_eq!(
        receipt_domain().to_string(),
        "ferrite:entity-service/applied_transfer_v1"
    );
    let active = id(1);
    let inactive = id(2);
    let pending = id(3);
    let mut source = runtime(0, 8);
    let mut target = runtime(1, 8);
    source.insert(active, state(0, 1)).unwrap();
    let mut inactive_state = state(1, 2);
    inactive_state.lifecycle = EntityLifecycleState::Inactive;
    source.insert(inactive, inactive_state).unwrap();
    source.insert(pending, state(7, 3)).unwrap();
    let transfer = source
        .prepare_transfer(transfer_request(&source, &target, pending, 0, 1, 8))
        .unwrap();
    target.accept_transfer(&transfer).unwrap();

    let source_records = source.snapshot_records().unwrap();
    assert!(
        source_records
            .iter()
            .all(|record| record.domain() == &entity_domain())
    );
    let restored_source = EntityServiceRegionRuntime::restore(
        region(0),
        ActivationGeneration::new(2).unwrap(),
        RegionMapping::V1,
        limits(8),
        &source_records,
    )
    .unwrap();
    assert_eq!(restored_source.entity_count(), 3);
    assert_eq!(
        restored_source.state(inactive).unwrap().lifecycle,
        EntityLifecycleState::Inactive
    );
    let retried = restored_source.retry_transfer(pending).unwrap();
    assert_eq!(
        retried.source_generation(),
        ActivationGeneration::new(2).unwrap()
    );
    assert_eq!(retried.state(), transfer.state());
    assert_eq!(restored_source.observer_count(), 0);

    let target_records = target.snapshot_records().unwrap();
    assert!(
        target_records
            .iter()
            .any(|record| record.domain() == &entity_domain())
    );
    assert!(
        target_records
            .iter()
            .any(|record| record.domain() == &receipt_domain())
    );
    let mut restored_target = EntityServiceRegionRuntime::restore(
        region(1),
        ActivationGeneration::INITIAL,
        RegionMapping::V1,
        limits(8),
        &target_records,
    )
    .unwrap();
    assert_eq!(restored_target.applied_transfer_count(), 1);
    assert!(matches!(
        restored_target.accept_transfer(&transfer).unwrap(),
        TransferAcceptance::AlreadyApplied(_)
    ));
    restored_target.prune_applied_transfers(GameTick::new(10));
    assert_eq!(restored_target.applied_transfer_count(), 0);
}

#[test]
fn continuity_is_stably_ordered_and_rejects_corruption_or_wrong_ownership() {
    let mut runtime = runtime(0, 8);
    runtime.insert(id(2), state(0, 2)).unwrap();
    runtime.insert(id(1), state(0, 1)).unwrap();
    let records = runtime.snapshot_records().unwrap();
    let entities = records
        .iter()
        .filter_map(|record| decode_entity(record).unwrap())
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(entities, vec![id(1), id(2)]);

    let mut corrupt = records[0].value().to_vec();
    corrupt.push(0);
    let corrupt = SnapshotRecord::new(
        SnapshotRecordKind::Entity,
        entity_domain(),
        id(9).to_be_bytes().to_vec(),
        corrupt,
    )
    .unwrap();
    assert!(
        EntityServiceRegionRuntime::restore(
            region(0),
            ActivationGeneration::INITIAL,
            RegionMapping::V1,
            limits(8),
            &[corrupt],
        )
        .is_err()
    );

    let wrong = encode_entity(id(9), &state(8, 1)).unwrap();
    assert!(matches!(
        EntityServiceRegionRuntime::restore(
            region(0),
            ActivationGeneration::INITIAL,
            RegionMapping::V1,
            limits(8),
            &[wrong],
        ),
        Err(EntityServiceRuntimeError::WrongChunkOwner { .. })
    ));
    assert!(EntityPayload::new(vec![0; MAX_ENTITY_PAYLOAD_BYTES + 1]).is_err());
}
