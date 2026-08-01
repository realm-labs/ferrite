use ferrite_testkit::world_service::surfaces::run_persistence_reload_surface;

#[test]
fn persistence_reload_restores_world_player_and_scheduler_continuity() {
    let report = run_persistence_reload_surface();
    assert_eq!(report.chunk_records, 1);
    assert!(report.auxiliary_records >= 2);
    assert_eq!(report.restored_players, 1);
    assert_eq!(report.restored_schedulers, 1);
}
