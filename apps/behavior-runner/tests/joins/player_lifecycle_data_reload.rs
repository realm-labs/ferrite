use ferrite_testkit::service_conformance::joins::{JoinOracle, run_player_lifecycle_data_reload};

#[test]
fn active_and_joining_players_converge_to_published_resources() {
    let report = run_player_lifecycle_data_reload();
    assert_eq!(report.oracle, JoinOracle::PublishThenConverge);
    assert_eq!(report.checkpoints, 3);
}
