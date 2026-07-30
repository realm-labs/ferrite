//! Ongoing raid state, cooldown, cleanup, victory, celebration, and rewards.

use crate::mob::runtime::mob_001::hostile::Difficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaidStatus {
    Ongoing,
    Victory,
    Loss,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveAdmission {
    pub active: bool,
    pub update_bossbar_visibility: bool,
    pub stop_peaceful: bool,
    pub continue_tick: bool,
}

#[must_use]
pub const fn active_admission(
    was_active: bool,
    center_chunk_loaded: bool,
    difficulty: Difficulty,
) -> ActiveAdmission {
    let active = center_chunk_loaded;
    let peaceful = matches!(difficulty, Difficulty::Peaceful);
    ActiveAdmission {
        active,
        update_bossbar_visibility: was_active != active,
        stop_peaceful: peaceful,
        continue_tick: active && !peaceful,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LostVillage {
    KeepCenter,
    MoveCenter,
    StopNoGroups,
    MarkLoss,
}

#[must_use]
pub const fn lost_village(
    center_still_village: bool,
    nearby_village_section_found: bool,
    groups_spawned: u8,
) -> LostVillage {
    if center_still_village {
        LostVillage::KeepCenter
    } else if nearby_village_section_found {
        LostVillage::MoveCenter
    } else if groups_spawned == 0 {
        LostVillage::StopNoGroups
    } else {
        LostVillage::MarkLoss
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveCounter {
    pub ticks_active: u64,
    pub stop_timeout: bool,
    pub cleanup_due: bool,
}

#[must_use]
pub const fn active_counter(ticks_active: u64) -> ActiveCounter {
    let ticks_active = ticks_active.saturating_add(1);
    ActiveCounter {
        ticks_active,
        stop_timeout: ticks_active >= 48_000,
        cleanup_due: ticks_active.is_multiple_of(20),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CooldownTick {
    pub cooldown: u16,
    pub recompute_spawn_position: bool,
    pub refresh_membership: bool,
    pub progress_numerator: u16,
}

#[must_use]
pub const fn cooldown_tick(
    cooldown: u16,
    cached_position_present: bool,
    cached_position_entity_ticking: bool,
) -> CooldownTick {
    let recompute = (!cached_position_present && cooldown.is_multiple_of(5))
        || (cached_position_present && !cached_position_entity_ticking);
    CooldownTick {
        cooldown: cooldown.saturating_sub(1),
        recompute_spawn_position: recompute,
        refresh_membership: cooldown == 300 || cooldown.is_multiple_of(20),
        progress_numerator: 300_u16.saturating_sub(cooldown),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveAdmission {
    ResetCooldownAndTitle,
    Spawn,
    StopAfterFailedProbes,
}

#[must_use]
pub const fn wave_admission(
    later_wave_without_raiders: bool,
    cooldown: u16,
    failed_outer_probes: u8,
) -> WaveAdmission {
    if later_wave_without_raiders && cooldown == 0 {
        WaveAdmission::ResetCooldownAndTitle
    } else if failed_outer_probes >= 6 {
        WaveAdmission::StopAfterFailedProbes
    } else {
        WaveAdmission::Spawn
    }
}

#[must_use]
pub const fn wave_name_suffix(pre_cleanup_raider_count: usize) -> Option<u8> {
    if pre_cleanup_raider_count == 1 {
        Some(1)
    } else if pre_cleanup_raider_count == 2 {
        Some(2)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostRaid {
    Wait { post_raid_ticks: u8 },
    Victory,
}

#[must_use]
pub const fn post_raid_tick(post_raid_ticks: u8) -> PostRaid {
    if post_raid_ticks < 40 {
        PostRaid::Wait {
            post_raid_ticks: post_raid_ticks + 1,
        }
    } else {
        PostRaid::Victory
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaiderCleanupInput {
    pub removed: bool,
    pub another_dimension: bool,
    pub distance_squared: u32,
    pub entity_age: u32,
    pub uuid_resolves: bool,
    pub no_action_time: u32,
    pub outside_village_checks: u8,
    pub outside_village: bool,
}

#[must_use]
pub const fn remove_tracked_raider(input: RaiderCleanupInput) -> bool {
    input.removed
        || input.another_dimension
        || input.distance_squared >= 12_544
        || (input.entity_age >= 600 && !input.uuid_resolves)
        || (input.outside_village_checks >= 30
            && input.no_action_time > 2_400
            && input.outside_village)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroReward {
    pub admitted: bool,
    pub duration: u32,
    pub amplifier: u8,
    pub hidden_particles: bool,
    pub visible_icon: bool,
    pub award_player_stat_and_criterion: bool,
}

#[must_use]
pub const fn hero_reward(
    resolves_living_nonspectator: bool,
    player: bool,
    raid_omen_level: u8,
) -> HeroReward {
    HeroReward {
        admitted: resolves_living_nonspectator,
        duration: 48_000,
        amplifier: raid_omen_level.saturating_sub(1),
        hidden_particles: true,
        visible_icon: true,
        award_player_stat_and_criterion: resolves_living_nonspectator && player,
    }
}

#[must_use]
pub const fn celebration_stops(celebration_ticks: u16) -> bool {
    celebration_ticks >= 600
}
