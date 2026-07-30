//! Piston power geometry, neighbor admission, and block-event selection.

use ferrite_foundation::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PistonState {
    pub facing: Direction,
    pub extended: bool,
    pub sticky: bool,
}

impl PistonState {
    pub const fn default_state(sticky: bool) -> Self {
        Self {
            facing: Direction::North,
            extended: false,
            sticky,
        }
    }
}

pub const fn placement_state(sticky: bool, nearest_looking_direction: Direction) -> PistonState {
    PistonState {
        facing: nearest_looking_direction.opposite(),
        extended: false,
        sticky,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerProbe {
    Adjacent(Direction),
    PistonTowardDown,
    AboveAdjacent(Direction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerResult {
    pub powered: bool,
    pub probes: Vec<PowerProbe>,
}

pub fn neighbor_power(
    piston_facing: Direction,
    adjacent: [bool; 6],
    piston_toward_down: bool,
    above_adjacent: [bool; 6],
) -> PowerResult {
    let mut probes = Vec::with_capacity(12);
    for (index, direction) in Direction::ALL.into_iter().enumerate() {
        if direction == piston_facing {
            continue;
        }
        probes.push(PowerProbe::Adjacent(direction));
        if adjacent[index] {
            return PowerResult {
                powered: true,
                probes,
            };
        }
    }
    probes.push(PowerProbe::PistonTowardDown);
    if piston_toward_down {
        return PowerResult {
            powered: true,
            probes,
        };
    }
    for (index, direction) in Direction::ALL.into_iter().enumerate() {
        if direction == Direction::Down {
            continue;
        }
        probes.push(PowerProbe::AboveAdjacent(direction));
        if above_adjacent[index] {
            return PowerResult {
                powered: true,
                probes,
            };
        }
    }
    PowerResult {
        powered: false,
        probes,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PistonEvent {
    Extend = 0,
    Contract = 1,
    Drop = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovingAhead {
    pub is_moving_piston: bool,
    pub facing: Direction,
    pub extending: bool,
    pub progress: f32,
    pub last_ticked: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionCheck {
    pub extension_plan_resolved: bool,
    pub queued_event: Option<PistonEvent>,
}

pub fn transition_check(
    state: PistonState,
    powered: bool,
    extension_resolves: bool,
    two_ahead: Option<MovingAhead>,
    game_time: u64,
    server_handling_tick: bool,
) -> TransitionCheck {
    if powered && !state.extended {
        return TransitionCheck {
            extension_plan_resolved: true,
            queued_event: if extension_resolves {
                Some(PistonEvent::Extend)
            } else {
                None
            },
        };
    }
    if !powered && state.extended {
        let drop = two_ahead.is_some_and(|moving| {
            moving.is_moving_piston
                && moving.facing == state.facing
                && moving.extending
                && (moving.progress < 0.5
                    || game_time == moving.last_ticked
                    || server_handling_tick)
        });
        return TransitionCheck {
            extension_plan_resolved: false,
            queued_event: Some(if drop {
                PistonEvent::Drop
            } else {
                PistonEvent::Contract
            }),
        };
    }
    TransitionCheck {
        extension_plan_resolved: false,
        queued_event: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckCause {
    PlacedBy,
    NeighborChanged,
    OnPlace {
        same_block_identity: bool,
        has_block_entity: bool,
    },
}

pub const fn should_check_extension(server: bool, cause: CheckCause) -> bool {
    if !server {
        return false;
    }
    match cause {
        CheckCause::PlacedBy | CheckCause::NeighborChanged => true,
        CheckCause::OnPlace {
            same_block_identity,
            has_block_entity,
        } => !same_block_identity && !has_block_entity,
    }
}
