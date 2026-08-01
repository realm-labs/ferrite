use ferrite_testkit::phase8::joins::run_player_lifecycle_world_lifecycle;

#[test]
fn world_shutdown_saves_players_before_removal() {
    let report = run_player_lifecycle_world_lifecycle();
    assert_eq!(report.checkpoints, 2);
    assert_eq!(report.rejected_faults, 0);
}
