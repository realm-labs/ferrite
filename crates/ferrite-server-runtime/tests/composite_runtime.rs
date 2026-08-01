use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{SnapshotRecord, SnapshotRecordKind};
use ferrite_server_runtime::composite::model::{
    CompositeCommand, CompositeOwner, CompositeProjection, CompositeStage,
};
use ferrite_server_runtime::composite::runtime::{
    CompositeCapacity, CompositeRegionRuntime, CompositeRuntimeConfig, CompositeRuntimeError,
};
use ferrite_server_runtime::continuity::identity::{
    ContinuityDomain, ContinuityGeneration, domain_id,
};
use ferrite_simulation::tick::GameTick;

fn key() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

fn runtime(config: CompositeRuntimeConfig) -> CompositeRegionRuntime {
    CompositeRegionRuntime::new(key(), ActivationGeneration::INITIAL, GameTick::ZERO, config)
        .unwrap()
}

fn command(owner: CompositeOwner, sequence: u64) -> CompositeCommand {
    CompositeCommand::new(
        GameTick::new(1),
        owner,
        sequence,
        ResourceId::new("ferrite", "test/command").unwrap(),
        vec![sequence as u8],
    )
}

fn continuity() -> Vec<SnapshotRecord> {
    vec![
        SnapshotRecord::new(
            SnapshotRecordKind::Extension,
            domain_id(
                ContinuityDomain::SimulationRuntime,
                ContinuityGeneration::Current,
            ),
            Vec::new(),
            vec![1],
        )
        .unwrap(),
    ]
}

fn run_to_commit(runtime: &mut CompositeRegionRuntime) -> [u8; 32] {
    runtime.begin_tick(GameTick::new(1)).unwrap();
    for stage in CompositeStage::ALL {
        runtime.enter_stage(stage).unwrap();
        if stage == CompositeStage::Continuity {
            runtime.prepare_continuity(continuity()).unwrap();
        }
        let receipt = runtime.complete_stage().unwrap();
        if let Some(receipt) = receipt {
            return receipt.replay_identity;
        }
    }
    panic!("commit stage must produce a receipt")
}

#[test]
fn stage_order_is_exact_and_commit_precedes_projection() {
    let mut runtime = runtime(CompositeRuntimeConfig::testing());
    runtime.begin_tick(GameTick::new(1)).unwrap();
    assert!(matches!(
        runtime.enter_stage(CompositeStage::Simulation),
        Err(CompositeRuntimeError::WrongStage {
            expected: CompositeStage::Ingress,
            actual: CompositeStage::Simulation,
        })
    ));
    for stage in CompositeStage::ALL {
        runtime.enter_stage(stage).unwrap();
        if stage == CompositeStage::Continuity {
            assert!(matches!(
                runtime.complete_stage(),
                Err(CompositeRuntimeError::ContinuityNotPrepared)
            ));
            runtime.prepare_continuity(continuity()).unwrap();
        }
        if stage == CompositeStage::Projection {
            assert_eq!(runtime.committed_tick(), GameTick::new(1));
        }
        runtime.complete_stage().unwrap();
    }
    assert_eq!(runtime.committed_tick(), GameTick::new(1));
    assert_eq!(runtime.take_events(usize::MAX).len(), 9);
}

#[test]
fn command_and_projection_budgets_fail_atomically() {
    let mut config = CompositeRuntimeConfig::testing();
    config.command_capacity = 1;
    config.projection_capacity = 1;
    let mut runtime = runtime(config);
    runtime
        .admit_command(command(CompositeOwner::Ingress, 1))
        .unwrap();
    assert!(matches!(
        runtime.admit_command(command(CompositeOwner::Simulation, 2)),
        Err(CompositeRuntimeError::Full {
            kind: CompositeCapacity::Commands,
            capacity: 1,
        })
    ));
    runtime.begin_tick(GameTick::new(1)).unwrap();
    runtime.enter_stage(CompositeStage::Ingress).unwrap();
    assert_eq!(runtime.commands(CompositeOwner::Ingress).unwrap().len(), 1);
    runtime
        .queue_projection(CompositeProjection::new(
            CompositeOwner::Ingress,
            1,
            ResourceId::new("ferrite", "test/projection").unwrap(),
            vec![1],
        ))
        .unwrap();
    assert!(matches!(
        runtime.queue_projection(CompositeProjection::new(
            CompositeOwner::Ingress,
            2,
            ResourceId::new("ferrite", "test/projection").unwrap(),
            vec![2],
        )),
        Err(CompositeRuntimeError::Full {
            kind: CompositeCapacity::Projections,
            capacity: 1,
        })
    ));
}

#[test]
fn replay_identity_is_independent_of_command_admission_order() {
    let mut first = runtime(CompositeRuntimeConfig::testing());
    let mut second = runtime(CompositeRuntimeConfig::testing());
    for runtime in [&mut first, &mut second] {
        runtime
            .admit_command(command(CompositeOwner::Simulation, 2))
            .unwrap();
        runtime
            .admit_command(command(CompositeOwner::Ingress, 1))
            .unwrap();
    }
    let commands = [
        command(CompositeOwner::Ingress, 1),
        command(CompositeOwner::Simulation, 2),
    ];
    second = runtime(CompositeRuntimeConfig::testing());
    for command in commands {
        second.admit_command(command).unwrap();
    }
    assert_eq!(run_to_commit(&mut first), run_to_commit(&mut second));
}

#[test]
fn projections_are_invisible_until_commit_and_legacy_continuity_is_denied() {
    let mut runtime = runtime(CompositeRuntimeConfig::testing());
    runtime.begin_tick(GameTick::new(1)).unwrap();
    for stage in CompositeStage::ALL.into_iter().take(7) {
        runtime.enter_stage(stage).unwrap();
        if stage == CompositeStage::Ingress {
            runtime
                .queue_projection(CompositeProjection::new(
                    CompositeOwner::Ingress,
                    2,
                    ResourceId::new("ferrite", "test/projection").unwrap(),
                    vec![2],
                ))
                .unwrap();
            assert!(matches!(
                runtime.drain_projections(1),
                Err(CompositeRuntimeError::WrongStage { .. })
            ));
        }
        if stage == CompositeStage::Continuity {
            let legacy = SnapshotRecord::new(
                SnapshotRecordKind::Extension,
                domain_id(
                    ContinuityDomain::SimulationRuntime,
                    ContinuityGeneration::Legacy,
                ),
                Vec::new(),
                vec![1],
            )
            .unwrap();
            assert!(matches!(
                runtime.prepare_continuity(vec![legacy]),
                Err(CompositeRuntimeError::LegacyContinuityWrite)
            ));
            runtime.prepare_continuity(continuity()).unwrap();
        }
        runtime.complete_stage().unwrap();
    }
    runtime.enter_stage(CompositeStage::Commit).unwrap();
    runtime.complete_stage().unwrap().unwrap();
    runtime.enter_stage(CompositeStage::Projection).unwrap();
    assert_eq!(runtime.drain_projections(usize::MAX).unwrap().len(), 1);
    runtime.complete_stage().unwrap();
}

#[test]
fn event_and_continuity_backpressure_preserve_the_active_stage() {
    let mut config = CompositeRuntimeConfig::testing();
    config.event_capacity = 1;
    config.continuity_record_capacity = 1;
    let mut runtime = runtime(config);
    runtime.begin_tick(GameTick::new(1)).unwrap();
    runtime.enter_stage(CompositeStage::Ingress).unwrap();
    runtime.complete_stage().unwrap();
    runtime.enter_stage(CompositeStage::PlayerService).unwrap();
    assert!(matches!(
        runtime.complete_stage(),
        Err(CompositeRuntimeError::Full {
            kind: CompositeCapacity::Events,
            capacity: 1,
        })
    ));
    assert_eq!(runtime.take_events(1).len(), 1);
    runtime.complete_stage().unwrap();
    assert_eq!(runtime.take_events(1).len(), 1);

    for stage in [
        CompositeStage::Simulation,
        CompositeStage::EntityService,
        CompositeStage::WorldService,
        CompositeStage::Reconciliation,
    ] {
        runtime.enter_stage(stage).unwrap();
        runtime.complete_stage().unwrap();
        runtime.take_events(1);
    }
    runtime.enter_stage(CompositeStage::Continuity).unwrap();
    let mut too_many = continuity();
    too_many.push(
        SnapshotRecord::new(
            SnapshotRecordKind::Extension,
            ResourceId::new("test", "auxiliary").unwrap(),
            vec![1],
            vec![2],
        )
        .unwrap(),
    );
    assert!(matches!(
        runtime.prepare_continuity(too_many),
        Err(CompositeRuntimeError::Full {
            kind: CompositeCapacity::ContinuityRecords,
            capacity: 1,
        })
    ));
    runtime.prepare_continuity(continuity()).unwrap();
}

#[test]
fn command_identity_and_tick_horizon_fail_closed() {
    let mut runtime = runtime(CompositeRuntimeConfig::testing());
    runtime
        .admit_command(command(CompositeOwner::Ingress, 1))
        .unwrap();
    assert!(matches!(
        runtime.admit_command(command(CompositeOwner::Ingress, 1)),
        Err(CompositeRuntimeError::DuplicateCommand { .. })
    ));
    assert!(matches!(
        runtime.admit_command(CompositeCommand::new(
            GameTick::new(5),
            CompositeOwner::Ingress,
            2,
            ResourceId::new("ferrite", "test/future").unwrap(),
            Vec::new(),
        )),
        Err(CompositeRuntimeError::CommandBeyondHorizon { .. })
    ));
}
