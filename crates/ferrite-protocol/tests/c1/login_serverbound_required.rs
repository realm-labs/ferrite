use ferrite_protocol::java_26_2::login::clientbound::packet::LoginFinished;
use ferrite_protocol::java_26_2::login::component_json::LoginDisconnectReason;
use ferrite_protocol::java_26_2::login::serverbound::codec::{
    LoginServerboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::login::serverbound::packet::{LoginHello, LoginServerboundPacket};
use ferrite_protocol::java_26_2::login::serverbound::session::{
    AdmissionSnapshot, ConfigurationTransitionStep, LoginDisconnect, LoginPolicy,
    LoginServerAction, LoginServerSession, LoginServerSessionError, LoginServerStage,
    ServerSessionIdPool, ServerSessionIdPoolError, offline_player_id,
};
use ferrite_protocol::java_26_2::wire::compression::{
    CompressionMode, encode_packet as encode_wire,
};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;

const PLAYER_ID: u128 = 0xa01e_3843_e521_3998_958a_f459_800e_4d11;
const SESSION_ID: u128 = 0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00;

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

fn hello(name: &str, supplied_profile_id: u128) -> LoginServerboundPacket {
    LoginServerboundPacket::Hello(LoginHello {
        name: name.to_owned(),
        supplied_profile_id,
    })
}

fn start(name: &str, policy: LoginPolicy) -> LoginServerSession {
    let mut session = LoginServerSession::new(policy);
    assert_eq!(
        session.apply(hello(name, u128::MAX)).unwrap(),
        LoginServerAction::None
    );
    session
}

fn assert_finished(action: LoginServerAction, expected_session_id: u128) -> LoginFinished {
    let LoginServerAction::SendFinished(finished) = action else {
        panic!("expected terminal login finished");
    };
    assert_eq!(finished.server_session_id, expected_session_id);
    finished
}

#[test]
fn matches_every_locked_required_login_serverbound_golden() {
    let hello = hello("Player", 0);
    let hello_body = encode_packet(&hello).unwrap();
    assert_eq!(
        encode_wire(
            &hello_body,
            CompressionMode::Disabled,
            FrameLimits::default()
        )
        .unwrap(),
        hex("180006506c6179657200000000000000000000000000000000")
    );
    assert_eq!(decode_packet(&hello_body).unwrap(), hello);

    let acknowledgement = LoginServerboundPacket::Acknowledged;
    let acknowledgement_body = encode_packet(&acknowledgement).unwrap();
    assert_eq!(
        encode_wire(
            &acknowledgement_body,
            CompressionMode::Disabled,
            FrameLimits::default()
        )
        .unwrap(),
        hex("0103")
    );
    assert_eq!(
        encode_wire(
            &acknowledgement_body,
            CompressionMode::enabled(256).unwrap(),
            FrameLimits::default()
        )
        .unwrap(),
        hex("020003")
    );
    assert_eq!(
        decode_packet(&acknowledgement_body).unwrap(),
        acknowledgement
    );
}

#[test]
fn codec_and_handler_apply_the_distinct_name_boundaries() {
    for accepted in ["", "0123456789ABCDEF"] {
        let packet = hello(accepted, 0);
        assert_eq!(
            decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
            packet
        );
        assert_eq!(
            LoginServerSession::new(LoginPolicy::default())
                .apply(packet)
                .unwrap(),
            LoginServerAction::None
        );
    }
    assert!(encode_packet(&hello("0123456789ABCDEFG", 0)).is_err());

    for rejected in ["has space", "\u{7f}", "\u{1f}", "玩家"] {
        let mut session = LoginServerSession::new(LoginPolicy::default());
        assert_eq!(
            session.apply(hello(rejected, 0)).unwrap(),
            LoginServerAction::Disconnect(LoginDisconnect::InvalidName)
        );
        assert_eq!(session.stage(), LoginServerStage::Disconnected);
    }
}

#[test]
fn offline_uuid_uses_exact_case_sensitive_java_uuid_v3_input() {
    assert_eq!(offline_player_id("Player"), PLAYER_ID);
    assert_ne!(offline_player_id("Player"), offline_player_id("player"));

    for supplied in [0, u128::MAX, PLAYER_ID] {
        let mut session = LoginServerSession::new(LoginPolicy {
            compression_threshold: -1,
            ..LoginPolicy::default()
        });
        session.apply(hello("Player", supplied)).unwrap();
        assert_eq!(session.profile().unwrap().id, offline_player_id("Player"));
        assert!(session.profile().unwrap().properties.is_empty());
    }
}

#[test]
fn admission_denial_and_intended_uuid_mismatch_disconnect_before_finished() {
    let reason = LoginDisconnectReason::literal("Banned").unwrap();
    let mut denied = start("Player", LoginPolicy::default());
    assert_eq!(
        denied
            .tick(
                AdmissionSnapshot {
                    policy_reason: Some(reason.clone()),
                    duplicate_active: false,
                },
                SESSION_ID,
            )
            .unwrap(),
        LoginServerAction::Disconnect(LoginDisconnect::AdmissionPolicy(reason))
    );
    assert_eq!(denied.stage(), LoginServerStage::Disconnected);

    let mut mismatch = start(
        "Player",
        LoginPolicy {
            intended_profile_id: Some(7),
            ..LoginPolicy::default()
        },
    );
    assert_eq!(
        mismatch
            .tick(AdmissionSnapshot::allowed(), SESSION_ID)
            .unwrap(),
        LoginServerAction::Disconnect(LoginDisconnect::IntendedProfileMismatch {
            intended: 7,
            normalized: PLAYER_ID,
        })
    );
}

#[test]
fn duplicate_identity_is_disconnected_once_and_login_waits_for_departure() {
    let mut session = start(
        "Player",
        LoginPolicy {
            compression_threshold: -1,
            ..LoginPolicy::default()
        },
    );
    let duplicate = AdmissionSnapshot {
        policy_reason: None,
        duplicate_active: true,
    };
    assert_eq!(
        session.tick(duplicate.clone(), SESSION_ID).unwrap(),
        LoginServerAction::DisconnectExistingAndWait {
            profile_id: PLAYER_ID
        }
    );
    assert_eq!(
        session.tick(duplicate, SESSION_ID).unwrap(),
        LoginServerAction::None
    );
    assert_eq!(
        assert_finished(
            session
                .tick(AdmissionSnapshot::allowed(), SESSION_ID)
                .unwrap(),
            SESSION_ID,
        )
        .profile
        .id,
        PLAYER_ID
    );
}

#[test]
fn compression_is_sent_raw_and_installed_only_by_its_completion_callback() {
    let mut session = start("Player", LoginPolicy::default());
    let action = session
        .tick(AdmissionSnapshot::allowed(), SESSION_ID)
        .unwrap();
    assert!(matches!(
        action,
        LoginServerAction::SendCompressionUncompressed(threshold)
            if threshold.get() == 256
    ));
    assert_eq!(session.stage(), LoginServerStage::CompressionSendPending);
    assert_eq!(session.compression(), CompressionMode::Disabled);
    assert!(matches!(
        session.apply(LoginServerboundPacket::Acknowledged),
        Err(LoginServerSessionError::UnexpectedStage { .. })
    ));

    let mut session = start("Player", LoginPolicy::default());
    session
        .tick(AdmissionSnapshot::allowed(), SESSION_ID)
        .unwrap();
    let finished = assert_finished(
        session.compression_send_completed(SESSION_ID).unwrap(),
        SESSION_ID,
    );
    assert_eq!(finished.profile.id, PLAYER_ID);
    assert_eq!(session.compression().threshold(), Some(256));
    assert_eq!(session.stage(), LoginServerStage::ProtocolSwitching);
}

#[test]
fn disabled_and_memory_connection_paths_skip_compression() {
    for policy in [
        LoginPolicy {
            compression_threshold: -1,
            ..LoginPolicy::default()
        },
        LoginPolicy {
            memory_connection: true,
            ..LoginPolicy::default()
        },
    ] {
        let mut session = start("Player", policy);
        assert_finished(
            session
                .tick(AdmissionSnapshot::allowed(), SESSION_ID)
                .unwrap(),
            SESSION_ID,
        );
        assert_eq!(session.compression(), CompressionMode::Disabled);
    }

    let mut zero = start(
        "Player",
        LoginPolicy {
            compression_threshold: 0,
            ..LoginPolicy::default()
        },
    );
    assert!(matches!(
        zero.tick(AdmissionSnapshot::allowed(), SESSION_ID)
            .unwrap(),
        LoginServerAction::SendCompressionUncompressed(threshold)
            if threshold.get() == 0
    ));
}

#[test]
fn acknowledgement_commits_the_ordered_configuration_transition_once() {
    let mut session = start(
        "Player",
        LoginPolicy {
            compression_threshold: -1,
            ..LoginPolicy::default()
        },
    );
    assert_finished(
        session
            .tick(AdmissionSnapshot::allowed(), SESSION_ID)
            .unwrap(),
        SESSION_ID,
    );
    let LoginServerAction::BeginConfiguration(transition) =
        session.apply(LoginServerboundPacket::Acknowledged).unwrap()
    else {
        panic!("expected configuration transition");
    };
    assert_eq!(transition.profile.id, PLAYER_ID);
    assert_eq!(
        transition.steps,
        vec![
            ConfigurationTransitionStep::InstallConfigurationClientbound,
            ConfigurationTransitionStep::BuildNormalizedConnectionCookie,
            ConfigurationTransitionStep::InstallConfigurationServerbound,
            ConfigurationTransitionStep::StartConfigurationTasks,
        ]
    );
    assert_eq!(session.stage(), LoginServerStage::Accepted);
    assert!(matches!(
        session.apply(LoginServerboundPacket::Acknowledged),
        Err(LoginServerSessionError::TerminalStage { .. })
    ));
}

#[test]
fn duplicate_hello_and_early_acknowledgement_are_terminal_faults() {
    let mut duplicate = start("Player", LoginPolicy::default());
    assert!(matches!(
        duplicate.apply(hello("Player", 0)),
        Err(LoginServerSessionError::UnexpectedStage { .. })
    ));
    assert_eq!(duplicate.stage(), LoginServerStage::Disconnected);

    let mut early = LoginServerSession::new(LoginPolicy::default());
    assert!(matches!(
        early.apply(LoginServerboundPacket::Acknowledged),
        Err(LoginServerSessionError::UnexpectedStage { .. })
    ));
    assert_eq!(early.stage(), LoginServerStage::Disconnected);
}

#[test]
fn prior_counter_value_600_triggers_the_slow_login_disconnect() {
    let mut session = LoginServerSession::new(LoginPolicy::default());
    for expected in 1..=600 {
        assert_eq!(
            session
                .tick(AdmissionSnapshot::allowed(), SESSION_ID)
                .unwrap(),
            LoginServerAction::None
        );
        assert_eq!(session.tick_counter(), expected);
    }
    assert_eq!(
        session
            .tick(AdmissionSnapshot::allowed(), SESSION_ID)
            .unwrap(),
        LoginServerAction::Disconnect(LoginDisconnect::SlowLogin)
    );
    assert_eq!(session.tick_counter(), 601);
    assert_eq!(session.stage(), LoginServerStage::Disconnected);
}

#[test]
fn optional_and_unknown_packet_ids_fail_closed() {
    for id in [1, 2, 4] {
        assert!(matches!(
            decode_packet(&[id]),
            Err(LoginServerboundCodecError::UnsupportedPacketIdentity { .. })
        ));
    }
    assert_eq!(
        decode_packet(&[5]),
        Err(LoginServerboundCodecError::UnknownPacketId { id: 5 })
    );
}

#[test]
fn concurrent_connections_share_one_session_id_until_the_pool_drains() {
    let mut pool = ServerSessionIdPool::default();
    assert_eq!(pool.acquire(11), 11);
    assert_eq!(pool.acquire(22), 11);
    assert_eq!(pool.active_connections(), 2);
    pool.release().unwrap();
    assert_eq!(pool.acquire(33), 11);
    pool.release().unwrap();
    pool.release().unwrap();
    assert_eq!(pool.active_connections(), 0);
    assert_eq!(pool.acquire(44), 44);

    let mut empty = ServerSessionIdPool::default();
    assert_eq!(
        empty.release(),
        Err(ServerSessionIdPoolError::NoActiveConnection)
    );
}
