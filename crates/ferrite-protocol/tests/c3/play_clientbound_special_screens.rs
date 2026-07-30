use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::container::packet::SetCursorItem;
use ferrite_protocol::java_26_2::play::clientbound::container::publication::MenuSnapshot;
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::special_screen::packet::{
    InteractionHand, MountScreenOpen, OpenSignEditor,
};
use ferrite_protocol::java_26_2::play::clientbound::special_screen::projection::{
    BookStackProjection, BookViewKind, FilteredBookPage, SignBlockProjection, SignKind,
    SpecialScreenAction, SpecialScreenClientProjection, SpecialScreenProjectionError,
    TrackedMountKind,
};
use ferrite_protocol::java_26_2::play::clientbound::special_screen::publication::{
    EditableSign, EditableSignLine, MountPublisher, SignOpenAdmission, SignOpenError,
    publish_open_book, publish_open_sign,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::item::ItemStack;
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;

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

fn menu(menu_type: &str) -> MenuSnapshot {
    MenuSnapshot {
        menu_type: id(menu_type),
        title: TextComponentNbt::literal("Special").unwrap(),
        slots: vec![ItemStack::Empty; 3],
        carried: ItemStack::Empty,
        data: Vec::new(),
    }
}

fn page(raw: &str, filtered: Option<&str>) -> FilteredBookPage {
    FilteredBookPage {
        raw: raw.to_owned(),
        filtered: filtered.map(str::to_owned),
    }
}

fn lines(prefix: &str) -> [String; 4] {
    std::array::from_fn(|index| format!("{prefix}{index}"))
}

fn editable_lines(prefix: &str) -> [EditableSignLine; 4] {
    std::array::from_fn(|index| EditableSignLine {
        text: format!("{prefix}{index}"),
        plain: true,
    })
}

fn editable_sign(position: BlockPos) -> EditableSign {
    EditableSign {
        position,
        block_state: 10,
        waxed: false,
        editor: None,
        front: editable_lines("front"),
        back: editable_lines("back"),
    }
}

#[test]
fn c3_gold_clientbound_special_screens_lock_all_three_packets() {
    let registries = PlayRegistries::default();
    let mount = PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
        container_id: -1,
        inventory_columns: 2,
        entity_id: -3,
    });
    let book = PlayClientboundPacket::OpenBook(InteractionHand::Off);
    let sign = PlayClientboundPacket::OpenSignEditor(OpenSignEditor {
        position: BlockPos::new(1, 2, 3),
        front_text: true,
    });
    assert_eq!(
        encode_packet(&mount, &registries).unwrap(),
        vec![
            0x29, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x02, 0xff, 0xff, 0xff, 0xfd
        ]
    );
    assert_eq!(encode_packet(&book, &registries).unwrap(), vec![0x3a, 0x01]);
    assert_eq!(
        encode_packet(&sign, &registries).unwrap(),
        vec![0x3c, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x30, 0x02, 0x01]
    );
}

#[test]
fn c3_special_screen_codecs_preserve_signed_bounds_and_reject_invalid_hands() {
    let registries = PlayRegistries::default();
    for packet in [
        PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
            container_id: i32::MIN,
            inventory_columns: i32::MAX,
            entity_id: i32::MIN,
        }),
        PlayClientboundPacket::OpenSignEditor(OpenSignEditor {
            position: BlockPos::new(-33_554_432, -2_048, 33_554_431),
            front_text: false,
        }),
    ] {
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
    assert!(matches!(
        decode_packet(&[0x3a, 0x02], context(&registries)),
        Err(PlayClientboundCodecError::SpecialScreen(_))
    ));
    assert!(decode_packet(&[0x29], context(&registries)).is_err());
    assert!(decode_packet(&[0x3a, 0x00, 0x00], context(&registries)).is_err());

    let mut sign = encode_packet(
        &PlayClientboundPacket::OpenSignEditor(OpenSignEditor {
            position: BlockPos::new(0, 0, 0),
            front_text: true,
        }),
        &registries,
    )
    .unwrap();
    *sign.last_mut().unwrap() = 7;
    let decoded = decode_packet(&sign, context(&registries)).unwrap();
    let canonical = encode_packet(&decoded, &registries).unwrap();
    assert_eq!(canonical.last(), Some(&1));
}

#[test]
fn c3_mount_screen_activation_allocates_before_entity_gate_and_selects_subtype() {
    let mut client = SpecialScreenClientProjection::new(12);
    let negative = PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
        container_id: 1,
        inventory_columns: i32::MIN,
        entity_id: 99,
    });
    assert!(matches!(
        client.apply(&negative),
        Err(SpecialScreenProjectionError::NegativeMountAllocation { .. })
    ));
    let oversized = PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
        container_id: 1,
        inventory_columns: i32::MAX,
        entity_id: 99,
    });
    assert!(matches!(
        client.apply(&oversized),
        Err(SpecialScreenProjectionError::MountAllocationLimit { .. })
    ));
    let safe_missing = PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
        container_id: 1,
        inventory_columns: 2,
        entity_id: 99,
    });
    assert_eq!(
        client.apply(&safe_missing).unwrap(),
        SpecialScreenAction::Ignored
    );

    client.track_mount(2, TrackedMountKind::Other);
    client.track_mount(3, TrackedMountKind::Horse);
    client.track_mount(4, TrackedMountKind::Nautilus);
    assert_eq!(
        client
            .apply(&PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
                container_id: 2,
                inventory_columns: 2,
                entity_id: 2,
            }))
            .unwrap(),
        SpecialScreenAction::Ignored
    );
    client
        .apply(&PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
            container_id: 3,
            inventory_columns: 2,
            entity_id: 3,
        }))
        .unwrap();
    assert_eq!(client.current_mount().unwrap().cargo_slots, 6);
    client
        .apply(&PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
            container_id: 4,
            inventory_columns: 2,
            entity_id: 4,
        }))
        .unwrap();
    let nautilus = client.current_mount().unwrap();
    assert_eq!(nautilus.allocated_inventory_slots, 6);
    assert_eq!(nautilus.cargo_slots, 0);
}

#[test]
fn c3_book_view_activation_reads_current_hand_and_prefers_written_content() {
    let mut client = SpecialScreenClientProjection::new(12);
    client.set_filtering_enabled(true);
    client.set_hand(
        InteractionHand::Main,
        BookStackProjection {
            written_pages: Some(vec![page("raw", Some("filtered")), page("fallback", None)]),
            writable_pages: Some(vec![page("draft", None)]),
        },
    );
    let open_main = PlayClientboundPacket::OpenBook(InteractionHand::Main);
    let SpecialScreenAction::BookOpened(view) = client.apply(&open_main).unwrap() else {
        panic!("written book should open");
    };
    assert_eq!(view.kind, BookViewKind::Written);
    assert_eq!(view.pages, vec!["filtered", "fallback"]);

    client.set_hand(InteractionHand::Main, BookStackProjection::default());
    assert_eq!(
        client.apply(&open_main).unwrap(),
        SpecialScreenAction::Ignored
    );
    assert_eq!(client.current_book().unwrap().kind, BookViewKind::Written);

    client.set_hand(
        InteractionHand::Off,
        BookStackProjection {
            written_pages: None,
            writable_pages: Some(vec![page("draft", Some("filtered draft"))]),
        },
    );
    let SpecialScreenAction::BookOpened(view) = client
        .apply(&PlayClientboundPacket::OpenBook(InteractionHand::Off))
        .unwrap()
    else {
        panic!("writable book should open on forged client packet");
    };
    assert_eq!(view.kind, BookViewKind::Writable);
    assert_eq!(view.pages, vec!["filtered draft"]);
}

#[test]
fn c3_sign_editor_activation_resolves_current_sign_subtype_side_and_lines() {
    let position = BlockPos::new(-4, 70, 9);
    let hanging_position = BlockPos::new(5, 71, 10);
    let mut client = SpecialScreenClientProjection::new(12);
    client.track_sign(
        position,
        SignBlockProjection {
            kind: SignKind::Ordinary,
            front: lines("front"),
            back: lines("back"),
        },
    );
    client.track_sign(
        hanging_position,
        SignBlockProjection {
            kind: SignKind::Hanging,
            front: lines("hanging"),
            back: lines("hidden"),
        },
    );
    assert_eq!(
        client
            .apply(&PlayClientboundPacket::OpenSignEditor(OpenSignEditor {
                position: BlockPos::new(0, 0, 0),
                front_text: true,
            }))
            .unwrap(),
        SpecialScreenAction::Ignored
    );
    client
        .apply(&PlayClientboundPacket::OpenSignEditor(OpenSignEditor {
            position,
            front_text: false,
        }))
        .unwrap();
    let editor = client.current_sign().unwrap();
    assert_eq!(editor.kind, SignKind::Ordinary);
    assert_eq!(editor.lines, lines("back"));
    client
        .apply(&PlayClientboundPacket::OpenSignEditor(OpenSignEditor {
            position: hanging_position,
            front_text: true,
        }))
        .unwrap();
    assert_eq!(client.current_sign().unwrap().kind, SignKind::Hanging);
}

#[test]
fn c3_special_screens_order_mount_book_and_sign_prerequisites_before_activation() {
    let mut mount = MountPublisher::default();
    mount
        .containers_mut()
        .open(menu("minecraft:generic_9x1"))
        .unwrap();
    let packets = mount.open(44, 2, menu("minecraft:horse")).unwrap();
    assert!(matches!(
        packets[0],
        PlayClientboundPacket::ContainerClose(_)
    ));
    assert!(matches!(
        packets[1],
        PlayClientboundPacket::MountScreenOpen(_)
    ));
    assert!(matches!(
        packets[2],
        PlayClientboundPacket::ContainerSetContent(_)
    ));

    let menu_change = PlayClientboundPacket::SetCursorItem(SetCursorItem {
        item: ItemStack::Empty,
    });
    let packets = publish_open_book(InteractionHand::Main, true, true, vec![menu_change.clone()]);
    assert_eq!(packets[0], menu_change);
    assert_eq!(
        packets[1],
        PlayClientboundPacket::OpenBook(InteractionHand::Main)
    );
    assert!(publish_open_book(InteractionHand::Main, false, true, vec![menu_change]).is_empty());

    let position = BlockPos::new(1, 64, 2);
    let mut sign = editable_sign(position);
    let packets = publish_open_sign(
        &mut sign,
        SignOpenAdmission {
            player: 7,
            front_text: true,
            command_consumed: false,
            may_build: true,
        },
    )
    .unwrap();
    assert_eq!(sign.editor, Some(7));
    assert!(matches!(packets[0], PlayClientboundPacket::BlockUpdate(_)));
    assert!(matches!(
        packets[1],
        PlayClientboundPacket::OpenSignEditor(_)
    ));
    sign.waxed = true;
    assert_eq!(
        publish_open_sign(
            &mut sign,
            SignOpenAdmission {
                player: 7,
                front_text: true,
                command_consumed: false,
                may_build: true,
            }
        ),
        Err(SignOpenError::Waxed)
    );
}

#[test]
fn c3_special_screens_end_to_end_decode_into_current_client_state() {
    let registries = PlayRegistries::default();
    let position = BlockPos::new(3, 65, -8);
    let packets = [
        PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
            container_id: 5,
            inventory_columns: 1,
            entity_id: 20,
        }),
        PlayClientboundPacket::OpenBook(InteractionHand::Main),
        PlayClientboundPacket::OpenSignEditor(OpenSignEditor {
            position,
            front_text: true,
        }),
    ];
    let mut client = SpecialScreenClientProjection::new(12);
    client.track_mount(20, TrackedMountKind::Horse);
    client.set_hand(
        InteractionHand::Main,
        BookStackProjection {
            written_pages: Some(vec![page("hello", None)]),
            writable_pages: None,
        },
    );
    client.track_sign(
        position,
        SignBlockProjection {
            kind: SignKind::Ordinary,
            front: lines("front"),
            back: lines("back"),
        },
    );
    for packet in packets {
        let encoded = encode_packet(&packet, &registries).unwrap();
        let decoded = decode_packet(&encoded, context(&registries)).unwrap();
        assert!(!matches!(
            client.apply(&decoded).unwrap(),
            SpecialScreenAction::Ignored
        ));
    }
    assert_eq!(client.current_mount().unwrap().cargo_slots, 3);
    assert_eq!(client.current_book().unwrap().pages, vec!["hello"]);
    assert_eq!(client.current_sign().unwrap().lines, lines("front"));
}
