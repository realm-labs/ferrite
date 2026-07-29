use ferrite_protocol::java_26_2::status::clientbound::codec::encode_packet as encode_clientbound;
use ferrite_protocol::java_26_2::status::clientbound::packet::{
    ServerStatus, StatusClientboundPacket, StatusDescription,
};
use ferrite_protocol::java_26_2::status::serverbound::codec::{
    StatusServerboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::status::serverbound::packet::StatusServerboundPacket;
use ferrite_protocol::java_26_2::status::serverbound::session::{
    StatusServerAction, StatusServerSession, StatusServerStage,
};
use ferrite_protocol::java_26_2::wire::compression::{
    CompressionMode, encode_packet as encode_wire,
};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;

fn frame(body: &[u8]) -> Vec<u8> {
    encode_wire(body, CompressionMode::Disabled, FrameLimits::default()).unwrap()
}

#[test]
fn matches_both_locked_status_serverbound_goldens() {
    let request = StatusServerboundPacket::Request;
    let request_body = encode_packet(request).unwrap();
    assert_eq!(frame(&request_body), [0x01, 0x00]);
    assert_eq!(decode_packet(&request_body).unwrap(), request);

    let ping = StatusServerboundPacket::Ping(0x0102_0304_0506_0708);
    let ping_body = encode_packet(ping).unwrap();
    assert_eq!(
        frame(&ping_body),
        [0x09, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
    );
    assert_eq!(decode_packet(&ping_body).unwrap(), ping);
}

#[test]
fn codec_preserves_signed_ping_bits_and_rejects_malformed_packets() {
    for token in [i64::MIN, -1, 0, 1, i64::MAX] {
        let packet = StatusServerboundPacket::Ping(token);
        assert_eq!(
            decode_packet(&encode_packet(packet).unwrap()).unwrap(),
            packet
        );
    }
    assert_eq!(
        decode_packet(&[0x80, 0]).unwrap(),
        StatusServerboundPacket::Request
    );
    assert!(decode_packet(&[0, 0]).is_err());
    assert!(decode_packet(&[1, 0, 0, 0, 0, 0, 0, 0]).is_err());
    assert!(decode_packet(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    assert!(matches!(
        decode_packet(&[2]),
        Err(StatusServerboundCodecError::UnknownPacketId { id: 2 })
    ));
}

#[test]
fn first_request_sends_the_cached_snapshot_and_duplicate_closes_without_response() {
    let snapshot = ServerStatus {
        description: StatusDescription::literal("cached"),
        ..ServerStatus::default()
    };
    let mut session = StatusServerSession::new(snapshot.clone());
    assert!(!session.request_handled());
    assert_eq!(
        session.apply(StatusServerboundPacket::Request),
        Ok(StatusServerAction::Send(StatusClientboundPacket::Response(
            snapshot.clone()
        )))
    );
    assert!(session.request_handled());
    assert_eq!(session.stage(), StatusServerStage::Open);
    assert_eq!(session.cached_status(), &snapshot);
    assert_eq!(
        session.apply(StatusServerboundPacket::Request),
        Ok(StatusServerAction::CloseRequestHandled)
    );
    assert_eq!(session.stage(), StatusServerStage::Closed);
    assert!(session.apply(StatusServerboundPacket::Request).is_err());
}

#[test]
fn ping_before_status_echoes_then_requires_send_completion_before_close() {
    let mut session = StatusServerSession::new(ServerStatus::default());
    assert_eq!(
        session.apply(StatusServerboundPacket::Ping(-1)),
        Ok(StatusServerAction::Send(StatusClientboundPacket::Pong(-1)))
    );
    assert_eq!(session.stage(), StatusServerStage::PongPending);
    assert!(!session.request_handled());
    assert!(session.apply(StatusServerboundPacket::Request).is_err());
    assert_eq!(
        session.pong_sent(),
        Ok(StatusServerAction::CloseRequestHandled)
    );
    assert_eq!(session.stage(), StatusServerStage::Closed);
    assert!(session.pong_sent().is_err());
}

#[test]
fn happy_trace_orders_status_response_then_exact_pong_then_close() {
    let mut session = StatusServerSession::new(ServerStatus::default());
    let StatusServerAction::Send(response) =
        session.apply(StatusServerboundPacket::Request).unwrap()
    else {
        panic!("expected status response");
    };
    assert_eq!(
        frame(&encode_clientbound(&response).unwrap()),
        [4, 0, 2, b'{', b'}']
    );

    let token = 0x0102_0304_0506_0708;
    let StatusServerAction::Send(pong) =
        session.apply(StatusServerboundPacket::Ping(token)).unwrap()
    else {
        panic!("expected pong");
    };
    assert_eq!(
        frame(&encode_clientbound(&pong).unwrap()),
        [9, 1, 1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(session.stage(), StatusServerStage::PongPending);
    assert_eq!(
        session.pong_sent(),
        Ok(StatusServerAction::CloseRequestHandled)
    );
}
