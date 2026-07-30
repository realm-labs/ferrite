//! Raid spawn probes, fixed/random group counts, membership, and horn projection.

use crate::mob::runtime::mob_001::hostile::Difficulty;

pub const FIXED_COUNTS: [[u8; 7]; 5] = [
    [0, 2, 0, 1, 4, 2, 5],
    [0, 0, 0, 0, 1, 1, 2],
    [4, 3, 3, 4, 4, 4, 2],
    [0, 0, 0, 3, 0, 0, 1],
    [0, 0, 1, 0, 1, 0, 2],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaiderType {
    Vindicator,
    Evoker,
    Pillager,
    Witch,
    Ravager,
}

#[must_use]
pub const fn fixed_count(kind: RaiderType, wave: u8) -> u8 {
    let row = match kind {
        RaiderType::Vindicator => 0,
        RaiderType::Evoker => 1,
        RaiderType::Pillager => 2,
        RaiderType::Witch => 3,
        RaiderType::Ravager => 4,
    };
    if wave == 0 || wave > 7 {
        0
    } else {
        FIXED_COUNTS[row][wave as usize - 1]
    }
}

#[must_use]
pub const fn has_bonus_group(omen_level: u8) -> bool {
    omen_level > 1
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnProbe {
    pub attempts: u8,
    pub angle_draws: u8,
    pub jitter_draws_per_attempt: u8,
    pub radial_factor: f32,
    pub outside_village_required: bool,
}

#[must_use]
pub fn spawn_probe(cooldown: u16, wave_time_fallback: bool) -> SpawnProbe {
    SpawnProbe {
        attempts: if wave_time_fallback { 20 } else { 8 },
        angle_draws: 1,
        jitter_draws_per_attempt: 2,
        radial_factor: 0.22 * f32::from(cooldown / 20) - 0.24,
        outside_village_required: cooldown / 20 > 7,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnCandidate {
    pub admitted: bool,
    pub vertical_within_ninety_six: bool,
    pub loaded_margin: u8,
}

#[must_use]
pub const fn spawn_candidate(
    vertical_difference: u32,
    chunks_loaded: bool,
    entity_ticking: bool,
    placement_or_snow_air: bool,
    outside_village_gate: bool,
) -> SpawnCandidate {
    let vertical = vertical_difference <= 96;
    SpawnCandidate {
        admitted: vertical
            && chunks_loaded
            && entity_ticking
            && placement_or_snow_air
            && outside_village_gate,
        vertical_within_ninety_six: vertical,
        loaded_margin: 10,
    }
}

#[must_use]
pub const fn random_extra_bound(
    kind: RaiderType,
    difficulty: Difficulty,
    wave: u8,
    bonus_group: bool,
    easy_initial_draw: u8,
) -> u8 {
    match kind {
        RaiderType::Vindicator | RaiderType::Pillager => match difficulty {
            Difficulty::Easy => easy_initial_draw % 2,
            Difficulty::Normal => 1,
            Difficulty::Hard => 2,
            Difficulty::Peaceful => 0,
        },
        RaiderType::Witch => {
            if !matches!(difficulty, Difficulty::Easy | Difficulty::Peaceful)
                && wave > 2
                && wave != 4
            {
                1
            } else {
                0
            }
        }
        RaiderType::Ravager => {
            if !matches!(difficulty, Difficulty::Easy | Difficulty::Peaceful) && bonus_group {
                1
            } else {
                0
            }
        }
        RaiderType::Evoker => 0,
    }
}

#[must_use]
pub const fn extra_draw_bound(difficulty: Difficulty, maximum: u8) -> u8 {
    if matches!(difficulty, Difficulty::Easy) && maximum > 0 {
        maximum + 1
    } else {
        maximum
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberCommit {
    pub leader_if_first_capable: bool,
    pub add_health_before_insert: bool,
    pub finalize_event: bool,
    pub insertion_result_ignored: bool,
    pub null_ends_only_type_loop: bool,
}

pub const MEMBER_COMMIT: MemberCommit = MemberCommit {
    leader_if_first_capable: true,
    add_health_before_insert: true,
    finalize_event: true,
    insertion_result_ignored: true,
    null_ends_only_type_loop: true,
};

#[must_use]
pub const fn completed_group_count(previous_groups_spawned: u8) -> u8 {
    previous_groups_spawned.saturating_add(1)
}

#[must_use]
pub const fn ravager_rider(wave: u8, ravager_index: u8) -> Option<RaiderType> {
    if wave == 5 {
        Some(RaiderType::Pillager)
    } else if wave >= 7 && ravager_index == 0 {
        Some(RaiderType::Evoker)
    } else if wave >= 7 {
        Some(RaiderType::Vindicator)
    } else {
        None
    }
}

#[must_use]
pub fn horn_recipient(horizontal_distance: f64, current_bossbar_member: bool) -> bool {
    horizontal_distance <= 64.0 || current_bossbar_member
}
