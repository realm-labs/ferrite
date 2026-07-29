//! Bamboo, Cactus, Sugar Cane, cave-vine, Nether-vine, and sapling semantics.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BambooLeaves {
    None,
    Small,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BambooState {
    pub age: u8,
    pub leaves: BambooLeaves,
    pub stage: u8,
}

impl Default for BambooState {
    fn default() -> Self {
        Self {
            age: 0,
            leaves: BambooLeaves::None,
            stage: 0,
        }
    }
}

impl BambooState {
    pub fn state_id(self) -> Option<u32> {
        if self.age > 1 || self.stage > 1 {
            return None;
        }
        let leaves = match self.leaves {
            BambooLeaves::None => 0,
            BambooLeaves::Small => 1,
            BambooLeaves::Large => 2,
        };
        Some(15_279 + self.age as u32 * 6 + leaves * 2 + self.stage as u32)
    }
}

pub const BAMBOO_SAPLING_STATE_ID: u32 = 15_278;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BambooGrowth {
    pub new_top: BambooState,
    pub rewrite_below: Option<BambooLeaves>,
    pub rewrite_two_below: Option<BambooLeaves>,
}

pub fn bamboo_random_admitted(next_int_3: u8, above_air: bool, brightness: u8) -> bool {
    next_int_3 == 0 && above_air && brightness >= 9
}

pub fn bamboo_grow(
    height: u8,
    below: Option<BambooState>,
    two_below: Option<BambooState>,
    terminal_draw_below_quarter: bool,
) -> Option<BambooGrowth> {
    if height >= 16 {
        return None;
    }
    let below_state = match below {
        Some(state) => state,
        None => BambooState {
            age: 0,
            leaves: BambooLeaves::None,
            stage: 0,
        },
    };
    let (leaves, rewrite_below, rewrite_two_below) = if height < 1 {
        (BambooLeaves::None, None, None)
    } else if below_state.leaves == BambooLeaves::None {
        (BambooLeaves::Small, None, None)
    } else {
        (
            BambooLeaves::Large,
            Some(BambooLeaves::Small),
            two_below.map(|_| BambooLeaves::None),
        )
    };
    let age = u8::from(below_state.age == 1 || two_below.is_some_and(|state| state.age == 1));
    let stage = if height < 11 {
        0
    } else if height == 15 || terminal_draw_below_quarter {
        1
    } else {
        0
    };
    Some(BambooGrowth {
        new_top: BambooState { age, leaves, stage },
        rewrite_below,
        rewrite_two_below,
    })
}

pub fn bamboo_bone_meal_attempts(next_int_2: u8) -> u8 {
    1 + next_int_2.min(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CactusTick {
    Blocked,
    HeightCap,
    Age(u8),
    Grow {
        upper_age: u8,
        reset_age: u8,
        notify_upper: bool,
    },
}

pub fn cactus_random_tick(age: u8, height: u8, above_air: bool) -> CactusTick {
    if !above_air {
        return CactusTick::Blocked;
    }
    if height >= 3 && age == 15 {
        return CactusTick::HeightCap;
    }
    if age < 15 {
        CactusTick::Age(age + 1)
    } else {
        CactusTick::Grow {
            upper_age: 0,
            reset_age: 0,
            notify_upper: true,
        }
    }
}

pub fn cactus_state_id(age: u8) -> Option<u32> {
    (age <= 15).then_some(6_929 + age as u32)
}

pub fn cactus_flower_attempt(new_height: u8, age: u8, probability_draw: f64) -> bool {
    if age != 8 {
        return false;
    }
    if new_height >= 3 {
        probability_draw <= 0.25
    } else {
        probability_draw <= 0.1
    }
}

pub fn cactus_survives(
    horizontal_solid_or_lava: [bool; 4],
    supported_by_cactus_or_sand: bool,
    above_has_liquid: bool,
) -> bool {
    !horizontal_solid_or_lava.into_iter().any(|blocked| blocked)
        && supported_by_cactus_or_sand
        && !above_has_liquid
}

pub fn sugar_cane_survives(
    below_is_cane: bool,
    substrate_supported: bool,
    neighboring_water_or_frosted_ice: bool,
) -> bool {
    below_is_cane || (substrate_supported && neighboring_water_or_frosted_ice)
}

pub fn sugar_cane_random_tick(age: u8, height: u8, above_air: bool) -> CactusTick {
    if !above_air {
        CactusTick::Blocked
    } else if height >= 3 && age == 15 {
        CactusTick::HeightCap
    } else if age < 15 {
        CactusTick::Age(age + 1)
    } else {
        CactusTick::Grow {
            upper_age: 0,
            reset_age: 0,
            notify_upper: false,
        }
    }
}

pub fn sugar_cane_state_id(age: u8) -> Option<u32> {
    (age <= 15).then_some(6_947 + age as u32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveVinePart {
    Head,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaveVineState {
    pub part: CaveVinePart,
    pub age: u8,
    pub berries: bool,
}

impl CaveVineState {
    pub fn state_id(self) -> Option<u32> {
        match self.part {
            CaveVinePart::Head if self.age <= 25 => {
                Some(30_249 + self.age as u32 * 2 + u32::from(!self.berries))
            }
            CaveVinePart::Body => Some(30_301 + u32::from(!self.berries)),
            CaveVinePart::Head => None,
        }
    }
}

pub fn cave_vine_placement_age(next_int_25: u8) -> u8 {
    next_int_25.min(24)
}

pub fn cave_vine_random_growth(
    state: CaveVineState,
    next_double: f64,
    below_air: bool,
    berry_draw: f32,
) -> Option<CaveVineState> {
    if state.part != CaveVinePart::Head || state.age >= 25 || next_double >= 0.1 || !below_air {
        return None;
    }
    Some(CaveVineState {
        part: CaveVinePart::Head,
        age: state.age + 1,
        berries: berry_draw < 0.11,
    })
}

pub fn cave_vine_harvest(state: CaveVineState) -> Option<CaveVineState> {
    if !state.berries {
        None
    } else {
        Some(CaveVineState {
            berries: false,
            ..state
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetherVineKind {
    Weeping,
    Twisting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherVineHead {
    pub kind: NetherVineKind,
    pub age: u8,
}

impl NetherVineHead {
    pub fn state_id(self) -> Option<u32> {
        if self.age > 25 {
            return None;
        }
        Some(match self.kind {
            NetherVineKind::Weeping => 20_977 + self.age as u32,
            NetherVineKind::Twisting => 21_004 + self.age as u32,
        })
    }

    pub fn body_state_id(self) -> u32 {
        match self.kind {
            NetherVineKind::Weeping => 21_003,
            NetherVineKind::Twisting => 21_030,
        }
    }

    pub fn random_growth(self, next_double: f64, target_air: bool) -> Option<Self> {
        if self.age < 25 && next_double < 0.1 && target_air {
            Some(Self {
                age: self.age + 1,
                ..self
            })
        } else {
            None
        }
    }
}

pub fn nether_vine_bone_meal_count(draws: &[f64]) -> usize {
    let mut probability = 1.0;
    let mut count = 0;
    for draw in draws {
        if *draw >= probability {
            break;
        }
        count += 1;
        probability *= 0.826;
    }
    count
}

pub fn nether_vine_extension_ages(age: u8, count: usize) -> Vec<u8> {
    let mut next = age.saturating_add(1).min(25);
    (0..count)
        .map(|_| {
            let result = next;
            next = next.saturating_add(1).min(25);
            result
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaplingKind {
    Oak,
    Spruce,
    Birch,
    Jungle,
    Acacia,
    Cherry,
    DarkOak,
    PaleOak,
}

impl SaplingKind {
    pub fn state_id(self, stage: u8) -> Option<u32> {
        if stage > 1 {
            return None;
        }
        Some(29 + self.index() * 2 + stage as u32)
    }

    pub fn minimum_height(self) -> u8 {
        match self {
            Self::Oak | Self::Jungle => 4,
            Self::Spruce | Self::Birch => 5,
            Self::Acacia => 5,
            Self::Cherry => 7,
            Self::DarkOak | Self::PaleOak => 0,
        }
    }

    const fn index(self) -> u32 {
        match self {
            Self::Oak => 0,
            Self::Spruce => 1,
            Self::Birch => 2,
            Self::Jungle => 3,
            Self::Acacia => 4,
            Self::Cherry => 5,
            Self::DarkOak => 6,
            Self::PaleOak => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaplingTick {
    TooDark,
    DrawRejected,
    StageOne,
    GrowTree,
}

pub fn sapling_random_tick(stage: u8, brightness: u8, next_int_7: u8) -> SaplingTick {
    if brightness < 9 {
        SaplingTick::TooDark
    } else if next_int_7 != 0 {
        SaplingTick::DrawRejected
    } else if stage == 0 {
        SaplingTick::StageOne
    } else {
        SaplingTick::GrowTree
    }
}

pub fn sapling_bone_meal_succeeds(next_float: f32) -> bool {
    next_float < 0.45
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmallTreeChoice {
    Primary,
    Secondary,
    FancyOak,
}

pub fn small_tree_choice(kind: SaplingKind, selection_float: f32) -> SmallTreeChoice {
    match kind {
        SaplingKind::Oak if selection_float < 0.1 => SmallTreeChoice::FancyOak,
        SaplingKind::Spruce if selection_float < 0.5 => SmallTreeChoice::Secondary,
        _ => SmallTreeChoice::Primary,
    }
}
