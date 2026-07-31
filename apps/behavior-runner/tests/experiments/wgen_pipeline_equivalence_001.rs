use ferrite_testkit::phase8::equivalence::run_worldgen_equivalence_boundary;

#[test]
fn source_control_flow_is_verified_while_statistical_equivalence_stays_deferred() {
    let report = run_worldgen_equivalence_boundary();
    assert_eq!(report.experiments, 3);
    assert_eq!(report.planned_repeats, 8_200);
    assert_eq!(report.source_specified_slices, 27);
    assert_eq!(report.source_inconclusive_slices, 1);
    assert!(report.project_seed_is_deterministic);
    assert!(report.distinct_project_seeds_diverge);
    assert!(!report.same_seed_vanilla_identity_claimed);
    assert!(!report.statistical_thresholds_committed);
}
