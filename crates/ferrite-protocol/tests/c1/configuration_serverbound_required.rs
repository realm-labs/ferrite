use ferrite_protocol::java_26_2::configuration::serverbound::codec::{
    ConfigurationServerboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::configuration::serverbound::packet::{
    ChatVisibility, ClientInformation, ConfigurationServerboundPacket, CustomPayload, MainHand,
    ParticleStatus,
};
use ferrite_protocol::java_26_2::configuration::serverbound::session::{
    ConfigurationServerSession, ConfigurationServerSessionError, ConfigurationTask,
    KEEPALIVE_INTERVAL_MILLIS, ServerAction,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::known_pack::KnownPack;
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

fn golden_frame(packet: &ConfigurationServerboundPacket) -> Vec<u8> {
    encode_wire(
        &encode_packet(packet).unwrap(),
        CompressionMode::enabled(256).unwrap(),
        FrameLimits::default(),
    )
    .unwrap()
}

fn session(offered_packs: Vec<KnownPack>) -> ConfigurationServerSession {
    ConfigurationServerSession::new(offered_packs, ClientInformation::default(), 100, 40)
}

#[test]
fn matches_every_locked_configuration_serverbound_golden() {
    let cases = [
        (
            ConfigurationServerboundPacket::ClientInformation(ClientInformation::default()),
            "10000005656e5f75730200010001000000",
        ),
        (
            ConfigurationServerboundPacket::CustomPayload(CustomPayload::Brand(
                "vanilla".to_owned(),
            )),
            "1a00020f6d696e6563726166743a6272616e640776616e696c6c61",
        ),
        (
            ConfigurationServerboundPacket::SelectKnownPacks(Vec::new()),
            "03000700",
        ),
        (
            ConfigurationServerboundPacket::FinishConfiguration,
            "020003",
        ),
        (
            ConfigurationServerboundPacket::KeepAlive(0x0102_0304_0506_0708),
            "0a00040102030405060708",
        ),
        (
            ConfigurationServerboundPacket::Pong(0x0102_0304),
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
fn client_information_enforces_fields_and_enum_ordinals() {
    let boundary = ClientInformation {
        language: "😀😀😀😀😀😀😀😀".to_owned(),
        view_distance: i8::MIN,
        chat_visibility: ChatVisibility::Hidden,
        chat_colors: true,
        model_customization: u8::MAX,
        main_hand: MainHand::Left,
        text_filtering: true,
        allows_listing: true,
        particle_status: ParticleStatus::Minimal,
    };
    let packet = ConfigurationServerboundPacket::ClientInformation(boundary);
    assert_eq!(
        decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
        packet
    );

    let too_long = ConfigurationServerboundPacket::ClientInformation(ClientInformation {
        language: "12345678901234567".to_owned(),
        ..ClientInformation::default()
    });
    assert!(matches!(
        encode_packet(&too_long),
        Err(ConfigurationServerboundCodecError::Wire(_))
    ));

    let mut invalid = encode_packet(&ConfigurationServerboundPacket::ClientInformation(
        ClientInformation::default(),
    ))
    .unwrap();
    invalid[8] = 3;
    assert!(matches!(
        decode_packet(&invalid),
        Err(ConfigurationServerboundCodecError::InvalidEnum {
            kind: "chat visibility",
            ordinal: 3
        })
    ));
}

#[test]
fn custom_payload_and_known_pack_counts_use_directional_caps() {
    let mut writer = WireWriter::new(100_000);
    writer.write_var_i32(2).unwrap();
    writer.write_utf("ferrite:probe", 32_767).unwrap();
    writer.write_bytes(&vec![7; 32_767]).unwrap();
    assert_eq!(
        decode_packet(writer.as_slice()).unwrap(),
        ConfigurationServerboundPacket::CustomPayload(CustomPayload::Discarded {
            channel: id("ferrite:probe"),
            length: 32_767,
        })
    );
    writer.write_u8(0).unwrap();
    assert!(decode_packet(writer.as_slice()).is_err());

    let packs = vec![KnownPack::vanilla_core(); 64];
    let packet = ConfigurationServerboundPacket::SelectKnownPacks(packs);
    assert_eq!(
        decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
        packet
    );
    assert!(
        encode_packet(&ConfigurationServerboundPacket::SelectKnownPacks(
            vec![KnownPack::vanilla_core(); 65]
        ))
        .is_err()
    );
    assert!(decode_packet(&[7, 65]).is_err());
}

#[test]
fn latest_information_and_exact_known_pack_equality_are_connection_local() {
    let offer = vec![KnownPack::vanilla_core()];
    let mut server = session(offer.clone());
    let replacement = ClientInformation {
        language: "zh_cn".to_owned(),
        view_distance: 12,
        ..ClientInformation::default()
    };
    assert_eq!(
        server
            .apply(
                ConfigurationServerboundPacket::ClientInformation(replacement.clone()),
                200,
                false,
            )
            .unwrap(),
        ServerAction::None
    );
    assert_eq!(server.client_information(), &replacement);
    assert_eq!(
        server
            .apply(
                ConfigurationServerboundPacket::CustomPayload(CustomPayload::Brand(
                    "ignored".to_owned()
                )),
                200,
                false,
            )
            .unwrap(),
        ServerAction::None
    );

    let action = server
        .apply(
            ConfigurationServerboundPacket::SelectKnownPacks(offer.clone()),
            200,
            false,
        )
        .unwrap();
    assert!(matches!(
        action,
        ServerAction::RegistrySelection(selection)
            if selection.exact_offer_match && selection.selected_packs == offer
    ));
    assert_eq!(server.task(), ConfigurationTask::PrepareSpawn);

    let mut reordered = session(vec![
        KnownPack::vanilla_core(),
        KnownPack {
            namespace: "ferrite".to_owned(),
            id: "extra".to_owned(),
            version: "1".to_owned(),
        },
    ]);
    let action = reordered
        .apply(
            ConfigurationServerboundPacket::SelectKnownPacks(vec![
                KnownPack {
                    namespace: "ferrite".to_owned(),
                    id: "extra".to_owned(),
                    version: "1".to_owned(),
                },
                KnownPack::vanilla_core(),
            ]),
            200,
            false,
        )
        .unwrap();
    assert!(matches!(
        action,
        ServerAction::RegistrySelection(selection) if !selection.exact_offer_match
    ));
}

#[test]
fn finish_is_terminal_only_for_the_current_join_world_task() {
    let mut early = session(Vec::new());
    assert!(matches!(
        early.apply(
            ConfigurationServerboundPacket::FinishConfiguration,
            200,
            false
        ),
        Err(ConfigurationServerSessionError::UnexpectedTask { .. })
    ));
    assert_eq!(early.task(), ConfigurationTask::Disconnected);

    let replacement = ClientInformation {
        language: "de_de".to_owned(),
        ..ClientInformation::default()
    };
    let mut server = session(Vec::new());
    server
        .apply(
            ConfigurationServerboundPacket::ClientInformation(replacement.clone()),
            200,
            false,
        )
        .unwrap();
    server
        .apply(
            ConfigurationServerboundPacket::SelectKnownPacks(Vec::new()),
            200,
            false,
        )
        .unwrap();
    server.spawn_ready_and_finish_sent().unwrap();
    assert_eq!(server.task(), ConfigurationTask::JoinWorld);
    assert!(matches!(
        server
            .apply(
                ConfigurationServerboundPacket::FinishConfiguration,
                200,
                false
            )
            .unwrap(),
        ServerAction::BeginPlayInstallation(installation)
            if installation.client_information == replacement
    ));
    assert_eq!(server.task(), ConfigurationTask::InstallingPlay);
    server.play_installation_completed().unwrap();
    assert_eq!(server.task(), ConfigurationTask::Play);
    assert!(
        server
            .apply(
                ConfigurationServerboundPacket::FinishConfiguration,
                200,
                false
            )
            .is_err()
    );
}

#[test]
fn keepalive_challenge_timeout_and_latency_match_the_locked_scheduler() {
    let mut server = session(Vec::new());
    assert_eq!(
        server.poll_liveness(15_099, false).unwrap(),
        ServerAction::None
    );
    assert_eq!(
        server.poll_liveness(15_100, false).unwrap(),
        ServerAction::SendKeepAlive(15_100)
    );
    assert_eq!(server.pending_keepalive(), Some(15_100));
    assert_eq!(
        server
            .apply(
                ConfigurationServerboundPacket::KeepAlive(15_100),
                15_120,
                false
            )
            .unwrap(),
        ServerAction::KeepAliveAccepted { latency_millis: 35 }
    );
    assert_eq!(server.pending_keepalive(), None);
    assert_eq!(server.latency_millis(), 35);

    assert_eq!(
        server
            .poll_liveness(15_100 + KEEPALIVE_INTERVAL_MILLIS, false)
            .unwrap(),
        ServerAction::SendKeepAlive(30_100)
    );
    assert_eq!(
        server
            .poll_liveness(30_100 + KEEPALIVE_INTERVAL_MILLIS, false)
            .unwrap(),
        ServerAction::DisconnectTimeout
    );
    assert_eq!(server.task(), ConfigurationTask::Disconnected);
}

#[test]
fn invalid_echo_disconnects_remote_but_owner_and_pong_are_ignored() {
    let mut remote = session(Vec::new());
    assert_eq!(
        remote
            .apply(ConfigurationServerboundPacket::KeepAlive(-1), 200, false)
            .unwrap(),
        ServerAction::DisconnectTimeout
    );

    let mut owner = session(Vec::new());
    assert_eq!(
        owner.poll_liveness(i64::MAX, true).unwrap(),
        ServerAction::None
    );
    assert_eq!(
        owner
            .apply(ConfigurationServerboundPacket::KeepAlive(-1), 200, true)
            .unwrap(),
        ServerAction::None
    );
    assert_eq!(
        owner
            .apply(ConfigurationServerboundPacket::Pong(i32::MIN), 200, true)
            .unwrap(),
        ServerAction::None
    );
    assert_eq!(owner.task(), ConfigurationTask::SynchronizeRegistries);
}

#[test]
fn optional_and_unknown_ids_fail_closed_at_the_family_boundary() {
    assert!(matches!(
        decode_packet(&[1]),
        Err(
            ConfigurationServerboundCodecError::UnsupportedPacketIdentity {
                identity: "minecraft:cookie_response"
            }
        )
    ));
    assert!(matches!(
        decode_packet(&[127]),
        Err(ConfigurationServerboundCodecError::UnknownPacketId { id: 127 })
    ));
}
