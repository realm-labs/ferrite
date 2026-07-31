use std::collections::{BTreeMap, BTreeSet};

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::common_services::codec::{
    CommonServicesCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::common_services::gate::{
    CommonService, CommonServiceContext, CommonServiceDecision, CommonServiceEffect,
    CommonServiceGates,
};
use ferrite_protocol::java_26_2::play::clientbound::common_services::packet::{
    CommonCustomPayload, CommonServicePacket, CommonServicePacketKind, DialogHolder,
    ResourcePackPush, ServerLink, ServerLinkLabel,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::{NbtQuota, NetworkNbt, TextComponentNbt};
use ferrite_protocol::java_26_2::wire::error::WireError;

fn identifier(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn component(text: &str) -> TextComponentNbt {
    TextComponentNbt::literal(text).unwrap()
}

fn dialog() -> NetworkNbt {
    NetworkNbt::from_bytes(vec![10, 0], NbtQuota::Trusted).unwrap()
}

fn packets() -> Vec<CommonServicePacket> {
    vec![
        CommonServicePacket::CookieRequest {
            key: identifier("ferrite:cookie"),
        },
        CommonServicePacket::CustomPayload(CommonCustomPayload::Brand("Ferrite".to_owned())),
        CommonServicePacket::CustomPayload(CommonCustomPayload::Discarded {
            channel: identifier("ferrite:extension"),
            payload: vec![0, 1, 255],
        }),
        CommonServicePacket::PongResponse { token: i64::MIN },
        CommonServicePacket::ResourcePackPop { pack_id: None },
        CommonServicePacket::ResourcePackPop {
            pack_id: Some(u128::MAX),
        },
        CommonServicePacket::ResourcePackPush(ResourcePackPush {
            pack_id: 7,
            url: "https://example.invalid/pack.zip".to_owned(),
            hash: "a".repeat(40),
            required: true,
            prompt: Some(component("Install?")),
        }),
        CommonServicePacket::StoreCookie {
            key: identifier("ferrite:cookie"),
            value: vec![1, 2, 3],
        },
        CommonServicePacket::Transfer {
            host: "example.invalid".to_owned(),
            port: i32::MIN,
        },
        CommonServicePacket::CustomReportDetails(BTreeMap::from([(
            "version".to_owned(),
            "26.2".to_owned(),
        )])),
        CommonServicePacket::ServerLinks(vec![
            ServerLink {
                label: ServerLinkLabel::Known(9),
                url: "https://example.invalid/known".to_owned(),
            },
            ServerLink {
                label: ServerLinkLabel::Custom(component("Docs")),
                url: "https://example.invalid/docs".to_owned(),
            },
        ]),
        CommonServicePacket::ClearDialog,
        CommonServicePacket::ShowDialog {
            dialog: DialogHolder::Registered(1),
        },
        CommonServicePacket::ShowDialog {
            dialog: DialogHolder::Direct(dialog()),
        },
    ]
}

#[test]
fn c4_play_common_inventory_locks_all_eleven_catalog_entries() {
    assert_eq!(CommonServicePacketKind::ALL.len(), 11);
    let ids = CommonServicePacketKind::ALL
        .into_iter()
        .map(CommonServicePacketKind::wire_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ids,
        BTreeSet::from([21, 24, 62, 80, 81, 120, 129, 136, 137, 139, 140])
    );
    for packet in CommonServicePacketKind::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Play,
            PacketDirection::Clientbound,
            packet.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), packet.identity());
    }
}

#[test]
fn c4_play_common_codec_round_trips_all_packet_forms() {
    for packet in packets() {
        let encoded = encode_packet(&packet, 2).unwrap();
        assert_eq!(decode_packet(&encoded, 2).unwrap(), packet);
    }
}

#[test]
fn c4_play_common_codec_enforces_custom_cookie_hash_and_report_bounds() {
    let oversized_custom = CommonServicePacket::CustomPayload(CommonCustomPayload::Discarded {
        channel: identifier("ferrite:large"),
        payload: vec![0; 1_048_577],
    });
    assert!(matches!(
        encode_packet(&oversized_custom, 0),
        Err(CommonServicesCodecError::Wire(WireError::LengthLimit {
            maximum: 1_048_576,
            ..
        }))
    ));
    let oversized_cookie = CommonServicePacket::StoreCookie {
        key: identifier("ferrite:cookie"),
        value: vec![0; 5_121],
    };
    assert!(matches!(
        encode_packet(&oversized_cookie, 0),
        Err(CommonServicesCodecError::Wire(WireError::LengthLimit {
            maximum: 5_120,
            ..
        }))
    ));
    let oversized_hash = CommonServicePacket::ResourcePackPush(ResourcePackPush {
        pack_id: 0,
        url: String::new(),
        hash: "a".repeat(41),
        required: false,
        prompt: None,
    });
    assert!(matches!(
        encode_packet(&oversized_hash, 0),
        Err(CommonServicesCodecError::Wire(
            WireError::UtfCodeUnitLimit { maximum: 40, .. }
        ))
    ));
    let reports = CommonServicePacket::CustomReportDetails(
        (0..33)
            .map(|index| (format!("key-{index}"), String::new()))
            .collect(),
    );
    assert!(matches!(
        encode_packet(&reports, 0),
        Err(CommonServicesCodecError::Wire(WireError::LengthLimit {
            maximum: 32,
            ..
        }))
    ));
}

#[test]
fn c4_dialog_holder_is_strict_and_known_link_ids_use_type_zero_fallback() {
    let unknown = CommonServicePacket::ShowDialog {
        dialog: DialogHolder::Registered(2),
    };
    assert_eq!(
        encode_packet(&unknown, 2),
        Err(CommonServicesCodecError::UnknownDialog {
            raw_id: 2,
            registry_len: 2,
        })
    );
    assert_eq!(ServerLinkLabel::Known(-1).effective_known_type(), Some(0));
    assert_eq!(ServerLinkLabel::Known(10).effective_known_type(), Some(0));
    assert_eq!(ServerLinkLabel::Known(9).effective_known_type(), Some(9));
    assert_eq!(
        ServerLinkLabel::Custom(component("custom")).effective_known_type(),
        None
    );
}

#[test]
fn c4_play_common_required_and_optional_decoders_are_isolated() {
    let registries = PlayRegistries::default();
    let values = RejectComponentValues;
    assert!(matches!(
        decode_required_packet(
            &[21],
            PlayDecodeContext {
                registries: &registries,
                component_values: &values,
                dimension_section_count: 24,
            },
        ),
        Err(PlayClientboundCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:cookie_request"
        })
    ));
    assert!(matches!(
        decode_packet(&[0], 0),
        Err(CommonServicesCodecError::OtherPacketIdentity {
            identity: "minecraft:bundle_delimiter"
        })
    ));
}

#[test]
fn c4_play_common_default_gate_omits_every_service() {
    let gates = CommonServiceGates::default();
    for packet in packets() {
        assert!(matches!(
            gates.decide(&packet, CommonServiceContext::default()),
            CommonServiceDecision::OmitDisabled(_)
        ));
    }
}

#[test]
fn c4_play_common_enabled_services_emit_only_connection_or_presentation_effects() {
    let gates = CommonServiceGates {
        cookies: true,
        custom_payload: true,
        pong: true,
        resource_packs: true,
        transfer: true,
        report_details: true,
        server_links: true,
        dialogs: true,
    };
    let effects = packets()
        .iter()
        .map(|packet| gates.decide(packet, CommonServiceContext::default()))
        .collect::<Vec<_>>();
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::RequestCookieResponse
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::ReplaceBrand
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::DiscardCustomPayload
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::LogPongSample
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::UpdateResourcePackState
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::StoreConnectionCookie
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::TransferConnection
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::ReplaceReportDetails
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::ReplaceValidatedServerLinks
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::ClearDialogPresentation
    )));
    assert!(effects.contains(&CommonServiceDecision::Emit(
        CommonServiceEffect::ShowDialogPresentation
    )));
}

#[test]
fn c4_pong_is_uncorrelated_and_transfer_refuses_singleplayer() {
    let gates = CommonServiceGates {
        pong: true,
        transfer: true,
        ..CommonServiceGates::default()
    };
    let pong = CommonServicePacket::PongResponse { token: 7 };
    let expected = CommonServiceDecision::Emit(CommonServiceEffect::LogPongSample);
    assert_eq!(
        gates.decide(&pong, CommonServiceContext::default()),
        expected
    );
    assert_eq!(
        gates.decide(&pong, CommonServiceContext::default()),
        expected
    );
    let transfer = CommonServicePacket::Transfer {
        host: String::new(),
        port: -1,
    };
    assert_eq!(
        gates.decide(&transfer, CommonServiceContext { singleplayer: true }),
        CommonServiceDecision::RefuseTransferInSingleplayer
    );
    let disabled = CommonServiceGates::default();
    assert_eq!(
        disabled.decide(&transfer, CommonServiceContext::default()),
        CommonServiceDecision::OmitDisabled(CommonService::Transfer)
    );
}
