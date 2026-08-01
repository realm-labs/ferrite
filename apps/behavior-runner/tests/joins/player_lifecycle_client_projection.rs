use ferrite_testkit::phase9::joins::{JoinOracle, run_player_lifecycle_client_projection};

#[test]
fn player_replacement_resets_then_reprojects_transient_state() {
    let report = run_player_lifecycle_client_projection();
    assert_eq!(report.oracle, JoinOracle::LifecycleThenProject);
    assert!(!report.transient_state_persisted);
}
