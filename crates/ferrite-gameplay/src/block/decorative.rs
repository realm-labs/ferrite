//! Cake, flower-pot, Pumpkin, and Carved-Pumpkin interaction decisions.

use ferrite_foundation::direction::Direction;

pub const EMPTY_FLOWER_POT_STATE_ID: u32 = 10_629;
pub const FLOWER_POT_ITEM_ID: u16 = 1_256;
pub const EARLY_POTTED_STATE_RANGE: core::ops::RangeInclusive<u32> = 10_630..=10_658;
pub const LATER_POTTED_STATE_IDS: [u32; 7] =
    [15_291, 21_826, 21_827, 32_073, 32_074, 32_363, 32_364];

pub fn is_flower_pot_state(state_id: u32) -> bool {
    state_id == EMPTY_FLOWER_POT_STATE_ID
        || EARLY_POTTED_STATE_RANGE.contains(&state_id)
        || LATER_POTTED_STATE_IDS.contains(&state_id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesiredEyeblossom {
    Default,
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeblossomTransform {
    pub target_state_id: u32,
    pub flags: u16,
    pub particle_id: u8,
    pub color: u32,
    pub sound_id: u16,
    pub target_offset: [f64; 3],
    pub lifetime: u32,
}

pub fn potted_eyeblossom_tick(
    currently_open: bool,
    desired: DesiredEyeblossom,
    draws: [f64; 4],
) -> Option<EyeblossomTransform> {
    let desired_open = match desired {
        DesiredEyeblossom::Default => return None,
        DesiredEyeblossom::Open => true,
        DesiredEyeblossom::Closed => false,
    };
    if desired_open == currently_open {
        return None;
    }
    let scale = 0.5 + draws[0];
    Some(EyeblossomTransform {
        target_state_id: if desired_open { 32_363 } else { 32_364 },
        flags: 3,
        particle_id: 56,
        color: if desired_open { 0xFC_78_12 } else { 0x5F_5F_5F },
        sound_id: if desired_open { 619 } else { 621 },
        target_offset: [
            (draws[1] - 0.5) * scale,
            (draws[2] + 1.0) * scale,
            (draws[3] - 0.5) * scale,
        ],
        lifetime: (20.0 * scale) as u32,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CakeState {
    pub bites: u8,
}

impl CakeState {
    pub fn new(bites: u8) -> Option<Self> {
        if bites <= 6 {
            Some(Self { bites })
        } else {
            None
        }
    }

    pub fn state_id(self) -> u32 {
        7_027 + self.bites as u32
    }

    pub fn min_x_sixteenths(self) -> u8 {
        1 + 2 * self.bites
    }

    pub fn analog_output(self) -> u8 {
        (7 - self.bites) * 2
    }

    pub fn eat(self) -> CakeEat {
        if self.bites < 6 {
            CakeEat::Next(Self {
                bites: self.bites + 1,
            })
        } else {
            CakeEat::Remove
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CakeEat {
    Next(CakeState),
    Remove,
}

pub fn candle_cake_admitted(cake: CakeState, item_in_candles_tag: bool) -> bool {
    cake.bites == 0 && item_in_candles_tag
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowerPotUse {
    Insert {
        flags: u16,
        award_stat: bool,
        consume_one: bool,
    },
    AlreadyFilled,
    Extract {
        flags: u16,
        add_or_drop_item: bool,
    },
    Empty,
}

pub fn flower_pot_use(
    currently_filled: bool,
    held_item_has_potted_mapping: bool,
    held_item_empty: bool,
) -> FlowerPotUse {
    if held_item_has_potted_mapping {
        if currently_filled {
            FlowerPotUse::AlreadyFilled
        } else {
            FlowerPotUse::Insert {
                flags: 3,
                award_stat: true,
                consume_one: true,
            }
        }
    } else if held_item_empty && currently_filled {
        FlowerPotUse::Extract {
            flags: 3,
            add_or_drop_item: true,
        }
    } else {
        FlowerPotUse::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarvedPumpkinFacing {
    North,
    South,
    West,
    East,
}

impl CarvedPumpkinFacing {
    pub fn state_id(self) -> u32 {
        match self {
            Self::North => 7_019,
            Self::South => 7_020,
            Self::West => 7_021,
            Self::East => 7_022,
        }
    }
}

pub fn carved_pumpkin_placement(player_direction: Direction) -> Option<CarvedPumpkinFacing> {
    match player_direction.opposite() {
        Direction::North => Some(CarvedPumpkinFacing::North),
        Direction::South => Some(CarvedPumpkinFacing::South),
        Direction::West => Some(CarvedPumpkinFacing::West),
        Direction::East => Some(CarvedPumpkinFacing::East),
        Direction::Down | Direction::Up => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GolemPattern {
    Snow,
    Iron,
    Copper,
}

pub fn first_creatable_golem(matches: &[(GolemPattern, bool, bool)]) -> Option<GolemPattern> {
    [GolemPattern::Snow, GolemPattern::Iron, GolemPattern::Copper]
        .into_iter()
        .find(|target| {
            matches
                .iter()
                .any(|(kind, pattern_matches, factory_created)| {
                    kind == target && *pattern_matches && *factory_created
                })
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PumpkinCarve {
    pub seeds: u8,
    pub write_flags: u16,
    pub durability_cost: u8,
}

pub fn carve_pumpkin(with_shears: bool) -> Option<PumpkinCarve> {
    if with_shears {
        Some(PumpkinCarve {
            seeds: 4,
            write_flags: 11,
            durability_cost: 1,
        })
    } else {
        None
    }
}

pub fn melon_slice_count(base_3_to_7: u8, fortune_bonus: u8) -> u8 {
    base_3_to_7.clamp(3, 7).saturating_add(fortune_bonus).min(9)
}
