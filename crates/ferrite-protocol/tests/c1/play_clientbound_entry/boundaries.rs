use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    GameMode, PlayClientboundPacket, PlayerPosition, Vector3,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::display::RecipeDisplay;
use ferrite_protocol::java_26_2::play::clientbound::recipe::slot::{
    DataComponentPatch, EncodedComponentValue, ItemStackTemplate, SlotDisplay,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::{
    RecipeBookAdd, RecipeBookEntry, RecipeError,
};
use ferrite_protocol::java_26_2::play::registry::PlayRegistryError;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

use super::fixtures::{context, id, registries};

fn write_login_body(dimension_type: i32) -> Vec<u8> {
    let mut body = WireWriter::new(1_024);
    body.write_var_i32(49).unwrap();
    body.write_i32(7).unwrap();
    body.write_bool(false).unwrap();
    body.write_count("levels", 2, 2).unwrap();
    body.write_utf("minecraft:overworld", 32_767).unwrap();
    body.write_utf("minecraft:overworld", 32_767).unwrap();
    body.write_var_i32(20).unwrap();
    body.write_var_i32(8).unwrap();
    body.write_var_i32(8).unwrap();
    body.write_bool(false).unwrap();
    body.write_bool(true).unwrap();
    body.write_bool(false).unwrap();
    body.write_var_i32(dimension_type).unwrap();
    body.write_utf("minecraft:overworld", 32_767).unwrap();
    body.write_i64(0).unwrap();
    body.write_i8(99).unwrap();
    body.write_i8(98).unwrap();
    body.write_bool(false).unwrap();
    body.write_bool(false).unwrap();
    body.write_bool(false).unwrap();
    body.write_var_i32(0).unwrap();
    body.write_var_i32(63).unwrap();
    body.write_bool(false).unwrap();
    body.write_bool(false).unwrap();
    body.into_inner()
}

#[test]
fn login_collapses_duplicate_levels_and_uses_survival_for_unknown_modes() {
    let registries = registries();
    let decoded = decode_packet(&write_login_body(0), context(&registries)).unwrap();
    let PlayClientboundPacket::Login(login) = decoded else {
        panic!("expected login");
    };
    assert_eq!(login.levels.len(), 1);
    assert_eq!(login.spawn.game_mode, GameMode::Survival);
    assert_eq!(login.spawn.previous_game_mode, Some(GameMode::Survival));

    assert!(matches!(
        decode_packet(&write_login_body(1), context(&registries)),
        Err(PlayClientboundCodecError::Registry(
            PlayRegistryError::UnknownRawId {
                registry: "minecraft:dimension_type",
                raw_id: 1
            }
        ))
    ));
}

#[test]
fn catalog_and_registry_dispatch_fail_closed() {
    let registries = registries();
    let mut unknown = WireWriter::new(8);
    unknown.write_var_i32(1_000).unwrap();
    assert!(matches!(
        decode_packet(unknown.as_slice(), context(&registries)),
        Err(PlayClientboundCodecError::UnknownPacketId { id: 1_000 })
    ));

    let mut outside_family = WireWriter::new(8);
    outside_family.write_var_i32(5).unwrap();
    assert!(matches!(
        decode_packet(outside_family.as_slice(), context(&registries)),
        Err(PlayClientboundCodecError::UnsupportedPacketIdentity { .. })
    ));
}

#[test]
fn duplicate_world_clock_raw_ids_are_rejected() {
    let registries = registries();
    let mut body = WireWriter::new(128);
    body.write_var_i32(113).unwrap();
    body.write_i64(10).unwrap();
    body.write_count("world clocks", 2, 2).unwrap();
    for total_ticks in [11, 12] {
        body.write_var_i32(0).unwrap();
        body.write_var_i64(total_ticks).unwrap();
        body.write_f32(0.5).unwrap();
        body.write_f32(1.0).unwrap();
    }
    assert!(matches!(
        decode_packet(body.as_slice(), context(&registries)),
        Err(PlayClientboundCodecError::DuplicateClock { .. })
    ));
}

fn item_stack_display(components: DataComponentPatch) -> RecipeDisplay {
    RecipeDisplay::CraftingShapeless {
        ingredients: Vec::new(),
        result: SlotDisplay::ItemStack(ItemStackTemplate {
            item: id("minecraft:stone"),
            count: 1,
            components,
        }),
        crafting_station: SlotDisplay::Empty,
    }
}

fn recipe_packet(display: RecipeDisplay) -> PlayClientboundPacket {
    PlayClientboundPacket::RecipeBookAdd(RecipeBookAdd {
        entries: vec![RecipeBookEntry {
            display_id: 0,
            display,
            group: None,
            category: id("minecraft:crafting_building_blocks"),
            crafting_requirements: None,
            show_notification: false,
            highlight: false,
        }],
        replace: false,
    })
}

#[test]
fn component_patch_rejects_duplicate_added_and_removed_types() {
    let registries = registries();
    let component = id("minecraft:custom_data");
    let packet = recipe_packet(item_stack_display(DataComponentPatch {
        added: vec![EncodedComponentValue {
            component: component.clone(),
            encoded_value: vec![0],
        }],
        removed: vec![component],
    }));
    assert!(matches!(
        encode_packet(&packet, &registries),
        Err(PlayClientboundCodecError::Recipe(
            RecipeError::DuplicateComponent { .. }
        ))
    ));
}

#[test]
fn recursive_slot_displays_stop_at_the_locked_depth_limit() {
    let registries = registries();
    let mut display = SlotDisplay::Empty;
    for _ in 0..512 {
        display = SlotDisplay::Composite(vec![display]);
    }
    let packet = recipe_packet(RecipeDisplay::CraftingShapeless {
        ingredients: vec![display],
        result: SlotDisplay::Empty,
        crafting_station: SlotDisplay::Empty,
    });
    assert!(matches!(
        encode_packet(&packet, &registries),
        Err(PlayClientboundCodecError::Recipe(
            RecipeError::SlotDisplayDepth { maximum: 512 }
        ))
    ));

    let mut body = WireWriter::new(4_096);
    body.write_var_i32(74).unwrap();
    body.write_count("recipe entries", 1, 1).unwrap();
    body.write_var_i32(0).unwrap();
    body.write_var_i32(0).unwrap();
    body.write_count("ingredients", 1, 1).unwrap();
    for _ in 0..512 {
        body.write_var_i32(10).unwrap();
        body.write_count("composite", 1, 1).unwrap();
    }
    body.write_var_i32(0).unwrap();
    body.write_var_i32(0).unwrap();
    body.write_var_i32(0).unwrap();
    body.write_var_i32(0).unwrap();
    body.write_var_i32(0).unwrap();
    body.write_bool(false).unwrap();
    body.write_u8(0).unwrap();
    body.write_bool(false).unwrap();
    assert!(matches!(
        decode_packet(body.as_slice(), context(&registries)),
        Err(PlayClientboundCodecError::Recipe(
            RecipeError::SlotDisplayDepth { maximum: 512 }
        ))
    ));
}

#[test]
fn player_position_preserves_non_finite_values_and_unknown_flag_bits() {
    let registries = registries();
    let packet = PlayClientboundPacket::PlayerPosition(PlayerPosition {
        teleport_id: i32::MIN,
        position: Vector3 {
            x: f64::from_bits(0x7ff8_0000_0000_0042),
            y: f64::INFINITY,
            z: f64::NEG_INFINITY,
        },
        motion: Vector3 {
            x: -0.0,
            y: f64::MIN,
            z: f64::MAX,
        },
        yaw: f32::from_bits(0x7fc0_0042),
        pitch: f32::NEG_INFINITY,
        relative_flags: u32::MAX,
    });
    let body = encode_packet(&packet, &registries).unwrap();
    let PlayClientboundPacket::PlayerPosition(decoded) =
        decode_packet(&body, context(&registries)).unwrap()
    else {
        panic!("expected player position");
    };
    let PlayClientboundPacket::PlayerPosition(expected) = packet else {
        unreachable!();
    };
    assert_eq!(decoded.teleport_id, expected.teleport_id);
    assert_eq!(decoded.relative_flags, expected.relative_flags);
    assert_eq!(decoded.position.x.to_bits(), expected.position.x.to_bits());
    assert_eq!(decoded.position.y.to_bits(), expected.position.y.to_bits());
    assert_eq!(decoded.position.z.to_bits(), expected.position.z.to_bits());
    assert_eq!(decoded.motion.x.to_bits(), expected.motion.x.to_bits());
    assert_eq!(decoded.motion.y.to_bits(), expected.motion.y.to_bits());
    assert_eq!(decoded.motion.z.to_bits(), expected.motion.z.to_bits());
    assert_eq!(decoded.yaw.to_bits(), expected.yaw.to_bits());
    assert_eq!(decoded.pitch.to_bits(), expected.pitch.to_bits());
}
