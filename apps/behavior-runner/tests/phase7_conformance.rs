use ferrite_testkit::phase7::entity_conformance::run_entity_conformance;

#[test]
fn entity_conformance_is_golden_property_fuzz_fault_replay_and_client_trace_locked() {
    let report = run_entity_conformance();
    assert_eq!(
        report.golden_digest,
        "28deb222fdc6efac437eb4b79944dd8ebcbb7467025b2431db7c25f5e82cbaaa"
    );
    assert_eq!(report.property_cases, 128);
    assert_eq!(report.fuzz_cases, 256);
    assert_eq!(report.fault_cases, 10);
    assert_eq!(report.transfer_cases, 64);
    assert_eq!(report.replay_frames, 8);
    assert_eq!(report.client_trace_events, 10);
}
