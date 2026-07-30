//! Shared delayed-diode input, scheduling, due-tick, and support semantics.

use ferrite_foundation::direction::Direction;

use crate::redstone::delay::orientation::{
    InitialOrientation, OUTPUT_NOTIFICATION_ORDER, OutputNotificationStage, initial_orientation,
};
use crate::redstone::signal::{SignalSample, combined_signal};

pub const STATE_WRITE_FLAGS: u16 = 2;
pub const PLACEMENT_DELAY: u8 = 1;
pub const SHAPE_HEIGHT: f32 = 2.0 / 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiodeInputSample {
    pub block: SignalSample,
    pub dust_power: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiodeInput {
    pub signal: u8,
    pub dust_queried: bool,
}

pub const fn diode_input(sample: DiodeInputSample) -> DiodeInput {
    let block_signal = combined_signal(sample.block);
    if block_signal == 15 {
        DiodeInput {
            signal: 15,
            dust_queried: false,
        }
    } else {
        let dust = match sample.dust_power {
            Some(power) => power,
            None => 0,
        };
        DiodeInput {
            signal: if block_signal > dust {
                block_signal
            } else {
                dust
            },
            dust_queried: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiodePriority {
    ExtremelyHigh,
    VeryHigh,
    High,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiodeSchedule {
    pub delay: u8,
    pub priority: DiodePriority,
}

pub fn should_prioritize(
    facing: Direction,
    output_is_diode: bool,
    output_diode_facing: Direction,
) -> bool {
    output_is_diode && output_diode_facing != facing.opposite()
}

pub const fn neighbor_schedule(
    powered: bool,
    input_signal: u8,
    locked: bool,
    already_due_this_tick: bool,
    prioritize_output: bool,
    delay: u8,
) -> Option<DiodeSchedule> {
    if locked || already_due_this_tick || powered == (input_signal > 0) {
        return None;
    }
    let priority = if prioritize_output {
        DiodePriority::ExtremelyHigh
    } else if powered {
        DiodePriority::VeryHigh
    } else {
        DiodePriority::High
    };
    Some(DiodeSchedule { delay, priority })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiodeTickPlan {
    pub offered_powered: Option<bool>,
    pub write_flags: Option<u16>,
    pub follow_up: Option<DiodeSchedule>,
}

pub const fn due_tick(powered: bool, input_signal: u8, locked: bool, delay: u8) -> DiodeTickPlan {
    if locked || powered && input_signal > 0 {
        return DiodeTickPlan {
            offered_powered: None,
            write_flags: None,
            follow_up: None,
        };
    }
    if powered {
        return DiodeTickPlan {
            offered_powered: Some(false),
            write_flags: Some(STATE_WRITE_FLAGS),
            follow_up: None,
        };
    }
    DiodeTickPlan {
        offered_powered: Some(true),
        write_flags: Some(STATE_WRITE_FLAGS),
        follow_up: if input_signal == 0 {
            Some(DiodeSchedule {
                delay,
                priority: DiodePriority::VeryHigh,
            })
        } else {
            None
        },
    }
}

pub const fn placement_schedule(input_signal: u8) -> Option<DiodeSchedule> {
    if input_signal > 0 {
        Some(DiodeSchedule {
            delay: PLACEMENT_DELAY,
            priority: DiodePriority::Normal,
        })
    } else {
        None
    }
}

pub const fn on_place_notifies_output() -> bool {
    true
}

pub const fn removal_notifies_output(moved_by_piston: bool) -> bool {
    !moved_by_piston
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiodeOutputNotification {
    pub output_direction: Direction,
    pub orientation: InitialOrientation,
    pub order: [OutputNotificationStage; 2],
}

pub const fn output_notification(
    facing: Direction,
    redstone_experiments: bool,
) -> DiodeOutputNotification {
    let output_direction = facing.opposite();
    DiodeOutputNotification {
        output_direction,
        orientation: initial_orientation(
            redstone_experiments,
            Some(output_direction),
            Some(Direction::Up),
        ),
        order: OUTPUT_NOTIFICATION_ORDER,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiodeSupportLoss {
    pub drop_resources: bool,
    pub remove_moving_false: bool,
    pub notify_all_six_neighbors: bool,
}

pub const fn support_loss(has_rigid_support: bool) -> Option<DiodeSupportLoss> {
    if has_rigid_support {
        None
    } else {
        Some(DiodeSupportLoss {
            drop_resources: true,
            remove_moving_false: true,
            notify_all_six_neighbors: true,
        })
    }
}
