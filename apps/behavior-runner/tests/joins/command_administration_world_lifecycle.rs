use ferrite_testkit::phase9::joins::{JoinOracle, run_command_administration_world_lifecycle};

#[test]
fn command_chunk_admission_precedes_world_publication() {
    let report = run_command_administration_world_lifecycle();
    assert_eq!(report.oracle, JoinOracle::CaptureThenResolve);
    assert_eq!(report.checkpoints, 3);
}
