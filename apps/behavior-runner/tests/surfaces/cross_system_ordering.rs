use ferrite_testkit::phase9::surfaces::run_cross_system_ordering_surface;

#[test]
fn cross_system_ordering_closes_every_remaining_phase9_join() {
    let report = run_cross_system_ordering_surface();
    assert_eq!(report.joins, 21);
    assert_eq!(report.checkpoints, 64);
    assert_eq!(report.digest.len(), 64);
}
