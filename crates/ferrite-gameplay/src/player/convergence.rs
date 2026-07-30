//! Client movement-message selection and teleport correction convergence.

use crate::player::state::{PlayerPose, Rotation, Vec3};

const POSITION_CHANGE_THRESHOLD_SQUARED: f64 = 4.0e-8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientMovementMessage {
    PositionRotation {
        pose: PlayerPose,
        on_ground: bool,
        horizontal_collision: bool,
    },
    Position {
        position: Vec3,
        on_ground: bool,
        horizontal_collision: bool,
    },
    Rotation {
        rotation: Rotation,
        on_ground: bool,
        horizontal_collision: bool,
    },
    StatusOnly {
        on_ground: bool,
        horizontal_collision: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientMovementProjection {
    last_position: Vec3,
    last_rotation: Rotation,
    last_on_ground: bool,
    last_horizontal_collision: bool,
    position_reminder: u8,
}

impl ClientMovementProjection {
    #[must_use]
    pub const fn new(pose: PlayerPose, on_ground: bool, horizontal_collision: bool) -> Self {
        Self {
            last_position: pose.position,
            last_rotation: pose.rotation,
            last_on_ground: on_ground,
            last_horizontal_collision: horizontal_collision,
            position_reminder: 0,
        }
    }

    pub fn select(
        &mut self,
        pose: PlayerPose,
        on_ground: bool,
        horizontal_collision: bool,
        controlled_camera: bool,
    ) -> Option<ClientMovementMessage> {
        if !controlled_camera {
            return None;
        }
        let delta = pose.position.subtract(self.last_position);
        let yaw_delta = pose.rotation.yaw - self.last_rotation.yaw;
        let pitch_delta = pose.rotation.pitch - self.last_rotation.pitch;
        self.position_reminder = self.position_reminder.saturating_add(1);
        let position_changed = delta.length_squared() > POSITION_CHANGE_THRESHOLD_SQUARED
            || self.position_reminder >= 20;
        let rotation_changed = f64::from(yaw_delta) != 0.0 || f64::from(pitch_delta) != 0.0;
        let status_changed = on_ground != self.last_on_ground
            || horizontal_collision != self.last_horizontal_collision;
        let message = match (position_changed, rotation_changed, status_changed) {
            (true, true, _) => Some(ClientMovementMessage::PositionRotation {
                pose,
                on_ground,
                horizontal_collision,
            }),
            (true, false, _) => Some(ClientMovementMessage::Position {
                position: pose.position,
                on_ground,
                horizontal_collision,
            }),
            (false, true, _) => Some(ClientMovementMessage::Rotation {
                rotation: pose.rotation,
                on_ground,
                horizontal_collision,
            }),
            (false, false, true) => Some(ClientMovementMessage::StatusOnly {
                on_ground,
                horizontal_collision,
            }),
            (false, false, false) => None,
        };
        if position_changed {
            self.last_position = pose.position;
            self.position_reminder = 0;
        }
        if rotation_changed {
            self.last_rotation = pose.rotation;
        }
        self.last_on_ground = on_ground;
        self.last_horizontal_collision = horizontal_collision;
        message
    }

    #[must_use]
    pub const fn position_reminder(&self) -> u8 {
        self.position_reminder
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RelativeTransform {
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub yaw: bool,
    pub pitch: bool,
    pub velocity_x: bool,
    pub velocity_y: bool,
    pub velocity_z: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionCorrection {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Rotation,
    pub relative: RelativeTransform,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PendingTeleport {
    pub id: i32,
    pub position: Vec3,
    pub issued_tick: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ServerTeleportState {
    next_id: i32,
    pending: Option<PendingTeleport>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TeleportAcknowledgement {
    Mismatched,
    Accepted { position: Vec3 },
    DisconnectInvalidMovement,
}

impl ServerTeleportState {
    #[must_use]
    pub fn issue(&mut self, position: Vec3, connection_tick: u64) -> PendingTeleport {
        self.next_id = if self.next_id == i32::MAX {
            0
        } else {
            self.next_id + 1
        };
        let pending = PendingTeleport {
            id: self.next_id,
            position,
            issued_tick: connection_tick,
        };
        self.pending = Some(pending);
        pending
    }

    #[must_use]
    pub fn resend_if_due(&mut self, connection_tick: u64) -> Option<PendingTeleport> {
        let pending = self.pending?;
        (connection_tick.saturating_sub(pending.issued_tick) > 20)
            .then(|| self.issue(pending.position, connection_tick))
    }

    #[must_use]
    pub fn acknowledge(&mut self, id: i32) -> TeleportAcknowledgement {
        if self.pending.is_none() && id == self.next_id {
            return TeleportAcknowledgement::DisconnectInvalidMovement;
        }
        let Some(pending) = self.pending else {
            return TeleportAcknowledgement::Mismatched;
        };
        if pending.id != id {
            return TeleportAcknowledgement::Mismatched;
        }
        self.pending = None;
        TeleportAcknowledgement::Accepted {
            position: pending.position,
        }
    }

    #[must_use]
    pub const fn pending(&self) -> Option<PendingTeleport> {
        self.pending
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientCorrectionState {
    pub pose: PlayerPose,
    pub old_pose: PlayerPose,
    pub velocity: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientCorrectionAction {
    Acknowledge(i32),
    SendPositionRotation {
        pose: PlayerPose,
        on_ground: bool,
        horizontal_collision: bool,
    },
    PredictionBarrier,
}

impl ClientCorrectionState {
    pub fn apply(
        &mut self,
        correction: PositionCorrection,
        teleport_id: i32,
        passenger: bool,
    ) -> [ClientCorrectionAction; 3] {
        if !passenger {
            let position = resolve_position(self.pose.position, correction);
            let velocity = resolve_velocity(self.velocity, correction);
            let rotation = resolve_rotation(self.pose.rotation, correction);
            self.pose = PlayerPose::new(position, rotation);
            self.velocity = velocity;
            self.old_pose = PlayerPose::new(
                resolve_position(self.old_pose.position, correction),
                resolve_rotation(self.old_pose.rotation, correction),
            );
        }
        [
            ClientCorrectionAction::Acknowledge(teleport_id),
            ClientCorrectionAction::SendPositionRotation {
                pose: self.pose,
                on_ground: false,
                horizontal_collision: false,
            },
            ClientCorrectionAction::PredictionBarrier,
        ]
    }
}

fn resolve_position(current: Vec3, correction: PositionCorrection) -> Vec3 {
    Vec3::new(
        resolve_component(current.x, correction.position.x, correction.relative.x),
        resolve_component(current.y, correction.position.y, correction.relative.y),
        resolve_component(current.z, correction.position.z, correction.relative.z),
    )
}

fn resolve_velocity(current: Vec3, correction: PositionCorrection) -> Vec3 {
    Vec3::new(
        resolve_component(
            current.x,
            correction.velocity.x,
            correction.relative.velocity_x,
        ),
        resolve_component(
            current.y,
            correction.velocity.y,
            correction.relative.velocity_y,
        ),
        resolve_component(
            current.z,
            correction.velocity.z,
            correction.relative.velocity_z,
        ),
    )
}

fn resolve_rotation(current: Rotation, correction: PositionCorrection) -> Rotation {
    Rotation {
        yaw: resolve_rotation_component(
            current.yaw,
            correction.rotation.yaw,
            correction.relative.yaw,
        ),
        pitch: resolve_rotation_component(
            current.pitch,
            correction.rotation.pitch,
            correction.relative.pitch,
        ),
    }
}

const fn resolve_component(current: f64, supplied: f64, relative: bool) -> f64 {
    if relative {
        current + supplied
    } else {
        supplied
    }
}

const fn resolve_rotation_component(current: f32, supplied: f32, relative: bool) -> f32 {
    if relative {
        current + supplied
    } else {
        supplied
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_reminder_and_status_forms_are_independent() {
        let pose = PlayerPose::default();
        let mut projection = ClientMovementProjection::new(pose, false, false);
        assert_eq!(
            projection.select(pose, true, false, true),
            Some(ClientMovementMessage::StatusOnly {
                on_ground: true,
                horizontal_collision: false,
            })
        );
        for _ in 0..18 {
            assert_eq!(projection.select(pose, true, false, true), None);
        }
        assert!(matches!(
            projection.select(pose, true, false, true),
            Some(ClientMovementMessage::Position { .. })
        ));
        assert_eq!(projection.position_reminder(), 0);
    }

    #[test]
    fn teleport_resends_strictly_after_twenty_ticks_and_acknowledges_exact_id() {
        let mut state = ServerTeleportState::default();
        let first = state.issue(Vec3::new(1.0, 2.0, 3.0), 10);
        assert_eq!(state.resend_if_due(30), None);
        let resent = state.resend_if_due(31).unwrap();
        assert_ne!(resent.id, first.id);
        assert_eq!(
            state.acknowledge(first.id),
            TeleportAcknowledgement::Mismatched
        );
        assert_eq!(
            state.acknowledge(resent.id),
            TeleportAcknowledgement::Accepted {
                position: resent.position
            }
        );
    }

    #[test]
    fn client_applies_relative_current_and_old_transforms_before_ordered_messages() {
        let mut state = ClientCorrectionState {
            pose: PlayerPose::new(
                Vec3::new(10.0, 20.0, 30.0),
                Rotation {
                    yaw: 10.0,
                    pitch: 20.0,
                },
            ),
            old_pose: PlayerPose::new(
                Vec3::new(9.0, 19.0, 29.0),
                Rotation {
                    yaw: 9.0,
                    pitch: 19.0,
                },
            ),
            velocity: Vec3::new(1.0, 2.0, 3.0),
        };
        let actions = state.apply(
            PositionCorrection {
                position: Vec3::new(1.0, 2.0, 3.0),
                velocity: Vec3::new(4.0, 5.0, 6.0),
                rotation: Rotation {
                    yaw: 5.0,
                    pitch: 6.0,
                },
                relative: RelativeTransform {
                    x: true,
                    yaw: true,
                    velocity_z: true,
                    ..RelativeTransform::default()
                },
            },
            7,
            false,
        );
        assert_eq!(state.pose.position, Vec3::new(11.0, 2.0, 3.0));
        assert_eq!(state.old_pose.position, Vec3::new(10.0, 2.0, 3.0));
        assert_eq!(state.velocity, Vec3::new(4.0, 5.0, 9.0));
        assert!(matches!(actions[0], ClientCorrectionAction::Acknowledge(7)));
        assert!(matches!(
            actions[2],
            ClientCorrectionAction::PredictionBarrier
        ));
    }
}
