use ferrite_testkit::phase8::joins::run_world_lifecycle_persistence_reload;

#[test]
fn level_global_state_survives_region_recovery() {
    let report = run_world_lifecycle_persistence_reload();
    assert_eq!(report.checkpoints, 5);
    assert_eq!(report.rejected_faults, 0);
}
