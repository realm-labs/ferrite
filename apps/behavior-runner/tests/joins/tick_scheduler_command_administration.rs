use ferrite_testkit::service_conformance::joins::{
    JoinOracle, run_tick_scheduler_command_administration,
};

#[test]
fn commands_execute_at_an_explicit_tick_phase_boundary() {
    let report = run_tick_scheduler_command_administration();
    assert_eq!(report.oracle, JoinOracle::QueueThenExecute);
    assert_eq!(report.checkpoints, 3);
}
