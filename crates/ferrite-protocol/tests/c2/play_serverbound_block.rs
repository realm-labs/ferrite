use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_protocol::java_26_2::play::serverbound::codec;
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    BlockHit, Hand, PickItemFromBlock, PlayServerboundEntryPacket, PlayerAction, PlayerActionKind,
    Swing, UseItem, UseItemOn,
};

#[test]
fn five_serverbound_block_packets_round_trip_locked_fields() {
    let position = BlockPos::new(-33_554_432, -2_048, 33_554_431);
    let hit = BlockHit {
        position,
        direction: Direction::East,
        offset_x: f32::NAN,
        offset_y: f32::INFINITY,
        offset_z: f32::NEG_INFINITY,
        inside: true,
        world_border_hit: false,
    };
    let packets = [
        PlayServerboundEntryPacket::PickItemFromBlock(PickItemFromBlock {
            position,
            include_data: true,
        }),
        PlayServerboundEntryPacket::PlayerAction(PlayerAction {
            action: PlayerActionKind::StopDestroyBlock,
            position,
            direction: Direction::North,
            sequence: i32::MIN,
        }),
        PlayServerboundEntryPacket::Swing(Swing { hand: Hand::Off }),
        PlayServerboundEntryPacket::UseItemOn(UseItemOn {
            hand: Hand::Main,
            hit,
            sequence: i32::MAX,
        }),
        PlayServerboundEntryPacket::UseItem(UseItem {
            hand: Hand::Off,
            sequence: 0,
            yaw: f32::NAN,
            pitch: f32::INFINITY,
        }),
    ];
    for packet in packets {
        let encoded = codec::encode_packet(packet).unwrap();
        let decoded = codec::decode_packet(&encoded).unwrap();
        match (packet, decoded) {
            (
                PlayServerboundEntryPacket::UseItemOn(expected),
                PlayServerboundEntryPacket::UseItemOn(actual),
            ) => {
                assert_eq!(actual.hand, expected.hand);
                assert_eq!(actual.hit.position, expected.hit.position);
                assert!(actual.hit.offset_x.is_nan());
                assert_eq!(actual.sequence, expected.sequence);
            }
            (
                PlayServerboundEntryPacket::UseItem(expected),
                PlayServerboundEntryPacket::UseItem(actual),
            ) => {
                assert_eq!(actual.hand, expected.hand);
                assert!(actual.yaw.is_nan());
                assert_eq!(actual.sequence, expected.sequence);
            }
            (expected, actual) => assert_eq!(actual, expected),
        }
    }
}

#[test]
fn action_direction_is_modulo_but_hit_direction_and_hands_are_strict() {
    let mut action = codec::encode_packet(PlayServerboundEntryPacket::PlayerAction(PlayerAction {
        action: PlayerActionKind::StartDestroyBlock,
        position: BlockPos::default(),
        direction: Direction::Down,
        sequence: 1,
    }))
    .unwrap();
    let direction_offset = action.len() - 2;
    action[direction_offset] = 255;
    let PlayServerboundEntryPacket::PlayerAction(decoded) = codec::decode_packet(&action).unwrap()
    else {
        panic!("player action identity changed");
    };
    assert_eq!(decoded.direction, Direction::South);

    let mut swing = codec::encode_packet(PlayServerboundEntryPacket::Swing(Swing {
        hand: Hand::Main,
    }))
    .unwrap();
    swing[1] = 2;
    assert!(codec::decode_packet(&swing).is_err());

    let mut use_on = codec::encode_packet(PlayServerboundEntryPacket::UseItemOn(UseItemOn {
        hand: Hand::Main,
        hit: BlockHit {
            position: BlockPos::default(),
            direction: Direction::Down,
            offset_x: 0.0,
            offset_y: 0.0,
            offset_z: 0.0,
            inside: false,
            world_border_hit: false,
        },
        sequence: 1,
    }))
    .unwrap();
    use_on[10] = 6;
    assert!(codec::decode_packet(&use_on).is_err());
}
