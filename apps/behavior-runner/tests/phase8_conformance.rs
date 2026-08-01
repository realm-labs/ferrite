use ferrite_testkit::phase8::conformance::run_world_conformance;
use ferrite_testkit::phase8::fixtures::bundle_available;

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
        "24cbf316b4c931022deb21cc24282f74abfd693ac181666c1bfefa05c5571f95"
    );
    assert_eq!(report.catalog_records, 963);
    assert_eq!(report.catalog_families, 14);
    assert_eq!(report.generation_cases, 64);
    assert_eq!(report.generation_statuses, 12);
    assert_eq!(report.boundary_cases, 8);
    assert_eq!(report.save_load_cases, 16);
    assert_eq!(report.crash_cases, 6);
}
