use ferrite_testkit::phase8::joins::run_content_dispatch_player_lifecycle;

#[test]
fn players_join_only_after_content_reaches_entity_ticking() {
    let report = run_content_dispatch_player_lifecycle();
    assert_eq!(report.checkpoints, 2);
    assert_eq!(report.rejected_faults, 1);
}
