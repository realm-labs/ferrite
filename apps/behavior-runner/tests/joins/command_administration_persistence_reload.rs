use ferrite_testkit::service_conformance::joins::{
    JoinOracle, run_command_administration_persistence_reload,
};

#[test]
fn only_committed_command_prefixes_are_reconstructed() {
    let report = run_command_administration_persistence_reload();
    assert_eq!(report.oracle, JoinOracle::CommitThenSave);
    assert!(!report.transient_state_persisted);
}
