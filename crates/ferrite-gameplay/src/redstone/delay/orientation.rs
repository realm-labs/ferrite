//! Experimental neighbor-orientation draw and fixed-axis metadata.

use ferrite_foundation::direction::Direction;

pub const ORIENTATION_BOUND: u8 = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitialOrientation {
    pub draw_consumed: bool,
    pub bound: Option<u8>,
    pub fixed_front: Option<Direction>,
    pub fixed_up: Option<Direction>,
}

pub const fn initial_orientation(
    redstone_experiments: bool,
    fixed_front: Option<Direction>,
    fixed_up: Option<Direction>,
) -> InitialOrientation {
    InitialOrientation {
        draw_consumed: redstone_experiments,
        bound: if redstone_experiments {
            Some(ORIENTATION_BOUND)
        } else {
            None
        },
        fixed_front: if redstone_experiments {
            fixed_front
        } else {
            None
        },
        fixed_up: if redstone_experiments { fixed_up } else { None },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputNotificationStage {
    NeighborChanged,
    NeighborsExceptFacing,
}

pub const OUTPUT_NOTIFICATION_ORDER: [OutputNotificationStage; 2] = [
    OutputNotificationStage::NeighborChanged,
    OutputNotificationStage::NeighborsExceptFacing,
];
