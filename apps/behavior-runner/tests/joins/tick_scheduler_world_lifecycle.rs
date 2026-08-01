use ferrite_testkit::world_service::joins::run_tick_scheduler_world_lifecycle;

#[test]
fn world_readiness_waits_for_scheduler_work() {
    let report = run_tick_scheduler_world_lifecycle();
    assert_eq!(report.checkpoints, 2);
    assert_eq!(report.rejected_faults, 0);
}
