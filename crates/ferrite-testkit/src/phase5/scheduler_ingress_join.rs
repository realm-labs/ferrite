//! Executable TickScheduler × NetworkIngress cross-system conformance.

use crate::phase5::fixtures::{block_for_region, chunk_for_region, region, simulation_state};
use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::resource::ResourceId;
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_region_runtime::logic::{
    ImmediateEffectContext, RegionLogic, RegionLogicError, RegionPhaseContext, RegionPhaseOutput,
};
use ferrite_server_runtime::session::router::RegionCommandRouter;
use ferrite_simulation::command::{CommandSource, RegionCommand};
use ferrite_simulation::journal::JournalDomain;
use ferrite_simulation::scheduled_tick::container::ChunkTickContainer;
use ferrite_simulation::scheduled_tick::level::{ScheduleOutcome, ScheduledTickQueue};
use ferrite_simulation::scheduled_tick::record::{ScheduledTick, TickPriority};
use ferrite_simulation::tick::{GameTick, TickPhase};
use ferrite_world::id::BlockStateId;

const POSITION: ferrite_foundation::coordinate::BlockPos = block_for_region(0, 0);
const PROPERTY_CASES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickSchedulerIngressReport {
    pub ingress_before_capture: BlockStateId,
    pub ingress_after_capture: BlockStateId,
    pub property_cases: usize,
    pub fault_cases: usize,
    pub callback_ticks: Vec<GameTick>,
}

pub fn run_tick_scheduler_network_ingress() -> TickSchedulerIngressReport {
    let before = run_before_capture(5);
    let after = run_after_capture(5);
    assert_eq!(before.final_state, BlockStateId::new(6));
    assert_eq!(after.final_state, BlockStateId::new(5));
    assert_eq!(
        before.journal_phases,
        [TickPhase::Ingress, TickPhase::ScheduledBlocks]
    );
    assert_eq!(after.first_journal_phases, [TickPhase::ScheduledBlocks]);
    run_property_sweep();
    run_fault_vectors();
    let callback_ticks = run_reschedule_vector();
    TickSchedulerIngressReport {
        ingress_before_capture: before.final_state,
        ingress_after_capture: after.final_state,
        property_cases: PROPERTY_CASES,
        fault_cases: 4,
        callback_ticks,
    }
}

fn run_property_sweep() {
    for replacement in 1..=PROPERTY_CASES as u32 {
        let before = run_before_capture(replacement);
        let after = run_after_capture(replacement);
        assert_eq!(
            before.final_state,
            BlockStateId::new(replacement + 1),
            "pre-capture ingress was not visible for state {replacement}"
        );
        assert_eq!(
            after.final_state,
            BlockStateId::new(replacement),
            "post-capture ingress leaked backward for state {replacement}"
        );
        assert_eq!(before.committed_commands, 1);
        assert_eq!(after.second_committed_commands, 1);
    }
}

fn run_fault_vectors() {
    let (mut duplicate_runner, mut duplicate_logic) =
        runner_and_logic(false, LocalRunnerConfig::testing());
    let duplicate = command(GameTick::new(1), 1, 4);
    RegionCommandRouter::route(&mut duplicate_runner, duplicate.clone()).unwrap();
    assert!(RegionCommandRouter::route(&mut duplicate_runner, duplicate).is_err());
    let report = duplicate_runner
        .run_tick(GameTick::new(1), &mut duplicate_logic)
        .unwrap();
    assert_eq!(report.committed_commands().len(), 1);
    assert_eq!(final_state(&duplicate_runner), BlockStateId::new(5));

    let (mut stale_runner, mut stale_logic) = runner_and_logic(false, LocalRunnerConfig::testing());
    stale_runner
        .run_tick(GameTick::new(1), &mut stale_logic)
        .unwrap();
    let before = final_state(&stale_runner);
    assert!(
        RegionCommandRouter::route(&mut stale_runner, command(GameTick::new(1), 2, 9),).is_err()
    );
    assert_eq!(final_state(&stale_runner), before);

    let constrained = LocalRunnerConfig {
        command_capacity: 1,
        ..LocalRunnerConfig::testing()
    };
    let (mut full_runner, mut full_logic) = runner_and_logic(false, constrained);
    RegionCommandRouter::route(&mut full_runner, command(GameTick::new(1), 3, 2)).unwrap();
    assert!(
        RegionCommandRouter::route(&mut full_runner, command(GameTick::new(1), 4, 8),).is_err()
    );
    full_runner
        .run_tick(GameTick::new(1), &mut full_logic)
        .unwrap();
    assert_eq!(final_state(&full_runner), BlockStateId::new(3));

    let (mut future_runner, _future_logic) = runner_and_logic(false, LocalRunnerConfig::testing());
    assert!(
        RegionCommandRouter::route(&mut future_runner, command(GameTick::new(6), 5, 7),).is_err()
    );
    assert_eq!(final_state(&future_runner), BlockStateId::new(0));
}

fn run_reschedule_vector() -> Vec<GameTick> {
    let (mut runner, mut logic) = runner_and_logic(true, LocalRunnerConfig::testing());
    runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    assert_eq!(logic.callback_ticks, [GameTick::new(1)]);
    runner.run_tick(GameTick::new(2), &mut logic).unwrap();
    assert_eq!(logic.callback_ticks, [GameTick::new(1), GameTick::new(2)]);
    logic.callback_ticks
}

fn run_before_capture(replacement: u32) -> BeforeCapture {
    let (mut runner, mut logic) = runner_and_logic(false, LocalRunnerConfig::testing());
    RegionCommandRouter::route(&mut runner, command(GameTick::new(1), 1, replacement)).unwrap();
    let report = runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    BeforeCapture {
        final_state: final_state(&runner),
        committed_commands: report.committed_commands().len(),
        journal_phases: report.commits()[0]
            .journal()
            .entries()
            .iter()
            .map(|entry| entry.phase())
            .collect(),
    }
}

fn run_after_capture(replacement: u32) -> AfterCapture {
    let (mut runner, mut logic) = runner_and_logic(false, LocalRunnerConfig::testing());
    let first = runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    let first_journal_phases = first.commits()[0]
        .journal()
        .entries()
        .iter()
        .map(|entry| entry.phase())
        .collect();
    RegionCommandRouter::route(&mut runner, command(GameTick::new(2), 1, replacement)).unwrap();
    let second = runner.run_tick(GameTick::new(2), &mut logic).unwrap();
    AfterCapture {
        final_state: final_state(&runner),
        second_committed_commands: second.committed_commands().len(),
        first_journal_phases,
    }
}

fn runner_and_logic(
    reschedule: bool,
    config: LocalRunnerConfig,
) -> (LocalRegionRunner, SchedulerIngressLogic) {
    let mut runner = LocalRegionRunner::new(config).unwrap();
    runner
        .insert_region(
            simulation_state(0),
            ActivationGeneration::INITIAL,
            GameTick::ZERO,
        )
        .unwrap();
    (runner, SchedulerIngressLogic::new(reschedule))
}

fn command(tick: GameTick, sequence: u64, replacement: u32) -> RegionCommand {
    RegionCommand::new(
        region(0),
        tick,
        CommandSource::Player(StableEntityId::new(1).expect("fixture player identity is nonzero")),
        sequence,
        ResourceId::new("ferrite", "network/set-block").expect("fixture command kind is valid"),
        replacement.to_be_bytes().to_vec(),
    )
    .expect("fixture semantic command is bounded")
}

fn final_state(runner: &LocalRegionRunner) -> BlockStateId {
    runner
        .region(&region(0))
        .expect("fixture Region remains active")
        .state()
        .voxels()
        .block_state(POSITION)
        .expect("fixture block remains loaded")
}

struct BeforeCapture {
    final_state: BlockStateId,
    committed_commands: usize,
    journal_phases: Vec<TickPhase>,
}

struct AfterCapture {
    final_state: BlockStateId,
    second_committed_commands: usize,
    first_journal_phases: Vec<TickPhase>,
}

struct SchedulerIngressLogic {
    scheduled: ScheduledTickQueue<u8>,
    reschedule: bool,
    callback_ticks: Vec<GameTick>,
}

impl SchedulerIngressLogic {
    fn new(reschedule: bool) -> Self {
        let mut scheduled = ScheduledTickQueue::new();
        scheduled.register_container(chunk_for_region(0), ChunkTickContainer::new());
        assert_eq!(
            scheduled.schedule(ScheduledTick::new(1, POSITION, 1, TickPriority::Normal, 0,)),
            ScheduleOutcome::Queued
        );
        Self {
            scheduled,
            reschedule,
            callback_ticks: Vec::new(),
        }
    }
}

impl RegionLogic for SchedulerIngressLogic {
    fn execute_phase(
        &mut self,
        mut context: RegionPhaseContext<'_>,
        _output: &mut RegionPhaseOutput,
    ) -> Result<(), RegionLogicError> {
        match context.phase() {
            TickPhase::Ingress => {
                let replacements = context
                    .commands()
                    .iter()
                    .map(|command| {
                        u32::from_be_bytes(
                            command
                                .payload()
                                .try_into()
                                .expect("fixture command has four-byte state"),
                        )
                    })
                    .collect::<Vec<_>>();
                for replacement in replacements {
                    context
                        .state_mut()
                        .voxels_mut()
                        .set_block(POSITION, BlockStateId::new(replacement))
                        .expect("fixture ingress position remains loaded");
                    context
                        .append_journal(
                            JournalDomain::Mutation,
                            ResourceId::new("ferrite", "network/set-block").unwrap(),
                            replacement.to_be_bytes().to_vec(),
                        )
                        .expect("fixture journal has capacity");
                }
            }
            TickPhase::ScheduledBlocks => {
                let tick = context.tick();
                let game_time = tick.get() as i64;
                self.scheduled.tick(
                    game_time,
                    16,
                    |_| true,
                    |scheduler, due| {
                        let current = context
                            .state()
                            .view()
                            .voxels()
                            .block_state(due.position)
                            .expect("scheduled position remains loaded");
                        let replacement = BlockStateId::new(current.get().wrapping_add(1));
                        context
                            .state_mut()
                            .voxels_mut()
                            .set_block(due.position, replacement)
                            .expect("scheduled mutation remains in Region");
                        context
                            .append_journal(
                                JournalDomain::Mutation,
                                ResourceId::new("ferrite", "scheduled/set-block").unwrap(),
                                replacement.get().to_be_bytes().to_vec(),
                            )
                            .expect("fixture journal has capacity");
                        self.callback_ticks.push(tick);
                        if self.reschedule && due.type_identity == 1 {
                            assert_eq!(
                                scheduler.schedule(ScheduledTick::new(
                                    2,
                                    due.position,
                                    game_time,
                                    TickPriority::ExtremelyHigh,
                                    1,
                                )),
                                ScheduleOutcome::Queued
                            );
                        }
                    },
                );
            }
            _ => {}
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
