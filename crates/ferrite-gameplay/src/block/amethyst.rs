//! Budding-Amethyst stage selection, placement, and deterministic random-tick transitions.

use ferrite_foundation::direction::Direction;

pub const BUDDING_AMETHYST_STATE_ID: u32 = 23_403;
pub const BUDDING_AMETHYST_BLOCK_ID: u16 = 979;
pub const BUDDING_AMETHYST_ITEM_ID: u16 = 116;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmethystStage {
    Small,
    Medium,
    Large,
    Cluster,
}

impl AmethystStage {
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Small => Some(Self::Medium),
            Self::Medium => Some(Self::Large),
            Self::Large => Some(Self::Cluster),
            Self::Cluster => None,
        }
    }

    pub fn state_range(self) -> core::ops::RangeInclusive<u32> {
        match self {
            Self::Small => 23_440..=23_451,
            Self::Medium => 23_428..=23_439,
            Self::Large => 23_416..=23_427,
            Self::Cluster => 23_404..=23_415,
        }
    }

    pub fn default_state_id(self) -> u32 {
        match self {
            Self::Small => 23_449,
            Self::Medium => 23_437,
            Self::Large => 23_425,
            Self::Cluster => 23_413,
        }
    }

    pub fn light(self) -> u8 {
        match self {
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 4,
            Self::Cluster => 5,
        }
    }

    pub fn height(self) -> u8 {
        match self {
            Self::Small => 3,
            Self::Medium => 4,
            Self::Large => 5,
            Self::Cluster => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmethystTarget {
    Air,
    FullSourceWater,
    Bud {
        stage: AmethystStage,
        facing: Direction,
        waterlogged: bool,
    },
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmethystGrowth {
    ProbabilityRejected,
    PlaceSmall {
        facing: Direction,
        waterlogged: bool,
    },
    Advance {
        stage: AmethystStage,
        facing: Direction,
        waterlogged: bool,
    },
    Blocked,
}

pub fn budding_random_tick(
    next_int_5: u8,
    next_direction: Direction,
    target: AmethystTarget,
) -> AmethystGrowth {
    if next_int_5 != 0 {
        return AmethystGrowth::ProbabilityRejected;
    }
    match target {
        AmethystTarget::Air => AmethystGrowth::PlaceSmall {
            facing: next_direction,
            waterlogged: false,
        },
        AmethystTarget::FullSourceWater => AmethystGrowth::PlaceSmall {
            facing: next_direction,
            waterlogged: true,
        },
        AmethystTarget::Bud {
            stage,
            facing,
            waterlogged,
        } if facing == next_direction => match stage.next() {
            Some(next) => AmethystGrowth::Advance {
                stage: next,
                facing,
                waterlogged,
            },
            None => AmethystGrowth::Blocked,
        },
        AmethystTarget::Bud { .. } | AmethystTarget::Other => AmethystGrowth::Blocked,
    }
}

pub fn amethyst_survives(support_face_sturdy: bool) -> bool {
    support_face_sturdy
}

pub fn cluster_shard_count(correct_pickaxe: bool, base: u8, fortune_bonus: u8) -> u8 {
    if correct_pickaxe {
        base.saturating_add(fortune_bonus)
    } else {
        2
    }
}
