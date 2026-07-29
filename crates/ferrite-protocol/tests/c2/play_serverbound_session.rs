use ferrite_protocol::java_26_2::play::serverbound::codec;
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    ChunkBatchReceived, KeepAlive, MovePlayerPosition, MovePlayerPositionRotation,
    MovePlayerRotation, MovePlayerStatusOnly, MovementFlags, PlayServerboundEntryPacket,
    PlayerPosition, PlayerRotation,
};

fn flags(on_ground: bool, horizontal_collision: bool) -> MovementFlags {
    MovementFlags {
        on_ground,
        horizontal_collision,
    }
}

fn position() -> PlayerPosition {
    PlayerPosition {
        x: 1.25,
        y: -2.5,
        z: 3.75,
    }
}

fn rotation() -> PlayerRotation {
    PlayerRotation {
        yaw: -180.0,
        pitch: 90.0,
    }
}

#[test]
fn four_movement_forms_preserve_omitted_fields_and_exact_wire_order() {
    let packets = [
        PlayServerboundEntryPacket::MovePlayerPosition(MovePlayerPosition {
            position: position(),
            flags: flags(true, false),
        }),
        PlayServerboundEntryPacket::MovePlayerPositionRotation(MovePlayerPositionRotation {
            position: position(),
            rotation: rotation(),
            flags: flags(false, true),
        }),
        PlayServerboundEntryPacket::MovePlayerRotation(MovePlayerRotation {
            rotation: rotation(),
            flags: flags(true, true),
        }),
        PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
            flags: flags(false, false),
        }),
    ];
    for packet in packets {
        let body = codec::encode_packet(packet).unwrap();
        assert_eq!(codec::decode_packet(&body).unwrap(), packet);
    }
    assert_eq!(codec::encode_packet(packets[0]).unwrap()[0], 30);
    assert_eq!(codec::encode_packet(packets[1]).unwrap()[0], 31);
    assert_eq!(codec::encode_packet(packets[2]).unwrap()[0], 32);
    assert_eq!(codec::encode_packet(packets[3]).unwrap()[0], 33);
}

#[test]
fn movement_high_flag_bits_decode_as_ignored_and_reencode_canonically() {
    let mut body = codec::encode_packet(PlayServerboundEntryPacket::MovePlayerStatusOnly(
        MovePlayerStatusOnly {
            flags: flags(true, true),
        },
    ))
    .unwrap();
    body[1] = 0xff;
    let decoded = codec::decode_packet(&body).unwrap();
    assert_eq!(
        decoded,
        PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
            flags: flags(true, true),
        })
    );
    assert_eq!(codec::encode_packet(decoded).unwrap(), [33, 3]);
    body.push(0);
    assert!(codec::decode_packet(&body).is_err());
}

#[test]
fn load_feedback_tick_end_and_keep_alive_have_locked_ids_and_bodies() {
    let packets = [
        PlayServerboundEntryPacket::ChunkBatchReceived(ChunkBatchReceived {
            desired_chunks_per_tick: f32::NAN,
        }),
        PlayServerboundEntryPacket::ClientTickEnd,
        PlayServerboundEntryPacket::KeepAlive(KeepAlive {
            challenge: i64::MIN,
        }),
        PlayServerboundEntryPacket::PlayerLoaded,
    ];
    let expected_ids = [11, 13, 28, 44];
    for (packet, expected_id) in packets.into_iter().zip(expected_ids) {
        let body = codec::encode_packet(packet).unwrap();
        assert_eq!(body[0], expected_id);
        let decoded = codec::decode_packet(&body).unwrap();
        if matches!(packet, PlayServerboundEntryPacket::ChunkBatchReceived(_)) {
            let PlayServerboundEntryPacket::ChunkBatchReceived(decoded) = decoded else {
                panic!("feedback identity changed");
            };
            assert!(decoded.desired_chunks_per_tick.is_nan());
        } else {
            assert_eq!(decoded, packet);
        }
    }
}
