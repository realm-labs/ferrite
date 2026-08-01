use ferrite_testkit::world_service::joins::run_tick_scheduler_content_dispatch;

#[test]
fn persisted_ticks_unpack_before_generated_chunk_ticking() {
    let report = run_tick_scheduler_content_dispatch();
    assert_eq!(report.checkpoints, 2);
    assert_eq!(report.rejected_faults, 0);
}
