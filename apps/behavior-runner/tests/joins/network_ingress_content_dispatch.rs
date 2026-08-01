use ferrite_testkit::phase8::joins::run_network_ingress_content_dispatch;

#[test]
fn ingress_targets_the_content_ready_region() {
    let report = run_network_ingress_content_dispatch();
    assert_eq!(report.checkpoints, 2);
    assert_eq!(report.rejected_faults, 0);
}
