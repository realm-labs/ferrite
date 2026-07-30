use ferrite_protocol::java_26_2::play::clientbound::packet::Vector3;
use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    AcceptTeleportation, PlayServerboundEntryPacket,
};
use ferrite_protocol::java_26_2::play::serverbound::teleport::{
    MovementDisposition, TeleportAcknowledgement, TeleportSynchronizer, next_teleport_challenge,
};
use ferrite_protocol::java_26_2::wire::compression::{
    CompressionMode, encode_packet as encode_wire,
};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;

fn accept(challenge: i32) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::AcceptTeleportation(AcceptTeleportation { challenge })
}

fn position(x: f64, y: f64, z: f64) -> Vector3 {
    Vector3 { x, y, z }
}

#[test]
fn matches_the_locked_teleport_acknowledgement_golden() {
    let packet = accept(1);
    let body = encode_packet(packet.clone()).unwrap();
    assert_eq!(
        encode_wire(
            &body,
            CompressionMode::enabled(256).unwrap(),
            FrameLimits::default(),
        )
        .unwrap(),
        [0x03, 0x00, 0x00, 0x01]
    );
    assert_eq!(decode_packet(&body).unwrap(), packet);
}

#[test]
fn codec_preserves_signed_varints_and_fails_closed_at_the_family_boundary() {
    for challenge in [i32::MIN, -1, 0, 1, i32::MAX] {
        let packet = accept(challenge);
        assert_eq!(
            decode_packet(&encode_packet(packet.clone()).unwrap()).unwrap(),
            packet
        );
    }

    assert_eq!(decode_packet(&[0, 0x81, 0]).unwrap(), accept(1));
    assert!(decode_packet(&[0, 0x80]).is_err());
    assert!(decode_packet(&[0, 1, 0]).is_err());
    assert!(matches!(
        decode_packet(&[1]),
        Err(PlayServerboundEntryCodecError::EntitySession(_))
    ));
    assert!(matches!(
        decode_packet(&[2]),
        Err(PlayServerboundEntryCodecError::UnsupportedPacketIdentity { .. })
    ));
    assert!(matches!(
        decode_packet(&[0xe8, 0x07]),
        Err(PlayServerboundEntryCodecError::UnknownPacketId { id: 1_000 })
    ));
}

#[test]
fn stale_matching_and_duplicate_acknowledgements_have_distinct_outcomes() {
    let initial = position(-1.0, 64.0, 2.0);
    let target = position(12.5, -4.0, 99.0);
    let mut synchronization = TeleportSynchronizer::new(initial);
    synchronization.mark_dimension_change_pending();
    let challenge = synchronization.issue_correction(target, 40);
    assert_eq!(challenge.challenge, 1);
    assert_eq!(
        synchronization.movement_disposition(),
        MovementDisposition::SuppressWhileTeleportPending
    );

    for stale in [0, 2, i32::MIN, i32::MAX] {
        assert_eq!(
            synchronization.handle(accept(stale)),
            TeleportAcknowledgement::IgnoredStale {
                received: stale,
                current: 1,
            }
        );
        assert_eq!(synchronization.pending_position(), Some(target));
    }

    assert_eq!(
        synchronization.handle(accept(1)),
        TeleportAcknowledgement::Accepted {
            authoritative_position: target,
            completed_dimension_change: true,
        }
    );
    assert_eq!(synchronization.pending_position(), None);
    assert_eq!(synchronization.last_good_position(), target);
    assert!(!synchronization.dimension_change_pending());
    assert_eq!(
        synchronization.movement_disposition(),
        MovementDisposition::Validate
    );
    assert_eq!(
        synchronization.handle(accept(1)),
        TeleportAcknowledgement::DisconnectInvalidMovement
    );
    assert_eq!(
        synchronization.handle(accept(0)),
        TeleportAcknowledgement::IgnoredStale {
            received: 0,
            current: 1,
        }
    );
}

#[test]
fn challenge_zero_without_a_pending_position_is_invalid_movement() {
    let mut synchronization = TeleportSynchronizer::default();
    assert_eq!(synchronization.current_challenge(), 0);
    assert_eq!(
        synchronization.handle(accept(0)),
        TeleportAcknowledgement::DisconnectInvalidMovement
    );
}

#[test]
fn resend_occurs_only_after_tick_twenty_and_replaces_the_current_challenge() {
    let target = position(1.0, 2.0, 3.0);
    let mut synchronization = TeleportSynchronizer::default();
    assert_eq!(synchronization.issue_correction(target, 10).challenge, 1);
    assert_eq!(synchronization.resend_if_due(30), None);
    assert_eq!(synchronization.resend_if_due(31).unwrap().challenge, 2);
    assert_eq!(
        synchronization.handle(accept(1)),
        TeleportAcknowledgement::IgnoredStale {
            received: 1,
            current: 2,
        }
    );
    assert!(matches!(
        synchronization.handle(accept(2)),
        TeleportAcknowledgement::Accepted { .. }
    ));

    let mut wrapping_ticks = TeleportSynchronizer::default();
    wrapping_ticks.issue_correction(target, i32::MAX - 10);
    assert_eq!(wrapping_ticks.resend_if_due(i32::MIN + 9), None);
    assert_eq!(
        wrapping_ticks
            .resend_if_due(i32::MIN + 10)
            .unwrap()
            .challenge,
        2
    );
}

#[test]
fn challenge_increment_wraps_only_maximum_to_zero() {
    assert_eq!(next_teleport_challenge(0), 1);
    assert_eq!(next_teleport_challenge(-1), 0);
    assert_eq!(next_teleport_challenge(i32::MAX - 1), i32::MAX);
    assert_eq!(next_teleport_challenge(i32::MAX), 0);
}
