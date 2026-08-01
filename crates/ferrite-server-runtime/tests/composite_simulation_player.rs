use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_server_runtime::composite::runtime::CompositeRuntimeConfig;
use ferrite_server_runtime::composite::services::{
    CompositeServiceAction, CompositeServiceCommand, CompositeServiceOutcome,
    CompositeServiceRuntimeError, SimulationPlayerRegionRuntime, SimulationPlayerRuntimeConfig,
};
use ferrite_server_runtime::player_service::model::{
    PlayerActionHeader, PlayerMutation, PlayerPersistentState,
};
use ferrite_server_runtime::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
use ferrite_server_runtime::simulation::continuity::ScheduledQueueKind;
use ferrite_server_runtime::simulation::runtime::SimulationRuntimeConfig;
use ferrite_simulation::scheduled_tick::record::TickPriority;
use ferrite_simulation::tick::GameTick;

fn key() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

fn config(projection_capacity: usize) -> SimulationPlayerRuntimeConfig {
    let mut coordinator = CompositeRuntimeConfig::testing();
    coordinator.projection_capacity = projection_capacity;
    SimulationPlayerRuntimeConfig {
        coordinator,
        simulation: SimulationRuntimeConfig {
            mapping: RegionMapping::V1,
            budget: SimulationQueueBudget::new([
                (SimulationQueueKind::ScheduledBlocks, 16),
                (SimulationQueueKind::ScheduledFluids, 16),
                (SimulationQueueKind::BoundaryTransactions, 16),
                (SimulationQueueKind::ImmediateNeighbors, 16),
                (SimulationQueueKind::Fluids, 16),
                (SimulationQueueKind::Redstone, 16),
                (SimulationQueueKind::Lighting, 16),
                (SimulationQueueKind::ProjectionPositions, 16),
            ])
            .unwrap(),
            projection_capacity: 16,
            receipt_capacity: 16,
            gameplay_random_seed: 7,
        },
        player_capacity: 8,
        projection_capacity_per_player: 8,
    }
}

fn runtime(projection_capacity: usize) -> SimulationPlayerRegionRuntime {
    SimulationPlayerRegionRuntime::new(
        key(),
        ActivationGeneration::INITIAL,
        GameTick::ZERO,
        0,
        [ChunkPos::new(0, 0)],
        config(projection_capacity),
    )
    .unwrap()
}

fn player(value: u128) -> StableEntityId {
    StableEntityId::new(value).unwrap()
}

fn join(tick: u64, sequence: u64, player: StableEntityId) -> CompositeServiceCommand {
    CompositeServiceCommand::new(
        GameTick::new(tick),
        sequence,
        CompositeServiceAction::JoinPlayer {
            player,
            state: PlayerPersistentState::default(),
        },
    )
}

fn schedule(tick: u64, sequence: u64) -> CompositeServiceCommand {
    CompositeServiceCommand::new(
        GameTick::new(tick),
        sequence,
        CompositeServiceAction::ScheduleSimulation {
            kind: ScheduledQueueKind::Block,
            type_identity: ResourceId::minecraft("stone").unwrap(),
            position: BlockPos::new(1, 64, 1),
            delay: 2,
            priority: TickPriority::Normal,
        },
    )
}

#[test]
fn player_and_simulation_services_commit_in_one_composite_tick() {
    let owner = player(1);
    let mut runtime = runtime(8);
    runtime.admit_command(schedule(1, 2)).unwrap();
    runtime.admit_command(join(1, 1, owner)).unwrap();

    let report = runtime.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();
    assert_eq!(report.commit.tick, GameTick::new(1));
    assert_eq!(report.commit.projection_count, 1);
    assert_ne!(report.commit.continuity_hash, [0; 32]);
    assert_eq!(report.events.len(), 9);
    assert_eq!(report.projections.len(), 1);
    assert_eq!(runtime.simulation().tick(), GameTick::new(1));
    assert!(runtime.players().state(owner).is_some());
    assert!(report.outcomes.iter().any(|outcome| matches!(
        outcome,
        CompositeServiceOutcome::PlayerJoined { player, .. } if *player == owner
    )));
    assert!(
        report
            .outcomes
            .iter()
            .any(|outcome| matches!(outcome, CompositeServiceOutcome::SimulationScheduled { .. }))
    );
}

#[test]
fn player_item_state_mutation_projects_only_after_composite_commit() {
    let owner = player(2);
    let mut runtime = runtime(8);
    runtime.admit_command(join(1, 1, owner)).unwrap();
    runtime.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();

    let session_epoch = runtime.players().session_epoch(owner).unwrap();
    let state = runtime.players().state(owner).unwrap();
    let mut mutation = PlayerMutation::from_state(&state);
    mutation.selected_slot = 4;
    runtime
        .admit_command(CompositeServiceCommand::new(
            GameTick::new(2),
            2,
            CompositeServiceAction::ApplyPlayerAction {
                header: PlayerActionHeader {
                    region: key(),
                    generation: ActivationGeneration::INITIAL,
                    player: owner,
                    session_epoch,
                    sequence: 1,
                },
                mutation,
            },
        ))
        .unwrap();

    let report = runtime.run_tick(GameTick::new(2), 2, usize::MAX).unwrap();
    assert_eq!(report.projections.len(), 1);
    assert_eq!(runtime.players().state(owner).unwrap().selected_slot, 4);
    assert!(report.outcomes.iter().any(|outcome| matches!(
        outcome,
        CompositeServiceOutcome::PlayerAction { player, .. } if *player == owner
    )));
}

#[test]
fn cross_service_replay_is_independent_of_admission_order() {
    let owner = player(3);
    let mut first = runtime(8);
    let mut second = runtime(8);
    first.admit_command(schedule(1, 2)).unwrap();
    first.admit_command(join(1, 1, owner)).unwrap();
    second.admit_command(join(1, 1, owner)).unwrap();
    second.admit_command(schedule(1, 2)).unwrap();

    let first = first.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();
    let second = second.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();
    assert_eq!(first.commit, second.commit);
    assert_eq!(first.outcomes, second.outcomes);
    assert_eq!(first.projections, second.projections);
}

#[test]
fn projection_backpressure_fails_before_player_mutation_and_poisons_tick() {
    let first = player(4);
    let second = player(5);
    let mut runtime = runtime(1);
    runtime.admit_command(join(1, 1, first)).unwrap();
    runtime.admit_command(join(1, 2, second)).unwrap();

    assert!(matches!(
        runtime.run_tick(GameTick::new(1), 1, usize::MAX),
        Err(CompositeServiceRuntimeError::ProjectionBackpressure {
            required: 2,
            remaining: 1,
        })
    ));
    assert!(runtime.players().state(first).is_none());
    assert!(runtime.players().state(second).is_none());
    assert!(runtime.is_poisoned());
    assert!(matches!(
        runtime.run_tick(GameTick::new(1), 1, usize::MAX),
        Err(CompositeServiceRuntimeError::Poisoned)
    ));
}
