use ferrite_testkit::phase8::joins::run_network_ingress_world_lifecycle;

#[test]
fn ingress_admission_follows_world_readiness() {
    let report = run_network_ingress_world_lifecycle();
    assert_eq!(report.checkpoints, 3);
    assert_eq!(report.rejected_faults, 0);
}
