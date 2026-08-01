use ferrite_testkit::phase9::joins::{JoinOracle, run_client_projection_data_reload};

#[test]
fn reload_publication_converges_active_client_projection() {
    let report = run_client_projection_data_reload();
    assert_eq!(report.oracle, JoinOracle::PublishThenConverge);
    assert!(!report.transient_state_persisted);
}
