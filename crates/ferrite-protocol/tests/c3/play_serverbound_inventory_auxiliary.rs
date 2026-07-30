use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::serverbound::inventory_auxiliary::advancement::{
    AdvancementClientEvent, AdvancementClientScreen, AdvancementNode, AdvancementTabState,
};
use ferrite_protocol::java_26_2::play::serverbound::inventory_auxiliary::book::{
    BookClient, BookFilterOutcome, BookFilterOutput, BookFilterService, BookInventory, BookStack,
    FilterableText, TextFilterResult,
};
use ferrite_protocol::java_26_2::play::serverbound::inventory_auxiliary::bundle::{
    BundleClientEvent, BundleClientMenu, BundleContentsProjection, BundleMenuStack,
    BundleSelectionOutcome, displayed_item_count, handle_bundle_selection,
};
use ferrite_protocol::java_26_2::play::serverbound::inventory_auxiliary::codec::InventoryAuxiliaryCodecError;
use ferrite_protocol::java_26_2::play::serverbound::inventory_auxiliary::packet::{
    BundleItemSelected, EditBook, SeenAdvancements,
};
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_protocol::java_26_2::value::identifier::Identifier;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn bundle(slot: i32, selected: i32) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::BundleItemSelected(BundleItemSelected { slot, selected })
}

fn edit(slot: i32, pages: &[&str], title: Option<&str>) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::EditBook(EditBook {
        slot,
        pages: pages.iter().map(ToString::to_string).collect(),
        title: title.map(ToString::to_string),
    })
}

fn opened(identifier: &str) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::SeenAdvancements(SeenAdvancements::OpenedTab(id(identifier)))
}

fn filter(raw: &str, filtered: Option<&str>) -> TextFilterResult {
    TextFilterResult {
        raw: raw.to_owned(),
        filtered: filtered.map(ToString::to_string),
    }
}

fn writable(name: &str, pages: &[&str]) -> BookStack {
    BookStack::writable(
        id(name),
        pages
            .iter()
            .map(|page| FilterableText {
                raw: (*page).to_owned(),
                filtered: None,
            })
            .collect(),
    )
}

fn bundle_stack(entries: usize) -> BundleMenuStack {
    BundleMenuStack {
        contents: Some(BundleContentsProjection::from_component_stream(
            (0..entries).map(|entry| entry as u64).collect(),
        )),
    }
}

#[test]
fn c3_gold_serverbound_inventory_auxiliary_locks_all_four_frames() {
    let vectors = [
        (
            bundle(0, -1),
            vec![0x03, 0x00, 0xff, 0xff, 0xff, 0xff, 0x0f],
        ),
        (edit(0, &[], None), vec![0x18, 0x00, 0x00, 0x00]),
        (
            opened("minecraft:story/root"),
            [vec![0x32, 0x00, 0x14], b"minecraft:story/root".to_vec()].concat(),
        ),
        (
            PlayServerboundEntryPacket::SeenAdvancements(SeenAdvancements::ClosedScreen),
            vec![0x32, 0x01],
        ),
    ];
    for (packet, body) in vectors {
        assert_eq!(encode_packet(packet.clone()).unwrap(), body);
        assert_eq!(decode_packet(&body).unwrap(), packet);
    }
}

#[test]
fn c3_inventory_aux_codecs_enforce_every_bound_and_complete_consumption() {
    for packet in [
        bundle(i32::MIN, -1),
        bundle(i32::MAX, i32::MAX),
        edit(-1, &["page", "a\u{1f680}"], Some("title")),
        opened("minecraft:a//../b"),
    ] {
        let encoded = encode_packet(packet.clone()).unwrap();
        assert_eq!(decode_packet(&encoded).unwrap(), packet);
    }

    assert!(matches!(
        encode_packet(bundle(0, -2)),
        Err(PlayServerboundEntryCodecError::InventoryAuxiliary(
            InventoryAuxiliaryCodecError::InvalidBundleSelection { selected: -2 }
        ))
    ));
    assert!(decode_packet(&[0x03, 0x00, 0xfe, 0xff, 0xff, 0xff, 0x0f]).is_err());
    assert!(decode_packet(&[0x18, 0x00, 0x65]).is_err());
    assert!(decode_packet(&[0x32, 0x02]).is_err());
    assert!(decode_packet(&[0x32, 0x00, 0x03, b'B', b'a', b'd']).is_err());
    assert!(decode_packet(&[0x32, 0x00]).is_err());
    assert!(decode_packet(&[0x32, 0x01, 0x00]).is_err());

    let malformed_utf = [0x18, 0x00, 0x01, 0x01, 0xff, 0x00];
    assert_eq!(
        decode_packet(&malformed_utf).unwrap(),
        edit(0, &["\u{fffd}"], None)
    );
    assert!(encode_packet(edit(0, &[&"x".repeat(1_025)], None)).is_err());
    assert!(encode_packet(edit(0, &[], Some(&"x".repeat(33)))).is_err());
    let one_hundred = PlayServerboundEntryPacket::EditBook(EditBook {
        slot: 40,
        pages: vec!["x".repeat(1_024); 100],
        title: Some("t".repeat(32)),
    });
    assert_eq!(
        decode_packet(&encode_packet(one_hundred.clone()).unwrap()).unwrap(),
        one_hundred
    );
}

#[test]
fn c3_bundle_selection_prediction_precedes_send_and_is_transient() {
    let expected = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 8, 9, 10, 11, 8];
    for (size, visible) in expected.into_iter().enumerate() {
        assert_eq!(displayed_item_count(size), visible);
    }

    let mut client = BundleClientMenu {
        slots: vec![bundle_stack(17)],
        events: Vec::new(),
    };
    assert_eq!(
        client.toggle(0, 16),
        Some(BundleItemSelected {
            slot: 0,
            selected: 16,
        })
    );
    assert_eq!(
        &client.events,
        &[
            BundleClientEvent::Mutated {
                slot: 0,
                selected: 16,
            },
            BundleClientEvent::Sent(BundleItemSelected {
                slot: 0,
                selected: 16,
            }),
        ]
    );
    assert_eq!(
        client.toggle(0, 16),
        Some(BundleItemSelected {
            slot: 0,
            selected: -1,
        })
    );
    assert_eq!(
        client.scroll(0, -1),
        Some(BundleItemSelected {
            slot: 0,
            selected: 7,
        }),
        "scroll only addresses the eight displayed entries for a 17-entry bundle"
    );
    assert_eq!(client.clear(0).unwrap().selected, -1);
    assert_eq!(client.clear(0).unwrap().selected, -1);

    let selected = client.slots[0].contents.as_mut().unwrap();
    selected.toggle_selected(12);
    let reconstructed = selected.reconstructed();
    assert_eq!(
        selected, &reconstructed,
        "selection is excluded from equality"
    );
    assert_eq!(reconstructed.selected(), -1);
}

#[test]
fn c3_bundle_selection_admission_uses_handler_time_current_menu_only() {
    let packet = BundleItemSelected {
        slot: 1,
        selected: 16,
    };
    let mut replacement_menu = vec![BundleMenuStack::default(), bundle_stack(17)];
    assert_eq!(
        handle_bundle_selection(&mut replacement_menu, packet),
        BundleSelectionOutcome::Applied { selected: 16 }
    );
    let contents = replacement_menu[1].contents.as_mut().unwrap();
    assert_eq!(contents.remove_selected_or_first(), Some(16));
    assert_eq!(contents.selected(), -1);

    assert_eq!(
        handle_bundle_selection(
            &mut replacement_menu,
            BundleItemSelected {
                slot: 1,
                selected: 99,
            },
        ),
        BundleSelectionOutcome::Applied { selected: -1 }
    );
    assert_eq!(
        handle_bundle_selection(
            &mut replacement_menu,
            BundleItemSelected {
                slot: -1,
                selected: 0,
            },
        ),
        BundleSelectionOutcome::IgnoredInvalidSlot
    );
    assert_eq!(
        handle_bundle_selection(
            &mut replacement_menu,
            BundleItemSelected {
                slot: 0,
                selected: 0,
            },
        ),
        BundleSelectionOutcome::IgnoredMissingComponent
    );
}

#[test]
fn c3_book_edit_admission_rechecks_only_callback_time_writable_content() {
    let mut service = BookFilterService::connected();
    assert_eq!(
        service.admit(EditBook {
            slot: 9,
            pages: vec![],
            title: None,
        }),
        Err(BookFilterOutcome::IgnoredInvalidSlot)
    );
    let task = service
        .admit(EditBook {
            slot: 4,
            pages: vec!["new".to_owned()],
            title: None,
        })
        .unwrap();
    let mut inventory = BookInventory::default();
    inventory.set(4, writable("minecraft:stone", &["replacement"]));
    assert_eq!(
        service.complete(
            task.id,
            Ok(BookFilterOutput {
                pages: vec![filter("new", Some("***"))],
                title: None,
            }),
            &mut inventory,
            false,
            "Player",
        ),
        BookFilterOutcome::UpdatedWritable
    );
    assert_eq!(
        inventory.get(4).unwrap().writable_pages.as_ref().unwrap(),
        &[FilterableText {
            raw: "new".to_owned(),
            filtered: Some("***".to_owned()),
        }]
    );
    assert_eq!(inventory.ordinary_projections(), 1);

    for slot in [0, 8, 40] {
        assert!(
            service
                .admit(EditBook {
                    slot,
                    pages: vec![],
                    title: None,
                })
                .is_ok()
        );
    }
    for slot in [i32::MIN, -1, 9, 39, 41, i32::MAX] {
        assert!(
            service
                .admit(EditBook {
                    slot,
                    pages: vec![],
                    title: None,
                })
                .is_err()
        );
    }
}

#[test]
fn c3_book_filter_completion_order_races_slot_occupancy_and_disconnect() {
    let mut service = BookFilterService::connected();
    let first = service
        .admit(EditBook {
            slot: 0,
            pages: vec!["first".to_owned()],
            title: None,
        })
        .unwrap();
    let second = service
        .admit(EditBook {
            slot: 0,
            pages: vec!["second".to_owned()],
            title: None,
        })
        .unwrap();
    let mut inventory = BookInventory::default();
    inventory.set(0, writable("minecraft:writable_book", &[]));
    assert_eq!(
        service.complete(
            second.id,
            Ok(BookFilterOutput {
                pages: vec![filter("second", None)],
                title: None,
            }),
            &mut inventory,
            false,
            "Player",
        ),
        BookFilterOutcome::UpdatedWritable
    );
    assert_eq!(
        service.complete(
            first.id,
            Ok(BookFilterOutput {
                pages: vec![filter("first", None)],
                title: None,
            }),
            &mut inventory,
            false,
            "Player",
        ),
        BookFilterOutcome::UpdatedWritable
    );
    assert_eq!(
        inventory.get(0).unwrap().writable_pages.as_ref().unwrap()[0].raw,
        "first"
    );

    let failed = service
        .admit(EditBook {
            slot: 0,
            pages: vec![],
            title: None,
        })
        .unwrap();
    assert_eq!(
        service.complete(failed.id, Err(()), &mut inventory, false, "Player"),
        BookFilterOutcome::FilterFailed
    );
    let cancelled = service
        .admit(EditBook {
            slot: 0,
            pages: vec![],
            title: None,
        })
        .unwrap();
    service.disconnect();
    assert_eq!(service.pending_count(), 0);
    assert_eq!(
        service.complete(
            cancelled.id,
            Ok(BookFilterOutput {
                pages: vec![],
                title: None,
            }),
            &mut inventory,
            false,
            "Player",
        ),
        BookFilterOutcome::UnknownOrCancelledTask
    );
}

#[test]
fn c3_book_finalization_preserves_components_and_accepts_transport_title() {
    assert_eq!(
        BookClient::done(2, vec!["page".to_owned(), String::new(), String::new(),],),
        EditBook {
            slot: 2,
            pages: vec!["page".to_owned()],
            title: None,
        }
    );
    assert_eq!(
        BookClient::finalize(2, vec!["page".to_owned(), String::new()], "  title  ").title,
        Some("title".to_owned())
    );
    assert_eq!(BookClient::escape(), None);
    assert_eq!(BookClient::cancel_signing(), None);

    let mut stack = writable("minecraft:custom_item", &["old"]);
    stack
        .retained_components
        .insert(id("minecraft:custom_data"), vec![1, 2, 3]);
    let retained = stack.retained_components.clone();
    let mut inventory = BookInventory::default();
    inventory.set(40, stack);
    let mut service = BookFilterService::connected();
    let task = service
        .admit(EditBook {
            slot: 40,
            pages: vec!["raw".to_owned()],
            title: Some(String::new()),
        })
        .unwrap();
    assert_eq!(
        service.complete(
            task.id,
            Ok(BookFilterOutput {
                pages: vec![filter("raw", Some("filtered"))],
                title: Some(filter("", Some("   "))),
            }),
            &mut inventory,
            true,
            "Alex",
        ),
        BookFilterOutcome::FinalizedWritten
    );
    let stack = inventory.get(40).unwrap();
    assert_eq!(stack.item, id("minecraft:written_book"));
    assert_eq!(stack.retained_components, retained);
    assert!(stack.writable_pages.is_none());
    let written = stack.written_content.as_ref().unwrap();
    assert_eq!(
        (&written.author, written.generation, written.resolved),
        (&"Alex".to_owned(), 0, true)
    );
    assert_eq!(
        written.title,
        FilterableText {
            raw: "   ".to_owned(),
            filtered: None,
        },
        "the server does not repeat client blank, trim, or 15-unit title checks"
    );
    assert_eq!(written.pages[0].raw, "filtered");

    let later = service
        .admit(EditBook {
            slot: 40,
            pages: vec![],
            title: None,
        })
        .unwrap();
    assert_eq!(
        service.complete(
            later.id,
            Ok(BookFilterOutput {
                pages: vec![],
                title: None,
            }),
            &mut inventory,
            false,
            "Alex",
        ),
        BookFilterOutcome::MissingWritableContent
    );
}

#[test]
fn c3_advancement_tab_correlation_distinguishes_unknown_and_normalizing_opens() {
    let root = AdvancementNode {
        id: id("minecraft:story/root"),
        root: id("minecraft:story/root"),
        has_display: true,
    };
    let child = AdvancementNode {
        id: id("minecraft:story/child"),
        root: id("minecraft:story/root"),
        has_display: true,
    };
    let hidden_root = AdvancementNode {
        id: id("minecraft:hidden/root"),
        root: id("minecraft:hidden/root"),
        has_display: false,
    };
    let mut state = AdvancementTabState::new([root.clone(), child.clone(), hidden_root.clone()]);
    let correction = state
        .handle(&SeenAdvancements::OpenedTab(root.id.clone()))
        .unwrap();
    assert_eq!(correction.selected, Some(root.id.clone()));
    assert!(
        state
            .handle(&SeenAdvancements::OpenedTab(id("minecraft:unknown")))
            .is_none()
    );
    assert_eq!(state.selected(), Some(&root.id));
    assert!(state.handle(&SeenAdvancements::ClosedScreen).is_none());
    assert_eq!(state.selected(), Some(&root.id));

    let correction = state
        .handle(&SeenAdvancements::OpenedTab(child.id))
        .unwrap();
    assert_eq!(correction.selected, None);
    assert!(
        state
            .handle(&SeenAdvancements::OpenedTab(hidden_root.id))
            .is_none()
    );
    state.reload([root]);
    assert_eq!(state.selected(), None);
}

#[test]
fn c3_inventory_aux_order_keeps_client_open_send_before_notification() {
    let root = id("minecraft:story/root");
    let mut client = AdvancementClientScreen::connected();
    assert_eq!(
        client.open_tab(root.clone()),
        SeenAdvancements::OpenedTab(root.clone())
    );
    assert_eq!(
        client.events,
        vec![
            AdvancementClientEvent::Sent(SeenAdvancements::OpenedTab(root.clone())),
            AdvancementClientEvent::Notified(Some(root.clone())),
        ]
    );
    client.open_tab(root.clone());
    assert_eq!(
        client.events.last(),
        Some(&AdvancementClientEvent::Sent(SeenAdvancements::OpenedTab(
            root
        ))),
        "identity-equal reopens still send but do not notify"
    );
    assert_eq!(client.remove(), Some(SeenAdvancements::ClosedScreen));
    client.disconnect();
    assert_eq!(client.remove(), None);
}

#[test]
fn c3_inventory_aux_end_to_end_preserves_tokenless_packet_order() {
    let packets = [
        bundle(0, 1),
        edit(0, &["page"], None),
        opened("minecraft:story/root"),
    ];
    let decoded = packets
        .iter()
        .cloned()
        .map(|packet| decode_packet(&encode_packet(packet).unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(decoded, packets);

    let mut menu = vec![bundle_stack(2)];
    let PlayServerboundEntryPacket::BundleItemSelected(packet) = decoded[0] else {
        panic!("bundle packet");
    };
    assert_eq!(
        handle_bundle_selection(&mut menu, packet),
        BundleSelectionOutcome::Applied { selected: 1 }
    );
    assert_eq!(
        menu[0]
            .contents
            .as_mut()
            .unwrap()
            .remove_selected_or_first(),
        Some(1)
    );

    let mut retained = BTreeMap::new();
    retained.insert(id("minecraft:story/root"), true);
    assert_eq!(
        retained.len(),
        1,
        "packet families share no acknowledgement map"
    );
}
