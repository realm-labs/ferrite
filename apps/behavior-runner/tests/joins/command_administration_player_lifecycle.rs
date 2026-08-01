use ferrite_testkit::phase9::joins::{JoinOracle, run_command_administration_player_lifecycle};

#[test]
fn command_targets_respect_player_replacement_boundaries() {
    let report = run_command_administration_player_lifecycle();
    assert_eq!(report.oracle, JoinOracle::CaptureThenResolve);
    assert_eq!(report.checkpoints, 3);
}
