use bevy_ecs::prelude::Component;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_region_runtime::immediate::{ImmediateBoundaryEffect, ImmediateEffectHeader};
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_region_runtime::logic::{
    ImmediateEffectContext, RegionLogic, RegionLogicError, RegionPhaseContext, RegionPhaseOutput,
};
use ferrite_region_runtime::transfer::{
    EntityTransfer, EntityTransferHeader, TransferRole, TransferredEntityState,
};
use ferrite_simulation::boundary::{BoundaryBatch, BoundaryBatchHeader, BoundaryEvent};
use ferrite_simulation::command::{CommandSource, RegionCommand};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::{GameTick, TickPhase};
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
struct Counter(u32);

fn region(x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(x, 0),
        RegionMappingVersion::V1,
    )
}

fn state(x: i32) -> RegionSimulationState {
    RegionSimulationState::new(
        RegionVoxelState::new(
            region(x),
            RegionMapping::V1,
            ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).unwrap(),
                BlockStateId::new(0),
                BiomeId::new(0),
            ),
        )
        .unwrap(),
    )
}

struct CrossingLogic {
    trace: Vec<(TickPhase, i32)>,
    command_seen: bool,
    boundary_seen: bool,
    transfer_id: StableEntityId,
    counter_id: StableEntityId,
}

impl RegionLogic for CrossingLogic {
    fn execute_phase(
        &mut self,
        context: RegionPhaseContext<'_>,
        output: &mut RegionPhaseOutput,
    ) -> Result<(), RegionLogicError> {
        let x = context.key().coordinate().x();
        let phase = context.phase();
        self.trace.push((phase, x));
        if x == 0 && phase == TickPhase::Ingress {
            self.command_seen = context.commands().len() == 1;
        }
        if x == 0 && phase == TickPhase::ReconcileBoundary {
            self.boundary_seen = context.boundaries().len() == 1;
        }
        if x == 1 && phase == TickPhase::ImmediateNeighbors {
            output
                .emit_immediate(
                    ImmediateBoundaryEffect::new(
                        ImmediateEffectHeader {
                            tick: context.tick(),
                            phase,
                            source: region(1),
                            target: region(0),
                            source_generation: ActivationGeneration::INITIAL,
                            target_generation: ActivationGeneration::INITIAL,
                            source_sequence: 1,
                        },
                        ResourceId::new("ferrite", "effect/increment").unwrap(),
                        vec![1],
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        if x == 1 && phase == TickPhase::EntityResolution {
            output
                .emit_transfer(
                    EntityTransfer::new(
                        EntityTransferHeader {
                            tick: context.tick(),
                            source: region(1),
                            target: region(0),
                            source_generation: ActivationGeneration::INITIAL,
                            target_generation: ActivationGeneration::INITIAL,
                            source_sequence: 2,
                            stable_id: self.transfer_id,
                            role: TransferRole::Player,
                        },
                        ResourceId::minecraft("player").unwrap(),
                        vec![7, 8],
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        if x == 1 && phase == TickPhase::EmitBoundary {
            output
                .emit_boundary(
                    BoundaryBatch::new(
                        BoundaryBatchHeader {
                            tick: context.tick(),
                            phase: TickPhase::ReconcileBoundary,
                            source: region(1),
                            target: region(0),
                            source_generation: ActivationGeneration::INITIAL,
                            source_sequence: 3,
                        },
                        vec![
                            BoundaryEvent::new(
                                0,
                                ResourceId::new("ferrite", "boundary/test").unwrap(),
                                vec![9],
                            )
                            .unwrap(),
                        ],
                        1,
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        Ok(())
    }

    fn apply_immediate_effect(
        &mut self,
        mut context: ImmediateEffectContext<'_>,
    ) -> Result<(), RegionLogicError> {
        assert_eq!(context.effect().payload(), [1]);
        context
            .state_mut()
            .entities_mut()
            .update_component::<Counter, _>(self.counter_id, |counter| counter.0 += 1)
            .unwrap();
        Ok(())
    }
}

#[test]
fn local_runner_orders_regions_and_applies_boundary_work_before_commit() {
    let transfer_id = StableEntityId::new(7).unwrap();
    let counter_id = StableEntityId::new(9).unwrap();
    let mut right = state(1);
    right.entities_mut().spawn(transfer_id).unwrap();
    let mut left = state(0);
    left.entities_mut().spawn(counter_id).unwrap();
    left.entities_mut()
        .insert_component(counter_id, Counter(0))
        .unwrap();

    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(right, ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    runner
        .insert_region(left, ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    runner
        .admit_command(
            RegionCommand::new(
                region(0),
                GameTick::new(1),
                CommandSource::System(ResourceId::new("ferrite", "test").unwrap()),
                0,
                ResourceId::new("ferrite", "command/test").unwrap(),
                vec![],
            )
            .unwrap(),
        )
        .unwrap();

    let mut logic = CrossingLogic {
        trace: Vec::new(),
        command_seen: false,
        boundary_seen: false,
        transfer_id,
        counter_id,
    };
    let report = runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    assert_eq!(report.commits().len(), 2);
    assert_eq!(report.commits()[0].key(), &region(0));
    assert_eq!(report.immediate_effects(), 1);
    assert_eq!(report.entity_transfers(), 1);
    assert!(logic.command_seen);
    assert!(logic.boundary_seen);
    for pair in logic.trace.chunks_exact(2) {
        assert_eq!(pair[0].1, 0);
        assert_eq!(pair[1].1, 1);
        assert_eq!(pair[0].0, pair[1].0);
    }

    let left = runner.region(&region(0)).unwrap().state().entities();
    let right = runner.region(&region(1)).unwrap().state().entities();
    assert!(left.contains(transfer_id));
    assert!(!right.contains(transfer_id));
    assert_eq!(left.component::<Counter>(counter_id), Some(&Counter(1)));
    let transferred = left
        .component::<TransferredEntityState>(transfer_id)
        .unwrap();
    assert_eq!(transferred.role(), TransferRole::Player);
    assert_eq!(transferred.state(), [7, 8]);
}

struct FailingLogic;

impl RegionLogic for FailingLogic {
    fn execute_phase(
        &mut self,
        context: RegionPhaseContext<'_>,
        _output: &mut RegionPhaseOutput,
    ) -> Result<(), RegionLogicError> {
        if context.phase() == TickPhase::PlayerIntent {
            return Err(RegionLogicError::new(
                ResourceId::new("ferrite", "test/failure").unwrap(),
            ));
        }
        Ok(())
    }

    fn apply_immediate_effect(
        &mut self,
        _context: ImmediateEffectContext<'_>,
    ) -> Result<(), RegionLogicError> {
        Ok(())
    }
}

#[test]
fn failed_tick_poisoning_prevents_accidental_continuation() {
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(state(0), ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    assert!(
        runner
            .run_tick(GameTick::new(1), &mut FailingLogic)
            .is_err()
    );
    assert!(runner.is_poisoned());
    assert_eq!(
        runner.region(&region(0)).unwrap().committed_tick(),
        GameTick::ZERO
    );
    assert!(
        runner
            .run_tick(GameTick::new(1), &mut FailingLogic)
            .is_err()
    );
}
