//! Brain memory, behavior, activity, sensor, and visibility cadence.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemorySlot {
    pub registered: bool,
    pub populated: bool,
    pub ttl: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTick {
    Unregistered,
    Empty,
    Expired,
    Retained,
}

pub fn tick_memory(slot: &mut MemorySlot) -> MemoryTick {
    if !slot.registered {
        return MemoryTick::Unregistered;
    }
    if !slot.populated {
        return MemoryTick::Empty;
    }
    match slot.ttl {
        Some(ttl) if ttl <= 0 => {
            slot.populated = false;
            MemoryTick::Expired
        }
        Some(ttl) => {
            slot.ttl = Some(ttl - 1);
            MemoryTick::Retained
        }
        None => MemoryTick::Retained,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryWrite {
    IgnoredUnregistered,
    Cleared,
    Stored,
}

pub const fn write_memory(
    slot: &mut MemorySlot,
    value_present_and_nonempty: bool,
    ttl: Option<i64>,
) -> MemoryWrite {
    if !slot.registered {
        MemoryWrite::IgnoredUnregistered
    } else if !value_present_and_nonempty {
        slot.populated = false;
        slot.ttl = None;
        MemoryWrite::Cleared
    } else {
        slot.populated = true;
        slot.ttl = ttl;
        MemoryWrite::Stored
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    Stopped,
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BehaviorState {
    pub status: BehaviorStatus,
    pub end_timestamp: i64,
}

#[must_use]
pub const fn behavior_duration(minimum: u32, maximum: u32, draw: u32) -> u32 {
    if maximum < minimum {
        minimum
    } else {
        minimum + draw % (maximum - minimum + 1)
    }
}

pub const fn try_start_behavior(
    state: &mut BehaviorState,
    memories_match: bool,
    extra_predicate: bool,
    game_time: i64,
    duration: u32,
) -> bool {
    if !matches!(state.status, BehaviorStatus::Stopped) || !memories_match || !extra_predicate {
        return false;
    }
    state.status = BehaviorStatus::Running;
    state.end_timestamp = game_time.wrapping_add(duration as i64);
    true
}

#[must_use]
pub const fn behavior_stops(state: BehaviorState, game_time: i64, can_still_use: bool) -> bool {
    matches!(state.status, BehaviorStatus::Running)
        && (game_time > state.end_timestamp || !can_still_use)
}

#[must_use]
pub const fn schedule_refresh_due(game_time: i64, last_update: i64) -> bool {
    game_time.wrapping_sub(last_update) > 20
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityTransition {
    pub erase_old_activity_memories_except_new: bool,
    pub replace_active_set: bool,
    pub include_all_core_activities: bool,
    pub selected_requested_activity: bool,
    pub use_default_activity: bool,
}

#[must_use]
pub const fn activity_transition(requested_requirements_met: bool) -> ActivityTransition {
    ActivityTransition {
        erase_old_activity_memories_except_new: true,
        replace_active_set: true,
        include_all_core_activities: true,
        selected_requested_activity: requested_requirements_met,
        use_default_activity: !requested_requirements_met,
    }
}

#[must_use]
pub fn first_valid_activity(requirements_in_requested_order: &[bool]) -> Option<usize> {
    requirements_in_requested_order
        .iter()
        .position(|requirements_met| *requirements_met)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrainTickOrder {
    pub expire_memories_first: bool,
    pub sensors_second: bool,
    pub offer_stopped_behaviors_by_priority: bool,
    pub tick_or_stop_running_last: bool,
}

pub const BRAIN_TICK_ORDER: BrainTickOrder = BrainTickOrder {
    expire_memories_first: true,
    sensors_second: true,
    offer_stopped_behaviors_by_priority: true,
    tick_or_stop_running_last: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorTick {
    pub next_time_to_tick: i32,
    pub run: bool,
    pub rewrite_ranges_from_follow_range: bool,
}

#[must_use]
pub const fn sensor_tick(time_to_tick: i32, scan_rate: i32) -> SensorTick {
    let decremented = time_to_tick.saturating_sub(1);
    if decremented <= 0 {
        SensorTick {
            next_time_to_tick: scan_rate,
            run: true,
            rewrite_ranges_from_follow_range: true,
        }
    } else {
        SensorTick {
            next_time_to_tick: decremented,
            run: false,
            rewrite_ranges_from_follow_range: false,
        }
    }
}

#[must_use]
pub const fn initial_sensor_delay(scan_rate: u32, bounded_draw: u32) -> u32 {
    if scan_rate == 0 {
        0
    } else {
        bounded_draw % scan_rate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SightQuery {
    pub perform_clip: bool,
    pub use_cached_result: bool,
}

#[must_use]
pub const fn sight_query(target_cached_seen: bool, target_cached_unseen: bool) -> SightQuery {
    SightQuery {
        perform_clip: !target_cached_seen && !target_cached_unseen,
        use_cached_result: target_cached_seen || target_cached_unseen,
    }
}
