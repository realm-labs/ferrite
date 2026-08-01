use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_server_runtime::composite::runtime::CompositeRuntimeConfig;
use ferrite_server_runtime::composite::services::{
    CompositeProductionRegionRuntime, CompositeProductionRuntimeConfig, CompositeServiceAction,
    CompositeServiceCommand, CompositeServiceOutcome,
};
use ferrite_server_runtime::entity_service::model::{
    EntityMutation, EntityPayload, EntityPersistentState, EntityTransferRequest,
};
use ferrite_server_runtime::entity_service::runtime::EntityServiceRuntimeLimits;
use ferrite_server_runtime::entity_service::transfer::TransferAcceptance;
use ferrite_server_runtime::simulation::boundary::{
    BoundaryMechanic, BoundaryMutation, BoundaryTransactionHeader, BoundaryTransactionLimits,
    MechanicBoundaryTransaction,
};
use ferrite_server_runtime::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
use ferrite_server_runtime::simulation::runtime::SimulationRuntimeConfig;
use ferrite_server_runtime::world_service::model::WorldServiceRuntimeConfig;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};

fn key(x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(x, 0),
        RegionMappingVersion::V1,
    )
}

fn config() -> CompositeProductionRuntimeConfig {
    CompositeProductionRuntimeConfig {
        coordinator: CompositeRuntimeConfig {
            command_capacity: 64,
            event_capacity: 32,
            projection_capacity: 64,
            continuity_record_capacity: 64,
            maximum_future_ticks: 4,
            maximum_payload_bytes: 1024 * 1024,
        },
        simulation: SimulationRuntimeConfig {
            mapping: RegionMapping::V1,
            budget: SimulationQueueBudget::new([
                (SimulationQueueKind::ScheduledBlocks, 32),
                (SimulationQueueKind::ScheduledFluids, 32),
                (SimulationQueueKind::BoundaryTransactions, 32),
                (SimulationQueueKind::ImmediateNeighbors, 32),
                (SimulationQueueKind::Fluids, 32),
                (SimulationQueueKind::Redstone, 32),
                (SimulationQueueKind::Lighting, 32),
                (SimulationQueueKind::ProjectionPositions, 32),
            ])
            .unwrap(),
            projection_capacity: 32,
            receipt_capacity: 32,
            gameplay_random_seed: 11,
        },
        entities: EntityServiceRuntimeLimits::new(32, 32, 32, 32),
        world: WorldServiceRuntimeConfig {
            mapping: RegionMapping::V1,
            layout: ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).unwrap(),
                BlockStateId::new(0),
                BiomeId::new(0),
            ),
            region_side_chunks: 8,
            chunk_capacity: 32,
            event_capacity: 64,
            content_manifest: [9; 32],
        },
        player_capacity: 16,
        projection_capacity_per_player: 16,
    }
}

fn runtime(region_x: i32, chunk: ChunkPos) -> CompositeProductionRegionRuntime {
    CompositeProductionRegionRuntime::new(
        key(region_x),
        ActivationGeneration::INITIAL,
        GameTick::ZERO,
        0,
        [chunk],
        config(),
    )
    .unwrap()
}

fn entity(value: u128) -> StableEntityId {
    StableEntityId::new(value).unwrap()
}

#[test]
fn boundary_transaction_mutates_world_and_publishes_semantic_projection_after_commit() {
    let position = BlockPos::new(1, 64, 1);
    let transaction = MechanicBoundaryTransaction::new(
        BoundaryTransactionHeader {
            tick: GameTick::ZERO,
            source: key(-1),
            source_generation: ActivationGeneration::INITIAL,
            target: key(0),
            target_generation: ActivationGeneration::INITIAL,
            source_sequence: 1,
        },
        BoundaryMechanic::Neighbor,
        vec![BoundaryMutation {
            order: 0,
            position,
            expected: BlockStateId::new(0),
            replacement: BlockStateId::new(9),
        }],
        Vec::new(),
        RegionMapping::V1,
        BoundaryTransactionLimits::new(8, 8),
    )
    .unwrap();
    let mut runtime = runtime(0, ChunkPos::new(0, 0));
    runtime
        .admit_command(CompositeServiceCommand::new(
            GameTick::new(1),
            1,
            CompositeServiceAction::ApplyBoundaryTransaction { transaction },
        ))
        .unwrap();

    let report = runtime.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();
    assert_eq!(report.projections.len(), 1);
    assert_eq!(
        runtime
            .world()
            .chunk(ChunkPos::new(0, 0))
            .unwrap()
            .block_state(position)
            .unwrap(),
        BlockStateId::new(9)
    );
    assert!(
        report
            .outcomes
            .iter()
            .any(|outcome| matches!(outcome, CompositeServiceOutcome::BoundaryApplied { .. }))
    );
    assert!(report.outcomes.iter().any(|outcome| matches!(
        outcome,
        CompositeServiceOutcome::SimulationEffects { effects, .. } if effects.len() == 1
    )));
}

#[test]
fn entity_observer_projection_and_four_service_continuity_share_one_commit() {
    let observer = entity(1);
    let target = entity(2);
    let state = EntityPersistentState::active(
        ResourceId::minecraft("pig").unwrap(),
        ChunkPos::new(0, 0),
        EntityPayload::default(),
    );
    let mut runtime = runtime(0, ChunkPos::new(0, 0));
    runtime
        .admit_command(CompositeServiceCommand::new(
            GameTick::new(1),
            2,
            CompositeServiceAction::InsertEntity {
                entity: target,
                state,
            },
        ))
        .unwrap();
    runtime
        .admit_command(CompositeServiceCommand::new(
            GameTick::new(1),
            1,
            CompositeServiceAction::AddEntityObserver { observer },
        ))
        .unwrap();

    let report = runtime.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();
    assert_eq!(runtime.entities().entity_count(), 1);
    assert_eq!(report.projections.len(), 1);
    assert_eq!(report.commit.continuity_record_count, 5);
    assert_eq!(report.continuity.records.len(), 5);
    assert_ne!(report.commit.continuity_hash, [0; 32]);
    assert_eq!(runtime.world().chunks().count(), 1);
}

#[test]
fn two_phase_entity_transfer_routes_prepare_accept_and_commit_across_regions() {
    let stable = entity(3);
    let source_key = key(0);
    let target_key = key(1);
    let initial = EntityPersistentState::active(
        ResourceId::minecraft("pig").unwrap(),
        ChunkPos::new(0, 0),
        EntityPayload::default(),
    );
    let mut source = runtime(0, ChunkPos::new(0, 0));
    let mut target = runtime(1, ChunkPos::new(8, 0));
    source
        .admit_command(CompositeServiceCommand::new(
            GameTick::new(1),
            1,
            CompositeServiceAction::InsertEntity {
                entity: stable,
                state: initial,
            },
        ))
        .unwrap();
    source.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();
    target.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();

    source
        .admit_command(CompositeServiceCommand::new(
            GameTick::new(2),
            1,
            CompositeServiceAction::PrepareEntityTransfer {
                request: EntityTransferRequest {
                    tick: GameTick::new(2),
                    source: source_key.clone(),
                    source_generation: ActivationGeneration::INITIAL,
                    target: target_key,
                    target_generation: ActivationGeneration::INITIAL,
                    entity: stable,
                    expected_revision: 0,
                    sequence: 1,
                    candidate: EntityMutation {
                        chunk: ChunkPos::new(8, 0),
                        payload: EntityPayload::default(),
                    },
                },
            },
        ))
        .unwrap();
    let prepared = source.run_tick(GameTick::new(2), 2, usize::MAX).unwrap();
    let transfer = prepared
        .outcomes
        .iter()
        .find_map(|outcome| match outcome {
            CompositeServiceOutcome::EntityTransferPrepared { transfer, .. } => {
                Some(transfer.clone())
            }
            _ => None,
        })
        .unwrap();

    target
        .admit_command(CompositeServiceCommand::new(
            GameTick::new(2),
            1,
            CompositeServiceAction::AcceptEntityTransfer { transfer },
        ))
        .unwrap();
    let accepted = target.run_tick(GameTick::new(2), 2, usize::MAX).unwrap();
    let receipt =
        accepted
            .outcomes
            .iter()
            .find_map(|outcome| match outcome {
                CompositeServiceOutcome::EntityTransferAccepted {
                    acceptance:
                        TransferAcceptance::Accepted(receipt)
                        | TransferAcceptance::AlreadyApplied(receipt),
                    ..
                } => Some(receipt.clone()),
                _ => None,
            })
            .unwrap();
    source
        .admit_command(CompositeServiceCommand::new(
            GameTick::new(3),
            1,
            CompositeServiceAction::CommitEntityTransfer { receipt },
        ))
        .unwrap();
    source.run_tick(GameTick::new(3), 3, usize::MAX).unwrap();

    assert!(source.entities().state(stable).is_none());
    assert!(target.entities().state(stable).is_some());
    assert_eq!(target.entities().applied_transfer_count(), 1);
}
