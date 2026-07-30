//! Skeleton-family combat, conversion, sunlight, and projectile modifiers.

pub const SKELETON_POWDER_SNOW_EXPOSURE_TICKS: i32 = 140;
pub const SKELETON_STRAY_CONVERSION_TICKS: i32 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }
}

#[must_use]
pub const fn skeleton_attack_interval(difficulty: Difficulty) -> u8 {
    if matches!(difficulty, Difficulty::Hard) {
        20
    } else {
        40
    }
}

#[must_use]
pub const fn slow_skeleton_attack_interval(difficulty: Difficulty) -> u8 {
    if matches!(difficulty, Difficulty::Hard) {
        50
    } else {
        70
    }
}

#[must_use]
pub const fn arrow_inaccuracy(difficulty: Difficulty) -> u8 {
    14 - 4 * difficulty.id()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowEffect {
    None,
    Poison { duration: u16 },
    Weakness { duration: u16 },
    Slowness { duration: u16 },
    Fire { duration: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonKind {
    Skeleton,
    Bogged,
    Parched,
    Stray,
    WitherSkeleton,
}

#[must_use]
pub const fn arrow_effect(kind: SkeletonKind, concrete_arrow: bool) -> ArrowEffect {
    if !concrete_arrow && !matches!(kind, SkeletonKind::WitherSkeleton) {
        return ArrowEffect::None;
    }
    match kind {
        SkeletonKind::Skeleton => ArrowEffect::None,
        SkeletonKind::Bogged => ArrowEffect::Poison { duration: 100 },
        SkeletonKind::Parched => ArrowEffect::Weakness { duration: 600 },
        SkeletonKind::Stray => ArrowEffect::Slowness { duration: 600 },
        SkeletonKind::WitherSkeleton => ArrowEffect::Fire { duration: 2_000 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkeletonConversion {
    pub exposure_ticks: i32,
    pub conversion_ticks: i32,
    pub converting: bool,
    pub converts_now: bool,
}

#[must_use]
pub const fn skeleton_powder_snow_step(
    mut state: SkeletonConversion,
    in_powder_snow: bool,
    alive: bool,
    effective_ai: bool,
) -> SkeletonConversion {
    state.converts_now = false;
    if !alive || !effective_ai {
        return state;
    }
    if !in_powder_snow {
        state.exposure_ticks = -1;
        state.converting = false;
        state.conversion_ticks = 0;
        return state;
    }
    if state.converting {
        state.conversion_ticks = state.conversion_ticks.wrapping_sub(1);
        state.converts_now = state.conversion_ticks < 0;
        return state;
    }
    state.exposure_ticks += 1;
    if state.exposure_ticks >= SKELETON_POWDER_SNOW_EXPOSURE_TICKS {
        state.converting = true;
        state.conversion_ticks = SKELETON_STRAY_CONVERSION_TICKS;
    }
    state
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaylightStep {
    pub burns: bool,
    pub head_item_damaged: bool,
    pub head_item_broken: bool,
}

#[must_use]
pub const fn daylight_step(
    daylight_burns: bool,
    has_head_item: bool,
    damage_draw_breaks_item: bool,
) -> DaylightStep {
    if !daylight_burns {
        return DaylightStep {
            burns: false,
            head_item_damaged: false,
            head_item_broken: false,
        };
    }
    if has_head_item {
        DaylightStep {
            burns: false,
            head_item_damaged: true,
            head_item_broken: damage_draw_breaks_item,
        }
    } else {
        DaylightStep {
            burns: true,
            head_item_damaged: false,
            head_item_broken: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoggedShear {
    pub sheared: bool,
    pub durability_spent: bool,
    pub game_event_emitted: bool,
    pub brown_mushrooms: u8,
    pub red_mushrooms: u8,
}

#[must_use]
pub const fn shear_bogged(
    server_side: bool,
    already_sheared: bool,
    first_draw: u8,
    second_draw: u8,
) -> BoggedShear {
    if !server_side || already_sheared {
        return BoggedShear {
            sheared: already_sheared,
            durability_spent: false,
            game_event_emitted: false,
            brown_mushrooms: 0,
            red_mushrooms: 0,
        };
    }
    let brown_mushrooms =
        (first_draw.is_multiple_of(2) as u8) + (second_draw.is_multiple_of(2) as u8);
    BoggedShear {
        sheared: true,
        durability_spent: true,
        game_event_emitted: true,
        brown_mushrooms,
        red_mushrooms: 2 - brown_mushrooms,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitherMelee {
    pub base_damage_attempted: bool,
    pub wither_duration: u16,
}

#[must_use]
pub const fn wither_melee(base_damage_admitted: bool) -> WitherMelee {
    WitherMelee {
        base_damage_attempted: true,
        wither_duration: if base_damage_admitted { 200 } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParchedJockey {
    pub equips_iron_spear: bool,
    pub creates_camel: bool,
    pub creates_husk: bool,
    pub camel_passenger_count: u8,
}

#[must_use]
pub const fn parched_jockey(draw_ten: u8, camel_husk_box_clear: bool) -> ParchedJockey {
    let creates = draw_ten == 0 && camel_husk_box_clear;
    ParchedJockey {
        equips_iron_spear: creates,
        creates_camel: creates,
        creates_husk: creates,
        camel_passenger_count: if creates { 2 } else { 0 },
    }
}
