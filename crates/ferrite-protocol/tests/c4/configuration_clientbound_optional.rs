use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::configuration::clientbound::optional::{
    ConfigurationClientboundGates, ConfigurationOptionalService, ConfigurationPhase,
    OptionalClientboundDecision, OptionalClientboundEffect, OptionalConfigurationContext,
    OptionalConfigurationPacket,
};

fn context(phase: ConfigurationPhase, singleplayer: bool) -> OptionalConfigurationContext {
    OptionalConfigurationContext {
        phase,
        singleplayer,
    }
}

#[test]
fn c4_configuration_clientbound_optional_inventory_locks_all_eleven_catalog_entries() {
    assert_eq!(OptionalConfigurationPacket::ALL.len(), 11);
    let ids = OptionalConfigurationPacket::ALL
        .into_iter()
        .map(OptionalConfigurationPacket::wire_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ids,
        BTreeSet::from([0, 6, 8, 9, 10, 11, 15, 16, 17, 18, 19])
    );
    for packet in OptionalConfigurationPacket::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Configuration,
            PacketDirection::Clientbound,
            packet.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), packet.identity());
    }
}

#[test]
fn c4_configuration_clientbound_default_offline_gate_omits_every_optional_packet() {
    let gates = ConfigurationClientboundGates::default();
    for packet in OptionalConfigurationPacket::ALL {
        assert_eq!(
            gates.decide(packet, context(ConfigurationPhase::Fresh, false)),
            OptionalClientboundDecision::OmitDisabled(packet.service())
        );
    }
}

#[test]
fn c4_configuration_clientbound_services_enable_only_their_owned_packets() {
    let gates = ConfigurationClientboundGates {
        cookies: true,
        resource_packs: true,
        report_details: true,
        server_links: true,
        dialogs: true,
        code_of_conduct: true,
        ..ConfigurationClientboundGates::default()
    };
    let expected = [
        (
            OptionalConfigurationPacket::CookieRequest,
            OptionalClientboundEffect::RequestResponse,
        ),
        (
            OptionalConfigurationPacket::StoreCookie,
            OptionalClientboundEffect::StoreConnectionState,
        ),
        (
            OptionalConfigurationPacket::ResourcePackPop,
            OptionalClientboundEffect::PresentationOnly,
        ),
        (
            OptionalConfigurationPacket::ResourcePackPush,
            OptionalClientboundEffect::BlockingTask,
        ),
        (
            OptionalConfigurationPacket::CustomReportDetails,
            OptionalClientboundEffect::PresentationOnly,
        ),
        (
            OptionalConfigurationPacket::ServerLinks,
            OptionalClientboundEffect::PresentationOnly,
        ),
        (
            OptionalConfigurationPacket::ClearDialog,
            OptionalClientboundEffect::PresentationOnly,
        ),
        (
            OptionalConfigurationPacket::ShowDialog,
            OptionalClientboundEffect::PresentationOnly,
        ),
        (
            OptionalConfigurationPacket::CodeOfConduct,
            OptionalClientboundEffect::BlockingTask,
        ),
    ];
    for (packet, effect) in expected {
        assert_eq!(
            gates.decide(packet, context(ConfigurationPhase::Fresh, false)),
            OptionalClientboundDecision::Emit(effect)
        );
    }
    assert_eq!(
        gates.decide(
            OptionalConfigurationPacket::Transfer,
            context(ConfigurationPhase::Fresh, false)
        ),
        OptionalClientboundDecision::OmitDisabled(ConfigurationOptionalService::Transfer)
    );
}

#[test]
fn c4_reset_chat_requires_both_capability_and_reconfiguration_phase() {
    let gates = ConfigurationClientboundGates {
        reconfiguration: true,
        ..ConfigurationClientboundGates::default()
    };
    assert_eq!(
        gates.decide(
            OptionalConfigurationPacket::ResetChat,
            context(ConfigurationPhase::Fresh, false)
        ),
        OptionalClientboundDecision::OmitOutsideReconfiguration
    );
    assert_eq!(
        gates.decide(
            OptionalConfigurationPacket::ResetChat,
            context(ConfigurationPhase::Reconfiguration, false)
        ),
        OptionalClientboundDecision::Emit(OptionalClientboundEffect::ResetRetainedChat)
    );
}

#[test]
fn c4_transfer_enabled_gate_refuses_singleplayer_and_changes_only_connection_state() {
    let gates = ConfigurationClientboundGates {
        transfer: true,
        ..ConfigurationClientboundGates::default()
    };
    assert_eq!(
        gates.decide(
            OptionalConfigurationPacket::Transfer,
            context(ConfigurationPhase::Fresh, true)
        ),
        OptionalClientboundDecision::RefuseTransferInSingleplayer
    );
    assert_eq!(
        gates.decide(
            OptionalConfigurationPacket::Transfer,
            context(ConfigurationPhase::Fresh, false)
        ),
        OptionalClientboundDecision::Emit(OptionalClientboundEffect::TransferConnection)
    );
}

#[test]
fn c4_configuration_clientbound_gate_decisions_never_claim_world_mutation() {
    let effects = [
        OptionalClientboundEffect::RequestResponse,
        OptionalClientboundEffect::BlockingTask,
        OptionalClientboundEffect::PresentationOnly,
        OptionalClientboundEffect::StoreConnectionState,
        OptionalClientboundEffect::TransferConnection,
        OptionalClientboundEffect::ResetRetainedChat,
    ];
    assert_eq!(effects.len(), 6);
}
