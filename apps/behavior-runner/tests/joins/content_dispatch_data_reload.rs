use ferrite_testkit::phase9::joins::{JoinOracle, run_content_dispatch_data_reload};

#[test]
fn captured_content_and_live_bindings_cross_reload_explicitly() {
    let report = run_content_dispatch_data_reload();
    assert_eq!(report.oracle, JoinOracle::CaptureThenResolve);
    assert_eq!(report.checkpoints, 4);
}
