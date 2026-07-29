use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_protocol::java_26_2::play::serverbound::block::{
    BlockDispatchError, BlockDispatchOutcome, BlockSequenceRegistrar, ServerboundBlockHandler,
    dispatch_block_packet,
};
use ferrite_protocol::java_26_2::play::serverbound::codec;
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    BlockHit, Hand, PickItemFromBlock, PlayServerboundEntryPacket, PlayerAction, PlayerActionKind,
    Swing, UseItem, UseItemOn,
};
use ferrite_protocol::java_26_2::play::serverbound::session::PlayServerSession;

fn default_hit() -> BlockHit {
    BlockHit {
        position: BlockPos::default(),
        direction: Direction::Down,
        offset_x: 0.0,
        offset_y: 0.0,
        offset_z: 0.0,
        inside: false,
        world_border_hit: false,
    }
}

#[test]
fn five_serverbound_block_packets_have_locked_default_goldens() {
    let vectors = [
        (
            PlayServerboundEntryPacket::PickItemFromBlock(PickItemFromBlock {
                position: BlockPos::default(),
                include_data: false,
            }),
            vec![36, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayServerboundEntryPacket::PlayerAction(PlayerAction {
                action: PlayerActionKind::StartDestroyBlock,
                position: BlockPos::default(),
                direction: Direction::Down,
                sequence: 0,
            }),
            vec![41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayServerboundEntryPacket::Swing(Swing { hand: Hand::Main }),
            vec![63, 0],
        ),
        (
            PlayServerboundEntryPacket::UseItemOn(UseItemOn {
                hand: Hand::Main,
                hit: default_hit(),
                sequence: 0,
            }),
            vec![
                66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        ),
        (
            PlayServerboundEntryPacket::UseItem(UseItem {
                hand: Hand::Main,
                sequence: 0,
                yaw: 0.0,
                pitch: 0.0,
            }),
            vec![67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
    ];
    for (packet, expected) in vectors {
        assert_eq!(codec::encode_packet(packet).unwrap(), expected);
        assert_eq!(codec::decode_packet(&expected).unwrap(), packet);
    }
}

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

#[test]
fn all_action_ordinals_and_all_unsigned_direction_bytes_follow_locked_mapping() {
    let actions = [
        PlayerActionKind::StartDestroyBlock,
        PlayerActionKind::AbortDestroyBlock,
        PlayerActionKind::StopDestroyBlock,
        PlayerActionKind::DropAllItems,
        PlayerActionKind::DropItem,
        PlayerActionKind::ReleaseUseItem,
        PlayerActionKind::SwapItemWithOffhand,
        PlayerActionKind::Stab,
    ];
    for (index, action_kind) in actions.into_iter().enumerate() {
        let packet = PlayServerboundEntryPacket::PlayerAction(PlayerAction {
            action: action_kind,
            position: BlockPos::default(),
            direction: Direction::Down,
            sequence: 0,
        });
        let mut encoded = codec::encode_packet(packet).unwrap();
        assert_eq!(encoded[1], index as u8);
        for direction in 0..=u8::MAX {
            encoded[10] = direction;
            let PlayServerboundEntryPacket::PlayerAction(decoded) =
                codec::decode_packet(&encoded).unwrap()
            else {
                panic!("expected player action");
            };
            assert_eq!(decoded.action, action_kind);
            assert_eq!(
                decoded.direction,
                [
                    Direction::Down,
                    Direction::Up,
                    Direction::North,
                    Direction::South,
                    Direction::West,
                    Direction::East,
                ][usize::from(direction % 6)]
            );
        }
    }

    let mut invalid =
        codec::encode_packet(PlayServerboundEntryPacket::PlayerAction(PlayerAction {
            action: PlayerActionKind::StartDestroyBlock,
            position: BlockPos::default(),
            direction: Direction::Down,
            sequence: 0,
        }))
        .unwrap();
    invalid[1] = 8;
    assert!(codec::decode_packet(&invalid).is_err());
    invalid[1] = 255;
    invalid.splice(2..2, [255, 255, 255, 15]);
    assert!(codec::decode_packet(&invalid).is_err());
}

#[test]
fn malformed_truncation_trailing_bytes_and_strict_hit_direction_fail_closed() {
    let packet = PlayServerboundEntryPacket::UseItemOn(UseItemOn {
        hand: Hand::Off,
        hit: default_hit(),
        sequence: i32::MAX,
    });
    let encoded = codec::encode_packet(packet).unwrap();
    for length in 0..encoded.len() {
        assert!(codec::decode_packet(&encoded[..length]).is_err());
    }
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(codec::decode_packet(&trailing).is_err());

    let mut invalid_direction = encoded;
    invalid_direction[10] = 6;
    assert!(codec::decode_packet(&invalid_direction).is_err());
}

#[test]
fn booleans_are_nonzero_and_float_payload_bits_survive_the_codec() {
    let mut pick = codec::encode_packet(PlayServerboundEntryPacket::PickItemFromBlock(
        PickItemFromBlock {
            position: BlockPos::default(),
            include_data: false,
        },
    ))
    .unwrap();
    *pick.last_mut().unwrap() = 255;
    let PlayServerboundEntryPacket::PickItemFromBlock(decoded_pick) =
        codec::decode_packet(&pick).unwrap()
    else {
        panic!("expected pick block");
    };
    assert!(decoded_pick.include_data);

    let hit = BlockHit {
        offset_x: f32::from_bits(0x7fc0_1234),
        offset_y: f32::from_bits(0x7f80_0000),
        offset_z: f32::from_bits(0xff80_0000),
        inside: true,
        world_border_hit: true,
        ..default_hit()
    };
    let PlayServerboundEntryPacket::UseItemOn(decoded_hit) = codec::decode_packet(
        &codec::encode_packet(PlayServerboundEntryPacket::UseItemOn(UseItemOn {
            hand: Hand::Main,
            hit,
            sequence: 1,
        }))
        .unwrap(),
    )
    .unwrap() else {
        panic!("expected use item on");
    };
    assert_eq!(decoded_hit.hit.offset_x.to_bits(), hit.offset_x.to_bits());
    assert_eq!(decoded_hit.hit.offset_y.to_bits(), hit.offset_y.to_bits());
    assert_eq!(decoded_hit.hit.offset_z.to_bits(), hit.offset_z.to_bits());
    assert!(decoded_hit.hit.inside);
    assert!(decoded_hit.hit.world_border_hit);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
    Handler(&'static str),
    Sequence(i32),
}

#[derive(Debug, Default)]
struct Recorder {
    events: Vec<Event>,
    fail_handler: bool,
}

impl ServerboundBlockHandler for Recorder {
    type Error = &'static str;

    fn pick_item_from_block(&mut self, _: PickItemFromBlock) -> Result<(), Self::Error> {
        self.record("pick")
    }

    fn destroy(&mut self, _: PlayerAction) -> Result<(), Self::Error> {
        self.record("destroy")
    }

    fn auxiliary_action(&mut self, _: PlayerAction) -> Result<(), Self::Error> {
        self.record("auxiliary")
    }

    fn swing(&mut self, _: Swing) -> Result<(), Self::Error> {
        self.record("swing")
    }

    fn use_item_on(&mut self, _: UseItemOn) -> Result<(), Self::Error> {
        self.record("use_on")
    }

    fn use_item(&mut self, _: UseItem) -> Result<(), Self::Error> {
        self.record("use")
    }
}

impl Recorder {
    fn record(&mut self, name: &'static str) -> Result<(), &'static str> {
        self.events.push(Event::Handler(name));
        if self.fail_handler {
            Err("handler fault")
        } else {
            Ok(())
        }
    }
}

impl BlockSequenceRegistrar for Recorder {
    type Error = i32;

    fn register_block_sequence(&mut self, sequence: i32) -> Result<(), Self::Error> {
        self.events.push(Event::Sequence(sequence));
        if sequence < 0 { Err(sequence) } else { Ok(()) }
    }
}

fn action(kind: PlayerActionKind, sequence: i32) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::PlayerAction(PlayerAction {
        action: kind,
        position: BlockPos::default(),
        direction: Direction::Down,
        sequence,
    })
}

#[test]
fn predictive_registration_has_path_specific_order_and_loaded_gate() {
    let mut destroy_handler = Recorder::default();
    let mut destroy_sequences = Recorder::default();
    assert_eq!(
        dispatch_block_packet(
            action(PlayerActionKind::StartDestroyBlock, 7),
            true,
            &mut destroy_handler,
            &mut destroy_sequences
        ),
        Ok(BlockDispatchOutcome::Handled)
    );
    assert_eq!(destroy_handler.events, [Event::Handler("destroy")]);
    assert_eq!(destroy_sequences.events, [Event::Sequence(7)]);

    let use_on = PlayServerboundEntryPacket::UseItemOn(UseItemOn {
        hand: Hand::Main,
        hit: default_hit(),
        sequence: 8,
    });
    let mut use_handler = Recorder::default();
    let mut use_sequences = Recorder::default();
    dispatch_block_packet(use_on, true, &mut use_handler, &mut use_sequences).unwrap();
    assert_eq!(use_sequences.events, [Event::Sequence(8)]);
    assert_eq!(use_handler.events, [Event::Handler("use_on")]);

    let mut dropped_handler = Recorder::default();
    let mut dropped_sequences = Recorder::default();
    assert_eq!(
        dispatch_block_packet(use_on, false, &mut dropped_handler, &mut dropped_sequences),
        Ok(BlockDispatchOutcome::DroppedBeforeClientLoaded)
    );
    assert!(dropped_handler.events.is_empty());
    assert!(dropped_sequences.events.is_empty());
}

#[test]
fn negative_sequence_faults_after_destroy_but_before_use_and_auxiliary_ignores_it() {
    let mut destroy_handler = Recorder::default();
    let mut destroy_sequences = Recorder::default();
    assert_eq!(
        dispatch_block_packet(
            action(PlayerActionKind::StopDestroyBlock, -1),
            true,
            &mut destroy_handler,
            &mut destroy_sequences
        ),
        Err(BlockDispatchError::Sequence(-1))
    );
    assert_eq!(destroy_handler.events, [Event::Handler("destroy")]);

    let mut use_handler = Recorder::default();
    let mut use_sequences = Recorder::default();
    assert_eq!(
        dispatch_block_packet(
            PlayServerboundEntryPacket::UseItem(UseItem {
                hand: Hand::Main,
                sequence: -2,
                yaw: 0.0,
                pitch: 0.0,
            }),
            true,
            &mut use_handler,
            &mut use_sequences
        ),
        Err(BlockDispatchError::Sequence(-2))
    );
    assert!(use_handler.events.is_empty());

    let mut auxiliary_handler = Recorder::default();
    let mut auxiliary_sequences = Recorder::default();
    assert_eq!(
        dispatch_block_packet(
            action(PlayerActionKind::DropItem, -3),
            false,
            &mut auxiliary_handler,
            &mut auxiliary_sequences
        ),
        Ok(BlockDispatchOutcome::Handled)
    );
    assert_eq!(auxiliary_handler.events, [Event::Handler("auxiliary")]);
    assert!(auxiliary_sequences.events.is_empty());
}

#[test]
fn handler_fault_preserves_before_and_after_registration_order() {
    let mut destroy_handler = Recorder {
        fail_handler: true,
        ..Recorder::default()
    };
    let mut destroy_sequences = Recorder::default();
    assert_eq!(
        dispatch_block_packet(
            action(PlayerActionKind::AbortDestroyBlock, 4),
            true,
            &mut destroy_handler,
            &mut destroy_sequences
        ),
        Err(BlockDispatchError::Handler("handler fault"))
    );
    assert!(destroy_sequences.events.is_empty());

    let mut use_handler = Recorder {
        fail_handler: true,
        ..Recorder::default()
    };
    let mut use_sequences = Recorder::default();
    assert_eq!(
        dispatch_block_packet(
            PlayServerboundEntryPacket::UseItem(UseItem {
                hand: Hand::Off,
                sequence: 5,
                yaw: 0.0,
                pitch: 0.0,
            }),
            true,
            &mut use_handler,
            &mut use_sequences
        ),
        Err(BlockDispatchError::Handler("handler fault"))
    );
    assert_eq!(use_sequences.events, [Event::Sequence(5)]);
    assert_eq!(use_handler.events, [Event::Handler("use")]);
}

#[test]
fn pick_swing_and_auxiliary_actions_do_not_use_the_client_loaded_gate() {
    let packets = [
        PlayServerboundEntryPacket::PickItemFromBlock(PickItemFromBlock {
            position: BlockPos::default(),
            include_data: false,
        }),
        PlayServerboundEntryPacket::Swing(Swing { hand: Hand::Main }),
        action(PlayerActionKind::ReleaseUseItem, -1),
    ];
    let mut handler = Recorder::default();
    let mut sequences = Recorder::default();
    for packet in packets {
        assert_eq!(
            dispatch_block_packet(packet, false, &mut handler, &mut sequences),
            Ok(BlockDispatchOutcome::Handled)
        );
    }
    assert_eq!(
        handler.events,
        [
            Event::Handler("pick"),
            Event::Handler("swing"),
            Event::Handler("auxiliary")
        ]
    );
    assert!(sequences.events.is_empty());
}

#[test]
fn dispatcher_writes_the_tick_local_play_session_accumulator() {
    let mut handler = Recorder::default();
    let mut session = PlayServerSession::default();
    dispatch_block_packet(
        PlayServerboundEntryPacket::UseItem(UseItem {
            hand: Hand::Main,
            sequence: 14,
            yaw: 0.0,
            pitch: 0.0,
        }),
        true,
        &mut handler,
        &mut session,
    )
    .unwrap();
    dispatch_block_packet(
        action(PlayerActionKind::StartDestroyBlock, 9),
        true,
        &mut handler,
        &mut session,
    )
    .unwrap();
    assert_eq!(session.take_block_sequence_ack(), Some(14));
    assert_eq!(session.take_block_sequence_ack(), None);
}
