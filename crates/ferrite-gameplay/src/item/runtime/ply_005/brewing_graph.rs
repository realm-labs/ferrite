//! Exact potion-holder edges owned by PLY-005 ingredient identities.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrewingIngredient {
    BlazePowder,
    BreezeRod,
    GhastTear,
    GlisteringMelonSlice,
    MagmaCream,
    RabbitFoot,
    SpiderEye,
    Sugar,
    GoldenCarrot,
    PhantomMembrane,
    Pufferfish,
    FermentedSpiderEye,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionKind {
    Water,
    Awkward,
    Mundane,
    Strength,
    WindCharged,
    Regeneration,
    Healing,
    FireResistance,
    Leaping,
    Poison,
    Swiftness,
    NightVision,
    SlowFalling,
    WaterBreathing,
    Invisibility,
    Slowness,
    Harming,
    Weakness,
    LongNightVision,
    LongInvisibility,
    LongLeaping,
    LongSlowness,
    LongSwiftness,
    StrongHealing,
    StrongHarming,
    LongPoison,
    StrongPoison,
}

pub const fn mix(ingredient: BrewingIngredient, source: PotionKind) -> Option<PotionKind> {
    use BrewingIngredient::{
        BlazePowder, BreezeRod, FermentedSpiderEye, GhastTear, GlisteringMelonSlice, GoldenCarrot,
        MagmaCream, PhantomMembrane, Pufferfish, RabbitFoot, SpiderEye, Sugar,
    };
    use PotionKind::{
        Awkward, FireResistance, Harming, Healing, Invisibility, Leaping, LongInvisibility,
        LongLeaping, LongNightVision, LongPoison, LongSlowness, LongSwiftness, Mundane,
        NightVision, Poison, Regeneration, SlowFalling, Slowness, Strength, StrongHarming,
        StrongHealing, StrongPoison, Swiftness, Water, WaterBreathing, Weakness, WindCharged,
    };
    match (ingredient, source) {
        (
            BlazePowder | BreezeRod | GhastTear | GlisteringMelonSlice | MagmaCream | RabbitFoot
            | SpiderEye | Sugar,
            Water,
        ) => Some(Mundane),
        (BlazePowder, Awkward) => Some(Strength),
        (BreezeRod, Awkward) => Some(WindCharged),
        (GhastTear, Awkward) => Some(Regeneration),
        (GlisteringMelonSlice, Awkward) => Some(Healing),
        (MagmaCream, Awkward) => Some(FireResistance),
        (RabbitFoot, Awkward) => Some(Leaping),
        (SpiderEye, Awkward) => Some(Poison),
        (Sugar, Awkward) => Some(Swiftness),
        (GoldenCarrot, Awkward) => Some(NightVision),
        (PhantomMembrane, Awkward) => Some(SlowFalling),
        (Pufferfish, Awkward) => Some(WaterBreathing),
        (FermentedSpiderEye, NightVision) => Some(Invisibility),
        (FermentedSpiderEye, LongNightVision) => Some(LongInvisibility),
        (FermentedSpiderEye, Leaping) => Some(Slowness),
        (FermentedSpiderEye, LongLeaping) => Some(LongSlowness),
        (FermentedSpiderEye, Swiftness) => Some(Slowness),
        (FermentedSpiderEye, LongSwiftness) => Some(LongSlowness),
        (FermentedSpiderEye, Healing | Poison | LongPoison) => Some(Harming),
        (FermentedSpiderEye, StrongHealing | StrongPoison) => Some(StrongHarming),
        (FermentedSpiderEye, Water) => Some(Weakness),
        _ => None,
    }
}

pub const fn owned_edge_count(ingredient: BrewingIngredient) -> usize {
    match ingredient {
        BrewingIngredient::BlazePowder
        | BrewingIngredient::BreezeRod
        | BrewingIngredient::GhastTear
        | BrewingIngredient::GlisteringMelonSlice
        | BrewingIngredient::MagmaCream
        | BrewingIngredient::RabbitFoot
        | BrewingIngredient::SpiderEye
        | BrewingIngredient::Sugar => 2,
        BrewingIngredient::GoldenCarrot
        | BrewingIngredient::PhantomMembrane
        | BrewingIngredient::Pufferfish => 1,
        BrewingIngredient::FermentedSpiderEye => 12,
    }
}
