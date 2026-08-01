use ferrite_testkit::phase9::joins::{JoinOracle, run_world_lifecycle_data_reload};

#[test]
fn world_consumers_revalidate_after_reload_publication() {
    let report = run_world_lifecycle_data_reload();
    assert_eq!(report.oracle, JoinOracle::CaptureThenResolve);
    assert_eq!(report.checkpoints, 3);
}
