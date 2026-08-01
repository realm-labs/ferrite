use ferrite_testkit::phase9::joins::{JoinOracle, run_persistence_reload_data_reload};

#[test]
fn saved_pack_selection_reconstructs_reload_snapshots() {
    let report = run_persistence_reload_data_reload();
    assert_eq!(report.oracle, JoinOracle::SaveThenReconstruct);
    assert_eq!(report.checkpoints, 3);
}
