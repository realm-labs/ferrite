//! Observer watched-face admission and two-tick output pulse lifecycle.

use ferrite_foundation::direction::Direction;

use crate::redstone::delay::orientation::{
    InitialOrientation, OUTPUT_NOTIFICATION_ORDER, OutputNotificationStage, initial_orientation,
};

pub const EDGE_DELAY: u8 = 2;
pub const STATE_WRITE_FLAGS: u16 = 2;
pub const REPLACEMENT_CLEAR_FLAGS: u16 = 18;
pub const OUTPUT_SIGNAL: u8 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverState {
    pub facing: Direction,
    pub powered: bool,
}

impl ObserverState {
    pub const fn default_state() -> Self {
        Self {
            facing: Direction::South,
            powered: false,
        }
    }
}

pub const fn placement_facing(nearest_looking_direction: Direction) -> Direction {
    nearest_looking_direction
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverStartPlan {
    pub schedule_after: Option<u8>,
}

pub fn start_signal(
    server: bool,
    powered: bool,
    facing: Direction,
    direction_to_neighbor: Direction,
    already_scheduled: bool,
) -> ObserverStartPlan {
    let admitted = server && !powered && direction_to_neighbor == facing && !already_scheduled;
    ObserverStartPlan {
        schedule_after: if admitted { Some(EDGE_DELAY) } else { None },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverTickPlan {
    pub offered_powered: bool,
    pub write_flags: u16,
    pub follow_up_after: Option<u8>,
    pub notify_output: bool,
}

pub const fn due_tick(powered: bool) -> ObserverTickPlan {
    ObserverTickPlan {
        offered_powered: !powered,
        write_flags: STATE_WRITE_FLAGS,
        follow_up_after: if powered { None } else { Some(EDGE_DELAY) },
        notify_output: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverReplacementPlan {
    pub offered_powered: Option<bool>,
    pub write_flags: Option<u16>,
    pub notify_output: bool,
}

pub const fn replacement_plan(
    server: bool,
    same_block_identity: bool,
    already_powered: bool,
    pending_tick: bool,
) -> ObserverReplacementPlan {
    if server && !same_block_identity && already_powered && !pending_tick {
        ObserverReplacementPlan {
            offered_powered: Some(false),
            write_flags: Some(REPLACEMENT_CLEAR_FLAGS),
            notify_output: true,
        }
    } else {
        ObserverReplacementPlan {
            offered_powered: None,
            write_flags: None,
            notify_output: false,
        }
    }
}

pub const fn removal_notifies_output(powered: bool, pending_tick: bool) -> bool {
    powered && pending_tick
}

pub const fn output_position_direction(facing: Direction) -> Direction {
    facing.opposite()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverOutputNotification {
    pub output_direction: Direction,
    pub orientation: InitialOrientation,
    pub order: [OutputNotificationStage; 2],
}

pub const fn output_notification(
    facing: Direction,
    redstone_experiments: bool,
) -> ObserverOutputNotification {
    let output_direction = facing.opposite();
    ObserverOutputNotification {
        output_direction,
        orientation: initial_orientation(redstone_experiments, Some(output_direction), None),
        order: OUTPUT_NOTIFICATION_ORDER,
    }
}

pub fn output_signal(powered: bool, facing: Direction, query_direction: Direction) -> u8 {
    if powered && facing == query_direction {
        OUTPUT_SIGNAL
    } else {
        0
    }
}
