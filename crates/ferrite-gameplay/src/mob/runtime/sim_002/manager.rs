//! Raid manager IDs, live-rule retirement, dirty cadence, and reconstruction facts.

#[must_use]
pub const fn unique_id(next_id: i32) -> (i32, i32) {
    let assigned = next_id.wrapping_add(1);
    (assigned, assigned)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagerRaidTick {
    pub manager_tick: u64,
    pub stop_raid: bool,
    pub remove_entry: bool,
    pub tick_raid: bool,
    pub mark_dirty: bool,
    pub remove_raiders: bool,
}

#[must_use]
pub const fn manager_raid_tick(
    manager_tick: u64,
    raids_rule: bool,
    raid_already_stopped: bool,
) -> ManagerRaidTick {
    let manager_tick = manager_tick.saturating_add(1);
    let remove = !raids_rule || raid_already_stopped;
    ManagerRaidTick {
        manager_tick,
        stop_raid: !raids_rule,
        remove_entry: remove,
        tick_raid: !remove,
        mark_dirty: remove || manager_tick.is_multiple_of(200),
        remove_raiders: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopRaid {
    pub active: bool,
    pub clear_bossbar_players: bool,
    pub status_stopped: bool,
}

pub const STOP_RAID: StopRaid = StopRaid {
    active: false,
    clear_bossbar_players: true,
    status_stopped: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistenceFacts {
    pub runtime_groups_persisted: bool,
    pub leaders_persisted: bool,
    pub rng_persisted: bool,
    pub cached_spawn_position_persisted: bool,
    pub celebration_ticks_persisted: bool,
    pub missing_partial_manager_falls_back_dirty: bool,
}

pub const PERSISTENCE_FACTS: PersistenceFacts = PersistenceFacts {
    runtime_groups_persisted: false,
    leaders_persisted: false,
    rng_persisted: false,
    cached_spawn_position_persisted: false,
    celebration_ticks_persisted: false,
    missing_partial_manager_falls_back_dirty: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaiderReattach {
    pub attach: bool,
    pub replace_equal_uuid: bool,
    pub count_health_again: bool,
    pub restore_leader: bool,
}

#[must_use]
pub const fn raider_reattach(
    raid_id_present_and_resolved: bool,
    stored_patrol_leader: bool,
) -> RaiderReattach {
    RaiderReattach {
        attach: raid_id_present_and_resolved,
        replace_equal_uuid: raid_id_present_and_resolved,
        count_health_again: false,
        restore_leader: raid_id_present_and_resolved && stored_patrol_leader,
    }
}
