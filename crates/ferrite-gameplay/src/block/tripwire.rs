//! Tripwire contact, source scans, and two-ended Hook transactions.

use ferrite_foundation::direction::Direction;

pub const TRIPWIRE_BLOCK_ID: u32 = 402;
pub const TRIPWIRE_FIRST_STATE_ID: u32 = 9_599;
pub const TRIPWIRE_LAST_STATE_ID: u32 = 9_726;
pub const TRIPWIRE_STATE_COUNT: usize = 128;
pub const HOOK_SCAN_MAXIMUM: usize = 41;
pub const MAXIMUM_INTERVENING_WIRES: usize = 40;
pub const RESCAN_DELAY_TICKS: u32 = 10;
pub const POWER_SIGNAL: u8 = 15;
pub const SHEARS_WRITE_FLAGS: u16 = 260;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TripwireState {
    pub attached: bool,
    pub disarmed: bool,
    pub east: bool,
    pub north: bool,
    pub powered: bool,
    pub south: bool,
    pub west: bool,
}

impl TripwireState {
    pub const fn connected(self, direction: Direction) -> bool {
        match direction {
            Direction::North => self.north,
            Direction::South => self.south,
            Direction::West => self.west,
            Direction::East => self.east,
            Direction::Down | Direction::Up => false,
        }
    }

    pub fn set_connected(&mut self, direction: Direction, connected: bool) {
        match direction {
            Direction::North => self.north = connected,
            Direction::South => self.south = connected,
            Direction::West => self.west = connected,
            Direction::East => self.east = connected,
            Direction::Down | Direction::Up => {}
        }
    }

    pub const fn clockwise(self) -> Self {
        Self {
            north: self.west,
            east: self.north,
            south: self.east,
            west: self.south,
            ..self
        }
    }

    pub const fn mirror_left_right(self) -> Self {
        Self {
            north: self.south,
            south: self.north,
            ..self
        }
    }

    pub const fn mirror_front_back(self) -> Self {
        Self {
            east: self.west,
            west: self.east,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalNeighbor {
    Tripwire,
    Hook { facing: Direction },
    Other,
}

pub fn connects_to(side: Direction, neighbor: HorizontalNeighbor) -> bool {
    match neighbor {
        HorizontalNeighbor::Tripwire => side.is_horizontal(),
        HorizontalNeighbor::Hook { facing } => side.is_horizontal() && facing == side.opposite(),
        HorizontalNeighbor::Other => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactShape {
    pub minimum_y: f64,
    pub maximum_y: f64,
}

pub const fn contact_shape(attached: bool) -> ContactShape {
    if attached {
        ContactShape {
            minimum_y: 1.0 / 16.0,
            maximum_y: 2.5 / 16.0,
        }
    } else {
        ContactShape {
            minimum_y: 0.0,
            maximum_y: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanCell {
    Wire(TripwireState),
    Hook { facing: Direction },
    Other,
}

pub fn first_source_hook(direction: Direction, cells: &[ScanCell]) -> Option<usize> {
    if !matches!(direction, Direction::South | Direction::West) {
        return None;
    }
    for (index, cell) in cells.iter().take(HOOK_SCAN_MAXIMUM).enumerate() {
        match cell {
            ScanCell::Wire(_) => {}
            ScanCell::Hook { facing } if *facing == direction.opposite() => {
                return Some(index + 1);
            }
            ScanCell::Hook { .. } | ScanCell::Other => return None,
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactInput {
    pub server_side: bool,
    pub currently_powered: bool,
    pub scheduled_tick_pending: bool,
    pub triggering_entity_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactOutcome {
    pub changed: bool,
    pub powered: bool,
    pub write_flags: u8,
    pub recalculate_hook: bool,
    pub wire_rescan_delay: Option<u32>,
    pub release_hook_delay: Option<u32>,
}

pub const fn contact(input: ContactInput) -> ContactOutcome {
    if !input.server_side || input.currently_powered || input.scheduled_tick_pending {
        return unchanged_contact(input.currently_powered);
    }
    if !input.triggering_entity_present {
        return unchanged_contact(false);
    }
    ContactOutcome {
        changed: true,
        powered: true,
        write_flags: 3,
        recalculate_hook: true,
        wire_rescan_delay: Some(RESCAN_DELAY_TICKS),
        release_hook_delay: None,
    }
}

pub const fn scheduled_rescan(
    live_powered: bool,
    triggering_entity_present: bool,
) -> ContactOutcome {
    if !live_powered {
        return unchanged_contact(false);
    }
    if triggering_entity_present {
        return ContactOutcome {
            changed: false,
            powered: true,
            write_flags: 0,
            recalculate_hook: false,
            wire_rescan_delay: Some(RESCAN_DELAY_TICKS),
            release_hook_delay: None,
        };
    }
    ContactOutcome {
        changed: true,
        powered: false,
        write_flags: 3,
        recalculate_hook: true,
        wire_rescan_delay: None,
        release_hook_delay: Some(0),
    }
}

const fn unchanged_contact(powered: bool) -> ContactOutcome {
    ContactOutcome {
        changed: false,
        powered,
        write_flags: 0,
        recalculate_hook: false,
        wire_rescan_delay: None,
        release_hook_delay: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookRecalculation {
    pub opposite_hook_distance: Option<usize>,
    pub attached: bool,
    pub powered: bool,
    pub write_opposite_hook: bool,
    pub write_origin_hook: bool,
    pub rewritten_intermediate_wires: usize,
    pub scheduled_hook_delay: u32,
}

pub fn recalculate_hook(
    origin_facing: Direction,
    cells: &[ScanCell],
    changed_line_removed: bool,
    origin_removed: bool,
    origin_was_attached: bool,
) -> HookRecalculation {
    let mut all_armed = !changed_line_removed;
    let mut any_powered = false;
    let mut wires = 0;
    let mut opposite_hook_distance = None;
    for (index, cell) in cells.iter().take(HOOK_SCAN_MAXIMUM).enumerate() {
        let distance = index + 1;
        match cell {
            ScanCell::Wire(state) => {
                wires += 1;
                all_armed &= !state.disarmed;
                any_powered |= !state.disarmed && state.powered;
            }
            ScanCell::Hook { facing } if distance >= 2 && *facing == origin_facing.opposite() => {
                opposite_hook_distance = Some(distance);
                break;
            }
            ScanCell::Hook { .. } | ScanCell::Other => break,
        }
    }
    let attached = opposite_hook_distance.is_some() && all_armed;
    HookRecalculation {
        opposite_hook_distance,
        attached,
        powered: attached && any_powered,
        write_opposite_hook: opposite_hook_distance.is_some(),
        write_origin_hook: !origin_removed,
        rewritten_intermediate_wires: if attached != origin_was_attached {
            wires
        } else {
            0
        },
        scheduled_hook_delay: RESCAN_DELAY_TICKS,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HookSound {
    Activate { volume: f32, pitch: f32 },
    Deactivate { volume: f32, pitch: f32 },
    Attach { volume: f32, pitch: f32 },
    Detach { volume: f32, pitch: f32 },
}

pub fn hook_sound(
    old_attached: bool,
    old_powered: bool,
    attached: bool,
    powered: bool,
    detach_random: f32,
) -> Option<HookSound> {
    if powered && !old_powered {
        Some(HookSound::Activate {
            volume: 0.4,
            pitch: 0.6,
        })
    } else if !powered && old_powered {
        Some(HookSound::Deactivate {
            volume: 0.4,
            pitch: 0.5,
        })
    } else if attached && !old_attached {
        Some(HookSound::Attach {
            volume: 0.4,
            pitch: 0.7,
        })
    } else if !attached && old_attached {
        Some(HookSound::Detach {
            volume: 0.4,
            pitch: 1.2 / (detach_random * 0.2 + 0.9),
        })
    } else {
        None
    }
}

pub fn hook_signal(powered: bool, queried_side: Direction, facing: Direction) -> (u8, u8) {
    if powered {
        (
            POWER_SIGNAL,
            if queried_side == facing {
                POWER_SIGNAL
            } else {
                0
            },
        )
    } else {
        (0, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShearsDisarm {
    pub disarmed_before_removal: bool,
    pub write_flags: u16,
    pub shear_game_event: bool,
    pub string_loot_suppressed: bool,
}

pub const SHEARS_DISARM: ShearsDisarm = ShearsDisarm {
    disarmed_before_removal: true,
    write_flags: SHEARS_WRITE_FLAGS,
    shear_game_event: true,
    string_loot_suppressed: false,
};
