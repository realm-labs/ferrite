use ferrite_testkit::service_conformance::joins::{JoinOracle, run_tick_scheduler_data_reload};

#[test]
fn in_flight_ticks_keep_their_snapshot_until_reload_publication() {
    let report = run_tick_scheduler_data_reload();
    assert_eq!(report.oracle, JoinOracle::CaptureThenResolve);
    assert_eq!(report.checkpoints, 4);
}
