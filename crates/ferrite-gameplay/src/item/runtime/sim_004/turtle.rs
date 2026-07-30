//! Turtle adulthood, Scute repair, Helmet refresh, and feeding boundaries.

pub const TURTLE_SCUTE_ITEM_ID: u32 = 916;
pub const TURTLE_HELMET_ITEM_ID: u32 = 915;
pub const BABY_START_AGE: i32 = -24_000;
pub const HELMET_MAXIMUM_DAMAGE: u32 = 275;
pub const HELMET_ARMOR: u8 = 2;
pub const HELMET_ENCHANTABILITY: u8 = 9;
pub const WATER_BREATHING_TICKS: u32 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdulthoodInput {
    pub old_age: i32,
    pub new_age: i32,
    pub server_side: bool,
    pub mob_drops_enabled: bool,
    pub growth_table_emits: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdulthoodOutcome {
    pub crossed_to_adult: bool,
    pub attempted_growth_table: bool,
    pub scutes_emitted: u8,
}

pub const fn adulthood(input: AdulthoodInput) -> AdulthoodOutcome {
    let crossed_to_adult = input.old_age < 0 && input.new_age >= 0;
    let attempted_growth_table = crossed_to_adult && input.server_side && input.mob_drops_enabled;
    AdulthoodOutcome {
        crossed_to_adult,
        attempted_growth_table,
        scutes_emitted: if attempted_growth_table && input.growth_table_emits {
            1
        } else {
            0
        },
    }
}

pub const fn seagrass_acceleration(remaining_ticks: u32) -> u32 {
    20 * ((remaining_ticks / 20) / 10)
}

pub const fn repair_per_scute(maximum_damage: u32) -> u32 {
    maximum_damage / 4
}

pub const fn scutes_to_repair(damage: u32, maximum_damage: u32) -> Option<u32> {
    if damage == 0 {
        return Some(0);
    }
    let repair = repair_per_scute(maximum_damage);
    if repair == 0 {
        return None;
    }
    Some(damage.div_ceil(repair))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelmetRefresh {
    pub duration_ticks: u32,
    pub amplifier: u8,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
}

pub const fn helmet_refresh(
    exact_helmet_equipped: bool,
    eyes_in_water: bool,
) -> Option<HelmetRefresh> {
    if !exact_helmet_equipped || eyes_in_water {
        return None;
    }
    Some(HelmetRefresh {
        duration_ticks: WATER_BREATHING_TICKS,
        amplifier: 0,
        ambient: false,
        show_particles: false,
        show_icon: true,
    })
}
