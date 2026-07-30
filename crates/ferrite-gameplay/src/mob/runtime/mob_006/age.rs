//! Signed age, forced growth, age lock, feeding, and love clocks.

pub const DEFAULT_BABY_AGE: i32 = -24_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgeTick {
    pub age: i32,
    pub crossed_zero: bool,
    pub update_synced_baby: bool,
    pub call_boundary_hook: bool,
}

#[must_use]
pub const fn age_tick(age: i32, age_locked: bool, living_server_tick: bool) -> AgeTick {
    let next = if !living_server_tick || age_locked || age == 0 {
        age
    } else if age < 0 {
        age + 1
    } else {
        age - 1
    };
    let crossed = age != 0 && next == 0;
    AgeTick {
        age: next,
        crossed_zero: crossed,
        update_synced_baby: crossed,
        call_boundary_hook: crossed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgeUp {
    pub age: i32,
    pub forced_age: i32,
    pub particle_timer: u8,
    pub crossed_zero: bool,
}

#[must_use]
pub const fn age_up(age: i32, forced_age: i32, seconds: i32, forced: bool) -> AgeUp {
    let requested = age.wrapping_add(seconds.wrapping_mul(20));
    let capped = if requested > 0 { 0 } else { requested };
    let delta = capped.wrapping_sub(age);
    let forced_age = if forced {
        forced_age.wrapping_add(delta)
    } else {
        forced_age
    };
    let crossed_zero = age != 0 && capped == 0;
    AgeUp {
        age: if crossed_zero { forced_age } else { capped },
        forced_age,
        particle_timer: if forced { 40 } else { 0 },
        crossed_zero,
    }
}

#[must_use]
pub const fn feeding_growth_seconds(ticks_until_adult: u32) -> u32 {
    ticks_until_adult / 200
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgeLockToggle {
    pub admitted: bool,
    pub age_locked: bool,
    pub age: i32,
    pub consume_item: bool,
    pub particle_timer: u8,
    pub set_persistence_required: bool,
}

#[must_use]
pub const fn toggle_age_lock(
    baby: bool,
    adult_cooldown_zero: bool,
    cannot_be_age_locked: bool,
    currently_locked: bool,
    species_baby_start: i32,
) -> AgeLockToggle {
    let admitted = baby && adult_cooldown_zero && !cannot_be_age_locked;
    AgeLockToggle {
        admitted,
        age_locked: if admitted {
            !currently_locked
        } else {
            currently_locked
        },
        age: if admitted { species_baby_start } else { 0 },
        consume_item: admitted,
        particle_timer: if admitted { 40 } else { 0 },
        set_persistence_required: admitted && !currently_locked,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodInteraction {
    EnterLove,
    GrowBaby,
    ClientConsume,
    Delegate,
}

#[must_use]
pub const fn food_interaction(
    is_food: bool,
    server_player: bool,
    server_side: bool,
    age: i32,
    love_timer: u16,
    age_locked: bool,
) -> FoodInteraction {
    if is_food && server_player && server_side && age == 0 && love_timer == 0 {
        FoodInteraction::EnterLove
    } else if is_food && server_player && server_side && age < 0 && !age_locked {
        FoodInteraction::GrowBaby
    } else if is_food && !server_side && !(age == 0 && love_timer > 0) && !age_locked {
        FoodInteraction::ClientConsume
    } else {
        FoodInteraction::Delegate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoveTick {
    pub love_timer: u16,
    pub clear: bool,
    pub emit_heart: bool,
}

#[must_use]
pub const fn love_tick(love_timer: u16, age: i32, damaged: bool) -> LoveTick {
    if age != 0 || damaged {
        LoveTick {
            love_timer: 0,
            clear: love_timer > 0,
            emit_heart: false,
        }
    } else {
        let next = love_timer.saturating_sub(1);
        LoveTick {
            love_timer: next,
            clear: false,
            emit_heart: next > 0 && next.is_multiple_of(10),
        }
    }
}
