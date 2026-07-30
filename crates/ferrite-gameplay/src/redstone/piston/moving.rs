//! Moving-piston progress, entity displacement, finalization, and moving-block hooks.

use ferrite_foundation::direction::{Axis, Direction};

use crate::redstone::delay::orientation::{InitialOrientation, initial_orientation};
use crate::redstone::piston::resolver::PushReaction;

pub const PROGRESS_STEP: f32 = 0.5;
pub const COLLISION_PADDING: f64 = 0.01;
pub const MOVEMENT_AREA_CONSTANT: f64 = 0.51;
pub const STICKY_TOP_MAX_Y: f64 = 1.5000010000000001;
pub const CLIENT_DEATH_TICKS: u8 = 5;
pub const NORMAL_FINAL_WRITE_FLAGS: u16 = 67;
pub const AIR_FALLBACK_WRITE_FLAGS: u16 = 340;
pub const UPDATE_OR_DESTROY_FLAGS: u16 = 3;
pub const FORCED_FINAL_WRITE_FLAGS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovingProgress {
    pub progress: f32,
    pub previous_progress: f32,
    pub death_ticks: u8,
    pub direction: Direction,
    pub extending: bool,
    pub source_piston: bool,
}

impl MovingProgress {
    pub const fn movement_direction(self) -> Direction {
        if self.extending {
            self.direction
        } else {
            self.direction.opposite()
        }
    }

    pub const fn push_direction(self) -> Direction {
        self.movement_direction()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionWrite {
    None,
    RestoreMovedThenUpdateOrDestroy {
        first_flags: u16,
        update_flags: u16,
    },
    AdjustedCarried {
        clear_waterlogged: bool,
        write_flags: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovingTickPlan {
    pub recorded_last_ticked: bool,
    pub previous_progress: f32,
    pub new_progress: f32,
    pub collision_delta: Option<f32>,
    pub move_collided_entities: bool,
    pub move_honey_entities: bool,
    pub incremented_client_death_ticks: Option<u8>,
    pub remove_block_entity: bool,
    pub completion_write: CompletionWrite,
    pub notify_completed_state: bool,
    pub orientation: InitialOrientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletionObservation {
    pub client: bool,
    pub live_state_is_moving_piston: bool,
    pub adjusted_carried_is_air: bool,
    pub adjusted_carried_waterlogged: bool,
    pub redstone_experiments: bool,
}

pub fn moving_tick(
    state: MovingProgress,
    observation: CompletionObservation,
    moved_block_is_honey: bool,
) -> MovingTickPlan {
    if state.progress >= 1.0 {
        if observation.client && state.death_ticks < CLIENT_DEATH_TICKS {
            return MovingTickPlan {
                recorded_last_ticked: true,
                previous_progress: state.progress,
                new_progress: state.progress,
                collision_delta: None,
                move_collided_entities: false,
                move_honey_entities: false,
                incremented_client_death_ticks: Some(state.death_ticks + 1),
                remove_block_entity: false,
                completion_write: CompletionWrite::None,
                notify_completed_state: false,
                orientation: initial_orientation(false, None, None),
            };
        }
        let write = if !observation.live_state_is_moving_piston {
            CompletionWrite::None
        } else if observation.adjusted_carried_is_air {
            CompletionWrite::RestoreMovedThenUpdateOrDestroy {
                first_flags: AIR_FALLBACK_WRITE_FLAGS,
                update_flags: UPDATE_OR_DESTROY_FLAGS,
            }
        } else {
            CompletionWrite::AdjustedCarried {
                clear_waterlogged: observation.adjusted_carried_waterlogged,
                write_flags: NORMAL_FINAL_WRITE_FLAGS,
            }
        };
        let notify = matches!(write, CompletionWrite::AdjustedCarried { .. });
        return MovingTickPlan {
            recorded_last_ticked: true,
            previous_progress: state.progress,
            new_progress: state.progress,
            collision_delta: None,
            move_collided_entities: false,
            move_honey_entities: false,
            incremented_client_death_ticks: None,
            remove_block_entity: true,
            completion_write: write,
            notify_completed_state: notify,
            orientation: initial_orientation(
                notify && observation.redstone_experiments,
                if notify {
                    Some(state.push_direction())
                } else {
                    None
                },
                None,
            ),
        };
    }
    let new_progress = (state.progress + PROGRESS_STEP).min(1.0);
    MovingTickPlan {
        recorded_last_ticked: true,
        previous_progress: state.progress,
        new_progress,
        collision_delta: Some(new_progress - state.progress),
        move_collided_entities: true,
        move_honey_entities: moved_block_is_honey && state.movement_direction().axis() != Axis::Y,
        incremented_client_death_ticks: None,
        remove_block_entity: false,
        completion_write: CompletionWrite::None,
        notify_completed_state: false,
        orientation: initial_orientation(false, None, None),
    }
}

pub fn collision_displacement(
    separations: impl IntoIterator<Item = f64>,
    delta_progress: f64,
) -> Option<f64> {
    let maximum = separations.into_iter().fold(0.0_f64, f64::max);
    (maximum > 0.0).then_some(maximum.min(delta_progress) + COLLISION_PADDING)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEntityPlan {
    pub ignored: bool,
    pub velocity: [f64; 3],
    pub displacement: Option<f64>,
    pub apply_block_effects: bool,
    pub remove_latest_movement_record: bool,
    pub eject_from_retracting_source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEntityInput {
    pub reaction: PushReaction,
    pub server_player: bool,
    pub moved_block_is_slime: bool,
    pub movement: Direction,
    pub velocity: [f64; 3],
    pub delta_progress: f64,
    pub retracting_source: bool,
}

pub fn collided_entity_plan(
    input: CollisionEntityInput,
    separations: impl IntoIterator<Item = f64>,
) -> CollisionEntityPlan {
    if matches!(input.reaction, PushReaction::Ignore) {
        return CollisionEntityPlan {
            ignored: true,
            velocity: input.velocity,
            displacement: None,
            apply_block_effects: false,
            remove_latest_movement_record: false,
            eject_from_retracting_source: false,
        };
    }
    let mut intended_velocity = input.velocity;
    if input.moved_block_is_slime && !input.server_player {
        let value = f64::from(input.movement.axis_direction().sign());
        match input.movement.axis() {
            Axis::X => intended_velocity[0] = value,
            Axis::Y => intended_velocity[1] = value,
            Axis::Z => intended_velocity[2] = value,
        }
    }
    let displacement = collision_displacement(separations, input.delta_progress);
    CollisionEntityPlan {
        ignored: false,
        velocity: intended_velocity,
        displacement,
        apply_block_effects: displacement.is_some(),
        remove_latest_movement_record: displacement.is_some(),
        eject_from_retracting_source: displacement.is_some() && input.retracting_source,
    }
}

pub const fn honey_carries_entity(
    movement: Direction,
    reaction: PushReaction,
    on_ground: bool,
    supported_by_position: bool,
    x_within_inclusive_top: bool,
    z_within_inclusive_top: bool,
) -> bool {
    movement.is_horizontal()
        && matches!(reaction, PushReaction::Normal)
        && on_ground
        && (supported_by_position || x_within_inclusive_top && z_within_inclusive_top)
}

pub const fn honey_displacement(delta_progress: f64) -> f64 {
    delta_progress
}

pub fn base_ejection_displacement(
    full_separation: f64,
    intersection_separation: f64,
    delta_progress: f64,
) -> Option<f64> {
    if (full_separation - intersection_separation).abs() < COLLISION_PADDING {
        Some((full_separation + COLLISION_PADDING).min(delta_progress) + COLLISION_PADDING)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForcedFinalState {
    NoOp,
    Air,
    AdjustedCarried,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForcedFinalPlan {
    pub target: ForcedFinalState,
    pub write_flags: Option<u16>,
    pub remove_block_entity: bool,
    pub notify: bool,
    pub orientation: InitialOrientation,
}

pub fn forced_final_tick(
    level_present: bool,
    client: bool,
    previous_progress: f32,
    source_piston: bool,
    push_direction: Direction,
    live_state_is_moving_piston: bool,
    redstone_experiments: bool,
) -> ForcedFinalPlan {
    let admitted = level_present && (previous_progress < 1.0 || client);
    if !admitted || !live_state_is_moving_piston {
        return ForcedFinalPlan {
            target: ForcedFinalState::NoOp,
            write_flags: None,
            remove_block_entity: admitted,
            notify: false,
            orientation: initial_orientation(false, None, None),
        };
    }
    ForcedFinalPlan {
        target: if source_piston {
            ForcedFinalState::Air
        } else {
            ForcedFinalState::AdjustedCarried
        },
        write_flags: Some(FORCED_FINAL_WRITE_FLAGS),
        remove_block_entity: true,
        notify: true,
        orientation: initial_orientation(redstone_experiments, Some(push_direction), None),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovingBlockContract {
    pub render_invisible: bool,
    pub outline_empty: bool,
    pub clone_item_empty: bool,
    pub pathfindable: bool,
    pub ordinary_block_entity_factory_returns_none: bool,
}

pub const MOVING_BLOCK_CONTRACT: MovingBlockContract = MovingBlockContract {
    render_invisible: true,
    outline_empty: true,
    clone_item_empty: true,
    pathfindable: false,
    ordinary_block_entity_factory_returns_none: true,
};

pub const fn destroy_removes_extended_base(
    behind_is_piston_base: bool,
    behind_extended: bool,
) -> bool {
    behind_is_piston_base && behind_extended
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovingBlockUse {
    Pass,
    ConsumeAndRemove,
}

pub const fn use_moving_block(server: bool, block_entity_present: bool) -> MovingBlockUse {
    if server && !block_entity_present {
        MovingBlockUse::ConsumeAndRemove
    } else {
        MovingBlockUse::Pass
    }
}

pub const fn drops_carried_state(block_entity_present: bool) -> bool {
    block_entity_present
}
