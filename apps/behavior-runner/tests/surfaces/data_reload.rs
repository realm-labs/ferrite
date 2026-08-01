use ferrite_testkit::phase9::surfaces::run_data_reload_surface;

#[test]
fn data_reload_preserves_failure_isolation_and_publication_order() {
    let report = run_data_reload_surface();
    assert_eq!(report.publication_steps, 3);
    assert_eq!(report.failure_prefixes, 1);
    assert_eq!(report.retained_cookie_fields, 4);
}
