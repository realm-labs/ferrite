use ferrite_testkit::player_service::player_lifecycle_surface::run_player_lifecycle_surface;

#[test]
fn player_lifecycle_surface_is_golden_property_fuzz_replay_and_client_trace_locked() {
    let report = run_player_lifecycle_surface();
    assert_eq!(
        report.golden_digest,
        "f6124ba1095b689b2e41e81f63ec6521a59007ce4f5bdf6415536b76ab324ea4"
    );
    assert_eq!(report.property_cases, 128);
    assert_eq!(report.fuzz_cases, 256);
    assert_eq!(report.fault_cases, 8);
    assert_eq!(report.replay_frames, 6);
    assert_eq!(report.client_trace_events, 121);
}
