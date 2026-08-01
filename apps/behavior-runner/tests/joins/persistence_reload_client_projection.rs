use ferrite_testkit::phase9::joins::{JoinOracle, run_persistence_reload_client_projection};

#[test]
fn first_projection_is_reconstructed_from_durable_authority() {
    let report = run_persistence_reload_client_projection();
    assert_eq!(report.oracle, JoinOracle::SaveThenReconstruct);
    assert!(!report.transient_state_persisted);
}
