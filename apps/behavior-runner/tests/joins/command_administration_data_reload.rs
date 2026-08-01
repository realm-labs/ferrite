use ferrite_testkit::service_conformance::joins::{
    JoinOracle, run_command_administration_data_reload,
};

#[test]
fn reload_command_completes_after_ordered_publication() {
    let report = run_command_administration_data_reload();
    assert_eq!(report.oracle, JoinOracle::PublishThenConverge);
    assert_eq!(report.checkpoints, 4);
}
