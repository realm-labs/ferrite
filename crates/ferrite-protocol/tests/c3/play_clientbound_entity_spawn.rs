use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_spawn::packet::{
    AddEntity, RemoveEntities,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_spawn::projection::{
    AttachmentDirection, EntitySpawnAction, EntitySpawnClientProjection,
    EntitySpawnProjectionError, SpawnAdmission, SpawnSound,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_spawn::publication::{
    PairingAdmission, PairingContent, PairingStep, UNPAIRING_ORDER, UnpairingStep,
    effective_tracking_range, pairing_allowed, pairing_plan,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::identifier::Identifier;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn add(entity_id: i32, uuid: u128, entity_type: &str, data: i32) -> PlayClientboundPacket {
    PlayClientboundPacket::AddEntity(Box::new(AddEntity {
        entity_id,
        uuid,
        entity_type: id(entity_type),
        position: Vector3::default(),
        motion: Vector3::default(),
        pitch: 0,
        yaw: 0,
        head_yaw: 0,
        data,
    }))
}

fn remove(entity_ids: &[i32]) -> PlayClientboundPacket {
    PlayClientboundPacket::RemoveEntities(RemoveEntities {
        entity_ids: entity_ids.to_vec(),
    })
}

fn raw_add(type_raw_id: i32) -> Vec<u8> {
    let mut body = vec![1, 0];
    body.extend_from_slice(&[0; 16]);
    write_var_i32(&mut body, type_raw_id);
    body.extend_from_slice(&[0; 24]);
    body.extend_from_slice(&[0, 0, 0, 0, 0]);
    body
}

fn write_var_i32(output: &mut Vec<u8>, value: i32) {
    let mut remaining = value as u32;
    loop {
        if remaining & !0x7f == 0 {
            output.push(remaining as u8);
            return;
        }
        output.push((remaining as u8 & 0x7f) | 0x80);
        remaining >>= 7;
    }
}

#[test]
fn c3_gold_entity_spawn_locks_both_packet_bodies() {
    let registries = PlayRegistries::default();
    let add = add(0, 0, "minecraft:acacia_boat", 0);
    let mut expected_add = vec![1, 0];
    expected_add.extend_from_slice(&[0; 16]);
    expected_add.push(0);
    expected_add.extend_from_slice(&[0; 24]);
    expected_add.extend_from_slice(&[0, 0, 0, 0, 0]);
    assert_eq!(encode_packet(&add, &registries).unwrap(), expected_add);
    assert_eq!(
        decode_packet(&expected_add, context(&registries)).unwrap(),
        add
    );

    let remove = remove(&[]);
    assert_eq!(encode_packet(&remove, &registries).unwrap(), [0x4d, 0]);
    assert_eq!(
        decode_packet(&[0x4d, 0], context(&registries)).unwrap(),
        remove
    );
}

#[test]
fn c3_entity_spawn_static_registry_covers_every_raw_id_and_defaults_to_pig() {
    let registries = PlayRegistries::default();
    for raw_id in 0..158 {
        let bytes = raw_add(raw_id);
        let decoded = decode_packet(&bytes, context(&registries)).unwrap();
        assert_eq!(encode_packet(&decoded, &registries).unwrap(), bytes);
    }

    for raw_id in [-1, 158, i32::MAX] {
        let decoded = decode_packet(&raw_add(raw_id), context(&registries)).unwrap();
        let PlayClientboundPacket::AddEntity(packet) = decoded else {
            panic!("expected add entity");
        };
        assert_eq!(packet.entity_type, id("minecraft:pig"));
        let canonical =
            encode_packet(&PlayClientboundPacket::AddEntity(packet), &registries).unwrap();
        assert_eq!(
            decode_packet(&canonical, context(&registries)).unwrap(),
            add(0, 0, "minecraft:pig", 0)
        );
    }

    for (raw_id, identity) in [
        (0, "minecraft:acacia_boat"),
        (100, "minecraft:pig"),
        (143, "minecraft:warden"),
        (156, "minecraft:player"),
        (157, "minecraft:fishing_bobber"),
    ] {
        let PlayClientboundPacket::AddEntity(packet) =
            decode_packet(&raw_add(raw_id), context(&registries)).unwrap()
        else {
            panic!("expected add entity");
        };
        assert_eq!(packet.entity_type, id(identity));
    }
}

#[test]
fn c3_entity_spawn_codec_preserves_signed_and_ieee_fields() {
    let registries = PlayRegistries::default();
    let packet = PlayClientboundPacket::AddEntity(Box::new(AddEntity {
        entity_id: -7,
        uuid: 0x0102_0304_0506_0708_1112_1314_1516_1718,
        entity_type: id("minecraft:warden"),
        position: Vector3 {
            x: f64::NEG_INFINITY,
            y: f64::from_bits(0x7ff8_0000_0000_002a),
            z: -0.0,
        },
        motion: Vector3 {
            x: 0.25,
            y: -0.5,
            z: 0.75,
        },
        pitch: i8::MIN,
        yaw: -1,
        head_yaw: i8::MAX,
        data: i32::MIN,
    }));
    let encoded = encode_packet(&packet, &registries).unwrap();
    let PlayClientboundPacket::AddEntity(decoded) =
        decode_packet(&encoded, context(&registries)).unwrap()
    else {
        panic!("expected add entity");
    };
    let PlayClientboundPacket::AddEntity(expected) = packet else {
        unreachable!();
    };
    assert_eq!(decoded.entity_id, expected.entity_id);
    assert_eq!(decoded.uuid, expected.uuid);
    assert_eq!(decoded.entity_type, expected.entity_type);
    assert_eq!(decoded.position.x.to_bits(), expected.position.x.to_bits());
    assert_eq!(decoded.position.y.to_bits(), expected.position.y.to_bits());
    assert_eq!(decoded.position.z.to_bits(), expected.position.z.to_bits());
    assert_eq!(decoded.pitch, expected.pitch);
    assert_eq!(decoded.yaw, expected.yaw);
    assert_eq!(decoded.head_yaw, expected.head_yaw);
    assert_eq!(decoded.data, expected.data);
    assert!((decoded.motion.x - 0.25).abs() < 0.001);
    assert!((decoded.motion.y + 0.5).abs() < 0.001);
    assert!((decoded.motion.z - 0.75).abs() < 0.001);

    let unknown = add(1, 2, "test:unknown", 0);
    assert!(matches!(
        encode_packet(&unknown, &registries),
        Err(PlayClientboundCodecError::EntitySpawn(_))
    ));
}

#[test]
fn c3_remove_entities_accepts_negative_empty_and_ordered_signed_ids() {
    let registries = PlayRegistries::default();
    let mut negative = vec![0x4d];
    write_var_i32(&mut negative, -1);
    assert_eq!(
        decode_packet(&negative, context(&registries)).unwrap(),
        remove(&[])
    );
    let mut trailing = negative;
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
    assert!(decode_packet(&[0x4d, 2], context(&registries)).is_err());

    let packet = remove(&[-1, 7, 7, i32::MIN]);
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );
}

#[test]
fn c3_spawn_construction_clamps_living_but_not_nonliving_state() {
    let mut projection = EntitySpawnClientProjection::default();
    let mut pig = match add(1, 11, "minecraft:pig", 0) {
        PlayClientboundPacket::AddEntity(packet) => packet,
        _ => unreachable!(),
    };
    pig.position = Vector3 {
        x: 40_000_000.0,
        y: 90_000_000.0,
        z: -40_000_000.0,
    };
    pig.pitch = i8::MIN;
    pig.yaw = 64;
    pig.head_yaw = -64;
    projection
        .apply(&PlayClientboundPacket::AddEntity(pig))
        .unwrap();
    let pig = projection.entity(1).unwrap();
    assert_eq!(pig.position.x, 30_000_000.0);
    assert_eq!(pig.position.y, 90_000_000.0);
    assert_eq!(pig.position.z, -30_000_000.0);
    assert_eq!(pig.pitch, -90.0);
    assert_eq!(pig.body_yaw, -90.0);
    assert_eq!(pig.head_yaw, -90.0);

    let mut frame = match add(2, 22, "minecraft:item_frame", 11) {
        PlayClientboundPacket::AddEntity(packet) => packet,
        _ => unreachable!(),
    };
    frame.position = Vector3 {
        x: 40_000_000.25,
        y: -1.1,
        z: -40_000_000.75,
    };
    frame.head_yaw = 64;
    projection
        .apply(&PlayClientboundPacket::AddEntity(frame))
        .unwrap();
    let frame = projection.entity(2).unwrap();
    assert_eq!(frame.position.x, 40_000_000.25);
    assert_eq!(frame.head_yaw, 0.0);
    assert_eq!(frame.attachment_direction, Some(AttachmentDirection::East));
    assert_eq!(frame.block_anchor, Some((40_000_000, -2, -40_000_001)));
}

#[test]
fn c3_player_info_and_factory_admission_skip_without_replacing() {
    let mut projection = EntitySpawnClientProjection::default();
    assert_eq!(
        projection
            .apply(&add(1, 10, "minecraft:player", 0))
            .unwrap(),
        EntitySpawnAction::SkippedConstruction
    );
    assert!(projection.entity(1).is_none());

    projection.set_admission(
        id("minecraft:pig"),
        SpawnAdmission {
            required_features_enabled: false,
            ..SpawnAdmission::default()
        },
    );
    assert_eq!(
        projection.apply(&add(2, 20, "minecraft:pig", 0)).unwrap(),
        EntitySpawnAction::SkippedConstruction
    );
    assert!(projection.entity(2).is_none());

    projection.add_player_info(10);
    projection
        .apply(&add(1, 10, "minecraft:player", 0))
        .unwrap();
    assert!(projection.entity(1).unwrap().player);
    assert!(projection.seen_player(10));
}

#[test]
fn c3_spawn_data_handles_directions_blocks_warden_and_fishing_owner() {
    let mut projection = EntitySpawnClientProjection::default();
    projection
        .apply(&add(1, 1, "minecraft:falling_block", 32_366))
        .unwrap();
    projection.apply(&add(2, 2, "minecraft:warden", 1)).unwrap();
    projection
        .apply(&add(3, 3, "minecraft:fishing_bobber", 99))
        .unwrap();
    assert_eq!(projection.entity(1).unwrap().block_state, Some(0));
    assert!(projection.entity(2).unwrap().emerging_warden);
    assert!(projection.entity(3).unwrap().discarded);

    projection.add_player_info(40);
    projection
        .apply(&add(4, 40, "minecraft:player", 0))
        .unwrap();
    projection
        .apply(&add(5, 50, "minecraft:fishing_bobber", 4))
        .unwrap();
    let bobber = projection.entity(5).unwrap();
    assert_eq!(bobber.owner_entity_id, Some(4));
    assert!(!bobber.discarded);

    assert!(matches!(
        projection.apply(&add(6, 60, "minecraft:painting", 0)),
        Err(EntitySpawnProjectionError::VerticalPaintingDirection {
            direction: AttachmentDirection::Down
        })
    ));
    projection
        .apply(&add(6, 60, "minecraft:painting", -3))
        .unwrap();
    assert_eq!(
        projection.entity(6).unwrap().attachment_direction,
        Some(AttachmentDirection::South)
    );
}

#[test]
fn c3_same_id_recreation_observes_old_owner_and_duplicate_uuid_stays_unregistered() {
    let mut projection = EntitySpawnClientProjection::default();
    projection.apply(&add(7, 70, "minecraft:pig", 0)).unwrap();
    let action = projection.apply(&add(7, 71, "minecraft:arrow", 7)).unwrap();
    assert_eq!(
        action,
        EntitySpawnAction::Inserted {
            replaced_same_id: true,
            uuid_registered: true
        }
    );
    let arrow = projection.entity(7).unwrap();
    assert_eq!(arrow.owner_entity_id, Some(7));
    assert_eq!(arrow.owner_uuid_seen_during_recreation, Some(70));

    projection.apply(&add(8, 80, "minecraft:pig", 0)).unwrap();
    let action = projection.apply(&add(9, 80, "minecraft:cow", 0)).unwrap();
    assert_eq!(
        action,
        EntitySpawnAction::Inserted {
            replaced_same_id: false,
            uuid_registered: false
        }
    );
    assert!(projection.entity(9).is_some());
    assert_eq!(projection.entity_id_by_uuid(80), Some(8));
}

#[test]
fn c3_specialized_spawn_state_matches_locked_construction_hooks() {
    let mut projection = EntitySpawnClientProjection::default();
    for (entity_id, entity_type) in [
        (1, "minecraft:ender_dragon"),
        (20, "minecraft:shulker"),
        (21, "minecraft:llama_spit"),
        (22, "minecraft:minecart"),
        (23, "minecraft:bee"),
        (24, "minecraft:shulker_bullet"),
        (25, "minecraft:leash_knot"),
    ] {
        projection
            .apply(&add(entity_id, entity_id as u128, entity_type, 0))
            .unwrap();
    }
    assert_eq!(
        projection.entity(1).unwrap().dragon_part_ids,
        (2..=9).collect::<Vec<_>>()
    );
    assert_eq!(projection.entity(20).unwrap().body_yaw, 0.0);
    assert_eq!(
        projection
            .entity(21)
            .unwrap()
            .llama_spit_particle_multipliers,
        vec![0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    );
    assert!(
        projection
            .entity(21)
            .unwrap()
            .movement_reapplied_after_construction
    );
    assert_eq!(
        projection.entity(22).unwrap().post_add_sound,
        Some(SpawnSound::MinecartRolling)
    );
    assert_eq!(
        projection.entity(23).unwrap().post_add_sound,
        Some(SpawnSound::BeeNonaggressiveFlying)
    );
    assert!(
        projection
            .entity(24)
            .unwrap()
            .movement_reapplied_after_construction
    );
    assert_eq!(projection.entity(25).unwrap().block_anchor, Some((0, 0, 0)));
}

#[test]
fn c3_removal_is_order_sensitive_and_clears_relationships_and_debug_state() {
    let mut projection = EntitySpawnClientProjection::default();
    projection.add_player_info(30);
    projection.apply(&add(1, 10, "minecraft:pig", 0)).unwrap();
    projection.apply(&add(2, 20, "minecraft:pig", 0)).unwrap();
    projection
        .apply(&add(3, 30, "minecraft:player", 0))
        .unwrap();
    projection.set_local_player_entity_id(3);
    projection.set_passengers(1, vec![2]);
    projection.set_passengers(2, vec![3]);
    projection.mark_debug_subscription(1);

    assert_eq!(
        projection.apply(&remove(&[1, 1, 99])).unwrap(),
        EntitySpawnAction::Removed { count: 1 }
    );
    assert_eq!(projection.removed_player_vehicle_id(), Some(1));
    assert!(!projection.has_debug_subscription(1));
    assert_eq!(projection.entity(2).unwrap().vehicle, None);

    projection.apply(&add(1, 11, "minecraft:pig", 0)).unwrap();
    assert_eq!(projection.removed_player_vehicle_id(), None);

    let mut reverse = EntitySpawnClientProjection::default();
    reverse.add_player_info(30);
    reverse.apply(&add(1, 10, "minecraft:pig", 0)).unwrap();
    reverse.apply(&add(3, 30, "minecraft:player", 0)).unwrap();
    reverse.set_local_player_entity_id(3);
    reverse.set_passengers(1, vec![3]);
    reverse.apply(&remove(&[3, 1])).unwrap();
    assert_eq!(reverse.removed_player_vehicle_id(), None);
    assert!(reverse.seen_player(30));
}

#[test]
fn c3_former_vehicle_marker_clears_before_failed_recreation() {
    let mut projection = EntitySpawnClientProjection::default();
    projection.add_player_info(20);
    projection.apply(&add(1, 10, "minecraft:pig", 0)).unwrap();
    projection
        .apply(&add(2, 20, "minecraft:player", 0))
        .unwrap();
    projection.set_local_player_entity_id(2);
    projection.set_passengers(1, vec![2]);
    projection.apply(&remove(&[1])).unwrap();
    assert_eq!(projection.removed_player_vehicle_id(), Some(1));

    assert!(
        projection
            .apply(&add(1, 30, "minecraft:painting", 0))
            .is_err()
    );
    assert_eq!(projection.removed_player_vehicle_id(), None);
}

#[test]
fn c3_pairing_range_audience_bundle_and_unpair_order_are_explicit() {
    let admission = PairingAdmission {
        horizontal_distance_squared: 63.5_f64.powi(2),
        effective_range_blocks: 80,
        view_distance_blocks: 64,
        broadcast_allowed: true,
        chunk_tracked: true,
        viewer_is_entity: false,
    };
    assert!(pairing_allowed(admission));
    assert!(!pairing_allowed(PairingAdmission {
        horizontal_distance_squared: 64.1_f64.powi(2),
        ..admission
    }));
    assert!(!pairing_allowed(PairingAdmission {
        viewer_is_entity: true,
        ..admission
    }));
    assert!(!pairing_allowed(PairingAdmission {
        horizontal_distance_squared: f64::NAN,
        ..admission
    }));
    assert_eq!(
        effective_tracking_range(32, &[48, 80, 64], |range| range * 2),
        160
    );

    assert_eq!(
        pairing_plan(PairingContent {
            player_info: true,
            metadata: true,
            attributes: false,
            equipment: true,
            own_passengers: true,
            vehicle_passengers: false,
            leash: true,
        }),
        [
            PairingStep::UpdateDataBeforeSync,
            PairingStep::PlayerInfo,
            PairingStep::AddEntity,
            PairingStep::Metadata,
            PairingStep::Equipment,
            PairingStep::OwnPassengers,
            PairingStep::Leash,
            PairingStep::SendBundle,
            PairingStep::StartSeenByPlayer,
        ]
    );
    assert_eq!(
        UNPAIRING_ORDER,
        [
            UnpairingStep::StopSeenByPlayer,
            UnpairingStep::SendSingleEntityRemoval,
        ]
    );
}
