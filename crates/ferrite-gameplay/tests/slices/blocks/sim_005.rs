use std::collections::{BTreeSet, VecDeque};

use ferrite_foundation::direction::Direction;
use ferrite_gameplay::block::bell::{
    Aabb16, BELL_RING_STAT_ID, BLOCK_ENTITY_PROTOCOL_ID as BELL_BLOCK_ENTITY_ID,
    BLOCK_ID as BELL_BLOCK_ID, BellAttachment, BellHit, BellLiving, BellMirror, BellRotation,
    BellRuntime, BellSide, BellState, BellSupportUpdate, BellSupports, CACHE_INTERVAL,
    DEFAULT_STATE_ID as BELL_DEFAULT_STATE_ID, ENTITY_EFFECT_PARTICLE_ID,
    FIRST_STATE_ID as BELL_FIRST_STATE_ID, FORCED_SOLID, GLOW_AMPLIFIER, GLOW_TICKS,
    HARDNESS as BELL_HARDNESS, HEARD_BELL_MEMORY_ID, HIT_Y_LIMIT, ITEM_ID as BELL_ITEM_ID,
    RAIDER_ENTITY_TYPE_IDS, RESISTANCE as BELL_RESISTANCE, RESONANCE_TICKS, RING_EVENT_ID,
    RING_SOUND_ID, STATE_COUNT as BELL_STATE_COUNT, bell_explosion, bell_loot_survives_explosion,
    bell_neighbour_signal, bell_placement, bell_render_rotation, bell_shape, bell_support_update,
    bell_use_without_item_admitted, mirror_bell, proper_bell_hit, rotate_bell,
};
use ferrite_gameplay::block::enchanting_table::{
    BLOCK_ENTITY_PROTOCOL_ID as TABLE_BLOCK_ENTITY_ID, BLOCK_ID as TABLE_BLOCK_ID,
    BLOCK_STATE_ID as TABLE_STATE_ID, BOOKSHELF_PROBE_COUNT, BookAnimation, BookshelfProbe,
    ClientPlayer, DEFAULT_TITLE, ENCHANT_PARTICLE_ID, EnchantingTableData,
    HARDNESS as TABLE_HARDNESS, ITEM_ID as TABLE_ITEM_ID, LIGHT_LEVEL, MENU_PROTOCOL_ID,
    REQUIRES_CORRECT_TOOL, RESISTANCE as TABLE_RESISTANCE, SHAPE_HEIGHT, StoredCustomName,
    TableRandom, TableSide, USES_SHAPE_FOR_LIGHT_OCCLUSION, bookshelf_offsets,
    enchanting_particle_scan, enchanting_particles, enchanting_table_loot, enchanting_table_pick,
    enchanting_table_render, enchanting_table_use, enchanting_use_without_item_admitted,
};

#[derive(Debug, Default)]
struct RandomScript {
    ints: VecDeque<u32>,
    floats: VecDeque<f32>,
    bounds: Vec<u32>,
    float_draws: usize,
}

impl RandomScript {
    fn new(ints: impl IntoIterator<Item = u32>, floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            ints: ints.into_iter().collect(),
            floats: floats.into_iter().collect(),
            bounds: Vec::new(),
            float_draws: 0,
        }
    }
}

impl TableRandom for RandomScript {
    fn next_int(&mut self, bound: u32) -> u32 {
        self.bounds.push(bound);
        let value = self.ints.pop_front().expect("scripted bounded draw");
        assert!(value < bound);
        value
    }

    fn next_float(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.pop_front().expect("scripted float draw")
    }
}

fn bell_state(facing: Direction, attachment: BellAttachment) -> BellState {
    BellState {
        facing,
        attachment,
        powered: false,
    }
}

fn living(id: u64, position: [f64; 3], alive: bool, removed: bool, raider: bool) -> BellLiving {
    BellLiving {
        id,
        position,
        alive,
        removed,
        raider,
    }
}

fn near_raider(id: u64) -> BellLiving {
    living(id, [1.0, 0.5, 0.5], true, false, true)
}

#[test]
fn bell_registry_and_all_32_state_ids_are_exact() {
    assert_eq!(
        (
            BELL_BLOCK_ID,
            BELL_ITEM_ID,
            BELL_BLOCK_ENTITY_ID,
            BELL_DEFAULT_STATE_ID,
        ),
        (848, 1_393, 30, 20_806)
    );
    assert_eq!(
        (
            RING_SOUND_ID,
            ENTITY_EFFECT_PARTICLE_ID,
            HEARD_BELL_MEMORY_ID,
            BELL_RING_STAT_ID,
        ),
        (167, 28, 29, 70)
    );
    assert_eq!(
        (BELL_HARDNESS, BELL_RESISTANCE, FORCED_SOLID),
        (5.0, 5.0, true)
    );
    assert_eq!(GLOW_AMPLIFIER, 0);
    assert_eq!(RAIDER_ENTITY_TYPE_IDS, [46, 103, 109, 141, 68, 145]);

    let mut ids = BTreeSet::new();
    for attachment in [
        BellAttachment::Floor,
        BellAttachment::Ceiling,
        BellAttachment::SingleWall,
        BellAttachment::DoubleWall,
    ] {
        for facing in Direction::HORIZONTAL {
            for powered in [true, false] {
                ids.insert(
                    BellState {
                        facing,
                        attachment,
                        powered,
                    }
                    .state_id()
                    .expect("horizontal bell"),
                );
            }
        }
    }
    assert_eq!(ids.len() as u32, BELL_STATE_COUNT);
    assert_eq!(ids.first(), Some(&BELL_FIRST_STATE_ID));
    assert_eq!(ids.last(), Some(&(BELL_FIRST_STATE_ID + 31)));
    assert_eq!(BellState::default().state_id(), Some(BELL_DEFAULT_STATE_ID));
    assert_eq!(
        BellState {
            facing: Direction::Up,
            ..BellState::default()
        }
        .state_id(),
        None
    );
}

#[test]
fn bell_rotation_mirror_and_empty_hand_routing_change_only_facing() {
    let state = BellState {
        facing: Direction::North,
        attachment: BellAttachment::DoubleWall,
        powered: true,
    };
    assert_eq!(
        rotate_bell(state, BellRotation::Clockwise90),
        BellState {
            facing: Direction::East,
            ..state
        }
    );
    assert_eq!(
        rotate_bell(state, BellRotation::CounterClockwise90).facing,
        Direction::West
    );
    assert_eq!(
        mirror_bell(state, BellMirror::LeftRight).facing,
        Direction::South
    );
    assert_eq!(
        mirror_bell(state, BellMirror::FrontBack).facing,
        Direction::North
    );
    assert!(bell_use_without_item_admitted(true, false, true, false));
    assert!(!bell_use_without_item_admitted(false, false, false, false));
    assert!(!bell_use_without_item_admitted(true, true, false, true));
    assert!(bell_use_without_item_admitted(true, true, false, false));
}

#[test]
fn bell_vertical_placement_tries_only_requested_attachment() {
    let supports = BellSupports {
        below_top: true,
        above_center: true,
        ..BellSupports::default()
    };
    assert_eq!(
        bell_placement(Direction::Up, Direction::East, supports),
        Some(bell_state(Direction::East, BellAttachment::Floor))
    );
    assert_eq!(
        bell_placement(Direction::Down, Direction::West, supports),
        Some(bell_state(Direction::West, BellAttachment::Ceiling))
    );
    assert_eq!(
        bell_placement(
            Direction::Down,
            Direction::West,
            BellSupports {
                below_top: true,
                above_center: false,
                ..BellSupports::default()
            }
        ),
        None
    );
    assert_eq!(
        bell_placement(
            Direction::Down,
            Direction::North,
            BellSupports {
                above_center: true,
                above_unstable_bottom_center: true,
                ..BellSupports::default()
            }
        ),
        None
    );
}

#[test]
fn bell_wall_placement_selects_double_and_ordered_fallbacks() {
    let double = BellSupports {
        west: true,
        east: true,
        ..BellSupports::default()
    };
    assert_eq!(
        bell_placement(Direction::East, Direction::South, double),
        Some(bell_state(Direction::West, BellAttachment::DoubleWall))
    );
    let single = BellSupports {
        west: true,
        ..BellSupports::default()
    };
    assert_eq!(
        bell_placement(Direction::East, Direction::South, single),
        Some(bell_state(Direction::West, BellAttachment::SingleWall))
    );
    let floor_fallback = BellSupports {
        below_top: true,
        ..BellSupports::default()
    };
    assert_eq!(
        bell_placement(Direction::East, Direction::South, floor_fallback),
        Some(bell_state(Direction::West, BellAttachment::Floor))
    );
    let ceiling_fallback = BellSupports {
        above_center: true,
        ..BellSupports::default()
    };
    assert_eq!(
        bell_placement(Direction::East, Direction::South, ceiling_fallback),
        Some(bell_state(Direction::West, BellAttachment::Ceiling))
    );
    assert_eq!(
        bell_placement(Direction::East, Direction::South, BellSupports::default()),
        None
    );
}

#[test]
fn bell_support_updates_upgrade_downgrade_flip_and_remove() {
    let double = bell_state(Direction::North, BellAttachment::DoubleWall);
    assert_eq!(
        bell_support_update(
            double,
            Direction::North,
            false,
            BellSupports {
                south: true,
                ..BellSupports::default()
            }
        ),
        BellSupportUpdate::State(bell_state(Direction::South, BellAttachment::SingleWall))
    );

    let single = bell_state(Direction::North, BellAttachment::SingleWall);
    assert_eq!(
        bell_support_update(
            single,
            Direction::South,
            true,
            BellSupports {
                north: true,
                south: true,
                ..BellSupports::default()
            }
        ),
        BellSupportUpdate::State(bell_state(Direction::North, BellAttachment::DoubleWall))
    );
    assert_eq!(
        bell_support_update(single, Direction::North, false, BellSupports::default()),
        BellSupportUpdate::Air
    );
    assert_eq!(
        bell_support_update(single, Direction::Up, false, BellSupports::default()),
        BellSupportUpdate::Unchanged
    );
}

#[test]
fn bell_collision_and_outline_shapes_share_exact_rotated_boxes() {
    assert_eq!(
        bell_shape(bell_state(Direction::North, BellAttachment::Floor)),
        vec![Aabb16 {
            min_x: 0.0,
            min_y: 0.0,
            min_z: 4.0,
            max_x: 16.0,
            max_y: 16.0,
            max_z: 12.0,
        }]
    );
    assert_eq!(
        bell_shape(bell_state(Direction::East, BellAttachment::Floor))[0],
        Aabb16 {
            min_x: 4.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 12.0,
            max_y: 16.0,
            max_z: 16.0,
        }
    );
    let ceiling = bell_shape(bell_state(Direction::North, BellAttachment::Ceiling));
    assert_eq!(ceiling.len(), 3);
    assert_eq!((ceiling[2].min_y, ceiling[2].max_y), (13.0, 16.0));
    let north = bell_shape(bell_state(Direction::North, BellAttachment::SingleWall));
    let south = bell_shape(bell_state(Direction::South, BellAttachment::SingleWall));
    assert_eq!((north[2].min_z, north[2].max_z), (0.0, 13.0));
    assert_eq!((south[2].min_z, south[2].max_z), (3.0, 16.0));
}

#[test]
fn bell_hit_geometry_keeps_axis_and_inclusive_height_rules() {
    let floor = bell_state(Direction::North, BellAttachment::Floor);
    assert!(proper_bell_hit(floor, Direction::South, HIT_Y_LIMIT));
    assert!(!proper_bell_hit(
        floor,
        Direction::South,
        HIT_Y_LIMIT + f64::EPSILON
    ));
    assert!(!proper_bell_hit(floor, Direction::East, 0.5));
    assert!(!proper_bell_hit(floor, Direction::Up, 0.5));

    let wall = bell_state(Direction::North, BellAttachment::SingleWall);
    assert!(proper_bell_hit(wall, Direction::East, 0.5));
    assert!(!proper_bell_hit(wall, Direction::South, 0.5));
    let ceiling = bell_state(Direction::West, BellAttachment::Ceiling);
    for direction in Direction::HORIZONTAL {
        assert!(proper_bell_hit(ceiling, direction, 0.5));
    }
}

#[test]
fn bell_hit_reports_success_before_side_specific_ring_admission() {
    let state = BellState::default();
    let mut runtime = BellRuntime::default();
    let client = runtime.hit(BellHit {
        side: BellSide::Client,
        matching_block_entity: true,
        state,
        clicked_face: Direction::North,
        local_y: 0.5,
        require_correct_side: true,
        player_source: true,
    });
    assert!(client.admitted_hit);
    assert!(!client.rang && !client.award_stat);
    let missing = runtime.hit(BellHit {
        side: BellSide::Server,
        matching_block_entity: false,
        state,
        clicked_face: Direction::North,
        local_y: 0.5,
        require_correct_side: true,
        player_source: true,
    });
    assert!(missing.admitted_hit);
    assert!(!missing.rang);

    let ring = runtime.hit(BellHit {
        side: BellSide::Server,
        matching_block_entity: true,
        state,
        clicked_face: Direction::North,
        local_y: 0.5,
        require_correct_side: true,
        player_source: true,
    });
    assert!(ring.rang && ring.play_sound && ring.emit_block_change);
    assert!(ring.award_stat);
    assert_eq!(
        ring.queue.map(|event| (event.event_id, event.parameter)),
        Some((RING_EVENT_ID, 2))
    );
    let bypass = runtime.hit(BellHit {
        side: BellSide::Server,
        matching_block_entity: true,
        state,
        clicked_face: Direction::Up,
        local_y: 1.0,
        require_correct_side: false,
        player_source: false,
    });
    assert!(bypass.rang);
    assert!(!bypass.award_stat);
}

#[test]
fn bell_repeated_ingress_resets_only_an_existing_shake_clock() {
    let state = BellState::default();
    let mut runtime = BellRuntime::default();
    runtime.ticks = 17;
    runtime.attempt_to_ring(BellSide::Server, true, state, None, false);
    assert!(runtime.shaking);
    assert_eq!(runtime.ticks, 17);
    runtime.ticks = 8;
    let second =
        runtime.attempt_to_ring(BellSide::Server, true, state, Some(Direction::East), true);
    assert_eq!(runtime.ticks, 0);
    assert_eq!(runtime.click_direction, Some(Direction::East));
    assert!(second.award_stat);
}

#[test]
fn bell_neighbour_edges_and_explosions_order_ring_before_state_write() {
    let mut runtime = BellRuntime::default();
    let state = BellState::default();
    let stable = bell_neighbour_signal(&mut runtime, state, false, true);
    assert!(!stable.write_state);
    let rising = bell_neighbour_signal(&mut runtime, state, true, true);
    assert!(rising.write_state);
    assert_eq!(rising.update_flags, Some(3));
    assert!(rising.ring.expect("rising ring").rang);
    assert!(rising.state.powered);
    let falling = bell_neighbour_signal(&mut runtime, rising.state, false, true);
    assert!(falling.write_state);
    assert!(falling.ring.is_none());
    assert!(!falling.state.powered);
    assert!(bell_explosion(&mut runtime, state, false, true).is_none());
    assert!(
        bell_explosion(&mut runtime, state, true, true)
            .expect("trigger-capable")
            .rang
    );
}

#[test]
fn queued_bell_event_refreshes_strictly_after_60_and_hears_all_living() {
    let initial = [
        living(1, [0.5, 0.5, 0.5], true, false, false),
        living(2, [32.5, 0.5, 0.5], true, false, true),
        living(3, [1.0, 0.5, 0.5], false, false, true),
        living(4, [1.0, 0.5, 0.5], true, true, true),
    ];
    let mut runtime = BellRuntime::default();
    let first = runtime.trigger_event(1, -1, 0, BellSide::Server, &initial);
    assert!(first.refreshed_cache);
    assert_eq!(first.heard_entity_ids, vec![1]);
    assert_eq!(runtime.click_direction, Some(Direction::East));
    assert_eq!(runtime.cached_entity_ids(), Some(vec![1, 2, 3, 4]));

    let replacement = [near_raider(9)];
    let equality = runtime.trigger_event(1, 4, CACHE_INTERVAL, BellSide::Server, &replacement);
    assert!(!equality.refreshed_cache);
    assert_eq!(runtime.cached_entity_ids(), Some(vec![1, 2, 3, 4]));
    let stale = runtime.trigger_event(1, 4, CACHE_INTERVAL + 1, BellSide::Client, &replacement);
    assert!(stale.refreshed_cache);
    assert!(stale.heard_entity_ids.is_empty());
    assert_eq!(runtime.cached_entity_ids(), Some(vec![9]));
    assert!(
        !runtime
            .trigger_event(2, 0, 100, BellSide::Server, &[])
            .handled
    );
}

#[test]
fn bell_shake_retries_for_late_raider_and_resonance_ends_after_clock_40() {
    let ordinary = [living(1, [1.0, 0.5, 0.5], true, false, false)];
    let raider = [near_raider(1)];
    let mut runtime = BellRuntime::default();
    runtime.trigger_event(1, 2, 0, BellSide::Server, &ordinary);
    for _ in 0..5 {
        let tick = runtime.tick(BellSide::Server, &ordinary);
        assert!(!tick.resonance_sound_call);
    }
    let started = runtime.tick(BellSide::Server, &raider);
    assert!(started.resonance_sound_call);
    assert!(started.audible_resonance_sound);
    assert_eq!(runtime.resonance_ticks, 1);
    for _ in 1..RESONANCE_TICKS {
        runtime.tick(BellSide::Server, &raider);
    }
    assert!(runtime.resonating);
    let ended = runtime.tick(BellSide::Server, &raider);
    assert!(!runtime.resonating);
    assert_eq!(ended.glowing_entity_ids, vec![1]);
    assert_eq!(GLOW_TICKS, 60);
}

#[test]
fn bell_rerun_during_resonance_restarts_clock_without_second_sound() {
    let raider = [near_raider(1)];
    let mut runtime = BellRuntime::default();
    runtime.trigger_event(1, 2, 0, BellSide::Server, &raider);
    for _ in 0..5 {
        runtime.tick(BellSide::Server, &raider);
    }
    assert!(runtime.resonating);
    runtime.trigger_event(1, 3, 1, BellSide::Server, &raider);
    assert!(runtime.resonating);
    assert_eq!(runtime.resonance_ticks, 0);
    for expected in 1..=4 {
        let tick = runtime.tick(BellSide::Server, &raider);
        assert!(!tick.resonance_sound_call);
        assert_eq!(runtime.resonance_ticks, expected);
    }
    let fifth = runtime.tick(BellSide::Server, &raider);
    assert!(!fifth.resonance_sound_call);
    assert_eq!(runtime.resonance_ticks, 5);
}

#[test]
fn bell_server_glow_uses_current_liveness_tag_and_strict_radius_48() {
    let initial = [
        near_raider(1),
        near_raider(2),
        near_raider(3),
        near_raider(4),
    ];
    let current = [
        living(1, [48.5, 0.5, 0.5], true, false, true),
        living(2, [47.0, 0.5, 0.5], true, false, true),
        living(3, [1.0, 0.5, 0.5], true, true, true),
        living(4, [1.0, 0.5, 0.5], true, false, false),
    ];
    let mut runtime = BellRuntime::default();
    runtime.trigger_event(1, 2, 0, BellSide::Server, &initial);
    for _ in 0..5 {
        runtime.tick(BellSide::Server, &initial);
    }
    for _ in 1..=RESONANCE_TICKS {
        let result = runtime.tick(BellSide::Server, &current);
        if !result.glowing_entity_ids.is_empty() {
            assert_eq!(result.glowing_entity_ids, vec![2]);
            return;
        }
    }
    panic!("resonance did not reach the glow transaction");
}

#[test]
fn bell_client_particles_count_all_cached_living_but_emit_for_raiders() {
    let entities = [
        living(1, [1.0, 0.5, 0.5], true, false, true),
        living(2, [0.0, 0.5, 0.0], true, false, true),
        living(3, [2.0, 0.5, 0.5], false, true, false),
    ];
    let mut runtime = BellRuntime::default();
    runtime.trigger_event(1, 2, 0, BellSide::Client, &entities);
    for _ in 0..5 {
        runtime.tick(BellSide::Client, &entities);
    }
    let mut ended = None;
    for _ in 1..=RESONANCE_TICKS {
        let result = runtime.tick(BellSide::Client, &entities);
        if !result.particles.is_empty() {
            ended = Some(result);
            break;
        }
    }
    let particles = ended.expect("client resonance end").particles;
    assert_eq!(particles.len(), 18);
    assert_eq!(particles[0].entity_id, 1);
    assert_eq!(particles[0].color, 16_700_990);
    assert_eq!(particles[8].color, 16_701_030);
    assert_eq!(particles[9].entity_id, 2);
    assert_eq!(particles[17].color, 16_701_075);
    assert!(particles[9].position[0].is_nan());
    assert!(particles[9].position[2].is_nan());
}

#[test]
fn bell_renderer_axes_and_transient_loot_are_exact() {
    let mut runtime = BellRuntime::default();
    runtime.ticks = 5;
    runtime.shaking = true;
    runtime.click_direction = Some(Direction::North);
    let north = bell_render_rotation(&runtime, 0.5);
    assert!(north[0] < 0.0);
    assert_eq!(north[1], 0.0);
    runtime.click_direction = Some(Direction::East);
    let east = bell_render_rotation(&runtime, 0.5);
    assert_eq!(east[0], 0.0);
    assert!(east[1] > 0.0);
    runtime.shaking = false;
    assert_eq!(bell_render_rotation(&runtime, 0.5), [0.0, 0.0]);
    assert!(bell_loot_survives_explosion(true));
    assert!(!bell_loot_survives_explosion(false));
}

#[test]
fn enchanting_table_registry_shape_and_menu_boundary_are_exact() {
    assert_eq!(
        (
            TABLE_BLOCK_ID,
            TABLE_ITEM_ID,
            TABLE_BLOCK_ENTITY_ID,
            TABLE_STATE_ID,
        ),
        (385, 461, 13, 9_451)
    );
    assert_eq!(
        (
            MENU_PROTOCOL_ID,
            ENCHANT_PARTICLE_ID,
            LIGHT_LEVEL,
            SHAPE_HEIGHT
        ),
        (13, 26, 7, 12)
    );
    assert_eq!(
        (
            TABLE_HARDNESS,
            TABLE_RESISTANCE,
            REQUIRES_CORRECT_TOOL,
            USES_SHAPE_FOR_LIGHT_OCCLUSION,
        ),
        (5.0, 1_200.0, true, true)
    );
    assert!(enchanting_use_without_item_admitted(
        true, false, true, false
    ));
    assert!(!enchanting_use_without_item_admitted(
        false, false, false, false
    ));
    assert!(!enchanting_use_without_item_admitted(
        true, true, true, false
    ));
    let named = EnchantingTableData {
        custom_name: Some("Arcana".into()),
    };
    let client = enchanting_table_use(TableSide::Client, true, Some(&named));
    assert!(client.success);
    assert!(!client.opens_menu);
    let missing = enchanting_table_use(TableSide::Server, false, None);
    assert!(missing.success);
    assert!(!missing.opens_menu);
    let server = enchanting_table_use(TableSide::Server, true, Some(&named));
    assert!(server.opens_menu && server.creates_level_access);
    assert_eq!(server.title.as_deref(), Some("Arcana"));
    assert_eq!(server.menu_protocol_id, Some(MENU_PROTOCOL_ID));
}

#[test]
fn enchanting_table_name_is_the_only_persistent_and_loot_component() {
    let missing = EnchantingTableData::load(StoredCustomName::Missing);
    let malformed = EnchantingTableData::load(StoredCustomName::Malformed);
    assert_eq!(missing.display_name(), DEFAULT_TITLE);
    assert_eq!(malformed.display_name(), DEFAULT_TITLE);
    let mut named = EnchantingTableData::load(StoredCustomName::Valid("Runes".into()));
    assert_eq!(named.saved_custom_name(), Some("Runes"));
    assert_eq!(named.collected_custom_name_component(), Some("Runes"));
    named.apply_custom_name_component(Some("Glyphs".into()));
    assert_eq!(
        enchanting_table_loot(true, Some(&named)),
        Some(
            ferrite_gameplay::block::enchanting_table::EnchantingTableDrop {
                custom_name: Some("Glyphs".into())
            }
        )
    );
    assert_eq!(enchanting_table_loot(false, Some(&named)), None);
    assert_eq!(enchanting_table_pick().custom_name, None);
}

#[test]
fn bookshelf_offsets_lock_between_closed_stream_order_and_midpoints() {
    let offsets = bookshelf_offsets();
    assert_eq!(offsets.len(), BOOKSHELF_PROBE_COUNT);
    assert_eq!(offsets[0], [-2, 0, -2]);
    assert_eq!(offsets[4], [2, 0, -2]);
    assert_eq!(offsets[5], [-2, 1, -2]);
    assert_eq!(offsets[9], [2, 1, -2]);
    assert_eq!(offsets[10], [-2, 0, -1]);
    assert_eq!(offsets[31], [2, 1, 2]);
    let unique = offsets.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 32);
    assert!(
        unique
            .iter()
            .all(|offset| offset[0].abs() == 2 || offset[2].abs() == 2)
    );
    assert_eq!([-1 / 2, 1, -1 / 2], [0, 1, 0]);
}

#[test]
fn bookshelf_scan_consumes_one_bound_before_validity_and_three_hit_floats() {
    let mut probes = [BookshelfProbe::INVALID; BOOKSHELF_PROBE_COUNT];
    probes[0] = BookshelfProbe {
        provider: true,
        transmitter: true,
    };
    probes[1] = BookshelfProbe {
        provider: true,
        transmitter: false,
    };
    let mut ints = vec![1; BOOKSHELF_PROBE_COUNT];
    ints[0] = 0;
    ints[1] = 0;
    let mut random = RandomScript::new(ints, [0.25, 0.5, 0.75]);
    let mut queried = Vec::new();
    let particles = enchanting_particle_scan(&mut random, |index, offset| {
        queried.push((index, offset));
        probes[index]
    });
    assert_eq!(random.bounds, vec![16; BOOKSHELF_PROBE_COUNT]);
    assert_eq!(queried, vec![(0, [-2, 0, -2]), (1, [-1, 0, -2])]);
    assert_eq!(random.float_draws, 3);
    assert_eq!(particles.len(), 1);
    assert_eq!(particles[0].offset, [-2, 0, -2]);
    assert_eq!(particles[0].position, [0.5, 2.0, 0.5]);
    assert_eq!(particles[0].velocity, [-2.25, -1.5, -1.75]);
}

#[test]
fn bookshelf_roll_misses_and_invalid_hits_consume_no_floats() {
    let probes = [BookshelfProbe::INVALID; BOOKSHELF_PROBE_COUNT];
    let mut random = RandomScript::new(vec![0; BOOKSHELF_PROBE_COUNT], []);
    assert!(enchanting_particles(&probes, &mut random).is_empty());
    assert_eq!(random.bounds.len(), 32);
    assert_eq!(random.float_draws, 0);

    let probes = [BookshelfProbe {
        provider: true,
        transmitter: true,
    }; BOOKSHELF_PROBE_COUNT];
    let mut random = RandomScript::new(vec![15; BOOKSHELF_PROBE_COUNT], []);
    assert!(enchanting_particles(&probes, &mut random).is_empty());
    assert_eq!(random.float_draws, 0);
}

#[test]
fn book_tick_selects_first_nearest_nonspectator_at_strict_radius_three() {
    let players = [
        ClientPlayer {
            id: 1,
            position: [3.5, 0.5, 0.5],
            spectator: false,
        },
        ClientPlayer {
            id: 2,
            position: [1.5, 0.5, 0.5],
            spectator: true,
        },
        ClientPlayer {
            id: 3,
            position: [1.5, 0.5, 0.5],
            spectator: false,
        },
        ClientPlayer {
            id: 4,
            position: [-0.5, 0.5, 0.5],
            spectator: false,
        },
    ];
    let mut animation = BookAnimation::default();
    let mut random = RandomScript::new([3, 1], []);
    let outcome = animation.tick([0, 0, 0], &players, &mut random);
    assert_eq!(outcome.nearest_player, Some(3));
    assert!(outcome.page_selected);
    assert!(!outcome.chance_draw_consumed);
    assert_eq!(animation.target_rotation, 0.0);
    assert_eq!(animation.open, 0.1);
}

#[test]
fn book_open_threshold_controls_page_chance_draw_order() {
    let player = [ClientPlayer {
        id: 1,
        position: [1.5, 0.5, 0.5],
        spectator: false,
    }];
    let mut below = BookAnimation {
        open: 0.39,
        ..BookAnimation::default()
    };
    let mut forced_random = RandomScript::new([2, 0], []);
    let forced = below.tick([0, 0, 0], &player, &mut forced_random);
    assert!(forced.page_selected);
    assert!(!forced.chance_draw_consumed);
    assert_eq!(forced_random.bounds, vec![4, 4]);

    let mut threshold = BookAnimation {
        open: 0.4,
        ..BookAnimation::default()
    };
    let mut miss_random = RandomScript::new([39], []);
    let miss = threshold.tick([0, 0, 0], &player, &mut miss_random);
    assert!(!miss.page_selected);
    assert!(miss.chance_draw_consumed);
    assert_eq!(miss_random.bounds, vec![40]);

    let mut chance = BookAnimation {
        open: 0.5,
        ..BookAnimation::default()
    };
    let mut hit_random = RandomScript::new([0, 3, 1], []);
    let hit = chance.tick([0, 0, 0], &player, &mut hit_random);
    assert!(hit.page_selected && hit.chance_draw_consumed);
    assert_eq!(hit_random.bounds, vec![40, 4, 4]);
}

#[test]
fn book_page_selection_repeats_pairs_until_target_changes() {
    let player = [ClientPlayer {
        id: 1,
        position: [1.0, 0.5, 0.5],
        spectator: false,
    }];
    let mut animation = BookAnimation::default();
    let mut random = RandomScript::new([1, 1, 2, 2, 3, 1], []);
    let outcome = animation.tick([0, 0, 0], &player, &mut random);
    assert!(outcome.page_selected);
    assert_eq!(animation.target_flip, 2.0);
    assert_eq!(random.bounds, vec![4, 4, 4, 4, 4, 4]);
}

#[test]
fn book_idle_tick_uses_no_rng_and_clamps_open() {
    let mut animation = BookAnimation {
        open: 0.05,
        target_rotation: std::f32::consts::PI - 0.01,
        ..BookAnimation::default()
    };
    let mut random = RandomScript::default();
    let outcome = animation.tick([0, 0, 0], &[], &mut random);
    assert_eq!(outcome.nearest_player, None);
    assert!(!outcome.page_selected && !outcome.chance_draw_consumed);
    assert_eq!(animation.open, 0.0);
    assert!(animation.target_rotation < -std::f32::consts::PI + 0.02);
    assert!(random.bounds.is_empty());
}

#[test]
fn book_rotation_flip_clamps_and_signed_time_follow_java_order() {
    let mut animation = BookAnimation {
        time: i32::MAX,
        flip: -10.0,
        target_flip: 10.0,
        rotation: std::f32::consts::PI + 0.2,
        target_rotation: -std::f32::consts::PI - 0.2,
        ..BookAnimation::default()
    };
    let mut random = RandomScript::default();
    animation.tick([0, 0, 0], &[], &mut random);
    assert_eq!(animation.time, i32::MIN);
    assert_eq!(animation.previous_flip, -10.0);
    assert!((animation.flip_acceleration - 0.18).abs() < 1.0e-6);
    assert!((animation.flip - -9.82).abs() < 1.0e-5);
    assert!(animation.rotation >= -std::f32::consts::PI);
    assert!(animation.rotation < std::f32::consts::PI);
}

#[test]
fn enchanting_renderer_interpolates_wraps_and_uses_floor_fraction() {
    let animation = BookAnimation {
        time: 9,
        flip: -0.25,
        previous_flip: -0.75,
        open: 1.0,
        previous_open: 0.0,
        rotation: -std::f32::consts::PI + 0.1,
        previous_rotation: std::f32::consts::PI - 0.1,
        ..BookAnimation::default()
    };
    let render = enchanting_table_render(animation, 0.5);
    assert_eq!(render.flip, -0.5);
    assert_eq!(render.open, 0.5);
    assert_eq!(render.time, 9.5);
    assert!((render.yaw - std::f32::consts::PI).abs() < 1.0e-5);
    assert_eq!(render.translation[0], 0.5);
    assert!((0.84..=0.86).contains(&render.translation[1]));
    assert_eq!(render.z_rotation_degrees, 80.0);
    assert!((render.left_page - 0.9).abs() < 1.0e-6);
    assert!((render.right_page - 0.1).abs() < 1.0e-6);
}

#[test]
fn caller_owned_page_stream_couples_tables_and_reload_resets_only_animation() {
    let player = [ClientPlayer {
        id: 1,
        position: [1.0, 0.5, 0.5],
        spectator: false,
    }];
    let mut shared = RandomScript::new([3, 1, 0, 2], []);
    let mut first = BookAnimation::default();
    let mut second = BookAnimation::default();
    first.tick([0, 0, 0], &player, &mut shared);
    second.tick([0, 0, 0], &player, &mut shared);
    assert_eq!(first.target_flip, 2.0);
    assert_eq!(second.target_flip, -2.0);

    let named = EnchantingTableData {
        custom_name: Some("Persistent".into()),
    };
    let reloaded_animation = BookAnimation::default();
    assert_eq!(reloaded_animation, BookAnimation::default());
    assert_eq!(named.display_name(), "Persistent");
}
