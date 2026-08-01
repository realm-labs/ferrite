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
        "0a39a6f5f44882457b4818c8aebb7242e787e347e392484be55e44abbbe6b5ee"
    );
    assert_eq!(report.catalog_records, 963);
    assert_eq!(report.catalog_families, 14);
    assert_eq!(report.generation_cases, 64);
    assert_eq!(report.generation_statuses, 12);
    assert_eq!(report.boundary_cases, 8);
    assert_eq!(report.save_load_cases, 16);
    assert_eq!(report.crash_cases, 6);
}
