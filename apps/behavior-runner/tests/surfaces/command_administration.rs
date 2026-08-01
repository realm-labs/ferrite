use ferrite_testkit::service_conformance::surfaces::run_command_administration_surface;

#[test]
fn command_administration_requires_typed_permissions_and_ordered_effects() {
    let report = run_command_administration_surface();
    assert_eq!(report.admitted_admin_cases, 1);
    assert_eq!(report.admitted_operator_cases, 1);
    assert_eq!(report.permission_factors, 2);
}
