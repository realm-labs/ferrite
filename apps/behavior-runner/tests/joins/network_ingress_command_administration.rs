use ferrite_testkit::phase9::joins::{JoinOracle, run_network_ingress_command_administration};

#[test]
fn packet_admission_precedes_serialized_command_execution() {
    let report = run_network_ingress_command_administration();
    assert_eq!(report.oracle, JoinOracle::QueueThenExecute);
    assert_eq!(report.checkpoints, 3);
}
