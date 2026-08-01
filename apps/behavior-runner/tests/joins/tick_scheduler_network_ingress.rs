use ferrite_testkit::simulation::scheduler_ingress_join::run_tick_scheduler_network_ingress;

#[test]
fn ingress_visibility_is_fixed_by_the_scheduler_capture_boundary() {
    let report = run_tick_scheduler_network_ingress();
    assert_eq!(report.ingress_before_capture.get(), 6);
    assert_eq!(report.ingress_after_capture.get(), 5);
    assert_eq!(report.property_cases, 64);
    assert_eq!(report.fault_cases, 4);
    assert_eq!(
        report
            .callback_ticks
            .into_iter()
            .map(|tick| tick.get())
            .collect::<Vec<_>>(),
        [1, 2]
    );
}
