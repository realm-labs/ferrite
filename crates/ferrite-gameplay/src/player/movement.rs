use crate::player::collision::CollisionWorld;
use crate::player::state::{PlayerPose, PlayerSessionState, Rotation, Vec3};

const MAX_HORIZONTAL_COORDINATE: f64 = 30_000_000.0;
const MAX_VERTICAL_COORDINATE: f64 = 20_000_000.0;
const MOVED_WRONGLY_DISTANCE_SQUARED: f64 = 0.0625;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerMove {
    pub position: Option<Vec3>,
    pub rotation: Option<Rotation>,
    pub on_ground: bool,
    pub horizontal_collision: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementContext {
    pub won_game: bool,
    pub teleport_pending: bool,
    pub passenger: bool,
    pub sleeping: bool,
    pub normal_tick_rate: bool,
    pub singleplayer_owner: bool,
    pub dimension_change: bool,
    pub movement_check_enabled: bool,
    pub elytra_movement_check_enabled: bool,
    pub fall_flying: bool,
    pub creative: bool,
    pub spectator: bool,
    pub post_impulse_grace: bool,
    pub no_physics: bool,
    pub server_flight_allowed: bool,
    pub may_fly: bool,
    pub levitating: bool,
    pub spin_attacking: bool,
}

impl Default for MovementContext {
    fn default() -> Self {
        Self {
            won_game: false,
            teleport_pending: false,
            passenger: false,
            sleeping: false,
            normal_tick_rate: true,
            singleplayer_owner: false,
            dimension_change: false,
            movement_check_enabled: true,
            elytra_movement_check_enabled: true,
            fall_flying: false,
            creative: false,
            spectator: false,
            post_impulse_grace: false,
            no_physics: false,
            server_flight_allowed: false,
            may_fly: false,
            levitating: false,
            spin_attacking: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementIgnoreReason {
    WonGame,
    ClientLoading,
    TeleportPending,
    Sleeping,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementOutcome {
    Accepted {
        pose: PlayerPose,
        requested_displacement: Vec3,
    },
    PassengerRotation {
        pose: PlayerPose,
    },
    Ignored(MovementIgnoreReason),
    Correct {
        authoritative_pose: PlayerPose,
    },
    DisconnectInvalidMovement,
    DisconnectFlying,
}

pub fn validate_movement(
    state: &mut PlayerSessionState,
    packet: PlayerMove,
    context: MovementContext,
    collision: &impl CollisionWorld,
) -> MovementOutcome {
    if packet
        .position
        .is_some_and(|position| position.x.is_nan() || position.y.is_nan() || position.z.is_nan())
        || packet
            .rotation
            .is_some_and(|rotation| !rotation.yaw.is_finite() || !rotation.pitch.is_finite())
    {
        return MovementOutcome::DisconnectInvalidMovement;
    }

    let current = state.pose();
    let rotation = packet.rotation.unwrap_or(current.rotation).wrapped();
    if context.won_game {
        return MovementOutcome::Ignored(MovementIgnoreReason::WonGame);
    }
    if !state.client_loaded() {
        return MovementOutcome::Ignored(MovementIgnoreReason::ClientLoading);
    }
    if context.teleport_pending {
        state.install_rotation(rotation);
        return MovementOutcome::Ignored(MovementIgnoreReason::TeleportPending);
    }
    if context.passenger {
        let pose = PlayerPose::new(current.position, rotation);
        state.install_rotation(rotation);
        return MovementOutcome::PassengerRotation { pose };
    }

    let target = clamp_position(packet.position.unwrap_or(current.position));
    if context.sleeping {
        let displacement = target.subtract(state.first_good_position());
        return if displacement.length_squared() > 1.0 {
            let authoritative_pose = PlayerPose::new(current.position, rotation);
            state.install_authoritative_pose(authoritative_pose);
            MovementOutcome::Correct { authoritative_pose }
        } else {
            MovementOutcome::Ignored(MovementIgnoreReason::Sleeping)
        };
    }

    let packet_multiplier = if context.normal_tick_rate {
        state.increment_movement_packets()
    } else {
        1
    };
    let from_tick_start = target.subtract(state.first_good_position());
    let too_quick = from_tick_start.length_squared() - state.velocity().length_squared()
        > movement_limit(context.fall_flying) * f64::from(packet_multiplier);
    let speed_check_exempt = context.singleplayer_owner
        || context.dimension_change
        || !context.movement_check_enabled
        || (context.fall_flying && !context.elytra_movement_check_enabled);
    if too_quick && !speed_check_exempt {
        return MovementOutcome::Correct {
            authoritative_pose: current,
        };
    }

    let requested = target.subtract(state.last_good_position());
    let probe = collision.probe_player_movement(state.last_good_position(), requested);
    let actual_position = Vec3::new(
        state.last_good_position().x + probe.actual_displacement.x,
        state.last_good_position().y + probe.actual_displacement.y,
        state.last_good_position().z + probe.actual_displacement.z,
    );
    let residual = target.subtract(actual_position);
    // The locked 26.2 adapter uses `y > -0.5 || y < 0.5`, which is always true.
    let residual_squared = residual.x * residual.x + residual.z * residual.z;
    let wrong_exempt = context.dimension_change
        || context.sleeping
        || context.creative
        || context.spectator
        || context.post_impulse_grace;
    let moved_wrongly = residual_squared > MOVED_WRONGLY_DISTANCE_SQUARED && !wrong_exempt;
    let collision_rejected = !context.no_physics
        && ((moved_wrongly && probe.old_box_collision_free) || probe.introduced_collision);
    if collision_rejected {
        let authoritative_pose = PlayerPose::new(state.last_good_position(), rotation);
        state.install_authoritative_pose(authoritative_pose);
        return MovementOutcome::Correct { authoritative_pose };
    }

    let floating = requested.y >= -0.03125
        && !probe.supporting_collision_before
        && !probe.nearby_block_below
        && !context.spectator
        && !context.server_flight_allowed
        && !context.may_fly
        && !context.levitating
        && !context.fall_flying
        && !context.spin_attacking;
    let pose = PlayerPose::new(target, rotation);
    state.accept_movement(
        pose,
        requested,
        packet.on_ground,
        packet.horizontal_collision,
        floating,
    );
    MovementOutcome::Accepted {
        pose,
        requested_displacement: requested,
    }
}

const fn movement_limit(fall_flying: bool) -> f64 {
    if fall_flying { 300.0 } else { 100.0 }
}

fn clamp_position(position: Vec3) -> Vec3 {
    Vec3::new(
        position
            .x
            .clamp(-MAX_HORIZONTAL_COORDINATE, MAX_HORIZONTAL_COORDINATE),
        position
            .y
            .clamp(-MAX_VERTICAL_COORDINATE, MAX_VERTICAL_COORDINATE),
        position
            .z
            .clamp(-MAX_HORIZONTAL_COORDINATE, MAX_HORIZONTAL_COORDINATE),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::collision::{CollisionProbe, NoCollision};

    fn loaded_state() -> PlayerSessionState {
        let mut state = PlayerSessionState::new(PlayerPose::new(
            Vec3::new(0.0, 65.0, 0.0),
            Rotation::default(),
        ));
        state.accept_player_loaded();
        state
    }

    fn movement(position: Vec3) -> PlayerMove {
        PlayerMove {
            position: Some(position),
            rotation: None,
            on_ground: true,
            horizontal_collision: false,
        }
    }

    #[test]
    fn invalid_values_precede_the_load_and_pending_gates() {
        let mut loading = PlayerSessionState::new(PlayerPose::default());
        assert_eq!(
            validate_movement(
                &mut loading,
                movement(Vec3::new(f64::NAN, 0.0, 0.0)),
                MovementContext {
                    teleport_pending: true,
                    ..MovementContext::default()
                },
                &NoCollision,
            ),
            MovementOutcome::DisconnectInvalidMovement
        );
        assert_eq!(
            validate_movement(
                &mut loading,
                movement(Vec3::new(1.0, 0.0, 0.0)),
                MovementContext::default(),
                &NoCollision,
            ),
            MovementOutcome::Ignored(MovementIgnoreReason::ClientLoading)
        );
    }

    #[test]
    fn pending_teleport_accepts_only_wrapped_rotation() {
        let mut state = loaded_state();
        let outcome = validate_movement(
            &mut state,
            PlayerMove {
                position: Some(Vec3::new(99.0, 99.0, 99.0)),
                rotation: Some(Rotation {
                    yaw: 721.0,
                    pitch: -181.0,
                }),
                on_ground: false,
                horizontal_collision: true,
            },
            MovementContext {
                teleport_pending: true,
                ..MovementContext::default()
            },
            &NoCollision,
        );
        assert_eq!(
            outcome,
            MovementOutcome::Ignored(MovementIgnoreReason::TeleportPending)
        );
        assert_eq!(state.pose().position, Vec3::new(0.0, 65.0, 0.0));
        assert_eq!(
            state.pose().rotation,
            Rotation {
                yaw: 1.0,
                pitch: 179.0,
            }
        );
    }

    #[test]
    fn passenger_and_sleeping_branches_install_only_the_locked_pose_changes() {
        let mut passenger = loaded_state();
        let outcome = validate_movement(
            &mut passenger,
            PlayerMove {
                position: Some(Vec3::new(99.0, 99.0, 99.0)),
                rotation: Some(Rotation {
                    yaw: 190.0,
                    pitch: 10.0,
                }),
                on_ground: true,
                horizontal_collision: true,
            },
            MovementContext {
                passenger: true,
                ..MovementContext::default()
            },
            &NoCollision,
        );
        assert!(matches!(outcome, MovementOutcome::PassengerRotation { .. }));
        assert_eq!(passenger.pose().position, Vec3::new(0.0, 65.0, 0.0));
        assert_eq!(passenger.pose().rotation.yaw, -170.0);

        let mut sleeping = loaded_state();
        let outcome = validate_movement(
            &mut sleeping,
            PlayerMove {
                position: Some(Vec3::new(2.0, 65.0, 0.0)),
                rotation: Some(Rotation {
                    yaw: 45.0,
                    pitch: 5.0,
                }),
                on_ground: false,
                horizontal_collision: false,
            },
            MovementContext {
                sleeping: true,
                ..MovementContext::default()
            },
            &NoCollision,
        );
        assert_eq!(
            outcome,
            MovementOutcome::Correct {
                authoritative_pose: PlayerPose::new(
                    Vec3::new(0.0, 65.0, 0.0),
                    Rotation {
                        yaw: 45.0,
                        pitch: 5.0,
                    },
                ),
            }
        );
        assert_eq!(sleeping.pose().rotation.yaw, 45.0);
    }

    #[test]
    fn position_infinities_clamp_and_accepted_flags_are_from_the_packet() {
        let mut state = loaded_state();
        let outcome = validate_movement(
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
        );
        assert!(matches!(outcome, MovementOutcome::Accepted { .. }));
        assert_eq!(
            state.pose().position,
            Vec3::new(30_000_000.0, 65.0, -30_000_000.0)
        );
        assert!(state.on_ground());
        assert!(state.horizontal_collision());
    }

    struct HorizontalClip;

    impl CollisionWorld for HorizontalClip {
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

    struct IntroducedCollision;

    impl CollisionWorld for IntroducedCollision {
        fn probe_player_movement(&self, _origin: Vec3, requested: Vec3) -> CollisionProbe {
            CollisionProbe {
                actual_displacement: requested,
                old_box_collision_free: true,
                introduced_collision: true,
                supporting_collision_before: false,
                nearby_block_below: false,
            }
        }
    }

    #[test]
    fn newly_introduced_collision_corrects_to_the_pre_packet_pose_and_packet_rotation() {
        let mut state = loaded_state();
        let outcome = validate_movement(
            &mut state,
            PlayerMove {
                position: Some(Vec3::new(1.0, 65.0, 0.0)),
                rotation: Some(Rotation {
                    yaw: 30.0,
                    pitch: 4.0,
                }),
                on_ground: false,
                horizontal_collision: false,
            },
            MovementContext::default(),
            &IntroducedCollision,
        );
        assert_eq!(
            outcome,
            MovementOutcome::Correct {
                authoritative_pose: PlayerPose::new(
                    Vec3::new(0.0, 65.0, 0.0),
                    Rotation {
                        yaw: 30.0,
                        pitch: 4.0,
                    },
                ),
            }
        );
        assert_eq!(state.pose().rotation.yaw, 30.0);
    }

    #[test]
    fn locked_residual_y_defect_ignores_arbitrary_vertical_residual() {
        let mut state = loaded_state();
        let outcome = validate_movement(
            &mut state,
            movement(Vec3::new(0.0, 64.0, 0.0)),
            MovementContext::default(),
            &HorizontalClip,
        );
        assert!(matches!(outcome, MovementOutcome::Accepted { .. }));
    }

    #[test]
    fn sixth_packet_uses_one_as_the_speed_multiplier() {
        let mut state = loaded_state();
        for index in 1..=5 {
            assert!(matches!(
                validate_movement(
                    &mut state,
                    movement(Vec3::new(f64::from(index), 65.0, 0.0)),
                    MovementContext::default(),
                    &NoCollision,
                ),
                MovementOutcome::Accepted { .. }
            ));
        }
        assert!(matches!(
            validate_movement(
                &mut state,
                movement(Vec3::new(11.0, 65.0, 0.0)),
                MovementContext::default(),
                &NoCollision,
            ),
            MovementOutcome::Correct { .. }
        ));
    }

    #[test]
    fn floating_timeout_scales_with_gravity_and_resets_on_support() {
        let mut state = loaded_state();
        assert!(matches!(
            validate_movement(
                &mut state,
                movement(Vec3::new(0.0, 65.0, 0.0)),
                MovementContext::default(),
                &NoCollision,
            ),
            MovementOutcome::Accepted { .. }
        ));
        for _ in 0..80 {
            assert_eq!(state.finish_server_tick(0.08, false), None);
        }
        assert_eq!(
            state.finish_server_tick(0.08, false),
            Some(MovementOutcome::DisconnectFlying)
        );
        assert_eq!(state.finish_server_tick(0.0, false), None);
    }
}
