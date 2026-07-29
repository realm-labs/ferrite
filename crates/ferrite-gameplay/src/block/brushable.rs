//! Brushable-block cooldown, dust, reset, loot, and completion semantics.

use ferrite_foundation::direction::Direction;

pub const BRUSH_COOLDOWN_TICKS: u64 = 10;
pub const RESET_DELAY_TICKS: u64 = 40;
pub const RESET_STEP_TICKS: u64 = 4;
pub const SCHEDULE_DELAY_TICKS: u64 = 2;
pub const FALLING_DURATION_TICKS: u32 = 200;
pub const COMPLETION_EVENT: u16 = 3008;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BrushableState {
    pub count: u8,
    pub dusted: u8,
    pub cooldown_ends: u64,
    pub reset_at: u64,
    pub direction: Option<Direction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushResult {
    CooldownRejected,
    Advanced {
        stage_changed: bool,
        completed: bool,
    },
}

impl BrushableState {
    pub fn brush(&mut self, game_time: u64, direction: Direction) -> BrushResult {
        self.reset_at = game_time + RESET_DELAY_TICKS;
        if game_time < self.cooldown_ends {
            return BrushResult::CooldownRejected;
        }
        self.cooldown_ends = game_time + BRUSH_COOLDOWN_TICKS;
        self.direction = Some(direction);
        self.count = self.count.saturating_add(1).min(10);
        let old_stage = self.dusted;
        self.dusted = dust_stage(self.count);
        BrushResult::Advanced {
            stage_changed: old_stage != self.dusted,
            completed: self.count == 10,
        }
    }

    pub fn reset_tick(&mut self, game_time: u64) -> ResetResult {
        if game_time < self.reset_at || self.count == 0 {
            return ResetResult::NoChange;
        }
        self.count = self.count.saturating_sub(2);
        let old_stage = self.dusted;
        self.dusted = dust_stage(self.count);
        if self.count == 0 {
            self.direction = None;
            self.cooldown_ends = 0;
            self.reset_at = 0;
            ResetResult::Cleared {
                stage_changed: old_stage != 0,
            }
        } else {
            self.reset_at = game_time + RESET_STEP_TICKS;
            ResetResult::Regressed {
                stage_changed: old_stage != self.dusted,
            }
        }
    }

    pub fn persisted(self) -> BrushablePersisted {
        BrushablePersisted {
            direction: self.direction,
        }
    }
}

pub const fn dust_stage(count: u8) -> u8 {
    match count {
        0 => 0,
        1..=2 => 1,
        3..=5 => 2,
        _ => 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetResult {
    NoChange,
    Regressed { stage_changed: bool },
    Cleared { stage_changed: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrushablePersisted {
    pub direction: Option<Direction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EjectedItem {
    pub split_count: u32,
    pub retained_count: u32,
    pub direction: Direction,
}

pub fn complete(stack_count: u32, split_draw: u32, direction: Option<Direction>) -> EjectedItem {
    let offered = 10 + split_draw.min(20);
    EjectedItem {
        split_count: stack_count.min(offered),
        retained_count: 0,
        direction: direction.unwrap_or(Direction::Up),
    }
}

pub fn materialize_loot(results: &[u32]) -> Option<u32> {
    results.first().copied()
}

pub const fn pulse_is_authoritative(remaining_use_ticks: u32) -> bool {
    remaining_use_ticks % 10 == 5
}

pub const fn falling_copies_block_entity_data() -> bool {
    false
}
