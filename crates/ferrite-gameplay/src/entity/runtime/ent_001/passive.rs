//! Golem, villager, and wandering-trader state transitions.

pub const IRON_GOLEM_ANGER_MIN_TICKS: u16 = 400;
pub const IRON_GOLEM_ANGER_MAX_TICKS: u16 = 780;
pub const VILLAGER_INVENTORY_SLOTS: usize = 8;
pub const VILLAGER_POI_RADIUS: u8 = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GolemCrackiness {
    None,
    Low,
    Medium,
    High,
}

#[must_use]
pub const fn golem_crackiness(health: u16, max_health: u16) -> GolemCrackiness {
    let four_health = health.saturating_mul(4);
    if four_health < max_health {
        GolemCrackiness::High
    } else if health.saturating_mul(2) < max_health {
        GolemCrackiness::Medium
    } else if four_health < max_health.saturating_mul(3) {
        GolemCrackiness::Low
    } else {
        GolemCrackiness::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GolemRepair {
    pub result_is_success: bool,
    pub healed: u8,
    pub ingot_consumed: bool,
    pub pitch_float_draws: u8,
}

#[must_use]
pub const fn repair_iron_golem(health: u16, max_health: u16) -> GolemRepair {
    if health >= max_health {
        GolemRepair {
            result_is_success: false,
            healed: 0,
            ingot_consumed: false,
            pitch_float_draws: 0,
        }
    } else {
        GolemRepair {
            result_is_success: true,
            healed: 25,
            ingot_consumed: true,
            pitch_float_draws: 2,
        }
    }
}

#[must_use]
pub fn iron_golem_attack_damage(attribute: f64, draw: u32) -> f64 {
    let bound = attribute as u32;
    if bound == 0 {
        attribute
    } else {
        attribute / 2.0 + f64::from(draw % bound)
    }
}

#[must_use]
pub const fn village_golem_candidate(
    slept_age: u32,
    detected_age: u16,
    nearby_eligible_villagers: u8,
) -> bool {
    slept_age <= 24_000 && detected_age >= 599 && nearby_eligible_villagers >= 5
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnowGolemTick {
    pub environmental_damage_attempts: u8,
    pub trail_positions_attempted: u8,
    pub block_place_events: u8,
}

#[must_use]
pub fn snow_golem_tick(
    wet_or_raining: bool,
    biome_or_dimension_melts: bool,
    mob_griefing: bool,
    survivable_air_positions: u8,
) -> SnowGolemTick {
    SnowGolemTick {
        environmental_damage_attempts: u8::from(wet_or_raining)
            + u8::from(biome_or_dimension_melts),
        trail_positions_attempted: if mob_griefing { 4 } else { 0 },
        block_place_events: if mob_griefing {
            survivable_air_positions.min(4)
        } else {
            0
        },
    }
}

#[must_use]
pub const fn snowball_damage(target_is_blaze: bool) -> u8 {
    if target_is_blaze { 3 } else { 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VillagerLevel {
    Novice,
    Apprentice,
    Journeyman,
    Expert,
    Master,
}

#[must_use]
pub const fn villager_level(experience: i32) -> VillagerLevel {
    match experience {
        ..=9 => VillagerLevel::Novice,
        10..=69 => VillagerLevel::Apprentice,
        70..=149 => VillagerLevel::Journeyman,
        150..=249 => VillagerLevel::Expert,
        _ => VillagerLevel::Master,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GossipKind {
    MajorNegative,
    MinorNegative,
    MinorPositive,
    MajorPositive,
    Trading,
}

impl GossipKind {
    #[must_use]
    pub const fn weight(self) -> i16 {
        match self {
            Self::MajorNegative => -5,
            Self::MinorNegative => -1,
            Self::MinorPositive => 1,
            Self::MajorPositive => 5,
            Self::Trading => 1,
        }
    }

    #[must_use]
    pub const fn daily_decay(self) -> u8 {
        match self {
            Self::MajorNegative => 10,
            Self::MinorNegative => 20,
            Self::MinorPositive => 1,
            Self::MajorPositive => 0,
            Self::Trading => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeRestock {
    pub restocks_today: u8,
    pub restock_now: bool,
}

#[must_use]
pub const fn villager_restock(
    restocks_today: u8,
    has_workstation: bool,
    worked_since_last_restock: bool,
) -> TradeRestock {
    let restock_now = restocks_today < 2 && has_workstation && worked_since_last_restock;
    TradeRestock {
        restocks_today: if restock_now {
            restocks_today + 1
        } else {
            restocks_today
        },
        restock_now,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WanderingTraderDespawn {
    pub delay: i32,
    pub discarded: bool,
    pub llama_delay: i32,
}

#[must_use]
pub const fn wandering_trader_despawn(delay: i32, trading: bool) -> WanderingTraderDespawn {
    if delay <= 0 || trading {
        return WanderingTraderDespawn {
            delay,
            discarded: false,
            llama_delay: delay - 1,
        };
    }
    let delay = delay - 1;
    WanderingTraderDespawn {
        delay,
        discarded: delay <= 0,
        llama_delay: delay - 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraderUseResult {
    Pass,
    Success,
    Consumed,
}

#[must_use]
pub const fn wandering_trader_interaction(
    client_side: bool,
    has_offers: bool,
    using_villager_spawn_egg: bool,
) -> TraderUseResult {
    if using_villager_spawn_egg || !has_offers {
        TraderUseResult::Pass
    } else if client_side {
        TraderUseResult::Success
    } else {
        TraderUseResult::Consumed
    }
}
