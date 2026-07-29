use ferrite_foundation::coordinate::{BlockPos, SectionPos};
use ferrite_protocol::java_26_2::play::clientbound::block::{
    BlockClientProjection, BlockProjectionAction, BlockProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    BlockChangedAck, BlockDestruction, BlockEntityData, BlockEvent, BlockUpdate,
    PlayClientboundPacket, SectionBlockChange, SectionBlocksUpdate,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::nbt::{NbtQuota, NetworkNbt};

static REJECT_COMPONENTS: RejectComponentValues = RejectComponentValues;

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &REJECT_COMPONENTS,
        dimension_section_count: 24,
    }
}

fn compound() -> NetworkNbt {
    NetworkNbt::from_bytes(vec![10, 0], NbtQuota::Trusted).unwrap()
}

fn append_var_i64(bytes: &mut Vec<u8>, value: i64) {
    let mut remaining = value as u64;
    loop {
        let mut byte = (remaining & 0x7f) as u8;
        remaining >>= 7;
        if remaining != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if remaining == 0 {
            return;
        }
    }
}

#[test]
fn six_packet_goldens_have_locked_ids_and_zero_bodies() {
    let registries = PlayRegistries::default();
    let vectors = [
        (
            PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: 0 }),
            vec![4, 0],
        ),
        (
            PlayClientboundPacket::BlockDestruction(BlockDestruction {
                breaker_entity_id: 0,
                position: BlockPos::default(),
                progress: 0,
            }),
            vec![5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::BlockEntityData(BlockEntityData {
                position: BlockPos::default(),
                type_raw_id: 0,
                update_tag: compound(),
            }),
            vec![6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0],
        ),
        (
            PlayClientboundPacket::BlockEvent(BlockEvent {
                position: BlockPos::default(),
                action: 0,
                parameter: 0,
                block_raw_id: 0,
            }),
            vec![7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::BlockUpdate(BlockUpdate {
                position: BlockPos::default(),
                state: 0,
            }),
            vec![8, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::SectionBlocksUpdate(SectionBlocksUpdate {
                section: SectionPos::default(),
                changes: Vec::new(),
            }),
            vec![84, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
    ];

    for (packet, expected) in vectors {
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(encoded, expected);
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
}

#[test]
fn signed_coordinates_and_registry_boundaries_round_trip() {
    let registries = PlayRegistries::default();
    let position = BlockPos::new(-33_554_432, -2_048, 33_554_431);
    let packets = [
        PlayClientboundPacket::BlockDestruction(BlockDestruction {
            breaker_entity_id: i32::MIN,
            position,
            progress: u8::MAX,
        }),
        PlayClientboundPacket::BlockEntityData(BlockEntityData {
            position,
            type_raw_id: 48,
            update_tag: compound(),
        }),
        PlayClientboundPacket::BlockEvent(BlockEvent {
            position,
            action: u8::MAX,
            parameter: u8::MAX,
            block_raw_id: 1_195,
        }),
        PlayClientboundPacket::BlockUpdate(BlockUpdate {
            position,
            state: 32_365,
        }),
        PlayClientboundPacket::SectionBlocksUpdate(SectionBlocksUpdate {
            section: SectionPos::new(-2_097_152, -524_288, 2_097_151),
            changes: vec![
                SectionBlockChange {
                    relative_position: 0,
                    state: Some(0),
                },
                SectionBlockChange {
                    relative_position: 4_095,
                    state: Some(32_365),
                },
            ],
        }),
        PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: i32::MIN }),
    ];

    for packet in packets {
        let bytes = encode_packet(&packet, &registries).unwrap();
        assert_eq!(decode_packet(&bytes, context(&registries)).unwrap(), packet);
    }
}

#[test]
fn required_registries_and_standalone_compound_fail_closed() {
    let registries = PlayRegistries::default();
    let invalid_packets = [
        PlayClientboundPacket::BlockEntityData(BlockEntityData {
            position: BlockPos::default(),
            type_raw_id: 49,
            update_tag: compound(),
        }),
        PlayClientboundPacket::BlockEvent(BlockEvent {
            position: BlockPos::default(),
            action: 0,
            parameter: 0,
            block_raw_id: 1_196,
        }),
        PlayClientboundPacket::BlockUpdate(BlockUpdate {
            position: BlockPos::default(),
            state: 32_366,
        }),
        PlayClientboundPacket::SectionBlocksUpdate(SectionBlocksUpdate {
            section: SectionPos::default(),
            changes: vec![SectionBlockChange {
                relative_position: 0,
                state: None,
            }],
        }),
    ];
    for packet in invalid_packets {
        assert!(encode_packet(&packet, &registries).is_err());
    }

    let scalar = NetworkNbt::from_bytes(vec![1, 0], NbtQuota::Trusted).unwrap();
    let scalar_packet = PlayClientboundPacket::BlockEntityData(BlockEntityData {
        position: BlockPos::default(),
        type_raw_id: 0,
        update_tag: scalar,
    });
    assert!(encode_packet(&scalar_packet, &registries).is_err());

    let mut null_tag = vec![6];
    null_tag.extend_from_slice(&0_i64.to_be_bytes());
    null_tag.extend_from_slice(&[0, 0]);
    assert!(decode_packet(&null_tag, context(&registries)).is_err());

    let mut trailing = encode_packet(
        &PlayClientboundPacket::BlockEntityData(BlockEntityData {
            position: BlockPos::default(),
            type_raw_id: 0,
            update_tag: compound(),
        }),
        &registries,
    )
    .unwrap();
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn standalone_block_entity_data_uses_trusted_not_default_nbt_quota() {
    let registries = PlayRegistries::default();
    let payload_len = 2_097_153_i32;
    let mut tag = vec![10, 7, 0, 0];
    tag.extend_from_slice(&payload_len.to_be_bytes());
    tag.resize(tag.len() + payload_len as usize, 0x5a);
    tag.push(0);
    let trusted = NetworkNbt::from_bytes(tag, NbtQuota::Trusted).unwrap();
    let packet = PlayClientboundPacket::BlockEntityData(BlockEntityData {
        position: BlockPos::default(),
        type_raw_id: 48,
        update_tag: trusted,
    });
    let bytes = encode_packet(&packet, &registries).unwrap();
    assert_eq!(decode_packet(&bytes, context(&registries)).unwrap(), packet);
}

#[test]
fn malformed_section_counts_varlongs_and_trailing_bytes_fault() {
    let registries = PlayRegistries::default();
    let mut negative_count = vec![84];
    negative_count.extend_from_slice(&0_i64.to_be_bytes());
    negative_count.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
    assert!(decode_packet(&negative_count, context(&registries)).is_err());

    let mut malformed_varlong = vec![84];
    malformed_varlong.extend_from_slice(&0_i64.to_be_bytes());
    malformed_varlong.push(1);
    malformed_varlong.extend_from_slice(&[0x80; 10]);
    assert!(decode_packet(&malformed_varlong, context(&registries)).is_err());

    let mut trailing = encode_packet(
        &PlayClientboundPacket::SectionBlocksUpdate(SectionBlocksUpdate {
            section: SectionPos::default(),
            changes: Vec::new(),
        }),
        &registries,
    )
    .unwrap();
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn section_nullable_lookup_survives_decode_until_state_write() {
    let registries = PlayRegistries::default();
    let mut bytes = vec![84];
    bytes.extend_from_slice(&0_i64.to_be_bytes());
    bytes.push(1);
    append_var_i64(&mut bytes, i64::from(32_366) << 12);
    let packet = decode_packet(&bytes, context(&registries)).unwrap();
    let PlayClientboundPacket::SectionBlocksUpdate(section) = &packet else {
        panic!("packet identity changed");
    };
    assert_eq!(section.changes[0].state, None);

    let mut immediate = BlockClientProjection::new(8).unwrap();
    assert_eq!(
        immediate.apply(&packet, 0),
        Err(BlockProjectionError::NullBlockState(BlockPos::default()))
    );

    let mut staged = BlockClientProjection::new(8).unwrap();
    staged.install_block(BlockPos::default(), 1, 1).unwrap();
    staged
        .retain_prediction(BlockPos::default(), 4, 2, [1.0, 2.0, 3.0])
        .unwrap();
    assert_eq!(
        staged.apply(&packet, 0).unwrap(),
        BlockProjectionAction::None
    );
    assert_eq!(
        staged.apply(
            &PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: 4 }),
            0,
        ),
        Err(BlockProjectionError::NullBlockState(BlockPos::default()))
    );
}

#[test]
fn prediction_stages_authoritative_state_and_same_position_advances_sequence() {
    let position = BlockPos::new(1, 64, 2);
    let mut projection = BlockClientProjection::new(8).unwrap();
    projection.install_block(position, 10, 7).unwrap();
    projection
        .retain_prediction(position, 1, 11, [10.0, 64.0, 10.0])
        .unwrap();
    projection
        .apply(
            &PlayClientboundPacket::BlockUpdate(BlockUpdate {
                position,
                state: 12,
            }),
            0,
        )
        .unwrap();
    projection
        .retain_prediction(position, 5, 13, [99.0, 99.0, 99.0])
        .unwrap();

    let stale = projection
        .apply(
            &PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: 1 }),
            0,
        )
        .unwrap();
    assert_eq!(
        stale,
        BlockProjectionAction::PredictionsResolved(Vec::new())
    );
    assert_eq!(projection.block_state(position), Some(13));

    let resolved = projection
        .apply(
            &PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: 5 }),
            0,
        )
        .unwrap();
    let BlockProjectionAction::PredictionsResolved(resolved) = resolved else {
        panic!("ACK action changed");
    };
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].state, 12);
    assert_eq!(
        resolved[0].captured_player_position,
        Some([10.0, 64.0, 10.0])
    );
    assert_eq!(projection.block_state(position), Some(12));
}

#[test]
fn teleport_suppresses_only_captured_position_rollback() {
    let position = BlockPos::new(4, 70, 9);
    let mut projection = BlockClientProjection::new(4).unwrap();
    projection.install_block(position, 1, 1).unwrap();
    projection
        .retain_prediction(position, 7, 2, [4.5, 70.0, 9.5])
        .unwrap();
    projection.record_teleport(7);

    let action = projection
        .apply(
            &PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: 7 }),
            0,
        )
        .unwrap();
    let BlockProjectionAction::PredictionsResolved(resolved) = action else {
        panic!("ACK action changed");
    };
    assert_eq!(resolved[0].captured_player_position, None);
    assert_eq!(projection.block_state(position), Some(1));
}

#[test]
fn prediction_ack_uses_locked_fastutil_zero_then_descending_slot_order() {
    let mut projection = BlockClientProjection::new(8).unwrap();
    for x in [-1, 1, -2, 0] {
        let position = BlockPos::new(x, 0, 0);
        projection.install_block(position, 1, 1).unwrap();
        projection
            .retain_prediction(position, 3, 2, [f64::from(x), 0.0, 0.0])
            .unwrap();
    }
    let action = projection
        .apply(
            &PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: 3 }),
            0,
        )
        .unwrap();
    let BlockProjectionAction::PredictionsResolved(resolved) = action else {
        panic!("ACK action changed");
    };
    assert_eq!(
        resolved
            .into_iter()
            .map(|resolution| resolution.position.x)
            .collect::<Vec<_>>(),
        [0, 1, -2, -1]
    );
}

#[test]
fn section_changes_apply_in_wire_order_with_xzy_relative_layout() {
    let mut projection = BlockClientProjection::new(8).unwrap();
    let packet = PlayClientboundPacket::SectionBlocksUpdate(SectionBlocksUpdate {
        section: SectionPos::new(1, 2, 3),
        changes: vec![
            SectionBlockChange {
                relative_position: 0xabc,
                state: Some(3),
            },
            SectionBlockChange {
                relative_position: 0xabc,
                state: Some(4),
            },
        ],
    });
    projection.apply(&packet, 0).unwrap();
    assert_eq!(projection.block_state(BlockPos::new(26, 44, 59)), Some(4));
}

#[test]
fn block_entity_updates_only_an_existing_exact_type() {
    let position = BlockPos::new(3, 4, 5);
    let mut projection = BlockClientProjection::new(8).unwrap();
    let ignored = PlayClientboundPacket::BlockEntityData(BlockEntityData {
        position,
        type_raw_id: 2,
        update_tag: compound(),
    });
    projection.apply(&ignored, 0).unwrap();
    assert_eq!(projection.block_entity(position), None);

    projection.install_block_entity(position, 1).unwrap();
    projection.apply(&ignored, 0).unwrap();
    assert_eq!(projection.block_entity(position).unwrap().update_tag, None);

    let matched = PlayClientboundPacket::BlockEntityData(BlockEntityData {
        position,
        type_raw_id: 1,
        update_tag: compound(),
    });
    projection.apply(&matched, 0).unwrap();
    assert_eq!(
        projection
            .block_entity(position)
            .unwrap()
            .update_tag
            .as_ref()
            .unwrap()
            .as_bytes(),
        [10, 0]
    );
}

#[test]
fn block_event_uses_current_local_block_not_packet_block() {
    let position = BlockPos::new(8, 9, 10);
    let mut projection = BlockClientProjection::new(4).unwrap();
    projection.install_block(position, 100, 7).unwrap();
    let action = projection
        .apply(
            &PlayClientboundPacket::BlockEvent(BlockEvent {
                position,
                action: 255,
                parameter: 128,
                block_raw_id: 999,
            }),
            0,
        )
        .unwrap();
    assert_eq!(
        action,
        BlockProjectionAction::BlockEvent {
            position,
            current_block_raw_id: Some(7),
            action: 255,
            parameter: 128,
        }
    );
}

#[test]
fn destruction_progress_moves_removes_and_expires_on_locked_scan() {
    let mut projection = BlockClientProjection::new(8).unwrap();
    for packet in [
        BlockDestruction {
            breaker_entity_id: -5,
            position: BlockPos::new(1, 2, 3),
            progress: 2,
        },
        BlockDestruction {
            breaker_entity_id: -5,
            position: BlockPos::new(4, 5, 6),
            progress: 9,
        },
    ] {
        projection
            .apply(&PlayClientboundPacket::BlockDestruction(packet), 20)
            .unwrap();
    }
    assert_eq!(
        projection.destruction(-5).unwrap().position,
        BlockPos::new(4, 5, 6)
    );
    projection.expire_destruction(420);
    assert!(projection.destruction(-5).is_some());

    projection
        .apply(
            &PlayClientboundPacket::BlockDestruction(BlockDestruction {
                breaker_entity_id: 6,
                position: BlockPos::default(),
                progress: 0,
            }),
            19,
        )
        .unwrap();
    projection.expire_destruction(419);
    assert!(projection.destruction(6).is_some());
    projection.expire_destruction(420);
    assert_eq!(projection.destruction(6), None);

    projection
        .apply(
            &PlayClientboundPacket::BlockDestruction(BlockDestruction {
                breaker_entity_id: -5,
                position: BlockPos::default(),
                progress: 10,
            }),
            421,
        )
        .unwrap();
    assert_eq!(projection.destruction(-5), None);
}

#[test]
fn projection_is_bounded_and_rejects_other_packet_families() {
    let mut projection = BlockClientProjection::new(1).unwrap();
    projection.install_block(BlockPos::default(), 0, 0).unwrap();
    assert!(matches!(
        projection.install_block(BlockPos::new(1, 0, 0), 0, 0),
        Err(BlockProjectionError::Full { capacity: 1 })
    ));
    assert!(BlockClientProjection::new(0).is_err());
}
