use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_gameplay::environment::fire::{
    AGE_WRITE_FLAGS, BaseFireKind, BaseFireState, CONTACT_ORDER, CONTACT_PLAYER_DRAW_BOUND,
    CONTACT_PLAYER_DRAW_ORIGIN, ContactStage, DEFAULT_SPREAD_RADIUS, DIRECT_BURN_ORDER,
    DO_FIRE_TICK_RULE_PRESENT, DirectBurnProbe, DirectDirection, FIRE_CALLBACK_ORDER,
    FIRE_ODDS_TABLE, FIRE_SCHEDULE_BASE, FIRE_SCHEDULE_SPREAD, FireCallbackStage,
    FireNeighbourOdds, FireOdds, FirePlayer, FuelMutation, INCREASED_BURNOUT_BIOME_COUNT,
    INCREASED_BURNOUT_BIOMES, InfiniburnSet, MAX_FIRE_AGE, NearRainResult, PORTAL_AXIS_DRAW_BOUND,
    PortalAxis, PortalDimension, RAIN_PROBE_ORDER, REGISTERED_FIRE_ODDS_COUNT,
    REPLACEMENT_AGE_DRAW_BOUND, RainProbe, SOUL_FIRE_CAN_BURN, SOUL_FIRE_HAS_SCHEDULED_CALLBACK,
    SPATIAL_CANDIDATE_COUNT, SPREAD_WRITE_FLAGS, SelfRemoval, SpatialSpreadProbe,
    UNLIMITED_SPREAD_RADIUS, can_be_placed, direct_age_gate_bound, direct_burn, direct_denominator,
    fire_contact, fire_odds, fire_schedule_delay, increased_burnout, is_infiniburn,
    near_player_admits, near_rain, next_fire_age, ordinary_fire_state, ordinary_survives,
    placement_plan, preferred_portal_axes, rain_extinguish, selected_fire_state, self_removal,
    soul_shape_update, spatial_denominator, spatial_offsets, spatial_spread, spread_threshold,
    survival_removal, tnt_prime,
};
use std::collections::{BTreeMap, BTreeSet};

fn neighbour_odds() -> FireNeighbourOdds {
    FireNeighbourOdds {
        below: 0,
        above: 5,
        north: 0,
        south: 15,
        west: 0,
        east: 30,
    }
}

fn direct_probe() -> DirectBurnProbe {
    DirectBurnProbe {
        burn_odds: 20,
        denominator_draw: 0,
        captured_age: 4,
        age_gate_draw: 0,
        target_raining: false,
        replacement_age_draw: 0,
        replacement_kind: BaseFireKind::Ordinary,
        captured_target_is_tnt: false,
    }
}

fn spatial_probe() -> SpatialSpreadProbe {
    SpatialSpreadProbe {
        candidate_empty: true,
        encouragement: 60,
        difficulty_id: 3,
        captured_age: 0,
        increased: false,
        spread_draw: 0,
        active_rain: false,
        candidate_near_rain: false,
        age_draw: 0,
        replacement_kind: BaseFireKind::Ordinary,
    }
}

#[test]
fn locked_fire_table_is_unique_closed_and_group_complete() {
    assert_eq!(FIRE_ODDS_TABLE.len(), REGISTERED_FIRE_ODDS_COUNT);
    let paths: BTreeSet<_> = FIRE_ODDS_TABLE.iter().map(|entry| entry.path).collect();
    assert_eq!(paths.len(), 207);

    let mut groups = BTreeMap::new();
    for entry in FIRE_ODDS_TABLE {
        *groups
            .entry((entry.odds.ignite, entry.odds.burn))
            .or_insert(0_usize) += 1;
    }
    assert_eq!(
        groups,
        BTreeMap::from([
            ((5, 5), 39),
            ((5, 20), 56),
            ((5, 100), 3),
            ((15, 20), 1),
            ((15, 60), 2),
            ((15, 100), 3),
            ((30, 20), 13),
            ((30, 60), 31),
            ((60, 20), 17),
            ((60, 60), 2),
            ((60, 100), 40),
        ])
    );
    assert_eq!(
        fire_odds("stripped_bamboo_block", false),
        FireOdds::new(5, 5)
    );
    assert_eq!(fire_odds("pale_oak_shelf", false), FireOdds::new(30, 20));
    assert_eq!(fire_odds("golden_dandelion", false), FireOdds::new(60, 100));
    assert_eq!(fire_odds("crimson_stem", false), FireOdds::new(0, 0));
    assert_eq!(fire_odds("oak_log", true), FireOdds::new(0, 0));
}

#[test]
fn ordinary_and_soul_placement_recompute_support_and_shape_exactly() {
    let neighbours = neighbour_odds();
    let shaped = ordinary_fire_state(7, false, neighbours);
    assert_eq!(
        (
            shaped.age,
            shaped.up,
            shaped.north,
            shaped.south,
            shaped.west,
            shaped.east
        ),
        (7, true, false, true, false, true)
    );
    let supported = ordinary_fire_state(
        4,
        true,
        FireNeighbourOdds {
            below: 0,
            above: 60,
            ..neighbours
        },
    );
    assert_eq!(
        (
            supported.up,
            supported.north,
            supported.south,
            supported.west,
            supported.east
        ),
        (false, false, false, false, false)
    );
    assert!(ordinary_survives(false, neighbours));
    assert_eq!(
        selected_fire_state(9, true, false, neighbours),
        BaseFireState::Soul
    );
    assert!(matches!(
        selected_fire_state(9, false, false, neighbours),
        BaseFireState::Ordinary(state) if state.age == 9
    ));
    assert_eq!(
        selected_fire_state(
            9,
            false,
            false,
            FireNeighbourOdds {
                below: 0,
                above: 0,
                north: 0,
                south: 0,
                west: 0,
                east: 0,
            }
        ),
        BaseFireState::Air
    );
    assert_eq!(soul_shape_update(true), BaseFireState::Soul);
    assert_eq!(soul_shape_update(false), BaseFireState::Air);
    assert!(std::hint::black_box(SOUL_FIRE_CAN_BURN));
    assert!(!std::hint::black_box(SOUL_FIRE_HAS_SCHEDULED_CALLBACK));
}

#[test]
fn portal_preference_placement_and_stale_schedule_boundaries_are_explicit() {
    assert_eq!(
        preferred_portal_axes(Direction::North, 1).axes,
        [PortalAxis::X, PortalAxis::Z]
    );
    assert!(!preferred_portal_axes(Direction::West, 0).vertical_draw_consumed);
    assert_eq!(
        preferred_portal_axes(Direction::West, 0).axes,
        [PortalAxis::Z, PortalAxis::X]
    );
    let vertical = preferred_portal_axes(Direction::Up, 1);
    assert!(vertical.vertical_draw_consumed);
    assert_eq!(vertical.axes, [PortalAxis::Z, PortalAxis::X]);
    assert_eq!(std::hint::black_box(PORTAL_AXIS_DRAW_BOUND), 2);

    assert!(can_be_placed(
        true,
        false,
        PortalDimension::Overworld,
        true,
        true
    ));
    assert!(!can_be_placed(
        true,
        false,
        PortalDimension::Other,
        true,
        true
    ));
    assert!(!can_be_placed(
        false,
        true,
        PortalDimension::Overworld,
        true,
        true
    ));

    let portal = placement_plan(false, PortalDimension::Nether, true, true, false, true, 9);
    assert_eq!(portal.create_portal, Some(PortalAxis::X));
    assert!(!portal.remove_without_drops);
    assert_eq!(portal.ordinary_schedule_delay, Some(39));

    let removed = placement_plan(false, PortalDimension::Other, false, false, false, true, 0);
    assert!(removed.remove_without_drops);
    assert_eq!(removed.ordinary_schedule_delay, Some(30));
    assert_eq!(
        placement_plan(true, PortalDimension::Overworld, true, true, false, true, 3)
            .ordinary_schedule_delay,
        Some(33)
    );
}

#[test]
fn nearby_player_gate_uses_integer_corner_strict_distance_and_spectators() {
    let fire = BlockPos::new(0, 64, 0);
    let equality = FirePlayer {
        position: [128.0, 64.0, 0.0],
        spectator: false,
    };
    assert!(!near_player_admits(
        fire,
        DEFAULT_SPREAD_RADIUS,
        &[equality]
    ));
    assert!(near_player_admits(
        fire,
        DEFAULT_SPREAD_RADIUS,
        &[FirePlayer {
            position: [127.999, 64.0, 0.0],
            spectator: false,
        }]
    ));
    assert!(!near_player_admits(
        fire,
        128,
        &[FirePlayer {
            spectator: true,
            ..equality
        }]
    ));
    assert!(!near_player_admits(fire, 0, &[equality]));
    assert!(near_player_admits(fire, UNLIMITED_SPREAD_RADIUS, &[]));
}

#[test]
fn callback_order_schedule_infiniburn_and_attribute_sets_are_locked() {
    assert_eq!(
        FIRE_CALLBACK_ORDER,
        [
            FireCallbackStage::Reschedule,
            FireCallbackStage::NearbyPlayer,
            FireCallbackStage::SurvivalRemoval,
            FireCallbackStage::Infiniburn,
            FireCallbackStage::Rain,
            FireCallbackStage::Age,
            FireCallbackStage::SelfRemoval,
            FireCallbackStage::IncreasedBurnout,
            FireCallbackStage::DirectFuel,
            FireCallbackStage::SpatialSpread,
        ]
    );
    assert_eq!(
        (
            FIRE_SCHEDULE_BASE,
            FIRE_SCHEDULE_SPREAD,
            fire_schedule_delay(0),
            fire_schedule_delay(9),
            AGE_WRITE_FLAGS,
            SPREAD_WRITE_FLAGS
        ),
        (30, 10, 30, 39, 260, 3)
    );
    assert!(!std::hint::black_box(DO_FIRE_TICK_RULE_PRESENT));
    let removed = survival_removal(false);
    assert!(removed.remove_without_drops);
    assert!(removed.continue_after_removal_attempt);
    assert!(!survival_removal(true).remove_without_drops);
    for set in [
        InfiniburnSet::Overworld,
        InfiniburnSet::Nether,
        InfiniburnSet::End,
    ] {
        assert!(is_infiniburn(set, "netherrack"));
        assert!(is_infiniburn(set, "magma_block"));
    }
    assert!(is_infiniburn(InfiniburnSet::End, "bedrock"));
    assert!(!is_infiniburn(InfiniburnSet::Overworld, "bedrock"));
    assert_eq!(
        INCREASED_BURNOUT_BIOMES.len(),
        INCREASED_BURNOUT_BIOME_COUNT
    );
    for biome in INCREASED_BURNOUT_BIOMES {
        assert!(increased_burnout(biome));
    }
    assert!(!increased_burnout("plains"));
}

#[test]
fn rain_short_circuit_chance_and_age_draws_keep_strict_boundaries() {
    assert_eq!(
        RAIN_PROBE_ORDER,
        [
            RainProbe::Current,
            RainProbe::West,
            RainProbe::East,
            RainProbe::North,
            RainProbe::South,
        ]
    );
    assert_eq!(
        near_rain([false, false, true, true, true]),
        NearRainResult {
            near: true,
            probes: 3,
        }
    );
    assert_eq!(near_rain([false; 5]).probes, 5);

    let equality = rain_extinguish(false, true, true, 10, 0.5);
    assert!(equality.draw_consumed);
    assert!(!equality.remove_and_return);
    assert!(rain_extinguish(false, true, true, 10, 0.499).remove_and_return);
    assert!(!rain_extinguish(true, true, true, 10, 0.0).draw_consumed);
    assert!(!rain_extinguish(false, false, true, 10, 0.0).draw_consumed);

    assert_eq!(next_fire_age(7, 0), 7);
    assert_eq!(next_fire_age(7, 1), 7);
    assert_eq!(next_fire_age(7, 2), 8);
    assert_eq!(next_fire_age(15, 2), MAX_FIRE_AGE);
}

#[test]
fn self_removal_uses_captured_age_and_draws_only_at_age_fifteen() {
    let supported = self_removal(false, false, true, false, 3, 0);
    assert_eq!(supported.outcome, SelfRemoval::Return);
    assert!(!supported.age_fifteen_draw_consumed);
    assert_eq!(
        self_removal(false, false, true, false, 4, 0).outcome,
        SelfRemoval::RemoveAndReturn
    );
    assert_eq!(
        self_removal(false, true, true, false, 14, 0).outcome,
        SelfRemoval::Continue
    );
    let old = self_removal(false, true, true, false, 15, 0);
    assert!(old.age_fifteen_draw_consumed);
    assert_eq!(old.outcome, SelfRemoval::RemoveAndReturn);
    assert_eq!(
        self_removal(false, true, true, true, 15, 0).outcome,
        SelfRemoval::Continue
    );
    assert!(!self_removal(true, false, false, false, 15, 0).age_fifteen_draw_consumed);
}

#[test]
fn direct_burn_order_denominators_and_six_unconditional_draws_are_exact() {
    assert_eq!(
        DIRECT_BURN_ORDER,
        [
            DirectDirection::East,
            DirectDirection::West,
            DirectDirection::Below,
            DirectDirection::Above,
            DirectDirection::North,
            DirectDirection::South,
        ]
    );
    assert_eq!(direct_denominator(DirectDirection::East, false), 300);
    assert_eq!(direct_denominator(DirectDirection::Below, false), 250);
    assert_eq!(direct_denominator(DirectDirection::South, true), 250);
    assert_eq!(direct_denominator(DirectDirection::Above, true), 200);
    assert_eq!(
        (direct_age_gate_bound(0), direct_age_gate_bound(15)),
        (10, 25)
    );
    assert_eq!(std::hint::black_box(REPLACEMENT_AGE_DRAW_BOUND), 5);

    let inert = direct_burn(DirectBurnProbe {
        burn_odds: 0,
        ..direct_probe()
    });
    assert!(inert.first_draw_consumed);
    assert_eq!(inert.mutation, FuelMutation::None);
    assert!(!inert.age_gate_draw_consumed);
}

#[test]
fn direct_target_replacement_removal_rain_and_tnt_are_nontransactional() {
    let replacement = direct_burn(DirectBurnProbe {
        denominator_draw: 19,
        captured_age: 14,
        age_gate_draw: 4,
        replacement_age_draw: 4,
        replacement_kind: BaseFireKind::Soul,
        captured_target_is_tnt: true,
        ..direct_probe()
    });
    assert_eq!(
        replacement.mutation,
        FuelMutation::ReplaceWithFire {
            kind: BaseFireKind::Soul,
            age: 15,
        }
    );
    assert!(replacement.age_gate_draw_consumed);
    assert!(replacement.rain_queried);
    assert!(replacement.replacement_age_draw_consumed);
    assert!(replacement.prime_tnt_after_mutation);

    assert_eq!(
        direct_burn(DirectBurnProbe {
            denominator_draw: 20,
            captured_age: 14,
            captured_target_is_tnt: true,
            ..direct_probe()
        })
        .mutation,
        FuelMutation::None
    );
    let age_denied = direct_burn(DirectBurnProbe {
        age_gate_draw: 5,
        ..direct_probe()
    });
    assert_eq!(age_denied.mutation, FuelMutation::RemoveWithoutDrops);
    assert!(!age_denied.rain_queried);
    let rain_denied = direct_burn(DirectBurnProbe {
        age_gate_draw: 4,
        target_raining: true,
        ..direct_probe()
    });
    assert_eq!(rain_denied.mutation, FuelMutation::RemoveWithoutDrops);
    assert!(rain_denied.rain_queried);
}

#[test]
fn spatial_scan_order_denominators_and_threshold_arithmetic_are_exact() {
    let offsets = spatial_offsets();
    assert_eq!(offsets.len(), SPATIAL_CANDIDATE_COUNT);
    assert_eq!(offsets[0], [-1, -1, -1]);
    assert_eq!(offsets[5], [-1, 4, -1]);
    assert!(!offsets.contains(&[0, 0, 0]));
    assert_eq!(offsets[52], [1, 4, 1]);
    assert_eq!(
        [
            spatial_denominator(-1),
            spatial_denominator(0),
            spatial_denominator(1),
            spatial_denominator(2),
            spatial_denominator(3),
            spatial_denominator(4),
        ],
        [100, 100, 100, 200, 300, 400]
    );
    assert_eq!(spread_threshold(60, 3, 0, false), 4);
    assert_eq!(spread_threshold(60, 3, 0, true), 2);
    assert_eq!(spread_threshold(1, 0, 15, false), 0);
}

#[test]
fn spatial_spread_uses_inclusive_comparison_then_rain_then_age() {
    let equality = spatial_spread(SpatialSpreadProbe {
        spread_draw: 4,
        candidate_near_rain: true,
        age_draw: 4,
        replacement_kind: BaseFireKind::Soul,
        ..spatial_probe()
    });
    assert!(equality.spread_draw_consumed);
    assert!(!equality.rain_queried);
    assert!(equality.age_draw_consumed);
    assert_eq!(equality.write_fire, Some((BaseFireKind::Soul, 1)));

    let miss = spatial_spread(SpatialSpreadProbe {
        spread_draw: 5,
        active_rain: true,
        candidate_near_rain: true,
        age_draw: 4,
        ..spatial_probe()
    });
    assert!(miss.spread_draw_consumed);
    assert!(!miss.rain_queried);
    assert_eq!(miss.write_fire, None);

    let rain = spatial_spread(SpatialSpreadProbe {
        active_rain: true,
        candidate_near_rain: true,
        age_draw: 4,
        ..spatial_probe()
    });
    assert!(rain.rain_queried);
    assert!(!rain.age_draw_consumed);
    assert_eq!(rain.write_fire, None);

    for plan in [
        spatial_spread(SpatialSpreadProbe {
            candidate_empty: false,
            ..spatial_probe()
        }),
        spatial_spread(SpatialSpreadProbe {
            encouragement: 0,
            ..spatial_probe()
        }),
        spatial_spread(SpatialSpreadProbe {
            encouragement: 1,
            difficulty_id: 0,
            captured_age: 15,
            ..spatial_probe()
        }),
    ] {
        assert!(!plan.spread_draw_consumed);
    }
}

#[test]
fn tnt_rule_and_admission_failure_preserve_post_mutation_sound_and_event() {
    let disabled = tnt_prime(false, true);
    assert!(!disabled.entity_admission_attempted);
    assert!(!disabled.play_primed_sound);
    assert!(!disabled.emit_prime_fuse);

    let failed = tnt_prime(true, false);
    assert!(failed.entity_admission_attempted);
    assert!(!failed.entity_admitted);
    assert!(failed.centered_at_integer_y);
    assert!(failed.play_primed_sound);
    assert!(failed.emit_prime_fuse);
}

#[test]
fn base_fire_contact_orders_effects_and_keeps_counter_rng_boundaries() {
    assert_eq!(
        CONTACT_ORDER,
        [
            ContactStage::ClearFreeze,
            ContactStage::FireIgnite,
            ContactStage::QueueDamage,
        ]
    );
    assert_eq!(
        std::hint::black_box((CONTACT_PLAYER_DRAW_ORIGIN, CONTACT_PLAYER_DRAW_BOUND)),
        (1, 3)
    );
    let negative = fire_contact(BaseFireKind::Ordinary, false, true, -2, 2);
    assert_eq!(negative.remaining_fire, -1);
    assert!(!negative.player_draw_consumed);
    assert_eq!(negative.set_seconds_on_fire, None);
    assert_eq!(negative.queued_in_fire_damage, 1.0);

    let crossing = fire_contact(BaseFireKind::Soul, false, false, -1, 2);
    assert_eq!(crossing.remaining_fire, 0);
    assert_eq!(crossing.set_seconds_on_fire, Some(8));
    assert_eq!(crossing.queued_in_fire_damage, 2.0);

    let player = fire_contact(BaseFireKind::Ordinary, false, true, 0, 2);
    assert!(player.player_draw_consumed);
    assert_eq!(player.remaining_fire, 2);
    let nonplayer = fire_contact(BaseFireKind::Ordinary, false, false, 0, 2);
    assert!(!nonplayer.player_draw_consumed);
    assert_eq!(nonplayer.remaining_fire, 0);

    let immune = fire_contact(BaseFireKind::Soul, true, true, 4, 2);
    assert!(immune.ignition_aborted);
    assert!(!immune.player_draw_consumed);
    assert_eq!(immune.remaining_fire, 4);
    assert_eq!(immune.queued_in_fire_damage, 2.0);
}
