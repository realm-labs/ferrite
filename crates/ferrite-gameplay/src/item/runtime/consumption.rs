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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsumeEffect {
    pub kind: EffectKind,
    pub duration_ticks: u32,
    pub amplifier: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    Regeneration,
    Resistance,
    FireResistance,
    Absorption,
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

const fn effect(kind: EffectKind, duration_ticks: u32, amplifier: u8) -> ConsumeEffect {
    ConsumeEffect {
        kind,
        duration_ticks,
        amplifier,
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
        _ => None,
    }
}

pub fn can_start_consuming(item: ItemKind, food_level: u8) -> bool {
    food_profile(item).is_some_and(|food| food.always_edible || food_level < 20)
}
