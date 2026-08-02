use ferrite_testkit::world_service::equivalence::run_worldgen_equivalence_boundary;

#[test]
fn historical_experiments_remain_diagnostic_under_the_exactness_contract() {
    let report = run_worldgen_equivalence_boundary();
    assert_eq!(report.experiments, 3);
    assert_eq!(report.planned_repeats, 8_200);
    assert_eq!(report.source_specified_slices, 27);
    assert_eq!(report.source_inconclusive_slices, 1);
    assert!(report.project_seed_is_deterministic);
    assert!(report.distinct_project_seeds_diverge);
    assert!(report.same_seed_semantic_identity_required);
    assert!(report.differential_oracle_implemented);
    assert!(!report.declared_population_verified);
    assert!(!report.statistical_thresholds_can_close_compatibility);
}
