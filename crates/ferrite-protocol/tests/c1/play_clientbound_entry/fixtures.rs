use std::collections::{BTreeMap, BTreeSet};

use ferrite_protocol::java_26_2::play::clientbound::command::{
    CommandNode, CommandNodeKind, CommandTree,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, GameMode, PlayClientboundPacket, PlayLogin,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::RecipeProjection;
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{
    COMMAND_ARGUMENT_TYPE, DATA_COMPONENT_TYPE, DIMENSION_TYPE, ITEM, PlayRegistries,
    RECIPE_BOOK_CATEGORY, RECIPE_DISPLAY, SLOT_DISPLAY, TRIM_PATTERN, WORLD_CLOCK,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::wire::compression::{
    CompressionMode, encode_packet as encode_wire,
};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;

static REJECT_COMPONENTS: RejectComponentValues = RejectComponentValues;

pub fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

pub fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}

pub fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &REJECT_COMPONENTS,
        dimension_section_count: 24,
    }
}

pub fn golden_frame(body: &[u8]) -> Vec<u8> {
    encode_wire(
        body,
        CompressionMode::enabled(256).unwrap(),
        FrameLimits::default(),
    )
    .unwrap()
}

pub fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(id(DIMENSION_TYPE), vec![id("minecraft:overworld")]);
    registries.insert(id("minecraft:worldgen/biome"), vec![id("minecraft:plains")]);
    registries.insert(id(WORLD_CLOCK), vec![id("minecraft:day_time")]);
    registries.insert(
        id(COMMAND_ARGUMENT_TYPE),
        [
            "brigadier:bool",
            "brigadier:float",
            "brigadier:double",
            "brigadier:integer",
            "brigadier:long",
            "brigadier:string",
            "minecraft:entity",
            "minecraft:game_profile",
            "minecraft:block_pos",
            "minecraft:column_pos",
            "minecraft:vec3",
            "minecraft:vec2",
            "minecraft:block_state",
            "minecraft:block_predicate",
            "minecraft:item_stack",
            "minecraft:item_predicate",
            "minecraft:team_color",
            "minecraft:hex_color",
            "minecraft:component",
            "minecraft:style",
            "minecraft:message",
            "minecraft:nbt_compound_tag",
            "minecraft:nbt_tag",
            "minecraft:nbt_path",
            "minecraft:objective",
            "minecraft:objective_criteria",
            "minecraft:operation",
            "minecraft:particle",
            "minecraft:angle",
            "minecraft:rotation",
            "minecraft:scoreboard_slot",
            "minecraft:score_holder",
            "minecraft:swizzle",
            "minecraft:team",
            "minecraft:item_slot",
            "minecraft:item_slots",
            "minecraft:resource_location",
            "minecraft:function",
            "minecraft:entity_anchor",
            "minecraft:int_range",
            "minecraft:float_range",
            "minecraft:dimension",
            "minecraft:gamemode",
            "minecraft:time",
            "minecraft:resource_or_tag",
            "minecraft:resource_or_tag_key",
            "minecraft:resource",
            "minecraft:resource_key",
            "minecraft:resource_selector",
            "minecraft:template_mirror",
            "minecraft:template_rotation",
            "minecraft:heightmap",
            "minecraft:loot_table",
            "minecraft:loot_predicate",
            "minecraft:loot_modifier",
            "minecraft:dialog",
            "minecraft:uuid",
        ]
        .into_iter()
        .map(id)
        .collect(),
    );
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
    registries.insert(
        id(RECIPE_BOOK_CATEGORY),
        vec![id("minecraft:crafting_building_blocks")],
    );
    registries.insert(id(ITEM), vec![id("minecraft:air"), id("minecraft:stone")]);
    registries.insert(id(DATA_COMPONENT_TYPE), vec![id("minecraft:custom_data")]);
    registries.insert(id(TRIM_PATTERN), vec![id("minecraft:sentry")]);
    registries
}

pub fn login() -> PlayClientboundPacket {
    PlayClientboundPacket::Login(PlayLogin {
        player_entity_id: 1,
        hardcore: false,
        levels: BTreeSet::from([id("minecraft:overworld")]),
        max_players: 20,
        chunk_radius: 2,
        simulation_distance: 2,
        reduced_debug_info: false,
        show_death_screen: true,
        limited_crafting: false,
        spawn: CommonSpawnInfo {
            dimension_type: id("minecraft:overworld"),
            dimension: id("minecraft:overworld"),
            obfuscated_seed: 0,
            game_mode: GameMode::Survival,
            previous_game_mode: None,
            is_debug: false,
            is_flat: false,
            last_death: None,
            portal_cooldown: 0,
            sea_level: 63,
        },
        online_mode: false,
        enforces_secure_chat: false,
    })
}

pub fn empty_commands() -> CommandTree {
    CommandTree {
        nodes: vec![CommandNode {
            executable: false,
            restricted: false,
            children: Vec::new(),
            redirect: None,
            kind: CommandNodeKind::Root,
        }],
        root_index: 0,
    }
}

pub fn empty_recipes() -> RecipeProjection {
    RecipeProjection {
        properties: BTreeMap::new(),
        stonecutter: Vec::new(),
    }
}
