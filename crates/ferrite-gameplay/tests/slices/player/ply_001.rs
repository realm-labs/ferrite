use std::collections::BTreeMap;

use ferrite_gameplay::player::auto_jump::{
    AutoJumpContext, AutoJumpDecision, AutoJumpState, AutoJumpWorld, BlockPos,
};
use ferrite_gameplay::player::collision::{
    Aabb, CollisionProbe, CollisionScene, CollisionWorld, EntityMotion, MoveContext, MoverType,
    NoCollision, back_off_from_edge, collide, move_entity,
};
use ferrite_gameplay::player::convergence::{
    ClientCorrectionAction, ClientCorrectionState, ClientMovementMessage, ClientMovementProjection,
    PositionCorrection, RelativeTransform, ServerTeleportState, TeleportAcknowledgement,
};
use ferrite_gameplay::player::input::{
    ButtonInput, InputModifiers, InputSyncMessage, InputSyncState, LocalInputContext,
    LocalInputState, SampledInput, ServerInputState, Vec2, shape_movement,
};
use ferrite_gameplay::player::movement::{
    MovementContext, MovementOutcome, PlayerMove, validate_movement,
};
use ferrite_gameplay::player::spectator::{
    AdmissionAction, ChunkPos, ChunkProjection, ChunkProjectionAction, SectionPos,
    SpectatorAdmissions,
};
use ferrite_gameplay::player::state::{PlayerPose, PlayerSessionState, Rotation, Vec3};
use ferrite_gameplay::player::travel::{
    TravelAttributes, TravelContext, TravelInput, TravelTimers, ordinary_travel_tick,
};

fn player_box(position: Vec3) -> Aabb {
    Aabb::new(
        Vec3::new(position.x - 0.3, position.y, position.z - 0.3),
        Vec3::new(position.x + 0.3, position.y + 1.8, position.z + 0.3),
    )
}

#[test]
fn input_conflicts_square_shaping_and_server_retention_are_source_ordered() {
    let canceled = SampledInput::from_buttons(ButtonInput {
        forward: true,
        backward: true,
        left: true,
        right: true,
        ..ButtonInput::default()
    });
    assert_eq!(canceled.movement, Vec2::ZERO);

    let diagonal = SampledInput::from_buttons(ButtonInput {
        forward: true,
        left: true,
        ..ButtonInput::default()
    });
    assert_eq!(diagonal.movement.x, 1.0_f32 / 2.0_f32.sqrt());
    assert_eq!(
        shape_movement(
            diagonal.movement,
            InputModifiers {
                using_item: true,
                item_speed_multiplier: 0.2,
                crouching: true,
                sneaking_speed: 0.3,
                ..InputModifiers::default()
            }
        ),
        Vec2::new(0.98_f32 * 0.2_f32 * 0.3_f32, 0.98_f32 * 0.2_f32 * 0.3_f32,)
    );

    let mut server = ServerInputState::default();
    let retained = ButtonInput {
        forward: true,
        shift: true,
        ..ButtonInput::default()
    };
    server.handle_input(retained, false);
    assert!(!server.shared_shift);
    assert_eq!(server.move_intent(90.0).x, -1.0);
    server.handle_input(retained, true);
    server.handle_sprint(true, true);
    server.handle_abilities(true, false);
    assert!(server.shared_shift);
    assert!(server.sprinting);
    assert!(!server.flying);
}

#[test]
fn previous_shift_auto_jump_and_independent_input_sprint_cadence_are_locked() {
    let mut local = LocalInputState::default();
    local.tick(
        ButtonInput {
            shift: true,
            ..ButtonInput::default()
        },
        LocalInputContext::default(),
    );
    local.schedule_auto_jump();
    local.tick(ButtonInput::default(), LocalInputContext::default());
    assert!(local.crouching);
    assert!(local.current().jump);
    assert_eq!(local.auto_jump_time, 0);

    let mut sync = InputSyncState::default();
    let current = ButtonInput {
        forward: true,
        sprint: true,
        ..ButtonInput::default()
    };
    assert_eq!(
        sync.select(current, true, false),
        vec![
            InputSyncMessage::Input(current),
            InputSyncMessage::StartSprinting
        ]
    );
    assert!(sync.select(current, true, false).is_empty());
}

#[test]
fn generic_collision_clips_axes_steps_and_edge_retention_at_locked_boundaries() {
    let origin = Vec3::new(0.0, 65.0, 0.0);
    let scene = CollisionScene {
        block_shapes: vec![
            Aabb::new(Vec3::new(0.3, 64.0, -1.0), Vec3::new(1.0, 65.5, 1.0)),
            Aabb::new(Vec3::new(-2.0, 64.0, -2.0), Vec3::new(0.45, 65.0, 2.0)),
        ],
        ..CollisionScene::default()
    };
    let stepped = collide(
        Vec3::new(1.0, 0.0, 0.0),
        player_box(origin),
        0.6,
        true,
        &scene,
    );
    assert_eq!(stepped.y, 0.5);
    assert!(stepped.x > 0.9);

    let edge_scene = CollisionScene {
        block_shapes: vec![Aabb::new(
            Vec3::new(-2.0, 64.0, -2.0),
            Vec3::new(0.45, 65.0, 2.0),
        )],
        ..CollisionScene::default()
    };
    let backed = back_off_from_edge(
        player_box(origin),
        Vec3::new(1.0, 0.0, 0.0),
        0.6,
        false,
        0.6,
        &edge_scene,
    );
    assert!(backed.x <= 0.75);
    assert!((backed.x * 20.0 - (backed.x * 20.0).round()).abs() < 1.0e-10);

    let mut piston = EntityMotion::new(origin, player_box(origin));
    let first = move_entity(
        &mut piston,
        Vec3::new(1.0, 0.2, 0.0),
        MoveContext {
            mover_type: MoverType::Piston,
            game_time: 4,
            ..MoveContext::default()
        },
        &CollisionScene::default(),
    );
    assert_eq!(first.actual, Vec3::new(0.51, 0.0, 0.0));
}

#[test]
fn ordinary_travel_preserves_jump_gravity_friction_and_drag_order() {
    let position = Vec3::new(0.0, 65.0, 0.0);
    let mut motion = EntityMotion::new(position, player_box(position));
    let mut timers = TravelTimers::default();
    let result = ordinary_travel_tick(
        &mut motion,
        &mut timers,
        TravelAttributes::default(),
        TravelContext {
            input: TravelInput {
                strafe: 0.0,
                vertical: 0.0,
                forward: 0.98,
            },
            yaw: 0.0,
            jumping: true,
            on_ground: true,
            sprinting: true,
            jump_boost_amplifier: Some(0),
            block_friction: 0.6,
            ..TravelContext::default()
        },
        &CollisionScene::default(),
    );
    assert!(result.jumped);
    assert_eq!(result.friction, 0.6);
    assert_eq!(
        result.acceleration_scale,
        TravelAttributes::default().movement_speed as f32
    );
    assert_eq!(timers.no_jump_delay, 10);
    assert!(motion.position.y > 65.51);
    assert!(motion.velocity.y < result.movement.actual.y);
    assert!(motion.position.z > 0.29);
}

#[derive(Default)]
struct AutoJumpTestWorld {
    contextual: BTreeMap<BlockPos, Aabb>,
    entities: Vec<Aabb>,
    blocks: Vec<Aabb>,
}

impl AutoJumpWorld for AutoJumpTestWorld {
    fn collision_shape(&self, position: BlockPos) -> Option<Aabb> {
        self.contextual.get(&position).copied()
    }

    fn entity_collision_shapes(&self, _query: Aabb) -> Vec<Aabb> {
        self.entities.clone()
    }

    fn block_collision_shapes(&self, _query: Aabb) -> Vec<Aabb> {
        self.blocks.clone()
    }
}

fn auto_jump_context() -> AutoJumpContext {
    let position = Vec3::new(0.0, 65.0, 0.0);
    AutoJumpContext {
        position,
        bounds: player_box(position),
        actual_horizontal: Vec2::new(0.0, 0.2),
        raw_movement: Vec2::new(0.0, 1.0),
        yaw: 0.0,
        pitch: 0.0,
        movement_speed: 0.1,
        on_ground: true,
        stay_on_ground: false,
        passenger: false,
        block_jump_factor: 1.0,
        jump_boost_amplifier: None,
    }
}

#[test]
fn auto_jump_uses_entity_then_block_last_match_and_delayed_consumption() {
    let world = AutoJumpTestWorld {
        entities: vec![Aabb::new(
            Vec3::new(-0.5, 65.0, 0.4),
            Vec3::new(0.5, 66.1, 0.7),
        )],
        blocks: vec![Aabb::new(
            Vec3::new(-0.5, 65.0, 0.8),
            Vec3::new(0.5, 65.8, 1.1),
        )],
        ..AutoJumpTestWorld::default()
    };
    let mut state = AutoJumpState::default();
    assert_eq!(
        state.detect(auto_jump_context(), &world),
        AutoJumpDecision::Scheduled
    );
    assert_eq!(state.timer, 1);
    assert!(state.consume(false));
    assert_eq!(state.timer, 0);

    let half_block = AutoJumpTestWorld {
        blocks: vec![Aabb::new(
            Vec3::new(-0.5, 65.0, 0.4),
            Vec3::new(0.5, 65.5, 0.7),
        )],
        ..AutoJumpTestWorld::default()
    };
    assert_eq!(
        state.detect(auto_jump_context(), &half_block),
        AutoJumpDecision::Rejected
    );
}

struct HorizontalResidual;

impl CollisionWorld for HorizontalResidual {
    fn probe_player_movement(&self, _origin: Vec3, requested: Vec3) -> CollisionProbe {
        CollisionProbe {
            actual_displacement: Vec3::new(0.0, requested.y - 100.0, requested.z),
            old_box_collision_free: true,
            introduced_collision: false,
            supporting_collision_before: true,
            nearby_block_below: true,
        }
    }
}

#[test]
fn movement_selection_validation_and_teleport_convergence_remain_distinct() {
    let initial = PlayerPose::new(Vec3::new(0.0, 65.0, 0.0), Rotation::default());
    let mut client = ClientMovementProjection::new(initial, false, false);
    assert!(matches!(
        client.select(initial, true, false, true),
        Some(ClientMovementMessage::StatusOnly { .. })
    ));
    for _ in 0..19 {
        let selected = client.select(initial, true, false, true);
        if client.position_reminder() == 0 {
            assert!(matches!(
                selected,
                Some(ClientMovementMessage::Position { .. })
            ));
        }
    }

    let mut server = PlayerSessionState::new(initial);
    server.accept_player_loaded();
    let outcome = validate_movement(
        &mut server,
        PlayerMove {
            position: Some(Vec3::new(0.0, 64.0, 0.0)),
            rotation: None,
            on_ground: true,
            horizontal_collision: false,
        },
        MovementContext::default(),
        &HorizontalResidual,
    );
    assert!(matches!(outcome, MovementOutcome::Accepted { .. }));

    let mut teleport = ServerTeleportState::default();
    let issued = teleport.issue(Vec3::new(4.0, 70.0, 8.0), 10);
    assert_eq!(teleport.resend_if_due(30), None);
    let resent = teleport.resend_if_due(31).unwrap();
    assert_eq!(
        teleport.acknowledge(issued.id),
        TeleportAcknowledgement::Mismatched
    );
    assert!(matches!(
        teleport.acknowledge(resent.id),
        TeleportAcknowledgement::Accepted { .. }
    ));

    let mut convergence = ClientCorrectionState {
        pose: initial,
        old_pose: initial,
        velocity: Vec3::ZERO,
    };
    let actions = convergence.apply(
        PositionCorrection {
            position: Vec3::new(1.0, 2.0, 3.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
            rotation: Rotation::default(),
            relative: RelativeTransform {
                x: true,
                ..RelativeTransform::default()
            },
        },
        resent.id,
        false,
    );
    assert!(matches!(
        actions,
        [
            ClientCorrectionAction::Acknowledge(_),
            ClientCorrectionAction::SendPositionRotation { .. },
            ClientCorrectionAction::PredictionBarrier
        ]
    ));
}

#[test]
fn spectator_distance_admission_is_delayed_but_client_projection_is_independent() {
    let section = SectionPos {
        chunk: ChunkPos::new(0, 0),
        section_y: 4,
    };
    let mut admissions = SpectatorAdmissions::default();
    let join = admissions.add_player(7, section, true, false);
    assert!(matches!(
        join.as_slice(),
        [
            AdmissionAction::PlayerMapAdded { ignored: true, .. },
            AdmissionAction::ResetClientView { .. },
            AdmissionAction::RefreshClientView { .. }
        ]
    ));
    assert_eq!(admissions.admitted_count(section.chunk), 0);

    let reconciled = admissions.move_player(7, section, true, true, false);
    assert!(matches!(
        reconciled.as_slice(),
        [
            AdmissionAction::ProjectEntities { .. },
            AdmissionAction::AddDistanceSource {
                first_at_chunk: true,
                ..
            },
            AdmissionAction::SetIgnored { ignored: false, .. },
            AdmissionAction::RefreshClientView { .. }
        ]
    ));
    assert_eq!(admissions.admitted_count(section.chunk), 1);
    assert_eq!(SpectatorAdmissions::simulation_ticket_level(10), 21);
    assert_eq!(SpectatorAdmissions::natural_spawn_cap(70, 289), 70);

    let mut projection = ChunkProjection::default();
    projection.update_view(section.chunk, 2, 10, true, |_| false);
    let external = ChunkPos::new(1, 1);
    assert_eq!(
        projection.on_chunk_ready(external),
        Some(ChunkProjectionAction::Pending(external))
    );
    assert!(matches!(
        projection.send_pending(1).as_slice(),
        [
            ChunkProjectionAction::BatchStart,
            ChunkProjectionAction::SendFullChunk(position),
            ChunkProjectionAction::BatchFinish
        ] if *position == external
    ));
}

#[test]
fn movement_infinity_clamp_and_sixth_packet_multiplier_match_server_authority() {
    let initial = PlayerPose::new(Vec3::new(0.0, 65.0, 0.0), Rotation::default());
    let mut state = PlayerSessionState::new(initial);
    state.accept_player_loaded();
    assert!(matches!(
        validate_movement(
            &mut state,
            PlayerMove {
                position: Some(Vec3::new(f64::INFINITY, 65.0, f64::NEG_INFINITY)),
                rotation: None,
                on_ground: true,
                horizontal_collision: true,
            },
            MovementContext {
                singleplayer_owner: true,
                ..MovementContext::default()
            },
            &NoCollision,
        ),
        MovementOutcome::Accepted { .. }
    ));
    assert_eq!(
        state.pose().position,
        Vec3::new(30_000_000.0, 65.0, -30_000_000.0)
    );
}
