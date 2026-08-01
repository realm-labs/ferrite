use ferrite_testkit::phase9::joins::{JoinOracle, run_command_administration_client_projection};

#[test]
fn command_commit_precedes_projection_and_feedback() {
    let report = run_command_administration_client_projection();
    assert_eq!(report.oracle, JoinOracle::CommitThenProject);
    assert_eq!(report.checkpoints, 3);
}
