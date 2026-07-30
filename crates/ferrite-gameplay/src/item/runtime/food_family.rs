//! Item-owned cooking, animal-food, compost and poisonous-food profiles.

use crate::item::runtime::catalog::ItemKind;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CookingProfile {
    pub input: &'static str,
    pub output: ItemKind,
    pub furnace_ticks: u16,
    pub smoker_ticks: u16,
    pub campfire_ticks: u16,
    pub furnace_experience: f32,
}

const fn cooking(input: &'static str, output: ItemKind, experience: f32) -> CookingProfile {
    CookingProfile {
        input,
        output,
        furnace_ticks: 200,
        smoker_ticks: 100,
        campfire_ticks: 600,
        furnace_experience: experience,
    }
}

pub const COOKING: [CookingProfile; 6] = [
    cooking("potato", ItemKind::BakedPotato, 0.35),
    cooking("beef", ItemKind::CookedBeef, 0.35),
    cooking("chicken", ItemKind::CookedChicken, 0.35),
    cooking("mutton", ItemKind::CookedMutton, 0.35),
    cooking("porkchop", ItemKind::CookedPorkchop, 0.35),
    cooking("rabbit", ItemKind::CookedRabbit, 0.35),
];

pub const fn wolf_healing(item: ItemKind) -> Option<u8> {
    match item {
        ItemKind::Beef => Some(6),
        ItemKind::CookedBeef => Some(16),
        ItemKind::Chicken | ItemKind::Mutton => Some(4),
        ItemKind::CookedChicken | ItemKind::CookedMutton => Some(12),
        ItemKind::Porkchop | ItemKind::Rabbit => Some(6),
        ItemKind::CookedPorkchop => Some(16),
        ItemKind::CookedRabbit => Some(10),
        _ => None,
    }
}

pub const fn is_piglin_food(item: ItemKind) -> bool {
    matches!(item, ItemKind::Porkchop | ItemKind::CookedPorkchop)
}

pub const fn compost_chance(item: ItemKind) -> Option<f32> {
    match item {
        ItemKind::BakedPotato | ItemKind::Cookie => Some(0.85),
        ItemKind::PumpkinPie => Some(1.0),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParrotPoison {
    pub poison_ticks: u16,
    pub requested_damage: Option<f32>,
    pub uses_food_transaction: bool,
}

pub const fn parrot_poison(item: ItemKind, player_source: bool) -> Option<ParrotPoison> {
    if !matches!(item, ItemKind::Cookie) {
        return None;
    }
    Some(ParrotPoison {
        poison_ticks: 900,
        requested_damage: if player_source { Some(f32::MAX) } else { None },
        uses_food_transaction: false,
    })
}
