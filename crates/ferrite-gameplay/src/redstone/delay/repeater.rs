//! Repeater delay, side-lock, interaction, shape-update, and output semantics.

use ferrite_foundation::direction::Direction;

use crate::redstone::signal::{ControlSource, control_input};

pub const OUTPUT_SIGNAL: u8 = 15;
pub const USE_WRITE_FLAGS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepeaterState {
    pub facing: Direction,
    pub delay: RepeaterDelay,
    pub locked: bool,
    pub powered: bool,
}

impl RepeaterState {
    pub const fn default_state() -> Self {
        Self {
            facing: Direction::North,
            delay: RepeaterDelay::One,
            locked: false,
            powered: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RepeaterDelay {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl RepeaterDelay {
    pub const fn ticks(self) -> u8 {
        self as u8 * 2
    }

    pub const fn cycled(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::Three,
            Self::Three => Self::Four,
            Self::Four => Self::One,
        }
    }
}

pub const fn is_locked(clockwise: ControlSource, counter_clockwise: ControlSource) -> bool {
    control_input(clockwise, true) > 0 || control_input(counter_clockwise, true) > 0
}

pub const fn placement_locked(clockwise: ControlSource, counter_clockwise: ControlSource) -> bool {
    is_locked(clockwise, counter_clockwise)
}

pub const fn placement_facing(player_horizontal_direction: Direction) -> Direction {
    player_horizontal_direction.opposite()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockShapePlan {
    pub becomes_air: bool,
    pub intended_locked: bool,
    pub server_write_offered: bool,
}

pub fn shape_update(
    direction_to_neighbor: Direction,
    facing: Direction,
    rigid_support: bool,
    current_locked: bool,
    sampled_locked: bool,
    server: bool,
) -> LockShapePlan {
    if direction_to_neighbor == Direction::Down && !rigid_support {
        return LockShapePlan {
            becomes_air: true,
            intended_locked: current_locked,
            server_write_offered: false,
        };
    }
    let resamples_lock = server && direction_to_neighbor.axis() != facing.axis();
    LockShapePlan {
        becomes_air: false,
        intended_locked: if resamples_lock {
            sampled_locked
        } else {
            current_locked
        },
        server_write_offered: resamples_lock && current_locked != sampled_locked,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeaterUseResult {
    Pass,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepeaterUsePlan {
    pub result: RepeaterUseResult,
    pub intended_delay: Option<RepeaterDelay>,
    pub state_write_offered: bool,
    pub write_flags: Option<u16>,
}

pub const fn use_repeater(delay: RepeaterDelay, may_build: bool) -> RepeaterUsePlan {
    if !may_build {
        RepeaterUsePlan {
            result: RepeaterUseResult::Pass,
            intended_delay: None,
            state_write_offered: false,
            write_flags: None,
        }
    } else {
        RepeaterUsePlan {
            result: RepeaterUseResult::Success,
            intended_delay: Some(delay.cycled()),
            state_write_offered: true,
            write_flags: Some(USE_WRITE_FLAGS),
        }
    }
}

pub fn output_signal(powered: bool, facing: Direction, query_direction: Direction) -> u8 {
    if powered && facing == query_direction {
        OUTPUT_SIGNAL
    } else {
        0
    }
}
