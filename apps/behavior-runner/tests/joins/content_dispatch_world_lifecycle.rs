use ferrite_testkit::world_service::joins::run_content_dispatch_world_lifecycle;

#[test]
fn world_readiness_and_generation_share_the_locked_content_identity() {
    let report = run_content_dispatch_world_lifecycle();
    assert_eq!(report.checkpoints, 3);
    assert_eq!(report.rejected_faults, 0);
}
