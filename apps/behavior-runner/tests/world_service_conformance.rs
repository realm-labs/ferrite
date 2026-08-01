use ferrite_testkit::world_service::conformance::run_world_conformance;
use ferrite_testkit::world_service::fixtures::bundle_available;

#[test]
fn world_conformance_locks_catalog_generation_boundaries_and_recovery() {
    if !bundle_available() {
        eprintln!(
            "locked local artifact bundle is absent; `cargo ferrite content verify` owns that gate"
        );
        return;
    }
    let report = run_world_conformance();
    assert_eq!(
        report.golden_digest,
        "19140a5608d4549ca22a1895d8f83ecfa96cfd216eae22de83089e7829e33fd1"
    );
    assert_eq!(report.catalog_records, 963);
    assert_eq!(report.catalog_families, 14);
    assert_eq!(report.generation_cases, 64);
    assert_eq!(report.generation_statuses, 12);
    assert_eq!(report.boundary_cases, 8);
    assert_eq!(report.save_load_cases, 16);
    assert_eq!(report.crash_cases, 6);
}
