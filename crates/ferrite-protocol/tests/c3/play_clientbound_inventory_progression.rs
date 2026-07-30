use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use ferrite_protocol::java_26_2::play::clientbound::codec::{
    decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::inventory_progression::advancement::projection::AdvancementClientProjection;
use ferrite_protocol::java_26_2::play::clientbound::inventory_progression::advancement::publication::AdvancementPublisher;
use ferrite_protocol::java_26_2::play::clientbound::inventory_progression::map::{
    MapClientProjection, MapHoldingPublisher, MapProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::inventory_progression::packet::{
    Advancement, AdvancementFrame, AdvancementHolder, AdvancementProgress, DisplayInfo,
    MapDecoration, MapItemData, MapPatch, TagQuery, UpdateAdvancements,
};
use ferrite_protocol::java_26_2::play::clientbound::inventory_progression::tag_query::{
    DebugQueryHandler, block_query_response, entity_query_response,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::context::{
    ComponentValueDecoder, ComponentValueError, PlayDecodeContext,
};
use ferrite_protocol::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStackTemplate,
};
use ferrite_protocol::java_26_2::play::registry::{
    DATA_COMPONENT_TYPE, ITEM, MAP_DECORATION_TYPE, PlayRegistries,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::{NbtQuota, NetworkNbt, TextComponentNbt};
use ferrite_protocol::java_26_2::wire::frame::MAX_FRAME_LENGTH;
use ferrite_protocol::java_26_2::wire::primitive::{WireReader, WireWriter};

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

fn compound() -> NetworkNbt {
    NetworkNbt::from_bytes(vec![10, 0], NbtQuota::Default).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(MAP_DECORATION_TYPE),
        vec![
            id("minecraft:player"),
            id("minecraft:target_x"),
            id("minecraft:red_marker"),
        ],
    );
    registries.insert(id(ITEM), vec![id("minecraft:air"), id("minecraft:stone")]);
    registries.insert(id(DATA_COMPONENT_TYPE), vec![id("minecraft:custom_data")]);
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn icon() -> ItemStackTemplate {
    ItemStackTemplate {
        item: id("minecraft:stone"),
        count: 1,
        components: DataComponentPatch::default(),
    }
}

fn display(show_toast: bool, hidden: bool) -> DisplayInfo {
    DisplayInfo {
        title: title("title"),
        description: title("description"),
        icon: icon(),
        frame: AdvancementFrame::Task,
        background: None,
        show_toast,
        hidden,
        x: 1.25,
        y: -2.5,
    }
}

fn holder(
    identity: &str,
    parent: Option<&str>,
    display: Option<DisplayInfo>,
    requirements: &[&[&str]],
) -> AdvancementHolder {
    AdvancementHolder {
        id: id(identity),
        advancement: Advancement {
            parent: parent.map(id),
            display,
            requirements: requirements
                .iter()
                .map(|group| group.iter().map(|name| (*name).to_owned()).collect())
                .collect(),
            sends_telemetry_event: false,
        },
    }
}

fn progress(criteria: &[(&str, Option<i64>)]) -> AdvancementProgress {
    AdvancementProgress {
        criteria: criteria
            .iter()
            .map(|(name, timestamp)| ((*name).to_owned(), *timestamp))
            .collect(),
    }
}

fn advancement_packet(
    reset: bool,
    added: Vec<AdvancementHolder>,
    removed: BTreeSet<Identifier>,
    progress: BTreeMap<Identifier, AdvancementProgress>,
    show_advancements: bool,
) -> UpdateAdvancements {
    UpdateAdvancements {
        reset,
        added,
        removed,
        progress,
        show_advancements,
    }
}

#[test]
fn three_packet_goldens_lock_ids_empty_forms_and_nullable_tag() {
    let registries = registries();
    let packets = [
        (
            PlayClientboundPacket::MapItemData(MapItemData {
                map_id: 0,
                scale: 0,
                locked: false,
                decorations: None,
                patch: None,
            }),
            vec![51, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::TagQuery(TagQuery {
                transaction: 0,
                tag: None,
            }),
            vec![123, 0, 0],
        ),
        (
            PlayClientboundPacket::UpdateAdvancements(advancement_packet(
                false,
                Vec::new(),
                BTreeSet::new(),
                BTreeMap::new(),
                false,
            )),
            vec![0x82, 0x01, 0, 0, 0, 0, 0],
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
fn structured_map_tag_and_advancement_payloads_round_trip() {
    let registries = registries();
    let map = PlayClientboundPacket::MapItemData(MapItemData {
        map_id: i32::MIN,
        scale: i8::MIN,
        locked: true,
        decorations: Some(vec![MapDecoration {
            decoration_type: id("minecraft:target_x"),
            x: i8::MIN,
            y: i8::MAX,
            rotation: 15,
            name: Some(title("target")),
        }]),
        patch: Some(MapPatch {
            width: 255,
            height: 0,
            start_x: 255,
            start_y: 255,
            colors: vec![1, 2, 3],
        }),
    });
    let tag = PlayClientboundPacket::TagQuery(TagQuery {
        transaction: i32::MAX,
        tag: Some(compound()),
    });
    let root = holder(
        "minecraft:story/root",
        None,
        Some(DisplayInfo {
            icon: ItemStackTemplate {
                components: DataComponentPatch {
                    added: vec![EncodedComponentValue {
                        component: id("minecraft:custom_data"),
                        encoded_value: vec![0xa5],
                    }],
                    removed: Vec::new(),
                },
                ..icon()
            },
            frame: AdvancementFrame::Challenge,
            background: Some(id("minecraft:textures/gui/advancements/backgrounds/stone")),
            ..display(true, true)
        }),
        &[&["a", "b"], &[]],
    );
    let advancements = PlayClientboundPacket::UpdateAdvancements(advancement_packet(
        true,
        vec![root],
        BTreeSet::from([id("minecraft:old")]),
        BTreeMap::from([(
            id("minecraft:story/root"),
            progress(&[("a", Some(i64::MIN)), ("b", None)]),
        )]),
        true,
    ));

    for packet in [map, tag, advancements] {
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
}

#[test]
fn codecs_normalize_rotation_duplicate_collections_and_display_flags() {
    let registries = registries();
    let map = vec![51, 0, 0, 0, 1, 1, 0, 0, 0, 0xff, 0, 0];
    let decoded = decode_packet(&map, context(&registries)).unwrap();
    let PlayClientboundPacket::MapItemData(map) = decoded else {
        panic!("wrong packet");
    };
    assert_eq!(map.decorations.unwrap()[0].rotation, 15);

    let duplicate = id("minecraft:duplicate");
    let mut writer = WireWriter::new(MAX_FRAME_LENGTH);
    writer.write_var_i32(130).unwrap();
    writer.write_bool(false).unwrap();
    writer.write_var_i32(0).unwrap();
    writer.write_var_i32(2).unwrap();
    writer.write_utf(&duplicate.to_string(), 32_767).unwrap();
    writer.write_utf(&duplicate.to_string(), 32_767).unwrap();
    writer.write_var_i32(2).unwrap();
    writer.write_utf(&duplicate.to_string(), 32_767).unwrap();
    writer.write_var_i32(0).unwrap();
    writer.write_utf(&duplicate.to_string(), 32_767).unwrap();
    writer.write_var_i32(1).unwrap();
    writer.write_utf("criterion", 32_767).unwrap();
    writer.write_bool(true).unwrap();
    writer.write_i64(2).unwrap();
    writer.write_bool(false).unwrap();
    let decoded = decode_packet(&writer.into_inner(), context(&registries)).unwrap();
    let PlayClientboundPacket::UpdateAdvancements(packet) = decoded else {
        panic!("wrong packet");
    };
    assert_eq!(packet.removed, BTreeSet::from([duplicate.clone()]));
    assert_eq!(
        packet.progress,
        BTreeMap::from([(duplicate, progress(&[("criterion", Some(2))]))])
    );
}

#[test]
fn malformed_registry_nbt_frame_counts_patch_and_residual_data_fail_closed() {
    let registries = registries();
    let vectors = [
        vec![51, 0, 0, 0, 1, 1, 9],
        vec![51, 0, 0, 0, 1, 0xff, 0xff, 0xff, 0xff, 0x0f],
        vec![123, 0, 1, 0],
        vec![123, 0, 10],
        vec![0x82, 1, 0, 0xff, 0xff, 0xff, 0xff, 0x0f],
        vec![0x82, 1, 0, 0, 0, 0],
    ];
    for vector in vectors {
        assert!(
            decode_packet(&vector, context(&registries)).is_err(),
            "{vector:?}"
        );
    }

    let invalid_patch = PlayClientboundPacket::MapItemData(MapItemData {
        map_id: 0,
        scale: 0,
        locked: false,
        decorations: None,
        patch: Some(MapPatch {
            width: 0,
            height: 1,
            start_x: 0,
            start_y: 0,
            colors: vec![1],
        }),
    });
    assert!(encode_packet(&invalid_patch, &registries).is_err());

    let invalid_frame = holder(
        "minecraft:root",
        None,
        Some(display(true, false)),
        &[&["a"]],
    );
    let mut encoded = encode_packet(
        &PlayClientboundPacket::UpdateAdvancements(advancement_packet(
            false,
            vec![invalid_frame],
            BTreeSet::new(),
            BTreeMap::new(),
            false,
        )),
        &registries,
    )
    .unwrap();
    let item_and_frame = encoded
        .windows(5)
        .position(|window| window == [1, 1, 0, 0, 0])
        .expect("packet contains stone, count, empty patch, and task frame");
    encoded[item_and_frame + 4] = 9;
    assert!(decode_packet(&encoded, context(&registries)).is_err());
}

#[test]
fn map_projection_keeps_creation_fields_and_reproduces_partial_patch_faults() {
    let tracking = BTreeSet::from([id("minecraft:player")]);
    let mut projection = MapClientProjection::new(id("minecraft:overworld"), 2, tracking);
    let initial = MapItemData {
        map_id: 7,
        scale: 2,
        locked: true,
        decorations: Some(vec![MapDecoration {
            decoration_type: id("minecraft:player"),
            x: 1,
            y: 2,
            rotation: 3,
            name: None,
        }]),
        patch: Some(MapPatch {
            width: 2,
            height: 2,
            start_x: 0,
            start_y: 0,
            colors: vec![10, 11, 12],
        }),
    };
    assert!(matches!(
        projection.apply(&initial),
        Err(MapProjectionError::PatchSource { index: 3, .. })
    ));
    let map = projection.get(7).unwrap();
    assert_eq!(map.decorations.len(), 1);
    assert_eq!(map.tracked_decoration_count, 1);
    assert_eq!(map.colors[0], 10);
    assert_eq!(map.colors[128], 12);
    assert_eq!(map.colors[1], 11);
    assert_eq!(map.texture_refreshes, 0);

    projection
        .apply(&MapItemData {
            map_id: 7,
            scale: 99,
            locked: false,
            decorations: None,
            patch: Some(MapPatch {
                width: 2,
                height: 1,
                start_x: 127,
                start_y: 0,
                colors: vec![5, 6],
            }),
        })
        .unwrap();
    let map = projection.get(7).unwrap();
    assert_eq!((map.scale, map.locked), (2, true));
    assert_eq!((map.colors[127], map.colors[128]), (5, 6));
    assert_eq!(map.decorations.len(), 1);
    assert_eq!(map.texture_refreshes, 1);
}

#[test]
fn map_publication_consumes_dirty_bounds_and_samples_decorations_every_fifth_dirty_opportunity() {
    let decoration = MapDecoration {
        decoration_type: id("minecraft:target_x"),
        x: 0,
        y: 0,
        rotation: 0,
        name: None,
    };
    let mut publisher = MapHoldingPublisher::new(3, true, 4);
    publisher.set_color(5, 6, 1).unwrap();
    publisher.set_color(7, 8, 2).unwrap();
    publisher
        .replace_decorations(vec![decoration.clone()])
        .unwrap();
    let first = publisher.next_packet(-1).unwrap();
    assert_eq!(first.decorations, Some(vec![decoration.clone()]));
    assert_eq!(
        first.patch,
        Some(MapPatch {
            width: 3,
            height: 3,
            start_x: 5,
            start_y: 6,
            colors: vec![1, 0, 0, 0, 0, 0, 0, 0, 2],
        })
    );
    assert!(publisher.next_packet(-1).is_none());

    publisher.replace_decorations(vec![decoration]).unwrap();
    for _ in 0..4 {
        assert!(publisher.next_packet(-1).is_none());
    }
    assert!(publisher.next_packet(-1).unwrap().decorations.is_some());
    publisher.set_color(1, 1, 9).unwrap();
    let pixel_only = publisher.next_packet(-1).unwrap();
    assert!(pixel_only.decorations.is_none());
    assert!(pixel_only.patch.is_some());
}

#[test]
fn tag_query_uses_only_latest_exact_transaction_and_clears_after_success() {
    let mut handler = DebugQueryHandler::default();
    assert_eq!(handler.start_transaction(), 0);
    assert_eq!(handler.start_transaction(), 1);
    let calls = Cell::new(0);
    assert!(
        !handler
            .handle_response(
                &TagQuery {
                    transaction: 0,
                    tag: None,
                },
                |_| -> Result<(), ()> {
                    calls.set(calls.get() + 1);
                    Ok(())
                },
            )
            .unwrap()
    );
    assert_eq!(calls.get(), 0);

    let failed = handler.handle_response(
        &TagQuery {
            transaction: 1,
            tag: Some(compound()),
        },
        |_| Err("callback failed"),
    );
    assert_eq!(failed, Err("callback failed"));
    assert!(handler.has_pending_callback());
    assert!(
        handler
            .handle_response(
                &TagQuery {
                    transaction: 1,
                    tag: None,
                },
                |_| -> Result<(), ()> {
                    calls.set(calls.get() + 1);
                    Ok(())
                },
            )
            .unwrap()
    );
    assert_eq!(calls.get(), 1);
    assert!(!handler.has_pending_callback());

    assert!(block_query_response(true, 4, None).is_some());
    assert!(block_query_response(false, 4, None).is_none());
    assert!(entity_query_response(true, false, 4, compound()).is_none());
    assert!(entity_query_response(true, true, 4, compound()).is_some());
}

#[test]
fn advancement_tree_retries_parents_normalizes_progress_and_repeats_presentation() {
    let root = holder(
        "minecraft:root",
        None,
        Some(display(false, false)),
        &[&["root"]],
    );
    let child = holder(
        "minecraft:child",
        Some("minecraft:root"),
        Some(display(true, false)),
        &[&["a", "b"], &["c"]],
    );
    let unresolved = holder(
        "minecraft:unresolved",
        Some("minecraft:missing"),
        None,
        &[&["x"]],
    );
    let mut projection = AdvancementClientProjection::new(16, 16);
    let packet = advancement_packet(
        false,
        vec![child.clone(), unresolved, root.clone()],
        BTreeSet::new(),
        BTreeMap::from([(
            child.id.clone(),
            progress(&[("a", Some(1)), ("c", Some(2)), ("extra", Some(3))]),
        )]),
        true,
    );
    let action = projection.apply(&packet, true).unwrap();
    assert!(projection.contains(&root.id));
    assert!(projection.contains(&child.id));
    assert_eq!(action.unresolved_added, vec![id("minecraft:unresolved")]);
    let normalized = projection.progress(&child.id).unwrap();
    assert!(normalized.complete);
    assert!(!normalized.criteria.contains_key("extra"));
    assert_eq!(normalized.criteria.get("b"), Some(&None));
    assert_eq!(action.telemetry, vec![child.id.clone()]);
    assert_eq!(action.toasts, vec![child.id.clone()]);

    let repeated = advancement_packet(
        false,
        Vec::new(),
        BTreeSet::new(),
        BTreeMap::from([(
            child.id.clone(),
            progress(&[("a", Some(4)), ("c", Some(5))]),
        )]),
        true,
    );
    let action = projection.apply(&repeated, true).unwrap();
    assert_eq!(action.telemetry, vec![child.id.clone()]);
    assert_eq!(action.toasts, vec![child.id.clone()]);

    projection.select_tab(Some(root.id.clone()));
    let reset = advancement_packet(
        true,
        vec![root.clone()],
        BTreeSet::new(),
        BTreeMap::from([(root.id.clone(), progress(&[("root", Some(9))]))]),
        true,
    );
    let action = projection.apply(&reset, true).unwrap();
    assert!(action.telemetry.is_empty());
    assert!(action.toasts.is_empty());
    assert_eq!(projection.selected_tab(), Some(&root.id));
}

#[test]
fn advancement_removal_is_recursive_but_progress_and_duplicate_nodes_are_retained() {
    let root = holder("minecraft:root", None, None, &[&["root"]]);
    let child = holder(
        "minecraft:child",
        Some("minecraft:root"),
        None,
        &[&["child"]],
    );
    let mut projection = AdvancementClientProjection::new(16, 16);
    projection
        .apply(
            &advancement_packet(
                false,
                vec![root.clone(), root.clone(), child.clone()],
                BTreeSet::new(),
                BTreeMap::from([(child.id.clone(), progress(&[("child", Some(1))]))]),
                false,
            ),
            false,
        )
        .unwrap();
    assert_eq!(projection.node_count(), 3);
    assert_eq!(projection.root_count(), 2);
    projection
        .apply(
            &advancement_packet(
                false,
                Vec::new(),
                BTreeSet::from([root.id.clone()]),
                BTreeMap::new(),
                false,
            ),
            false,
        )
        .unwrap();
    assert!(!projection.contains(&root.id));
    assert!(!projection.contains(&child.id));
    assert!(projection.progress(&child.id).is_some());
    assert_eq!(projection.node_count(), 1);
}

#[test]
fn advancement_publisher_evaluates_descendants_and_clears_first_reset_without_output() {
    let root = holder(
        "minecraft:root",
        None,
        Some(display(false, false)),
        &[&["root"]],
    );
    let child = holder(
        "minecraft:child",
        Some("minecraft:root"),
        Some(display(true, false)),
        &[&["child"]],
    );
    let leaf = holder(
        "minecraft:leaf",
        Some("minecraft:child"),
        Some(display(true, true)),
        &[&["leaf"]],
    );
    let mut publisher = AdvancementPublisher::new(8);
    for holder in [root.clone(), child.clone(), leaf.clone()] {
        publisher
            .insert(holder, AdvancementProgress::default())
            .unwrap();
    }
    assert!(publisher.flush(true).unwrap().is_none());

    publisher
        .update_progress(&leaf.id, progress(&[("leaf", Some(1))]))
        .unwrap();
    let packet = publisher.flush(true).unwrap().unwrap();
    assert!(!packet.reset);
    assert_eq!(
        packet
            .added
            .iter()
            .map(|holder| holder.id.clone())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([root.id.clone(), child.id.clone(), leaf.id.clone()])
    );
    assert!(publisher.is_visible(&root.id));
    assert!(publisher.is_visible(&child.id));
    assert!(publisher.is_visible(&leaf.id));

    publisher
        .update_progress(&leaf.id, AdvancementProgress::default())
        .unwrap();
    let packet = publisher.flush(true).unwrap().unwrap();
    assert_eq!(packet.removed, BTreeSet::from([root.id, child.id, leaf.id]));
}

#[test]
fn advancement_publication_codec_and_client_projection_converge_end_to_end() {
    let registries = registries();
    let root = holder(
        "minecraft:root",
        None,
        Some(display(true, false)),
        &[&["done"]],
    );
    let mut publisher = AdvancementPublisher::new(4);
    publisher
        .insert(root.clone(), progress(&[("done", Some(42))]))
        .unwrap();
    let packet = publisher.flush(true).unwrap().unwrap();
    assert!(packet.reset);

    let encoded = encode_packet(
        &PlayClientboundPacket::UpdateAdvancements(packet),
        &registries,
    )
    .unwrap();
    let decoded = decode_packet(&encoded, context(&registries)).unwrap();
    let PlayClientboundPacket::UpdateAdvancements(decoded) = decoded else {
        panic!("wrong packet");
    };
    let mut client = AdvancementClientProjection::new(4, 4);
    let action = client.apply(&decoded, true).unwrap();
    assert!(client.contains(&root.id));
    assert!(client.progress(&root.id).unwrap().complete);
    assert!(action.telemetry.is_empty());
    assert!(action.toasts.is_empty());
}
