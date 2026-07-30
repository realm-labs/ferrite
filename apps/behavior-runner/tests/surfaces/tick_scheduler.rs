use ferrite_testkit::phase5::scheduler_surface::run_tick_scheduler_surface;

#[test]
fn tick_scheduler_surface_is_golden_bounded_replayable_and_region_equivalent() {
    let report = run_tick_scheduler_surface();
    assert_eq!(
        report.golden_digest,
        "eadb63bd70aec010d8d5854b817ec04172320a823e419093e744c08a579cc501"
    );
    assert_eq!(report.property_cases, 128);
    assert_eq!(report.fault_cases, 4);
    assert_eq!(report.boundary_cases, 5);
    assert_eq!(report.replay_frames, 4);
}
