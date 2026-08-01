//! Manifest, packet-inventory, optional-gate, and transition closure for Phase 9.

use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use ferrite_protocol::java_26_2::catalog::PacketCatalog;
use ferrite_protocol::java_26_2::configuration::clientbound::optional::ConfigurationClientboundGates;
use ferrite_protocol::java_26_2::configuration::serverbound::optional::ConfigurationServerboundGates;
use ferrite_protocol::java_26_2::login::clientbound::optional::LoginClientboundGates;
use ferrite_protocol::java_26_2::login::serverbound::optional::LoginServerboundGates;
use ferrite_protocol::java_26_2::play::clientbound::admin_presentation::gate::AdminPresentationGates;
use ferrite_protocol::java_26_2::play::clientbound::common_services::gate::CommonServiceGates;
use ferrite_protocol::java_26_2::play::clientbound::debug_projection::gate::DebugProjectionGates;
use ferrite_protocol::java_26_2::play::clientbound::live_tags::gate::LiveTagsGates;
use ferrite_protocol::java_26_2::play::clientbound::reconfiguration::gate::ReconfigurationGates;
use ferrite_protocol::java_26_2::play::serverbound::admin_state::gate::AdminStateGates;
use ferrite_protocol::java_26_2::play::serverbound::common_services::gate::PlayCommonServerboundGates;
use ferrite_protocol::java_26_2::play::serverbound::debug_subscription::gate::DebugSubscriptionRequestGates;
use ferrite_protocol::java_26_2::play::serverbound::operator_blocks::gate::OperatorBlockGates;
use ferrite_protocol::java_26_2::play::serverbound::reconfiguration::gate::ServerboundReconfigurationGates;
use toml::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phase9ProtocolAuditReport {
    pub packets: usize,
    pub required_families: usize,
    pub optional_families: usize,
    pub verified_families: usize,
    pub default_closed_gate_types: usize,
    pub test_owners_present: usize,
}

#[must_use]
pub fn run_phase9_protocol_audit() -> Phase9ProtocolAuditReport {
    let implementation = parse("../../goals/minecraft-java-26.2/implementation.toml");
    let batches = implementation
        .get("protocol_batch")
        .and_then(Value::as_array)
        .expect("implementation manifest contains protocol batches");
    let required_families = batches
        .iter()
        .filter(|record| string(record, "source_responsibility") == "Required")
        .count();
    let optional = batches
        .iter()
        .filter(|record| string(record, "implementation_mode") == "ConfigurationGate")
        .collect::<Vec<_>>();
    let verified_families = batches
        .iter()
        .filter(|record| string(record, "disposition") == "Verified")
        .count();
    assert_eq!(PacketCatalog::all().len(), 256);
    assert_eq!(required_families, 44);
    assert_eq!(optional.len(), 14);
    assert_eq!(verified_families, 58);

    let workspace = workspace_root();
    for record in batches {
        assert_eq!(string(record, "disposition"), "Verified");
        let test_owner = string(record, "test_owner");
        assert!(workspace.join(test_owner).is_file(), "missing {test_owner}");
    }

    assert_default_closed(ConfigurationClientboundGates::default());
    assert_default_closed(ConfigurationServerboundGates::default());
    assert_default_closed(LoginClientboundGates::default());
    assert_default_closed(LoginServerboundGates::default());
    assert_default_closed(AdminPresentationGates::default());
    assert_default_closed(CommonServiceGates::default());
    assert_default_closed(DebugProjectionGates::default());
    assert_default_closed(LiveTagsGates::default());
    assert_default_closed(ReconfigurationGates::default());
    assert_default_closed(AdminStateGates::default());
    assert_default_closed(PlayCommonServerboundGates::default());
    assert_default_closed(DebugSubscriptionRequestGates::default());
    assert_default_closed(OperatorBlockGates::default());
    assert_default_closed(ServerboundReconfigurationGates::default());

    Phase9ProtocolAuditReport {
        packets: PacketCatalog::all().len(),
        required_families,
        optional_families: optional.len(),
        verified_families,
        default_closed_gate_types: 14,
        test_owners_present: batches.len(),
    }
}

fn assert_default_closed(gate: impl Debug) {
    let representation = format!("{gate:?}");
    assert!(
        !representation.contains("true"),
        "optional gate is enabled by default: {representation}"
    );
}

fn parse(relative: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
        .parse()
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn string<'a>(record: &'a Value, field: &str) -> &'a str {
    record
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("record field {field} is not a string"))
}
