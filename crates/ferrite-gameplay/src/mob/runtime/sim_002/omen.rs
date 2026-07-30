//! Bad/Raid Omen conversion, create-or-reuse admission, center, and absorption.

use crate::mob::runtime::mob_001::hostile::Difficulty;

pub const RAIDS_DEFAULT: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BadOmenConversion {
    pub convert: bool,
    pub raid_omen_duration: u16,
    pub preserve_amplifier: bool,
    pub snapshot_player_position: bool,
}

#[must_use]
pub const fn bad_omen_conversion(
    nonspectator: bool,
    difficulty: Difficulty,
    in_village: bool,
) -> BadOmenConversion {
    let convert = nonspectator && !matches!(difficulty, Difficulty::Peaceful) && in_village;
    BadOmenConversion {
        convert,
        raid_omen_duration: if convert { 600 } else { 0 },
        preserve_amplifier: convert,
        snapshot_player_position: convert,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaidOmenTick {
    pub call_create_or_extend: bool,
    pub clear_saved_position: bool,
    pub remove_effect: bool,
}

#[must_use]
pub const fn raid_omen_tick(remaining_duration: u16) -> RaidOmenTick {
    let expires = remaining_duration == 1;
    RaidOmenTick {
        call_create_or_extend: expires,
        clear_saved_position: expires,
        remove_effect: expires,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateAdmission {
    Spectator,
    RuleDisabled,
    AttributeDisabled,
    Admitted,
}

#[must_use]
pub const fn create_admission(
    spectator: bool,
    raids_rule: bool,
    can_start_raid_attribute: bool,
) -> CreateAdmission {
    if spectator {
        CreateAdmission::Spectator
    } else if !raids_rule {
        CreateAdmission::RuleDisabled
    } else if !can_start_raid_attribute {
        CreateAdmission::AttributeDisabled
    } else {
        CreateAdmission::Admitted
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[must_use]
pub fn raid_center(
    saved_position: BlockPosition,
    occupied_pois_in_iteration_order: &[BlockPosition],
) -> BlockPosition {
    if occupied_pois_in_iteration_order.is_empty() {
        return saved_position;
    }
    let count = occupied_pois_in_iteration_order.len() as i64;
    let sums =
        occupied_pois_in_iteration_order
            .iter()
            .fold((0_i64, 0_i64, 0_i64), |sum, position| {
                (
                    sum.0 + i64::from(position.x),
                    sum.1 + i64::from(position.y),
                    sum.2 + i64::from(position.z),
                )
            });
    BlockPosition {
        x: (sums.0 as f64 / count as f64).floor() as i32,
        y: (sums.1 as f64 / count as f64).floor() as i32,
        z: (sums.2 as f64 / count as f64).floor() as i32,
    }
}

#[must_use]
pub fn reuse_nearest_active(active_distances_squared_in_map_order: &[f64]) -> Option<usize> {
    active_distances_squared_in_map_order
        .iter()
        .enumerate()
        .filter(|(_, distance)| **distance < 9_216.0)
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
}

#[must_use]
pub const fn ordinary_groups(difficulty: Difficulty) -> u8 {
    match difficulty {
        Difficulty::Peaceful => 0,
        Difficulty::Easy => 3,
        Difficulty::Normal => 5,
        Difficulty::Hard => 7,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmenAbsorption {
    pub call_absorb: bool,
    pub new_level: u8,
    pub award_trigger_if_no_wave: bool,
    pub mark_manager_dirty: bool,
}

#[must_use]
pub const fn absorb_omen(
    raid_started: bool,
    current_level: u8,
    effect_present: bool,
    amplifier: u8,
    groups_spawned: u8,
) -> OmenAbsorption {
    let call_absorb = !(raid_started && current_level == 5);
    let added = if call_absorb && effect_present {
        amplifier.saturating_add(1)
    } else {
        0
    };
    let increased = current_level.saturating_add(added);
    OmenAbsorption {
        call_absorb,
        new_level: if increased > 5 { 5 } else { increased },
        award_trigger_if_no_wave: call_absorb && effect_present && groups_spawned == 0,
        mark_manager_dirty: true,
    }
}
