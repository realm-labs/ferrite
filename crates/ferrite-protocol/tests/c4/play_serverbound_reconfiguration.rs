use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::clientbound::reconfiguration::transition::{
    ClientReconfigurationStep, ServerReconfigurationStep,
};
use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::serverbound::reconfiguration::gate::{
    ServerboundReconfigurationContext, ServerboundReconfigurationDecision,
    ServerboundReconfigurationGates,
};
use ferrite_protocol::java_26_2::play::serverbound::reconfiguration::packet::ServerboundReconfigurationPacketKind;
use ferrite_protocol::java_26_2::play::serverbound::reconfiguration::transition::{
    AcknowledgementExecutionLane, ReplacementCommonListenerCookieField,
    ServerInboundAcknowledgementStep, ServerInboundReconfigurationEffect,
    ServerInboundReconfigurationError, ServerInboundReconfigurationStage,
    ServerInboundReconfigurationTransition, acknowledgement_execution_lane,
};
use ferrite_protocol::java_26_2::wire::compression::{CompressionMode, encode_packet};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;

#[test]
fn c4_serverbound_reconfiguration_inventory_and_fieldless_golden_are_exact() {
    assert_eq!(
        ServerboundReconfigurationPacketKind::ALL,
        [ServerboundReconfigurationPacketKind::ConfigurationAcknowledged]
    );
    let kind = ServerboundReconfigurationPacketKind::ConfigurationAcknowledged;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Play,
        PacketDirection::Serverbound,
        kind.wire_id(),
    )
    .unwrap();
    assert_eq!(descriptor.identity(), kind.identity());
    assert_eq!(
        encode_packet(
            &[u8::try_from(kind.wire_id()).unwrap()],
            CompressionMode::enabled(256).unwrap(),
            FrameLimits::default(),
        )
        .unwrap(),
        [2, 0, 16]
    );
}

#[test]
fn c4_serverbound_reconfiguration_gate_is_default_closed_and_service_bound() {
    assert_eq!(
        ServerboundReconfigurationGates::default().decide(ServerboundReconfigurationContext {
            service_registered: true,
        }),
        ServerboundReconfigurationDecision::OmitDisabled(
            ServerboundReconfigurationPacketKind::ConfigurationAcknowledged
        )
    );
    let gates = ServerboundReconfigurationGates {
        reconfiguration: true,
    };
    assert_eq!(
        gates.decide(ServerboundReconfigurationContext::default()),
        ServerboundReconfigurationDecision::DegradeServiceUnavailable
    );
    assert_eq!(
        gates.decide(ServerboundReconfigurationContext {
            service_registered: true,
        }),
        ServerboundReconfigurationDecision::AdmitTerminalAcknowledgement
    );
}

#[test]
fn c4_serverbound_reconfiguration_faults_early_and_duplicate_acknowledgements() {
    let mut transition = ServerInboundReconfigurationTransition::new();
    assert_eq!(transition.stage(), ServerInboundReconfigurationStage::Play);
    assert_eq!(
        transition.handle_acknowledgement(),
        Err(ServerInboundReconfigurationError::AcknowledgementOutsideWaiting)
    );
    transition.begin_waiting().unwrap();
    assert_eq!(
        transition.stage(),
        ServerInboundReconfigurationStage::WaitingForAcknowledgement
    );
    assert_eq!(
        transition.handle_acknowledgement().unwrap(),
        ServerInboundReconfigurationEffect::InstallConfigurationInboundAtTerminalBoundary
    );
    assert_eq!(
        transition.stage(),
        ServerInboundReconfigurationStage::Configuration
    );
    assert_eq!(
        transition.handle_acknowledgement(),
        Err(ServerInboundReconfigurationError::AcknowledgementOutsideWaiting)
    );
    assert_eq!(
        transition.begin_waiting(),
        Err(ServerInboundReconfigurationError::StartOutsidePlay)
    );
}

#[test]
fn c4_serverbound_acknowledgement_is_a_direct_terminal_inbound_switch() {
    assert_eq!(
        acknowledgement_execution_lane(),
        AcknowledgementExecutionLane::DirectTerminalNetworkBoundary
    );
    assert_eq!(
        ServerInboundAcknowledgementStep::ORDER,
        [
            ServerInboundAcknowledgementStep::ValidateOldPlayListenerWaiting,
            ServerInboundAcknowledgementStep::CaptureReplacementCommonListenerCookie,
            ServerInboundAcknowledgementStep::InstallConfigurationInbound,
        ]
    );
}

#[test]
fn c4_replacement_cookie_excludes_the_client_cookie_map() {
    assert_eq!(ReplacementCommonListenerCookieField::ALL.len(), 4);
    assert_eq!(
        ReplacementCommonListenerCookieField::ALL,
        [
            ReplacementCommonListenerCookieField::Profile,
            ReplacementCommonListenerCookieField::CurrentLatency,
            ReplacementCommonListenerCookieField::LatestClientInformation,
            ReplacementCommonListenerCookieField::Transferred,
        ]
    );
}

#[test]
fn c4_direction_local_terminal_order_keeps_ack_under_play() {
    assert_eq!(
        ServerReconfigurationStep::ORDER,
        [
            ServerReconfigurationStep::SetWaitingForAcknowledgement,
            ServerReconfigurationStep::SaveAndRemovePlayerFromPlay,
            ServerReconfigurationStep::SendTerminalStartConfiguration,
            ServerReconfigurationStep::InstallConfigurationOutbound,
        ]
    );
    let install_inbound = ClientReconfigurationStep::ORDER
        .iter()
        .position(|step| matches!(step, ClientReconfigurationStep::InstallConfigurationInbound))
        .unwrap();
    let send_ack = ClientReconfigurationStep::ORDER
        .iter()
        .position(|step| {
            matches!(
                step,
                ClientReconfigurationStep::SendTerminalPlayAcknowledgement
            )
        })
        .unwrap();
    let install_outbound = ClientReconfigurationStep::ORDER
        .iter()
        .position(|step| {
            matches!(
                step,
                ClientReconfigurationStep::InstallConfigurationOutbound
            )
        })
        .unwrap();
    assert!(install_inbound < send_ack);
    assert!(send_ack < install_outbound);
}

#[test]
fn c4_serverbound_reconfiguration_required_decoder_remains_fail_closed() {
    let kind = ServerboundReconfigurationPacketKind::ConfigurationAcknowledged;
    assert!(matches!(
        decode_required_packet(&[u8::try_from(kind.wire_id()).unwrap()]),
        Err(PlayServerboundEntryCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:configuration_acknowledged"
        })
    ));
}
