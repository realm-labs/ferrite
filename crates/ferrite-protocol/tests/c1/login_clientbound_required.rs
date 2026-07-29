use ferrite_protocol::java_26_2::login::clientbound::codec::{
    LoginClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::login::clientbound::packet::{
    LoginClientboundPacket, LoginFinished,
};
use ferrite_protocol::java_26_2::login::clientbound::projection::{
    LoginClientAction, LoginClientProjection, LoginClientProjectionError, LoginClientStage,
};
use ferrite_protocol::java_26_2::login::component_json::{
    LoginDisconnectReason, LoginDisconnectReasonError,
};
use ferrite_protocol::java_26_2::login::profile::{GameProfile, ProfileProperty};
use ferrite_protocol::java_26_2::wire::compression::{
    CompressionMode, encode_packet as encode_wire,
};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

const PROFILE_ID: u128 = 0xa01e_3843_e521_3998_958a_f459_800e_4d11;
const SESSION_ID: u128 = 0;

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

fn finished(properties: Vec<ProfileProperty>) -> LoginClientboundPacket {
    LoginClientboundPacket::Finished(LoginFinished {
        profile: GameProfile {
            id: PROFILE_ID,
            name: "Player".to_owned(),
            properties,
        },
        server_session_id: SESSION_ID,
    })
}

#[test]
fn matches_every_locked_required_login_clientbound_golden() {
    let compression = LoginClientboundPacket::Compression(256);
    let compression_body = encode_packet(&compression).unwrap();
    let compression_frame = encode_wire(
        &compression_body,
        CompressionMode::Disabled,
        FrameLimits::default(),
    )
    .unwrap();
    assert_eq!(compression_frame, hex("03038002"));
    assert_eq!(decode_packet(&compression_body).unwrap(), compression);

    let terminal = finished(Vec::new());
    let terminal_body = encode_packet(&terminal).unwrap();
    assert_eq!(
        encode_wire(
            &terminal_body,
            CompressionMode::Disabled,
            FrameLimits::default()
        )
        .unwrap(),
        hex("2902a01e3843e5213998958af459800e4d1106506c617965720000000000000000000000000000000000")
    );
    assert_eq!(
        encode_wire(
            &terminal_body,
            CompressionMode::enabled(256).unwrap(),
            FrameLimits::default()
        )
        .unwrap(),
        hex(
            "2a0002a01e3843e5213998958af459800e4d1106506c617965720000000000000000000000000000000000"
        )
    );
    assert_eq!(decode_packet(&terminal_body).unwrap(), terminal);
}

#[test]
fn disconnect_uses_bounded_lenient_component_json() {
    let reason = LoginDisconnectReason::literal("No room").unwrap();
    let packet = LoginClientboundPacket::Disconnect(reason.clone());
    assert_eq!(
        decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
        packet
    );
    assert_eq!(reason.as_json(), "\"No room\"");

    assert_eq!(
        LoginDisconnectReason::from_json("null"),
        Err(LoginDisconnectReasonError::InvalidComponentShape)
    );
    assert_eq!(
        LoginDisconnectReason::from_json("{"),
        Err(LoginDisconnectReasonError::MalformedJson)
    );

    let mut invalid = WireWriter::new(1_000_000);
    invalid.write_var_i32(0).unwrap();
    invalid.write_utf("7", 262_144).unwrap();
    assert!(matches!(
        decode_packet(invalid.as_slice()),
        Err(LoginClientboundCodecError::InvalidDisconnectReason(_))
    ));

    let too_long = LoginClientboundPacket::Disconnect(
        LoginDisconnectReason::literal(&"a".repeat(262_144)).unwrap(),
    );
    assert!(matches!(
        encode_packet(&too_long),
        Err(LoginClientboundCodecError::Wire(_))
    ));
}

#[test]
fn profile_bounds_nullable_signature_and_separate_session_uuid_are_exact() {
    let signed = ProfileProperty {
        name: "x".repeat(64),
        value: "v".repeat(32_767),
        signature: Some("s".repeat(1_024)),
    };
    let unsigned = ProfileProperty {
        name: "skin".to_owned(),
        value: "value".to_owned(),
        signature: None,
    };
    let boundary = finished(
        std::iter::once(signed)
            .chain(std::iter::repeat_n(unsigned, 15))
            .collect(),
    );
    assert_eq!(
        decode_packet(&encode_packet(&boundary).unwrap()).unwrap(),
        boundary
    );

    let too_many = finished(vec![
        ProfileProperty {
            name: "n".to_owned(),
            value: "v".to_owned(),
            signature: None,
        };
        17
    ]);
    assert!(matches!(
        encode_packet(&too_many),
        Err(LoginClientboundCodecError::Wire(_))
    ));

    let decoded = decode_packet(&encode_packet(&finished(Vec::new())).unwrap()).unwrap();
    let LoginClientboundPacket::Finished(decoded) = decoded else {
        panic!("expected login finished");
    };
    assert_eq!(decoded.profile.id, PROFILE_ID);
    assert_eq!(decoded.server_session_id, SESSION_ID);
    assert_ne!(decoded.profile.id, decoded.server_session_id);
}

#[test]
fn every_profile_string_limit_is_enforced_in_utf16_code_units() {
    let too_long_name = LoginClientboundPacket::Finished(LoginFinished {
        profile: GameProfile {
            id: PROFILE_ID,
            name: "😀".repeat(9),
            properties: Vec::new(),
        },
        server_session_id: SESSION_ID,
    });
    assert!(encode_packet(&too_long_name).is_err());

    for property in [
        ProfileProperty {
            name: "n".repeat(65),
            value: "v".to_owned(),
            signature: None,
        },
        ProfileProperty {
            name: "n".to_owned(),
            value: "😀".repeat(16_384),
            signature: None,
        },
        ProfileProperty {
            name: "n".to_owned(),
            value: "v".to_owned(),
            signature: Some("😀".repeat(513)),
        },
    ] {
        assert!(encode_packet(&finished(vec![property])).is_err());
    }
}

#[test]
fn optional_and_unknown_packet_ids_fail_closed() {
    for id in [1, 4, 5] {
        assert!(matches!(
            decode_packet(&[id]),
            Err(LoginClientboundCodecError::UnsupportedPacketIdentity { .. })
        ));
    }
    assert_eq!(
        decode_packet(&[6]),
        Err(LoginClientboundCodecError::UnknownPacketId { id: 6 })
    );
}

#[test]
fn compression_callback_precedes_the_terminal_login_transition() {
    let mut client = LoginClientProjection::new();
    let action = client
        .apply(LoginClientboundPacket::Compression(256))
        .unwrap();
    assert!(matches!(
        action,
        LoginClientAction::InstallCompressionAfterCurrentPacket(threshold)
            if threshold.get() == 256
    ));
    assert_eq!(client.compression(), CompressionMode::Disabled);
    assert_eq!(
        client.apply(finished(Vec::new())),
        Err(LoginClientProjectionError::CompressionCallbackPending)
    );

    client.compression_installed().unwrap();
    assert_eq!(client.compression().threshold(), Some(256));
    assert!(matches!(
        client.apply(finished(Vec::new())).unwrap(),
        LoginClientAction::InstallConfigurationClientbound(_)
    ));
    assert_eq!(client.stage(), LoginClientStage::LoginFinishedReceived);
}

#[test]
fn terminal_transition_installs_each_codec_on_the_locked_side_of_acknowledgement() {
    let mut client = LoginClientProjection::new();
    client.apply(finished(Vec::new())).unwrap();
    assert_eq!(
        client.configuration_clientbound_installed().unwrap(),
        LoginClientAction::SendAcknowledgementUnderLogin
    );
    assert_eq!(
        client.acknowledgement_sent().unwrap(),
        LoginClientAction::InstallConfigurationServerbound
    );
    client.configuration_serverbound_installed().unwrap();
    assert_eq!(client.stage(), LoginClientStage::Configuration);
    assert!(matches!(
        client.acknowledgement_sent(),
        Err(LoginClientProjectionError::UnexpectedStage { .. })
    ));
}

#[test]
fn compression_minus_one_is_disabled_by_omission_and_zero_is_valid() {
    let mut disabled = LoginClientProjection::new();
    assert_eq!(
        disabled.apply(LoginClientboundPacket::Compression(-1)),
        Err(LoginClientProjectionError::NegativeCompressionThreshold { threshold: -1 })
    );
    assert_eq!(disabled.compression(), CompressionMode::Disabled);

    let mut zero = LoginClientProjection::new();
    assert!(matches!(
        zero.apply(LoginClientboundPacket::Compression(0)).unwrap(),
        LoginClientAction::InstallCompressionAfterCurrentPacket(threshold)
            if threshold.get() == 0
    ));
    zero.compression_installed().unwrap();
    assert_eq!(zero.compression().threshold(), Some(0));
}

#[test]
fn disconnect_is_terminal_from_every_live_login_stage() {
    let reason = LoginDisconnectReason::literal("Denied").unwrap();
    let mut client = LoginClientProjection::new();
    assert_eq!(
        client
            .apply(LoginClientboundPacket::Disconnect(reason.clone()))
            .unwrap(),
        LoginClientAction::Disconnect(reason)
    );
    assert_eq!(client.stage(), LoginClientStage::Disconnected);
    assert_eq!(
        client.apply(LoginClientboundPacket::Compression(0)),
        Err(LoginClientProjectionError::TerminalStage)
    );
}
