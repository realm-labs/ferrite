use ferrite_foundation::direction::Direction;
use ferrite_gameplay::environment::fluid::{
    BASE_FIRE_STATE_ID, BLOCK_UPDATE_FLAGS, BUBBLE_COLUMN_CHECK_DELAY, COBBLESTONE_STATE_ID,
    ContainerKind, FluidDestination, FluidFamily, FluidParameters, FluidRandom, FluidSpreadRequest,
    FluidState, FluidTickWrite, HorizontalFireProbe, HorizontalFluid, LAVA_BLOCK_ID,
    LAVA_FIRST_STATE_ID, LAVA_SOURCE_CONVERSION_DEFAULT, LEVEL_TICK_ORDER, LIQUID_MIX_EVENT,
    LavaFirePlan, LavaMixNeighbour, LavaMixProduct, LevelTickQueue, LocalFluidInputs,
    OBSIDIAN_STATE_ID, RisingFireProbe, SCHEDULED_TICK_CAP, SIMPLE_WATERLOGGED_BLOCK_COUNT,
    STONE_STATE_ID, SpreadCandidate, WATER_BLOCK_ID, WATER_FIRST_STATE_ID,
    WATER_SOURCE_CONVERSION_DEFAULT, destination_admits, downward_lava_into_water,
    explicitly_unholdable, fluid_from_legacy, fluid_parameters, fluid_spread_plan, fluid_tick_plan,
    lava_mix_before_schedule, lava_random_fire, place_simple_waterlogged, recompute_local_fluid,
    shape_update_schedules, spread_commit_plan, spread_delay, water_evaporation,
};
use ferrite_gameplay::environment::geyser::{
    ACTIVE_SOUND_FREQUENCY, BLOCK_ENTITY_PROTOCOL_ID, BLOCK_ID, CLOUD_FREQUENCY,
    COLUMN_PROBE_COUNT, CONTINUOUS_ACTIVE_SOUND_ID, CONTINUOUS_START_SOUND_ID, ColumnCell,
    CountdownOutcome, ENDER_DRAGON_ENTITY_TYPE_ID, ERUPTION_ACTIVE_SOUND_ID,
    ERUPTION_START_SOUND_ID, FIRST_STATE_ID, GEYSER_PARTICLE_ID, GEYSER_SALT, GasEntity, GasSource,
    GeyserGameEvent, GeyserRuntime, GeyserSupport, GeyserTickStage, HARDNESS, ITEM_ID,
    LAUNCH_FORCE, LAUNCH_HEIGHT_MULTIPLIER, LaunchEntity, MAX_WATER_BLOCKS, NAUSEA_DURATION,
    NOXIOUS_GAS_CLOUD_PARTICLE_ID, NOXIOUS_GAS_SOUND_ID, PLUME_FREQUENCY, PotentSulfurState,
    REQUIRES_CORRECT_TOOL, RESISTANCE, STATE_COUNT, SULFUR_BUBBLE_PARTICLE_ID, SupportFluid,
    client_gas_cloud, client_plume, countdown_tick, derive_potent_sulfur, find_gas_source,
    geyser_display_tick, geyser_launch, geyser_on_place, geyser_tick_stages, launch_query_expand_y,
    nausea_applications, potent_sulfur_loot, unobstructed_count,
};

#[derive(Debug, Default)]
struct RandomScript {
    ints: Vec<u32>,
    unbounded: Vec<i32>,
    floats: Vec<f32>,
    int_index: usize,
    unbounded_index: usize,
    float_index: usize,
    bounds: Vec<u32>,
}

impl RandomScript {
    fn new(ints: Vec<u32>, unbounded: Vec<i32>, floats: Vec<f32>) -> Self {
        Self {
            ints,
            unbounded,
            floats,
            ..Self::default()
        }
    }
}

impl FluidRandom for RandomScript {
    fn next_int(&mut self, bound: u32) -> u32 {
        self.bounds.push(bound);
        let value = self.ints[self.int_index];
        self.int_index += 1;
        assert!(value < bound);
        value
    }

    fn next_unbounded_int(&mut self) -> i32 {
        let value = self.unbounded[self.unbounded_index];
        self.unbounded_index += 1;
        value
    }

    fn next_float(&mut self) -> f32 {
        let value = self.floats[self.float_index];
        self.float_index += 1;
        value
    }
}

fn horizontal(state: FluidState, face_passes: bool) -> HorizontalFluid {
    HorizontalFluid { state, face_passes }
}

fn candidates(distances: [Option<u8>; 4]) -> [SpreadCandidate; 4] {
    [
        SpreadCandidate {
            direction: Direction::North,
            admitted: distances[0].is_some(),
            hole_distance: distances[0],
        },
        SpreadCandidate {
            direction: Direction::East,
            admitted: distances[1].is_some(),
            hole_distance: distances[1],
        },
        SpreadCandidate {
            direction: Direction::South,
            admitted: distances[2].is_some(),
            hole_distance: distances[2],
        },
        SpreadCandidate {
            direction: Direction::West,
            admitted: distances[3].is_some(),
            hole_distance: distances[3],
        },
    ]
}

fn passable_air() -> ColumnCell {
    ColumnCell {
        source_water: false,
        water_block: false,
        air: true,
        empty_collision: true,
    }
}

fn source_water() -> ColumnCell {
    ColumnCell {
        source_water: true,
        water_block: true,
        air: false,
        empty_collision: false,
    }
}

#[test]
fn fluid_registry_legacy_encoding_and_heights_are_exact() {
    assert_eq!((WATER_BLOCK_ID, LAVA_BLOCK_ID), (35, 36));
    assert_eq!((WATER_FIRST_STATE_ID, LAVA_FIRST_STATE_ID), (86, 102));
    assert_eq!(FluidState::Empty.protocol_id(), 0);
    assert_eq!(
        FluidState::Flowing {
            family: FluidFamily::Water,
            amount: 7,
            falling: false
        }
        .protocol_id(),
        1
    );
    assert_eq!(FluidState::Source(FluidFamily::Water).protocol_id(), 2);
    assert_eq!(
        FluidState::Flowing {
            family: FluidFamily::Lava,
            amount: 8,
            falling: true
        }
        .protocol_id(),
        3
    );
    assert_eq!(FluidState::Source(FluidFamily::Lava).protocol_id(), 4);

    for family in [FluidFamily::Water, FluidFamily::Lava] {
        assert_eq!(fluid_from_legacy(family, 0), FluidState::Source(family));
        for level in 1..=7 {
            assert_eq!(
                fluid_from_legacy(family, level),
                FluidState::Flowing {
                    family,
                    amount: 8 - level,
                    falling: false
                }
            );
        }
        for level in 8..=15 {
            assert_eq!(
                fluid_from_legacy(family, level),
                FluidState::Flowing {
                    family,
                    amount: 8,
                    falling: true
                }
            );
        }
    }
    let source = FluidState::Source(FluidFamily::Water);
    assert!((source.own_height(false) - 8.0 / 9.0).abs() < f32::EPSILON);
    assert_eq!(source.own_height(true), 1.0);
    assert_eq!(
        fluid_from_legacy(FluidFamily::Water, 15).block_state_id(),
        Some(94)
    );
}

#[test]
fn water_and_fast_lava_parameters_lock_drop_range_delay_and_caps() {
    assert_eq!(
        fluid_parameters(FluidFamily::Water, false),
        FluidParameters {
            drop_off: 1,
            slope_range: 4,
            tick_delay: 5
        }
    );
    assert_eq!(
        fluid_parameters(FluidFamily::Lava, false),
        FluidParameters {
            drop_off: 2,
            slope_range: 2,
            tick_delay: 30
        }
    );
    assert_eq!(
        fluid_parameters(FluidFamily::Lava, true),
        FluidParameters {
            drop_off: 1,
            slope_range: 4,
            tick_delay: 10
        }
    );
    assert_eq!(
        (
            SCHEDULED_TICK_CAP,
            BUBBLE_COLUMN_CHECK_DELAY,
            SIMPLE_WATERLOGGED_BLOCK_COUNT
        ),
        (65_536, 20, 429)
    );
    assert_eq!(
        (
            WATER_SOURCE_CONVERSION_DEFAULT,
            LAVA_SOURCE_CONVERSION_DEFAULT,
            LEVEL_TICK_ORDER
        ),
        (true, false, [LevelTickQueue::Block, LevelTickQueue::Fluid])
    );
}

#[test]
fn local_fluid_recomputation_scans_admitted_faces_and_source_rules() {
    let source = FluidState::Source(FluidFamily::Water);
    let flowing = FluidState::Flowing {
        family: FluidFamily::Water,
        amount: 6,
        falling: false,
    };
    let base = LocalFluidInputs {
        family: FluidFamily::Water,
        horizontal: [
            horizontal(source, true),
            horizontal(source, true),
            horizontal(flowing, true),
            horizontal(source, false),
        ],
        same_family_above_admitted: false,
        below_solid: true,
        below_same_family_source: false,
        source_conversion: true,
        drop_off: 1,
    };
    assert_eq!(recompute_local_fluid(base), source);
    assert_eq!(
        recompute_local_fluid(LocalFluidInputs {
            below_solid: false,
            source_conversion: true,
            ..base
        }),
        FluidState::Flowing {
            family: FluidFamily::Water,
            amount: 7,
            falling: false
        }
    );
    assert_eq!(
        recompute_local_fluid(LocalFluidInputs {
            same_family_above_admitted: true,
            source_conversion: false,
            ..base
        }),
        FluidState::Flowing {
            family: FluidFamily::Water,
            amount: 8,
            falling: true
        }
    );
    assert_eq!(
        recompute_local_fluid(LocalFluidInputs {
            horizontal: [horizontal(FluidState::Empty, true); 4],
            below_solid: false,
            ..base
        }),
        FluidState::Empty
    );
}

#[test]
fn lava_spread_delay_consumes_only_the_rising_nonfalling_draw() {
    let old = FluidState::Flowing {
        family: FluidFamily::Lava,
        amount: 3,
        falling: false,
    };
    let rising = FluidState::Flowing {
        family: FluidFamily::Lava,
        amount: 4,
        falling: false,
    };
    assert_eq!(spread_delay(old, rising, false, 0).delay, 30);
    let slowed = spread_delay(old, rising, false, 3);
    assert!(slowed.slowdown_draw_consumed);
    assert_eq!(slowed.delay, 120);
    assert_eq!(spread_delay(old, rising, true, 1).delay, 40);
    let falling = FluidState::Flowing {
        family: FluidFamily::Lava,
        amount: 8,
        falling: true,
    };
    assert!(!spread_delay(old, falling, false, 3).slowdown_draw_consumed);
    assert_eq!(spread_delay(old, falling, false, 3).delay, 30);
}

#[test]
fn replacement_asymmetry_and_container_admission_are_exact() {
    let water = FluidState::Source(FluidFamily::Water);
    let lava_deep = FluidState::Flowing {
        family: FluidFamily::Lava,
        amount: 4,
        falling: false,
    };
    let lava_shallow = FluidState::Flowing {
        family: FluidFamily::Lava,
        amount: 3,
        falling: false,
    };
    let ordinary = |fluid| FluidDestination {
        current_fluid: fluid,
        generally_holdable: true,
        joined_face_passes: true,
        container: ContainerKind::None,
    };
    assert!(destination_admits(
        ordinary(lava_deep),
        water,
        Direction::East
    ));
    assert!(!destination_admits(
        ordinary(lava_shallow),
        water,
        Direction::East
    ));
    assert!(destination_admits(
        ordinary(water),
        FluidState::Source(FluidFamily::Lava),
        Direction::Down
    ));
    assert!(!destination_admits(
        ordinary(water),
        FluidState::Source(FluidFamily::Lava),
        Direction::North
    ));
    assert!(!destination_admits(
        FluidDestination {
            generally_holdable: false,
            ..ordinary(FluidState::Empty)
        },
        water,
        Direction::Down
    ));
    assert!(!destination_admits(
        FluidDestination {
            container: ContainerKind::IntrinsicAquatic,
            ..ordinary(FluidState::Empty)
        },
        water,
        Direction::Down
    ));
}

#[test]
fn simple_waterlogging_accepts_exact_source_and_client_skips_write() {
    let water = FluidState::Source(FluidFamily::Water);
    let client = place_simple_waterlogged(true, false, water);
    assert!(client.accepted);
    assert!(!client.write_waterlogged);
    assert_eq!(client.schedule_delay, None);
    let server = place_simple_waterlogged(false, false, water);
    assert!(server.write_waterlogged);
    assert_eq!(server.write_flags, Some(BLOCK_UPDATE_FLAGS));
    assert_eq!(server.schedule_delay, Some(5));
    assert!(
        !place_simple_waterlogged(
            false,
            false,
            FluidState::Flowing {
                family: FluidFamily::Water,
                amount: 8,
                falling: true
            }
        )
        .accepted
    );
    assert!(!place_simple_waterlogged(false, true, water).accepted);
}

#[test]
fn explicit_holdability_and_commit_hooks_preserve_displacement_order() {
    for path in [
        "oak_door",
        "oak_sign",
        "oak_wall_sign",
        "oak_hanging_sign",
        "ladder",
        "sugar_cane",
        "bubble_column",
        "nether_portal",
        "end_portal",
        "end_gateway",
        "structure_void",
    ] {
        assert!(explicitly_unholdable(path), "{path}");
    }
    assert!(!explicitly_unholdable("stone"));
    let water = spread_commit_plan(FluidFamily::Water, false, true);
    assert!(water.drop_target_resources);
    assert!(!water.fizz_before_write);
    assert!(water.write_legacy_liquid);
    assert_eq!(water.write_flags, Some(3));
    let lava = spread_commit_plan(FluidFamily::Lava, false, true);
    assert!(!lava.drop_target_resources);
    assert!(lava.fizz_before_write);
    let container = spread_commit_plan(FluidFamily::Water, true, true);
    assert!(container.call_container_place && container.ignore_container_result);
    assert!(!container.write_legacy_liquid);
    assert!(shape_update_schedules(true, false));
    assert!(shape_update_schedules(false, true));
    assert!(!shape_update_schedules(false, false));
}

#[test]
fn downward_spread_precedes_tied_horizontal_commit_order() {
    let source = FluidState::Source(FluidFamily::Water);
    let downward = FluidState::Flowing {
        family: FluidFamily::Water,
        amount: 8,
        falling: true,
    };
    let plan = fluid_spread_plan(FluidSpreadRequest {
        origin: source,
        drop_off: 1,
        downward_admitted: true,
        downward_state: downward,
        below_is_open_hole: true,
        horizontal_source_neighbours: 3,
        candidates: candidates([Some(2), Some(2), Some(2), Some(2)]),
    });
    assert_eq!(plan.downward, Some(downward));
    assert_eq!(
        plan.horizontal_state,
        Some(FluidState::Flowing {
            family: FluidFamily::Water,
            amount: 7,
            falling: false
        })
    );
    assert_eq!(
        plan.horizontal_directions,
        [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East
        ]
    );
    let no_sides = fluid_spread_plan(FluidSpreadRequest {
        horizontal_source_neighbours: 2,
        ..FluidSpreadRequest {
            origin: source,
            drop_off: 1,
            downward_admitted: true,
            downward_state: downward,
            below_is_open_hole: true,
            horizontal_source_neighbours: 3,
            candidates: candidates([Some(0); 4]),
        }
    });
    assert!(no_sides.horizontal_directions.is_empty());
}

#[test]
fn nonsource_holes_stop_sides_and_falling_columns_emit_level_seven() {
    let flowing = FluidState::Flowing {
        family: FluidFamily::Water,
        amount: 5,
        falling: false,
    };
    let stopped = fluid_spread_plan(FluidSpreadRequest {
        origin: flowing,
        drop_off: 1,
        downward_admitted: false,
        downward_state: FluidState::Empty,
        below_is_open_hole: true,
        horizontal_source_neighbours: 0,
        candidates: candidates([Some(0); 4]),
    });
    assert!(stopped.horizontal_state.is_none());
    let falling = fluid_spread_plan(FluidSpreadRequest {
        origin: FluidState::Flowing {
            family: FluidFamily::Water,
            amount: 8,
            falling: true,
        },
        below_is_open_hole: false,
        ..FluidSpreadRequest {
            origin: flowing,
            drop_off: 1,
            downward_admitted: false,
            downward_state: FluidState::Empty,
            below_is_open_hole: true,
            horizontal_source_neighbours: 0,
            candidates: candidates([Some(3), Some(1), Some(1), None]),
        }
    });
    assert_eq!(
        falling.horizontal_state,
        Some(FluidState::Flowing {
            family: FluidFamily::Water,
            amount: 7,
            falling: false
        })
    );
    assert_eq!(
        falling.horizontal_directions,
        [Direction::South, Direction::East]
    );
}

#[test]
fn due_fluid_tick_writes_then_spreads_result_and_sources_skip_recompute() {
    let source = FluidState::Source(FluidFamily::Water);
    assert_eq!(
        fluid_tick_plan(source, FluidState::Empty, 5).spread_state,
        source
    );
    let old = FluidState::Flowing {
        family: FluidFamily::Water,
        amount: 4,
        falling: false,
    };
    assert_eq!(
        fluid_tick_plan(old, FluidState::Empty, 5).write,
        FluidTickWrite::Air { flags: 3 }
    );
    let new = FluidState::Flowing {
        family: FluidFamily::Water,
        amount: 3,
        falling: false,
    };
    let changed = fluid_tick_plan(old, new, 5);
    assert_eq!(
        changed.write,
        FluidTickWrite::Fluid {
            state: new,
            flags: 3,
            schedule_delay: 5
        }
    );
    assert_eq!(changed.spread_state, new);
    assert_eq!(fluid_tick_plan(old, old, 5).write, FluidTickWrite::None);
}

#[test]
fn lava_mixing_scans_up_north_south_west_east_and_aborts_schedule() {
    let none = LavaMixNeighbour {
        water_tagged: false,
        blue_ice: false,
    };
    let water = LavaMixNeighbour {
        water_tagged: true,
        blue_ice: false,
    };
    let blue = LavaMixNeighbour {
        water_tagged: false,
        blue_ice: true,
    };
    let source =
        lava_mix_before_schedule(true, true, [none, water, blue, none, none]).expect("north water");
    assert_eq!(source.product, LavaMixProduct::Obsidian);
    assert_eq!(source.product.state_id(), OBSIDIAN_STATE_ID);
    assert_eq!(source.neighbour, Direction::North);
    assert_eq!(source.level_event, LIQUID_MIX_EVENT);
    assert!(source.abort_schedule);

    let flowing = lava_mix_before_schedule(false, false, [none, none, water, none, none])
        .expect("south water");
    assert_eq!(flowing.product, LavaMixProduct::Cobblestone);
    assert_eq!(flowing.product.state_id(), COBBLESTONE_STATE_ID);
    let basalt =
        lava_mix_before_schedule(false, true, [none, blue, none, none, none]).expect("blue ice");
    assert_eq!(basalt.product, LavaMixProduct::Basalt);
    assert_eq!(basalt.product.state_id(), 7_001);
    assert!(lava_mix_before_schedule(false, false, [none; 5]).is_none());
}

#[test]
fn downward_lava_water_always_fizzes_but_stone_requires_liquid_block() {
    let plain = downward_lava_into_water(true);
    assert!(plain.write_stone);
    assert_eq!(plain.stone_state_id, Some(STONE_STATE_ID));
    assert_eq!(plain.fizz_event, 1_501);
    assert!(!plain.generic_placement);
    let waterlogged = downward_lava_into_water(false);
    assert!(!waterlogged.write_stone);
    assert_eq!(waterlogged.stone_state_id, None);
    assert_eq!(waterlogged.fizz_event, 1_501);
}

#[test]
fn lava_fire_k_zero_can_place_three_and_unloaded_aborts_without_rollback() {
    let rising = [];
    let horizontal = [
        HorizontalFireProbe {
            loaded: true,
            base_ignited: true,
            above_empty: true,
        },
        HorizontalFireProbe {
            loaded: true,
            base_ignited: false,
            above_empty: true,
        },
        HorizontalFireProbe {
            loaded: true,
            base_ignited: true,
            above_empty: true,
        },
    ];
    let mut random = RandomScript::new(vec![0, 0, 1, 2, 0, 1, 2], vec![], vec![]);
    let plan = lava_random_fire(true, &rising, &horizontal, &mut random);
    assert_eq!(plan.fire_probe_indices, [0, 2]);
    assert_eq!(plan.sampled_offsets, [[-1, 0], [1, -1], [0, 1]]);
    assert!(!plan.aborted_unloaded);
    assert_eq!(random.bounds, [3; 7]);
    assert_eq!(BASE_FIRE_STATE_ID, 3_406);

    let mut unloaded = horizontal;
    unloaded[1].loaded = false;
    let mut random = RandomScript::new(vec![0, 1, 1, 1, 1], vec![], vec![]);
    let plan = lava_random_fire(true, &rising, &unloaded, &mut random);
    assert_eq!(plan.fire_probe_indices, [0]);
    assert!(plan.aborted_unloaded);
}

#[test]
fn lava_fire_rising_walk_stops_on_first_fire_motion_or_unloaded_probe() {
    let horizontal = [HorizontalFireProbe {
        loaded: true,
        base_ignited: false,
        above_empty: false,
    }; 3];
    let rising = [
        RisingFireProbe {
            loaded: true,
            air: true,
            ignited_neighbour: false,
            motion_blocking: false,
        },
        RisingFireProbe {
            loaded: true,
            air: true,
            ignited_neighbour: true,
            motion_blocking: false,
        },
    ];
    let mut random = RandomScript::new(vec![2, 2, 0, 1, 1], vec![], vec![]);
    let plan = lava_random_fire(true, &rising, &horizontal, &mut random);
    assert_eq!(plan.fire_probe_indices, [1]);
    assert_eq!(plan.sampled_offsets.len(), 2);

    let mut denied_random = RandomScript::default();
    assert_eq!(
        lava_random_fire(false, &rising, &horizontal, &mut denied_random),
        LavaFirePlan {
            sampled_offsets: Vec::new(),
            fire_probe_indices: Vec::new(),
            aborted_unloaded: false
        }
    );
    assert!(denied_random.bounds.is_empty());
}

#[test]
fn water_evaporation_consumes_26_floats_and_suppresses_normal_commit() {
    let floats: Vec<_> = (0..26).map(|value| value as f32 / 100.0).collect();
    let mut random = RandomScript::new(vec![], vec![], floats);
    let result = water_evaporation(true, true, true, &mut random);
    assert!(result.success && result.evaporated);
    assert!(!result.write_fluid && !result.ordinary_sound && !result.fluid_place_event);
    assert!((result.extinguish_pitch.expect("pitch") - 2.592).abs() < 1.0e-6);
    assert_eq!(result.smoke_samples.len(), 8);
    assert_eq!(result.smoke_samples[0], [0.02, 0.03, 0.04]);
    assert_eq!(random.float_index, 26);

    let mut normal_random = RandomScript::default();
    let normal = water_evaporation(true, true, false, &mut normal_random);
    assert!(normal.write_fluid && normal.ordinary_sound && normal.fluid_place_event);
    assert_eq!(normal_random.float_index, 0);
    assert!(!water_evaporation(false, true, true, &mut normal_random).success);
}

#[test]
fn potent_sulfur_registry_states_profile_and_resources_are_exact() {
    assert_eq!((BLOCK_ID, ITEM_ID, BLOCK_ENTITY_PROTOCOL_ID), (999, 27, 48));
    assert_eq!(
        (
            FIRST_STATE_ID,
            STATE_COUNT,
            HARDNESS,
            RESISTANCE,
            REQUIRES_CORRECT_TOOL
        ),
        (24_688, 5, 1.5, 6.0, true)
    );
    assert_eq!(PotentSulfurState::Dry.state_id(), 24_688);
    assert_eq!(PotentSulfurState::Wet.state_id(), 24_689);
    assert_eq!(PotentSulfurState::Dormant.state_id(), 24_690);
    assert_eq!(PotentSulfurState::Erupting.state_id(), 24_691);
    assert_eq!(PotentSulfurState::Continuous.state_id(), 24_692);
    assert_eq!(
        (
            SULFUR_BUBBLE_PARTICLE_ID,
            NOXIOUS_GAS_CLOUD_PARTICLE_ID,
            GEYSER_PARTICLE_ID,
            NOXIOUS_GAS_SOUND_ID,
            ENDER_DRAGON_ENTITY_TYPE_ID
        ),
        (4, 6, 7, 1_962, 43)
    );
    assert_eq!(GEYSER_SALT, -904_011_478);
}

#[test]
fn potent_sulfur_derivation_distinguishes_source_support_and_resets() {
    let other = GeyserSupport {
        continuous_tag: false,
        periodic_tag: false,
        fluid: SupportFluid::Empty,
    };
    assert_eq!(
        derive_potent_sulfur(PotentSulfurState::Continuous, false, other, true).state,
        PotentSulfurState::Dry
    );
    let continuous = GeyserSupport {
        continuous_tag: true,
        periodic_tag: false,
        fluid: SupportFluid::Source,
    };
    assert_eq!(
        derive_potent_sulfur(PotentSulfurState::Dry, true, continuous, true).state,
        PotentSulfurState::Continuous
    );
    assert_eq!(
        derive_potent_sulfur(
            PotentSulfurState::Dry,
            true,
            GeyserSupport {
                fluid: SupportFluid::Flowing,
                ..continuous
            },
            true
        )
        .state,
        PotentSulfurState::Wet
    );
    let periodic = GeyserSupport {
        continuous_tag: false,
        periodic_tag: true,
        fluid: SupportFluid::Empty,
    };
    let entered = derive_potent_sulfur(PotentSulfurState::Wet, true, periodic, true);
    assert_eq!(entered.state, PotentSulfurState::Dormant);
    assert!(entered.reset_countdown);
    assert!(
        !derive_potent_sulfur(PotentSulfurState::Dormant, true, periodic, true).reset_countdown
    );
    assert_eq!(
        derive_potent_sulfur(PotentSulfurState::Erupting, true, periodic, true).state,
        PotentSulfurState::Erupting
    );
    assert_eq!(
        derive_potent_sulfur(PotentSulfurState::Dry, true, other, true).state,
        PotentSulfurState::Wet
    );
}

#[test]
fn geyser_placement_and_ticker_stage_order_are_state_and_side_exact() {
    let erupting = geyser_on_place(PotentSulfurState::Erupting);
    assert_eq!(erupting.queue_event, Some((0, 0)));
    assert_eq!(erupting.sound_id, Some(ERUPTION_START_SOUND_ID));
    assert_eq!(erupting.game_event, Some(GeyserGameEvent::BlockActivate));
    assert_eq!(
        geyser_on_place(PotentSulfurState::Continuous).sound_id,
        Some(CONTINUOUS_START_SOUND_ID)
    );
    assert!(
        geyser_on_place(PotentSulfurState::Dormant)
            .queue_event
            .is_none()
    );
    assert_eq!(
        geyser_tick_stages(PotentSulfurState::Dormant, false),
        [GeyserTickStage::Countdown, GeyserTickStage::Nausea]
    );
    assert_eq!(
        geyser_tick_stages(PotentSulfurState::Erupting, false),
        [GeyserTickStage::Launch, GeyserTickStage::Countdown]
    );
    assert_eq!(
        geyser_tick_stages(PotentSulfurState::Continuous, true),
        [GeyserTickStage::Plume, GeyserTickStage::Launch]
    );
}

#[test]
fn geyser_source_scan_admits_zero_to_four_water_and_rejects_five_or_collision() {
    let mut cells = [passable_air(); COLUMN_PROBE_COUNT];
    assert_eq!(
        find_gas_source(&cells),
        Some(GasSource {
            source_offset_y: 1,
            water_blocks: 0
        })
    );
    for index in 0..MAX_WATER_BLOCKS {
        cells[index] = source_water();
        let source = find_gas_source(&cells).expect("bounded source");
        assert_eq!(source.water_blocks, index as u8 + 1);
    }
    cells[4] = source_water();
    assert_eq!(find_gas_source(&cells), None);

    let mut waterlogged_collision = [passable_air(); COLUMN_PROBE_COUNT];
    waterlogged_collision[0] = ColumnCell {
        source_water: true,
        water_block: false,
        air: false,
        empty_collision: false,
    };
    assert_eq!(find_gas_source(&waterlogged_collision), None);
    let mut obstruction = [passable_air(); COLUMN_PROBE_COUNT];
    obstruction[0] = source_water();
    obstruction[1] = ColumnCell {
        source_water: false,
        water_block: false,
        air: false,
        empty_collision: false,
    };
    assert_eq!(find_gas_source(&obstruction), None);
}

#[test]
fn geyser_runtime_persists_only_countdown_and_event_resets_client_epoch() {
    let mut runtime = GeyserRuntime::load_countdown(Some(i32::MIN));
    assert_eq!(runtime.saved_countdown(), i32::MIN);
    assert_eq!(runtime.eruption_tick, -1);
    runtime.set_level(40);
    runtime.set_level(80);
    assert_eq!(runtime.eruption_tick, 40);
    assert!(runtime.trigger_event(false, 100));
    assert_eq!(runtime.eruption_tick, 40);
    assert!(runtime.trigger_event(true, 100));
    assert_eq!(runtime.eruption_tick, 100);
    runtime.reset_countdown();
    assert_eq!(runtime.waiting_countdown, -1);
}

#[test]
fn dormant_countdown_initializes_from_bounded_draw_and_decrements_same_tick() {
    let source = Some(GasSource {
        source_offset_y: 3,
        water_blocks: 2,
    });
    let mut runtime = GeyserRuntime::default();
    let mut random = RandomScript::new(vec![0], vec![], vec![]);
    let skipped = countdown_tick(
        PotentSulfurState::Dormant,
        19,
        source,
        &mut runtime,
        &mut random,
    );
    assert!(!skipped.ran);
    let result = countdown_tick(
        PotentSulfurState::Dormant,
        20,
        source,
        &mut runtime,
        &mut random,
    );
    assert!(result.ran && result.initialized);
    assert_eq!(result.countdown, 24);
    assert_eq!(random.bounds, [16]);
    assert_eq!(result.state_write, None);

    runtime.waiting_countdown = 1;
    let transitioned = countdown_tick(
        PotentSulfurState::Dormant,
        40,
        source,
        &mut runtime,
        &mut random,
    );
    assert_eq!(transitioned.state_write, Some(PotentSulfurState::Erupting));
    assert_eq!(transitioned.write_flags, Some(3));
    assert_eq!(transitioned.game_event, None);
}

#[test]
fn erupting_zero_water_discards_draw_and_can_return_dormant_immediately() {
    let source = Some(GasSource {
        source_offset_y: 1,
        water_blocks: 0,
    });
    let mut runtime = GeyserRuntime::default();
    let mut random = RandomScript::new(vec![0], vec![123], vec![]);
    let result = countdown_tick(
        PotentSulfurState::Erupting,
        20,
        source,
        &mut runtime,
        &mut random,
    );
    assert_eq!(
        result,
        CountdownOutcome {
            ran: true,
            initialized: true,
            discarded_unbounded_draw: true,
            countdown: 0,
            state_write: Some(PotentSulfurState::Dormant),
            write_flags: Some(3),
            game_event: Some(GeyserGameEvent::BlockDeactivate)
        }
    );
    assert_eq!(random.unbounded_index, 1);
    assert_eq!(random.bounds, [2]);
}

#[test]
fn countdown_freezes_when_column_scan_fails_without_consuming_randomness() {
    let mut runtime = GeyserRuntime {
        waiting_countdown: 7,
        eruption_tick: -1,
    };
    let mut random = RandomScript::default();
    let result = countdown_tick(
        PotentSulfurState::Dormant,
        20,
        None,
        &mut runtime,
        &mut random,
    );
    assert!(!result.ran);
    assert_eq!(runtime.waiting_countdown, 7);
    assert!(random.bounds.is_empty());
}

#[test]
fn nausea_uses_inclusive_distance_and_every_visibility_gate() {
    let base = GasEntity {
        id: 1,
        alive: true,
        spectator: false,
        intersects_horizontal_query: true,
        eye_cell_passable: true,
        eye_distance_squared: 9.0,
        source_water_below_eye: true,
        collider_clip_hit: false,
    };
    let entities = [
        base,
        GasEntity {
            id: 2,
            eye_distance_squared: 9.000_001,
            ..base
        },
        GasEntity {
            id: 3,
            spectator: true,
            ..base
        },
        GasEntity {
            id: 4,
            collider_clip_hit: true,
            ..base
        },
        GasEntity {
            id: 5,
            source_water_below_eye: false,
            ..base
        },
    ];
    let source = Some(GasSource {
        source_offset_y: 2,
        water_blocks: 1,
    });
    assert!(nausea_applications(9, source, &entities).is_empty());
    let applications = nausea_applications(10, source, &entities);
    assert_eq!(applications.len(), 1);
    assert_eq!(applications[0].entity_id, 1);
    assert_eq!(applications[0].duration, NAUSEA_DURATION);
    assert!(applications[0].ambient && applications[0].visible);
}

#[test]
fn gas_cloud_and_obstruction_height_follow_modulo_and_six_per_water() {
    let source = Some(GasSource {
        source_offset_y: 3,
        water_blocks: 2,
    });
    assert!(client_gas_cloud(CLOUD_FREQUENCY, source));
    assert!(!client_gas_cloud(CLOUD_FREQUENCY - 1, source));
    assert!(!client_gas_cloud(CLOUD_FREQUENCY, None));
    assert_eq!(unobstructed_count(2, &[true; 12]), 12);
    assert_eq!(unobstructed_count(2, &[true, true, false, true]), 2);
    assert_eq!(unobstructed_count(0, &[]), 0);
    assert_eq!(launch_query_expand_y(0), -1);
    assert_eq!(LAUNCH_HEIGHT_MULTIPLIER, 6);
}

#[test]
fn geyser_launch_accumulates_fall_before_all_later_rejection_gates() {
    let base = LaunchEntity {
        id: 1,
        alive: true,
        spectator: false,
        can_simulate_movement: true,
        flying_player: false,
        passenger: false,
        immune_to_geysers: false,
        vertical_velocity: 0.49,
    };
    let entities = [
        base,
        LaunchEntity {
            id: 2,
            vertical_velocity: 0.5,
            ..base
        },
        LaunchEntity {
            id: 3,
            can_simulate_movement: false,
            ..base
        },
        LaunchEntity {
            id: 4,
            flying_player: true,
            ..base
        },
        LaunchEntity {
            id: 5,
            passenger: true,
            ..base
        },
        LaunchEntity {
            id: 6,
            immune_to_geysers: true,
            ..base
        },
        LaunchEntity {
            id: 7,
            alive: false,
            ..base
        },
    ];
    let plan = geyser_launch(2, &entities);
    assert!((plan.threshold - 0.5).abs() < 1.0e-7);
    assert_eq!(plan.fall_distance_entity_ids, [1, 2, 3, 4, 5, 6]);
    assert_eq!(plan.launched_entity_ids, [1, 2]);
    assert_eq!(plan.velocity_addition, LAUNCH_FORCE);
    assert!(plan.mark_sync);
}

#[test]
fn client_plume_uses_event_epoch_and_state_specific_active_sound() {
    let source = Some(GasSource {
        source_offset_y: 3,
        water_blocks: 2,
    });
    let eruption = client_plume(PotentSulfurState::Erupting, 100, 60, source);
    assert!(eruption.geyser_particle && eruption.play_active_sound);
    assert_eq!(eruption.particle_water_blocks, Some(2));
    assert_eq!(eruption.sound_id, Some(ERUPTION_ACTIVE_SOUND_ID));
    let continuous = client_plume(PotentSulfurState::Continuous, 80, 40, source);
    assert_eq!(continuous.sound_id, Some(CONTINUOUS_ACTIVE_SOUND_ID));
    let off_cadence = client_plume(PotentSulfurState::Erupting, 101, 60, source);
    assert!(!off_cadence.geyser_particle && !off_cadence.play_active_sound);
    assert_eq!((PLUME_FREQUENCY, ACTIVE_SOUND_FREQUENCY), (20, 40));
}

#[test]
fn client_display_consumes_six_floats_then_one_ten_bound_draw() {
    let mut random = RandomScript::new(vec![0], vec![], vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]);
    let display = geyser_display_tick(PotentSulfurState::Wet, true, &mut random);
    assert_eq!(display.bubble_positions, [[0.1, 1.2, 0.3], [0.4, 1.5, 0.6]]);
    assert!(display.play_noxious_sound);
    assert!(display.sound_position_at_integer_corner);
    assert_eq!(random.float_index, 6);
    assert_eq!(random.bounds, [10]);

    let mut denied = RandomScript::default();
    assert!(
        geyser_display_tick(PotentSulfurState::Dry, true, &mut denied)
            .bubble_positions
            .is_empty()
    );
    assert!(
        geyser_display_tick(PotentSulfurState::Wet, false, &mut denied)
            .bubble_positions
            .is_empty()
    );
    assert_eq!(denied.float_index, 0);
}

#[test]
fn potent_sulfur_loot_copies_no_countdown_and_uses_explosion_gate() {
    assert!(potent_sulfur_loot(true));
    assert!(!potent_sulfur_loot(false));
}
