use std::collections::{BTreeMap, BTreeSet};

use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStack, StackContents,
};
use ferrite_protocol::java_26_2::play::registry::{DATA_COMPONENT_TYPE, ITEM, PlayRegistries};
use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet_with_registries, encode_packet_with_registries,
};
use ferrite_protocol::java_26_2::play::serverbound::container::controls::{
    CarriedSelectionOutcome, CarriedSelectionState, CloseableMenu, ContainerButtonOutcome,
    ContainerCloseOutcome, ContainerCloseSession, CrafterMenuState, CrafterSlotStateOutcome,
    TransferableRemoteSlot, handle_button, handle_crafter_slot_state,
};
use ferrite_protocol::java_26_2::play::serverbound::container::hash::{ComponentHashCache, crc32c};
use ferrite_protocol::java_26_2::play::serverbound::container::packet::{
    ContainerButtonClick, ContainerClick, ContainerClose, ContainerInput,
    ContainerSlotStateChanged, HashedComponentPatch, HashedStack, HashedStackContents,
    SetCarriedItem,
};
use ferrite_protocol::java_26_2::play::serverbound::container::transaction::{
    ContainerActor, ContainerAuthoritativeState, ContainerClickIgnore, ContainerClickOutcome,
    ContainerClientClick, ContainerClientMenu, ContainerMenuTransaction,
};
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_protocol::java_26_2::value::identifier::Identifier;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
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
    registries
}

fn stack(item: &str, count: i32, components: Vec<EncodedComponentValue>) -> ItemStack {
    ItemStack::present(
        id(item),
        count,
        DataComponentPatch {
            added: components,
            removed: Vec::new(),
        },
    )
}

fn component(identity: &str, bytes: &[u8]) -> EncodedComponentValue {
    EncodedComponentValue {
        component: id(identity),
        encoded_value: bytes.to_vec(),
    }
}

fn hashed_stone() -> HashedStack {
    HashedStack::Present(HashedStackContents {
        item: id("minecraft:stone"),
        count: 3,
        components: HashedComponentPatch {
            added: BTreeMap::from([(id("minecraft:custom_data"), 0x0102_0304)]),
            removed: BTreeSet::from([id("minecraft:custom_name")]),
        },
    })
}

fn wrapped(packet: ContainerClick) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::ContainerClick(packet)
}

fn click_packet() -> ContainerClick {
    ContainerClick {
        container_id: 1,
        state_id: 2,
        slot: -999,
        button: -1,
        input: ContainerInput::QuickMove,
        changed_slots: BTreeMap::from([(0, hashed_stone())]),
        carried: HashedStack::Empty,
    }
}

#[test]
fn c3_gold_serverbound_container_locks_all_five_packets() {
    let registries = registries();
    let vectors = [
        (
            PlayServerboundEntryPacket::ContainerButtonClick(ContainerButtonClick {
                container_id: -1,
                button_id: 2,
            }),
            vec![0x11, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x02],
        ),
        (
            wrapped(click_packet()),
            vec![
                0x12, 0x01, 0x02, 0xfc, 0x19, 0xff, 0x01, 0x01, 0x00, 0x00, 0x01, 0x01, 0x03, 0x01,
                0x00, 0x01, 0x02, 0x03, 0x04, 0x01, 0x01, 0x00,
            ],
        ),
        (
            PlayServerboundEntryPacket::ContainerClose(ContainerClose { container_id: 127 }),
            vec![0x13, 0x7f],
        ),
        (
            PlayServerboundEntryPacket::ContainerSlotStateChanged(ContainerSlotStateChanged {
                slot_id: 8,
                container_id: 1,
                new_state: true,
            }),
            vec![0x14, 0x08, 0x01, 0x01],
        ),
        (
            PlayServerboundEntryPacket::SetCarriedItem(SetCarriedItem { slot: -1 }),
            vec![0x35, 0xff, 0xff],
        ),
    ];
    for (packet, golden) in vectors {
        assert_eq!(
            encode_packet_with_registries(packet.clone(), &registries).unwrap(),
            golden
        );
        assert_eq!(
            decode_packet_with_registries(&golden, &registries).unwrap(),
            packet
        );
    }
}

#[test]
fn c3_container_ingress_codecs_normalize_duplicates_and_reject_bounds() {
    let registries = registries();
    let fallback = [
        0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x00, 0x00,
    ];
    let PlayServerboundEntryPacket::ContainerClick(decoded) =
        decode_packet_with_registries(&fallback, &registries).unwrap()
    else {
        panic!("expected click");
    };
    assert_eq!(decoded.input, ContainerInput::Pickup);

    let duplicate_slots = [0x12, 0, 0, 0, 0, 0, 0, 2, 0, 1, 0, 0, 1, 0, 0];
    let PlayServerboundEntryPacket::ContainerClick(decoded) =
        decode_packet_with_registries(&duplicate_slots, &registries).unwrap()
    else {
        panic!("expected click");
    };
    assert_eq!(
        decoded.changed_slots,
        BTreeMap::from([(1, HashedStack::Empty)])
    );

    let negative_count = [0x12, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x0f];
    assert!(decode_packet_with_registries(&negative_count, &registries).is_err());
    let oversized_count = [0x12, 0, 0, 0, 0, 0, 0, 0x81, 0x01];
    assert!(decode_packet_with_registries(&oversized_count, &registries).is_err());
    let unknown_item = [0x12, 0, 0, 0, 0, 0, 0, 0, 1, 3];
    assert!(decode_packet_with_registries(&unknown_item, &registries).is_err());
    assert!(decode_packet_with_registries(&[0x12], &registries).is_err());
    assert!(decode_packet_with_registries(&[0x13, 0, 0], &registries).is_err());
    assert!(matches!(
        ferrite_protocol::java_26_2::play::serverbound::codec::decode_packet(&[
            0x12, 0, 0, 0, 0, 0, 0, 0, 0,
        ]),
        Err(PlayServerboundEntryCodecError::MissingRegistryContext { .. })
    ));
}

#[test]
fn c3_container_hashed_stacks_use_crc32c_exact_shapes_and_bounded_cache() {
    assert_eq!(crc32c(b"123456789"), 0xe306_9283);
    let actual = stack(
        "minecraft:stone",
        3,
        vec![component("minecraft:custom_data", b"value")],
    );
    let mut cache = ComponentHashCache::default();
    let hash = cache.hash_stack(&actual);
    assert!(cache.matches(&hash, &actual));
    assert!(!cache.matches(
        &hash,
        &stack(
            "minecraft:stone",
            4,
            vec![component("minecraft:custom_data", b"value")],
        ),
    ));
    assert_eq!(
        cache.hash_stack(&ItemStack::Present(StackContents {
            item: id("minecraft:air"),
            count: 4,
            components: DataComponentPatch::default(),
        })),
        HashedStack::Empty
    );

    let HashedStack::Present(contents) = hash else {
        panic!("stone must hash as present");
    };
    let collision_evidence = HashedStack::Present(HashedStackContents {
        item: contents.item,
        count: contents.count,
        components: HashedComponentPatch {
            added: contents.components.added,
            removed: contents.components.removed,
        },
    });
    assert!(cache.matches(&collision_evidence, &actual));

    for value in 0..300_u16 {
        let _ = cache.hash_stack(&stack(
            "minecraft:stone",
            1,
            vec![component("minecraft:custom_data", &value.to_be_bytes())],
        ));
    }
    assert_eq!(cache.len(), 256);
}

#[test]
fn c3_container_click_prediction_hashes_only_post_click_differences() {
    let stone = stack("minecraft:stone", 1, Vec::new());
    let diamond = stack("minecraft:diamond", 1, Vec::new());
    let reordered_before = stack(
        "minecraft:stone",
        1,
        vec![
            component("minecraft:custom_data", b"a"),
            component("minecraft:custom_name", b"b"),
        ],
    );
    let mut client = ContainerClientMenu::new(
        7,
        11,
        vec![stone.clone(), reordered_before],
        ItemStack::Empty,
    );
    assert_eq!(
        client
            .predict_click(8, 0, 0, ContainerInput::Pickup, |_, _| {
                panic!("wrong container must not predict")
            })
            .unwrap(),
        ContainerClientClick::IgnoredWrongContainer
    );
    let outcome = client
        .predict_click(7, 0, 1, ContainerInput::Pickup, |slots, carried| {
            slots[0] = stack("minecraft:stone", 2, Vec::new());
            slots[1] = stack(
                "minecraft:stone",
                1,
                vec![
                    component("minecraft:custom_name", b"b"),
                    component("minecraft:custom_data", b"a"),
                ],
            );
            *carried = diamond;
        })
        .unwrap();
    let ContainerClientClick::PredictedAndSend(packet) = outcome else {
        panic!("matching container must emit");
    };
    assert_eq!(packet.state_id, 11);
    assert_eq!(packet.changed_slots.len(), 1);
    assert!(packet.changed_slots.contains_key(&0));
    assert!(matches!(packet.carried, HashedStack::Present(_)));
    assert_eq!(client.predictions, 1);
    assert_eq!(client.slots[0].count(), 2);
}

#[test]
fn c3_container_click_admission_executes_stale_and_gates_other_branches() {
    let state = ContainerAuthoritativeState {
        slots: vec![stack("minecraft:stone", 1, Vec::new())],
        carried: ItemStack::Empty,
        data: vec![3],
    };
    let mut wrong = ContainerMenuTransaction::new(4, 5, state.clone()).unwrap();
    let mut packet = click_packet();
    packet.container_id = 3;
    let mut executed = 0;
    assert_eq!(
        wrong
            .handle_click(
                packet,
                ContainerActor {
                    spectator: false,
                    dead_or_dying: false,
                },
                |_, _, _, _| {
                    executed += 1;
                    Ok(())
                },
            )
            .unwrap(),
        ContainerClickOutcome::Ignored(ContainerClickIgnore::WrongContainer)
    );
    assert_eq!((executed, wrong.idle_resets), (0, 1));

    let mut spectator = ContainerMenuTransaction::new(4, 5, state.clone()).unwrap();
    packet = click_packet();
    packet.container_id = 4;
    assert!(matches!(
        spectator
            .handle_click(
                packet.clone(),
                ContainerActor {
                    spectator: true,
                    dead_or_dying: false,
                },
                |_, _, _, _| panic!("spectator cannot execute"),
            )
            .unwrap(),
        ContainerClickOutcome::Converged {
            click_executed: false,
            packets,
            ..
        } if matches!(packets.as_slice(), [PlayClientboundPacket::ContainerSetContent(_), PlayClientboundPacket::ContainerSetData(_)])
    ));

    let mut invalid = ContainerMenuTransaction::new(4, 5, state.clone()).unwrap();
    invalid.still_valid = false;
    assert_eq!(
        invalid
            .handle_click(
                packet.clone(),
                ContainerActor {
                    spectator: false,
                    dead_or_dying: false,
                },
                |_, _, _, _| panic!("invalid menu cannot execute"),
            )
            .unwrap(),
        ContainerClickOutcome::Ignored(ContainerClickIgnore::InvalidMenu)
    );

    let mut rejected = ContainerMenuTransaction::new(4, 5, state.clone()).unwrap();
    packet.slot = 1;
    assert_eq!(
        rejected
            .handle_click(
                packet.clone(),
                ContainerActor {
                    spectator: false,
                    dead_or_dying: false,
                },
                |_, _, _, _| panic!("slot at size must not execute"),
            )
            .unwrap(),
        ContainerClickOutcome::Ignored(ContainerClickIgnore::RejectedSlot)
    );

    let mut correction = ContainerMenuTransaction::new(4, 5, state.clone()).unwrap();
    packet.slot = 0;
    packet.state_id = 5;
    packet.changed_slots = BTreeMap::from([(0, HashedStack::Empty)]);
    packet.carried = HashedStack::Empty;
    let outcome = correction
        .handle_click(
            packet.clone(),
            ContainerActor {
                spectator: false,
                dead_or_dying: false,
            },
            |slots, carried, data, _| {
                slots[0] = stack("minecraft:diamond", 1, Vec::new());
                *carried = stack("minecraft:diamond", 1, Vec::new());
                data[0] = 4;
                Ok(())
            },
        )
        .unwrap();
    assert!(matches!(
        outcome,
        ContainerClickOutcome::Converged {
            stale_state: false,
            packets,
            ..
        } if matches!(
            packets.as_slice(),
            [
                PlayClientboundPacket::ContainerSetSlot(_),
                PlayClientboundPacket::SetCursorItem(_),
                PlayClientboundPacket::ContainerSetData(_),
            ]
        )
    ));

    let mut stale = ContainerMenuTransaction::new(4, 5, state).unwrap();
    packet.state_id = 4;
    packet.slot = -2;
    packet.changed_slots = BTreeMap::from([(8, HashedStack::Empty)]);
    let outcome = stale
        .handle_click(
            packet,
            ContainerActor {
                spectator: false,
                dead_or_dying: false,
            },
            |slots, _, _, command| {
                assert_eq!(command.slot, -2);
                slots[0] = stack("minecraft:diamond", 1, Vec::new());
                Ok(())
            },
        )
        .unwrap();
    assert!(matches!(
        outcome,
        ContainerClickOutcome::Converged {
            click_executed: true,
            stale_state: true,
            ignored_changed_slots: 1,
            packets,
        } if matches!(packets.first(), Some(PlayClientboundPacket::ContainerSetContent(_)))
    ));
}

#[test]
fn c3_container_controls_apply_independent_button_and_crafter_gates() {
    let state = ContainerAuthoritativeState {
        slots: vec![ItemStack::Empty],
        carried: ItemStack::Empty,
        data: vec![0],
    };
    let mut menu = ContainerMenuTransaction::new(3, 0, state).unwrap();
    assert!(matches!(
        handle_button(
            Some(&mut menu),
            ContainerButtonClick {
                container_id: 3,
                button_id: 9,
            },
            false,
            |button, slots, _, data| {
                assert_eq!(button, 9);
                slots[0] = stack("minecraft:stone", 1, Vec::new());
                data[0] = 2;
                Ok(true)
            },
        )
        .unwrap(),
        ContainerButtonOutcome::Applied { packets }
            if matches!(
                packets.as_slice(),
                [
                    PlayClientboundPacket::ContainerSetSlot(_),
                    PlayClientboundPacket::ContainerSetData(_),
                ]
            )
    ));
    assert_eq!(menu.idle_resets, 1);
    assert_eq!(
        handle_button(
            Some(&mut menu),
            ContainerButtonClick {
                container_id: 4,
                button_id: 0,
            },
            false,
            |_, _, _, _| panic!("wrong container"),
        )
        .unwrap(),
        ContainerButtonOutcome::IgnoredWrongContainer
    );

    let mut crafter = CrafterMenuState {
        container_id: 8,
        real_block_entity: true,
        slots: std::array::from_fn(|_| ItemStack::Empty),
        disabled: [false; 9],
        dirty_writes: 0,
    };
    assert_eq!(
        handle_crafter_slot_state(
            Some(&mut crafter),
            false,
            ContainerSlotStateChanged {
                slot_id: 2,
                container_id: 8,
                new_state: false,
            },
        ),
        CrafterSlotStateOutcome::Applied {
            slot: 2,
            stored_value: 1,
        }
    );
    assert!(crafter.disabled[2]);
    assert_eq!(crafter.dirty_writes, 1);
    crafter.slots[3] = stack("minecraft:stone", 1, Vec::new());
    assert_eq!(
        handle_crafter_slot_state(
            Some(&mut crafter),
            false,
            ContainerSlotStateChanged {
                slot_id: 3,
                container_id: 8,
                new_state: false,
            },
        ),
        CrafterSlotStateOutcome::IgnoredNonempty
    );
}

#[test]
fn c3_container_close_ignores_wire_id_and_transfers_shared_remote_state() {
    let shared = TransferableRemoteSlot {
        backing_container: 10,
        backing_slot: 2,
        exact: Some(stack("minecraft:diamond", 1, Vec::new())),
        predicted: Some(hashed_stone()),
    };
    let mut session = ContainerCloseSession {
        current: Some(CloseableMenu {
            container_id: 9,
            remote_slots: vec![shared],
        }),
        inventory_remote_slots: vec![TransferableRemoteSlot {
            backing_container: 10,
            backing_slot: 2,
            exact: None,
            predicted: None,
        }],
        removals: 0,
    };
    assert_eq!(
        session.handle_close(ContainerClose { container_id: -500 }),
        ContainerCloseOutcome::InventoryMenuSelected {
            closed_container_id: Some(9),
            transferred_slots: 1,
            response_packets: 0,
        }
    );
    assert!(session.current.is_none());
    assert_eq!(session.removals, 1);
    assert!(session.inventory_remote_slots[0].exact.is_some());
    assert!(session.inventory_remote_slots[0].predicted.is_some());
}

#[test]
fn c3_carried_selection_resets_idle_only_for_valid_slots_and_stops_use_on_change() {
    let mut selection = CarriedSelectionState::new(2);
    selection.active_main_hand_use = true;
    assert_eq!(
        selection.handle_set_carried(SetCarriedItem { slot: -1 }),
        CarriedSelectionOutcome::IgnoredInvalidSlot
    );
    assert_eq!(selection.idle_resets, 0);
    assert_eq!(
        selection.handle_set_carried(SetCarriedItem { slot: 2 }),
        CarriedSelectionOutcome::AcceptedUnchanged
    );
    assert_eq!(selection.idle_resets, 1);
    assert!(selection.active_main_hand_use);
    assert_eq!(
        selection.handle_set_carried(SetCarriedItem { slot: 8 }),
        CarriedSelectionOutcome::Changed {
            previous: 2,
            selected: 8,
        }
    );
    assert!(!selection.active_main_hand_use);
    assert_eq!(selection.stopped_main_hand_use, 1);
    assert!(selection.equipment_dirty);
}

#[test]
fn c3_container_convergence_end_to_end_suppresses_matching_prediction() {
    let registries = registries();
    let stone = stack(
        "minecraft:stone",
        1,
        vec![component("minecraft:custom_data", b"x")],
    );
    let diamond = stack("minecraft:diamond", 1, Vec::new());
    let mut client = ContainerClientMenu::new(5, 7, vec![stone.clone()], ItemStack::Empty);
    let ContainerClientClick::PredictedAndSend(packet) = client
        .predict_click(5, 0, 0, ContainerInput::Pickup, |slots, carried| {
            slots[0] = stack(
                "minecraft:stone",
                2,
                vec![component("minecraft:custom_data", b"x")],
            );
            *carried = diamond.clone();
        })
        .unwrap()
    else {
        panic!("matching client menu must emit");
    };
    let body = encode_packet_with_registries(wrapped(packet), &registries).unwrap();
    let PlayServerboundEntryPacket::ContainerClick(packet) =
        decode_packet_with_registries(&body, &registries).unwrap()
    else {
        panic!("expected click");
    };
    let state = ContainerAuthoritativeState {
        slots: vec![stone],
        carried: ItemStack::Empty,
        data: Vec::new(),
    };
    let mut server = ContainerMenuTransaction::new(5, 7, state).unwrap();
    let outcome = server
        .handle_click(
            packet,
            ContainerActor {
                spectator: false,
                dead_or_dying: false,
            },
            |slots, carried, _, _| {
                slots[0] = stack(
                    "minecraft:stone",
                    2,
                    vec![component("minecraft:custom_data", b"x")],
                );
                *carried = diamond;
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        outcome,
        ContainerClickOutcome::Converged {
            click_executed: true,
            stale_state: false,
            ignored_changed_slots: 0,
            packets: Vec::new(),
        }
    );
    assert_eq!(server.suppression_windows, 1);
}
