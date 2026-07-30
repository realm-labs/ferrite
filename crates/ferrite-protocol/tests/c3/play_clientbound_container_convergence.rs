use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::container::packet::{
    ContainerClose, ContainerSetContent, ContainerSetData, ContainerSetSlot, OpenScreen,
    SetCursorItem, SetPlayerInventory,
};
use ferrite_protocol::java_26_2::play::clientbound::container::projection::{
    ContainerClientProjection, ContainerProjectionAction, ContainerProjectionError, MenuDefinition,
};
use ferrite_protocol::java_26_2::play::clientbound::container::publication::{
    ContainerPublisher, MenuSnapshot,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::context::{
    ComponentValueDecoder, ComponentValueError, PlayDecodeContext,
};
use ferrite_protocol::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStack, StackContents,
};
use ferrite_protocol::java_26_2::play::registry::{
    DATA_COMPONENT_TYPE, ITEM, MENU, PlayRegistries,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;
use ferrite_protocol::java_26_2::wire::primitive::WireReader;

struct OneByteComponent;

impl ComponentValueDecoder for OneByteComponent {
    fn decode_value(
        &self,
        component: &Identifier,
        reader: &mut WireReader<'_>,
    ) -> Result<Vec<u8>, ComponentValueError> {
        reader
            .read_u8()
            .map(|value| vec![value])
            .map_err(|error| ComponentValueError::Malformed {
                component: component.clone(),
                reason: error.to_string(),
            })
    }
}

static COMPONENTS: OneByteComponent = OneByteComponent;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn title(value: &str) -> TextComponentNbt {
    TextComponentNbt::literal(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(ITEM),
        vec![
            id("minecraft:air"),
            id("minecraft:stone"),
            id("minecraft:diamond"),
        ],
    );
    registries.insert(
        id(DATA_COMPONENT_TYPE),
        vec![id("minecraft:custom_data"), id("minecraft:custom_name")],
    );
    registries.insert(
        id(MENU),
        vec![
            id("minecraft:generic_9x1"),
            id("minecraft:furnace"),
            id("minecraft:generic_3x3"),
        ],
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

fn stone(count: i32) -> ItemStack {
    ItemStack::present(id("minecraft:stone"), count, DataComponentPatch::default())
}

fn diamond(count: i32) -> ItemStack {
    ItemStack::present(
        id("minecraft:diamond"),
        count,
        DataComponentPatch::default(),
    )
}

fn definitions() -> BTreeMap<Identifier, MenuDefinition> {
    BTreeMap::from([
        (
            id("minecraft:generic_9x1"),
            MenuDefinition {
                slots: 9,
                data_slots: 2,
                has_screen: true,
            },
        ),
        (
            id("minecraft:furnace"),
            MenuDefinition {
                slots: 3,
                data_slots: 4,
                has_screen: true,
            },
        ),
        (
            id("minecraft:generic_3x3"),
            MenuDefinition {
                slots: 9,
                data_slots: 0,
                has_screen: false,
            },
        ),
    ])
}

fn open(container_id: i32, menu_type: &str) -> PlayClientboundPacket {
    PlayClientboundPacket::OpenScreen(OpenScreen {
        container_id,
        menu_type: id(menu_type),
        title: title("menu"),
    })
}

fn snapshot(slots: Vec<ItemStack>, carried: ItemStack, data: Vec<i16>) -> MenuSnapshot {
    MenuSnapshot {
        menu_type: id("minecraft:generic_9x1"),
        title: title("menu"),
        slots,
        carried,
        data,
    }
}

#[test]
fn seven_packet_goldens_lock_ids_and_field_order() {
    let registries = registries();
    let packets = [
        (
            PlayClientboundPacket::ContainerClose(ContainerClose { container_id: 0 }),
            vec![17, 0],
        ),
        (
            PlayClientboundPacket::ContainerSetContent(ContainerSetContent {
                container_id: 0,
                state_id: 0,
                slots: Vec::new(),
                carried: ItemStack::Empty,
            }),
            vec![18, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::ContainerSetData(ContainerSetData {
                container_id: 0,
                property_id: 0,
                value: 0,
            }),
            vec![19, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::ContainerSetSlot(ContainerSetSlot {
                container_id: 0,
                state_id: 0,
                slot: 0,
                item: ItemStack::Empty,
            }),
            vec![20, 0, 0, 0, 0, 0],
        ),
        (
            open(0, "minecraft:generic_9x1"),
            vec![59, 0, 0, 8, 0, 4, 109, 101, 110, 117],
        ),
        (
            PlayClientboundPacket::SetCursorItem(SetCursorItem {
                item: ItemStack::Empty,
            }),
            vec![96, 0],
        ),
        (
            PlayClientboundPacket::SetPlayerInventory(SetPlayerInventory {
                slot: 0,
                item: ItemStack::Empty,
            }),
            vec![108, 0, 0],
        ),
    ];

    for (packet, golden) in packets {
        assert_eq!(encode_packet(&packet, &registries).unwrap(), golden);
        assert_eq!(
            decode_packet(&golden, context(&registries)).unwrap(),
            packet
        );
    }
}

#[test]
fn signed_fields_and_component_stacks_round_trip_without_stack_clamping() {
    let registries = registries();
    let component_stack = ItemStack::present(
        id("minecraft:stone"),
        i32::MAX,
        DataComponentPatch {
            added: vec![EncodedComponentValue {
                component: id("minecraft:custom_data"),
                encoded_value: vec![0xa5],
            }],
            removed: vec![id("minecraft:custom_name")],
        },
    );
    let packets = [
        PlayClientboundPacket::ContainerClose(ContainerClose {
            container_id: i32::MIN,
        }),
        PlayClientboundPacket::ContainerSetContent(ContainerSetContent {
            container_id: i32::MIN,
            state_id: i32::MAX,
            slots: vec![component_stack.clone()],
            carried: stone(65),
        }),
        PlayClientboundPacket::ContainerSetData(ContainerSetData {
            container_id: -1,
            property_id: i16::MIN,
            value: i16::MAX,
        }),
        PlayClientboundPacket::ContainerSetSlot(ContainerSetSlot {
            container_id: i32::MAX,
            state_id: i32::MIN,
            slot: i16::MAX,
            item: component_stack,
        }),
        PlayClientboundPacket::SetPlayerInventory(SetPlayerInventory {
            slot: i32::MAX,
            item: stone(64),
        }),
    ];

    for packet in packets {
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
}

#[test]
fn optional_air_and_nonpositive_counts_normalize_to_canonical_empty() {
    let registries = registries();
    let forged_positive_air = vec![18, 0, 7, 1, 1, 0, 1, 0, 0, 0x5a, 0];
    let decoded = decode_packet(&forged_positive_air, context(&registries)).unwrap();
    let PlayClientboundPacket::ContainerSetContent(content) = &decoded else {
        panic!("wrong packet");
    };
    assert_eq!(content.slots, vec![ItemStack::Empty]);
    assert_eq!(
        encode_packet(&decoded, &registries).unwrap(),
        vec![18, 0, 7, 1, 0, 0]
    );

    let forged_negative = vec![96, 0xff, 0xff, 0xff, 0xff, 0x0f];
    assert_eq!(
        decode_packet(&forged_negative, context(&registries)).unwrap(),
        PlayClientboundPacket::SetCursorItem(SetCursorItem {
            item: ItemStack::Empty,
        })
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::SetCursorItem(SetCursorItem {
                item: ItemStack::Present(StackContents {
                    item: id("minecraft:air"),
                    count: 99,
                    components: DataComponentPatch::default(),
                }),
            }),
            &registries,
        )
        .unwrap(),
        vec![96, 0]
    );
}

#[test]
fn component_patch_replaces_duplicates_and_preserves_signed_count_faults() {
    let registries = registries();
    let removal_wins = vec![96, 1, 1, 2, 1, 0, 0x11, 0, 0x22, 0];
    let decoded = decode_packet(&removal_wins, context(&registries)).unwrap();
    let PlayClientboundPacket::SetCursorItem(SetCursorItem {
        item: ItemStack::Present(contents),
    }) = decoded
    else {
        panic!("expected present cursor stack");
    };
    assert!(contents.components.added.is_empty());
    assert_eq!(
        contents.components.removed,
        vec![id("minecraft:custom_data")]
    );

    let negative_added = vec![96, 1, 1, 0xff, 0xff, 0xff, 0xff, 0x0f, 1, 0];
    let decoded = decode_packet(&negative_added, context(&registries)).unwrap();
    let PlayClientboundPacket::SetCursorItem(SetCursorItem {
        item: ItemStack::Present(contents),
    }) = decoded
    else {
        panic!("expected present cursor stack");
    };
    assert_eq!(
        contents.components.removed,
        vec![id("minecraft:custom_data")]
    );

    let negative_capacity = vec![96, 1, 1, 0xfe, 0xff, 0xff, 0xff, 0x0f, 1, 0];
    assert!(decode_packet(&negative_capacity, context(&registries)).is_err());
}

#[test]
fn malformed_lengths_registries_components_nbt_and_residual_bytes_fail_closed() {
    let registries = registries();
    let vectors = [
        vec![17, 0x80, 0x80, 0x80, 0x80, 0x80, 0],
        vec![18, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x0f],
        vec![18, 0, 0, 1, 1, 9, 0, 0],
        vec![18, 0, 0, 1, 1, 1, 1, 0, 0],
        vec![59, 0, 9, 8, 0, 0],
        vec![59, 0, 0, 13],
        vec![96],
        vec![108, 0, 0, 1],
    ];
    for vector in vectors {
        assert!(
            decode_packet(&vector, context(&registries)).is_err(),
            "{vector:?}"
        );
    }

    let duplicate = PlayClientboundPacket::SetCursorItem(SetCursorItem {
        item: ItemStack::present(
            id("minecraft:stone"),
            1,
            DataComponentPatch {
                added: vec![EncodedComponentValue {
                    component: id("minecraft:custom_data"),
                    encoded_value: vec![1],
                }],
                removed: vec![id("minecraft:custom_data")],
            },
        ),
    });
    assert!(matches!(
        encode_packet(&duplicate, &registries),
        Err(PlayClientboundCodecError::Container(_))
    ));
}

#[test]
fn open_close_and_full_content_follow_locked_client_rules() {
    let mut client = ContainerClientProjection::new(46, definitions(), 64).unwrap();
    let missing = open(4, "minecraft:generic_3x3");
    assert_eq!(
        client.apply(&missing).unwrap(),
        ContainerProjectionAction::MissingScreen {
            menu_type: id("minecraft:generic_3x3")
        }
    );
    assert_eq!(client.active_menu().container_id, 0);

    assert_eq!(
        client.apply(&open(7, "minecraft:generic_9x1")).unwrap(),
        ContainerProjectionAction::ScreenOpened { container_id: 7 }
    );
    client
        .apply(&PlayClientboundPacket::ContainerSetContent(
            ContainerSetContent {
                container_id: 7,
                state_id: -91,
                slots: vec![stone(1), diamond(2)],
                carried: stone(3),
            },
        ))
        .unwrap();
    assert_eq!(client.active_menu().state_id, -91);
    assert_eq!(client.active_menu().slots[0], stone(1));
    assert_eq!(client.active_menu().slots[2], ItemStack::Empty);

    client
        .apply(&PlayClientboundPacket::ContainerSetContent(
            ContainerSetContent {
                container_id: 99,
                state_id: 3,
                slots: vec![diamond(1)],
                carried: ItemStack::Empty,
            },
        ))
        .unwrap();
    assert_eq!(client.active_menu().slots[0], stone(1));

    assert_eq!(
        client
            .apply(&PlayClientboundPacket::ContainerClose(ContainerClose {
                container_id: -999,
            }))
            .unwrap(),
        ContainerProjectionAction::ScreenClosed
    );
    assert_eq!(client.active_menu().container_id, 0);
}

#[test]
fn client_faults_after_exact_partial_mutation_and_preserves_handler_order() {
    let mut client = ContainerClientProjection::new(46, definitions(), 64).unwrap();
    client.apply(&open(5, "minecraft:generic_9x1")).unwrap();
    let error = client
        .apply(&PlayClientboundPacket::ContainerSetContent(
            ContainerSetContent {
                container_id: 5,
                state_id: 88,
                slots: vec![stone(1); 10],
                carried: diamond(1),
            },
        ))
        .unwrap_err();
    assert!(matches!(
        error,
        ContainerProjectionError::InvalidSlot { slot: 9, .. }
    ));
    assert_eq!(client.active_menu().slots, vec![stone(1); 9]);
    assert_eq!(client.active_menu().carried, ItemStack::Empty);
    assert_eq!(client.active_menu().state_id, 0);

    let bad_slot = PlayClientboundPacket::ContainerSetSlot(ContainerSetSlot {
        container_id: 5,
        state_id: 4,
        slot: -1,
        item: diamond(1),
    });
    assert!(client.apply(&bad_slot).is_err());
    assert_eq!(client.tutorial_observations(), 1);

    let bad_data = PlayClientboundPacket::ContainerSetData(ContainerSetData {
        container_id: 5,
        property_id: -1,
        value: 3,
    });
    assert!(client.apply(&bad_data).is_err());
}

#[test]
fn creative_cursor_slot_and_player_inventory_quirks_are_reproduced() {
    let mut client = ContainerClientProjection::new(46, definitions(), 64).unwrap();
    client.apply(&open(7, "minecraft:generic_9x1")).unwrap();
    client.set_creative_screen_visible(true);

    client
        .apply(&PlayClientboundPacket::ContainerSetSlot(ContainerSetSlot {
            container_id: 99,
            state_id: 12,
            slot: 36,
            item: stone(2),
        }))
        .unwrap();
    assert_eq!(client.inventory_menu().slots[36], stone(2));
    assert_eq!(client.inventory_menu().pop_times[36], 5);
    assert_eq!(client.inventory_menu().state_id, 12);
    assert_eq!(client.local_broadcasts(), 1);

    client
        .apply(&PlayClientboundPacket::SetCursorItem(SetCursorItem {
            item: diamond(1),
        }))
        .unwrap();
    assert_eq!(client.active_menu().carried, ItemStack::Empty);

    for slot in [0, 35, 36, 39, 40, 41, 42, 43] {
        client
            .apply(&PlayClientboundPacket::SetPlayerInventory(
                SetPlayerInventory {
                    slot,
                    item: stone(slot + 1),
                },
            ))
            .unwrap();
    }
    assert_eq!(client.player_inventory().ordinary[0], stone(1));
    assert_eq!(client.player_inventory().ordinary[35], stone(36));
    assert_eq!(client.player_inventory().armor[0], stone(37));
    assert_eq!(client.player_inventory().armor[3], stone(40));
    assert_eq!(client.player_inventory().offhand, stone(41));
    assert_eq!(client.player_inventory().body, stone(42));
    assert_eq!(client.player_inventory().saddle, stone(43));
    assert!(
        client
            .apply(&PlayClientboundPacket::SetPlayerInventory(
                SetPlayerInventory {
                    slot: -1,
                    item: stone(1),
                },
            ))
            .is_err()
    );
    assert_eq!(client.tutorial_observations(), 11);
}

#[test]
fn publisher_orders_open_full_delta_cursor_data_and_wraps_ids() {
    let mut publisher = ContainerPublisher::default();
    let initial = snapshot(
        vec![ItemStack::Empty, stone(1), ItemStack::Empty],
        ItemStack::Empty,
        vec![4, 5],
    );
    let opened = publisher.open(initial.clone()).unwrap();
    assert!(matches!(opened[0], PlayClientboundPacket::OpenScreen(_)));
    assert!(matches!(
        opened[1],
        PlayClientboundPacket::ContainerSetContent(ContainerSetContent { state_id: 1, .. })
    ));
    assert!(matches!(
        opened[2],
        PlayClientboundPacket::ContainerSetData(ContainerSetData { property_id: 0, .. })
    ));
    assert!(matches!(
        opened[3],
        PlayClientboundPacket::ContainerSetData(ContainerSetData { property_id: 1, .. })
    ));

    let changed = snapshot(vec![diamond(1), stone(1), stone(2)], diamond(3), vec![7, 8]);
    let delta = publisher.broadcast_changes(&changed).unwrap();
    assert!(matches!(
        delta[0],
        PlayClientboundPacket::ContainerSetSlot(ContainerSetSlot {
            slot: 0,
            state_id: 2,
            ..
        })
    ));
    assert!(matches!(
        delta[1],
        PlayClientboundPacket::ContainerSetSlot(ContainerSetSlot {
            slot: 2,
            state_id: 3,
            ..
        })
    ));
    assert!(matches!(delta[2], PlayClientboundPacket::SetCursorItem(_)));
    assert!(matches!(
        delta[3],
        PlayClientboundPacket::ContainerSetData(ContainerSetData { property_id: 0, .. })
    ));
    assert!(matches!(
        delta[4],
        PlayClientboundPacket::ContainerSetData(ContainerSetData { property_id: 1, .. })
    ));
    assert_eq!(publisher.current_state_id(), Some(3));
    assert!(publisher.broadcast_changes(&changed).unwrap().is_empty());

    let second = publisher.open(initial.clone()).unwrap();
    assert!(matches!(
        second[0],
        PlayClientboundPacket::ContainerClose(ContainerClose { container_id: 1 })
    ));
    assert!(matches!(
        second[1],
        PlayClientboundPacket::OpenScreen(OpenScreen {
            container_id: 2,
            ..
        })
    ));
    for _ in 0..99 {
        publisher.open(initial.clone()).unwrap();
    }
    assert_eq!(publisher.current_container_id(), Some(1));

    for _ in 1..=32_767 {
        publisher.broadcast_full(initial.clone()).unwrap();
    }
    assert_eq!(publisher.current_state_id(), Some(0));
}

#[test]
fn canonical_publication_round_trips_and_converges_end_to_end() {
    let registries = registries();
    let mut publisher = ContainerPublisher::default();
    let mut client = ContainerClientProjection::new(46, definitions(), 64).unwrap();
    let initial = snapshot(
        vec![
            stone(1),
            ItemStack::Empty,
            diamond(2),
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
        ],
        stone(4),
        vec![11, 12],
    );
    for packet in publisher.open(initial.clone()).unwrap() {
        let encoded = encode_packet(&packet, &registries).unwrap();
        let decoded = decode_packet(&encoded, context(&registries)).unwrap();
        client.apply(&decoded).unwrap();
    }
    assert_eq!(client.active_menu().slots, initial.slots);
    assert_eq!(client.active_menu().carried, initial.carried);
    assert_eq!(client.active_menu().data, initial.data);

    let converged = snapshot(
        vec![
            diamond(5),
            stone(6),
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
            ItemStack::Empty,
        ],
        ItemStack::Empty,
        vec![-1, i16::MAX],
    );
    for packet in publisher.broadcast_changes(&converged).unwrap() {
        let encoded = encode_packet(&packet, &registries).unwrap();
        let decoded = decode_packet(&encoded, context(&registries)).unwrap();
        client.apply(&decoded).unwrap();
    }
    assert_eq!(client.active_menu().slots, converged.slots);
    assert_eq!(client.active_menu().carried, converged.carried);
    assert_eq!(client.active_menu().data, converged.data);
    assert_eq!(
        client.active_menu().state_id,
        publisher.current_state_id().unwrap()
    );
}
