use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_gameplay::player::breaking::input::{
    HeldAttackContext, HeldAttackDecision, HeldBlockHit, decide_held_attack,
};
use ferrite_gameplay::player::breaking::mutation::{
    LocalDestroyContext, LocalDestroyEffect, plan_local_destroy,
};
use ferrite_gameplay::player::breaking::prediction::{
    ClientPredictionClock, PredictionError, PredictionResolution, RetainedPrediction,
    ServerPredictionAck,
};
use ferrite_gameplay::player::breaking::session::{
    ClientBreakSession, ContinueBreakContext, StartBreakContext,
};
use ferrite_gameplay::player::breaking::{
    BreakingItem, ClientBreakEffect, PlayerAction, TargetState,
};
use ferrite_gameplay::player::state::Vec3;
use ferrite_world::id::BlockStateId;

fn position(x: i32) -> BlockPos {
    BlockPos::new(x, 64, 0)
}

fn item(object_id: u64, count: u32, components_fingerprint: u64) -> BreakingItem {
    BreakingItem {
        object_id,
        item_id: 5,
        components_fingerprint,
        count,
    }
}

fn target(progress: f32) -> TargetState {
    TargetState {
        is_air: false,
        destroy_progress: progress,
        sound_volume: 1.0,
        sound_pitch: 0.8,
    }
}

fn start_context(position: BlockPos, item: BreakingItem, progress: f32) -> StartBreakContext {
    StartBreakContext {
        position,
        face: Direction::North,
        item,
        target: target(progress),
        action_restricted: false,
        inside_world_border: true,
        instabuild: false,
    }
}

#[test]
fn held_input_suppresses_same_tick_continuation_and_preserves_early_return_asymmetry() {
    let hit = HeldBlockHit {
        position: position(0),
        face: Direction::North,
        is_air: false,
    };
    let common = HeldAttackContext {
        click_succeeded_this_tick: false,
        miss_delay_positive: false,
        using_item: false,
        piercing_weapon: false,
        screen_open: false,
        attack_held: true,
        mouse_captured: true,
        hit: Some(hit),
    };
    assert_eq!(
        decide_held_attack(HeldAttackContext {
            click_succeeded_this_tick: true,
            ..common
        }),
        HeldAttackDecision::SuppressedByClick
    );
    assert_eq!(
        decide_held_attack(HeldAttackContext {
            using_item: true,
            attack_held: false,
            ..common
        }),
        HeldAttackDecision::EarlyReturn
    );
    assert_eq!(
        decide_held_attack(HeldAttackContext {
            screen_open: true,
            ..common
        }),
        HeldAttackDecision::Stop
    );
    assert_eq!(
        decide_held_attack(HeldAttackContext {
            hit: Some(HeldBlockHit {
                is_air: true,
                ..hit
            }),
            ..common
        }),
        HeldAttackDecision::AirWithoutStop
    );
    assert_eq!(
        decide_held_attack(common),
        HeldAttackDecision::Continue(hit)
    );
}

#[test]
fn restricted_replacement_keeps_old_session_and_new_target_abort_uses_new_face() {
    let held = item(1, 1, 9);
    let mut session = ClientBreakSession::default();
    session.start(start_context(position(0), held, 0.2));
    let old = session;

    let rejected = session.start(StartBreakContext {
        position: position(1),
        action_restricted: true,
        ..start_context(position(1), held, 0.2)
    });
    assert!(!rejected.continued);
    assert_eq!(session, old);

    let replacement = session.start(StartBreakContext {
        face: Direction::East,
        ..start_context(position(1), held, 0.2)
    });
    assert_eq!(
        replacement.effects.first(),
        Some(&ClientBreakEffect::SendAction {
            action: PlayerAction::AbortDestroyBlock,
            position: position(0),
            face: Direction::East,
            sequence: 0,
        })
    );
    assert_eq!(session.destroy_position, position(1));
}

#[test]
fn start_orders_callback_prediction_and_packet_while_instant_paths_leave_no_new_record() {
    let held = item(1, 1, 9);
    let mut session = ClientBreakSession::default();
    let ordinary = session.start(start_context(position(0), held, 0.25));
    assert_eq!(
        ordinary.effects,
        vec![
            ClientBreakEffect::TutorialProgress {
                position: position(0),
                progress: 0.0,
            },
            ClientBreakEffect::BeginPrediction(1),
            ClientBreakEffect::AttackBlock(position(0)),
            ClientBreakEffect::PublishCrack {
                position: position(0),
                stage: -1,
            },
            ClientBreakEffect::SendAction {
                action: PlayerAction::StartDestroyBlock,
                position: position(0),
                face: Direction::North,
                sequence: 1,
            },
            ClientBreakEffect::EndPrediction(1),
        ]
    );

    let instant = session.start(start_context(position(1), held, 1.0));
    assert!(
        instant
            .effects
            .contains(&ClientBreakEffect::AttemptLocalDestroy(position(1)))
    );
    assert!(session.is_destroying);
    assert_eq!(session.destroy_position, position(0));

    let mut creative = ClientBreakSession::default();
    let creative_result = creative.start(StartBreakContext {
        instabuild: true,
        ..start_context(position(2), held, 0.0)
    });
    assert_eq!(creative.destroy_delay, 5);
    assert!(!creative.is_destroying);
    assert!(matches!(
        creative_result.effects.as_slice(),
        [
            ClientBreakEffect::TutorialProgress { progress: 1.0, .. },
            ClientBreakEffect::BeginPrediction(1),
            ClientBreakEffect::AttemptLocalDestroy(_),
            ClientBreakEffect::SendAction {
                action: PlayerAction::StartDestroyBlock,
                sequence: 1,
                ..
            },
            ClientBreakEffect::EndPrediction(1)
        ]
    ));
}

#[test]
fn delay_precedes_validation_and_item_matching_ignores_count_or_same_object_mutation() {
    let original = item(1, 1, 9);
    let mut session = ClientBreakSession::default();
    session.start(start_context(position(0), original, 0.1));
    session.destroy_delay = 2;
    let delayed = session.continue_break(ContinueBreakContext {
        start: StartBreakContext {
            position: position(99),
            inside_world_border: false,
            ..start_context(position(99), item(2, 1, 2), 0.1)
        },
        selected_slot_changed: true,
    });
    assert_eq!(delayed.effects, vec![ClientBreakEffect::SendCarriedSlot]);
    assert_eq!(session.destroy_delay, 1);

    session.destroy_delay = 0;
    let count_only = session.continue_break(ContinueBreakContext {
        start: start_context(position(0), item(2, 64, 9), 0.1),
        selected_slot_changed: false,
    });
    assert!(count_only.effects.iter().any(|effect| matches!(
        effect,
        ClientBreakEffect::PublishCrack {
            position: block,
            ..
        } if *block == position(0)
    )));

    let same_object_mutated = original.same_item_and_components(item(original.object_id, 1, 500));
    assert!(same_object_mutated);
}

#[test]
fn continuation_uses_historical_float_progress_sound_cadence_and_nan_completion() {
    let held = item(1, 1, 9);
    let mut session = ClientBreakSession::default();
    session.start(start_context(position(0), held, 0.1));
    let first = session.continue_break(ContinueBreakContext {
        start: start_context(position(0), held, 0.25),
        selected_slot_changed: false,
    });
    assert!(first.effects.contains(&ClientBreakEffect::PlayHitSound {
        position: position(0),
        volume: 0.25,
        pitch: 0.4,
    }));
    assert!(first.effects.contains(&ClientBreakEffect::PublishCrack {
        position: position(0),
        stage: 2,
    }));

    session.destroy_progress = f32::NAN;
    let completed = session.continue_break(ContinueBreakContext {
        start: start_context(position(0), held, 0.0),
        selected_slot_changed: false,
    });
    assert!(!session.is_destroying);
    assert_eq!(session.destroy_delay, 5);
    assert!(completed.effects.iter().any(|effect| matches!(
        effect,
        ClientBreakEffect::SendAction {
            action: PlayerAction::StopDestroyBlock,
            ..
        }
    )));
}

#[test]
fn explicit_stop_uses_down_and_leaves_ticks_delay_target_and_item_intact() {
    let held = item(1, 1, 9);
    let mut session = ClientBreakSession::default();
    session.start(start_context(position(0), held, 0.1));
    session.destroy_ticks = 7.0;
    session.destroy_delay = 3;
    let stopped = session.stop();
    assert_eq!(
        stopped.effects[1],
        ClientBreakEffect::SendAction {
            action: PlayerAction::AbortDestroyBlock,
            position: position(0),
            face: Direction::Down,
            sequence: 0,
        }
    );
    assert!(!session.is_destroying);
    assert_eq!(session.destroy_progress, 0.0);
    assert_eq!(session.destroy_ticks, 7.0);
    assert_eq!(session.destroy_delay, 3);
    assert_eq!(session.destroy_position, position(0));
    assert_eq!(session.destroying_item, Some(held));
}

#[test]
fn local_destroy_repeats_gates_and_reads_fluid_after_player_callback() {
    let denied = plan_local_destroy(LocalDestroyContext {
        position: position(0),
        action_restricted: false,
        item_allows_destroy: false,
        game_master_allows_destroy: true,
        is_air: false,
        fluid_legacy_state: 2,
        write_succeeds: true,
    });
    assert!(!denied.destroyed);
    assert_eq!(
        denied.effects,
        vec![
            LocalDestroyEffect::AdventureCheck,
            LocalDestroyEffect::ItemDestroyCheck
        ]
    );

    let accepted = plan_local_destroy(LocalDestroyContext {
        position: position(0),
        action_restricted: false,
        item_allows_destroy: true,
        game_master_allows_destroy: true,
        is_air: false,
        fluid_legacy_state: 2,
        write_succeeds: true,
    });
    assert_eq!(
        accepted.effects,
        vec![
            LocalDestroyEffect::AdventureCheck,
            LocalDestroyEffect::ItemDestroyCheck,
            LocalDestroyEffect::GameMasterCheck,
            LocalDestroyEffect::PlayerWillDestroy(position(0)),
            LocalDestroyEffect::ReadFluidAfterCallback(position(0)),
            LocalDestroyEffect::WriteFluid {
                position: position(0),
                state: 2,
                flags: 11,
            },
            LocalDestroyEffect::BlockDestroyHook(position(0)),
        ]
    );
}

#[test]
fn cumulative_ack_restores_logically_before_update_without_claiming_a_rendered_frame() {
    let mut server = ServerPredictionAck::default();
    assert_eq!(
        server.register(-1),
        Err(PredictionError::NegativeSequence(-1))
    );
    server.register(7).unwrap();
    server.register(3).unwrap();
    assert_eq!(server.acknowledgement(), 7);

    let stone = BlockStateId::new(1);
    let air = BlockStateId::new(0);
    let captured = Vec3::new(1.5, 64.0, 0.5);
    let mut retained = RetainedPrediction {
        sequence: 2,
        authoritative_state: stone,
        captured_player_position: captured,
    };
    retained.retain_again(7);
    assert_eq!(
        retained.resolve(6, air, ClientPredictionClock::default(), true),
        PredictionResolution::Pending
    );
    assert_eq!(
        retained.resolve(7, air, ClientPredictionClock::default(), true),
        PredictionResolution::Restore {
            state: stone,
            flags: 19,
            snap_to: Some(captured),
        }
    );

    retained.stage_authoritative(air);
    assert_eq!(
        retained.resolve(7, air, ClientPredictionClock::default(), true),
        PredictionResolution::RemoveUnchanged
    );
}
