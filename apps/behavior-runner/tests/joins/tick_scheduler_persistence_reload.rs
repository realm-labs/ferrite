use ferrite_testkit::world_service::joins::run_tick_scheduler_persistence_reload;

#[test]
fn scheduler_continuity_survives_world_reload() {
    let report = run_tick_scheduler_persistence_reload();
    assert!(report.checkpoints >= 2);
    assert_eq!(report.rejected_faults, 0);
}
