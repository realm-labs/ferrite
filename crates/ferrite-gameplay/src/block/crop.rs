//! Deterministic crop, stem, Cocoa, and berry transitions.
//!
//! Random values are explicit inputs so Region replay observes the same draw and write order.

use ferrite_foundation::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropKind {
    Wheat,
    Carrots,
    Potatoes,
    Beetroot,
}

impl CropKind {
    pub fn max_age(self) -> u8 {
        match self {
            Self::Beetroot => 3,
            Self::Wheat | Self::Carrots | Self::Potatoes => 7,
        }
    }

    pub fn state_id(self, age: u8) -> Option<u32> {
        if age > self.max_age() {
            return None;
        }
        Some(match self {
            Self::Wheat => 5_311 + age as u32,
            Self::Carrots => 10_659 + age as u32,
            Self::Potatoes => 10_667 + age as u32,
            Self::Beetroot => 14_811 + age as u32,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FarmlandCell {
    pub grows_crop: bool,
    pub moisture: u8,
    pub off_center: bool,
}

pub fn growth_speed(
    cells: &[FarmlandCell],
    same_on_x: bool,
    same_on_z: bool,
    diagonal: bool,
) -> f32 {
    let mut speed = 1.0;
    for cell in cells.iter().take(9).filter(|cell| cell.grows_crop) {
        let contribution = if cell.moisture == 0 { 1.0 } else { 3.0 };
        speed += if cell.off_center {
            contribution / 4.0
        } else {
            contribution
        };
    }
    if diagonal || (same_on_x && same_on_z) {
        speed / 2.0
    } else {
        speed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropTick {
    BeetrootOuterRejected,
    TooDark,
    Mature,
    GrowthDrawRejected { bound: u32 },
    Advance { age: u8, flags: u16 },
}

pub fn random_tick(
    kind: CropKind,
    age: u8,
    beetroot_outer_draw: Option<u8>,
    brightness_above: u8,
    speed: f32,
    growth_draw: u32,
) -> CropTick {
    if kind == CropKind::Beetroot && beetroot_outer_draw.is_some_and(|draw| draw == 0) {
        return CropTick::BeetrootOuterRejected;
    }
    if age >= kind.max_age() {
        return CropTick::Mature;
    }
    if brightness_above < 9 {
        return CropTick::TooDark;
    }
    let bound = ((25.0 / speed) as u32).saturating_add(1);
    if growth_draw != 0 {
        CropTick::GrowthDrawRejected { bound }
    } else {
        CropTick::Advance {
            age: age + 1,
            flags: 2,
        }
    }
}

pub fn bone_meal_growth(kind: CropKind, draw_0_to_3: u8) -> u8 {
    let ordinary = 2 + draw_0_to_3.min(3);
    if matches!(kind, CropKind::Beetroot) {
        ordinary / 3
    } else {
        ordinary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CocoaState {
    pub age: u8,
}

impl CocoaState {
    pub fn new(age: u8) -> Option<Self> {
        if age <= 2 { Some(Self { age }) } else { None }
    }

    pub fn random_tick(self, next_int_5: u8) -> Option<Self> {
        if self.age < 2 && next_int_5 == 0 {
            Some(Self { age: self.age + 1 })
        } else {
            None
        }
    }

    pub fn bone_meal(self) -> Self {
        Self {
            age: if self.age < 2 { self.age + 1 } else { 2 },
        }
    }

    pub fn loot_count(self) -> u8 {
        if self.age == 2 { 3 } else { 1 }
    }

    pub fn state_id(self, facing: Direction) -> Option<u32> {
        let facing_offset = match facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            Direction::Down | Direction::Up => return None,
        };
        Some(9_481 + self.age as u32 * 4 + facing_offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitcherHalf {
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitcherState {
    pub age: u8,
    pub half: PitcherHalf,
}

impl PitcherState {
    pub fn state_id(self) -> Option<u32> {
        if self.age > 4 {
            return None;
        }
        if self.age == 4 {
            return Some(match self.half {
                PitcherHalf::Lower => 14_810,
                PitcherHalf::Upper => 14_809,
            });
        }
        Some(14_799 + self.age as u32 * 2 + half_index(self.half))
    }

    pub fn is_double(self) -> bool {
        self.age >= 3
    }
}

const fn half_index(half: PitcherHalf) -> u32 {
    match half {
        PitcherHalf::Lower => 1,
        PitcherHalf::Upper => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitcherGrowth {
    Reject,
    Single {
        lower: PitcherState,
        flags: u16,
    },
    Double {
        lower: PitcherState,
        lower_flags: u16,
        upper: PitcherState,
        upper_flags: u16,
    },
}

pub fn pitcher_grow(
    current: PitcherState,
    brightness: u8,
    above_in_height: bool,
    above_replaceable: bool,
) -> PitcherGrowth {
    if current.age >= 4 || brightness < 8 || !above_in_height {
        return PitcherGrowth::Reject;
    }
    let next = current.age + 1;
    if next >= 3 && !above_replaceable {
        return PitcherGrowth::Reject;
    }
    let lower = PitcherState {
        age: next,
        half: PitcherHalf::Lower,
    };
    if next < 3 {
        PitcherGrowth::Single { lower, flags: 2 }
    } else {
        PitcherGrowth::Double {
            lower,
            lower_flags: 2,
            upper: PitcherState {
                age: next,
                half: PitcherHalf::Upper,
            },
            upper_flags: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitcherRandomTick {
    NotLowerGrowingState,
    GrowthDrawRejected { bound: u32 },
    Growth(PitcherGrowth),
}

pub fn pitcher_random_tick(
    current: PitcherState,
    speed: f32,
    growth_draw: u32,
    brightness: u8,
    above_in_height: bool,
    above_replaceable: bool,
) -> PitcherRandomTick {
    if current.half != PitcherHalf::Lower || current.age >= 4 {
        return PitcherRandomTick::NotLowerGrowingState;
    }
    let bound = ((25.0 / speed) as u32).saturating_add(1);
    if growth_draw != 0 {
        return PitcherRandomTick::GrowthDrawRejected { bound };
    }
    PitcherRandomTick::Growth(pitcher_grow(
        current,
        brightness,
        above_in_height,
        above_replaceable,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorchflowerState {
    Crop(u8),
    Flower,
}

impl TorchflowerState {
    pub fn state_id(self) -> Option<u32> {
        match self {
            Self::Crop(age) if age <= 1 => Some(14_797 + age as u32),
            Self::Flower => Some(2_323),
            Self::Crop(_) => None,
        }
    }
}

pub fn torchflower_advance(age: u8) -> TorchflowerState {
    if age == 0 {
        TorchflowerState::Crop(1)
    } else {
        TorchflowerState::Flower
    }
}

pub fn torchflower_random_tick(
    age: u8,
    outer_next_int_3: u8,
    brightness: u8,
    speed: f32,
    growth_draw: u32,
) -> CropTick {
    if outer_next_int_3 == 0 {
        return CropTick::BeetrootOuterRejected;
    }
    if brightness < 9 {
        return CropTick::TooDark;
    }
    let bound = ((25.0 / speed) as u32).saturating_add(1);
    if growth_draw != 0 {
        CropTick::GrowthDrawRejected { bound }
    } else {
        CropTick::Advance {
            age: age.saturating_add(1).min(2),
            flags: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemKind {
    Pumpkin,
    Melon,
}

impl StemKind {
    pub fn state_id(self, age: u8) -> Option<u32> {
        if age > 7 {
            return None;
        }
        Some(
            match self {
                Self::Pumpkin => 8_342,
                Self::Melon => 8_350,
            } + age as u32,
        )
    }

    pub fn attached_state_id(self, facing: Direction) -> Option<u32> {
        let offset = match facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            Direction::Down | Direction::Up => return None,
        };
        Some(
            match self {
                Self::Pumpkin => 8_334,
                Self::Melon => 8_338,
            } + offset,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemTick {
    TooDark,
    GrowthDrawRejected {
        bound: u32,
    },
    Age(u8),
    FruitRejected,
    Fruit {
        fruit_flags: u16,
        attached_flags: u16,
    },
}

pub fn stem_random_tick(
    age: u8,
    brightness_above: u8,
    speed: f32,
    growth_draw: u32,
    fruit_target_air: bool,
    fruit_supports: bool,
) -> StemTick {
    if brightness_above < 9 {
        return StemTick::TooDark;
    }
    let bound = ((25.0 / speed) as u32).saturating_add(1);
    if growth_draw != 0 {
        return StemTick::GrowthDrawRejected { bound };
    }
    if age < 7 {
        return StemTick::Age(age + 1);
    }
    if fruit_target_air && fruit_supports {
        StemTick::Fruit {
            fruit_flags: 3,
            attached_flags: 3,
        }
    } else {
        StemTick::FruitRejected
    }
}

pub fn stem_bone_meal_age(age: u8, draw_0_to_3: u8) -> (u8, bool) {
    let next = age.saturating_add(2 + draw_0_to_3.min(3)).min(7);
    (next, next == 7)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BerryHarvest {
    pub berries: u8,
    pub next_age: u8,
    pub flags: u16,
}

pub fn berry_random_growth(age: u8, next_int_5: u8, brightness_above: u8) -> Option<u8> {
    if age < 3 && next_int_5 == 0 && brightness_above >= 9 {
        Some(age + 1)
    } else {
        None
    }
}

pub fn berry_state_id(age: u8) -> Option<u32> {
    (age <= 3).then_some(20_941 + age as u32)
}

pub fn berry_harvest(age: u8, random_bonus_0_or_1: u8) -> Option<BerryHarvest> {
    if age < 2 {
        return None;
    }
    Some(BerryHarvest {
        berries: age - 1 + random_bonus_0_or_1.min(1),
        next_age: 1,
        flags: 2,
    })
}

pub fn berry_contact_damage(
    age: u8,
    exempt_fox_or_bee: bool,
    horizontal_dx: f64,
    horizontal_dz: f64,
) -> bool {
    age >= 1 && !exempt_fox_or_bee && (horizontal_dx.abs() >= 0.003 || horizontal_dz.abs() >= 0.003)
}
