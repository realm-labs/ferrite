use ferrite_testkit::world_service::fixtures::bundle_available;
use ferrite_testkit::world_service::surfaces::run_content_dispatch_surface;

#[test]
fn content_dispatch_is_catalog_complete_deterministic_and_manifest_fenced() {
    if !bundle_available() {
        eprintln!(
            "locked local artifact bundle is absent; `cargo ferrite content verify` owns that gate"
        );
        return;
    }
    let report = run_content_dispatch_surface();
    assert_eq!(report.catalog_records, 963);
    assert_eq!(report.catalog_families, 14);
    assert_eq!(report.deterministic_cases, 32);
    assert_eq!(report.manifest_fences, 2);
}
