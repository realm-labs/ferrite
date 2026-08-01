use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::recovery::RegionHandoffState;
use ferrite_persistence::snapshot::{
    PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_server_runtime::chunk::projection::JavaTerrainRegistryMap;
use ferrite_server_runtime::simulation::boundary::{
    BoundaryMechanic, BoundaryMutation, BoundarySchedule, BoundaryTransactionHeader,
    BoundaryTransactionLimits, MechanicBoundaryTransaction,
};
use ferrite_server_runtime::simulation::budget::{
    QueueBudgetError, SimulationQueueBudget, SimulationQueueKind,
};
use ferrite_server_runtime::simulation::continuity::{ScheduledQueueKind, SimulationContinuity};
use ferrite_server_runtime::simulation::runtime::{
    BoundaryApplyOutcome, SimulationRegionRuntime, SimulationRuntimeConfig, SimulationRuntimeError,
};
use ferrite_simulation::scheduled_tick::level::ScheduleOutcome;
use ferrite_simulation::scheduled_tick::record::TickPriority;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;
use std::collections::BTreeSet;

const TARGET_CHUNK: ChunkPos = ChunkPos::new(8, 0);
const FIRST: BlockPos = BlockPos::new(128, 64, 0);
const SECOND: BlockPos = BlockPos::new(129, 64, 0);

fn dimension() -> DimensionId {
    DimensionId::new(ResourceId::minecraft("overworld").unwrap())
}

fn region(coordinate: RegionCoord) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        dimension(),
        coordinate,
        RegionMappingVersion::V1,
    )
}

fn config(scheduled: usize, effects: usize, projection: usize) -> SimulationRuntimeConfig {
    SimulationRuntimeConfig {
        mapping: RegionMapping::V1,
        budget: SimulationQueueBudget::new([
            (SimulationQueueKind::ScheduledBlocks, scheduled),
            (SimulationQueueKind::ScheduledFluids, scheduled),
            (SimulationQueueKind::BoundaryTransactions, 8),
            (SimulationQueueKind::ImmediateNeighbors, effects),
            (SimulationQueueKind::Fluids, effects),
            (SimulationQueueKind::Redstone, effects),
            (SimulationQueueKind::Lighting, effects),
            (SimulationQueueKind::ProjectionPositions, projection),
        ])
        .unwrap(),
        projection_capacity: projection.max(8),
        receipt_capacity: 64,
        gameplay_random_seed: 99,
    }
}

fn voxels() -> RegionVoxelState {
    let key = region(RegionCoord::new(1, 0));
    let layout = ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).unwrap(),
        BlockStateId::new(0),
        BiomeId::new(0),
    );
    let mut state = RegionVoxelState::new(key, RegionMapping::V1, layout).unwrap();
    state.ensure_chunk(TARGET_CHUNK).unwrap();
    state
}

fn transaction(
    target_generation: ActivationGeneration,
    source_sequence: u64,
    expected: [BlockStateId; 2],
) -> MechanicBoundaryTransaction {
    MechanicBoundaryTransaction::new(
        BoundaryTransactionHeader {
            tick: GameTick::new(7),
            source: region(RegionCoord::new(0, 0)),
            source_generation: ActivationGeneration::INITIAL,
            target: region(RegionCoord::new(1, 0)),
            target_generation,
            source_sequence,
        },
        BoundaryMechanic::Redstone,
        vec![
            BoundaryMutation {
                order: 2,
                position: SECOND,
                expected: expected[1],
                replacement: BlockStateId::new(2),
            },
            BoundaryMutation {
                order: 1,
                position: FIRST,
                expected: expected[0],
                replacement: BlockStateId::new(1),
            },
        ],
        vec![
            BoundarySchedule {
                order: 2,
                kind: ScheduledQueueKind::Fluid,
                type_identity: ResourceId::minecraft("water").unwrap(),
                position: SECOND,
                delay: 3,
                priority: TickPriority::Normal,
            },
            BoundarySchedule {
                order: 1,
                kind: ScheduledQueueKind::Block,
                type_identity: ResourceId::minecraft("redstone_wire").unwrap(),
                position: FIRST,
                delay: 2,
                priority: TickPriority::High,
            },
        ],
        RegionMapping::V1,
        BoundaryTransactionLimits::new(16, 16),
    )
    .unwrap()
}

fn runtime(config: SimulationRuntimeConfig) -> SimulationRegionRuntime {
    SimulationRegionRuntime::new(
        region(RegionCoord::new(1, 0)),
        ActivationGeneration::INITIAL,
        GameTick::new(7),
        100,
        [TARGET_CHUNK],
        config,
    )
    .unwrap()
}

fn registry_map(include_replacements: bool) -> JavaTerrainRegistryMap {
    let mut map = JavaTerrainRegistryMap::new(8, BlockStateId::new(0)).unwrap();
    map.insert_block_state(BlockStateId::new(0), 0).unwrap();
    if include_replacements {
        map.insert_block_state(BlockStateId::new(1), 1).unwrap();
        map.insert_block_state(BlockStateId::new(2), 2).unwrap();
    }
    map
}

#[test]
fn queue_reservations_and_releases_fail_atomically() {
    let mut budget = SimulationQueueBudget::new([
        (SimulationQueueKind::ScheduledBlocks, 2),
        (SimulationQueueKind::ScheduledFluids, 1),
    ])
    .unwrap();
    budget
        .try_reserve([
            (SimulationQueueKind::ScheduledBlocks, 1),
            (SimulationQueueKind::ScheduledFluids, 1),
        ])
        .unwrap();

    assert!(matches!(
        budget.try_reserve([
            (SimulationQueueKind::ScheduledBlocks, 1),
            (SimulationQueueKind::ScheduledFluids, 1),
        ]),
        Err(QueueBudgetError::Full {
            kind: SimulationQueueKind::ScheduledFluids,
            ..
        })
    ));
    assert_eq!(
        budget
            .pressure(SimulationQueueKind::ScheduledBlocks)
            .unwrap()
            .used,
        1
    );
    assert!(matches!(
        budget.release_usage([
            (SimulationQueueKind::ScheduledBlocks, 1),
            (SimulationQueueKind::ScheduledFluids, 2),
        ]),
        Err(QueueBudgetError::ReleaseExceedsUsage {
            kind: SimulationQueueKind::ScheduledFluids,
            ..
        })
    ));
    assert_eq!(
        budget
            .pressure(SimulationQueueKind::ScheduledBlocks)
            .unwrap()
            .used,
        1
    );
}

#[test]
fn boundary_transaction_commits_as_one_unit_and_is_idempotent() {
    let mut runtime = runtime(config(8, 8, 8));
    let mut voxels = voxels();
    let transaction = transaction(
        ActivationGeneration::INITIAL,
        12,
        [BlockStateId::new(0), BlockStateId::new(0)],
    );

    assert_eq!(
        runtime
            .apply_transaction(&mut voxels, &transaction)
            .unwrap(),
        BoundaryApplyOutcome::Applied {
            mutations: 2,
            scheduled_blocks: 1,
            scheduled_fluids: 1,
            deferred_effects: 2,
            projected_positions: 2,
        }
    );
    assert_eq!(
        voxels.view().block_state(FIRST).unwrap(),
        BlockStateId::new(1)
    );
    assert_eq!(
        voxels.view().block_state(SECOND).unwrap(),
        BlockStateId::new(2)
    );
    assert_eq!(
        runtime
            .apply_transaction(&mut voxels, &transaction)
            .unwrap(),
        BoundaryApplyOutcome::AlreadyApplied
    );
    assert_eq!(
        runtime
            .queue_pressure(SimulationQueueKind::BoundaryTransactions)
            .unwrap()
            .used,
        0
    );

    let effects = runtime
        .drain_effects(SimulationQueueKind::Redstone, 8)
        .unwrap();
    assert_eq!(
        effects
            .iter()
            .map(|effect| effect.position)
            .collect::<Vec<_>>(),
        [FIRST, SECOND]
    );
}

#[test]
fn capacity_and_expected_state_failures_leave_no_partial_state() {
    let mut constrained = runtime(config(8, 1, 1));
    let mut state = voxels();
    let initial_transaction = transaction(
        ActivationGeneration::INITIAL,
        13,
        [BlockStateId::new(0), BlockStateId::new(0)],
    );
    assert!(matches!(
        constrained.apply_transaction(&mut state, &initial_transaction),
        Err(SimulationRuntimeError::Budget(
            QueueBudgetError::Full { .. }
        ))
    ));
    assert_eq!(
        state.view().block_state(FIRST).unwrap(),
        BlockStateId::new(0)
    );
    assert_eq!(
        constrained
            .queue_pressure(SimulationQueueKind::ScheduledBlocks)
            .unwrap()
            .used,
        0
    );

    let mut mismatch = runtime(config(8, 8, 8));
    let wrong = transaction(
        ActivationGeneration::INITIAL,
        14,
        [BlockStateId::new(9), BlockStateId::new(0)],
    );
    assert!(matches!(
        mismatch.apply_transaction(&mut state, &wrong),
        Err(SimulationRuntimeError::UnexpectedBlockState {
            position: FIRST,
            ..
        })
    ));
    assert_eq!(
        state.view().block_state(SECOND).unwrap(),
        BlockStateId::new(0)
    );
}

#[test]
fn failed_projection_retains_committed_updates_for_retry() {
    let mut runtime = runtime(config(8, 8, 8));
    let mut state = voxels();
    runtime
        .apply_transaction(
            &mut state,
            &transaction(
                ActivationGeneration::INITIAL,
                15,
                [BlockStateId::new(0), BlockStateId::new(0)],
            ),
        )
        .unwrap();

    assert!(runtime.project_and_clear(&registry_map(false)).is_err());
    assert_eq!(
        runtime
            .queue_pressure(SimulationQueueKind::ProjectionPositions)
            .unwrap()
            .used,
        2
    );
    let packets = runtime.project_and_clear(&registry_map(true)).unwrap();
    assert!(matches!(
        packets.as_slice(),
        [PlayClientboundPacket::SectionBlocksUpdate(_)]
    ));
    assert_eq!(
        runtime
            .queue_pressure(SimulationQueueKind::ProjectionPositions)
            .unwrap()
            .used,
        0
    );
}

#[test]
fn continuity_survives_snapshot_handoff_and_fences_replay() {
    let mut source = runtime(config(16, 16, 16));
    let mut state = voxels();
    let initial_transaction = transaction(
        ActivationGeneration::INITIAL,
        16,
        [BlockStateId::new(0), BlockStateId::new(0)],
    );
    source
        .apply_transaction(&mut state, &initial_transaction)
        .unwrap();
    assert_eq!(
        source
            .schedule_local(
                ScheduledQueueKind::Block,
                ResourceId::minecraft("stone").unwrap(),
                BlockPos::new(130, 64, 0),
                5,
                TickPriority::Low,
            )
            .unwrap(),
        ScheduleOutcome::Queued
    );
    source.next_random_position(BlockPos::new(128, 0, 0), 15);
    source.gameplay_random_mut().next_u64();
    assert!(matches!(
        source.capture_continuity(),
        Err(SimulationRuntimeError::TransientStateAtCommit {
            effects: 2,
            projection: 2,
        })
    ));
    source
        .drain_effects(SimulationQueueKind::Redstone, usize::MAX)
        .unwrap();
    source.project_and_clear(&registry_map(true)).unwrap();
    let continuity = source.capture_continuity().unwrap();
    let continuity_records = continuity.to_records().unwrap();
    let domains = continuity_records
        .iter()
        .map(|record| record.domain().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        domains,
        [
            "ferrite:phase5/boundary_receipt_v1".to_owned(),
            "ferrite:phase5/runtime_v1".to_owned(),
            "ferrite:phase5/scheduled_block_v1".to_owned(),
            "ferrite:phase5/scheduled_fluid_v1".to_owned(),
        ]
        .into_iter()
        .collect()
    );
    let snapshot = RegionCommitSnapshot::new(
        RegionSnapshotHeader {
            key: source.key().clone(),
            generation: source.generation(),
            committed_tick: source.tick().get(),
            persistence_revision: PersistenceRevision::INITIAL,
            region_side_chunks: 8,
            content_manifest: [1; 32],
            state_hash: [2; 32],
        },
        continuity_records,
    )
    .unwrap();
    let point = RegionRecoveryPoint::new(snapshot, Vec::new()).unwrap();
    let target_generation = ActivationGeneration::new(2).unwrap();
    let handoff = RegionHandoffState::prepare(point, target_generation).unwrap();
    let digest = *handoff.digest();
    let recovered = handoff.install(source.key(), digest).unwrap();
    let encoded = recovered.recovery_point().encode().unwrap();
    let decoded = RegionRecoveryPoint::decode(&encoded).unwrap();
    let restored_continuity =
        SimulationContinuity::from_records(decoded.snapshot().records()).unwrap();
    let mut restored = SimulationRegionRuntime::restore(
        decoded.snapshot().key().clone(),
        recovered.generation(),
        GameTick::new(decoded.committed_tick()),
        100,
        restored_continuity,
        config(16, 16, 16),
    )
    .unwrap();

    assert_eq!(
        restored.capture_continuity().unwrap(),
        source.capture_continuity().unwrap()
    );
    assert_eq!(
        restored.next_random_position(BlockPos::new(128, 0, 0), 15),
        source.next_random_position(BlockPos::new(128, 0, 0), 15)
    );
    assert_eq!(
        restored.gameplay_random_mut().next_u64(),
        source.gameplay_random_mut().next_u64()
    );
    let replay = transaction(
        target_generation,
        16,
        [BlockStateId::new(0), BlockStateId::new(0)],
    );
    assert_eq!(
        restored.apply_transaction(&mut state, &replay).unwrap(),
        BoundaryApplyOutcome::AlreadyApplied
    );

    restored.advance_commit(GameTick::new(8), 105).unwrap();
    let mut due = Vec::new();
    assert_eq!(
        restored.tick_scheduled(
            ScheduledQueueKind::Block,
            16,
            |_| true,
            |tick| due.push(tick.type_identity),
        ),
        2
    );
    assert_eq!(
        due,
        [
            ResourceId::minecraft("redstone_wire").unwrap(),
            ResourceId::minecraft("stone").unwrap(),
        ]
    );
}

#[test]
fn stale_target_generation_is_rejected_before_mutation() {
    let mut runtime = runtime(config(8, 8, 8));
    let mut state = voxels();
    let transaction = transaction(
        ActivationGeneration::new(2).unwrap(),
        17,
        [BlockStateId::new(0), BlockStateId::new(0)],
    );
    assert!(matches!(
        runtime.apply_transaction(&mut state, &transaction),
        Err(SimulationRuntimeError::StaleTargetGeneration { .. })
    ));
    assert_eq!(
        state.view().block_state(FIRST).unwrap(),
        BlockStateId::new(0)
    );
}
