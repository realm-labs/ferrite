use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::configuration::serverbound::codec::{
    ConfigurationServerboundCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::configuration::serverbound::optional::{
    ConfigurationServerboundGate, ConfigurationServerboundGates,
    ConfigurationServerboundOptionalService, OptionalConfigurationCodecError,
    OptionalConfigurationGateError, OptionalConfigurationPacket, OptionalConfigurationPacketKind,
    OptionalConfigurationTask, OptionalServerboundDecision, ResourcePackAction, decode_packet,
    encode_packet,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt};
use ferrite_protocol::java_26_2::wire::error::WireError;
use ferrite_protocol::java_26_2::wire::frame::MAX_FRAME_LENGTH;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

fn identifier(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn custom_click(action: &str) -> OptionalConfigurationPacket {
    OptionalConfigurationPacket::CustomClickAction {
        action: identifier(action),
        payload: None,
    }
}

fn write_identifier(writer: &mut WireWriter, value: &str) {
    writer.write_utf(value, 32_767).unwrap();
}

fn custom_click_body(payload: &[u8]) -> Vec<u8> {
    let mut writer = WireWriter::new(MAX_FRAME_LENGTH);
    writer.write_var_i32(8).unwrap();
    write_identifier(&mut writer, "ferrite:action");
    writer
        .write_var_i32(i32::try_from(payload.len()).unwrap())
        .unwrap();
    writer.write_bytes(payload).unwrap();
    writer.into_inner()
}

#[test]
fn c4_configuration_serverbound_inventory_locks_all_four_catalog_entries() {
    assert_eq!(OptionalConfigurationPacketKind::ALL.len(), 4);
    let ids = OptionalConfigurationPacketKind::ALL
        .into_iter()
        .map(OptionalConfigurationPacketKind::wire_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, BTreeSet::from([1, 6, 8, 9]));
    for packet in OptionalConfigurationPacketKind::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Configuration,
            PacketDirection::Serverbound,
            packet.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), packet.identity());
    }
}

#[test]
fn c4_configuration_serverbound_optional_codec_round_trips_exact_schemas() {
    let packets = [
        OptionalConfigurationPacket::CookieResponse {
            key: identifier("ferrite:cookie"),
            value: Some(vec![0, 1, 255]),
        },
        OptionalConfigurationPacket::CookieResponse {
            key: identifier("ferrite:empty"),
            value: None,
        },
        OptionalConfigurationPacket::ResourcePack {
            pack_id: u128::MAX,
            action: ResourcePackAction::FailedReload,
        },
        custom_click("ferrite:no_payload"),
        OptionalConfigurationPacket::CustomClickAction {
            action: identifier("ferrite:payload"),
            payload: Some(NetworkNbt::from_bytes(vec![10, 0], NbtQuota::Trusted).unwrap()),
        },
        OptionalConfigurationPacket::AcceptCodeOfConduct,
    ];
    for packet in packets {
        assert_eq!(
            decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
            packet
        );
    }
}

#[test]
fn c4_configuration_serverbound_optional_codec_enforces_cookie_and_action_bounds() {
    let oversized = OptionalConfigurationPacket::CookieResponse {
        key: identifier("ferrite:cookie"),
        value: Some(vec![0; 5_121]),
    };
    assert!(matches!(
        encode_packet(&oversized),
        Err(OptionalConfigurationCodecError::Wire(
            WireError::LengthLimit { maximum: 5_120, .. }
        ))
    ));

    let mut cookie = WireWriter::new(MAX_FRAME_LENGTH);
    cookie.write_var_i32(1).unwrap();
    write_identifier(&mut cookie, "ferrite:cookie");
    cookie.write_bool(true).unwrap();
    cookie.write_var_i32(5_121).unwrap();
    cookie.write_bytes(&vec![0; 5_121]).unwrap();
    assert!(matches!(
        decode_packet(&cookie.into_inner()),
        Err(OptionalConfigurationCodecError::Wire(
            WireError::LengthLimit { maximum: 5_120, .. }
        ))
    ));

    let mut action = vec![6];
    action.extend_from_slice(&0_u128.to_be_bytes());
    action.push(8);
    assert_eq!(
        decode_packet(&action),
        Err(OptionalConfigurationCodecError::InvalidResourcePackAction { ordinal: 8 })
    );
}

#[test]
fn c4_custom_click_codec_enforces_prefix_accumulator_and_depth_independently() {
    let oversized_prefix = vec![0; 65_537];
    assert!(matches!(
        decode_packet(&custom_click_body(&oversized_prefix)),
        Err(OptionalConfigurationCodecError::Wire(
            WireError::LengthLimit {
                maximum: 65_536,
                ..
            }
        ))
    ));

    let mut oversized_nbt = vec![7];
    oversized_nbt.extend_from_slice(&32_769_i32.to_be_bytes());
    oversized_nbt.extend_from_slice(&vec![0; 32_769]);
    assert!(matches!(
        decode_packet(&custom_click_body(&oversized_nbt)),
        Err(OptionalConfigurationCodecError::Nbt(
            NbtError::QuotaExceeded { quota: 32_768 }
        ))
    ));

    let mut too_deep = vec![10];
    for _ in 0..16 {
        too_deep.extend_from_slice(&[10, 0, 0]);
    }
    too_deep.extend_from_slice(&[0; 17]);
    assert_eq!(
        decode_packet(&custom_click_body(&too_deep)),
        Err(OptionalConfigurationCodecError::Nbt(
            NbtError::DepthExceeded { maximum: 16 }
        ))
    );
}

#[test]
fn c4_required_and_optional_decoders_keep_their_fail_closed_family_boundary() {
    assert!(matches!(
        decode_required_packet(&[1]),
        Err(
            ConfigurationServerboundCodecError::UnsupportedPacketIdentity {
                identity: "minecraft:cookie_response"
            }
        )
    ));
    assert!(matches!(
        decode_packet(&[3]),
        Err(OptionalConfigurationCodecError::RequiredPacketIdentity {
            identity: "minecraft:finish_configuration"
        })
    ));
}

#[test]
fn c4_default_gate_rejects_every_optional_service_explicitly() {
    let cases = [
        (
            OptionalConfigurationPacket::CookieResponse {
                key: identifier("ferrite:cookie"),
                value: None,
            },
            ConfigurationServerboundOptionalService::Cookies,
        ),
        (
            OptionalConfigurationPacket::ResourcePack {
                pack_id: 1,
                action: ResourcePackAction::Accepted,
            },
            ConfigurationServerboundOptionalService::ResourcePacks,
        ),
        (
            custom_click("ferrite:action"),
            ConfigurationServerboundOptionalService::CustomClick,
        ),
        (
            OptionalConfigurationPacket::AcceptCodeOfConduct,
            ConfigurationServerboundOptionalService::CodeOfConduct,
        ),
    ];
    for (packet, service) in cases {
        let mut gate = ConfigurationServerboundGate::new(
            ConfigurationServerboundGates::default(),
            OptionalConfigurationTask::None,
        );
        assert_eq!(
            gate.apply(packet),
            Err(OptionalConfigurationGateError::Disabled { service })
        );
    }
}

#[test]
fn c4_cookie_response_requires_enabled_matching_request_and_consumes_it_once() {
    let key = identifier("ferrite:cookie");
    let mut gate = ConfigurationServerboundGate::new(
        ConfigurationServerboundGates {
            cookies: true,
            ..ConfigurationServerboundGates::default()
        },
        OptionalConfigurationTask::CookieRequest { key: key.clone() },
    );
    assert!(matches!(
        gate.apply(OptionalConfigurationPacket::CookieResponse {
            key: identifier("ferrite:wrong"),
            value: None,
        }),
        Err(OptionalConfigurationGateError::CookieKeyMismatch { .. })
    ));
    assert_eq!(
        gate.apply(OptionalConfigurationPacket::CookieResponse {
            key: key.clone(),
            value: Some(vec![1]),
        }),
        Ok(OptionalServerboundDecision::CookieResponse {
            value: Some(vec![1])
        })
    );
    assert_eq!(gate.task(), &OptionalConfigurationTask::None);
    assert!(matches!(
        gate.apply(OptionalConfigurationPacket::CookieResponse { key, value: None }),
        Err(OptionalConfigurationGateError::UnexpectedTask { .. })
    ));
}

#[test]
fn c4_resource_pack_only_advances_on_terminal_and_does_not_correlate_terminal_uuid() {
    let mut gate = ConfigurationServerboundGate::new(
        ConfigurationServerboundGates {
            resource_packs: true,
            ..ConfigurationServerboundGates::default()
        },
        OptionalConfigurationTask::ResourcePack { required: false },
    );
    assert_eq!(
        gate.apply(OptionalConfigurationPacket::ResourcePack {
            pack_id: 1,
            action: ResourcePackAction::Accepted,
        }),
        Ok(OptionalServerboundDecision::AwaitResourcePackTerminal {
            action: ResourcePackAction::Accepted
        })
    );
    assert_eq!(
        gate.task(),
        &OptionalConfigurationTask::ResourcePack { required: false }
    );
    assert_eq!(
        gate.apply(OptionalConfigurationPacket::ResourcePack {
            pack_id: 999,
            action: ResourcePackAction::SuccessfullyLoaded,
        }),
        Ok(OptionalServerboundDecision::ResourcePackCompleted {
            pack_id: 999,
            action: ResourcePackAction::SuccessfullyLoaded,
        })
    );
    assert_eq!(gate.task(), &OptionalConfigurationTask::None);
}

#[test]
fn c4_required_resource_pack_decline_disconnects_and_conduct_accept_requires_current_task() {
    let mut pack = ConfigurationServerboundGate::new(
        ConfigurationServerboundGates {
            resource_packs: true,
            ..ConfigurationServerboundGates::default()
        },
        OptionalConfigurationTask::ResourcePack { required: true },
    );
    assert_eq!(
        pack.apply(OptionalConfigurationPacket::ResourcePack {
            pack_id: 7,
            action: ResourcePackAction::Declined,
        }),
        Ok(OptionalServerboundDecision::DisconnectRequiredPackDeclined { pack_id: 7 })
    );

    let gates = ConfigurationServerboundGates {
        code_of_conduct: true,
        ..ConfigurationServerboundGates::default()
    };
    let mut unsolicited = ConfigurationServerboundGate::new(gates, OptionalConfigurationTask::None);
    assert!(matches!(
        unsolicited.apply(OptionalConfigurationPacket::AcceptCodeOfConduct),
        Err(OptionalConfigurationGateError::UnexpectedTask { .. })
    ));
    let mut current =
        ConfigurationServerboundGate::new(gates, OptionalConfigurationTask::CodeOfConduct);
    assert_eq!(
        current.apply(OptionalConfigurationPacket::AcceptCodeOfConduct),
        Ok(OptionalServerboundDecision::CodeOfConductAccepted)
    );
    assert_eq!(current.task(), &OptionalConfigurationTask::None);
}

#[test]
fn c4_custom_click_dispatches_only_to_owned_handler_without_advancing_task() {
    let task = OptionalConfigurationTask::ResourcePack { required: false };
    let mut gate = ConfigurationServerboundGate::new(
        ConfigurationServerboundGates {
            custom_click: true,
            ..ConfigurationServerboundGates::default()
        },
        task.clone(),
    );
    assert_eq!(
        gate.apply(custom_click("ferrite:action")),
        Ok(OptionalServerboundDecision::DispatchCustomClick {
            action: identifier("ferrite:action"),
            payload: None,
        })
    );
    assert_eq!(gate.task(), &task);
}
