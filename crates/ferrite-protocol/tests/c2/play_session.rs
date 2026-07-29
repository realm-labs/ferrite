use ferrite_protocol::java_26_2::play::clientbound::codec as clientbound;
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    KeepAlive as ClientboundKeepAlive, PlayClientboundPacket, PlayerRotation as RotationCorrection,
    Vector3, VehiclePosition,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::play::serverbound::codec as serverbound;
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
        let body = serverbound::encode_packet(packet).unwrap();
        assert_eq!(serverbound::decode_packet(&body).unwrap(), packet);
    }
    assert_eq!(serverbound::encode_packet(packets[0]).unwrap()[0], 30);
    assert_eq!(serverbound::encode_packet(packets[1]).unwrap()[0], 31);
    assert_eq!(serverbound::encode_packet(packets[2]).unwrap()[0], 32);
    assert_eq!(serverbound::encode_packet(packets[3]).unwrap()[0], 33);
}

#[test]
fn movement_high_flag_bits_decode_as_ignored_and_reencode_canonically() {
    let mut body = serverbound::encode_packet(PlayServerboundEntryPacket::MovePlayerStatusOnly(
        MovePlayerStatusOnly {
            flags: flags(true, true),
        },
    ))
    .unwrap();
    body[1] = 0xff;
    let decoded = serverbound::decode_packet(&body).unwrap();
    assert_eq!(
        decoded,
        PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
            flags: flags(true, true),
        })
    );
    assert_eq!(serverbound::encode_packet(decoded).unwrap(), [33, 3]);
    body.push(0);
    assert!(serverbound::decode_packet(&body).is_err());
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
        let body = serverbound::encode_packet(packet).unwrap();
        assert_eq!(body[0], expected_id);
        let decoded = serverbound::decode_packet(&body).unwrap();
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

#[test]
fn clientbound_session_packets_round_trip_exceptional_floats() {
    let registries = PlayRegistries::default();
    let component_values = RejectComponentValues;
    let context = PlayDecodeContext {
        registries: &registries,
        component_values: &component_values,
        dimension_section_count: 24,
    };
    let packets = [
        PlayClientboundPacket::KeepAlive(ClientboundKeepAlive {
            challenge: i64::MAX,
        }),
        PlayClientboundPacket::MoveVehicle(VehiclePosition {
            position: Vector3 {
                x: f64::NAN,
                y: f64::INFINITY,
                z: f64::NEG_INFINITY,
            },
            yaw: f32::NAN,
            pitch: f32::INFINITY,
        }),
        PlayClientboundPacket::PlayerRotation(RotationCorrection {
            yaw: f32::NEG_INFINITY,
            relative_yaw: true,
            pitch: f32::NAN,
            relative_pitch: false,
        }),
    ];
    for packet in packets {
        let body = clientbound::encode_packet(&packet, &registries).unwrap();
        let decoded = clientbound::decode_packet(&body, context).unwrap();
        assert_eq!(
            clientbound::encode_packet(&decoded, &registries).unwrap(),
            body
        );
    }
}
