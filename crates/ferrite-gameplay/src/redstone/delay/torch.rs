//! Floor/wall torch input, scheduling, toggle history, and burnout semantics.

use std::collections::VecDeque;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::redstone::delay::orientation::{InitialOrientation, initial_orientation};

pub const TOGGLE_DELAY: u8 = 2;
pub const HISTORY_MAX_AGE: u64 = 60;
pub const BURNOUT_THRESHOLD: usize = 8;
pub const RESTART_DELAY: u16 = 160;
pub const BURNOUT_LEVEL_EVENT: u16 = 1502;
pub const STATE_WRITE_FLAGS: u16 = 3;
pub const OUTPUT_SIGNAL: u8 = 15;
pub const DEFAULT_LIT: bool = true;
pub const DEFAULT_WALL_FACING: Direction = Direction::North;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorchAttachment {
    Floor,
    Wall { facing: Direction },
}

pub const fn support_direction(attachment: TorchAttachment) -> Direction {
    match attachment {
        TorchAttachment::Floor => Direction::Down,
        TorchAttachment::Wall { facing } => facing.opposite(),
    }
}

pub const fn input_query_direction(attachment: TorchAttachment) -> Direction {
    support_direction(attachment)
}

pub const fn has_neighbor_signal(sampled_ordinary_signal: u8) -> bool {
    sampled_ordinary_signal > 0
}

pub const fn neighbor_schedule(
    lit: bool,
    neighbor_signal: bool,
    already_due_this_tick: bool,
) -> Option<u8> {
    if lit == neighbor_signal && !already_due_this_tick {
        Some(TOGGLE_DELAY)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorchToggle {
    pub position: BlockPos,
    pub when: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TorchToggleHistory {
    entries: VecDeque<TorchToggle>,
}

impl TorchToggleHistory {
    pub fn from_entries(entries: impl IntoIterator<Item = TorchToggle>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    pub fn entries(&self) -> &VecDeque<TorchToggle> {
        &self.entries
    }

    pub fn purge(&mut self, game_time: u64) -> usize {
        let mut removed = 0;
        while self.entries.front().is_some_and(|entry| {
            game_time >= entry.when && game_time - entry.when > HISTORY_MAX_AGE
        }) {
            self.entries.pop_front();
            removed += 1;
        }
        removed
    }

    pub fn toggled_too_frequently(
        &mut self,
        position: BlockPos,
        game_time: u64,
        record: bool,
    ) -> bool {
        if record {
            self.entries.push_back(TorchToggle {
                position,
                when: game_time,
            });
        }
        self.entries
            .iter()
            .filter(|entry| entry.position == position)
            .count()
            >= BURNOUT_THRESHOLD
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorchTickPlan {
    pub purged_entries: usize,
    pub offered_lit: Option<bool>,
    pub write_flags: Option<u16>,
    pub recorded_toggle: bool,
    pub emitted_level_event: Option<u16>,
    pub restart_after: Option<u16>,
    pub restart_targets_live_block: bool,
    pub burnout_suppressed_relight: bool,
}

pub fn due_tick(
    history: &mut TorchToggleHistory,
    position: BlockPos,
    game_time: u64,
    lit: bool,
    neighbor_signal: bool,
) -> TorchTickPlan {
    let purged_entries = history.purge(game_time);
    if lit && neighbor_signal {
        let burned_out = history.toggled_too_frequently(position, game_time, true);
        return TorchTickPlan {
            purged_entries,
            offered_lit: Some(false),
            write_flags: Some(STATE_WRITE_FLAGS),
            recorded_toggle: true,
            emitted_level_event: if burned_out {
                Some(BURNOUT_LEVEL_EVENT)
            } else {
                None
            },
            restart_after: if burned_out {
                Some(RESTART_DELAY)
            } else {
                None
            },
            restart_targets_live_block: burned_out,
            burnout_suppressed_relight: false,
        };
    }
    if !lit && !neighbor_signal {
        let burned_out = history.toggled_too_frequently(position, game_time, false);
        return TorchTickPlan {
            purged_entries,
            offered_lit: if burned_out { None } else { Some(true) },
            write_flags: if burned_out {
                None
            } else {
                Some(STATE_WRITE_FLAGS)
            },
            recorded_toggle: false,
            emitted_level_event: None,
            restart_after: None,
            restart_targets_live_block: false,
            burnout_suppressed_relight: burned_out,
        };
    }
    TorchTickPlan {
        purged_entries,
        offered_lit: None,
        write_flags: None,
        recorded_toggle: false,
        emitted_level_event: None,
        restart_after: None,
        restart_targets_live_block: false,
        burnout_suppressed_relight: false,
    }
}

pub const fn floor_shape_becomes_air(direction_to_neighbor: Direction, can_survive: bool) -> bool {
    matches!(direction_to_neighbor, Direction::Down) && !can_survive
}

pub fn wall_shape_becomes_air(
    facing: Direction,
    direction_to_neighbor: Direction,
    can_survive: bool,
) -> bool {
    direction_to_neighbor.opposite() == facing && !can_survive
}

pub fn ordinary_signal(attachment: TorchAttachment, lit: bool, query_direction: Direction) -> u8 {
    let excluded = match attachment {
        TorchAttachment::Floor => matches!(query_direction, Direction::Up),
        TorchAttachment::Wall { facing } => query_direction == facing,
    };
    if lit && !excluded { OUTPUT_SIGNAL } else { 0 }
}

pub fn direct_signal(attachment: TorchAttachment, lit: bool, query_direction: Direction) -> u8 {
    if matches!(query_direction, Direction::Down) {
        ordinary_signal(attachment, lit, query_direction)
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorchNotificationPlan {
    pub orientation: InitialOrientation,
    pub directions: [Direction; 6],
}

pub const fn notification_plan(
    attachment: TorchAttachment,
    redstone_experiments: bool,
) -> TorchNotificationPlan {
    TorchNotificationPlan {
        orientation: initial_orientation(
            redstone_experiments,
            match attachment {
                TorchAttachment::Floor => None,
                TorchAttachment::Wall { facing } => Some(facing.opposite()),
            },
            Some(Direction::Up),
        ),
        directions: Direction::ALL,
    }
}

pub const fn placement_notifies_neighbors() -> bool {
    true
}

pub const fn removal_notifies_neighbors(moved_by_piston: bool) -> bool {
    !moved_by_piston
}
