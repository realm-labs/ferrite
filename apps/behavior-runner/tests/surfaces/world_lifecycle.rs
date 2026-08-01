use ferrite_testkit::phase8::surfaces::run_world_lifecycle_surface;

#[test]
fn world_lifecycle_is_ordered_bounded_and_failure_continuing() {
    let report = run_world_lifecycle_surface();
    assert_eq!(
        report.golden_digest,
        "22081fb59b8f0695261ccb2d0acccbf663f6226ecbf49068fe14e3fa31fae7be"
    );
    assert_eq!(report.property_cases, 64);
    assert_eq!(report.dimensions, 3);
    assert_eq!(report.bootstrap_events, 9);
    assert_eq!(report.shutdown_events, 18);
}
