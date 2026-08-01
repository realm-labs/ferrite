use ferrite_testkit::player_service::ingress_player_lifecycle_join::run_network_ingress_player_lifecycle;

#[test]
fn ingress_transitions_route_exactly_one_join_and_one_leave() {
    let report = run_network_ingress_player_lifecycle();
    assert_eq!(report.property_cases, 64);
    assert_eq!(report.fault_cases, 5);
    assert_eq!(report.join_effects, 26);
    assert_eq!(report.leave_effects, 20);
    assert_eq!(report.routed_commands, 2);
}
