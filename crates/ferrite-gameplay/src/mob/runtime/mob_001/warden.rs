//! Sculk-shrieker warning synchronization, delayed Warden response, and Darkness.

use crate::mob::runtime::mob_001::hostile::Difficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribution {
    DirectServerPlayer,
    ControllingPassenger,
    ProjectileOwner,
    ItemOwner,
    None,
}

#[must_use]
pub const fn attributed_server_player(attribution: Attribution) -> bool {
    !matches!(attribution, Attribution::None)
}

#[must_use]
pub const fn can_respond(can_summon: bool, difficulty: Difficulty, spawn_wardens: bool) -> bool {
    can_summon && !matches!(difficulty, Difficulty::Peaceful) && spawn_wardens
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WarningTracker {
    pub ticks_since_last_warning: u32,
    pub warning_level: i32,
    pub cooldown_ticks: u32,
}

impl WarningTracker {
    pub fn tick(&mut self) {
        if self.ticks_since_last_warning < 12_000 {
            self.ticks_since_last_warning += 1;
        } else {
            self.warning_level = self.warning_level.wrapping_sub(1).clamp(0, 4);
            self.ticks_since_last_warning = 0;
        }
        self.cooldown_ticks = self.cooldown_ticks.saturating_sub(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningAdmission {
    NearbyWarden,
    Cooldown,
    NoTrackers,
    Admitted { warning_level: i32 },
}

pub fn try_warn(
    nearby_warden: bool,
    collected_trackers_in_player_order: &mut [WarningTracker],
) -> WarningAdmission {
    if nearby_warden {
        return WarningAdmission::NearbyWarden;
    }
    if collected_trackers_in_player_order
        .iter()
        .any(|tracker| tracker.cooldown_ticks > 0)
    {
        return WarningAdmission::Cooldown;
    }
    let Some(maximum) = collected_trackers_in_player_order
        .iter()
        .max_by_key(|tracker| tracker.warning_level)
        .copied()
    else {
        return WarningAdmission::NoTrackers;
    };
    let synchronized = WarningTracker {
        ticks_since_last_warning: 0,
        warning_level: maximum.warning_level.wrapping_add(1).clamp(0, 4),
        cooldown_ticks: 200,
    };
    collected_trackers_in_player_order.fill(synchronized);
    WarningAdmission::Admitted {
        warning_level: synchronized.warning_level,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VibrationAdmission {
    pub listen: bool,
    pub radius: u8,
    pub prefer_projectile_owner: bool,
}

#[must_use]
pub const fn vibration_admission(
    adjacent_chunks_ticking: bool,
    event_in_listen_tag: bool,
    already_shrieking: bool,
    attributed_player: bool,
) -> VibrationAdmission {
    VibrationAdmission {
        listen: adjacent_chunks_ticking
            && event_in_listen_tag
            && !already_shrieking
            && attributed_player,
        radius: 8,
        prefer_projectile_owner: true,
    }
}

#[must_use]
pub fn player_within_warning_radius(distance_squared: f64) -> bool {
    distance_squared < 256.0
}

#[must_use]
pub fn warden_inside_suppression_box(dx: f64, dy: f64, dz: f64) -> bool {
    dx.abs() <= 24.0 && dy.abs() <= 24.0 && dz.abs() <= 24.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShriekCommit {
    pub set_shrieking_flags: u8,
    pub schedule_delay: u8,
    pub level_event: u16,
    pub game_event_with_player: bool,
    pub local_warning_level: i32,
}

#[must_use]
pub const fn shriek_commit(warning: Option<i32>) -> ShriekCommit {
    ShriekCommit {
        set_shrieking_flags: 2,
        schedule_delay: 90,
        level_event: 3007,
        game_event_with_player: true,
        local_warning_level: match warning {
            Some(level) => level,
            None => 0,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduledShriek {
    pub clear_shrieking: bool,
    pub clear_flags: u8,
    pub try_respond_after_clear: bool,
}

#[must_use]
pub const fn scheduled_shriek(incoming_shrieking: bool) -> ScheduledShriek {
    ScheduledShriek {
        clear_shrieking: incoming_shrieking,
        clear_flags: if incoming_shrieking { 3 } else { 0 },
        try_respond_after_clear: incoming_shrieking,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplySound {
    NearbyClose,
    NearbyCloser,
    NearbyClosest,
    ListeningAngry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WardenResponse {
    pub attempt_spawn: bool,
    pub spawn_attempts: u8,
    pub reply_sound: Option<ReplySound>,
    pub reply_offset_draws: u8,
    pub apply_darkness: bool,
}

#[must_use]
pub const fn warden_response(
    gates_pass: bool,
    local_warning_level: i32,
    spawn_succeeded: bool,
) -> WardenResponse {
    if !gates_pass || local_warning_level <= 0 {
        return WardenResponse {
            attempt_spawn: false,
            spawn_attempts: 0,
            reply_sound: None,
            reply_offset_draws: 0,
            apply_darkness: false,
        };
    }
    let attempt_spawn = local_warning_level == 4;
    let success = attempt_spawn && spawn_succeeded;
    let reply_sound = if success {
        None
    } else {
        match local_warning_level {
            1 => Some(ReplySound::NearbyClose),
            2 => Some(ReplySound::NearbyCloser),
            3 => Some(ReplySound::NearbyClosest),
            4 => Some(ReplySound::ListeningAngry),
            _ => None,
        }
    };
    WardenResponse {
        attempt_spawn,
        spawn_attempts: if attempt_spawn { 20 } else { 0 },
        reply_offset_draws: if reply_sound.is_some() { 3 } else { 0 },
        reply_sound,
        apply_darkness: success || reply_sound.is_some(),
    }
}

#[must_use]
pub const fn reply_offset(draw_below_twenty_one: u32) -> i32 {
    -10 + (draw_below_twenty_one % 21) as i32
}

#[must_use]
pub const fn warden_horizontal_offset(draw_below_eleven: u32) -> i32 {
    -5 + (draw_below_eleven % 11) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WardenSearch {
    pub attempts: u8,
    pub horizontal_range: u8,
    pub vertical_range: u8,
    pub ground_cells_per_attempt: u8,
    pub precreation_collision_check: bool,
}

pub const WARDEN_SEARCH: WardenSearch = WardenSearch {
    attempts: 20,
    horizontal_range: 5,
    vertical_range: 6,
    ground_cells_per_attempt: 13,
    precreation_collision_check: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WardenFinalization {
    pub dig_cooldown_ticks: u16,
    pub emerging_memory_ticks: u8,
    pub set_emerging_pose: bool,
    pub play_agitated_before_superclass: bool,
    pub discard_failed_constructed_candidate: bool,
    pub insertion_result_ignored: bool,
}

pub const WARDEN_FINALIZATION: WardenFinalization = WardenFinalization {
    dig_cooldown_ticks: 1_200,
    emerging_memory_ticks: 134,
    set_emerging_pose: true,
    play_agitated_before_superclass: true,
    discard_failed_constructed_candidate: true,
    insertion_result_ignored: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Adventure,
    Creative,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExistingDarkness {
    pub amplifier: u8,
    pub duration: Option<u32>,
}

#[must_use]
pub fn darkness_admitted(
    mode: GameMode,
    distance_squared: f64,
    existing: Option<ExistingDarkness>,
) -> bool {
    if !matches!(mode, GameMode::Survival | GameMode::Adventure) || distance_squared >= 1_600.0 {
        return false;
    }
    match existing {
        None => true,
        Some(effect) => effect.duration.is_some_and(|duration| duration <= 199),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DarknessApplication {
    pub duration: u16,
    pub amplifier: u8,
    pub ambient: bool,
    pub particles: bool,
    pub copy_per_player: bool,
    pub source_is_null: bool,
    pub ignore_application_result: bool,
}

pub const DARKNESS_APPLICATION: DarknessApplication = DarknessApplication {
    duration: 260,
    amplifier: 0,
    ambient: false,
    particles: false,
    copy_per_player: true,
    source_is_null: true,
    ignore_application_result: true,
};
