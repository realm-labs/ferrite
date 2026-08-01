use ferrite_testkit::phase9::joins::{JoinOracle, run_content_dispatch_client_projection};

#[test]
fn content_owned_effects_project_only_after_commit() {
    let report = run_content_dispatch_client_projection();
    assert_eq!(report.oracle, JoinOracle::CommitThenProject);
    assert_eq!(report.checkpoints, 3);
}
