use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_gameplay::redstone::delay::orientation::ORIENTATION_BOUND;
use ferrite_gameplay::redstone::piston::execution::{
    CLEAR_SHAPE_ORDER, CLEARED_SOURCE_WRITE_FLAGS, ClearShapeStage, DESTROY_AIR_WRITE_FLAGS,
    DESTROY_NOTIFICATION_ORDER, DestroyNotificationStage, EXTENDED_BASE_WRITE_FLAGS,
    EXTENSION_ORDER, EventRevalidation, ExtensionStage, MOVING_DESTINATION_WRITE_FLAGS,
    RETRACTING_BASE_WRITE_FLAGS, RETRACTION_ORDER, RetractionStage, SOUND_VOLUME,
    extension_completion, extension_pitch, movement_execution_plan, retraction_pitch,
    retraction_plan, revalidate_event,
};
use ferrite_gameplay::redstone::piston::moving::{
    AIR_FALLBACK_WRITE_FLAGS, CLIENT_DEATH_TICKS, COLLISION_PADDING, CollisionEntityInput,
    CompletionObservation, CompletionWrite, FORCED_FINAL_WRITE_FLAGS, ForcedFinalState,
    MOVEMENT_AREA_CONSTANT, MOVING_BLOCK_CONTRACT, MovingBlockUse, MovingProgress,
    NORMAL_FINAL_WRITE_FLAGS, PROGRESS_STEP, STICKY_TOP_MAX_Y, UPDATE_OR_DESTROY_FLAGS,
    base_ejection_displacement, collided_entity_plan, collision_displacement,
    destroy_removes_extended_base, drops_carried_state, forced_final_tick, honey_carries_entity,
    honey_displacement, moving_tick, use_moving_block,
};
use ferrite_gameplay::redstone::piston::power::{
    CheckCause, MovingAhead, PistonEvent, PistonState, PowerProbe, neighbor_power, placement_state,
    should_check_extension, transition_check,
};
use ferrite_gameplay::redstone::piston::resolver::{
    MAX_PUSH_DEPTH, PistonBlock, PistonBlockKind, PushReaction, ResolvedStructure, ResolverWorld,
    can_stick, is_pushable, resolve_structure,
};

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}

fn block(kind: PistonBlockKind, reaction: PushReaction) -> PistonBlock {
    PistonBlock {
        kind,
        reaction,
        destroy_speed: 1.0,
        has_block_entity: false,
    }
}

fn moving(facing: Direction, progress: f32, last_ticked: u64) -> MovingAhead {
    MovingAhead {
        is_moving_piston: true,
        facing,
        extending: true,
        progress,
        last_ticked,
    }
}

#[test]
fn piston_power_probes_direct_self_down_and_above_positions_in_source_order() {
    let none = [false; 6];
    let result = neighbor_power(Direction::North, none, false, none);
    assert!(!result.powered);
    assert_eq!(
        result.probes,
        [
            PowerProbe::Adjacent(Direction::Down),
            PowerProbe::Adjacent(Direction::Up),
            PowerProbe::Adjacent(Direction::South),
            PowerProbe::Adjacent(Direction::West),
            PowerProbe::Adjacent(Direction::East),
            PowerProbe::PistonTowardDown,
            PowerProbe::AboveAdjacent(Direction::Up),
            PowerProbe::AboveAdjacent(Direction::North),
            PowerProbe::AboveAdjacent(Direction::South),
            PowerProbe::AboveAdjacent(Direction::West),
            PowerProbe::AboveAdjacent(Direction::East),
        ]
    );

    let mut direct = none;
    direct[4] = true;
    let result = neighbor_power(Direction::North, direct, true, [true; 6]);
    assert!(result.powered);
    assert_eq!(
        result.probes.last(),
        Some(&PowerProbe::Adjacent(Direction::West))
    );

    let result = neighbor_power(Direction::North, none, true, [true; 6]);
    assert_eq!(result.probes.last(), Some(&PowerProbe::PistonTowardDown));
    let mut above = none;
    above[5] = true;
    assert_eq!(
        neighbor_power(Direction::North, none, false, above)
            .probes
            .last(),
        Some(&PowerProbe::AboveAdjacent(Direction::East))
    );
}

#[test]
fn extension_checks_resolve_before_event_and_fast_retraction_selects_drop() {
    assert_eq!(
        PistonState::default_state(true),
        PistonState {
            facing: Direction::North,
            extended: false,
            sticky: true,
        }
    );
    assert_eq!(
        placement_state(false, Direction::Up),
        PistonState {
            facing: Direction::Down,
            extended: false,
            sticky: false,
        }
    );
    let state = PistonState::default_state(false);
    let failed = transition_check(state, true, false, None, 9, false);
    assert!(failed.extension_plan_resolved);
    assert_eq!(failed.queued_event, None);
    assert_eq!(
        transition_check(state, true, true, None, 9, false).queued_event,
        Some(PistonEvent::Extend)
    );

    let extended = PistonState {
        extended: true,
        ..state
    };
    assert_eq!(
        transition_check(
            extended,
            false,
            false,
            Some(moving(Direction::North, 0.49, 1)),
            9,
            false,
        )
        .queued_event,
        Some(PistonEvent::Drop)
    );
    assert_eq!(
        transition_check(
            extended,
            false,
            false,
            Some(moving(Direction::North, 0.5, 9)),
            9,
            false,
        )
        .queued_event,
        Some(PistonEvent::Drop)
    );
    assert_eq!(
        transition_check(
            extended,
            false,
            false,
            Some(moving(Direction::North, 0.5, 1)),
            9,
            true,
        )
        .queued_event,
        Some(PistonEvent::Drop)
    );
    assert_eq!(
        transition_check(
            extended,
            false,
            false,
            Some(moving(Direction::North, 0.5, 1)),
            9,
            false,
        )
        .queued_event,
        Some(PistonEvent::Contract)
    );
}

#[test]
fn piston_callback_gates_do_not_manufacture_missing_neighbor_updates() {
    assert!(should_check_extension(true, CheckCause::PlacedBy));
    assert!(should_check_extension(true, CheckCause::NeighborChanged));
    assert!(!should_check_extension(false, CheckCause::NeighborChanged));
    assert!(!should_check_extension(
        true,
        CheckCause::OnPlace {
            same_block_identity: true,
            has_block_entity: false,
        }
    ));
    assert!(!should_check_extension(
        true,
        CheckCause::OnPlace {
            same_block_identity: false,
            has_block_entity: true,
        }
    ));
    assert!(should_check_extension(
        true,
        CheckCause::OnPlace {
            same_block_identity: false,
            has_block_entity: false,
        }
    ));
}

#[test]
fn event_revalidation_cancels_stale_edges_before_rng_or_world_work() {
    assert_eq!(
        revalidate_event(true, true, PistonEvent::Contract),
        EventRevalidation::RestoreExtendedAndCancel { write_flags: 2 }
    );
    assert_eq!(
        revalidate_event(true, true, PistonEvent::Drop),
        EventRevalidation::RestoreExtendedAndCancel { write_flags: 2 }
    );
    assert_eq!(
        revalidate_event(true, false, PistonEvent::Extend),
        EventRevalidation::CancelWithoutWrite
    );
    assert_eq!(
        revalidate_event(true, true, PistonEvent::Extend),
        EventRevalidation::Execute
    );
    assert_eq!(
        revalidate_event(false, false, PistonEvent::Extend),
        EventRevalidation::Execute
    );
}

#[test]
fn pushability_rejects_bounds_exclusions_extended_pistons_and_block_entities() {
    let mut world = ResolverWorld::new(0, 10);
    let ordinary = PistonBlock::ordinary(PushReaction::Normal);
    assert!(is_pushable(
        PistonBlock::AIR,
        &world,
        pos(0, 0, 0),
        Direction::East,
        false,
        Direction::East
    ));
    for kind in [
        PistonBlockKind::Obsidian,
        PistonBlockKind::CryingObsidian,
        PistonBlockKind::RespawnAnchor,
        PistonBlockKind::ReinforcedDeepslate,
        PistonBlockKind::Piston { extended: true },
        PistonBlockKind::StickyPiston { extended: true },
    ] {
        assert!(!is_pushable(
            block(kind, PushReaction::Normal),
            &world,
            pos(0, 5, 0),
            Direction::East,
            true,
            Direction::East
        ));
    }
    assert!(!is_pushable(
        ordinary,
        &world,
        pos(0, 0, 0),
        Direction::Down,
        false,
        Direction::Down
    ));
    assert!(!is_pushable(
        ordinary,
        &world,
        pos(0, 10, 0),
        Direction::Up,
        false,
        Direction::Up
    ));
    world.mark_outside_border(pos(1, 5, 0));
    assert!(!is_pushable(
        ordinary,
        &world,
        pos(1, 5, 0),
        Direction::East,
        false,
        Direction::East
    ));
    let unbreakable = PistonBlock {
        destroy_speed: -1.0,
        ..ordinary
    };
    assert!(!is_pushable(
        unbreakable,
        &world,
        pos(0, 5, 0),
        Direction::East,
        false,
        Direction::East
    ));
    let entity = PistonBlock {
        has_block_entity: true,
        ..ordinary
    };
    assert!(!is_pushable(
        entity,
        &world,
        pos(0, 5, 0),
        Direction::East,
        false,
        Direction::East
    ));
}

#[test]
fn push_reactions_keep_destroy_permission_and_connection_direction_distinct() {
    let world = ResolverWorld::new(-64, 320);
    let position = pos(1, 0, 0);
    let destroy = PistonBlock::ordinary(PushReaction::Destroy);
    assert!(!is_pushable(
        destroy,
        &world,
        position,
        Direction::East,
        false,
        Direction::East
    ));
    assert!(is_pushable(
        destroy,
        &world,
        position,
        Direction::East,
        true,
        Direction::East
    ));
    let push_only = PistonBlock::ordinary(PushReaction::PushOnly);
    assert!(is_pushable(
        push_only,
        &world,
        position,
        Direction::East,
        false,
        Direction::East
    ));
    assert!(!is_pushable(
        push_only,
        &world,
        position,
        Direction::East,
        false,
        Direction::North
    ));
    assert!(!is_pushable(
        PistonBlock::ordinary(PushReaction::Block),
        &world,
        position,
        Direction::East,
        true,
        Direction::East
    ));
}

#[test]
fn resolver_accepts_air_single_destroy_and_rejects_thirteen_without_mutation() {
    let piston = pos(0, 0, 0);
    let empty = ResolverWorld::new(-64, 320);
    let resolved = resolve_structure(&empty, piston, Direction::East, true).unwrap();
    assert!(resolved.to_push.is_empty());
    assert!(resolved.to_destroy.is_empty());

    let mut destroy_world = ResolverWorld::new(-64, 320);
    destroy_world.insert(pos(1, 0, 0), PistonBlock::ordinary(PushReaction::Destroy));
    let resolved = resolve_structure(&destroy_world, piston, Direction::East, true).unwrap();
    assert_eq!(resolved.to_destroy, [pos(1, 0, 0)]);
    let mut retract_destroy = ResolverWorld::new(-64, 320);
    retract_destroy.insert(pos(2, 0, 0), PistonBlock::ordinary(PushReaction::Destroy));
    assert!(resolve_structure(&retract_destroy, piston, Direction::East, false).is_none());

    let mut twelve = ResolverWorld::new(-64, 320);
    for x in 1..=MAX_PUSH_DEPTH as i32 {
        twelve.insert(pos(x, 0, 0), PistonBlock::ordinary(PushReaction::Normal));
    }
    assert_eq!(
        resolve_structure(&twelve, piston, Direction::East, true)
            .unwrap()
            .to_push
            .len(),
        MAX_PUSH_DEPTH
    );
    twelve.insert(
        pos(MAX_PUSH_DEPTH as i32 + 1, 0, 0),
        PistonBlock::ordinary(PushReaction::Normal),
    );
    assert!(resolve_structure(&twelve, piston, Direction::East, true).is_none());
}

#[test]
fn resolver_sticky_backward_and_perpendicular_branches_keep_direction_order() {
    let piston = pos(0, 0, 0);
    let slime = block(PistonBlockKind::Slime, PushReaction::Normal);
    let honey = block(PistonBlockKind::Honey, PushReaction::Normal);
    assert!(can_stick(
        slime,
        PistonBlock::ordinary(PushReaction::Normal)
    ));
    assert!(!can_stick(slime, honey));
    assert!(!can_stick(honey, slime));

    let mut world = ResolverWorld::new(-64, 320);
    world.insert(pos(1, 0, 0), slime);
    world.insert(pos(1, -1, 0), PistonBlock::ordinary(PushReaction::Normal));
    world.insert(pos(1, 1, 0), PistonBlock::ordinary(PushReaction::Normal));
    world.insert(pos(1, 0, -1), honey);
    world.insert(pos(1, 0, 1), PistonBlock::ordinary(PushReaction::Normal));
    let resolved = resolve_structure(&world, piston, Direction::East, true).unwrap();
    assert_eq!(
        resolved.to_push,
        [pos(1, 0, 0), pos(1, -1, 0), pos(1, 1, 0), pos(1, 0, 1),]
    );
}

#[test]
fn execution_snapshots_then_destroys_moves_clears_and_notifies_in_distinct_orders() {
    let mut world = ResolverWorld::new(-64, 320);
    let first = PistonBlock::ordinary(PushReaction::Normal);
    let second = block(PistonBlockKind::Slime, PushReaction::Normal);
    let destroyed = PistonBlock::ordinary(PushReaction::Destroy);
    world.insert(pos(1, 0, 0), first);
    world.insert(pos(2, 0, 0), second);
    world.insert(pos(3, 0, 0), destroyed);
    let resolved = ResolvedStructure {
        push_direction: Direction::East,
        to_push: vec![pos(1, 0, 0), pos(2, 0, 0)],
        to_destroy: vec![pos(3, 0, 0)],
    };
    let plan = movement_execution_plan(
        &world,
        pos(0, 0, 0),
        Direction::East,
        true,
        false,
        &resolved,
        true,
    )
    .unwrap();
    assert_eq!(plan.destroy_reverse[0].position, pos(3, 0, 0));
    assert_eq!(plan.destroy_reverse[0].write_flags, DESTROY_AIR_WRITE_FLAGS);
    assert_eq!(
        plan.move_reverse
            .iter()
            .map(|step| (step.source, step.destination, step.snapshot.kind))
            .collect::<Vec<_>>(),
        [
            (pos(2, 0, 0), pos(3, 0, 0), PistonBlockKind::Slime),
            (pos(1, 0, 0), pos(2, 0, 0), PistonBlockKind::Other),
        ]
    );
    assert!(
        plan.move_reverse
            .iter()
            .all(|step| step.write_flags == MOVING_DESTINATION_WRITE_FLAGS)
    );
    assert_eq!(plan.extension_head, Some(pos(1, 0, 0)));
    assert!(plan.clear_sources_unordered.is_empty());
    assert_eq!(plan.clear_write_flags, CLEARED_SOURCE_WRITE_FLAGS);
    assert_eq!(
        plan.push_notifications_reverse,
        [pos(2, 0, 0), pos(1, 0, 0)]
    );
    assert_eq!(plan.destroy_updates_reverse, [pos(3, 0, 0)]);
    assert_eq!(
        CLEAR_SHAPE_ORDER,
        [
            ClearShapeStage::SourceIndirect,
            ClearShapeStage::AirNeighbor,
            ClearShapeStage::AirIndirect,
        ]
    );
    assert_eq!(
        DESTROY_NOTIFICATION_ORDER,
        [
            DestroyNotificationStage::RemovalHook,
            DestroyNotificationStage::SourceIndirectShape,
            DestroyNotificationStage::OrientedNeighbors,
        ]
    );
    assert_eq!(plan.orientation.bound, Some(ORIENTATION_BOUND));
    assert_eq!(plan.orientation.fixed_front, Some(Direction::East));
}

#[test]
fn retraction_preclear_and_sticky_pull_keep_event_one_two_and_piece_edges_separate() {
    let mut world = ResolverWorld::new(-64, 320);
    world.insert(pos(2, 0, 0), PistonBlock::ordinary(PushReaction::Normal));
    let resolved = ResolvedStructure {
        push_direction: Direction::West,
        to_push: vec![pos(2, 0, 0)],
        to_destroy: vec![],
    };
    let execution = movement_execution_plan(
        &world,
        pos(0, 0, 0),
        Direction::East,
        false,
        true,
        &resolved,
        false,
    )
    .unwrap();
    assert_eq!(execution.preclear_retraction_head, Some(pos(1, 0, 0)));
    assert_eq!(execution.preclear_flags, Some(RETRACTING_BASE_WRITE_FLAGS));
    assert_eq!(execution.move_reverse[0].destination, pos(1, 0, 0));

    let pull = retraction_plan(
        &world,
        pos(0, 0, 0),
        Direction::East,
        true,
        PistonEvent::Contract,
        true,
        None,
    )
    .unwrap();
    assert!(pull.finalize_head_entity);
    assert!(pull.start_fresh_pull);
    assert!(!pull.remove_head);
    assert_eq!(pull.base_write_flags, RETRACTING_BASE_WRITE_FLAGS);
    assert_eq!(pull.order, RETRACTION_ORDER);

    let drop = retraction_plan(
        &world,
        pos(0, 0, 0),
        Direction::East,
        true,
        PistonEvent::Drop,
        false,
        None,
    )
    .unwrap();
    assert!(!drop.start_fresh_pull);
    assert!(drop.remove_head);

    let piece = retraction_plan(
        &world,
        pos(0, 0, 0),
        Direction::East,
        true,
        PistonEvent::Drop,
        false,
        Some(moving(Direction::East, 0.8, 1)),
    )
    .unwrap();
    assert!(piece.finalize_compatible_two_ahead);
    assert!(!piece.start_fresh_pull);
    assert!(!piece.remove_head);
    assert_eq!(
        RETRACTION_ORDER,
        [
            RetractionStage::FinalizeHeadMovingEntity,
            RetractionStage::WriteRetractingBase,
            RetractionStage::InstallSourceMovingEntity,
            RetractionStage::UpdateBaseNeighbors,
            RetractionStage::UpdateBaseShapes,
            RetractionStage::StickyOrDefaultHeadHandling,
            RetractionStage::PlaySound,
            RetractionStage::EmitBlockDeactivate,
        ]
    );
}

#[test]
fn successful_sound_pitch_draws_are_post_write_and_use_distinct_ranges() {
    assert_eq!(extension_completion(false), None);
    let completion = extension_completion(true).unwrap();
    assert_eq!(completion.base_write_flags, EXTENDED_BASE_WRITE_FLAGS);
    assert!(completion.sound_draw_consumed_after_writes);
    assert_eq!(
        completion.order,
        [
            ExtensionStage::MoveBlocks,
            ExtensionStage::WriteExtendedBase,
            ExtensionStage::PlaySound,
            ExtensionStage::EmitBlockActivate,
        ]
    );
    assert_eq!(completion.order, EXTENSION_ORDER);
    assert_eq!(extension_pitch(0.0), 0.6);
    assert_eq!(extension_pitch(1.0), 0.85);
    assert_eq!(retraction_pitch(0.0), 0.6);
    assert_eq!(retraction_pitch(1.0), 0.75);
    assert_eq!(std::hint::black_box(SOUND_VOLUME), 0.5);
    assert_eq!(std::hint::black_box(EXTENDED_BASE_WRITE_FLAGS), 67);
}

#[test]
fn moving_progress_advances_half_steps_then_finalizes_on_the_following_tick() {
    let observation = CompletionObservation {
        client: false,
        live_state_is_moving_piston: true,
        adjusted_carried_is_air: false,
        adjusted_carried_waterlogged: true,
        redstone_experiments: true,
    };
    let initial = MovingProgress {
        progress: 0.0,
        previous_progress: 0.0,
        death_ticks: 0,
        direction: Direction::East,
        extending: true,
        source_piston: false,
    };
    let first = moving_tick(initial, observation, false);
    assert_eq!(first.new_progress, 0.5);
    assert_eq!(first.collision_delta, Some(PROGRESS_STEP));
    assert!(first.move_collided_entities);
    let second = moving_tick(
        MovingProgress {
            progress: first.new_progress,
            ..initial
        },
        observation,
        false,
    );
    assert_eq!(second.new_progress, 1.0);
    assert!(!second.remove_block_entity);
    let finalization = moving_tick(
        MovingProgress {
            progress: 1.0,
            previous_progress: 0.5,
            ..initial
        },
        observation,
        false,
    );
    assert!(finalization.remove_block_entity);
    assert_eq!(
        finalization.completion_write,
        CompletionWrite::AdjustedCarried {
            clear_waterlogged: true,
            write_flags: NORMAL_FINAL_WRITE_FLAGS,
        }
    );
    assert!(finalization.notify_completed_state);
    assert_eq!(finalization.orientation.bound, Some(ORIENTATION_BOUND));
}

#[test]
fn moving_completion_keeps_air_fallback_and_five_client_linger_ticks() {
    let state = MovingProgress {
        progress: 1.0,
        previous_progress: 1.0,
        death_ticks: 4,
        direction: Direction::North,
        extending: false,
        source_piston: false,
    };
    let client = moving_tick(
        state,
        CompletionObservation {
            client: true,
            live_state_is_moving_piston: true,
            adjusted_carried_is_air: true,
            adjusted_carried_waterlogged: false,
            redstone_experiments: false,
        },
        false,
    );
    assert_eq!(
        client.incremented_client_death_ticks,
        Some(CLIENT_DEATH_TICKS)
    );
    assert!(!client.remove_block_entity);

    let server = moving_tick(
        state,
        CompletionObservation {
            client: false,
            live_state_is_moving_piston: true,
            adjusted_carried_is_air: true,
            adjusted_carried_waterlogged: false,
            redstone_experiments: false,
        },
        false,
    );
    assert_eq!(
        server.completion_write,
        CompletionWrite::RestoreMovedThenUpdateOrDestroy {
            first_flags: AIR_FALLBACK_WRITE_FLAGS,
            update_flags: UPDATE_OR_DESTROY_FLAGS,
        }
    );
    assert!(!server.notify_completed_state);
}

#[test]
fn collision_slime_honey_and_base_ejection_preserve_exact_motion_boundaries() {
    assert_eq!(
        collision_displacement([0.2, 0.7, 0.4], 0.5),
        Some(0.5 + COLLISION_PADDING)
    );
    assert_eq!(collision_displacement([0.0, -1.0], 0.5), None);
    let slime = collided_entity_plan(
        CollisionEntityInput {
            reaction: PushReaction::Normal,
            server_player: false,
            moved_block_is_slime: true,
            movement: Direction::West,
            velocity: [0.2, 0.3, 0.4],
            delta_progress: 0.5,
            retracting_source: false,
        },
        [0.25],
    );
    assert_eq!(slime.velocity, [-1.0, 0.3, 0.4]);
    assert_eq!(slime.displacement, Some(0.26));
    assert!(slime.apply_block_effects);
    assert!(slime.remove_latest_movement_record);
    let player = collided_entity_plan(
        CollisionEntityInput {
            reaction: PushReaction::Normal,
            server_player: true,
            moved_block_is_slime: true,
            movement: Direction::Up,
            velocity: [0.2, 0.3, 0.4],
            delta_progress: 0.5,
            retracting_source: true,
        },
        [0.25],
    );
    assert_eq!(player.velocity, [0.2, 0.3, 0.4]);
    assert!(player.eject_from_retracting_source);
    assert!(
        collided_entity_plan(
            CollisionEntityInput {
                reaction: PushReaction::Ignore,
                server_player: false,
                moved_block_is_slime: true,
                movement: Direction::Up,
                velocity: [0.0; 3],
                delta_progress: 0.5,
                retracting_source: true,
            },
            [1.0],
        )
        .ignored
    );

    assert!(honey_carries_entity(
        Direction::East,
        PushReaction::Normal,
        true,
        false,
        true,
        true,
    ));
    assert!(!honey_carries_entity(
        Direction::Up,
        PushReaction::Normal,
        true,
        true,
        true,
        true,
    ));
    assert_eq!(honey_displacement(0.5), 0.5);
    assert_eq!(
        base_ejection_displacement(0.2, 0.195, 0.5),
        Some(0.22000000000000003)
    );
    assert_eq!(base_ejection_displacement(0.2, 0.18, 0.5), None);
    assert_eq!(std::hint::black_box(STICKY_TOP_MAX_Y), 1.5000010000000001);
    assert_eq!(std::hint::black_box(MOVEMENT_AREA_CONSTANT), 0.51);
}

#[test]
fn forced_finalization_and_moving_block_hooks_preserve_source_and_missing_entity_quirks() {
    let source = forced_final_tick(true, false, 0.5, true, Direction::West, true, true);
    assert_eq!(source.target, ForcedFinalState::Air);
    assert_eq!(source.write_flags, Some(FORCED_FINAL_WRITE_FLAGS));
    assert!(source.remove_block_entity);
    assert!(source.notify);
    assert_eq!(source.orientation.bound, Some(ORIENTATION_BOUND));
    let carried = forced_final_tick(true, false, 0.5, false, Direction::West, true, false);
    assert_eq!(carried.target, ForcedFinalState::AdjustedCarried);
    let stale = forced_final_tick(true, false, 1.0, false, Direction::West, true, false);
    assert_eq!(stale.target, ForcedFinalState::NoOp);
    assert!(!stale.remove_block_entity);
    let missing_live = forced_final_tick(true, true, 1.0, false, Direction::West, false, false);
    assert_eq!(missing_live.target, ForcedFinalState::NoOp);
    assert!(missing_live.remove_block_entity);
    assert!(destroy_removes_extended_base(true, true));
    assert!(!destroy_removes_extended_base(true, false));
    assert_eq!(
        use_moving_block(true, false),
        MovingBlockUse::ConsumeAndRemove
    );
    assert_eq!(use_moving_block(false, false), MovingBlockUse::Pass);
    assert!(drops_carried_state(true));
    assert!(!drops_carried_state(false));
    let contract = std::hint::black_box(MOVING_BLOCK_CONTRACT);
    assert!(contract.render_invisible);
    assert!(contract.outline_empty);
    assert!(contract.clone_item_empty);
    assert!(!contract.pathfindable);
    assert!(contract.ordinary_block_entity_factory_returns_none);
}
