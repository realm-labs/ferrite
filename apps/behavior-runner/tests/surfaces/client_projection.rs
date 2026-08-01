use ferrite_testkit::phase9::surfaces::run_client_projection_surface;

#[test]
fn client_projection_closes_inventory_prediction_menu_and_lifecycle_boundaries() {
    let report = run_client_projection_surface();
    assert_eq!(report.clientbound_packets, 169);
    assert_eq!(report.prediction_cases, 2);
    assert_eq!(report.menu_cases, 3);
    assert_eq!(report.lifecycle_cases, 3);
}
