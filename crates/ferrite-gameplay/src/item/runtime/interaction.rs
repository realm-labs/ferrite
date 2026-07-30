//! Item-owned entity and block interaction decisions.

use crate::item::runtime::catalog::ItemKind;
use crate::item::runtime::materials::{ItemRole, has_role};

pub const ALLAY_DUPLICATION_COOLDOWN_TICKS: u32 = 6_000;
pub const APPLE_COMPOST_CHANCE: f64 = 0.65;
pub const IRON_GOLEM_HEAL: f32 = 25.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllayState {
    pub dancing: bool,
    pub duplication_cooldown: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllayDuplication {
    Pass,
    ConsumedWithoutSpawn,
    Spawned {
        parent_cooldown: u32,
        child_cooldown: u32,
    },
}

pub fn duplicate_allay(
    item: ItemKind,
    state: AllayState,
    can_spawn_child: bool,
) -> AllayDuplication {
    if !has_role(item, ItemRole::DuplicatesAllays)
        || !state.dancing
        || state.duplication_cooldown != 0
    {
        return AllayDuplication::Pass;
    }
    if !can_spawn_child {
        return AllayDuplication::ConsumedWithoutSpawn;
    }
    AllayDuplication::Spawned {
        parent_cooldown: ALLAY_DUPLICATION_COOLDOWN_TICKS,
        child_cooldown: ALLAY_DUPLICATION_COOLDOWN_TICKS,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorseFood {
    pub heal: f32,
    pub growth_ticks: u32,
    pub temper: u8,
    pub induces_love: bool,
}

pub const fn horse_food(item: ItemKind) -> Option<HorseFood> {
    match item {
        ItemKind::Apple => Some(HorseFood {
            heal: 3.0,
            growth_ticks: 60,
            temper: 3,
            induces_love: false,
        }),
        ItemKind::GoldenApple | ItemKind::EnchantedGoldenApple => Some(HorseFood {
            heal: 10.0,
            growth_ticks: 240,
            temper: 10,
            induces_love: true,
        }),
        _ => None,
    }
}

pub const fn starts_zombie_villager_cure(item: ItemKind, has_weakness: bool) -> bool {
    matches!(item, ItemKind::GoldenApple) && has_weakness
}

pub const fn heals_iron_golem(item: ItemKind, current_health: f32, maximum_health: f32) -> bool {
    matches!(item, ItemKind::IronIngot) && current_health < maximum_health
}

pub const fn compost_chance(item: ItemKind) -> Option<f64> {
    match item {
        ItemKind::Apple => Some(APPLE_COMPOST_CHANCE),
        _ => None,
    }
}
