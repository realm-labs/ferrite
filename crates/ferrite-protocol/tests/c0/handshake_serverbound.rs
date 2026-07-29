use ferrite_protocol::java_26_2::handshake::codec::{
    HandshakeCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::handshake::packet::{ClientIntention, ClientIntentionPacket};
use ferrite_protocol::java_26_2::handshake::transition::{
    HandshakePolicy, HandshakeSession, HandshakeStep, HandshakeTransitionError, LoginRefusal,
};
use ferrite_protocol::java_26_2::wire::frame::{FrameLimits, encode_frame};
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

fn packet(
    protocol_version: i32,
    host: impl Into<String>,
    port: u16,
    intention: ClientIntention,
) -> ClientIntentionPacket {
    ClientIntentionPacket {
        protocol_version,
        host: host.into(),
        port,
        intention,
    }
}

#[test]
fn matches_the_locked_status_intention_golden() {
    let intention = packet(776, "localhost", 25_565, ClientIntention::Status);
    let body = encode_packet(&intention).unwrap();
    assert_eq!(
        encode_frame(&body, FrameLimits::default()).unwrap(),
        [
            0x10, 0x00, 0x88, 0x06, 0x09, b'l', b'o', b'c', b'a', b'l', b'h', b'o', b's', b't',
            0x63, 0xdd, 0x01,
        ]
    );
    assert_eq!(decode_packet(&body).unwrap(), intention);
}

#[test]
fn host_port_and_lossy_utf_boundaries_are_exact() {
    for host in ["a".repeat(255), "€".repeat(255)] {
        let intention = packet(-1, host, u16::MAX, ClientIntention::Status);
        assert_eq!(
            decode_packet(&encode_packet(&intention).unwrap()).unwrap(),
            intention
        );
    }
    assert!(encode_packet(&packet(776, "a".repeat(256), 0, ClientIntention::Status)).is_err());
    assert!(encode_packet(&packet(776, "€".repeat(256), 0, ClientIntention::Status)).is_err());

    for port in [0, u16::MAX] {
        let intention = packet(776, "", port, ClientIntention::Status);
        assert_eq!(
            decode_packet(&encode_packet(&intention).unwrap())
                .unwrap()
                .port,
            port
        );
    }

    let mut malformed = WireWriter::new(32);
    malformed.write_var_i32(0).unwrap();
    malformed.write_var_i32(776).unwrap();
    malformed.write_var_i32(1).unwrap();
    malformed.write_u8(0xff).unwrap();
    malformed.write_u16(0).unwrap();
    malformed.write_var_i32(1).unwrap();
    assert_eq!(decode_packet(malformed.as_slice()).unwrap().host, "�");
}

#[test]
fn illegal_intents_unknown_ids_and_trailing_data_fail_closed() {
    for intention in [-1, 0, 4] {
        let mut writer = WireWriter::new(32);
        writer.write_var_i32(0).unwrap();
        writer.write_var_i32(776).unwrap();
        writer.write_utf("", 255).unwrap();
        writer.write_u16(0).unwrap();
        writer.write_var_i32(intention).unwrap();
        assert_eq!(
            decode_packet(writer.as_slice()),
            Err(HandshakeCodecError::InvalidIntention { id: intention })
        );
    }
    assert!(matches!(
        decode_packet(&[1]),
        Err(HandshakeCodecError::UnknownPacketId { id: 1 })
    ));
    let mut trailing =
        encode_packet(&packet(776, "localhost", 25_565, ClientIntention::Status)).unwrap();
    trailing.push(0);
    assert!(decode_packet(&trailing).is_err());
}

#[test]
fn status_protocol_is_opaque_and_availability_controls_only_listener_installation() {
    for protocol_version in [-1, 775, 776, 777] {
        let mut session = HandshakeSession::new(HandshakePolicy::default());
        let plan = session
            .route(packet(
                protocol_version,
                "status.example",
                25_565,
                ClientIntention::Status,
            ))
            .unwrap();
        assert_eq!(
            plan.steps,
            vec![
                HandshakeStep::InstallStatusClientbound,
                HandshakeStep::InstallStatusServerbound,
            ]
        );
        assert_eq!(plan.routing_context.protocol_version, protocol_version);
    }

    for policy in [
        HandshakePolicy {
            status_replies_enabled: false,
            ..HandshakePolicy::default()
        },
        HandshakePolicy {
            cached_status_available: false,
            ..HandshakePolicy::default()
        },
    ] {
        let mut session = HandshakeSession::new(policy);
        assert_eq!(
            session
                .route(packet(776, "", 0, ClientIntention::Status))
                .unwrap()
                .steps,
            vec![
                HandshakeStep::InstallStatusClientbound,
                HandshakeStep::Close,
            ]
        );
    }
}

#[test]
fn login_version_classification_and_transfer_gate_preserve_step_order() {
    let cases = [
        (
            753,
            vec![
                HandshakeStep::InstallLoginClientbound,
                HandshakeStep::SendLoginDisconnect(LoginRefusal::OutdatedClient),
                HandshakeStep::Close,
            ],
        ),
        (
            754,
            vec![
                HandshakeStep::InstallLoginClientbound,
                HandshakeStep::SendLoginDisconnect(LoginRefusal::IncompatibleVersion),
                HandshakeStep::Close,
            ],
        ),
        (
            777,
            vec![
                HandshakeStep::InstallLoginClientbound,
                HandshakeStep::SendLoginDisconnect(LoginRefusal::IncompatibleVersion),
                HandshakeStep::Close,
            ],
        ),
        (
            776,
            vec![
                HandshakeStep::InstallLoginClientbound,
                HandshakeStep::InstallLoginServerbound { transferred: false },
            ],
        ),
    ];
    for (protocol_version, expected) in cases {
        let mut session = HandshakeSession::new(HandshakePolicy::default());
        assert_eq!(
            session
                .route(packet(
                    protocol_version,
                    "login.example",
                    25_565,
                    ClientIntention::Login
                ))
                .unwrap()
                .steps,
            expected
        );
    }

    let mut disabled = HandshakeSession::new(HandshakePolicy::default());
    assert_eq!(
        disabled
            .route(packet(
                776,
                "transfer.example",
                25_565,
                ClientIntention::Transfer
            ))
            .unwrap()
            .steps,
        vec![
            HandshakeStep::InstallLoginClientbound,
            HandshakeStep::SendLoginDisconnect(LoginRefusal::TransfersDisabled),
            HandshakeStep::Close,
        ]
    );

    let mut enabled = HandshakeSession::new(HandshakePolicy {
        transfers_enabled: true,
        ..HandshakePolicy::default()
    });
    assert_eq!(
        enabled
            .route(packet(
                776,
                "transfer.example",
                25_565,
                ClientIntention::Transfer
            ))
            .unwrap()
            .steps,
        vec![
            HandshakeStep::InstallLoginClientbound,
            HandshakeStep::InstallLoginServerbound { transferred: true },
        ]
    );
}

#[test]
fn handshake_session_is_terminal_after_one_complete_intention() {
    let mut session = HandshakeSession::new(HandshakePolicy::default());
    session
        .route(packet(776, "", 0, ClientIntention::Status))
        .unwrap();
    assert_eq!(
        session.route(packet(776, "", 0, ClientIntention::Login)),
        Err(HandshakeTransitionError::HandshakeAlreadyComplete)
    );
}
