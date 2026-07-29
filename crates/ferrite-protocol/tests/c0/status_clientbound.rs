use ferrite_protocol::java_26_2::status::clientbound::codec::{
    StatusClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::status::clientbound::json::StatusJsonError;
use ferrite_protocol::java_26_2::status::clientbound::packet::{
    ServerStatus, StatusClientboundPacket, StatusDescription, StatusPlayers, StatusSample,
    StatusVersion,
};
use ferrite_protocol::java_26_2::status::clientbound::projection::{
    StatusClientAction, StatusClientSession, StatusClientStage,
};
use ferrite_protocol::java_26_2::wire::compression::{
    CompressionMode, encode_packet as encode_wire,
};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

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

fn populated_status() -> ServerStatus {
    ServerStatus {
        description: StatusDescription::literal("Ferrite"),
        players: Some(StatusPlayers {
            max: 20,
            online: 0,
            sample: Vec::new(),
        }),
        version: Some(StatusVersion {
            name: "26.2".to_owned(),
            protocol: 776,
        }),
        favicon: None,
        enforces_secure_chat: false,
    }
}

fn encode_frame(packet: &StatusClientboundPacket) -> Vec<u8> {
    encode_wire(
        &encode_packet(packet).unwrap(),
        CompressionMode::Disabled,
        FrameLimits::default(),
    )
    .unwrap()
}

#[test]
fn matches_all_locked_status_clientbound_goldens() {
    let minimal = StatusClientboundPacket::Response(ServerStatus::default());
    assert_eq!(encode_frame(&minimal), hex("0400027b7d"));
    assert_eq!(
        decode_packet(&encode_packet(&minimal).unwrap()).unwrap(),
        minimal
    );

    let populated = StatusClientboundPacket::Response(populated_status());
    assert_eq!(
        encode_frame(&populated),
        hex(
            "6400627b226465736372697074696f6e223a2246657272697465222c22706c6179\
             657273223a7b226d6178223a32302c226f6e6c696e65223a307d2c227665727369\
             6f6e223a7b226e616d65223a2232362e32222c2270726f746f636f6c223a373736\
             7d7d"
        )
    );
    assert_eq!(
        decode_packet(&encode_packet(&populated).unwrap()).unwrap(),
        populated
    );

    let pong = StatusClientboundPacket::Pong(0x0102_0304_0506_0708);
    assert_eq!(encode_frame(&pong), hex("09010102030405060708"));
    assert_eq!(decode_packet(&encode_packet(&pong).unwrap()).unwrap(), pong);
}

fn response_body(json: &str, write_limit: usize) -> Vec<u8> {
    let mut writer = WireWriter::new(200_000);
    writer.write_var_i32(0).unwrap();
    writer.write_utf(json, write_limit).unwrap();
    writer.into_inner()
}

fn response(json: &str) -> Result<ServerStatus, StatusClientboundCodecError> {
    let StatusClientboundPacket::Response(status) = decode_packet(&response_body(json, 100_000))?
    else {
        unreachable!();
    };
    Ok(status)
}

#[test]
fn lenient_optional_fields_degrade_without_invalidating_the_response() {
    let status = response(
        r#"{"description":7,"players":{},"version":{"name":1,"protocol":"x"},"favicon":"bad","enforcesSecureChat":"yes","unknown":true}"#,
    )
    .unwrap();
    assert_eq!(status.description, StatusDescription::default());
    assert_eq!(status.players, None);
    assert_eq!(status.version, None);
    assert_eq!(status.favicon, None);
    assert!(!status.enforces_secure_chat);

    let sample_default = response(
        r#"{"players":{"max":-1,"online":2147483647,"sample":[{"id":"bad","name":"Player"}]}}"#,
    )
    .unwrap();
    assert_eq!(
        sample_default.players,
        Some(StatusPlayers {
            max: -1,
            online: i32::MAX,
            sample: Vec::new(),
        })
    );
}

#[test]
fn normalized_status_round_trips_samples_components_favicon_and_defaults() {
    let favicon = vec![0, 1, 2, 253, 254, 255];
    let status = ServerStatus {
        description: StatusDescription::from_json(r#"{"text":"Structured"}"#).unwrap(),
        players: Some(StatusPlayers {
            max: i32::MIN,
            online: i32::MAX,
            sample: vec![StatusSample {
                id: 0x0011_2233_4455_6677_8899_aabb_ccdd_eeff,
                name: "Player".to_owned(),
            }],
        }),
        version: Some(StatusVersion {
            name: "custom".to_owned(),
            protocol: i32::MIN,
        }),
        favicon: Some(favicon),
        enforces_secure_chat: true,
    };
    let packet = StatusClientboundPacket::Response(status);
    assert_eq!(
        decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
        packet
    );

    let with_line_feed =
        response(r#"{"favicon":"data:image/png;base64,AAEC\n/f7/","enforcesSecureChat":true}"#)
            .unwrap();
    assert_eq!(with_line_feed.favicon, Some(vec![0, 1, 2, 253, 254, 255]));
    assert!(with_line_feed.enforces_secure_chat);
}

#[test]
fn malformed_roots_lengths_and_packet_boundaries_fail_closed() {
    assert!(matches!(
        response("[]"),
        Err(StatusClientboundCodecError::Json(
            StatusJsonError::InvalidRoot
        ))
    ));
    assert!(matches!(
        response("{bad"),
        Err(StatusClientboundCodecError::Json(
            StatusJsonError::MalformedJson
        ))
    ));

    let exact = format!(r#"{{"x":"{}"}}"#, "x".repeat(32_759));
    assert_eq!(exact.encode_utf16().count(), 32_767);
    assert!(decode_packet(&response_body(&exact, 32_767)).is_ok());
    let too_long = format!(r#"{{"x":"{}"}}"#, "x".repeat(32_760));
    assert!(decode_packet(&response_body(&too_long, 32_768)).is_err());

    assert!(decode_packet(&[0, 2, b'{']).is_err());
    assert!(decode_packet(&[1, 0, 0, 0, 0, 0, 0, 0]).is_err());
    assert!(decode_packet(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    assert!(matches!(
        decode_packet(&[2]),
        Err(StatusClientboundCodecError::UnknownPacketId { id: 2 })
    ));
}

fn png_header(width: i32, height: i32) -> Vec<u8> {
    let mut bytes = Vec::from([137, 80, 78, 71, 13, 10, 26, 10]);
    bytes.extend_from_slice(&13_u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes
}

fn status_with_icon(icon: Option<Vec<u8>>) -> StatusClientboundPacket {
    StatusClientboundPacket::Response(ServerStatus {
        favicon: icon,
        ..ServerStatus::default()
    })
}

#[test]
fn favicon_projection_matches_the_locked_header_and_dimension_checks() {
    for (width, height) in [(1_024, 1_024), (-1, 1), (1, -1)] {
        let icon = png_header(width, height);
        let mut session = StatusClientSession::default();
        assert_eq!(
            session.apply(status_with_icon(Some(icon.clone())), 1),
            Ok(StatusClientAction::SendPing {
                token: 1,
                persistent_icon_changed: true,
            })
        );
        assert_eq!(session.presentation().icon, Some(icon));
    }

    for invalid in [
        png_header(1_025, 1),
        png_header(1, 1_025),
        vec![0; 24],
        vec![137, 80, 78],
    ] {
        let mut session = StatusClientSession::new(Some(vec![7]));
        assert!(matches!(
            session.apply(status_with_icon(Some(invalid)), 1),
            Ok(StatusClientAction::SendPing {
                persistent_icon_changed: true,
                ..
            })
        ));
        assert_eq!(session.presentation().icon, None);
    }

    let retained = png_header(1, 1);
    let mut absent = StatusClientSession::new(Some(retained.clone()));
    assert!(matches!(
        absent.apply(status_with_icon(None), 1),
        Ok(StatusClientAction::SendPing {
            persistent_icon_changed: false,
            ..
        })
    ));
    assert_eq!(absent.presentation().icon, Some(retained));
}

#[test]
fn client_lifecycle_sends_one_ping_rejects_a_second_response_and_ignores_pong_token() {
    let mut session = StatusClientSession::default();
    assert_eq!(
        session.apply(StatusClientboundPacket::Response(populated_status()), 1_000,),
        Ok(StatusClientAction::SendPing {
            token: 1_000,
            persistent_icon_changed: false,
        })
    );
    assert_eq!(session.stage(), StatusClientStage::AwaitingPong);
    assert!(session.successful_response());
    assert_eq!(
        session.apply(StatusClientboundPacket::Pong(i64::MIN), 1_025),
        Ok(StatusClientAction::Complete {
            latency_millis: 25,
            legacy_fallback_on_disconnect: false,
        })
    );
    assert_eq!(session.stage(), StatusClientStage::Closed);
    assert!(
        session
            .apply(StatusClientboundPacket::Pong(0), 1_026)
            .is_err()
    );

    let mut duplicate = StatusClientSession::default();
    duplicate
        .apply(
            StatusClientboundPacket::Response(ServerStatus::default()),
            0,
        )
        .unwrap();
    assert_eq!(
        duplicate.apply(
            StatusClientboundPacket::Response(ServerStatus::default()),
            1,
        ),
        Ok(StatusClientAction::CloseUnrequestedResponse)
    );

    let mut unsolicited_pong = StatusClientSession::default();
    assert_eq!(
        unsolicited_pong.apply(StatusClientboundPacket::Pong(-1), 7),
        Ok(StatusClientAction::Complete {
            latency_millis: 7,
            legacy_fallback_on_disconnect: true,
        })
    );
}
