use ferrite_testkit::service_conformance::joins::{
    JoinOracle, run_network_ingress_client_projection,
};

#[test]
fn ingress_commit_precedes_acknowledgement_or_correction() {
    let report = run_network_ingress_client_projection();
    assert_eq!(report.oracle, JoinOracle::CommitThenProject);
    assert_eq!(report.checkpoints, 3);
}
