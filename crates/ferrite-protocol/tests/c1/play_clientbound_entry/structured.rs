use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::login::profile::ProfileProperty;
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::command::{
    CommandArgumentPayload, CommandNode, CommandNodeKind, CommandTree, CommandTreeError,
    NumericBounds, StringArgumentKind,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{GameMode, PlayClientboundPacket};
use ferrite_protocol::java_26_2::play::clientbound::player_info::{
    AddedProfile, ChatSession, PlayerInfoActions, PlayerInfoEntry, PlayerInfoUpdate,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::display::RecipeDisplay;
use ferrite_protocol::java_26_2::play::clientbound::recipe::slot::{HolderSet, SlotDisplay};
use ferrite_protocol::java_26_2::play::clientbound::recipe::{
    RecipeBookAdd, RecipeBookEntry, RecipeError, RecipeProjection, StonecutterSelection,
};
use ferrite_protocol::java_26_2::play::context::{
    ComponentValueDecoder, ComponentValueError, PlayDecodeContext,
};
use ferrite_protocol::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStackTemplate,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;
use ferrite_protocol::java_26_2::wire::primitive::{WireReader, WireWriter};

use super::fixtures::{context, id, registries};

struct OneByteComponent;

impl ComponentValueDecoder for OneByteComponent {
    fn decode_value(
        &self,
        _component: &Identifier,
        reader: &mut WireReader<'_>,
    ) -> Result<Vec<u8>, ComponentValueError> {
        Ok(vec![reader.read_u8().map_err(|error| {
            ComponentValueError::Malformed {
                component: id("minecraft:custom_data"),
                reason: error.to_string(),
            }
        })?])
    }
}

fn argument(name: &str, argument_type: &str, payload: CommandArgumentPayload) -> CommandNode {
    CommandNode {
        executable: true,
        restricted: false,
        children: Vec::new(),
        redirect: None,
        kind: CommandNodeKind::Argument {
            name: name.to_owned(),
            argument_type: id(argument_type),
            payload,
            suggestion_provider: None,
        },
    }
}

#[test]
fn command_tree_dispatches_every_payload_shape_and_validates_graphs() {
    let registries = registries();
    let mut nodes = vec![CommandNode {
        executable: false,
        restricted: false,
        children: (1..=11).collect(),
        redirect: None,
        kind: CommandNodeKind::Root,
    }];
    nodes.extend([
        argument(
            "float",
            "brigadier:float",
            CommandArgumentPayload::Float(NumericBounds {
                minimum: Some(-1.0),
                maximum: Some(1.0),
            }),
        ),
        argument(
            "double",
            "brigadier:double",
            CommandArgumentPayload::Double(NumericBounds {
                minimum: None,
                maximum: Some(2.0),
            }),
        ),
        argument(
            "integer",
            "brigadier:integer",
            CommandArgumentPayload::Integer(NumericBounds {
                minimum: Some(i32::MIN),
                maximum: None,
            }),
        ),
        argument(
            "long",
            "brigadier:long",
            CommandArgumentPayload::Long(NumericBounds {
                minimum: None,
                maximum: None,
            }),
        ),
        argument(
            "string",
            "brigadier:string",
            CommandArgumentPayload::String(StringArgumentKind::GreedyPhrase),
        ),
        argument(
            "entity",
            "minecraft:entity",
            CommandArgumentPayload::Flags(3),
        ),
        argument(
            "score",
            "minecraft:score_holder",
            CommandArgumentPayload::Flags(1),
        ),
        argument(
            "time",
            "minecraft:time",
            CommandArgumentPayload::TimeMinimum(-20),
        ),
        argument(
            "resource",
            "minecraft:resource_or_tag",
            CommandArgumentPayload::Registry(id("minecraft:item")),
        ),
        argument("bool", "brigadier:bool", CommandArgumentPayload::None),
        CommandNode {
            executable: false,
            restricted: true,
            children: Vec::new(),
            redirect: Some(1),
            kind: CommandNodeKind::Literal {
                name: "literal".to_owned(),
            },
        },
    ]);
    let tree = CommandTree {
        nodes,
        root_index: 0,
    };
    let packet = PlayClientboundPacket::Commands(tree);
    let body = encode_packet(&packet, &registries).unwrap();
    assert_eq!(decode_packet(&body, context(&registries)).unwrap(), packet);

    let cycle = PlayClientboundPacket::Commands(CommandTree {
        nodes: vec![CommandNode {
            executable: false,
            restricted: false,
            children: vec![0],
            redirect: None,
            kind: CommandNodeKind::Root,
        }],
        root_index: 0,
    });
    assert!(matches!(
        encode_packet(&cycle, &registries),
        Err(PlayClientboundCodecError::CommandTree(
            CommandTreeError::Cycle { .. }
        ))
    ));
}

#[test]
fn command_placeholders_skip_reachable_children_and_unreachable_bad_references() {
    let registries = registries();
    let mut body = WireWriter::new(256);
    body.write_var_i32(16).unwrap();
    body.write_count("command nodes", 3, 3).unwrap();
    body.write_u8(0).unwrap();
    body.write_count("children", 1, 1).unwrap();
    body.write_var_i32(1).unwrap();
    body.write_u8(0x12).unwrap();
    body.write_count("children", 0, 0).unwrap();
    body.write_utf("unknown", 32_767).unwrap();
    body.write_var_i32(999).unwrap();
    body.write_utf("minecraft:ask_server", 32_767).unwrap();
    body.write_u8(8).unwrap();
    body.write_count("children", 1, 1).unwrap();
    body.write_var_i32(999).unwrap();
    body.write_var_i32(999).unwrap();
    body.write_var_i32(0).unwrap();

    let packet = decode_packet(body.as_slice(), context(&registries)).unwrap();
    let PlayClientboundPacket::Commands(tree) = packet else {
        panic!("expected command tree");
    };
    assert!(matches!(tree.nodes[1].kind, CommandNodeKind::Placeholder));

    let mut trailing = body.into_inner();
    trailing.push(1);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

fn player_entry(mask: u8) -> PlayerInfoEntry {
    let actions = PlayerInfoActions::from_bits(mask);
    PlayerInfoEntry {
        profile_id: 7,
        added_profile: actions
            .contains(PlayerInfoActions::ADD_PLAYER)
            .then(|| AddedProfile {
                name: "Player".to_owned(),
                properties: vec![ProfileProperty {
                    name: "textures".to_owned(),
                    value: "value".to_owned(),
                    signature: Some("signature".to_owned()),
                }],
            }),
        chat_session: actions
            .contains(PlayerInfoActions::INITIALIZE_CHAT)
            .then(|| {
                Some(ChatSession {
                    session_id: 8,
                    expires_at_millis: 9,
                    public_key: vec![1, 2],
                    key_signature: vec![3, 4],
                })
            }),
        game_mode: actions
            .contains(PlayerInfoActions::UPDATE_GAME_MODE)
            .then_some(GameMode::Spectator),
        listed: actions
            .contains(PlayerInfoActions::UPDATE_LISTED)
            .then_some(true),
        latency_millis: actions
            .contains(PlayerInfoActions::UPDATE_LATENCY)
            .then_some(-7),
        display_name: actions
            .contains(PlayerInfoActions::UPDATE_DISPLAY_NAME)
            .then(|| Some(TextComponentNbt::literal("Display").unwrap())),
        list_order: actions
            .contains(PlayerInfoActions::UPDATE_LIST_ORDER)
            .then_some(-3),
        show_hat: actions
            .contains(PlayerInfoActions::UPDATE_HAT)
            .then_some(false),
    }
}

#[test]
fn player_info_reads_all_action_masks_in_bit_order() {
    let registries = registries();
    for mask in 0..=u8::MAX {
        let packet = PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
            actions: PlayerInfoActions::from_bits(mask),
            entries: vec![player_entry(mask)],
        });
        let body = encode_packet(&packet, &registries).unwrap();
        assert_eq!(decode_packet(&body, context(&registries)).unwrap(), packet);
    }

    let mut invalid = player_entry(PlayerInfoActions::ADD_PLAYER);
    invalid.added_profile = None;
    assert!(
        encode_packet(
            &PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
                actions: PlayerInfoActions::from_bits(PlayerInfoActions::ADD_PLAYER),
                entries: vec![invalid],
            }),
            &registries,
        )
        .is_err()
    );
}

fn all_slots() -> Vec<SlotDisplay> {
    let empty = || Box::new(SlotDisplay::Empty);
    vec![
        SlotDisplay::Empty,
        SlotDisplay::AnyFuel,
        SlotDisplay::WithAnyPotion,
        SlotDisplay::OnlyWithComponent {
            source: empty(),
            component: id("minecraft:custom_data"),
        },
        SlotDisplay::Item(id("minecraft:stone")),
        SlotDisplay::ItemStack(ItemStackTemplate {
            item: id("minecraft:stone"),
            count: -2,
            components: DataComponentPatch {
                added: vec![EncodedComponentValue {
                    component: id("minecraft:custom_data"),
                    encoded_value: vec![42],
                }],
                removed: Vec::new(),
            },
        }),
        SlotDisplay::Tag(id("minecraft:stones")),
        SlotDisplay::Dyed {
            dye: empty(),
            target: empty(),
        },
        SlotDisplay::SmithingTrim {
            base: empty(),
            material: empty(),
            pattern: id("minecraft:sentry"),
        },
        SlotDisplay::WithRemainder {
            input: empty(),
            remainder: empty(),
        },
        SlotDisplay::Composite(vec![SlotDisplay::Empty, SlotDisplay::AnyFuel]),
    ]
}

fn all_displays() -> Vec<RecipeDisplay> {
    let mut slots = all_slots().into_iter();
    vec![
        RecipeDisplay::CraftingShapeless {
            ingredients: vec![slots.next().unwrap()],
            result: slots.next().unwrap(),
            crafting_station: slots.next().unwrap(),
        },
        RecipeDisplay::CraftingShaped {
            width: 1,
            height: 2,
            ingredients: vec![slots.next().unwrap(), slots.next().unwrap()],
            result: slots.next().unwrap(),
            crafting_station: slots.next().unwrap(),
        },
        RecipeDisplay::Furnace {
            ingredient: slots.next().unwrap(),
            fuel: slots.next().unwrap(),
            result: slots.next().unwrap(),
            crafting_station: slots.next().unwrap(),
            duration: -20,
            experience: f32::INFINITY,
        },
        RecipeDisplay::Stonecutter {
            input: SlotDisplay::Empty,
            result: SlotDisplay::AnyFuel,
            crafting_station: SlotDisplay::WithAnyPotion,
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

#[test]
fn recipe_and_slot_dispatch_cover_every_locked_type() {
    let registries = registries();
    let entries = all_displays()
        .into_iter()
        .enumerate()
        .map(|(index, display)| RecipeBookEntry {
            display_id: index as i32,
            display,
            group: Some(index as i32),
            category: id("minecraft:crafting_building_blocks"),
            crafting_requirements: Some(vec![
                HolderSet::Tag(id("minecraft:stones")),
                HolderSet::Direct(vec![id("minecraft:air"), id("minecraft:stone")]),
            ]),
            show_notification: index % 2 == 0,
            highlight: index % 2 != 0,
        })
        .collect();
    let packet = PlayClientboundPacket::RecipeBookAdd(RecipeBookAdd {
        entries,
        replace: true,
    });
    let body = encode_packet(&packet, &registries).unwrap();
    let component_decoder = OneByteComponent;
    let decoded = decode_packet(
        &body,
        PlayDecodeContext {
            registries: &registries,
            component_values: &component_decoder,
            dimension_section_count: 24,
        },
    )
    .unwrap();
    assert_eq!(decoded, packet);
}

#[test]
fn recipe_projection_resolves_items_and_rejects_malformed_dispatch() {
    let registries = registries();
    let packet = PlayClientboundPacket::UpdateRecipes(RecipeProjection {
        properties: BTreeMap::from([(id("minecraft:furnace_input"), vec![id("minecraft:stone")])]),
        stonecutter: vec![StonecutterSelection {
            input: HolderSet::Direct(vec![id("minecraft:stone")]),
            display: SlotDisplay::Item(id("minecraft:stone")),
        }],
    });
    let body = encode_packet(&packet, &registries).unwrap();
    assert_eq!(decode_packet(&body, context(&registries)).unwrap(), packet);

    let malformed = PlayClientboundPacket::RecipeBookAdd(RecipeBookAdd {
        entries: vec![RecipeBookEntry {
            display_id: 0,
            display: RecipeDisplay::CraftingShaped {
                width: 2,
                height: 2,
                ingredients: vec![SlotDisplay::Empty],
                result: SlotDisplay::Empty,
                crafting_station: SlotDisplay::Empty,
            },
            group: None,
            category: id("minecraft:crafting_building_blocks"),
            crafting_requirements: None,
            show_notification: false,
            highlight: false,
        }],
        replace: false,
    });
    assert!(matches!(
        encode_packet(&malformed, &registries),
        Err(PlayClientboundCodecError::Recipe(
            RecipeError::ShapedDimensions { .. }
        ))
    ));
}
