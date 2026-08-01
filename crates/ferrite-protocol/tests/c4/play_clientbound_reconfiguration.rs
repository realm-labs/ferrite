use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::reconfiguration::gate::{
    ReconfigurationContext, ReconfigurationDecision, ReconfigurationEffect, ReconfigurationGates,
};
use ferrite_protocol::java_26_2::play::clientbound::reconfiguration::packet::ReconfigurationPacketKind;
use ferrite_protocol::java_26_2::play::clientbound::reconfiguration::transition::{
    CarriedReconfigurationState, ClientReconfigurationEffect, ClientReconfigurationError,
    ClientReconfigurationStage, ClientReconfigurationStep, ClientReconfigurationTransition,
    ServerReconfigurationStep,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::wire::compression::{CompressionMode, encode_packet};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;

fn context() -> ReconfigurationContext {
    ReconfigurationContext {
        service_registered: true,
        administrator: true,
        play_removal_committed: true,
    }
}

#[test]
fn c4_reconfiguration_inventory_and_fieldless_golden_lock_terminal_id() {
    assert_eq!(
        ReconfigurationPacketKind::ALL,
        [ReconfigurationPacketKind::StartConfiguration]
    );
    let packet = ReconfigurationPacketKind::StartConfiguration;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Play,
        PacketDirection::Clientbound,
        packet.wire_id(),
    )
    .unwrap();
    assert_eq!(descriptor.identity(), packet.identity());
    assert_eq!(
        encode_packet(
            &[u8::try_from(packet.wire_id()).unwrap()],
            CompressionMode::enabled(256).unwrap(),
            FrameLimits::default(),
        )
        .unwrap(),
        [2, 0, 118]
    );
}

#[test]
fn c4_reconfiguration_default_gate_omits_terminal_output() {
    assert_eq!(
        ReconfigurationGates::default().decide(context()),
        ReconfigurationDecision::OmitDisabled(ReconfigurationPacketKind::StartConfiguration)
    );
}

#[test]
fn c4_reconfiguration_gate_refuses_unauthorized_and_unavailable_requests() {
    let gates = ReconfigurationGates {
        reconfiguration: true,
    };
    assert_eq!(
        gates.decide(ReconfigurationContext {
            administrator: false,
            ..context()
        }),
        ReconfigurationDecision::RefuseUnauthorized
    );
    assert_eq!(
        gates.decide(ReconfigurationContext {
            service_registered: false,
            ..context()
        }),
        ReconfigurationDecision::DegradeServiceUnavailable
    );
    assert_eq!(
        gates.decide(ReconfigurationContext {
            play_removal_committed: false,
            ..context()
        }),
        ReconfigurationDecision::RefuseBeforeCommittedPlayRemoval
    );
    assert_eq!(
        gates.decide(context()),
        ReconfigurationDecision::Emit(
            ReconfigurationEffect::BeginTerminalPlayToConfigurationTransition
        )
    );
}

#[test]
fn c4_reconfiguration_server_terminal_order_sends_before_outbound_switch() {
    assert_eq!(
        ServerReconfigurationStep::ORDER,
        [
            ServerReconfigurationStep::SetWaitingForAcknowledgement,
            ServerReconfigurationStep::SaveAndRemovePlayerFromPlay,
            ServerReconfigurationStep::SendTerminalStartConfiguration,
            ServerReconfigurationStep::InstallConfigurationOutbound,
        ]
    );
}

#[test]
fn c4_reconfiguration_client_switches_inbound_before_ack_and_outbound_after() {
    assert_eq!(
        ClientReconfigurationStep::ORDER,
        [
            ClientReconfigurationStep::FlushDelayedChat,
            ClientReconfigurationStep::SendPendingLastSeenAcknowledgement,
            ClientReconfigurationStep::StoreChatAndCommonState,
            ClientReconfigurationStep::ClearLevelAndShowReconfigurationScreen,
            ClientReconfigurationStep::CreateConfigurationListenerWithFreshLoadTracker,
            ClientReconfigurationStep::InstallConfigurationInbound,
            ClientReconfigurationStep::SendTerminalPlayAcknowledgement,
            ClientReconfigurationStep::InstallConfigurationOutbound,
        ]
    );
}

#[test]
fn c4_reconfiguration_carries_common_state_but_recreates_projection() {
    assert_eq!(CarriedReconfigurationState::ALL.len(), 13);
    assert!(CarriedReconfigurationState::ALL.contains(&CarriedReconfigurationState::Cookies));
    assert!(CarriedReconfigurationState::ALL.contains(&CarriedReconfigurationState::ChatState));
    assert!(CarriedReconfigurationState::ALL.contains(&CarriedReconfigurationState::Registries));

    let mut transition = ClientReconfigurationTransition::new();
    assert_eq!(transition.stage(), ClientReconfigurationStage::Play);
    assert_eq!(
        transition.handle_start().unwrap(),
        ClientReconfigurationEffect::ExecuteTerminalPlan
    );
    assert_eq!(
        transition.stage(),
        ClientReconfigurationStage::Configuration
    );
    assert_eq!(
        transition.handle_start(),
        Err(ClientReconfigurationError::StartOutsidePlay)
    );
    assert_eq!(
        transition.finish_configuration().unwrap(),
        ClientReconfigurationEffect::CreateFreshPlayProjection
    );
    assert_eq!(transition.stage(), ClientReconfigurationStage::Play);
    assert_eq!(
        transition.finish_configuration(),
        Err(ClientReconfigurationError::FinishOutsideConfiguration)
    );
}

#[test]
fn c4_reconfiguration_required_decoder_remains_fail_closed() {
    let registries = PlayRegistries::default();
    let values = RejectComponentValues;
    assert!(matches!(
        decode_required_packet(
            &[118],
            PlayDecodeContext {
                registries: &registries,
                component_values: &values,
                dimension_section_count: 24,
            },
        ),
        Err(PlayClientboundCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:start_configuration"
        })
    ));
}
