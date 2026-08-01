use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::configuration::serverbound::optional::ResourcePackAction;
use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::serverbound::common_services::gate::{
    PlayCommonExecutionLane, PlayCommonServerboundContext, PlayCommonServerboundDecision,
    PlayCommonServerboundEffect, PlayCommonServerboundGates, execution_lane,
};
use ferrite_protocol::java_26_2::play::serverbound::common_services::packet::{
    PlayCommonServerboundPacket, PlayCommonServerboundPacketKind, PlayCustomPayloadKind,
};
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

#[test]
fn c4_play_common_serverbound_inventory_locks_all_five_entries() {
    assert_eq!(PlayCommonServerboundPacketKind::ALL.len(), 5);
    assert_eq!(
        PlayCommonServerboundPacketKind::ALL
            .into_iter()
            .map(PlayCommonServerboundPacketKind::wire_id)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([21, 22, 38, 49, 68])
    );
    for packet in PlayCommonServerboundPacketKind::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Play,
            PacketDirection::Serverbound,
            packet.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), packet.identity());
    }
}

#[test]
fn c4_gold_play_ping_request_preserves_every_signed_long_bit() {
    for token in [i64::MIN, -1, 0, i64::MAX] {
        let mut writer = WireWriter::new(9);
        writer.write_var_i32(38).unwrap();
        writer.write_i64(token).unwrap();
        let bytes = writer.into_inner();
        assert_eq!(bytes[0], 38);
        assert_eq!(&bytes[1..], &token.to_be_bytes());
    }
}

#[test]
fn c4_play_common_fixed_base_refusal_and_degradation_are_not_feature_gated() {
    let gates = PlayCommonServerboundGates::default();
    let context = PlayCommonServerboundContext::default();
    assert_eq!(
        gates.decide(PlayCommonServerboundPacket::CookieResponse, context),
        PlayCommonServerboundDecision::Emit(
            PlayCommonServerboundEffect::DisconnectUnexpectedCookieResponse
        )
    );
    for payload in [
        PlayCustomPayloadKind::Brand,
        PlayCustomPayloadKind::Discarded,
    ] {
        assert_eq!(
            gates.decide(PlayCommonServerboundPacket::CustomPayload(payload), context),
            PlayCommonServerboundDecision::Emit(PlayCommonServerboundEffect::IgnoreCustomPayload)
        );
    }
    assert_eq!(
        gates.decide(PlayCommonServerboundPacket::CustomClickAction, context),
        PlayCommonServerboundDecision::Emit(PlayCommonServerboundEffect::LogCustomClickOnly)
    );
}

#[test]
fn c4_play_ping_is_direct_uncorrelated_and_default_disabled() {
    let packet = PlayCommonServerboundPacket::PingRequest { token: i64::MIN };
    assert_eq!(
        PlayCommonServerboundGates::default()
            .decide(packet, PlayCommonServerboundContext::default()),
        PlayCommonServerboundDecision::OmitDisabled(PlayCommonServerboundPacketKind::PingRequest)
    );
    assert_eq!(
        PlayCommonServerboundGates {
            ping: true,
            ..PlayCommonServerboundGates::default()
        }
        .decide(packet, PlayCommonServerboundContext::default()),
        PlayCommonServerboundDecision::Emit(PlayCommonServerboundEffect::EchoPingDirect {
            token: i64::MIN,
        })
    );
}

#[test]
fn c4_play_resource_pack_has_no_uuid_correlation_and_only_required_decline_disconnects() {
    let gates = PlayCommonServerboundGates {
        resource_packs: true,
        ..PlayCommonServerboundGates::default()
    };
    for action in [
        ResourcePackAction::SuccessfullyLoaded,
        ResourcePackAction::FailedDownload,
        ResourcePackAction::Accepted,
        ResourcePackAction::Downloaded,
        ResourcePackAction::InvalidUrl,
        ResourcePackAction::FailedReload,
        ResourcePackAction::Discarded,
    ] {
        assert_eq!(
            gates.decide(
                PlayCommonServerboundPacket::ResourcePack { action },
                PlayCommonServerboundContext {
                    required_resource_pack: true,
                    ..PlayCommonServerboundContext::default()
                }
            ),
            PlayCommonServerboundDecision::Emit(
                PlayCommonServerboundEffect::RecordResourcePackStatus { action }
            )
        );
    }
    assert_eq!(
        gates.decide(
            PlayCommonServerboundPacket::ResourcePack {
                action: ResourcePackAction::Declined,
            },
            PlayCommonServerboundContext {
                required_resource_pack: true,
                ..PlayCommonServerboundContext::default()
            }
        ),
        PlayCommonServerboundDecision::Emit(
            PlayCommonServerboundEffect::DisconnectRequiredPackDeclined
        )
    );
}

#[test]
fn c4_play_custom_click_requires_an_explicit_registered_handler() {
    let gates = PlayCommonServerboundGates {
        custom_click_dispatch: true,
        ..PlayCommonServerboundGates::default()
    };
    assert_eq!(
        gates.decide(
            PlayCommonServerboundPacket::CustomClickAction,
            PlayCommonServerboundContext::default()
        ),
        PlayCommonServerboundDecision::DegradeNoCustomClickHandler
    );
    assert_eq!(
        gates.decide(
            PlayCommonServerboundPacket::CustomClickAction,
            PlayCommonServerboundContext {
                custom_click_handler_registered: true,
                ..PlayCommonServerboundContext::default()
            }
        ),
        PlayCommonServerboundDecision::Emit(
            PlayCommonServerboundEffect::DispatchRegisteredCustomClick
        )
    );
}

#[test]
fn c4_play_common_execution_lanes_preserve_direct_and_server_processor_order() {
    for kind in [
        PlayCommonServerboundPacketKind::CookieResponse,
        PlayCommonServerboundPacketKind::CustomPayload,
        PlayCommonServerboundPacketKind::PingRequest,
    ] {
        assert_eq!(
            execution_lane(kind),
            PlayCommonExecutionLane::ReceivingThreadDirect
        );
    }
    for kind in [
        PlayCommonServerboundPacketKind::ResourcePack,
        PlayCommonServerboundPacketKind::CustomClickAction,
    ] {
        assert_eq!(
            execution_lane(kind),
            PlayCommonExecutionLane::ServerProcessor
        );
    }
}

#[test]
fn c4_play_common_required_decoder_remains_fail_closed() {
    for packet in PlayCommonServerboundPacketKind::ALL {
        assert!(matches!(
            decode_required_packet(&[u8::try_from(packet.wire_id()).unwrap()]),
            Err(PlayServerboundEntryCodecError::UnsupportedPacketIdentity { identity })
                if identity == packet.identity()
        ));
    }
}
