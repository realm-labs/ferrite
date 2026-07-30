//! Spellcaster, raid equipment, crossbow, and piglin-brute transitions.

use crate::entity::runtime::ent_001::undead::Difficulty;

pub const VINDICATOR_WAVE_COUNTS: [u8; 7] = [0, 2, 0, 1, 4, 2, 5];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvokerSpell {
    SummonVex,
    Fangs,
    Wololo,
}

#[must_use]
pub const fn evoker_spell(
    summon_admitted: bool,
    fang_admitted: bool,
    wololo_admitted: bool,
) -> Option<EvokerSpell> {
    if summon_admitted {
        Some(EvokerSpell::SummonVex)
    } else if fang_admitted {
        Some(EvokerSpell::Fangs)
    } else if wololo_admitted {
        Some(EvokerSpell::Wololo)
    } else {
        None
    }
}

#[must_use]
pub const fn evoker_fang_count(squared_distance: u16) -> u8 {
    if squared_distance < 9 { 13 } else { 16 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VexSummon {
    pub admitted: bool,
    pub attempts: u8,
    pub minimum_life_ticks: u16,
    pub maximum_life_ticks: u16,
}

#[must_use]
pub const fn evoker_vex_summon(nearby_vexes: u8, draw_eight: u8) -> VexSummon {
    VexSummon {
        admitted: draw_eight >= nearby_vexes,
        attempts: 3,
        minimum_life_ticks: 600,
        maximum_life_ticks: 2_380,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IllusionerSpell {
    pub mirror_admitted: bool,
    pub blindness_admitted: bool,
    pub invisibility_ticks: u16,
    pub blindness_ticks: u16,
}

#[must_use]
pub const fn illusioner_spells(
    has_invisibility: bool,
    target_changed: bool,
    regional_difficulty: u8,
) -> IllusionerSpell {
    IllusionerSpell {
        mirror_admitted: !has_invisibility,
        blindness_admitted: target_changed && regional_difficulty > 2,
        invisibility_ticks: 1_200,
        blindness_ticks: 400,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiglinBruteConversion {
    pub time_in_overworld: u16,
    pub converts: bool,
    pub nausea_ticks: u16,
}

#[must_use]
pub const fn piglin_brute_conversion(
    time_in_overworld: u16,
    immune: bool,
    in_safe_dimension: bool,
) -> PiglinBruteConversion {
    if immune || in_safe_dimension {
        return PiglinBruteConversion {
            time_in_overworld: 0,
            converts: false,
            nausea_ticks: 0,
        };
    }
    let next = time_in_overworld.saturating_add(1);
    PiglinBruteConversion {
        time_in_overworld: next,
        converts: next > 300,
        nausea_ticks: if next > 300 { 200 } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossbowState {
    Uncharged,
    Charging,
    Charged,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrossbowStep {
    pub state: CrossbowState,
    pub delay: u8,
    pub shoots: bool,
}

#[must_use]
pub const fn pillager_crossbow_step(
    state: CrossbowState,
    delay: u8,
    charge_duration: u8,
    post_charge_draw: u8,
) -> CrossbowStep {
    match state {
        CrossbowState::Uncharged => CrossbowStep {
            state: CrossbowState::Charging,
            delay: charge_duration,
            shoots: false,
        },
        CrossbowState::Charging if delay > 1 => CrossbowStep {
            state,
            delay: delay - 1,
            shoots: false,
        },
        CrossbowState::Charging => CrossbowStep {
            state: CrossbowState::Charged,
            delay: 20 + post_charge_draw % 20,
            shoots: false,
        },
        CrossbowState::Charged if delay > 1 => CrossbowStep {
            state,
            delay: delay - 1,
            shoots: false,
        },
        CrossbowState::Charged => CrossbowStep {
            state: CrossbowState::Ready,
            delay: 0,
            shoots: false,
        },
        CrossbowState::Ready => CrossbowStep {
            state: CrossbowState::Uncharged,
            delay: 0,
            shoots: true,
        },
    }
}

#[must_use]
pub const fn pillager_charge_duration(quick_charge_level: u8) -> u8 {
    match quick_charge_level {
        0 => 25,
        1 => 20,
        _ => 15,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VindicatorWeapon {
    IronAxe,
    SharpnessOneAxe,
    SharpnessTwoAxe,
}

#[must_use]
pub const fn vindicator_raid_weapon(wave: u8, enchantment_admitted: bool) -> VindicatorWeapon {
    if !enchantment_admitted {
        VindicatorWeapon::IronAxe
    } else if wave > 5 {
        VindicatorWeapon::SharpnessTwoAxe
    } else {
        VindicatorWeapon::SharpnessOneAxe
    }
}

#[must_use]
pub fn johnny_latch(current: bool, flattened_name: Option<&str>) -> bool {
    current || flattened_name == Some("Johnny")
}

#[must_use]
pub const fn vindicator_break_door(
    active_raid: bool,
    mob_griefing: bool,
    difficulty: Difficulty,
    progress: u16,
) -> bool {
    active_raid
        && mob_griefing
        && matches!(difficulty, Difficulty::Normal | Difficulty::Hard)
        && progress >= 240
}

#[must_use]
pub const fn vindicator_wave_count(wave: u8) -> u8 {
    if wave == 0 || wave > VINDICATOR_WAVE_COUNTS.len() as u8 {
        0
    } else {
        VINDICATOR_WAVE_COUNTS[wave as usize - 1]
    }
}
