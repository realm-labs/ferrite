use behavior_runner::client::input_prediction::{
    ClientTimeDomains, GameplayAction, GameplayBindings, GameplayContext, InputAction, InputGate,
    KeyMappingState, KeyPolicy, MouseAccumulator, MouseMovement,
};
use ferrite_gameplay::player::convergence::{
    ClientCorrectionAction, ClientCorrectionState, ClientMovementMessage, ClientMovementProjection,
    PositionCorrection, RelativeTransform,
};
use ferrite_gameplay::player::state::{PlayerPose, Rotation, Vec3};
use ferrite_testkit::service_conformance::prediction::run_same_position_prediction;

#[test]
fn keyboard_mouse_focus_screen_and_toggle_policies_are_exact() {
    let mut held = KeyMappingState::new(KeyPolicy::Hold);
    held.keyboard_event(InputAction::Press, InputGate::default());
    held.keyboard_event(InputAction::Repeat, InputGate::default());
    assert!(held.is_down());
    assert_eq!(held.click_count(), 2);
    held.keyboard_event(InputAction::Release, InputGate::default());
    assert!(!held.is_down());

    let blocked = InputGate {
        screen_open: true,
        screen_consumes: true,
        ..InputGate::default()
    };
    held.keyboard_event(InputAction::Press, blocked);
    held.mouse_event(InputAction::Press, blocked);
    assert_eq!(held.click_count(), 2);
    held.release_for_focus_or_screen();
    assert_eq!(held.click_count(), 0);
    held.resample_after_focus(true, true);
    assert!(held.is_down());

    let mut toggle = KeyMappingState::new(KeyPolicy::Toggle {
        enabled: true,
        restore_after_screen: true,
    });
    toggle.keyboard_event(InputAction::Press, InputGate::default());
    toggle.keyboard_event(InputAction::Release, InputGate::default());
    assert!(toggle.is_down(), "release does not reverse a toggle");
    toggle.release_for_focus_or_screen();
    assert!(!toggle.is_down());
    toggle.resample_after_focus(true, true);
    assert!(
        !toggle.is_down(),
        "physical focus resampling skips active toggles"
    );
    toggle.restore_after_screen_closed(true);
    assert!(toggle.is_down());
}

#[test]
fn render_frames_consume_mouse_delta_without_advancing_tick_cooldowns() {
    let mut mouse = MouseAccumulator::new(10.0, 20.0, true);
    mouse.on_move(100.0, 200.0, true);
    mouse.on_move(103.0, 196.0, true);
    mouse.on_move(105.0, 198.0, true);
    assert_eq!(mouse.render_frame(true), MouseMovement { x: 5.0, y: -2.0 });
    assert_eq!(mouse.render_frame(true), MouseMovement { x: 0.0, y: 0.0 });

    let mut time = ClientTimeDomains::default();
    for partial in [0.0, 0.25, 0.75] {
        time.render(partial);
    }
    assert_eq!(time.render_frames, 3);
    assert_eq!(time.client_ticks, 0);
    assert_eq!(time.gameplay_cooldown, 0);
    time.tick(true);
    time.tick(false);
    assert_eq!(time.client_ticks, 2);
    assert_eq!(time.gameplay_cooldown, 1);
}

#[test]
fn client_tick_drains_actions_in_attack_use_pick_then_held_order() {
    let mut bindings = GameplayBindings::default();
    for mapping in [
        &mut bindings.pick,
        &mut bindings.use_item,
        &mut bindings.attack,
    ] {
        mapping.keyboard_event(InputAction::Press, InputGate::default());
    }
    assert_eq!(
        bindings.client_tick(GameplayContext {
            using_item: false,
            screen_open: false,
            mouse_grabbed: true,
            right_click_delay: 0,
            instant_attack: false,
        }),
        vec![
            GameplayAction::StartAttack,
            GameplayAction::StartUse,
            GameplayAction::Pick,
            GameplayAction::StartUse,
            GameplayAction::ContinueAttack(true),
        ]
    );

    bindings
        .attack
        .keyboard_event(InputAction::Press, InputGate::default());
    bindings
        .use_item
        .keyboard_event(InputAction::Release, InputGate::default());
    assert_eq!(
        bindings.client_tick(GameplayContext {
            using_item: true,
            screen_open: false,
            mouse_grabbed: true,
            right_click_delay: 0,
            instant_attack: false,
        }),
        vec![
            GameplayAction::ReleaseUse,
            GameplayAction::ContinueAttack(true)
        ]
    );
    assert_eq!(bindings.attack.click_count(), 0);
}

#[test]
fn same_position_predictions_stage_updates_and_resolve_cumulatively() {
    let report = run_same_position_prediction();
    assert_eq!(report.predicted_before_covering_ack, 12);
    assert_eq!(report.pending_after_old_ack, 1);
    assert_eq!(report.resolved_by_covering_ack, 1);
    assert_eq!(report.captured_authoritative_state, 13);
    assert_eq!(report.state_after_covering_ack, 13);
}

#[test]
fn movement_forms_heartbeat_and_relative_correction_ack_order_converge() {
    let initial = PlayerPose::default();
    let mut movement = ClientMovementProjection::new(initial, false, false);
    assert!(matches!(
        movement.select(
            PlayerPose::new(Vec3::new(1.0, 0.0, 0.0), Rotation::default()),
            false,
            false,
            true
        ),
        Some(ClientMovementMessage::Position { .. })
    ));
    assert!(matches!(
        movement.select(
            PlayerPose::new(
                Vec3::new(1.0, 0.0, 0.0),
                Rotation {
                    yaw: 5.0,
                    pitch: 0.0
                }
            ),
            false,
            false,
            true
        ),
        Some(ClientMovementMessage::Rotation { .. })
    ));
    assert!(matches!(
        movement.select(
            PlayerPose::new(
                Vec3::new(2.0, 0.0, 0.0),
                Rotation {
                    yaw: 6.0,
                    pitch: 0.0
                }
            ),
            true,
            false,
            true
        ),
        Some(ClientMovementMessage::PositionRotation { .. })
    ));
    assert!(matches!(
        movement.select(
            PlayerPose::new(
                Vec3::new(2.0, 0.0, 0.0),
                Rotation {
                    yaw: 6.0,
                    pitch: 0.0
                }
            ),
            true,
            true,
            true
        ),
        Some(ClientMovementMessage::StatusOnly { .. })
    ));
    let unchanged = PlayerPose::new(
        Vec3::new(2.0, 0.0, 0.0),
        Rotation {
            yaw: 6.0,
            pitch: 0.0,
        },
    );
    for _ in 0..18 {
        assert_eq!(movement.select(unchanged, true, true, true), None);
    }
    assert!(matches!(
        movement.select(unchanged, true, true, true),
        Some(ClientMovementMessage::Position { .. })
    ));

    let pose = PlayerPose::new(
        Vec3::new(10.0, 20.0, 30.0),
        Rotation {
            yaw: 40.0,
            pitch: 5.0,
        },
    );
    let mut correction = ClientCorrectionState {
        pose,
        old_pose: pose,
        velocity: Vec3::new(1.0, 2.0, 3.0),
    };
    let actions = correction.apply(
        PositionCorrection {
            position: Vec3::new(1.0, 2.0, 3.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
            rotation: Rotation {
                yaw: 10.0,
                pitch: -2.0,
            },
            relative: RelativeTransform {
                x: true,
                yaw: true,
                ..RelativeTransform::default()
            },
        },
        77,
        false,
    );
    assert_eq!(actions[0], ClientCorrectionAction::Acknowledge(77));
    assert!(matches!(
        actions[1],
        ClientCorrectionAction::SendPositionRotation { .. }
    ));
    assert_eq!(actions[2], ClientCorrectionAction::PredictionBarrier);
    assert_eq!(correction.pose.position, Vec3::new(11.0, 2.0, 3.0));
    assert_eq!(
        correction.pose.rotation,
        Rotation {
            yaw: 50.0,
            pitch: -2.0
        }
    );
}
