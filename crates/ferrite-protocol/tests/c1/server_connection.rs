use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::ConnectionState;
use ferrite_protocol::java_26_2::configuration::clientbound::codec as configuration_clientbound_codec;
use ferrite_protocol::java_26_2::configuration::clientbound::packet::{
    ConfigurationClientboundPacket, CustomPayload as ClientboundCustomPayload, RegistryTags,
    TagDefinition,
};
use ferrite_protocol::java_26_2::configuration::registry::SYNCHRONIZED_REGISTRY_IDENTITIES;
use ferrite_protocol::java_26_2::configuration::serverbound::codec as configuration_serverbound_codec;
use ferrite_protocol::java_26_2::configuration::serverbound::packet::{
    ClientInformation, ConfigurationServerboundPacket, CustomPayload as ServerboundCustomPayload,
};
use ferrite_protocol::java_26_2::connection::bootstrap::{
    ConfigurationSnapshot, ConfigurationSnapshotError, RegistryProjection, RegistryProjectionEntry,
};
use ferrite_protocol::java_26_2::connection::driver::ServerConnection;
use ferrite_protocol::java_26_2::connection::output::{
    ConnectionCloseReason, ServerConnectionEvent, ServerConnectionStage,
};
use ferrite_protocol::java_26_2::connection::settings::{
    DisconnectMessages, ServerConnectionSettings,
};
use ferrite_protocol::java_26_2::handshake::codec as handshake_codec;
use ferrite_protocol::java_26_2::handshake::packet::{ClientIntention, ClientIntentionPacket};
use ferrite_protocol::java_26_2::handshake::transition::LoginRefusal;
use ferrite_protocol::java_26_2::login::clientbound::codec as login_clientbound_codec;
use ferrite_protocol::java_26_2::login::clientbound::packet::LoginClientboundPacket;
use ferrite_protocol::java_26_2::login::serverbound::codec as login_serverbound_codec;
use ferrite_protocol::java_26_2::login::serverbound::packet::{LoginHello, LoginServerboundPacket};
use ferrite_protocol::java_26_2::login::serverbound::session::{
    AdmissionSnapshot, LoginDisconnect,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::clientbound::terrain::packet::{
    ChunkCoordinate, TerrainPacket,
};
use ferrite_protocol::java_26_2::play::registry::{BIOME, PlayRegistries};
use ferrite_protocol::java_26_2::play::serverbound::codec as play_serverbound_codec;
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    AcceptTeleportation, KeepAlive as ServerboundKeepAlive, MovePlayerStatusOnly, MovementFlags,
    PlayServerboundEntryPacket,
};
use ferrite_protocol::java_26_2::play::serverbound::teleport::TeleportAcknowledgement;
use ferrite_protocol::java_26_2::status::clientbound::codec as status_clientbound_codec;
use ferrite_protocol::java_26_2::status::clientbound::packet::{
    ServerStatus, StatusClientboundPacket, StatusDescription,
};
use ferrite_protocol::java_26_2::status::serverbound::codec as status_serverbound_codec;
use ferrite_protocol::java_26_2::status::serverbound::packet::StatusServerboundPacket;
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::known_pack::KnownPack;
use ferrite_protocol::java_26_2::value::nbt::NetworkNbt;
use ferrite_protocol::java_26_2::wire::compression::CompressionMode;
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::stream::{PacketStreamDecoder, PacketStreamEncoder};

const SERVER_SESSION_ID: u128 = 0x1234_5678_9abc_def0;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn core_pack() -> KnownPack {
    KnownPack::vanilla_core()
}

fn registry_projection() -> Vec<RegistryProjection> {
    let nbt = NetworkNbt::literal_component("entry").unwrap();
    SYNCHRONIZED_REGISTRY_IDENTITIES
        .iter()
        .enumerate()
        .map(|(index, identity)| RegistryProjection {
            registry: id(identity),
            entries: match index {
                0 => vec![RegistryProjectionEntry {
                    id: id("minecraft:plains"),
                    data: Some(nbt.clone()),
                    source_pack: Some(core_pack()),
                }],
                1 => vec![RegistryProjectionEntry {
                    id: id("minecraft:chat"),
                    data: Some(nbt.clone()),
                    source_pack: None,
                }],
                _ => Vec::new(),
            },
        })
        .collect()
}

fn configuration_snapshot() -> ConfigurationSnapshot {
    ConfigurationSnapshot::new(
        "Ferrite".to_owned(),
        BTreeSet::from([id("minecraft:vanilla")]),
        vec![core_pack()],
        registry_projection(),
        vec![RegistryTags {
            registry: id("minecraft:worldgen/biome"),
            tags: vec![TagDefinition {
                id: id("minecraft:is_overworld"),
                members: vec![0],
            }],
        }],
    )
    .unwrap()
}

fn settings(status: Option<ServerStatus>) -> ServerConnectionSettings {
    ServerConnectionSettings::with_required_defaults(
        status,
        configuration_snapshot(),
        DisconnectMessages::standard().unwrap(),
    )
}

fn frame(body: &[u8], compression: CompressionMode) -> Vec<u8> {
    PacketStreamEncoder::new(FrameLimits::default(), compression)
        .encode(body)
        .unwrap()
}

fn frame_body(bytes: &[u8], compression: CompressionMode) -> Vec<u8> {
    let mut decoder = PacketStreamDecoder::new(FrameLimits::default(), compression);
    decoder.push(bytes).unwrap();
    let body = decoder.next_packet().unwrap().unwrap();
    decoder.finish().unwrap();
    body
}

fn intention(intent: ClientIntention, protocol_version: i32) -> Vec<u8> {
    let body = handshake_codec::encode_packet(&ClientIntentionPacket {
        protocol_version,
        host: "mc.example".to_owned(),
        port: 25_565,
        intention: intent,
    })
    .unwrap();
    frame(&body, CompressionMode::Disabled)
}

fn complete_next(
    connection: &mut ServerConnection,
    compression: CompressionMode,
    now_millis: i64,
) -> (ConnectionState, &'static str, Vec<u8>) {
    let outbound = connection.take_outbound().unwrap();
    let state = outbound.state;
    let identity = outbound.identity;
    let body = frame_body(&outbound.bytes, compression);
    connection
        .outbound_sent(outbound.sequence, now_millis, false)
        .unwrap();
    (state, identity, body)
}

fn connection_at_configuration_without_compression() -> ServerConnection {
    let mut configured = settings(Some(ServerStatus::default()));
    configured.login_policy.compression_threshold = -1;
    let mut connection = ServerConnection::new(configured);
    connection
        .receive(&intention(ClientIntention::Login, 776), 0, false)
        .unwrap();
    connection.take_event().unwrap();
    let hello =
        login_serverbound_codec::encode_packet(&LoginServerboundPacket::Hello(LoginHello {
            name: "Player".to_owned(),
            supplied_profile_id: 0,
        }))
        .unwrap();
    connection
        .receive(&frame(&hello, CompressionMode::Disabled), 1, false)
        .unwrap();
    connection
        .tick(AdmissionSnapshot::allowed(), SERVER_SESSION_ID, 2, false)
        .unwrap();
    let finished = connection.take_outbound().unwrap();
    connection
        .outbound_sent(finished.sequence, 3, false)
        .unwrap();
    let acknowledgement =
        login_serverbound_codec::encode_packet(&LoginServerboundPacket::Acknowledged).unwrap();
    connection
        .receive(
            &frame(&acknowledgement, CompressionMode::Disabled),
            4,
            false,
        )
        .unwrap();
    assert!(matches!(
        connection.take_event(),
        Some(ServerConnectionEvent::ConfigurationStarted { .. })
    ));
    for _ in 0..3 {
        complete_next(&mut connection, CompressionMode::Disabled, 4);
    }
    connection
}

#[test]
fn configuration_snapshot_locks_registry_order_features_tags_and_pack_elision() {
    let snapshot = configuration_snapshot();
    let packets = snapshot.synchronization_packets(true);
    let ConfigurationClientboundPacket::RegistryData(first) = &packets[0] else {
        panic!("first synchronization packet must be registry data");
    };
    assert_eq!(first.registry, id("minecraft:worldgen/biome"));
    assert_eq!(first.entries[0].data, None);
    let ConfigurationClientboundPacket::RegistryData(second) = &packets[1] else {
        panic!("second synchronization packet must be registry data");
    };
    assert!(second.entries[0].data.is_some());
    assert!(matches!(
        packets.last(),
        Some(ConfigurationClientboundPacket::UpdateTags(_))
    ));
    let full_packets = snapshot.synchronization_packets(false);
    let ConfigurationClientboundPacket::RegistryData(full_first) = &full_packets[0] else {
        panic!("full projection must begin with registry data");
    };
    assert!(full_first.entries[0].data.is_some());

    let mut wrong_order = registry_projection();
    wrong_order.swap(0, 1);
    assert!(matches!(
        ConfigurationSnapshot::new(
            "Ferrite".to_owned(),
            BTreeSet::from([id("minecraft:vanilla")]),
            vec![core_pack()],
            wrong_order,
            Vec::new(),
        ),
        Err(ConfigurationSnapshotError::RegistryOrder { index: 0, .. })
    ));

    assert!(matches!(
        ConfigurationSnapshot::new(
            "Ferrite".to_owned(),
            BTreeSet::from([id("minecraft:unknown")]),
            vec![core_pack()],
            registry_projection(),
            Vec::new(),
        ),
        Err(ConfigurationSnapshotError::MissingVanillaFeature)
    ));
}

#[test]
fn status_trace_routes_fragmented_input_and_closes_only_after_exact_pong_is_sent() {
    let status = ServerStatus {
        description: StatusDescription::literal("Ferrite"),
        ..ServerStatus::default()
    };
    let mut connection = ServerConnection::new(settings(Some(status.clone())));
    let handshake = intention(ClientIntention::Status, -99);
    let split = handshake.len() / 2;
    connection.receive(&handshake[..split], 0, false).unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Handshake);
    connection.receive(&handshake[split..], 0, false).unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Status);
    assert_eq!(connection.serverbound_state(), ConnectionState::Status);
    assert_eq!(connection.clientbound_state(), ConnectionState::Status);
    assert!(matches!(
        connection.take_event(),
        Some(ServerConnectionEvent::Routed(_))
    ));

    let request =
        status_serverbound_codec::encode_packet(StatusServerboundPacket::Request).unwrap();
    connection
        .receive(&frame(&request, CompressionMode::Disabled), 1, false)
        .unwrap();
    let (_, identity, body) = complete_next(&mut connection, CompressionMode::Disabled, 2);
    assert_eq!(identity, "minecraft:status_response");
    assert_eq!(
        status_clientbound_codec::decode_packet(&body).unwrap(),
        StatusClientboundPacket::Response(status)
    );

    let token = i64::MIN + 77;
    let ping =
        status_serverbound_codec::encode_packet(StatusServerboundPacket::Ping(token)).unwrap();
    connection
        .receive(&frame(&ping, CompressionMode::Disabled), 3, false)
        .unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Closing);
    let outbound = connection.take_outbound().unwrap();
    assert_eq!(outbound.identity, "minecraft:pong_response");
    let body = frame_body(&outbound.bytes, CompressionMode::Disabled);
    assert_eq!(
        status_clientbound_codec::decode_packet(&body).unwrap(),
        StatusClientboundPacket::Pong(token)
    );
    assert_eq!(connection.stage(), ServerConnectionStage::Closing);
    connection
        .outbound_sent(outbound.sequence, 4, false)
        .unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Closed);
    assert_eq!(
        connection.take_event(),
        Some(ServerConnectionEvent::Closed(
            ConnectionCloseReason::StatusRequestHandled
        ))
    );
}

#[test]
fn wrong_version_refusal_switches_only_clientbound_login_and_flushes_before_close() {
    let messages = DisconnectMessages::standard().unwrap();
    let expected_reason = messages.outdated_client.clone();
    let mut configured = settings(Some(ServerStatus::default()));
    configured.disconnect_messages = messages;
    let mut connection = ServerConnection::new(configured);
    connection
        .receive(&intention(ClientIntention::Login, 753), 0, false)
        .unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Closing);
    assert_eq!(connection.clientbound_state(), ConnectionState::Login);
    assert_eq!(connection.serverbound_state(), ConnectionState::Handshake);
    assert!(matches!(
        connection.take_event(),
        Some(ServerConnectionEvent::Routed(_))
    ));

    let outbound = connection.take_outbound().unwrap();
    let body = frame_body(&outbound.bytes, CompressionMode::Disabled);
    assert_eq!(
        login_clientbound_codec::decode_packet(&body).unwrap(),
        LoginClientboundPacket::Disconnect(expected_reason)
    );
    connection
        .outbound_sent(outbound.sequence, 1, false)
        .unwrap();
    assert_eq!(
        connection.take_event(),
        Some(ServerConnectionEvent::Closed(
            ConnectionCloseReason::HandshakeRefused(LoginRefusal::OutdatedClient)
        ))
    );
}

#[test]
fn unavailable_status_closes_without_installing_serverbound_status_or_emitting_bytes() {
    let mut connection = ServerConnection::new(settings(None));
    connection
        .receive(&intention(ClientIntention::Status, i32::MIN), 0, false)
        .unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Closed);
    assert_eq!(connection.clientbound_state(), ConnectionState::Status);
    assert_eq!(connection.serverbound_state(), ConnectionState::Handshake);
    assert_eq!(connection.pending_outbound(), 0);
    assert!(matches!(
        connection.take_event(),
        Some(ServerConnectionEvent::Routed(_))
    ));
    assert_eq!(
        connection.take_event(),
        Some(ServerConnectionEvent::Closed(
            ConnectionCloseReason::StatusUnavailable
        ))
    );
}

#[test]
fn integrated_login_timeout_uses_the_post_increment_boundary_and_flushes_before_close() {
    let messages = DisconnectMessages::standard().unwrap();
    let expected_reason = messages.slow_login.clone();
    let mut configured = settings(Some(ServerStatus::default()));
    configured.disconnect_messages = messages;
    let mut connection = ServerConnection::new(configured);
    connection
        .receive(&intention(ClientIntention::Login, 776), 0, false)
        .unwrap();
    connection.take_event();

    for tick in 0..=600 {
        connection
            .tick(
                AdmissionSnapshot::allowed(),
                SERVER_SESSION_ID,
                i64::from(tick),
                false,
            )
            .unwrap();
        if tick < 600 {
            assert_eq!(connection.pending_outbound(), 0);
        }
    }
    assert_eq!(connection.stage(), ServerConnectionStage::Closing);
    let outbound = connection.take_outbound().unwrap();
    let body = frame_body(&outbound.bytes, CompressionMode::Disabled);
    assert_eq!(
        login_clientbound_codec::decode_packet(&body).unwrap(),
        LoginClientboundPacket::Disconnect(expected_reason)
    );
    connection
        .outbound_sent(outbound.sequence, 601, false)
        .unwrap();
    assert!(matches!(
        connection.take_event(),
        Some(ServerConnectionEvent::Closed(
            ConnectionCloseReason::LoginRejected(LoginDisconnect::SlowLogin)
        ))
    ));
}

#[test]
fn offline_login_configuration_and_play_boundary_preserve_every_directional_switch() {
    let mut connection = ServerConnection::new(settings(Some(ServerStatus::default())));
    connection
        .receive(&intention(ClientIntention::Login, 776), 10, false)
        .unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Login);
    connection.take_event().unwrap();

    let hello =
        login_serverbound_codec::encode_packet(&LoginServerboundPacket::Hello(LoginHello {
            name: "FerriteUser".to_owned(),
            supplied_profile_id: u128::MAX,
        }))
        .unwrap();
    connection
        .receive(&frame(&hello, CompressionMode::Disabled), 11, false)
        .unwrap();
    connection
        .tick(AdmissionSnapshot::allowed(), SERVER_SESSION_ID, 12, false)
        .unwrap();

    let compression_frame = connection.take_outbound().unwrap();
    assert_eq!(compression_frame.identity, "minecraft:login_compression");
    let compression_body = frame_body(&compression_frame.bytes, CompressionMode::Disabled);
    assert_eq!(
        login_clientbound_codec::decode_packet(&compression_body).unwrap(),
        LoginClientboundPacket::Compression(256)
    );

    let compressed = CompressionMode::enabled(256).unwrap();
    let acknowledgement =
        login_serverbound_codec::encode_packet(&LoginServerboundPacket::Acknowledged).unwrap();
    connection
        .receive(&frame(&acknowledgement, compressed), 13, false)
        .unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Login);
    connection
        .outbound_sent(compression_frame.sequence, 14, false)
        .unwrap();
    assert_eq!(connection.compression(), compressed);
    assert_eq!(connection.stage(), ServerConnectionStage::Login);

    let finished = connection.take_outbound().unwrap();
    assert_eq!(finished.identity, "minecraft:login_finished");
    let finished_body = frame_body(&finished.bytes, compressed);
    let LoginClientboundPacket::Finished(finished_packet) =
        login_clientbound_codec::decode_packet(&finished_body).unwrap()
    else {
        panic!("expected login finished");
    };
    assert_eq!(finished_packet.server_session_id, SERVER_SESSION_ID);
    assert_eq!(finished_packet.profile.name, "FerriteUser");
    connection
        .outbound_sent(finished.sequence, 15, false)
        .unwrap();

    assert_eq!(connection.stage(), ServerConnectionStage::Configuration);
    assert_eq!(
        connection.serverbound_state(),
        ConnectionState::Configuration
    );
    assert_eq!(
        connection.clientbound_state(),
        ConnectionState::Configuration
    );
    assert!(matches!(
        connection.take_event(),
        Some(ServerConnectionEvent::ConfigurationStarted { .. })
    ));

    let mut initial = Vec::new();
    for _ in 0..3 {
        let (_, _, body) = complete_next(&mut connection, compressed, 16);
        initial.push(configuration_clientbound_codec::decode_packet(&body).unwrap());
    }
    assert!(matches!(
        &initial[0],
        ConfigurationClientboundPacket::CustomPayload(ClientboundCustomPayload::Brand(brand))
            if brand == "Ferrite"
    ));
    assert!(matches!(
        &initial[1],
        ConfigurationClientboundPacket::UpdateEnabledFeatures(features)
            if features.contains(&id("minecraft:vanilla"))
    ));
    assert_eq!(
        initial[2],
        ConfigurationClientboundPacket::SelectKnownPacks(vec![core_pack()])
    );

    let latest_information = ClientInformation {
        language: "zh_cn".to_owned(),
        view_distance: 12,
        ..ClientInformation::default()
    };
    for packet in [
        ConfigurationServerboundPacket::CustomPayload(ServerboundCustomPayload::Brand(
            "vanilla".to_owned(),
        )),
        ConfigurationServerboundPacket::ClientInformation(latest_information.clone()),
        ConfigurationServerboundPacket::SelectKnownPacks(vec![core_pack()]),
    ] {
        let body = configuration_serverbound_codec::encode_packet(&packet).unwrap();
        connection
            .receive(&frame(&body, compressed), 20, false)
            .unwrap();
    }
    assert_eq!(
        connection.take_event(),
        Some(ServerConnectionEvent::RegistrySelection {
            selected_packs: vec![core_pack()],
            exact_offer_match: true,
        })
    );

    let mut synchronization = Vec::new();
    for _ in 0..=SYNCHRONIZED_REGISTRY_IDENTITIES.len() {
        let (_, _, body) = complete_next(&mut connection, compressed, 21);
        synchronization.push(configuration_clientbound_codec::decode_packet(&body).unwrap());
    }
    let ConfigurationClientboundPacket::RegistryData(first) = &synchronization[0] else {
        panic!("first synchronization packet must be registry data");
    };
    assert_eq!(first.entries[0].data, None);
    let ConfigurationClientboundPacket::RegistryData(second) = &synchronization[1] else {
        panic!("second synchronization packet must be registry data");
    };
    assert!(second.entries[0].data.is_some());
    assert!(matches!(
        synchronization.last(),
        Some(ConfigurationClientboundPacket::UpdateTags(_))
    ));

    connection.spawn_ready().unwrap();
    let (_, identity, body) = complete_next(&mut connection, compressed, 22);
    assert_eq!(identity, "minecraft:finish_configuration");
    assert_eq!(
        configuration_clientbound_codec::decode_packet(&body).unwrap(),
        ConfigurationClientboundPacket::FinishConfiguration
    );

    let finish = configuration_serverbound_codec::encode_packet(
        &ConfigurationServerboundPacket::FinishConfiguration,
    )
    .unwrap();
    connection
        .receive(&frame(&finish, compressed), 23, false)
        .unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::InstallingPlay);
    assert_eq!(connection.clientbound_state(), ConnectionState::Play);
    assert_eq!(
        connection.serverbound_state(),
        ConnectionState::Configuration
    );
    let Some(ServerConnectionEvent::PlayInstallationRequested(request)) = connection.take_event()
    else {
        panic!("play installation must cross the semantic boundary");
    };
    assert_eq!(request.profile.name, "FerriteUser");
    assert_eq!(request.client_information, latest_information);
    assert!(!request.transferred);

    connection.complete_play_installation().unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Play);
    assert_eq!(connection.clientbound_state(), ConnectionState::Play);
    assert_eq!(connection.serverbound_state(), ConnectionState::Play);

    let mut play_registries = PlayRegistries::default();
    play_registries.insert(id(BIOME), vec![id("minecraft:plains")]);
    connection
        .enqueue_play(
            &[PlayClientboundPacket::Terrain(
                TerrainPacket::SetChunkCacheCenter(ChunkCoordinate { x: -2, z: 7 }),
            )],
            &play_registries,
        )
        .unwrap();
    let (state, identity, body) = complete_next(&mut connection, compressed, 24);
    assert_eq!(state, ConnectionState::Play);
    assert_eq!(identity, "minecraft:set_chunk_cache_center");
    assert_eq!(body, vec![94, 254, 255, 255, 255, 15, 7]);

    assert_eq!(
        connection
            .issue_player_correction(
                Vector3 {
                    x: 8.5,
                    y: 65.0,
                    z: 8.5,
                },
                10.0,
                20.0,
                &play_registries,
            )
            .unwrap(),
        1
    );
    let (_, identity, body) = complete_next(&mut connection, compressed, 25);
    assert_eq!(identity, "minecraft:player_position");
    assert_eq!(body[0], 72);

    let movement = play_serverbound_codec::encode_packet(
        PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
            flags: MovementFlags {
                on_ground: true,
                horizontal_collision: false,
            },
        }),
    )
    .unwrap();
    connection
        .receive(&frame(&movement, compressed), 26, false)
        .unwrap();
    assert_eq!(
        connection.take_event(),
        Some(ServerConnectionEvent::PlayPacket {
            packet: PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
                flags: MovementFlags {
                    on_ground: true,
                    horizontal_collision: false,
                },
            }),
            teleport_pending: true,
        })
    );

    let acknowledgement = play_serverbound_codec::encode_packet(
        PlayServerboundEntryPacket::AcceptTeleportation(AcceptTeleportation { challenge: 1 }),
    )
    .unwrap();
    connection
        .receive(&frame(&acknowledgement, compressed), 27, false)
        .unwrap();
    assert!(matches!(
        connection.take_event(),
        Some(ServerConnectionEvent::TeleportAcknowledged(
            TeleportAcknowledgement::Accepted { .. }
        ))
    ));

    connection
        .tick(
            AdmissionSnapshot::allowed(),
            SERVER_SESSION_ID,
            1_000,
            false,
        )
        .unwrap();
    connection
        .tick(
            AdmissionSnapshot::allowed(),
            SERVER_SESSION_ID,
            16_000,
            false,
        )
        .unwrap();
    let (_, identity, body) = complete_next(&mut connection, compressed, 16_000);
    assert_eq!(identity, "minecraft:keep_alive");
    assert_eq!(body[0], 44);
    let echo = play_serverbound_codec::encode_packet(PlayServerboundEntryPacket::KeepAlive(
        ServerboundKeepAlive { challenge: 16_000 },
    ))
    .unwrap();
    connection
        .receive(&frame(&echo, compressed), 16_040, false)
        .unwrap();
    assert_eq!(
        connection.take_event(),
        Some(ServerConnectionEvent::LatencyUpdated { latency_millis: 10 })
    );

    let duplicate = play_serverbound_codec::encode_packet(
        PlayServerboundEntryPacket::AcceptTeleportation(AcceptTeleportation { challenge: 1 }),
    )
    .unwrap();
    connection
        .receive(&frame(&duplicate, compressed), 16_041, false)
        .unwrap();
    assert_eq!(connection.stage(), ServerConnectionStage::Closing);
    let (_, identity, _) = complete_next(&mut connection, compressed, 16_042);
    assert_eq!(identity, "minecraft:disconnect");
    assert_eq!(
        connection.take_event(),
        Some(ServerConnectionEvent::Closed(ConnectionCloseReason::Play(
            ferrite_protocol::java_26_2::connection::output::PlayDisconnectReason::InvalidPlayerMovement
        )))
    );
}

#[test]
fn configuration_liveness_is_integrated_without_advancing_its_current_task() {
    let mut connection = connection_at_configuration_without_compression();
    connection
        .tick(
            AdmissionSnapshot::allowed(),
            SERVER_SESSION_ID,
            15_003,
            false,
        )
        .unwrap();
    assert_eq!(connection.pending_outbound(), 0);
    connection
        .tick(
            AdmissionSnapshot::allowed(),
            SERVER_SESSION_ID,
            15_004,
            false,
        )
        .unwrap();
    let (_, identity, body) = complete_next(&mut connection, CompressionMode::Disabled, 15_005);
    assert_eq!(identity, "minecraft:keep_alive");
    assert_eq!(
        configuration_clientbound_codec::decode_packet(&body).unwrap(),
        ConfigurationClientboundPacket::KeepAlive(15_004)
    );

    let echo = configuration_serverbound_codec::encode_packet(
        &ConfigurationServerboundPacket::KeepAlive(15_004),
    )
    .unwrap();
    connection
        .receive(&frame(&echo, CompressionMode::Disabled), 15_008, false)
        .unwrap();
    assert_eq!(
        connection.take_event(),
        Some(ServerConnectionEvent::LatencyUpdated { latency_millis: 1 })
    );
    assert_eq!(connection.stage(), ServerConnectionStage::Configuration);
}

#[test]
fn early_configuration_finish_faults_without_crossing_the_play_boundary() {
    let mut connection = connection_at_configuration_without_compression();
    let early_finish = configuration_serverbound_codec::encode_packet(
        &ConfigurationServerboundPacket::FinishConfiguration,
    )
    .unwrap();
    assert!(
        connection
            .receive(&frame(&early_finish, CompressionMode::Disabled), 5, false,)
            .is_err()
    );
    assert_eq!(connection.stage(), ServerConnectionStage::Faulted);
    assert_eq!(connection.pending_outbound(), 0);
    assert_eq!(
        connection.clientbound_state(),
        ConnectionState::Configuration
    );
    assert_eq!(
        connection.serverbound_state(),
        ConnectionState::Configuration
    );
}
