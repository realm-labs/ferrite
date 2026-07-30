use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::recipe::display::RecipeDisplay;
use ferrite_protocol::java_26_2::play::clientbound::recipe::slot::SlotDisplay;
use ferrite_protocol::java_26_2::play::item::{DataComponentPatch, ItemStack};
use ferrite_protocol::java_26_2::play::serverbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_protocol::java_26_2::play::serverbound::recipe_book::packet::{
    PlaceRecipe, RecipeBookChangeSettings, RecipeBookSeenRecipe, RecipeBookType,
};
use ferrite_protocol::java_26_2::play::serverbound::recipe_book::placement::{
    PlaceRecipeIgnore, PlaceRecipeOutcome, PlacementBracket, PlacementItem, PlacementMutation,
    RecipeMenuTransaction, RecipePlacementIndex, RecipePlacementSession, RecipePlacementSource,
    ResolvedRecipePlacement,
};
use ferrite_protocol::java_26_2::play::serverbound::recipe_book::state::{
    RecipeBookClientEvent, RecipeBookClientState, ServerRecipeBook,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn stack(item: &str, count: i32) -> ItemStack {
    ItemStack::present(id(item), count, DataComponentPatch::default())
}

fn placement_item(item: &str, maximum: i32) -> PlacementItem {
    PlacementItem {
        stack: stack(item, 1),
        maximum_stack_size: maximum,
    }
}

fn placement(slots: Vec<Option<PlacementItem>>) -> ResolvedRecipePlacement {
    ResolvedRecipePlacement {
        width: slots.len(),
        height: 1,
        slots,
    }
}

fn display(item: &str) -> RecipeDisplay {
    RecipeDisplay::CraftingShapeless {
        ingredients: vec![SlotDisplay::Item(id(item))],
        result: SlotDisplay::Item(id("minecraft:emerald")),
        crafting_station: SlotDisplay::Item(id("minecraft:crafting_table")),
    }
}

fn source(
    parent: &str,
    enabled: bool,
    placement: Option<ResolvedRecipePlacement>,
) -> RecipePlacementSource {
    RecipePlacementSource {
        parent: id(parent),
        display: display("minecraft:apple"),
        enabled,
        placement,
    }
}

fn index() -> RecipePlacementIndex {
    RecipePlacementIndex::rebuild([source(
        "minecraft:apple_trade",
        true,
        Some(placement(vec![Some(placement_item("minecraft:apple", 64))])),
    )])
    .unwrap()
}

fn place_packet(
    container_id: i32,
    display_id: i32,
    use_maximum_items: bool,
) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::PlaceRecipe(PlaceRecipe {
        container_id,
        display_id,
        use_maximum_items,
    })
}

#[test]
fn c3_gold_serverbound_recipe_book_locks_all_three_packets() {
    let packets = [
        (place_packet(1, 0, false), vec![0x27, 0x01, 0x00, 0x00]),
        (
            PlayServerboundEntryPacket::RecipeBookChangeSettings(RecipeBookChangeSettings {
                book_type: RecipeBookType::Crafting,
                open: false,
                filtering: false,
            }),
            vec![0x2e, 0x00, 0x00, 0x00],
        ),
        (
            PlayServerboundEntryPacket::RecipeBookSeenRecipe(RecipeBookSeenRecipe {
                display_id: 0,
            }),
            vec![0x2f, 0x00],
        ),
    ];
    for (packet, body) in packets {
        assert_eq!(encode_packet(packet.clone()).unwrap(), body);
        assert_eq!(decode_packet(&body).unwrap(), packet);
    }
}

#[test]
fn c3_recipe_book_codecs_keep_signed_ids_strict_types_and_boolean_normalization() {
    for packet in [
        place_packet(i32::MIN, i32::MAX, true),
        PlayServerboundEntryPacket::RecipeBookChangeSettings(RecipeBookChangeSettings {
            book_type: RecipeBookType::Smoker,
            open: true,
            filtering: true,
        }),
        PlayServerboundEntryPacket::RecipeBookSeenRecipe(RecipeBookSeenRecipe {
            display_id: i32::MIN,
        }),
    ] {
        assert_eq!(
            decode_packet(&encode_packet(packet.clone()).unwrap()).unwrap(),
            packet
        );
    }

    assert_eq!(
        decode_packet(&[0x2e, 0x03, 0x02, 0xff]).unwrap(),
        PlayServerboundEntryPacket::RecipeBookChangeSettings(RecipeBookChangeSettings {
            book_type: RecipeBookType::Smoker,
            open: true,
            filtering: true,
        },)
    );
    for ordinal in [-1, 4, i32::MAX] {
        let mut body = vec![0x2e];
        body.extend(ferrite_protocol::java_26_2::wire::varint::encode_i32(ordinal).as_slice());
        body.extend([0, 0]);
        assert!(decode_packet(&body).is_err());
    }
    assert!(decode_packet(&[0x27, 0x00, 0x00]).is_err());
    assert!(decode_packet(&[0x2f, 0x80]).is_err());
    assert!(decode_packet(&[0x2f, 0x00, 0x00]).is_err());
}

#[test]
fn c3_recipe_display_id_mapping_is_contiguous_feature_filtered_and_reload_local() {
    let mapped = RecipePlacementIndex::rebuild([
        source("minecraft:first", true, Some(placement(vec![None]))),
        source("minecraft:disabled", false, Some(placement(vec![None]))),
        source("minecraft:first", true, Some(placement(vec![None]))),
        source("minecraft:last", true, None),
    ])
    .unwrap();
    assert_eq!(
        mapped
            .entries()
            .iter()
            .map(|entry| (entry.display_id, entry.parent.clone()))
            .collect::<Vec<_>>(),
        vec![
            (0, id("minecraft:first")),
            (1, id("minecraft:first")),
            (2, id("minecraft:last")),
        ]
    );
    assert!(mapped.resolve(-1).is_none());
    assert!(mapped.resolve(3).is_none());

    let reloaded = RecipePlacementIndex::rebuild([
        source("minecraft:last", true, None),
        source("minecraft:first", true, Some(placement(vec![None]))),
    ])
    .unwrap();
    assert_eq!(
        reloaded.resolve(0).unwrap().parent,
        id("minecraft:last"),
        "display IDs are rebuilt indices rather than durable recipe identities"
    );
}

#[test]
fn c3_recipe_place_admission_resets_idle_before_every_ordered_gate() {
    let mut session = RecipePlacementSession {
        index: index(),
        ..RecipePlacementSession::default()
    };
    session.book.known.insert(id("minecraft:apple_trade"));
    let packet = PlaceRecipe {
        container_id: 1,
        display_id: 0,
        use_maximum_items: false,
    };
    assert_eq!(
        session.handle_place_recipe(None, packet),
        PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::SpectatorOrWrongContainer)
    );
    let mut menu = RecipeMenuTransaction::new(1, 1, 1, 4);
    session.spectator = true;
    assert!(matches!(
        session.handle_place_recipe(Some(&mut menu), packet),
        PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::SpectatorOrWrongContainer)
    ));
    session.spectator = false;
    menu.still_valid = false;
    assert!(matches!(
        session.handle_place_recipe(Some(&mut menu), packet),
        PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::InvalidMenu)
    ));
    assert_eq!(session.invalid_menu_logs, 1);
    menu.still_valid = true;
    assert!(matches!(
        session.handle_place_recipe(
            Some(&mut menu),
            PlaceRecipe {
                display_id: -1,
                ..packet
            },
        ),
        PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::UnknownDisplay)
    ));
    session.book.known.clear();
    assert!(matches!(
        session.handle_place_recipe(Some(&mut menu), packet),
        PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::LockedParent)
    ));
    session.book.known.insert(id("minecraft:apple_trade"));
    menu.recipe_book_menu = false;
    assert!(matches!(
        session.handle_place_recipe(Some(&mut menu), packet),
        PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::NotRecipeMenu)
    ));

    let impossible_index =
        RecipePlacementIndex::rebuild([source("minecraft:apple_trade", true, None)]).unwrap();
    session.index = impossible_index;
    menu.recipe_book_menu = true;
    assert!(matches!(
        session.handle_place_recipe(Some(&mut menu), packet),
        PlaceRecipeOutcome::Ignored(PlaceRecipeIgnore::ImpossiblePlacement)
    ));
    assert_eq!(session.impossible_placement_logs, 1);
    assert_eq!(session.idle_resets, 7);
}

#[test]
fn c3_recipe_place_mutation_covers_capacity_ghost_increment_maximum_and_guard() {
    let mut session = RecipePlacementSession {
        index: index(),
        ..RecipePlacementSession::default()
    };
    session.book.known.insert(id("minecraft:apple_trade"));
    let packet = PlaceRecipe {
        container_id: 1,
        display_id: 0,
        use_maximum_items: false,
    };

    let mut blocked = RecipeMenuTransaction::new(1, 1, 1, 1);
    blocked.grid[0] = stack("minecraft:stone", 1);
    blocked.player_inventory[0] = stack("minecraft:diamond", 64);
    assert_eq!(
        session.handle_place_recipe(Some(&mut blocked), packet),
        PlaceRecipeOutcome::Applied(PlacementMutation::NoChange)
    );
    assert_eq!(blocked.grid[0], stack("minecraft:stone", 1));
    assert!(!blocked.inventory_changed);

    let mut ghost = RecipeMenuTransaction::new(1, 1, 1, 2);
    ghost.grid[0] = stack("minecraft:stone", 1);
    let PlaceRecipeOutcome::Applied(PlacementMutation::Ghost(ghost_response)) =
        session.handle_place_recipe(Some(&mut ghost), packet)
    else {
        panic!("uncraftable clearable grid should ghost");
    };
    let PlayClientboundPacket::PlaceGhostRecipe(ghost_packet) = ghost_response.as_ref() else {
        panic!("ghost response packet");
    };
    assert_eq!(ghost_packet.container_id, 1);
    assert!(ghost.grid[0].is_empty());
    assert!(ghost.inventory_changed);
    assert_eq!(
        ghost.brackets,
        [PlacementBracket::Begin, PlacementBracket::Finish]
    );

    let mut increment = RecipeMenuTransaction::new(1, 1, 1, 2);
    increment.grid[0] = stack("minecraft:apple", 1);
    increment.player_inventory[0] = stack("minecraft:apple", 4);
    assert_eq!(
        session.handle_place_recipe(Some(&mut increment), packet),
        PlaceRecipeOutcome::Applied(PlacementMutation::Placed)
    );
    assert_eq!(increment.grid[0].count(), 2);

    let clamped_index = RecipePlacementIndex::rebuild([source(
        "minecraft:apple_trade",
        true,
        Some(placement(vec![Some(placement_item("minecraft:apple", 4))])),
    )])
    .unwrap();
    session.index = clamped_index;
    let mut maximum = RecipeMenuTransaction::new(1, 1, 1, 2);
    maximum.player_inventory[0] = stack("minecraft:apple", 9);
    assert!(matches!(
        session.handle_place_recipe(
            Some(&mut maximum),
            PlaceRecipe {
                use_maximum_items: true,
                ..packet
            },
        ),
        PlaceRecipeOutcome::Applied(PlacementMutation::Placed)
    ));
    assert_eq!(maximum.grid[0].count(), 4);

    let mut guarded = RecipeMenuTransaction::new(1, 1, 1, 2);
    guarded.grid[0] = stack("minecraft:apple", 4);
    guarded.player_inventory[0] = stack("minecraft:apple", 4);
    assert_eq!(
        session.handle_place_recipe(Some(&mut guarded), packet),
        PlaceRecipeOutcome::Applied(PlacementMutation::NoChange)
    );
    assert_eq!(guarded.grid[0].count(), 4);
}

#[test]
fn c3_recipe_book_settings_replace_exact_type_without_other_gates() {
    let mut server = ServerRecipeBook::default();
    let mut client = RecipeBookClientState {
        connected: true,
        ..RecipeBookClientState::default()
    };
    for book_type in [
        RecipeBookType::Crafting,
        RecipeBookType::Furnace,
        RecipeBookType::BlastFurnace,
        RecipeBookType::Smoker,
    ] {
        let packet = RecipeBookChangeSettings {
            book_type,
            open: true,
            filtering: true,
        };
        assert_eq!(client.change_settings(packet), Some(packet));
        server.change_settings(packet);
    }
    assert!(server.settings.crafting_open && server.settings.crafting_filtering);
    assert!(server.settings.furnace_open && server.settings.furnace_filtering);
    assert!(server.settings.blast_furnace_open && server.settings.blast_furnace_filtering);
    assert!(server.settings.smoker_open && server.settings.smoker_filtering);
    assert!(matches!(
        client.events.as_slice(),
        [
            RecipeBookClientEvent::ChangedSetting(_),
            RecipeBookClientEvent::SentSettings(_),
            ..
        ]
    ));
}

#[test]
fn c3_recipe_book_highlights_map_display_to_shared_parent() {
    let index = RecipePlacementIndex::rebuild([
        source("minecraft:shared", true, Some(placement(vec![None]))),
        source("minecraft:shared", true, Some(placement(vec![None]))),
        source("minecraft:other", true, Some(placement(vec![None]))),
    ])
    .unwrap();
    let mut server = ServerRecipeBook::default();
    server
        .highlighted
        .extend([id("minecraft:shared"), id("minecraft:other")]);
    assert!(server.see_recipe(&index, RecipeBookSeenRecipe { display_id: 1 }));
    assert!(!server.highlighted.contains(&id("minecraft:shared")));
    assert!(server.highlighted.contains(&id("minecraft:other")));
    assert!(!server.see_recipe(&index, RecipeBookSeenRecipe { display_id: -1 }));

    let mut client = RecipeBookClientState {
        connected: true,
        highlighted_displays: [0, 1, 2].into_iter().collect(),
        ..RecipeBookClientState::default()
    };
    assert_eq!(client.see(1), Some(RecipeBookSeenRecipe { display_id: 1 }));
    assert!(client.highlighted_displays.contains(&0));
    assert!(!client.highlighted_displays.contains(&1));
}

#[test]
fn c3_recipe_book_order_is_local_first_for_ui_and_ghost_first_for_failure() {
    let mut client = RecipeBookClientState {
        connected: true,
        highlighted_displays: [7].into_iter().collect(),
        ..RecipeBookClientState::default()
    };
    client.see(7);
    assert_eq!(
        client.events,
        [
            RecipeBookClientEvent::RemovedDisplayHighlight(7),
            RecipeBookClientEvent::SentSeen(RecipeBookSeenRecipe { display_id: 7 }),
        ]
    );

    let setting = RecipeBookChangeSettings {
        book_type: RecipeBookType::Crafting,
        open: true,
        filtering: false,
    };
    client.events.clear();
    client.change_settings(setting);
    assert_eq!(
        client.events,
        [
            RecipeBookClientEvent::ChangedSetting(setting),
            RecipeBookClientEvent::SentSettings(setting),
        ]
    );
}

#[test]
fn c3_recipe_book_end_to_end_decodes_places_and_converges_without_success_ack() {
    let wire = encode_packet(place_packet(4, 0, false)).unwrap();
    let PlayServerboundEntryPacket::PlaceRecipe(packet) = decode_packet(&wire).unwrap() else {
        panic!("place recipe packet");
    };
    let mut session = RecipePlacementSession {
        index: index(),
        ..RecipePlacementSession::default()
    };
    session.book.known.insert(id("minecraft:apple_trade"));
    let mut menu = RecipeMenuTransaction::new(4, 1, 1, 4);
    menu.player_inventory[0] = stack("minecraft:apple", 3);
    assert_eq!(
        session.handle_place_recipe(Some(&mut menu), packet),
        PlaceRecipeOutcome::Applied(PlacementMutation::Placed)
    );
    assert_eq!(menu.grid[0].count(), 1);
    assert!(menu.inventory_changed);
    assert_eq!(
        menu.brackets,
        [PlacementBracket::Begin, PlacementBracket::Finish]
    );
}
