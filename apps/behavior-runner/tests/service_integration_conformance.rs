use ferrite_testkit::service_conformance::conformance::run_service_protocol_audit;
use ferrite_testkit::service_conformance::surfaces::run_cross_system_ordering_surface;

#[test]
fn service_conformance_closes_packet_family_gate_surface_and_join_denominators() {
    let protocol = run_service_protocol_audit();
    assert_eq!(protocol.packets, 256);
    assert_eq!(protocol.required_families, 44);
    assert_eq!(protocol.optional_families, 14);
    assert_eq!(protocol.verified_families, 58);
    assert_eq!(protocol.default_closed_gate_types, 14);
    assert_eq!(protocol.test_owners_present, 58);

    let ordering = run_cross_system_ordering_surface();
    assert_eq!(ordering.joins, 21);
    assert_eq!(ordering.checkpoints, 64);
}
