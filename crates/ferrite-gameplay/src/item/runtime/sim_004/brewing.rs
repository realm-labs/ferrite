//! Potion and container edges owned by SIM-004 material identities.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialBrewingIngredient {
    GlowstoneDust,
    RedstoneDust,
    Gunpowder,
    TurtleHelmet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionKind {
    Water,
    Awkward,
    Thick,
    Mundane,
    NightVision,
    LongNightVision,
    Invisibility,
    LongInvisibility,
    FireResistance,
    LongFireResistance,
    Leaping,
    LongLeaping,
    StrongLeaping,
    Slowness,
    LongSlowness,
    StrongSlowness,
    TurtleMaster,
    LongTurtleMaster,
    StrongTurtleMaster,
    Swiftness,
    LongSwiftness,
    StrongSwiftness,
    WaterBreathing,
    LongWaterBreathing,
    Healing,
    StrongHealing,
    Harming,
    StrongHarming,
    Poison,
    LongPoison,
    StrongPoison,
    Regeneration,
    LongRegeneration,
    StrongRegeneration,
    Strength,
    LongStrength,
    StrongStrength,
    Weakness,
    LongWeakness,
    SlowFalling,
    LongSlowFalling,
}

pub const fn potion_mix(
    ingredient: MaterialBrewingIngredient,
    source: PotionKind,
) -> Option<PotionKind> {
    match ingredient {
        MaterialBrewingIngredient::GlowstoneDust => glowstone_mix(source),
        MaterialBrewingIngredient::RedstoneDust => redstone_mix(source),
        MaterialBrewingIngredient::TurtleHelmet => match source {
            PotionKind::Awkward => Some(PotionKind::TurtleMaster),
            _ => None,
        },
        MaterialBrewingIngredient::Gunpowder => None,
    }
}

const fn glowstone_mix(source: PotionKind) -> Option<PotionKind> {
    match source {
        PotionKind::Water => Some(PotionKind::Thick),
        PotionKind::Leaping => Some(PotionKind::StrongLeaping),
        PotionKind::Slowness => Some(PotionKind::StrongSlowness),
        PotionKind::TurtleMaster => Some(PotionKind::StrongTurtleMaster),
        PotionKind::Swiftness => Some(PotionKind::StrongSwiftness),
        PotionKind::Healing => Some(PotionKind::StrongHealing),
        PotionKind::Harming => Some(PotionKind::StrongHarming),
        PotionKind::Poison => Some(PotionKind::StrongPoison),
        PotionKind::Regeneration => Some(PotionKind::StrongRegeneration),
        PotionKind::Strength => Some(PotionKind::StrongStrength),
        _ => None,
    }
}

const fn redstone_mix(source: PotionKind) -> Option<PotionKind> {
    match source {
        PotionKind::Water => Some(PotionKind::Mundane),
        PotionKind::NightVision => Some(PotionKind::LongNightVision),
        PotionKind::Invisibility => Some(PotionKind::LongInvisibility),
        PotionKind::FireResistance => Some(PotionKind::LongFireResistance),
        PotionKind::Leaping => Some(PotionKind::LongLeaping),
        PotionKind::Slowness => Some(PotionKind::LongSlowness),
        PotionKind::TurtleMaster => Some(PotionKind::LongTurtleMaster),
        PotionKind::Swiftness => Some(PotionKind::LongSwiftness),
        PotionKind::WaterBreathing => Some(PotionKind::LongWaterBreathing),
        PotionKind::Poison => Some(PotionKind::LongPoison),
        PotionKind::Regeneration => Some(PotionKind::LongRegeneration),
        PotionKind::Strength => Some(PotionKind::LongStrength),
        PotionKind::Weakness => Some(PotionKind::LongWeakness),
        PotionKind::SlowFalling => Some(PotionKind::LongSlowFalling),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionContainer {
    Potion,
    SplashPotion,
    LingeringPotion,
}

pub const fn gunpowder_container_mix(source: PotionContainer) -> Option<PotionContainer> {
    match source {
        PotionContainer::Potion => Some(PotionContainer::SplashPotion),
        PotionContainer::SplashPotion | PotionContainer::LingeringPotion => None,
    }
}

pub const fn owned_edge_count(ingredient: MaterialBrewingIngredient) -> usize {
    match ingredient {
        MaterialBrewingIngredient::GlowstoneDust => 10,
        MaterialBrewingIngredient::RedstoneDust => 14,
        MaterialBrewingIngredient::Gunpowder | MaterialBrewingIngredient::TurtleHelmet => 1,
    }
}
