use crate::java_26_2::play::clientbound::entity_motion::packet::PositionMoveRotation;
use crate::java_26_2::play::clientbound::packet::Vector3;

pub const POSITION_THRESHOLD_SQUARED: f64 = 7.629_394_531_25e-6;
pub const VELOCITY_THRESHOLD_SQUARED: f64 = 1.0e-7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosePublication {
    PositionSync,
    RelativePosition,
    RelativePositionRotation,
    RelativeRotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerPublicationStep {
    Motion,
    ProjectilePower,
    MinecartSteps { current_snapshot: bool },
    Pose(PosePublication),
    DirtyState,
    HeadRotation,
    HurtMotionToTrackersAndSelf,
    ResetPacketPositionBase,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackerPublicationInput {
    pub passenger: bool,
    pub new_behavior_minecart: bool,
    pub minecart_has_recorded_steps: bool,
    pub velocity_changed: bool,
    pub hurting_projectile: bool,
    pub position_changed: bool,
    pub sixty_tick_refresh: bool,
    pub rotation_changed: bool,
    pub abstract_arrow: bool,
    pub precise_position_required: bool,
    pub delta_x: i64,
    pub delta_y: i64,
    pub delta_z: i64,
    pub ordinary_passes_since_absolute: u16,
    pub just_stopped_riding: bool,
    pub on_ground_changed: bool,
    pub dirty_state: bool,
    pub head_rotation_changed: bool,
    pub hurt_marked: bool,
}

#[must_use]
pub fn tracker_publication_plan(input: TrackerPublicationInput) -> Vec<TrackerPublicationStep> {
    let mut plan = Vec::new();
    if input.passenger {
        if input.rotation_changed {
            plan.push(TrackerPublicationStep::Pose(
                PosePublication::RelativeRotation,
            ));
        }
        plan.push(TrackerPublicationStep::ResetPacketPositionBase);
        if input.dirty_state {
            plan.push(TrackerPublicationStep::DirtyState);
        }
        if input.hurt_marked {
            plan.push(TrackerPublicationStep::HurtMotionToTrackersAndSelf);
        }
        return plan;
    }
    if input.new_behavior_minecart {
        plan.push(TrackerPublicationStep::MinecartSteps {
            current_snapshot: !input.minecart_has_recorded_steps,
        });
        plan.push(TrackerPublicationStep::ResetPacketPositionBase);
    } else {
        if input.velocity_changed {
            plan.push(TrackerPublicationStep::Motion);
            if input.hurting_projectile {
                plan.push(TrackerPublicationStep::ProjectilePower);
            }
        }
        if let Some(pose) = select_pose(input) {
            plan.push(TrackerPublicationStep::Pose(pose));
        }
    }
    if input.dirty_state {
        plan.push(TrackerPublicationStep::DirtyState);
    }
    if input.head_rotation_changed {
        plan.push(TrackerPublicationStep::HeadRotation);
    }
    if input.hurt_marked {
        plan.push(TrackerPublicationStep::HurtMotionToTrackersAndSelf);
    }
    plan
}

#[must_use]
pub fn should_publish_velocity(
    previous: Vector3,
    current: Vector3,
    tracking_enabled_or_required: bool,
) -> bool {
    if !tracking_enabled_or_required {
        return false;
    }
    let difference = squared_difference(previous, current);
    difference > VELOCITY_THRESHOLD_SQUARED
        || (current == Vector3::default() && previous != Vector3::default())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RidingTeleportPublication {
    pub controller: PositionMoveRotation,
    pub controller_relative_flags: u32,
    pub other_passenger: PositionMoveRotation,
    pub other_passenger_relative_flags: u32,
}

#[must_use]
pub const fn riding_teleport_publication(
    transition: PositionMoveRotation,
    current_absolute: PositionMoveRotation,
    transition_relative_flags: u32,
) -> RidingTeleportPublication {
    RidingTeleportPublication {
        controller: transition,
        controller_relative_flags: transition_relative_flags,
        other_passenger: current_absolute,
        other_passenger_relative_flags: 0,
    }
}

fn select_pose(input: TrackerPublicationInput) -> Option<PosePublication> {
    let position_eligible = input.position_changed || input.sixty_tick_refresh;
    let delta_out_of_range =
        !fits_short(input.delta_x) || !fits_short(input.delta_y) || !fits_short(input.delta_z);
    if input.precise_position_required
        || delta_out_of_range
        || input.ordinary_passes_since_absolute > 400
        || input.just_stopped_riding
        || input.on_ground_changed
    {
        return Some(PosePublication::PositionSync);
    }
    if position_eligible && (input.rotation_changed || input.abstract_arrow) {
        Some(PosePublication::RelativePositionRotation)
    } else if position_eligible {
        Some(PosePublication::RelativePosition)
    } else if input.rotation_changed {
        Some(PosePublication::RelativeRotation)
    } else {
        None
    }
}

const fn fits_short(value: i64) -> bool {
    value >= i16::MIN as i64 && value <= i16::MAX as i64
}

fn squared_difference(left: Vector3, right: Vector3) -> f64 {
    let x = left.x - right.x;
    let y = left.y - right.y;
    let z = left.z - right.z;
    x * x + y * y + z * z
}
