use ferrite_testkit::phase9::joins::{JoinOracle, run_network_ingress_data_reload};

#[test]
fn captured_listener_boundary_survives_reload_publication() {
    let report = run_network_ingress_data_reload();
    assert_eq!(report.oracle, JoinOracle::CaptureThenResolve);
    assert_eq!(report.checkpoints, 3);
}
