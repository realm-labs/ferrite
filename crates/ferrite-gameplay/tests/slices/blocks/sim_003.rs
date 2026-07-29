use ferrite_foundation::direction::Direction;
use ferrite_gameplay::block::beacon::{
    BLOCK_ENTITY_PROTOCOL_ID as BEACON_ENTITY_ID, BLOCK_STATE_ID as BEACON_STATE_ID, BeaconScan,
    BeaconSelection, BeamCell, BeamSection, effect_application, validate_selection,
};
use ferrite_gameplay::block::brushable::{
    BrushResult, BrushableState, ResetResult, complete, dust_stage, materialize_loot,
    pulse_is_authoritative,
};
use ferrite_gameplay::block::command_block::{
    ChainStep, CommandMode, CommandState, ExecutionResult, chain_step, due_tick, minecart_admitted,
    neighbor_edge,
};
use ferrite_gameplay::block::conduit::{
    TargetRefresh, TargetSnapshot, attack, client_particle_draws, effect_radius,
    next_short_ambient_deadline, player_receives_power, scan_frame, short_ambient_due,
    target_refresh,
};
use ferrite_gameplay::block::coral::{
    CoralBlockDrop, CoralBlockIdentity, CoralColor, CoralDue, CoralForm, CoralPlantDrop,
    CoralPlantState, CoralPlantUpdate, adjacent_water, coral_block_drop, coral_block_due,
    coral_plant_drop, coral_plant_due, coral_plant_update, drying_delay,
};
use ferrite_gameplay::block::lectern::{
    InsertResult, LecternContent, LecternState, PageResult, direct_signal, due_state,
    removal_outcome, weak_signal,
};
use ferrite_gameplay::block::nether::{
    Axis, ComposterResult, NetherPlant, NetherWartState, StemKind, VegetationChoice, WartBlockKind,
    compost, crimson_vegetation_choice, explosion_decay, strip, warped_vegetation_choice,
    wart_loot_base,
};
use ferrite_gameplay::block::sculk_sensor::{
    SensorPhase, VibrationCandidate, activation, arrival_power, calibrated_admits,
    calibrated_state_id, chunks_admit_delivery, comparator_signal, direct_signal as sensor_direct,
    ordinary_state_id, resonance_note, select_candidate, selection_ready, travel_delay,
    vibration_frequency, weak_signal as sensor_weak,
};
use ferrite_gameplay::block::sign::{
    Applicator, ApplicatorResult, ClickAction, EditResult, EditorLease, EmptyHandResult, SignLine,
    SignText, apply, commit_edit, empty_hand, hanging_chain_precedence, outline_visible,
    renderer_light, selected_side,
};
use ferrite_gameplay::block::skull::{
    NoteSound, SkullAnimation, SkullType, WitherAdmission, WitherResult, dragon_jaw_rotation,
    floor_note_sound, neighbor_power, piglin_ear_rotations, summon_wither,
};
use ferrite_gameplay::block::test_block::{
    EDIT_EFFECTS, ScanResult, TestBlockEntity, TestBlockMode, edit, scan_outcomes,
    start_cardinality_valid, truncate_ui_message,
};
use ferrite_gameplay::item::honeycomb::{CopperChestType, WaxEffect, use_honeycomb};

#[test]
fn beacon_scan_publication_effects_and_selection_are_source_ordered() {
    assert_eq!(BEACON_STATE_ID, 9_980);
    assert_eq!(BEACON_ENTITY_ID, 15);
    let mut scan = BeaconScan::new(-64);
    scan.begin(10);
    let first = scan.advance(
        [
            BeamCell::Transparent,
            BeamCell::Color(0xffff_0000),
            BeamCell::Color(0xffff_0000),
            BeamCell::Color(0xff00_00ff),
        ],
        14,
    );
    assert_eq!(first.visited, 4);
    assert!(first.completed);
    assert_eq!(
        scan.published,
        [
            BeamSection {
                color: 0xffff_ffff,
                height: 2,
            },
            BeamSection {
                color: 0xffff_0000,
                height: 2,
            },
            BeamSection {
                color: 0xff7f_007f,
                height: 1,
            },
        ]
    );

    scan.begin(10);
    let blocked = scan.advance([BeamCell::Dampening(15)], 20);
    assert!(blocked.blocked);
    assert!(scan.published.is_empty());
    assert_eq!(
        effect_application(4, true, true, false)
            .unwrap()
            .primary_amplifier,
        1
    );
    assert!(
        effect_application(4, true, false, true)
            .unwrap()
            .apply_secondary
    );
    assert_eq!(
        validate_selection(3, false, 1, true, false, false),
        BeaconSelection::NullSecondaryFault
    );
}

#[test]
fn brushable_cooldown_reset_loot_and_completion_preserve_quirks() {
    assert_eq!(
        (0..=10).map(dust_stage).collect::<Vec<_>>(),
        [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3]
    );
    let mut state = BrushableState::default();
    assert_eq!(
        state.brush(100, Direction::North),
        BrushResult::Advanced {
            stage_changed: true,
            completed: false,
        }
    );
    assert_eq!(
        state.brush(109, Direction::South),
        BrushResult::CooldownRejected
    );
    assert_eq!(state.reset_at, 149);
    assert!(matches!(state.reset_tick(149), ResetResult::Cleared { .. }));
    assert_eq!(state.direction, None);
    assert!(pulse_is_authoritative(65));
    assert!(!pulse_is_authoritative(64));
    assert_eq!(materialize_loot(&[3, 7, 9]), Some(3));
    let ejected = complete(64, 20, None);
    assert_eq!((ejected.split_count, ejected.retained_count), (30, 0));
    assert_eq!(ejected.direction, Direction::Up);
}

#[test]
fn command_blocks_keep_scheduler_and_chain_live_behind_dispatch_gate() {
    let mut state = CommandState {
        command: "say hi".to_owned(),
        last_output: Some("old".to_owned()),
        ..CommandState::default()
    };
    assert_eq!(
        state.perform(10, false, 9),
        ExecutionResult::Completed { dispatched: false }
    );
    assert_eq!(state.success_count, 0);
    assert_eq!(state.last_output.as_deref(), Some("old"));
    assert_eq!(state.last_execution, 10);
    assert_eq!(
        state.perform(10, true, 1),
        ExecutionResult::SameTickSuppressed
    );
    state.set_command("SeArGe");
    assert_eq!(state.perform(11, false, 0), ExecutionResult::Searge);
    assert_eq!(state.last_execution, 10);
    assert_eq!(state.last_output.as_deref(), Some("#itzlipofutzli"));

    let edge = neighbor_edge(false, true, false, CommandMode::Redstone, true);
    assert_eq!(edge.schedule_delay, Some(1));
    assert_eq!(
        due_tick(CommandMode::Auto, false, true, true),
        ferrite_gameplay::block::command_block::DueResult {
            execute: false,
            clear_success: true,
            next_condition: true,
            reschedule: true,
            update_comparator: true,
        }
    );
    assert!(minecart_admitted(true, 7, 3));
    assert!(!minecart_admitted(true, 6, 3));
    assert_eq!(
        chain_step(true, CommandMode::Sequence, false, true),
        ChainStep::SkipAndContinue
    );
}

#[test]
fn conduit_frame_effect_target_and_ambient_boundaries_are_exact() {
    let frame = scan_frame(&[true; 27], &[true; 42]);
    assert_eq!(
        (frame.frame_count, frame.active, frame.hunting),
        (42, true, true)
    );
    assert_eq!(effect_radius(16), 32);
    assert_eq!(effect_radius(42), 96);
    assert!(player_receives_power(42, 96 * 96, true));
    assert!(!player_receives_power(42, 96 * 96 + 1, true));
    assert_eq!(
        target_refresh(
            true,
            true,
            Some(TargetSnapshot {
                alive: false,
                block_distance_squared: 1,
            })
        ),
        TargetRefresh::ClearWithoutReselect
    );
    assert_eq!(
        target_refresh(false, false, None),
        TargetRefresh::InactiveRetain
    );
    assert_eq!(attack().damage, 4);
    let deadline = next_short_ambient_deadline(100, 39);
    assert_eq!(deadline, 199);
    assert!(!short_ambient_due(199, deadline));
    assert!(short_ambient_due(200, deadline));
    assert_eq!(client_particle_draws(false, true).target_float_draws, 3);
}

#[test]
fn coral_blocks_scan_water_then_terminally_convert_with_locked_loot() {
    assert_eq!(
        adjacent_water([false, false, true, true, true, true]),
        Some(Direction::North)
    );
    let live = CoralBlockIdentity {
        color: CoralColor::Fire,
        live: true,
    };
    assert_eq!(
        (live.state_id(), live.block_id(), live.item_id()),
        (15_145, 756, 685)
    );
    assert_eq!(drying_delay(0), 60);
    assert_eq!(drying_delay(39), 99);
    assert_eq!(
        coral_block_due(live, false),
        CoralDue::Convert {
            target: live.dead(),
            flags: 2,
        }
    );
    assert_eq!(
        coral_block_drop(live, true, true, false),
        CoralBlockDrop::Live(CoralColor::Fire)
    );
    assert_eq!(
        coral_block_drop(live, true, false, true),
        CoralBlockDrop::Dead(CoralColor::Fire)
    );
}

#[test]
fn coral_plants_prioritize_support_and_preserve_wall_facing_on_drying() {
    let wall = CoralPlantState {
        form: CoralForm::WallFan,
        color: CoralColor::Horn,
        live: true,
        waterlogged: true,
        facing: Direction::East,
    };
    assert_eq!(wall.state_id(), 15_265);
    assert_eq!(
        coral_plant_update(wall, false, true),
        CoralPlantUpdate::Remove
    );
    assert_eq!(
        coral_plant_update(wall, true, true),
        CoralPlantUpdate::Keep {
            dry_tick: false,
            water_tick_requests: 2,
        }
    );
    let dead = coral_plant_due(
        CoralPlantState {
            waterlogged: false,
            ..wall
        },
        false,
    )
    .unwrap();
    assert_eq!(dead.facing, Direction::East);
    assert!(!dead.live);
    assert!(!dead.waterlogged);
    assert_eq!(
        coral_plant_drop(wall, false, true),
        CoralPlantDrop::FloorFan(CoralColor::Horn)
    );
}

#[test]
fn lectern_content_remains_independent_from_state_and_pulses_deduplicate() {
    let empty_state = LecternState {
        has_book: false,
        powered: false,
    };
    let mut content = LecternContent::default();
    assert!(matches!(
        content.insert(empty_state, 5),
        InsertResult::Inserted {
            update_flags: 3,
            ..
        }
    ));
    assert_eq!(
        content.set_page(4, false),
        PageResult::Changed {
            page: 4,
            schedule_delay: Some(2),
            level_event: 1043,
        }
    );
    assert_eq!(content.set_page(4, true), PageResult::Unchanged);
    let zero_pages = LecternContent::load(true, 0, 5);
    assert_eq!(zero_pages.page, -1);
    let divergent = LecternState {
        has_book: true,
        powered: true,
    };
    assert_eq!(LecternContent::default().analog_output(divergent), 14);
    assert_eq!(
        (weak_signal(divergent), direct_signal(divergent, true)),
        (15, 15)
    );
    assert!(!due_state(divergent).powered);
    assert!(!removal_outcome(divergent).clear_content);
}

#[test]
fn nether_roots_and_sprouts_lock_identity_support_loot_and_composting() {
    assert_eq!(
        (
            NetherPlant::WarpedRoots.state_id(),
            NetherPlant::CrimsonRoots.state_id(),
            NetherPlant::NetherSprouts.state_id(),
        ),
        (20_960, 21_031, 20_961)
    );
    for plant in [
        NetherPlant::WarpedRoots,
        NetherPlant::CrimsonRoots,
        NetherPlant::NetherSprouts,
    ] {
        assert!(plant.survives_on(13));
        assert!(!plant.survives_on(14));
    }
    assert!(NetherPlant::NetherSprouts.drops_item(true, false));
    assert!(!NetherPlant::NetherSprouts.drops_item(false, true));
    assert_eq!(
        compost(
            0,
            NetherPlant::NetherSprouts.composter_chance(),
            None,
            false
        ),
        ComposterResult::Inserted {
            new_level: 1,
            schedule: false,
        }
    );
    assert_eq!(crimson_vegetation_choice(98), VegetationChoice::Other);
    assert_eq!(
        warped_vegetation_choice(98),
        VegetationChoice::NetherSprouts
    );
}

#[test]
fn nether_stems_preserve_axis_and_nonflammable_identity_through_stripping() {
    assert_eq!(StemKind::WarpedStem.state_id(Axis::X), 20_945);
    assert_eq!(StemKind::WarpedStem.state_id(Axis::Y), 20_946);
    assert_eq!(StemKind::CrimsonHyphae.state_id(Axis::Z), 20_970);
    let stripped = strip(StemKind::CrimsonHyphae, Axis::X, false).unwrap();
    assert_eq!(stripped.target, StemKind::StrippedCrimsonHyphae);
    assert_eq!(stripped.axis, Axis::X);
    assert_eq!((stripped.flags, stripped.durability_cost), (11, 1));
    assert_eq!(strip(StemKind::CrimsonStem, Axis::Y, true), None);
    assert_eq!(StemKind::StrippedWarpedStem.burn_time(), 0);
}

#[test]
fn nether_wart_growth_loot_and_visual_stage_follow_four_server_ages() {
    let ages = (0..=3)
        .map(|age| NetherWartState::new(age).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        ages.iter()
            .map(|state| state.state_id())
            .collect::<Vec<_>>(),
        [9_447, 9_448, 9_449, 9_450]
    );
    assert_eq!(
        ages.iter()
            .map(|state| state.selection_height())
            .collect::<Vec<_>>(),
        [5, 8, 11, 14]
    );
    assert_eq!(
        ages.iter()
            .map(|state| state.client_stage())
            .collect::<Vec<_>>(),
        [0, 1, 1, 2]
    );
    assert_eq!(ages[2].random_tick(0).unwrap().age, 3);
    assert_eq!(ages[3].random_tick(0), None);
    assert_eq!(wart_loot_base(3, 2, 3), 7);
    assert_eq!(explosion_decay(5, &[true, false, true, false, true]), 3);
}

#[test]
fn both_wart_blocks_share_compost_and_tags_but_not_exact_spawn_vetoes() {
    assert_eq!(
        (
            WartBlockKind::Nether.state_id(),
            WartBlockKind::Nether.block_id(),
            WartBlockKind::Nether.item_id(),
        ),
        (14_846, 672, 604)
    );
    assert_eq!(
        (
            WartBlockKind::Warped.state_id(),
            WartBlockKind::Warped.block_id(),
            WartBlockKind::Warped.item_id(),
        ),
        (20_959, 868, 605)
    );
    assert!(WartBlockKind::Nether.vetoes_piglin_family_spawn());
    assert!(!WartBlockKind::Warped.vetoes_piglin_family_spawn());
    assert_eq!(
        compost(6, f64::from(0.85_f32), Some(0.85), true),
        ComposterResult::Inserted {
            new_level: 7,
            schedule: true,
        }
    );
    assert_eq!(
        compost(1, f64::from(0.85_f32), Some(0.9), false),
        ComposterResult::ConsumedWithoutInsert
    );
}

#[test]
fn sculk_sensor_selects_travels_activates_and_outputs_exactly() {
    assert_eq!(
        ordinary_state_id(0, SensorPhase::Inactive, true),
        Some(27_163)
    );
    assert_eq!(
        calibrated_state_id(Direction::East, 15, SensorPhase::Cooldown, false),
        Some(27_642)
    );
    assert_eq!(vibration_frequency("step"), 1);
    assert_eq!(vibration_frequency("resonate_15"), 15);
    assert_eq!(vibration_frequency("custom"), 0);
    let first = VibrationCandidate {
        tick: 10,
        exact_distance: 3.0,
        frequency: 1,
    };
    let higher = VibrationCandidate {
        frequency: 15,
        ..first
    };
    assert_eq!(select_candidate(Some(first), higher), Some(higher));
    assert!(!selection_ready(10, 10));
    assert!(selection_ready(10, 11));
    assert_eq!(travel_delay(3.99), 3);
    assert_eq!(arrival_power(8.0, 8), 1);
    assert!(calibrated_admits(0, 8));
    assert!(!calibrated_admits(7, 8));
    assert!(!chunks_admit_delivery([
        true, true, true, true, false, true, true, true, true
    ]));
    assert_eq!(activation(15, false, false).active_ticks, 30);
    assert_eq!(sensor_weak(15, Some(Direction::North), Direction::North), 0);
    assert_eq!(sensor_direct(15, Direction::Up), 15);
    assert_eq!(comparator_signal(SensorPhase::Cooldown, true, 12), 0);
    assert_eq!(resonance_note(15), Some(24));
}

fn clickable_text() -> SignText {
    let mut text = SignText::default();
    text.lines[0] = SignLine {
        text: "run".to_owned(),
        editable_literal: true,
        click: Some(ClickAction::RunCommand("say hi".to_owned())),
    };
    text
}

#[test]
fn signs_keep_two_sides_editor_lease_applicators_clicks_and_render_bounds() {
    assert_eq!(
        selected_side(0.0, 90.0),
        ferrite_gameplay::block::sign::SignSide::Front
    );
    assert_eq!(
        selected_side(0.0, 90.01),
        ferrite_gameplay::block::sign::SignSide::Back
    );
    let mut text = SignText::default();
    let mut waxed = false;
    assert_eq!(
        apply(&mut text, &mut waxed, Applicator::Honeycomb),
        ApplicatorResult::Changed {
            level_event: Some(3003),
        }
    );
    assert!(waxed);
    assert_eq!(
        empty_hand(&clickable_text(), true, true, true),
        EmptyHandResult::WaxedFailure
    );
    let mut lease = EditorLease { editor: None };
    lease.open(7);
    assert!(lease.can_mutate(7));
    assert!(!lease.can_mutate(8));
    lease.tick(true, false);
    assert_eq!(lease.editor, None);
    let mut edited = clickable_text();
    assert_eq!(
        commit_edit(&mut edited, false, true, ["a", "b", "c", "d"]),
        EditResult::Accepted { update_requests: 2 }
    );
    assert!(hanging_chain_precedence(false, true, true));
    assert_eq!(renderer_light(true, 0), 15_728_880);
    assert!(!outline_visible(false, false, 256.0));
}

#[test]
fn skulls_lock_state_ids_durable_power_animation_note_and_wither_order() {
    assert_eq!(SkullType::Skeleton.floor_state_id(0, false), Some(10_915));
    assert_eq!(
        SkullType::Piglin.wall_state_id(Direction::East, true),
        Some(11_194)
    );
    let update = neighbor_power(false, true, true);
    assert_eq!((update.powered, update.flags), (true, Some(2)));
    let animation = SkullAnimation {
        counter: 3,
        active: true,
    }
    .tick(false);
    assert_eq!(animation.sample(0.5), 3.0);
    assert!(dragon_jaw_rotation(1.0).is_finite());
    let ears = piglin_ear_rotations(1.0);
    assert!(ears.0 < 0.0 && ears.1 > 0.0);
    assert_eq!(floor_note_sound(SkullType::Player, false), NoteSound::None);
    assert!(matches!(
        floor_note_sound(SkullType::Player, true),
        NoteSound::Custom { volume: 3, .. }
    ));
    assert_eq!(
        summon_wither(WitherAdmission {
            server: true,
            correct_skull: true,
            at_or_above_minimum_y: true,
            peaceful: false,
            pattern_matches: true,
            entity_created: true,
        }),
        WitherResult::Created {
            cleared_cells: 9,
            break_events: 9,
            criterion_before_entity_admission: true,
            entity_admission_result_ignored: true,
            neighbor_updates: 9,
        }
    );
}

#[test]
fn test_blocks_separate_state_and_entity_modes_and_accept_precedes_fail() {
    assert_eq!(
        (0..=3)
            .map(TestBlockMode::from_wire_id)
            .map(TestBlockMode::state_id)
            .collect::<Vec<_>>(),
        [21_738, 21_739, 21_740, 21_741]
    );
    assert_eq!(TestBlockMode::from_wire_id(-1), TestBlockMode::Start);
    let mut log = TestBlockEntity::new(TestBlockMode::Log);
    assert!(log.neighbor_signal(true));
    assert!(log.triggered);
    assert!(!log.neighbor_signal(true));
    assert!(!log.neighbor_signal(false));
    assert!(!log.powered);
    assert_eq!(edit(&mut log, TestBlockMode::Fail, "failure"), EDIT_EFFECTS);
    assert_eq!(log.mode, TestBlockMode::Fail);
    let accept = TestBlockEntity {
        triggered: true,
        ..TestBlockEntity::new(TestBlockMode::Accept)
    };
    let fail = TestBlockEntity {
        message: "failure".to_owned(),
        triggered: true,
        ..TestBlockEntity::new(TestBlockMode::Fail)
    };
    assert_eq!(
        scan_outcomes(&[accept], &[fail], &[log]),
        ScanResult::Success
    );
    assert!(start_cardinality_valid(1));
    assert!(!start_cardinality_valid(0));
    assert_eq!(
        TestBlockEntity::load(None, None, None).mode,
        TestBlockMode::Fail
    );
    let bounded = format!("{}😀", "a".repeat(127));
    assert_eq!(truncate_ui_message(&bounded), "a".repeat(127));
}

#[test]
fn honeycomb_directly_shrinks_before_ignored_write_and_duplicates_chest_effects() {
    let unrelated = use_honeycomb(false, true, 3, None);
    assert!(!unrelated.mapped);
    assert!(unrelated.effects.is_empty());
    let waxed = use_honeycomb(true, true, 1, Some(CopperChestType::Left));
    assert_eq!(waxed.remaining, 0);
    assert_eq!(
        waxed.effects,
        [
            WaxEffect::Criterion,
            WaxEffect::Shrink,
            WaxEffect::Write { flags: 11 },
            WaxEffect::BlockChange { companion: false },
            WaxEffect::LevelEvent { companion: false },
            WaxEffect::BlockChange { companion: true },
            WaxEffect::LevelEvent { companion: true },
        ]
    );
    let infinite = use_honeycomb(true, false, 64, Some(CopperChestType::Single));
    assert_eq!(infinite.remaining, 63);
    assert_eq!(infinite.effects.len(), 4);
}
