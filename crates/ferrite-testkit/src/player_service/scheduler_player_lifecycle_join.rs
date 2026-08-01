//! Executable TickScheduler × PlayerLifecycle conformance.

use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_server_runtime::player::logic::PlayerRegionLogic;
use ferrite_simulation::command::{CommandSource, RegionCommand};
use ferrite_simulation::tick::GameTick;

use crate::player_service::fixtures::{
    initial_generation, join_command, leave_command, leave_payload, player, region,
    simulation_state,
};

const PROPERTY_CASES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerPlayerLifecycleReport {
    pub property_cases: usize,
    pub fault_cases: usize,
    pub visible_after_join_capture: bool,
    pub visible_before_leave_capture: bool,
    pub visible_after_leave_capture: bool,
    pub same_tick_final_membership: usize,
}

pub fn run_tick_scheduler_player_lifecycle() -> SchedulerPlayerLifecycleReport {
    let (after_join, before_leave, after_leave) = run_capture_boundary();
    let same_tick_final_membership = run_same_tick_order();
    run_property_sweep();
    let fault_cases = run_fault_vectors();
    SchedulerPlayerLifecycleReport {
        property_cases: PROPERTY_CASES,
        fault_cases,
        visible_after_join_capture: after_join,
        visible_before_leave_capture: before_leave,
        visible_after_leave_capture: after_leave,
        same_tick_final_membership,
    }
}

fn run_capture_boundary() -> (bool, bool, bool) {
    let mut runner = runner();
    let mut logic = PlayerRegionLogic;
    runner.admit_command(join_command(1, 1, 0)).unwrap();
    runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    let after_join = contains(&runner, 1);
    runner.admit_command(leave_command(1, 2, 1)).unwrap();
    let before_leave = contains(&runner, 1);
    runner.run_tick(GameTick::new(2), &mut logic).unwrap();
    (after_join, before_leave, contains(&runner, 1))
}

fn run_same_tick_order() -> usize {
    let mut runner = runner();
    let mut logic = PlayerRegionLogic;
    runner.admit_command(join_command(1, 1, 0)).unwrap();
    runner.admit_command(leave_command(1, 1, 1)).unwrap();
    runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    runner
        .region(&region())
        .unwrap()
        .state()
        .entities()
        .stable_ids()
        .count()
}

fn run_property_sweep() {
    for case in 0..PROPERTY_CASES {
        let value = case as u128 + 1;
        let mut runner = runner();
        let mut logic = PlayerRegionLogic;
        runner.admit_command(join_command(value, 1, 0)).unwrap();
        runner.run_tick(GameTick::new(1), &mut logic).unwrap();
        assert!(contains(&runner, value));
        runner.admit_command(leave_command(value, 2, 1)).unwrap();
        assert!(contains(&runner, value));
        runner.run_tick(GameTick::new(2), &mut logic).unwrap();
        assert!(!contains(&runner, value));
    }
}

fn run_fault_vectors() -> usize {
    assert!(runner().admit_command(join_command(1, 5, 0)).is_err());

    let mut duplicate = runner();
    duplicate.admit_command(join_command(1, 1, 0)).unwrap();
    duplicate.admit_command(join_command(1, 1, 1)).unwrap();
    assert!(
        duplicate
            .run_tick(GameTick::new(1), &mut PlayerRegionLogic)
            .is_err()
    );

    let mut unknown_leave = runner();
    unknown_leave.admit_command(leave_command(1, 1, 0)).unwrap();
    assert!(
        unknown_leave
            .run_tick(GameTick::new(1), &mut PlayerRegionLogic)
            .is_err()
    );

    let mut stale = runner();
    stale
        .run_tick(GameTick::new(1), &mut PlayerRegionLogic)
        .unwrap();
    assert!(stale.admit_command(join_command(1, 1, 0)).is_err());

    let mut mismatched = runner();
    mismatched
        .admit_command(
            RegionCommand::new(
                region(),
                GameTick::new(1),
                CommandSource::Player(player(2)),
                0,
                ferrite_foundation::resource::ResourceId::new("ferrite", "session/leave").unwrap(),
                leave_payload(1).encode(),
            )
            .unwrap(),
        )
        .unwrap();
    assert!(
        mismatched
            .run_tick(GameTick::new(1), &mut PlayerRegionLogic)
            .is_err()
    );
    5
}

fn runner() -> LocalRegionRunner {
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(simulation_state(), initial_generation(), GameTick::ZERO)
        .unwrap();
    runner
}

fn contains(runner: &LocalRegionRunner, value: u128) -> bool {
    runner
        .region(&region())
        .unwrap()
        .state()
        .entities()
        .contains(player(value))
}
