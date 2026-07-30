use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::recipe::book::projection::{
    RecipeBookClientAction, RecipeBookClientProjection,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::book::publication::{
    GhostPublicationStep, RecipeBookPublisher, RecipeDisplayIndex, RecipeDisplaySource,
    publish_failed_placement,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::book::{
    PlaceGhostRecipe, RecipeBookRemove,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::display::RecipeDisplay;
use ferrite_protocol::java_26_2::play::clientbound::recipe::slot::SlotDisplay;
use ferrite_protocol::java_26_2::play::clientbound::recipe::{RecipeBookAdd, RecipeBookEntry};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::item::{DataComponentPatch, ItemStack};
use ferrite_protocol::java_26_2::play::registry::{
    DATA_COMPONENT_TYPE, ITEM, PlayRegistries, RECIPE_DISPLAY, SLOT_DISPLAY, TRIM_PATTERN,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(RECIPE_DISPLAY),
        [
            "minecraft:crafting_shapeless",
            "minecraft:crafting_shaped",
            "minecraft:furnace",
            "minecraft:stonecutter",
            "minecraft:smithing",
        ]
        .into_iter()
        .map(id)
        .collect(),
    );
    registries.insert(
        id(SLOT_DISPLAY),
        [
            "minecraft:empty",
            "minecraft:any_fuel",
            "minecraft:with_any_potion",
            "minecraft:only_with_component",
            "minecraft:item",
            "minecraft:item_stack",
            "minecraft:tag",
            "minecraft:dyed",
            "minecraft:smithing_trim",
            "minecraft:with_remainder",
            "minecraft:composite",
        ]
        .into_iter()
        .map(id)
        .collect(),
    );
    registries.insert(id(ITEM), vec![id("minecraft:air"), id("minecraft:stone")]);
    registries.insert(id(DATA_COMPONENT_TYPE), vec![id("minecraft:custom_data")]);
    registries.insert(id(TRIM_PATTERN), vec![id("minecraft:sentry")]);
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn empty_display() -> RecipeDisplay {
    RecipeDisplay::CraftingShapeless {
        ingredients: Vec::new(),
        result: SlotDisplay::Empty,
        crafting_station: SlotDisplay::Empty,
    }
}

fn displays() -> Vec<RecipeDisplay> {
    vec![
        empty_display(),
        RecipeDisplay::CraftingShaped {
            width: 1,
            height: 1,
            ingredients: vec![SlotDisplay::AnyFuel],
            result: SlotDisplay::Empty,
            crafting_station: SlotDisplay::Empty,
        },
        RecipeDisplay::Furnace {
            ingredient: SlotDisplay::Empty,
            fuel: SlotDisplay::AnyFuel,
            result: SlotDisplay::Empty,
            crafting_station: SlotDisplay::Empty,
            duration: -20,
            experience: 1.25,
        },
        RecipeDisplay::Stonecutter {
            input: SlotDisplay::Empty,
            result: SlotDisplay::Empty,
            crafting_station: SlotDisplay::Empty,
        },
        RecipeDisplay::Smithing {
            template: SlotDisplay::Empty,
            base: SlotDisplay::Empty,
            addition: SlotDisplay::Empty,
            result: SlotDisplay::Empty,
            crafting_station: SlotDisplay::Empty,
        },
    ]
}

fn entry(display_id: i32, highlight: bool) -> RecipeBookEntry {
    RecipeBookEntry {
        display_id,
        display: empty_display(),
        group: None,
        category: id("minecraft:crafting_building_blocks"),
        crafting_requirements: None,
        show_notification: false,
        highlight,
    }
}

fn stack(count: i32) -> ItemStack {
    ItemStack::present(id("minecraft:stone"), count, DataComponentPatch::default())
}

#[test]
fn c3_gold_clientbound_recipe_book_locks_both_packet_forms() {
    let registry = registries();
    let ghost = PlayClientboundPacket::PlaceGhostRecipe(Box::new(PlaceGhostRecipe {
        container_id: 7,
        display: empty_display(),
    }));
    let remove = PlayClientboundPacket::RecipeBookRemove(RecipeBookRemove {
        display_ids: vec![-1, 0, 5],
    });
    let ghost_bytes = encode_packet(&ghost, &registry).unwrap();
    let remove_bytes = encode_packet(&remove, &registry).unwrap();
    assert_eq!(ghost_bytes, vec![0x3f, 0x07, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(
        remove_bytes,
        vec![0x4b, 0x03, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x00, 0x05]
    );
    assert_eq!(
        decode_packet(&ghost_bytes, context(&registry)).unwrap(),
        ghost
    );
    assert_eq!(
        decode_packet(&remove_bytes, context(&registry)).unwrap(),
        remove
    );
}

#[test]
fn c3_recipe_book_codecs_dispatch_every_display_and_fail_closed() {
    let registry = registries();
    for (container_id, display) in displays().into_iter().enumerate() {
        let packet = PlayClientboundPacket::PlaceGhostRecipe(Box::new(PlaceGhostRecipe {
            container_id: container_id as i32,
            display,
        }));
        let encoded = encode_packet(&packet, &registry).unwrap();
        assert_eq!(decode_packet(&encoded, context(&registry)).unwrap(), packet);
    }
    assert!(matches!(
        decode_packet(&[0x3f, 0x00, 0x05], context(&registry)),
        Err(PlayClientboundCodecError::Recipe(_))
    ));
    assert!(matches!(
        decode_packet(&[0x4b, 0xff, 0xff, 0xff, 0xff, 0x0f], context(&registry)),
        Err(PlayClientboundCodecError::Recipe(_))
    ));
    assert!(decode_packet(&[0x4b, 0x01], context(&registry)).is_err());
    assert!(decode_packet(&[0x4b, 0x00, 0x00], context(&registry)).is_err());
}

#[test]
fn c3_recipe_display_id_mapping_is_contiguous_feature_filtered_and_generation_local() {
    let parent_a = id("minecraft:a");
    let parent_b = id("minecraft:b");
    let index = RecipeDisplayIndex::rebuild([
        RecipeDisplaySource {
            parent: parent_a.clone(),
            display: empty_display(),
            enabled: true,
        },
        RecipeDisplaySource {
            parent: parent_b.clone(),
            display: empty_display(),
            enabled: false,
        },
        RecipeDisplaySource {
            parent: parent_b.clone(),
            display: empty_display(),
            enabled: true,
        },
        RecipeDisplaySource {
            parent: parent_a.clone(),
            display: empty_display(),
            enabled: true,
        },
    ])
    .unwrap();
    assert_eq!(
        index
            .entries()
            .iter()
            .map(|entry| (entry.display_id, entry.parent.clone()))
            .collect::<Vec<_>>(),
        vec![(0, parent_a.clone()), (1, parent_b), (2, parent_a.clone())]
    );
    assert_eq!(index.display_ids_for_parent(&parent_a), vec![0, 2]);
    assert!(index.resolve(-1).is_none());
    assert!(index.resolve(3).is_none());
}

#[test]
fn c3_ghost_recipe_application_requires_exact_menu_and_recipe_listener_screen() {
    let display = RecipeDisplay::Stonecutter {
        input: SlotDisplay::AnyFuel,
        result: SlotDisplay::Empty,
        crafting_station: SlotDisplay::Empty,
    };
    let packet = PlayClientboundPacket::PlaceGhostRecipe(Box::new(PlaceGhostRecipe {
        container_id: 4,
        display: display.clone(),
    }));
    let mut client = RecipeBookClientProjection::default();
    assert_eq!(
        client.apply(&packet).unwrap(),
        RecipeBookClientAction::Ignored
    );
    client.open_menu(3, true);
    assert_eq!(
        client.apply(&packet).unwrap(),
        RecipeBookClientAction::Ignored
    );
    client.open_menu(4, false);
    assert_eq!(
        client.apply(&packet).unwrap(),
        RecipeBookClientAction::Ignored
    );
    client.open_menu(4, true);
    assert_eq!(
        client.apply(&packet).unwrap(),
        RecipeBookClientAction::GhostReplaced
    );
    assert_eq!(client.ghost(), Some(&display));
}

#[test]
fn c3_recipe_book_remove_preserves_wire_order_and_refreshes_once_even_when_empty() {
    let mut client = RecipeBookClientProjection::default();
    client.install_add(&RecipeBookAdd {
        entries: vec![entry(-1, true), entry(2, false), entry(3, true)],
        replace: true,
    });
    client.open_menu(8, true);
    let packet = PlayClientboundPacket::RecipeBookRemove(RecipeBookRemove {
        display_ids: vec![3, 3, 99, -1],
    });
    assert_eq!(
        client.apply(&packet).unwrap(),
        RecipeBookClientAction::Refreshed
    );
    assert_eq!(client.known().keys().copied().collect::<Vec<_>>(), vec![2]);
    assert!(client.highlights().is_empty());
    assert_eq!(client.last_removal_order(), &[3, 3, 99, -1]);
    assert_eq!(client.collection_refreshes(), 1);
    assert_eq!(client.search_refreshes(), 1);
    assert_eq!(client.screen_refreshes(), 1);

    client
        .apply(&PlayClientboundPacket::RecipeBookRemove(RecipeBookRemove {
            display_ids: Vec::new(),
        }))
        .unwrap();
    assert_eq!(client.collection_refreshes(), 2);
    assert_eq!(client.search_refreshes(), 2);
    assert_eq!(client.screen_refreshes(), 2);
}

#[test]
fn c3_recipe_book_highlights_and_parent_removal_map_to_all_current_display_ids() {
    let parent_a = id("minecraft:a");
    let parent_b = id("minecraft:b");
    let index = RecipeDisplayIndex::rebuild([
        RecipeDisplaySource {
            parent: parent_a.clone(),
            display: empty_display(),
            enabled: true,
        },
        RecipeDisplaySource {
            parent: parent_b.clone(),
            display: empty_display(),
            enabled: true,
        },
        RecipeDisplaySource {
            parent: parent_a.clone(),
            display: empty_display(),
            enabled: true,
        },
    ])
    .unwrap();
    let mut publisher = RecipeBookPublisher::new(index);
    publisher.mark_known(parent_a.clone(), true);
    publisher.mark_known(parent_b.clone(), false);
    let publication = publisher.remove_recipes(&[
        id("minecraft:missing"),
        parent_a.clone(),
        parent_a,
        parent_b,
    ]);
    assert_eq!(publication.removed_display_count, 3);
    let Some(PlayClientboundPacket::RecipeBookRemove(packet)) = publication.packet else {
        panic!("known parents must publish their current display IDs");
    };
    assert_eq!(packet.display_ids, vec![0, 2, 1]);
    assert!(publisher.known().is_empty());
    assert!(publisher.highlighted().is_empty());
    assert!(publisher.remove_recipes(&[]).packet.is_none());
}

#[test]
fn c3_recipe_book_order_returns_and_clears_inputs_before_sending_ghost() {
    let mut inputs = vec![stack(2), ItemStack::Empty, stack(3)];
    let publication = publish_failed_placement(9, &empty_display(), &mut inputs);
    assert_eq!(publication.returned_inputs, vec![stack(2), stack(3)]);
    assert_eq!(inputs, vec![ItemStack::Empty; 3]);
    assert_eq!(
        publication.steps,
        vec![
            GhostPublicationStep::ReturnedInputs,
            GhostPublicationStep::ClearedGrid,
            GhostPublicationStep::SentGhost,
        ]
    );
    assert!(matches!(
        publication.packet,
        PlayClientboundPacket::PlaceGhostRecipe(_)
    ));
}

#[test]
fn c3_recipe_book_end_to_end_converges_ghost_and_removal_without_acknowledgement() {
    let registry = registries();
    let mut inputs = vec![stack(1)];
    let publication = publish_failed_placement(5, &empty_display(), &mut inputs);
    let encoded = encode_packet(&publication.packet, &registry).unwrap();
    let decoded = decode_packet(&encoded, context(&registry)).unwrap();
    let mut client = RecipeBookClientProjection::default();
    client.install_add(&RecipeBookAdd {
        entries: vec![entry(0, true)],
        replace: true,
    });
    client.open_menu(5, true);
    assert_eq!(
        client.apply(&decoded).unwrap(),
        RecipeBookClientAction::GhostReplaced
    );

    let remove = PlayClientboundPacket::RecipeBookRemove(RecipeBookRemove {
        display_ids: vec![0],
    });
    let encoded = encode_packet(&remove, &registry).unwrap();
    let decoded = decode_packet(&encoded, context(&registry)).unwrap();
    assert_eq!(
        client.apply(&decoded).unwrap(),
        RecipeBookClientAction::Refreshed
    );
    assert!(client.known().is_empty());
    assert_eq!(client.collection_refreshes(), 1);
}
