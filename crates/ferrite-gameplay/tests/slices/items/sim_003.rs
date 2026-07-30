use ferrite_foundation::direction::Direction;
use ferrite_gameplay::block::tripwire::{
    ContactInput, HookSound, HorizontalNeighbor, SHEARS_DISARM, ScanCell, TripwireState,
    connects_to, contact, contact_shape, first_source_hook, hook_signal, hook_sound,
    recalculate_hook, scheduled_rescan,
};
use ferrite_gameplay::item::runtime::string::{
    DIRECT_ACQUISITION_TABLES, STRING_ITEM_ID, STRING_MAXIMUM_STACK, STRING_RECIPES, STRING_TRADES,
    STRING_UNLOCKS, STRUCTURE_STRING, fishing_junk_probability_denominator, looting_bonus,
    verify_string_family,
};
use ferrite_registry::bundle::ContentBundle;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn string_identity_and_catalog_joins_are_exact() {
    assert_eq!((STRING_ITEM_ID, STRING_MAXIMUM_STACK), (976, 64));
    assert_eq!(
        DIRECT_ACQUISITION_TABLES
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        17
    );
    assert_eq!(STRING_RECIPES.len(), 9);
    assert_eq!(STRING_UNLOCKS.len(), 9);
    assert_eq!(
        STRING_RECIPES
            .iter()
            .map(|recipe| (recipe.id, recipe.string_count, recipe.output_count))
            .collect::<Vec<_>>(),
        [
            ("bow", 3, 1),
            ("bundle", 1, 1),
            ("candle", 1, 1),
            ("crossbow", 2, 1),
            ("fishing_rod", 2, 1),
            ("lead", 5, 2),
            ("loom", 2, 1),
            ("scaffolding", 1, 6),
            ("white_wool_from_string", 4, 1),
        ]
    );
    assert!(!STRING_UNLOCKS.contains(&"scaffolding"));
    assert!(STRING_UNLOCKS.contains(&"tripwire_hook"));

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("../../target/ferrite-content/26.2/content-bundle.json");
    if !path.is_file() {
        eprintln!(
            "locked local artifact bundle is absent; `cargo ferrite content verify` owns that gate"
        );
        return;
    }
    let bundle = serde_json::from_slice::<ContentBundle>(&fs::read(path).unwrap()).unwrap();
    let registry = bundle
        .registries()
        .find(|registry| registry.name().to_string() == "minecraft:item")
        .unwrap();
    verify_string_family(registry).unwrap();
}

#[test]
fn acquisition_trades_structure_and_looting_keep_source_values() {
    assert_eq!(
        STRING_TRADES.map(|trade| (
            trade.profession,
            trade.level,
            trade.string_cost,
            trade.emerald_output,
            trade.inclusion_probability,
            trade.maximum_uses,
            trade.villager_experience,
            trade.price_multiplier,
        )),
        [
            ("fisherman", 1, 20, 1, 0.5, 16, 2, 0.05),
            ("fletcher", 3, 14, 1, 1.0, 16, 20, 0.05),
        ]
    );
    assert_eq!(
        (
            STRUCTURE_STRING.decoded_templates,
            STRUCTURE_STRING.matching_templates,
            STRUCTURE_STRING.path,
            STRUCTURE_STRING.stored_count,
        ),
        (1_212, 1, "trial_chambers/intersection/intersection_2", 3,)
    );
    assert_eq!(fishing_junk_probability_denominator(false), 20);
    assert_eq!(fishing_junk_probability_denominator(true), 22);
    assert_eq!(looting_bonus(3, 0.49), 1);
    assert_eq!(looting_bonus(3, 0.50), 2);
}

#[test]
fn tripwire_state_connections_transforms_and_shapes_are_exact() {
    let mut state = TripwireState::default();
    state.set_connected(Direction::North, true);
    state.set_connected(Direction::West, true);
    state.attached = true;
    assert!(state.connected(Direction::North));
    assert!(!state.connected(Direction::Up));
    assert_eq!(
        state.clockwise(),
        TripwireState {
            attached: true,
            north: true,
            east: true,
            ..TripwireState::default()
        }
    );
    assert!(state.mirror_left_right().south);
    assert!(state.mirror_front_back().east);
    assert!(connects_to(Direction::East, HorizontalNeighbor::Tripwire));
    assert!(connects_to(
        Direction::East,
        HorizontalNeighbor::Hook {
            facing: Direction::West,
        }
    ));
    assert!(!connects_to(
        Direction::East,
        HorizontalNeighbor::Hook {
            facing: Direction::East,
        }
    ));
    assert_eq!(
        (contact_shape(true).minimum_y, contact_shape(true).maximum_y),
        (1.0 / 16.0, 2.5 / 16.0)
    );
    assert_eq!(
        (
            contact_shape(false).minimum_y,
            contact_shape(false).maximum_y
        ),
        (0.0, 0.5)
    );
}

#[test]
fn source_hook_scan_is_limited_to_south_and_west_and_stops_at_obstacles() {
    let wire = ScanCell::Wire(TripwireState::default());
    let south_hook = ScanCell::Hook {
        facing: Direction::North,
    };
    assert_eq!(
        first_source_hook(Direction::South, &[wire, wire, south_hook]),
        Some(3)
    );
    assert_eq!(
        first_source_hook(Direction::North, &[wire, south_hook]),
        None
    );
    assert_eq!(
        first_source_hook(Direction::South, &[wire, ScanCell::Other, south_hook]),
        None
    );
    assert_eq!(
        first_source_hook(
            Direction::West,
            &[ScanCell::Wire(TripwireState::default()); 41]
        ),
        None
    );
}

#[test]
fn contact_and_scheduled_rescan_preserve_server_tick_transitions() {
    let pressed = contact(ContactInput {
        server_side: true,
        currently_powered: false,
        scheduled_tick_pending: false,
        triggering_entity_present: true,
    });
    assert_eq!(
        (
            pressed.changed,
            pressed.powered,
            pressed.write_flags,
            pressed.recalculate_hook,
            pressed.wire_rescan_delay,
        ),
        (true, true, 3, true, Some(10))
    );
    assert!(
        !contact(ContactInput {
            server_side: false,
            currently_powered: false,
            scheduled_tick_pending: false,
            triggering_entity_present: true,
        })
        .changed
    );
    assert!(
        !contact(ContactInput {
            server_side: true,
            currently_powered: false,
            scheduled_tick_pending: true,
            triggering_entity_present: true,
        })
        .changed
    );
    assert_eq!(scheduled_rescan(true, true).wire_rescan_delay, Some(10));
    let released = scheduled_rescan(true, false);
    assert_eq!(
        (
            released.changed,
            released.powered,
            released.recalculate_hook,
            released.release_hook_delay,
        ),
        (true, false, true, Some(0))
    );
    assert!(!scheduled_rescan(false, true).changed);
}

#[test]
fn hook_transaction_enforces_length_arming_and_attachment_rewrites() {
    let powered = ScanCell::Wire(TripwireState {
        powered: true,
        ..TripwireState::default()
    });
    let opposite = ScanCell::Hook {
        facing: Direction::West,
    };
    let attached = recalculate_hook(Direction::East, &[powered, opposite], false, false, false);
    assert_eq!(attached.opposite_hook_distance, Some(2));
    assert!(attached.attached);
    assert!(attached.powered);
    assert_eq!(attached.rewritten_intermediate_wires, 1);
    assert!(attached.write_opposite_hook);
    assert!(attached.write_origin_hook);

    let too_close = recalculate_hook(Direction::East, &[opposite], false, false, false);
    assert_eq!(too_close.opposite_hook_distance, None);

    let disarmed = TripwireState {
        disarmed: true,
        powered: true,
        ..TripwireState::default()
    };
    let detached = recalculate_hook(
        Direction::East,
        &[ScanCell::Wire(disarmed), opposite],
        false,
        false,
        true,
    );
    assert!(!detached.attached);
    assert!(!detached.powered);
    assert_eq!(detached.rewritten_intermediate_wires, 1);

    let forty_wires = vec![ScanCell::Wire(TripwireState::default()); 40];
    let mut maximum_line = forty_wires;
    maximum_line.push(opposite);
    let maximum = recalculate_hook(Direction::East, &maximum_line, false, false, false);
    assert_eq!(maximum.opposite_hook_distance, Some(41));
    assert!(maximum.attached);

    let removed = recalculate_hook(Direction::East, &[powered, opposite], true, true, true);
    assert!(!removed.attached);
    assert!(!removed.write_origin_hook);
}

#[test]
fn hook_signals_sounds_and_shears_contract_are_exact() {
    assert_eq!(
        hook_sound(false, false, true, true, 0.0),
        Some(HookSound::Activate {
            volume: 0.4,
            pitch: 0.6,
        })
    );
    assert_eq!(
        hook_sound(true, true, true, false, 0.0),
        Some(HookSound::Deactivate {
            volume: 0.4,
            pitch: 0.5,
        })
    );
    assert_eq!(
        hook_sound(false, false, true, false, 0.0),
        Some(HookSound::Attach {
            volume: 0.4,
            pitch: 0.7,
        })
    );
    assert_eq!(
        hook_sound(true, false, false, false, 0.5),
        Some(HookSound::Detach {
            volume: 0.4,
            pitch: 1.2,
        })
    );
    assert_eq!(
        hook_signal(true, Direction::East, Direction::East),
        (15, 15)
    );
    assert_eq!(hook_signal(true, Direction::West, Direction::East), (15, 0));
    assert_eq!(hook_signal(false, Direction::East, Direction::East), (0, 0));
    const {
        assert!(SHEARS_DISARM.write_flags == 260);
        assert!(SHEARS_DISARM.disarmed_before_removal);
        assert!(SHEARS_DISARM.shear_game_event);
        assert!(!SHEARS_DISARM.string_loot_suppressed);
    }
}
