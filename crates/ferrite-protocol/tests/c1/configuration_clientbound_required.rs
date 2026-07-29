use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::configuration::clientbound::codec::{
    ConfigurationClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::configuration::clientbound::packet::{
    ConfigurationClientboundPacket, CustomPayload, KnownPack, RegistryData, RegistryEntry,
    RegistryTags, TagDefinition,
};
use ferrite_protocol::java_26_2::configuration::clientbound::projection::{
    ClientAction, ConfigurationProjection, ConfigurationProjectionError, ConfigurationStage,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;
use ferrite_protocol::java_26_2::wire::compression::{
    CompressionMode, encode_packet as encode_wire,
};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}

fn golden_frame(packet: &ConfigurationClientboundPacket) -> Vec<u8> {
    encode_wire(
        &encode_packet(packet).unwrap(),
        CompressionMode::enabled(256).unwrap(),
        FrameLimits::default(),
    )
    .unwrap()
}

#[test]
fn matches_all_locked_configuration_clientbound_goldens() {
    let cases = [
        (
            ConfigurationClientboundPacket::CustomPayload(CustomPayload::Brand(
                "vanilla".to_owned(),
            )),
            "1a00010f6d696e6563726166743a6272616e640776616e696c6c61",
        ),
        (
            ConfigurationClientboundPacket::UpdateEnabledFeatures(BTreeSet::from([id(
                "minecraft:vanilla",
            )])),
            "15000c01116d696e6563726166743a76616e696c6c61",
        ),
        (
            ConfigurationClientboundPacket::SelectKnownPacks(Vec::new()),
            "03000e00",
        ),
        (
            ConfigurationClientboundPacket::RegistryData(RegistryData {
                registry: id("minecraft:timeline"),
                entries: Vec::new(),
            }),
            "160007126d696e6563726166743a74696d656c696e6500",
        ),
        (
            ConfigurationClientboundPacket::UpdateTags(Vec::new()),
            "03000d00",
        ),
        (
            ConfigurationClientboundPacket::FinishConfiguration,
            "020003",
        ),
        (
            ConfigurationClientboundPacket::KeepAlive(0x0102_0304_0506_0708),
            "0a00040102030405060708",
        ),
        (
            ConfigurationClientboundPacket::Ping(0x0102_0304),
            "06000501020304",
        ),
    ];

    for (packet, expected) in cases {
        assert_eq!(golden_frame(&packet), hex(expected));
        let body = encode_packet(&packet).unwrap();
        assert_eq!(decode_packet(&body).unwrap(), packet);
    }
}

#[test]
fn disconnect_uses_trusted_context_free_component_nbt() {
    let packet = ConfigurationClientboundPacket::Disconnect(
        TextComponentNbt::literal("configuration failed").unwrap(),
    );
    let body = encode_packet(&packet).unwrap();
    assert_eq!(body[0], 2);
    assert_eq!(body[1], 8);
    assert_eq!(decode_packet(&body).unwrap(), packet);

    assert!(matches!(
        decode_packet(&[2, 1, 0]),
        Err(ConfigurationClientboundCodecError::InvalidNbt(_))
    ));
}

#[test]
fn custom_payload_enforces_brand_and_unknown_channel_boundaries() {
    let mut writer = WireWriter::new(2_000_000);
    writer.write_var_i32(1).unwrap();
    writer.write_utf("ferrite:probe", 32_767).unwrap();
    writer.write_bytes(&vec![7; 1_048_576]).unwrap();
    let decoded = decode_packet(writer.as_slice()).unwrap();
    assert_eq!(
        decoded,
        ConfigurationClientboundPacket::CustomPayload(CustomPayload::Discarded {
            channel: id("ferrite:probe"),
            length: 1_048_576,
        })
    );
    assert!(encode_packet(&decoded).is_err());

    let mut oversized = writer;
    oversized.write_u8(0).unwrap();
    assert!(matches!(
        decode_packet(oversized.as_slice()),
        Err(ConfigurationClientboundCodecError::Wire(_))
    ));
}

#[test]
fn malformed_counts_identifiers_and_packet_ids_fail_closed() {
    assert!(matches!(
        decode_packet(&[7, 1, b'A', 0]),
        Err(ConfigurationClientboundCodecError::InvalidIdentifier(_))
    ));
    assert!(decode_packet(&[12, 0xff, 0xff, 0xff, 0xff, 0x0f]).is_err());
    assert!(matches!(
        decode_packet(&[0]),
        Err(
            ConfigurationClientboundCodecError::UnsupportedPacketIdentity {
                identity: "minecraft:cookie_request"
            }
        )
    ));
    assert!(matches!(
        decode_packet(&[127]),
        Err(ConfigurationClientboundCodecError::UnknownPacketId { id: 127 })
    ));
}

#[test]
fn registry_tags_and_finish_follow_the_locked_transition_order() {
    let registry = id("minecraft:worldgen/biome");
    let first = id("minecraft:plains");
    let second = id("minecraft:desert");
    let tag = id("minecraft:is_overworld");
    let mut projection = ConfigurationProjection::default();

    assert_eq!(
        projection
            .apply(ConfigurationClientboundPacket::CustomPayload(
                CustomPayload::Brand("Ferrite".to_owned())
            ))
            .unwrap(),
        ClientAction::None
    );
    projection
        .apply(ConfigurationClientboundPacket::UpdateEnabledFeatures(
            BTreeSet::from([id("minecraft:vanilla"), id("ferrite:ignored_feature")]),
        ))
        .unwrap();
    assert_eq!(
        projection.enabled_features(),
        &BTreeSet::from([id("minecraft:vanilla")])
    );
    assert_eq!(
        projection
            .apply(ConfigurationClientboundPacket::SelectKnownPacks(vec![
                KnownPack::vanilla_core()
            ]))
            .unwrap(),
        ClientAction::SelectKnownPacks(vec![KnownPack::vanilla_core()])
    );
    projection.known_pack_response_sent().unwrap();

    for entry in [first.clone(), second.clone()] {
        projection
            .apply(ConfigurationClientboundPacket::RegistryData(RegistryData {
                registry: registry.clone(),
                entries: vec![RegistryEntry {
                    id: entry,
                    data: None,
                }],
            }))
            .unwrap();
    }
    projection
        .apply(ConfigurationClientboundPacket::UpdateTags(vec![
            RegistryTags {
                registry: registry.clone(),
                tags: vec![TagDefinition {
                    id: tag.clone(),
                    members: vec![1],
                }],
            },
        ]))
        .unwrap();
    assert_eq!(
        projection
            .registry(&registry)
            .unwrap()
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert_eq!(projection.tags(&registry).unwrap()[0].id, tag);
    assert_eq!(
        projection.stage(),
        ConfigurationStage::AwaitingSpawnReadiness
    );

    projection.spawn_ready().unwrap();
    assert_eq!(
        projection
            .apply(ConfigurationClientboundPacket::FinishConfiguration)
            .unwrap(),
        ClientAction::InstallPlayThenAcknowledgeFinish
    );
    assert_eq!(
        projection.stage(),
        ConfigurationStage::PlayInstalledAwaitingFinishAcknowledgement
    );
    projection.finish_acknowledgement_sent().unwrap();
    assert_eq!(projection.stage(), ConfigurationStage::Play);
}

#[test]
fn projection_rejects_order_duplicates_and_bad_tag_members() {
    let mut projection = ConfigurationProjection::default();
    assert!(matches!(
        projection.apply(ConfigurationClientboundPacket::FinishConfiguration),
        Err(ConfigurationProjectionError::UnexpectedStage { .. })
    ));

    projection
        .apply(ConfigurationClientboundPacket::CustomPayload(
            CustomPayload::Brand("Ferrite".to_owned()),
        ))
        .unwrap();
    projection
        .apply(ConfigurationClientboundPacket::UpdateEnabledFeatures(
            BTreeSet::new(),
        ))
        .unwrap();
    projection
        .apply(ConfigurationClientboundPacket::SelectKnownPacks(Vec::new()))
        .unwrap();
    projection.known_pack_response_sent().unwrap();

    let registry = id("minecraft:timeline");
    let entry = id("minecraft:overworld");
    projection
        .apply(ConfigurationClientboundPacket::RegistryData(RegistryData {
            registry: registry.clone(),
            entries: vec![RegistryEntry {
                id: entry.clone(),
                data: None,
            }],
        }))
        .unwrap();
    assert!(matches!(
        projection.apply(ConfigurationClientboundPacket::RegistryData(RegistryData {
            registry: registry.clone(),
            entries: vec![RegistryEntry {
                id: entry,
                data: None,
            }],
        })),
        Err(ConfigurationProjectionError::DuplicateRegistryEntry { .. })
    ));
    assert!(matches!(
        projection.apply(ConfigurationClientboundPacket::UpdateTags(vec![
            RegistryTags {
                registry,
                tags: vec![TagDefinition {
                    id: id("minecraft:any"),
                    members: vec![1],
                }],
            },
        ])),
        Err(ConfigurationProjectionError::TagMemberOutOfRange { .. })
    ));
}

#[test]
fn liveness_tokens_echo_exactly_without_advancing_configuration() {
    let mut projection = ConfigurationProjection::default();
    assert_eq!(
        projection
            .apply(ConfigurationClientboundPacket::KeepAlive(i64::MIN))
            .unwrap(),
        ClientAction::EchoKeepAlive(i64::MIN)
    );
    assert_eq!(
        projection
            .apply(ConfigurationClientboundPacket::Ping(i32::MIN))
            .unwrap(),
        ClientAction::EchoPing(i32::MIN)
    );
    assert_eq!(projection.stage(), ConfigurationStage::AwaitingBrand);
}
