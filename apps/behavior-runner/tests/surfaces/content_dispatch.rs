use ferrite_testkit::phase8::surfaces::run_content_dispatch_surface;

#[test]
fn content_dispatch_is_catalog_complete_deterministic_and_manifest_fenced() {
    let report = run_content_dispatch_surface();
    assert_eq!(report.catalog_records, 963);
    assert_eq!(report.catalog_families, 14);
    assert_eq!(report.deterministic_cases, 32);
    assert_eq!(report.manifest_fences, 2);
}
