use ferrite_testkit::service_conformance::joins::{
    JoinOracle, run_command_administration_content_dispatch,
};

#[test]
fn command_arguments_are_captured_before_live_content_resolution() {
    let report = run_command_administration_content_dispatch();
    assert_eq!(report.oracle, JoinOracle::CaptureThenResolve);
    assert_eq!(report.checkpoints, 4);
}
