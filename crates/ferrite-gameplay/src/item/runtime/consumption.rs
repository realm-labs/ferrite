//! Audited default food and ordered consume-effect profiles.

use crate::item::runtime::catalog::ItemKind;

pub const DEFAULT_EAT_TICKS: u16 = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodProfile {
    pub nutrition: u8,
    pub saturation: f32,
    pub always_edible: bool,
    pub effects: &'static [ConsumeEffect],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeEffect {
    pub kind: EffectKind,
    pub duration_ticks: u32,
    pub amplifier: u8,
    pub probability: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    Regeneration,
    Resistance,
    FireResistance,
    Absorption,
    Hunger,
}

const GOLDEN_APPLE_EFFECTS: [ConsumeEffect; 2] = [
    effect(EffectKind::Regeneration, 100, 1),
    effect(EffectKind::Absorption, 2_400, 0),
];
const ENCHANTED_GOLDEN_APPLE_EFFECTS: [ConsumeEffect; 4] = [
    effect(EffectKind::Regeneration, 400, 1),
    effect(EffectKind::Resistance, 6_000, 0),
    effect(EffectKind::FireResistance, 6_000, 0),
    effect(EffectKind::Absorption, 2_400, 3),
];
const RAW_CHICKEN_EFFECTS: [ConsumeEffect; 1] = [chance_effect(EffectKind::Hunger, 600, 0, 0.3)];

const fn effect(kind: EffectKind, duration_ticks: u32, amplifier: u8) -> ConsumeEffect {
    ConsumeEffect {
        kind,
        duration_ticks,
        amplifier,
        probability: 1.0,
    }
}

const fn chance_effect(
    kind: EffectKind,
    duration_ticks: u32,
    amplifier: u8,
    probability: f32,
) -> ConsumeEffect {
    ConsumeEffect {
        kind,
        duration_ticks,
        amplifier,
        probability,
    }
}

pub const fn food_profile(item: ItemKind) -> Option<FoodProfile> {
    match item {
        ItemKind::Apple => Some(FoodProfile {
            nutrition: 4,
            saturation: 2.4,
            always_edible: false,
            effects: &[],
        }),
        ItemKind::GoldenApple => Some(FoodProfile {
            nutrition: 4,
            saturation: 9.6,
            always_edible: true,
            effects: &GOLDEN_APPLE_EFFECTS,
        }),
        ItemKind::EnchantedGoldenApple => Some(FoodProfile {
            nutrition: 4,
            saturation: 9.6,
            always_edible: true,
            effects: &ENCHANTED_GOLDEN_APPLE_EFFECTS,
        }),
        ItemKind::BakedPotato => ordinary_food(5, 6.0),
        ItemKind::Beef => ordinary_food(3, 1.800_000_1),
        ItemKind::CookedBeef => ordinary_food(8, 12.8),
        ItemKind::Chicken => Some(FoodProfile {
            nutrition: 2,
            saturation: 1.2,
            always_edible: false,
            effects: &RAW_CHICKEN_EFFECTS,
        }),
        ItemKind::CookedChicken => ordinary_food(6, 7.200_000_3),
        ItemKind::Cookie => ordinary_food(2, 0.4),
        ItemKind::Mutton => ordinary_food(2, 1.2),
        ItemKind::CookedMutton => ordinary_food(6, 9.6),
        ItemKind::Porkchop => ordinary_food(3, 1.800_000_1),
        ItemKind::CookedPorkchop => ordinary_food(8, 12.8),
        ItemKind::PumpkinPie => ordinary_food(8, 4.8),
        ItemKind::Rabbit => ordinary_food(3, 1.800_000_1),
        ItemKind::CookedRabbit => ordinary_food(5, 6.0),
        _ => None,
    }
}

const fn ordinary_food(nutrition: u8, saturation: f32) -> Option<FoodProfile> {
    Some(FoodProfile {
        nutrition,
        saturation,
        always_edible: false,
        effects: &[],
    })
}

pub fn can_start_consuming(item: ItemKind, food_level: u8) -> bool {
    food_profile(item).is_some_and(|food| food.always_edible || food_level < 20)
}
