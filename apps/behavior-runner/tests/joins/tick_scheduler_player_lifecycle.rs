use ferrite_testkit::phase6::scheduler_player_lifecycle_join::run_tick_scheduler_player_lifecycle;

#[test]
fn lifecycle_membership_changes_only_at_the_scheduler_capture_boundary() {
    let report = run_tick_scheduler_player_lifecycle();
    assert_eq!(report.property_cases, 64);
    assert_eq!(report.fault_cases, 5);
    assert!(report.visible_after_join_capture);
    assert!(report.visible_before_leave_capture);
    assert!(!report.visible_after_leave_capture);
    assert_eq!(report.same_tick_final_membership, 0);
}
