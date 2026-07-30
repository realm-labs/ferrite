//! Food, drink, ominous bottle, stew, and potion consume listeners.

use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumableKind {
    Bread,
    Cod,
    CookedCod,
    GoldenCarrot,
    Pufferfish,
    RottenFlesh,
    Salmon,
    CookedSalmon,
    SpiderEye,
    TropicalFish,
    BeetrootSoup,
    MushroomStew,
    RabbitStew,
    SuspiciousStew,
    HoneyBottle,
    MilkBucket,
    OminousBottle,
    Potion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumableProfile {
    pub duration_ticks: u32,
    pub nutrition: u8,
    pub saturation: f32,
    pub always_edible: bool,
    pub remainder: Remainder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Remainder {
    None,
    Bowl,
    GlassBottle,
    Bucket,
}

pub const fn profile(kind: ConsumableKind) -> ConsumableProfile {
    match kind {
        ConsumableKind::Bread => food(5, 6.0, Remainder::None),
        ConsumableKind::Cod => food(2, 0.4, Remainder::None),
        ConsumableKind::CookedCod => food(5, 6.0, Remainder::None),
        ConsumableKind::GoldenCarrot => food(6, 14.4, Remainder::None),
        ConsumableKind::Pufferfish => food(1, 0.2, Remainder::None),
        ConsumableKind::RottenFlesh => food(4, 0.8, Remainder::None),
        ConsumableKind::Salmon => food(2, 0.4, Remainder::None),
        ConsumableKind::CookedSalmon => food(6, 9.6, Remainder::None),
        ConsumableKind::SpiderEye => food(2, 3.2, Remainder::None),
        ConsumableKind::TropicalFish => food(1, 0.2, Remainder::None),
        ConsumableKind::BeetrootSoup | ConsumableKind::MushroomStew => {
            food(6, 7.200_000_3, Remainder::Bowl)
        }
        ConsumableKind::RabbitStew => food(10, 12.0, Remainder::Bowl),
        ConsumableKind::SuspiciousStew => ConsumableProfile {
            always_edible: true,
            ..food(6, 7.200_000_3, Remainder::Bowl)
        },
        ConsumableKind::HoneyBottle => ConsumableProfile {
            duration_ticks: 40,
            nutrition: 6,
            saturation: 1.2,
            always_edible: false,
            remainder: Remainder::GlassBottle,
        },
        ConsumableKind::MilkBucket => ConsumableProfile {
            duration_ticks: 32,
            nutrition: 0,
            saturation: 0.0,
            always_edible: true,
            remainder: Remainder::Bucket,
        },
        ConsumableKind::OminousBottle | ConsumableKind::Potion => ConsumableProfile {
            duration_ticks: 32,
            nutrition: 0,
            saturation: 0.0,
            always_edible: true,
            remainder: if matches!(kind, ConsumableKind::Potion) {
                Remainder::GlassBottle
            } else {
                Remainder::None
            },
        },
    }
}

const fn food(nutrition: u8, saturation: f32, remainder: Remainder) -> ConsumableProfile {
    ConsumableProfile {
        duration_ticks: 32,
        nutrition,
        saturation,
        always_edible: false,
        remainder,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumeEffect {
    Poison { duration: u32, amplifier: u8 },
    Hunger { duration: u32, amplifier: u8 },
    Nausea { duration: u32, amplifier: u8 },
    ClearPoison,
    ClearAllEffects,
    BadOmen { duration: u32, amplifier: u8 },
    Potion(PotionEffect),
}

pub fn effects(kind: ConsumableKind, chance: f32, ominous_level: u8) -> Vec<ConsumeEffect> {
    match kind {
        ConsumableKind::Pufferfish => vec![
            ConsumeEffect::Poison {
                duration: 1_200,
                amplifier: 1,
            },
            ConsumeEffect::Hunger {
                duration: 300,
                amplifier: 2,
            },
            ConsumeEffect::Nausea {
                duration: 300,
                amplifier: 0,
            },
        ],
        ConsumableKind::RottenFlesh if chance < 0.8_f32 => {
            vec![ConsumeEffect::Hunger {
                duration: 600,
                amplifier: 0,
            }]
        }
        ConsumableKind::SpiderEye => vec![ConsumeEffect::Poison {
            duration: 100,
            amplifier: 0,
        }],
        ConsumableKind::HoneyBottle => vec![ConsumeEffect::ClearPoison],
        ConsumableKind::MilkBucket => vec![ConsumeEffect::ClearAllEffects],
        ConsumableKind::OminousBottle => vec![ConsumeEffect::BadOmen {
            duration: 120_000,
            amplifier: ominous_level.min(4),
        }],
        _ => Vec::new(),
    }
}

pub const fn effect_probability_draws(kind: ConsumableKind) -> u8 {
    if matches!(
        kind,
        ConsumableKind::Pufferfish | ConsumableKind::RottenFlesh | ConsumableKind::SpiderEye
    ) {
        1
    } else {
        0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionEffect {
    pub effect: ResourceId,
    pub duration: i32,
    pub amplifier: u8,
    pub instantaneous: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionContents {
    pub base_effects: Vec<PotionEffect>,
    pub custom_effects: Vec<PotionEffect>,
    pub duration_scale_bits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuspiciousStewEffect {
    pub effect: ResourceId,
    pub duration: u32,
}

pub fn suspicious_stew_effects(effects: &[SuspiciousStewEffect]) -> Vec<SuspiciousStewEffect> {
    effects.to_vec()
}

pub const fn suspicious_stew_default_duration(encoded: Option<u32>) -> u32 {
    match encoded {
        Some(duration) => duration,
        None => 160,
    }
}

pub const fn stew_remainder_count(
    pre_use_count: u32,
    post_use_count: u32,
    infinite_materials: bool,
) -> u8 {
    if !infinite_materials && post_use_count < pre_use_count {
        1
    } else {
        0
    }
}

pub fn scaled_potion_effects(contents: &PotionContents) -> Vec<PotionEffect> {
    let scale = f32::from_bits(contents.duration_scale_bits);
    contents
        .base_effects
        .iter()
        .chain(&contents.custom_effects)
        .cloned()
        .map(|mut effect| {
            if effect.duration > 0 && !effect.instantaneous {
                effect.duration = ((effect.duration as f32 * scale).floor() as i32).max(1);
            }
            effect
        })
        .collect()
}

pub const fn water_transaction(holder_is_water: bool, custom_effects_empty: bool) -> bool {
    holder_is_water && custom_effects_empty
}

pub const fn periodic_drink_sound(duration: u32, remaining: u32) -> bool {
    let elapsed_gate = duration.saturating_mul(7) / 32;
    remaining <= duration.saturating_sub(elapsed_gate).saturating_sub(1)
        && remaining >= 4
        && remaining.is_multiple_of(4)
}

pub const fn ominous_level(encoded: i32) -> u8 {
    if encoded < 0 {
        0
    } else if encoded > 4 {
        4
    } else {
        encoded as u8
    }
}
