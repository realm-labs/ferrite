use ferrite_testkit::phase8::joins::run_content_dispatch_persistence_reload;

#[test]
fn content_identity_is_revalidated_at_persistence_reload() {
    let report = run_content_dispatch_persistence_reload();
    assert_eq!(report.checkpoints, 2);
    assert_eq!(report.rejected_faults, 1);
}
