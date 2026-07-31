use ferrite_testkit::phase8::joins::run_network_ingress_persistence_reload;

#[test]
fn ingress_targets_recovered_region_state() {
    let report = run_network_ingress_persistence_reload();
    assert_eq!(report.checkpoints, 2);
    assert_eq!(report.rejected_faults, 0);
}
