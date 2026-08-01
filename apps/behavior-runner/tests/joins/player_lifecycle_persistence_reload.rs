use ferrite_testkit::world_service::joins::run_player_lifecycle_persistence_reload;

#[test]
fn player_continuity_survives_world_reload() {
    let report = run_player_lifecycle_persistence_reload();
    assert!(report.checkpoints >= 3);
    assert_eq!(report.rejected_faults, 0);
}
