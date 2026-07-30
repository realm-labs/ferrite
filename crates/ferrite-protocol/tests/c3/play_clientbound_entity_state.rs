use std::collections::BTreeMap;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::login::profile::ProfileProperty;
use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::particle::{
    Particle, ParticleOptions,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_state::accessor_registry::{
    declarations, schema_for_hierarchy,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_state::metadata::{
    ACCESSOR_DECLARATION_COUNT, ACCESSOR_TABLE_SHA1, GlobalPos, HumanoidArm, MetadataEntry,
    MetadataSerializer, MetadataValue, PlayerSkinModel, PlayerSkinPatch, ResolvableProfile,
    SERIALIZER_COUNT, SERIALIZER_TABLE_SHA1, VillagerData,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_state::packet::{
    AttributeModifier, AttributeOperation, AttributeSnapshot, EquipmentEntry, EquipmentSlot,
    SetEntityData, SetEntityLink, SetEquipment, SetPassengers, UpdateAttributes,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_state::projection::{
    EntityStateAction, EntityStateCapabilities, EntityStateClientProjection, EntityStateProjection,
    EntityStateProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_state::publication::{
    ATTRIBUTE_AUDIENCE, ENTITY_STATE_PAIRING_ORDER, EQUIPMENT_AUDIENCE, EntityStateAudience,
    EntityStatePairingStep, LEASH_PUBLICATION_ORDER, LeashPublicationStep, METADATA_AUDIENCE,
    RIDER_START_ORDER, RiderTransitionStep, collect_equipment_changes, pairing_equipment,
    passenger_tracker_receives_broadcast,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::context::{
    ComponentValueDecoder, ComponentValueError, PlayDecodeContext,
};
use ferrite_protocol::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStack,
};
use ferrite_protocol::java_26_2::play::registry::{
    ATTRIBUTE, CAT_SOUND_VARIANT, CAT_VARIANT, CHICKEN_SOUND_VARIANT, CHICKEN_VARIANT,
    COW_SOUND_VARIANT, COW_VARIANT, DATA_COMPONENT_TYPE, FROG_VARIANT, ITEM, PAINTING_VARIANT,
    PIG_SOUND_VARIANT, PIG_VARIANT, PlayRegistries, VILLAGER_PROFESSION, VILLAGER_TYPE,
    WOLF_SOUND_VARIANT, WOLF_VARIANT, ZOMBIE_NAUTILUS_VARIANT,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;
use ferrite_protocol::java_26_2::wire::primitive::WireReader;

struct OneByteComponents;

impl ComponentValueDecoder for OneByteComponents {
    fn decode_value(
        &self,
        _component: &Identifier,
        reader: &mut WireReader<'_>,
    ) -> Result<Vec<u8>, ComponentValueError> {
        Ok(vec![reader.read_u8().map_err(|error| {
            ComponentValueError::Malformed {
                component: id("minecraft:test_component"),
                reason: error.to_string(),
            }
        })?])
    }
}

static COMPONENTS: OneByteComponents = OneByteComponents;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    for registry in [
        ATTRIBUTE,
        CAT_SOUND_VARIANT,
        CAT_VARIANT,
        CHICKEN_SOUND_VARIANT,
        CHICKEN_VARIANT,
        COW_SOUND_VARIANT,
        COW_VARIANT,
        FROG_VARIANT,
        PAINTING_VARIANT,
        PIG_SOUND_VARIANT,
        PIG_VARIANT,
        VILLAGER_PROFESSION,
        VILLAGER_TYPE,
        WOLF_SOUND_VARIANT,
        WOLF_VARIANT,
        ZOMBIE_NAUTILUS_VARIANT,
    ] {
        let path = registry.replace("minecraft:", "test:");
        registries.insert(id(registry), vec![id(&path)]);
    }
    registries.insert(id(ITEM), vec![id("minecraft:air"), id("minecraft:stone")]);
    registries.insert(
        id(DATA_COMPONENT_TYPE),
        vec![id("minecraft:test_component")],
    );
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn metadata_packet(values: Vec<MetadataEntry>) -> PlayClientboundPacket {
    PlayClientboundPacket::SetEntityData(SetEntityData {
        entity_id: 1,
        values,
    })
}

fn entry(slot: u8, value: MetadataValue) -> MetadataEntry {
    MetadataEntry {
        slot,
        serializer: value.serializer(),
        value,
    }
}

fn holder(serializer: MetadataSerializer, registry: &'static str) -> MetadataValue {
    MetadataValue::Holder {
        serializer,
        identity: id(&registry.replace("minecraft:", "test:")),
    }
}

fn living(local_player: bool) -> EntityStateProjection {
    EntityStateProjection::new(EntityStateCapabilities {
        living: true,
        leashable: true,
        boat: false,
        local_player,
        riding_allowed: true,
    })
}

#[test]
fn c3_gold_entity_state_locks_all_five_packet_bodies() {
    let registries = registries();
    assert_eq!(
        encode_packet(&metadata_packet(Vec::new()), &registries).unwrap(),
        [0x63, 1, 0xff]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::SetEntityLink(SetEntityLink {
                source_entity_id: 0,
                destination_entity_id: 0,
            }),
            &registries,
        )
        .unwrap(),
        [0x64, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::SetEquipment(SetEquipment {
                entity_id: 0,
                entries: vec![EquipmentEntry {
                    slot: EquipmentSlot::Mainhand,
                    stack: ItemStack::Empty,
                }],
            }),
            &registries,
        )
        .unwrap(),
        [0x66, 0, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::SetPassengers(SetPassengers {
                vehicle_id: 0,
                passenger_ids: Vec::new(),
            }),
            &registries,
        )
        .unwrap(),
        [0x6b, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::UpdateAttributes(UpdateAttributes {
                entity_id: 0,
                snapshots: Vec::new(),
            }),
            &registries,
        )
        .unwrap(),
        [0x83, 0x01, 0, 0]
    );
}

#[test]
fn c3_metadata_all_43_serializers_round_trip_through_locked_dispatch() {
    let registries = registries();
    let component = TextComponentNbt::literal("test").unwrap();
    let holder_values = [
        (MetadataSerializer::CatVariant, CAT_VARIANT),
        (MetadataSerializer::CatSoundVariant, CAT_SOUND_VARIANT),
        (MetadataSerializer::CowVariant, COW_VARIANT),
        (MetadataSerializer::CowSoundVariant, COW_SOUND_VARIANT),
        (MetadataSerializer::WolfVariant, WOLF_VARIANT),
        (MetadataSerializer::WolfSoundVariant, WOLF_SOUND_VARIANT),
        (MetadataSerializer::FrogVariant, FROG_VARIANT),
        (MetadataSerializer::PigVariant, PIG_VARIANT),
        (MetadataSerializer::PigSoundVariant, PIG_SOUND_VARIANT),
        (MetadataSerializer::ChickenVariant, CHICKEN_VARIANT),
        (
            MetadataSerializer::ChickenSoundVariant,
            CHICKEN_SOUND_VARIANT,
        ),
        (
            MetadataSerializer::ZombieNautilusVariant,
            ZOMBIE_NAUTILUS_VARIANT,
        ),
    ];
    let mut values = vec![
        MetadataValue::Byte(-1),
        MetadataValue::Int(i32::MIN),
        MetadataValue::Long(i64::MIN),
        MetadataValue::Float(-3.5),
        MetadataValue::String("state".to_owned()),
        MetadataValue::Component(component.clone()),
        MetadataValue::OptionalComponent(Some(component)),
        MetadataValue::ItemStack(ItemStack::Empty),
        MetadataValue::Boolean(true),
        MetadataValue::Rotations([1.0, 2.0, 3.0]),
        MetadataValue::BlockPos(BlockPos::new(-1, 2, 3)),
        MetadataValue::OptionalBlockPos(Some(BlockPos::new(4, 5, 6))),
        MetadataValue::Direction(5),
        MetadataValue::OptionalLivingEntityReference(Some(7)),
        MetadataValue::BlockState(Some(32_365)),
        MetadataValue::OptionalBlockState(Some(1)),
        MetadataValue::Particle(Particle {
            raw_type: 0,
            options: ParticleOptions::Simple,
        }),
        MetadataValue::Particles(vec![Particle {
            raw_type: 0,
            options: ParticleOptions::Simple,
        }]),
        MetadataValue::VillagerData(VillagerData {
            villager_type: id("test:villager_type"),
            profession: id("test:villager_profession"),
            level: -1,
        }),
        MetadataValue::OptionalUnsignedInt(Some(7)),
        MetadataValue::Pose(17),
    ];
    values.extend(
        holder_values
            .into_iter()
            .map(|(serializer, registry)| holder(serializer, registry)),
    );
    values.extend([
        MetadataValue::OptionalGlobalPos(Some(GlobalPos {
            dimension: id("minecraft:overworld"),
            position: BlockPos::new(1, 2, 3),
        })),
        holder(MetadataSerializer::PaintingVariant, PAINTING_VARIANT),
        MetadataValue::EnumState {
            serializer: MetadataSerializer::SnifferState,
            value: 6,
        },
        MetadataValue::EnumState {
            serializer: MetadataSerializer::ArmadilloState,
            value: 3,
        },
        MetadataValue::EnumState {
            serializer: MetadataSerializer::CopperGolemState,
            value: 4,
        },
        MetadataValue::EnumState {
            serializer: MetadataSerializer::WeatheringCopperState,
            value: 3,
        },
        MetadataValue::Vector3([4.0, 5.0, 6.0]),
        MetadataValue::Quaternion([1.0, 2.0, 3.0, 4.0]),
        MetadataValue::ResolvableProfile(ResolvableProfile::Partial {
            name: Some("alex".to_owned()),
            uuid: Some(8),
            properties: vec![ProfileProperty {
                name: "textures".to_owned(),
                value: "value".to_owned(),
                signature: Some("signature".to_owned()),
            }],
            skin: PlayerSkinPatch {
                body: Some(id("minecraft:body")),
                cape: None,
                elytra: Some(id("minecraft:elytra")),
                model: Some(PlayerSkinModel::Slim),
            },
        }),
        MetadataValue::HumanoidArm(HumanoidArm::Right),
    ]);
    assert_eq!(values.len(), SERIALIZER_COUNT as usize);

    let packet = metadata_packet(
        values
            .into_iter()
            .enumerate()
            .map(|(slot, value)| entry(slot as u8, value))
            .collect(),
    );
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );
    assert_eq!(ACCESSOR_DECLARATION_COUNT, 221);
    assert_eq!(declarations().len(), ACCESSOR_DECLARATION_COUNT);
    assert_eq!(
        ACCESSOR_TABLE_SHA1,
        "b489eec18fc1981ebfb7ac97c54a4485fe2f938a"
    );
    assert_eq!(
        SERIALIZER_TABLE_SHA1,
        "96047ad220ac7064e205594f3222d182c87591d7"
    );
}

#[test]
fn c3_metadata_accessor_lock_composes_only_the_selected_source_hierarchy() {
    let schema = schema_for_hierarchy(&[
        "net.minecraft.world.entity.Entity",
        "net.minecraft.world.entity.LivingEntity",
        "net.minecraft.world.entity.Mob",
        "net.minecraft.world.entity.AgeableMob",
    ])
    .unwrap();
    assert_eq!(schema.len(), 18);
    assert_eq!(schema[&0], MetadataSerializer::Byte);
    assert_eq!(schema[&8], MetadataSerializer::Byte);
    assert_eq!(schema[&15], MetadataSerializer::Byte);
    assert_eq!(schema[&16], MetadataSerializer::Boolean);
    assert_eq!(schema[&17], MetadataSerializer::Boolean);

    assert!(
        schema_for_hierarchy(&[
            "net.minecraft.world.entity.animal.feline.Cat",
            "net.minecraft.world.entity.animal.wolf.Wolf",
        ])
        .is_err()
    );

    let mut entity = EntityStateProjection::new(EntityStateCapabilities::default());
    entity
        .install_locked_metadata_hierarchy(
            &["net.minecraft.world.entity.Entity"],
            BTreeMap::from([
                (0, MetadataValue::Byte(0)),
                (1, MetadataValue::Int(300)),
                (2, MetadataValue::OptionalComponent(None)),
                (3, MetadataValue::Boolean(false)),
                (4, MetadataValue::Boolean(false)),
                (5, MetadataValue::Boolean(false)),
                (6, MetadataValue::Pose(0)),
                (7, MetadataValue::Int(0)),
            ]),
        )
        .unwrap();
    assert_eq!(entity.metadata.len(), 8);
}

#[test]
fn c3_metadata_normalizes_by_id_policies_and_rejects_structural_faults() {
    let registries = registries();
    let decoded = decode_packet(
        &[
            0x63, 1, 0, 12, 0xff, 0xff, 0xff, 0xff, 0x0f, 1, 20, 99, 2, 35, 99, 3, 38, 99, 4, 42,
            99, 0xff,
        ],
        context(&registries),
    )
    .unwrap();
    let PlayClientboundPacket::SetEntityData(packet) = decoded else {
        panic!("expected metadata packet");
    };
    assert_eq!(packet.values[0].value, MetadataValue::Direction(5));
    assert_eq!(packet.values[1].value, MetadataValue::Pose(0));
    assert_eq!(
        packet.values[2].value,
        MetadataValue::EnumState {
            serializer: MetadataSerializer::SnifferState,
            value: 0,
        }
    );
    assert_eq!(
        packet.values[3].value,
        MetadataValue::EnumState {
            serializer: MetadataSerializer::WeatheringCopperState,
            value: 3,
        }
    );
    assert_eq!(
        packet.values[4].value,
        MetadataValue::HumanoidArm(HumanoidArm::Left)
    );

    assert!(decode_packet(&[0x63, 1, 0, 43], context(&registries)).is_err());
    assert!(decode_packet(&[0x63, 1, 0, 0, 7], context(&registries)).is_err());
    assert!(decode_packet(&[0x63, 1, 0xff, 0], context(&registries)).is_err());
}

#[test]
fn c3_equipment_passenger_and_attribute_codec_boundaries_are_strict() {
    let registries = registries();
    assert!(decode_packet(&[0x66, 1], context(&registries)).is_err());
    assert!(decode_packet(&[0x66, 1, 8, 0], context(&registries)).is_err());
    assert!(decode_packet(&[0x66, 1, 0x80, 0], context(&registries)).is_err());
    assert!(
        decode_packet(
            &[0x6b, 1, 0xff, 0xff, 0xff, 0xff, 0x0f],
            context(&registries)
        )
        .is_err()
    );
    assert!(decode_packet(&[0x6b, 1, 2, 1], context(&registries)).is_err());
    assert!(
        decode_packet(
            &[0x83, 1, 1, 0xff, 0xff, 0xff, 0xff, 0x0f],
            context(&registries)
        )
        .is_err()
    );
    assert!(
        encode_packet(
            &PlayClientboundPacket::SetEquipment(SetEquipment {
                entity_id: 1,
                entries: Vec::new(),
            }),
            &registries
        )
        .is_err()
    );

    let packet = PlayClientboundPacket::UpdateAttributes(UpdateAttributes {
        entity_id: -1,
        snapshots: vec![AttributeSnapshot {
            attribute: id("test:attribute"),
            base: f64::from_bits(0x7ff8_0000_0000_0042),
            modifiers: vec![AttributeModifier {
                identity: id("test:modifier"),
                amount: f64::NEG_INFINITY,
                operation: AttributeOperation::AddValue,
            }],
        }],
    });
    let encoded = encode_packet(&packet, &registries).unwrap();
    let decoded = decode_packet(&encoded, context(&registries)).unwrap();
    let PlayClientboundPacket::UpdateAttributes(decoded) = decoded else {
        panic!("expected attributes");
    };
    assert_eq!(decoded.snapshots[0].base.to_bits(), 0x7ff8_0000_0000_0042);
    assert_eq!(decoded.snapshots[0].modifiers[0].amount, f64::NEG_INFINITY);
}

#[test]
fn c3_equipment_item_patch_preserves_count_and_air_normalizes_to_empty() {
    let registries = registries();
    let stack = ItemStack::present(
        id("minecraft:stone"),
        99,
        DataComponentPatch {
            added: vec![EncodedComponentValue {
                component: id("minecraft:test_component"),
                encoded_value: vec![0x5a],
            }],
            removed: Vec::new(),
        },
    );
    let packet = PlayClientboundPacket::SetEquipment(SetEquipment {
        entity_id: 1,
        entries: vec![EquipmentEntry {
            slot: EquipmentSlot::Saddle,
            stack,
        }],
    });
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );

    let decoded = decode_packet(&[0x66, 1, 0, 1, 0, 0, 0], context(&registries)).unwrap();
    let PlayClientboundPacket::SetEquipment(packet) = decoded else {
        panic!("expected equipment");
    };
    assert!(packet.entries[0].stack.is_empty());
    assert_eq!(
        encode_packet(&PlayClientboundPacket::SetEquipment(packet), &registries).unwrap(),
        [0x66, 1, 0, 0]
    );
}

#[test]
fn c3_metadata_application_is_ordered_and_faults_wrong_runtime_schema() {
    let mut projection = EntityStateClientProjection::default();
    let mut entity = living(false);
    entity.define_metadata(0, MetadataValue::Byte(0));
    entity.define_metadata(1, MetadataValue::Int(0));
    projection.insert_entity(1, entity);

    assert_eq!(
        projection
            .apply(&metadata_packet(vec![
                entry(0, MetadataValue::Byte(1)),
                entry(0, MetadataValue::Byte(2)),
                entry(1, MetadataValue::Int(3)),
            ]))
            .unwrap(),
        EntityStateAction::MetadataApplied { entries: 3 }
    );
    let entity = projection.entity(1).unwrap();
    assert_eq!(entity.metadata[&0].current, MetadataValue::Byte(2));
    assert_eq!(entity.metadata_callback_log, [0, 0, 1]);
    assert_eq!(entity.metadata_aggregate_callback_log, [vec![0, 0, 1]]);

    assert!(matches!(
        projection.apply(&metadata_packet(vec![entry(2, MetadataValue::Byte(1))])),
        Err(EntityStateProjectionError::MissingMetadataSlot {
            entity_id: 1,
            slot: 2
        })
    ));
    assert!(matches!(
        projection.apply(&metadata_packet(vec![entry(0, MetadataValue::Int(1))])),
        Err(EntityStateProjectionError::MetadataSerializerMismatch { .. })
    ));
    assert_eq!(
        projection
            .apply(&PlayClientboundPacket::SetEntityData(SetEntityData {
                entity_id: 99,
                values: vec![entry(0, MetadataValue::Byte(1))],
            }))
            .unwrap(),
        EntityStateAction::Ignored
    );
}

#[test]
fn c3_metadata_dirty_and_pairing_snapshots_preserve_default_return_updates() {
    let mut projection = EntityStateClientProjection::default();
    let mut entity = living(false);
    entity.define_metadata(1, MetadataValue::Int(0));
    entity.define_metadata(0, MetadataValue::Byte(0));
    projection.insert_entity(1, entity);

    projection
        .set_local_metadata(1, 1, MetadataValue::Int(5))
        .unwrap();
    assert_eq!(projection.pack_dirty_metadata(1).len(), 1);
    assert_eq!(
        projection.metadata_pairing_values(1),
        [entry(1, MetadataValue::Int(5))]
    );
    projection
        .set_local_metadata(1, 1, MetadataValue::Int(0))
        .unwrap();
    assert_eq!(
        projection.pack_dirty_metadata(1),
        [entry(1, MetadataValue::Int(0))]
    );
    assert!(projection.metadata_pairing_values(1).is_empty());
    assert!(projection.pack_dirty_metadata(1).is_empty());
}

#[test]
fn c3_attributes_replace_sanitize_skip_and_fault_by_runtime_type() {
    let mut projection = EntityStateClientProjection::default();
    let mut entity = living(false);
    entity.define_attribute(id("test:attribute"), 1.0, 0.0, 10.0, true);
    projection.insert_entity(1, entity);
    projection.insert_entity(
        2,
        EntityStateProjection::new(EntityStateCapabilities::default()),
    );
    let update = PlayClientboundPacket::UpdateAttributes(UpdateAttributes {
        entity_id: 1,
        snapshots: vec![
            AttributeSnapshot {
                attribute: id("test:missing"),
                base: 3.0,
                modifiers: Vec::new(),
            },
            AttributeSnapshot {
                attribute: id("test:attribute"),
                base: 99.0,
                modifiers: vec![AttributeModifier {
                    identity: id("test:first"),
                    amount: 2.0,
                    operation: AttributeOperation::AddMultipliedBase,
                }],
            },
        ],
    });
    assert_eq!(
        projection.apply(&update).unwrap(),
        EntityStateAction::AttributesApplied {
            snapshots: 1,
            skipped: 1
        }
    );
    let instance = &projection.entity(1).unwrap().attributes[&id("test:attribute")];
    assert_eq!(instance.base, 10.0);
    assert_eq!(instance.modifiers.len(), 1);
    assert_eq!(projection.syncable_attributes(1).len(), 1);
    projection
        .mark_attribute_to_sync(1, id("test:attribute"))
        .unwrap();
    assert_eq!(projection.take_attributes_to_sync(1).len(), 1);
    assert!(projection.take_attributes_to_sync(1).is_empty());

    assert!(matches!(
        projection.apply(&PlayClientboundPacket::UpdateAttributes(UpdateAttributes {
            entity_id: 2,
            snapshots: Vec::new(),
        })),
        Err(EntityStateProjectionError::AttributesRequireLiving { entity_id: 2 })
    ));

    let duplicate = AttributeModifier {
        identity: id("test:duplicate"),
        amount: 1.0,
        operation: AttributeOperation::AddValue,
    };
    assert!(matches!(
        projection.apply(&PlayClientboundPacket::UpdateAttributes(UpdateAttributes {
            entity_id: 1,
            snapshots: vec![AttributeSnapshot {
                attribute: id("test:attribute"),
                base: 2.0,
                modifiers: vec![
                    duplicate.clone(),
                    AttributeModifier {
                        amount: 9.0,
                        ..duplicate
                    },
                ],
            }],
        })),
        Err(EntityStateProjectionError::DuplicateAttributeModifier { .. })
    ));
    assert_eq!(
        projection.entity(1).unwrap().attributes[&id("test:attribute")].modifiers
            [&id("test:duplicate")]
            .amount,
        1.0
    );
}

#[test]
fn c3_equipment_replaces_in_wire_order_and_publication_handles_hand_swap() {
    let mut projection = EntityStateClientProjection::default();
    projection.insert_entity(1, living(false));
    projection.insert_entity(
        2,
        EntityStateProjection::new(EntityStateCapabilities::default()),
    );
    let stone = ItemStack::present(id("minecraft:stone"), 1, DataComponentPatch::default());
    let packet = PlayClientboundPacket::SetEquipment(SetEquipment {
        entity_id: 1,
        entries: vec![
            EquipmentEntry {
                slot: EquipmentSlot::Mainhand,
                stack: stone.clone(),
            },
            EquipmentEntry {
                slot: EquipmentSlot::Mainhand,
                stack: ItemStack::Empty,
            },
        ],
    });
    projection.apply(&packet).unwrap();
    assert!(projection.entity(1).unwrap().equipment[&EquipmentSlot::Mainhand].is_empty());
    assert_eq!(
        projection
            .apply(&PlayClientboundPacket::SetEquipment(SetEquipment {
                entity_id: 2,
                entries: vec![EquipmentEntry {
                    slot: EquipmentSlot::Head,
                    stack: stone.clone(),
                }],
            }))
            .unwrap(),
        EntityStateAction::Ignored
    );

    let mut remembered = BTreeMap::from([
        (EquipmentSlot::Mainhand, stone.clone()),
        (EquipmentSlot::Offhand, ItemStack::Empty),
    ]);
    let current = BTreeMap::from([
        (EquipmentSlot::Mainhand, ItemStack::Empty),
        (EquipmentSlot::Offhand, stone.clone()),
        (EquipmentSlot::Head, stone),
    ]);
    let publication = collect_equipment_changes(&current, &mut remembered);
    assert!(publication.hand_swap_event);
    assert_eq!(publication.changes.len(), 1);
    assert_eq!(publication.changes[0].slot, EquipmentSlot::Head);
    assert_eq!(pairing_equipment(&current).len(), 2);
}

#[test]
fn c3_passengers_replace_sequentially_clear_marker_and_onboard_once() {
    let mut projection = EntityStateClientProjection::default();
    let mut boat = EntityStateProjection::new(EntityStateCapabilities {
        boat: true,
        riding_allowed: true,
        ..EntityStateCapabilities::default()
    });
    boat.yaw = 70.0;
    projection.insert_entity(1, boat);
    projection.insert_entity(2, living(true));
    projection.insert_entity(3, living(false));
    projection.insert_entity(4, living(false));
    projection.set_removed_player_vehicle_id(Some(1));

    projection
        .apply(&PlayClientboundPacket::SetPassengers(SetPassengers {
            vehicle_id: 3,
            passenger_ids: vec![2],
        }))
        .unwrap();
    projection
        .apply(&PlayClientboundPacket::SetPassengers(SetPassengers {
            vehicle_id: 1,
            passenger_ids: vec![2, 2, 99],
        }))
        .unwrap();
    assert!(projection.entity(3).unwrap().passengers.is_empty());
    assert_eq!(projection.entity(2).unwrap().vehicle, Some(1));
    assert_eq!(projection.entity(2).unwrap().yaw, 70.0);
    assert_eq!(projection.removed_player_vehicle_id(), None);
    assert_eq!(projection.riding_onboarding_presentations(), 1);

    projection
        .apply(&PlayClientboundPacket::SetPassengers(SetPassengers {
            vehicle_id: 1,
            passenger_ids: Vec::new(),
        }))
        .unwrap();
    projection
        .apply(&PlayClientboundPacket::SetPassengers(SetPassengers {
            vehicle_id: 1,
            passenger_ids: vec![2],
        }))
        .unwrap();
    assert_eq!(projection.riding_onboarding_presentations(), 1);
    assert_eq!(
        projection
            .apply(&PlayClientboundPacket::SetPassengers(SetPassengers {
                vehicle_id: 99,
                passenger_ids: vec![2],
            }))
            .unwrap(),
        EntityStateAction::Ignored
    );
}

#[test]
fn c3_leash_link_retains_delayed_ids_and_resolves_lazily() {
    let mut projection = EntityStateClientProjection::default();
    projection.insert_entity(1, living(false));
    projection
        .apply(&PlayClientboundPacket::SetEntityLink(SetEntityLink {
            source_entity_id: 1,
            destination_entity_id: 9,
        }))
        .unwrap();
    assert_eq!(
        projection.entity(1).unwrap().delayed_leash_holder_id,
        Some(9)
    );
    assert_eq!(projection.resolved_leash_holder(1), None);
    projection.insert_entity(9, living(false));
    assert_eq!(projection.resolved_leash_holder(1), Some(9));
    projection
        .apply(&PlayClientboundPacket::SetEntityLink(SetEntityLink {
            source_entity_id: 1,
            destination_entity_id: 0,
        }))
        .unwrap();
    assert_eq!(projection.entity(1).unwrap().delayed_leash_holder_id, None);
}

#[test]
fn c3_entity_state_publication_audiences_and_orders_are_locked() {
    assert_eq!(
        METADATA_AUDIENCE,
        EntityStateAudience::TrackingPlayersAndSelf
    );
    assert_eq!(
        ATTRIBUTE_AUDIENCE,
        EntityStateAudience::TrackingPlayersAndSelf
    );
    assert_eq!(EQUIPMENT_AUDIENCE, EntityStateAudience::TrackingPlayers);
    assert_eq!(
        ENTITY_STATE_PAIRING_ORDER,
        [
            EntityStatePairingStep::UpdateDataBeforeSync,
            EntityStatePairingStep::NondefaultMetadata,
            EntityStatePairingStep::SyncableAttributes,
            EntityStatePairingStep::NonemptyEquipment,
            EntityStatePairingStep::OwnPassengers,
            EntityStatePairingStep::VehiclePassengers,
            EntityStatePairingStep::Leash,
        ]
    );
    assert!(passenger_tracker_receives_broadcast(7, &[1], &[2]));
    assert!(!passenger_tracker_receives_broadcast(1, &[1], &[]));
    assert_eq!(
        RIDER_START_ORDER,
        [
            RiderTransitionStep::PositionRider,
            RiderTransitionStep::PlayerPositionChallenge,
            RiderTransitionStep::LivingVehicleEffects,
            RiderTransitionStep::FullPassengerList,
        ]
    );
    assert_eq!(
        LEASH_PUBLICATION_ORDER,
        [
            LeashPublicationStep::MutateRelation,
            LeashPublicationStep::BroadcastLink,
        ]
    );
}
