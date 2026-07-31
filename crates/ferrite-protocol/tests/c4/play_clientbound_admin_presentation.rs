use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::clientbound::admin_presentation::codec::{
    AdminPresentationCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::admin_presentation::gate::{
    AdminPresentationContext, AdminPresentationDecision, AdminPresentationEffect,
    AdminPresentationGates, LOW_DISK_WARNING_THRESHOLD_BYTES,
};
use ferrite_protocol::java_26_2::play::clientbound::admin_presentation::packet::{
    AdminPresentationPacket, AdminPresentationPacketKind, Vec3i,
};
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;
use ferrite_protocol::java_26_2::wire::error::WireError;
use ferrite_protocol::java_26_2::wire::frame::MAX_FRAME_LENGTH;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

fn identifier(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn component(text: &str) -> TextComponentNbt {
    TextComponentNbt::literal(text).unwrap()
}

fn context() -> AdminPresentationContext {
    AdminPresentationContext {
        authorized_request: true,
        direct_recipient: true,
        dedicated_server: true,
        administrator: true,
        usable_space_bytes: Some(LOW_DISK_WARNING_THRESHOLD_BYTES - 1),
    }
}

fn packets() -> [AdminPresentationPacket; 4] {
    [
        AdminPresentationPacket::GameRuleValues(BTreeMap::from([
            (
                identifier("minecraft:do_daylight_cycle"),
                "false".to_owned(),
            ),
            (identifier("minecraft:random_tick_speed"), "3".to_owned()),
        ])),
        AdminPresentationPacket::GameTestHighlightPosition {
            absolute: BlockPos::new(-33_554_432, -2_048, 33_554_431),
            relative: BlockPos::new(1, 2, 3),
        },
        AdminPresentationPacket::LowDiskSpaceWarning,
        AdminPresentationPacket::TestInstanceBlockStatus {
            status: component("ready"),
            size: Some(Vec3i {
                x: i32::MIN,
                y: 0,
                z: i32::MAX,
            }),
        },
    ]
}

#[test]
fn c4_admin_presentation_inventory_locks_all_four_catalog_entries() {
    assert_eq!(AdminPresentationPacketKind::ALL.len(), 4);
    let ids = AdminPresentationPacketKind::ALL
        .into_iter()
        .map(AdminPresentationPacketKind::wire_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, BTreeSet::from([39, 40, 50, 126]));
    for packet in AdminPresentationPacketKind::ALL {
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
fn c4_admin_presentation_codec_round_trips_all_fields() {
    for packet in packets() {
        assert_eq!(
            decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
            packet
        );
    }
    let absent_size = AdminPresentationPacket::TestInstanceBlockStatus {
        status: component("missing"),
        size: None,
    };
    assert_eq!(
        decode_packet(&encode_packet(&absent_size).unwrap()).unwrap(),
        absent_size
    );
}

#[test]
fn c4_gamerule_decode_overwrites_duplicate_keys_and_retains_unknown_values() {
    let mut writer = WireWriter::new(MAX_FRAME_LENGTH);
    writer.write_var_i32(39).unwrap();
    writer.write_var_i32(2).unwrap();
    writer.write_utf("minecraft:unknown", 32_767).unwrap();
    writer.write_utf("first", 32_767).unwrap();
    writer.write_utf("minecraft:unknown", 32_767).unwrap();
    writer
        .write_utf("not-a-known-parser-value", 32_767)
        .unwrap();
    assert_eq!(
        decode_packet(&writer.into_inner()).unwrap(),
        AdminPresentationPacket::GameRuleValues(BTreeMap::from([(
            identifier("minecraft:unknown"),
            "not-a-known-parser-value".to_owned(),
        )]))
    );
}

#[test]
fn c4_admin_presentation_family_and_unit_body_fail_closed() {
    let registries = PlayRegistries::default();
    let values = RejectComponentValues;
    assert!(matches!(
        decode_required_packet(
            &[39],
            PlayDecodeContext {
                registries: &registries,
                component_values: &values,
                dimension_section_count: 24,
            },
        ),
        Err(PlayClientboundCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:game_rule_values"
        })
    ));
    assert!(matches!(
        decode_packet(&[0]),
        Err(AdminPresentationCodecError::OtherPacketIdentity {
            identity: "minecraft:bundle_delimiter"
        })
    ));
    assert_eq!(
        decode_packet(&[50, 0]),
        Err(AdminPresentationCodecError::Wire(WireError::LengthLimit {
            field: "trailing packet data",
            length: 1,
            maximum: 0,
        }))
    );
}

#[test]
fn c4_admin_presentation_default_gate_omits_every_packet() {
    let gates = AdminPresentationGates::default();
    for packet in packets() {
        assert_eq!(
            gates.decide(&packet, context()),
            AdminPresentationDecision::OmitDisabled(packet.kind())
        );
    }
}

#[test]
fn c4_gamerule_and_test_status_require_authorized_direct_requester() {
    let gates = AdminPresentationGates {
        game_rule_values: true,
        test_instance_status: true,
        ..AdminPresentationGates::default()
    };
    for packet in [packets()[0].clone(), packets()[3].clone()] {
        let mut denied = context();
        denied.authorized_request = false;
        assert_eq!(
            gates.decide(&packet, denied),
            AdminPresentationDecision::RefuseUnauthorizedRequest
        );
        let mut wrong_recipient = context();
        wrong_recipient.direct_recipient = false;
        assert_eq!(
            gates.decide(&packet, wrong_recipient),
            AdminPresentationDecision::OmitNonRequester
        );
    }
}

#[test]
fn c4_highlight_targets_only_invoker_and_all_admin_effects_are_presentation() {
    let gates = AdminPresentationGates {
        game_rule_values: true,
        game_test_highlight: true,
        test_instance_status: true,
        ..AdminPresentationGates::default()
    };
    let mut other = context();
    other.direct_recipient = false;
    assert_eq!(
        gates.decide(&packets()[1], other),
        AdminPresentationDecision::OmitNonRequester
    );
    assert_eq!(
        gates.decide(&packets()[0], context()),
        AdminPresentationDecision::Emit(AdminPresentationEffect::PresentGameRuleValues)
    );
    assert_eq!(
        gates.decide(&packets()[1], context()),
        AdminPresentationDecision::Emit(AdminPresentationEffect::HighlightGameTestPosition)
    );
    assert_eq!(
        gates.decide(&packets()[3], context()),
        AdminPresentationDecision::Emit(AdminPresentationEffect::PresentTestInstanceStatus)
    );
}

#[test]
fn c4_low_disk_warning_uses_strict_threshold_dedicated_admin_and_repeats() {
    let gates = AdminPresentationGates {
        low_disk_warning: true,
        ..AdminPresentationGates::default()
    };
    let packet = AdminPresentationPacket::LowDiskSpaceWarning;
    for omitted in [
        AdminPresentationContext {
            dedicated_server: false,
            ..context()
        },
        AdminPresentationContext {
            administrator: false,
            ..context()
        },
        AdminPresentationContext {
            usable_space_bytes: Some(LOW_DISK_WARNING_THRESHOLD_BYTES),
            ..context()
        },
        AdminPresentationContext {
            usable_space_bytes: None,
            ..context()
        },
    ] {
        assert_eq!(
            gates.decide(&packet, omitted),
            AdminPresentationDecision::OmitLowDiskWarningConditions
        );
    }
    let expected = AdminPresentationDecision::Emit(AdminPresentationEffect::ShowLowDiskToast);
    assert_eq!(gates.decide(&packet, context()), expected);
    assert_eq!(gates.decide(&packet, context()), expected);
}
