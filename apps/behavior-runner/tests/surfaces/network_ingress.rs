use ferrite_testkit::service_conformance::surfaces::run_network_ingress_surface;

#[test]
fn network_ingress_closes_all_serverbound_and_terminal_transition_boundaries() {
    let report = run_network_ingress_surface();
    assert_eq!(report.serverbound_packets, 87);
    assert_eq!(report.prediction_boundaries, 2);
    assert_eq!(report.reconfiguration_stages, 3);
}
