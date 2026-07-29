//! Nether roots, sprouts, stems, wart crop, and wart-block semantic kernels.

pub const NETHER_PLANT_SUPPORT_COUNT: u8 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetherPlant {
    CrimsonRoots,
    WarpedRoots,
    NetherSprouts,
}

impl NetherPlant {
    pub const fn state_id(self) -> u32 {
        match self {
            Self::CrimsonRoots => 21_031,
            Self::WarpedRoots => 20_960,
            Self::NetherSprouts => 20_961,
        }
    }

    pub const fn block_id(self) -> u32 {
        match self {
            Self::CrimsonRoots => 882,
            Self::WarpedRoots => 869,
            Self::NetherSprouts => 870,
        }
    }

    pub const fn item_id(self) -> u32 {
        match self {
            Self::CrimsonRoots => 279,
            Self::WarpedRoots => 280,
            Self::NetherSprouts => 281,
        }
    }

    pub const fn composter_chance(self) -> f64 {
        match self {
            Self::NetherSprouts => 0.5,
            Self::CrimsonRoots | Self::WarpedRoots => 0.65,
        }
    }

    pub const fn survives_on(self, support_index: u8) -> bool {
        support_index < NETHER_PLANT_SUPPORT_COUNT
    }

    pub const fn drops_item(self, shears: bool, survives_explosion: bool) -> bool {
        match self {
            Self::NetherSprouts => shears,
            Self::CrimsonRoots | Self::WarpedRoots => survives_explosion,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FootstepSounds {
    pub plant_volume: f32,
    pub plant_pitch: f32,
    pub support_volume_scale: f32,
    pub support_pitch_scale: f32,
}

pub const COMBINED_FOOTSTEPS: FootstepSounds = FootstepSounds {
    plant_volume: 0.15,
    plant_pitch: 1.0,
    support_volume_scale: 0.05,
    support_pitch_scale: 0.8,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemKind {
    WarpedStem,
    StrippedWarpedStem,
    WarpedHyphae,
    StrippedWarpedHyphae,
    CrimsonStem,
    StrippedCrimsonStem,
    CrimsonHyphae,
    StrippedCrimsonHyphae,
}

impl StemKind {
    const fn index(self) -> u32 {
        match self {
            Self::WarpedStem => 0,
            Self::StrippedWarpedStem => 1,
            Self::WarpedHyphae => 2,
            Self::StrippedWarpedHyphae => 3,
            Self::CrimsonStem => 4,
            Self::StrippedCrimsonStem => 5,
            Self::CrimsonHyphae => 6,
            Self::StrippedCrimsonHyphae => 7,
        }
    }

    pub const fn state_id(self, axis: Axis) -> u32 {
        let base = if self.index() < 4 {
            20_945 + self.index() * 3
        } else {
            20_962 + (self.index() - 4) * 3
        };
        base + axis_index(axis)
    }

    pub const fn stripped(self) -> Option<Self> {
        match self {
            Self::WarpedStem => Some(Self::StrippedWarpedStem),
            Self::WarpedHyphae => Some(Self::StrippedWarpedHyphae),
            Self::CrimsonStem => Some(Self::StrippedCrimsonStem),
            Self::CrimsonHyphae => Some(Self::StrippedCrimsonHyphae),
            _ => None,
        }
    }

    pub const fn burn_time(self) -> u16 {
        0
    }
}

const fn axis_index(axis: Axis) -> u32 {
    match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripResult {
    pub target: StemKind,
    pub axis: Axis,
    pub flags: u16,
    pub durability_cost: u8,
}

pub const fn strip(kind: StemKind, axis: Axis, offhand_blocks: bool) -> Option<StripResult> {
    if offhand_blocks {
        return None;
    }
    match kind.stripped() {
        Some(target) => Some(StripResult {
            target,
            axis,
            flags: 11,
            durability_cost: 1,
        }),
        None => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherWartState {
    pub age: u8,
}

impl NetherWartState {
    pub const fn new(age: u8) -> Option<Self> {
        if age <= 3 { Some(Self { age }) } else { None }
    }

    pub const fn state_id(self) -> u32 {
        9_447 + self.age as u32
    }

    pub const fn selection_height(self) -> u8 {
        5 + self.age * 3
    }

    pub const fn randomly_ticking(self) -> bool {
        self.age < 3
    }

    pub const fn random_tick(self, next_int_10: u8) -> Option<Self> {
        if self.age < 3 && next_int_10 == 0 {
            Some(Self { age: self.age + 1 })
        } else {
            None
        }
    }

    pub const fn client_stage(self) -> u8 {
        match self.age {
            0 => 0,
            1 | 2 => 1,
            _ => 2,
        }
    }
}

pub const fn wart_loot_base(age: u8, base_draw_0_to_2: u8, fortune_draw: u8) -> u8 {
    if age < 3 {
        1
    } else {
        2 + if base_draw_0_to_2 < 3 {
            base_draw_0_to_2
        } else {
            2
        } + fortune_draw
    }
}

pub fn explosion_decay(count: u8, retained: &[bool]) -> u8 {
    retained
        .iter()
        .take(count as usize)
        .filter(|keep| **keep)
        .count() as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WartBlockKind {
    Nether,
    Warped,
}

impl WartBlockKind {
    pub const fn state_id(self) -> u32 {
        match self {
            Self::Nether => 14_846,
            Self::Warped => 20_959,
        }
    }

    pub const fn block_id(self) -> u32 {
        match self {
            Self::Nether => 672,
            Self::Warped => 868,
        }
    }

    pub const fn item_id(self) -> u32 {
        match self {
            Self::Nether => 604,
            Self::Warped => 605,
        }
    }

    pub const fn vetoes_piglin_family_spawn(self) -> bool {
        matches!(self, Self::Nether)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposterResult {
    Delegated,
    AcceptedNoInsert,
    Inserted { new_level: u8, schedule: bool },
    ConsumedWithoutInsert,
}

pub fn compost(level: u8, chance: f64, draw: Option<f64>, automated: bool) -> ComposterResult {
    if level >= 8 {
        return ComposterResult::Delegated;
    }
    if level == 7 {
        return ComposterResult::AcceptedNoInsert;
    }
    let success = level == 0 || draw.is_some_and(|value| value < chance);
    if success {
        let new_level = level + 1;
        ComposterResult::Inserted {
            new_level,
            schedule: new_level == 7,
        }
    } else {
        let _ = automated;
        ComposterResult::ConsumedWithoutInsert
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VegetationChoice {
    CrimsonRoots,
    WarpedRoots,
    NetherSprouts,
    Fungus,
    Other,
}

pub const fn crimson_vegetation_choice(draw: u8) -> VegetationChoice {
    match draw % 99 {
        0..=86 => VegetationChoice::CrimsonRoots,
        87..=97 => VegetationChoice::Fungus,
        _ => VegetationChoice::Other,
    }
}

pub const fn warped_vegetation_choice(draw: u8) -> VegetationChoice {
    match draw {
        0..=84 => VegetationChoice::WarpedRoots,
        85 => VegetationChoice::Fungus,
        86..=98 => VegetationChoice::NetherSprouts,
        _ => VegetationChoice::Other,
    }
}
