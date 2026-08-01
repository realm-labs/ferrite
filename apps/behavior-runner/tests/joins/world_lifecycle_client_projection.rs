use ferrite_testkit::service_conformance::joins::{
    JoinOracle, run_world_lifecycle_client_projection,
};

#[test]
fn world_lifecycle_commit_precedes_load_or_unload_projection() {
    let report = run_world_lifecycle_client_projection();
    assert_eq!(report.oracle, JoinOracle::LifecycleThenProject);
    assert_eq!(report.checkpoints, 2);
}
