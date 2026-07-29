//! Coral block and plant identity, support, water, drying, and loot kernels.

use ferrite_foundation::direction::Direction;

pub const DRYING_BASE_DELAY: u64 = 60;
pub const DRYING_RANDOM_BOUND: u32 = 40;
pub const DRYING_WRITE_FLAGS: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralColor {
    Tube,
    Brain,
    Bubble,
    Fire,
    Horn,
}

impl CoralColor {
    const fn index(self) -> u32 {
        match self {
            Self::Tube => 0,
            Self::Brain => 1,
            Self::Bubble => 2,
            Self::Fire => 3,
            Self::Horn => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoralBlockIdentity {
    pub color: CoralColor,
    pub live: bool,
}

impl CoralBlockIdentity {
    pub const fn state_id(self) -> u32 {
        15_137 + self.color.index() + if self.live { 5 } else { 0 }
    }

    pub const fn block_id(self) -> u32 {
        748 + self.color.index() + if self.live { 5 } else { 0 }
    }

    pub const fn item_id(self) -> u32 {
        677 + self.color.index() + if self.live { 5 } else { 0 }
    }

    pub const fn dead(self) -> Self {
        Self {
            color: self.color,
            live: false,
        }
    }
}

pub fn adjacent_water(fluid_is_water: [bool; 6]) -> Option<Direction> {
    let directions = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];
    directions
        .into_iter()
        .zip(fluid_is_water)
        .find_map(|(direction, wet)| wet.then_some(direction))
}

pub const fn drying_delay(next_int_40: u32) -> u64 {
    DRYING_BASE_DELAY
        + if next_int_40 < DRYING_RANDOM_BOUND {
            next_int_40
        } else {
            DRYING_RANDOM_BOUND - 1
        } as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralDue {
    WetNoOp,
    Convert {
        target: CoralBlockIdentity,
        flags: u16,
    },
    DeadNoOp,
}

pub const fn coral_block_due(identity: CoralBlockIdentity, wet: bool) -> CoralDue {
    if !identity.live {
        CoralDue::DeadNoOp
    } else if wet {
        CoralDue::WetNoOp
    } else {
        CoralDue::Convert {
            target: identity.dead(),
            flags: DRYING_WRITE_FLAGS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralBlockDrop {
    None,
    Live(CoralColor),
    Dead(CoralColor),
}

pub const fn coral_block_drop(
    identity: CoralBlockIdentity,
    correct_pickaxe: bool,
    silk_touch: bool,
    survives_explosion: bool,
) -> CoralBlockDrop {
    if !correct_pickaxe {
        return CoralBlockDrop::None;
    }
    if identity.live && silk_touch {
        CoralBlockDrop::Live(identity.color)
    } else if survives_explosion {
        CoralBlockDrop::Dead(identity.color)
    } else {
        CoralBlockDrop::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralForm {
    Upright,
    FloorFan,
    WallFan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoralPlantState {
    pub form: CoralForm,
    pub color: CoralColor,
    pub live: bool,
    pub waterlogged: bool,
    pub facing: Direction,
}

impl CoralPlantState {
    pub const fn state_id(self) -> u32 {
        let color = self.color.index();
        let wet = if self.waterlogged { 0 } else { 1 };
        match self.form {
            CoralForm::Upright => 15_147 + color * 2 + wet + if self.live { 10 } else { 0 },
            CoralForm::FloorFan => 15_167 + color * 2 + wet + if self.live { 10 } else { 0 },
            CoralForm::WallFan => {
                let facing = horizontal_facing_index(self.facing);
                15_187 + color * 8 + facing * 2 + wet + if self.live { 40 } else { 0 }
            }
        }
    }
}

const fn horizontal_facing_index(direction: Direction) -> u32 {
    match direction {
        Direction::North => 0,
        Direction::South => 1,
        Direction::West => 2,
        Direction::East => 3,
        Direction::Down | Direction::Up => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralPlantUpdate {
    Remove,
    Keep {
        dry_tick: bool,
        water_tick_requests: u8,
    },
}

pub const fn coral_plant_update(
    state: CoralPlantState,
    support_valid: bool,
    wet: bool,
) -> CoralPlantUpdate {
    if !support_valid {
        return CoralPlantUpdate::Remove;
    }
    CoralPlantUpdate::Keep {
        dry_tick: state.live && !wet,
        water_tick_requests: if state.waterlogged {
            if state.live { 2 } else { 1 }
        } else {
            0
        },
    }
}

pub const fn coral_plant_due(mut state: CoralPlantState, wet: bool) -> Option<CoralPlantState> {
    if !state.live || wet {
        return None;
    }
    state.live = false;
    state.waterlogged = false;
    Some(state)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralPlantDrop {
    None,
    Upright(CoralColor),
    FloorFan(CoralColor),
}

pub const fn coral_plant_drop(
    state: CoralPlantState,
    correct_pickaxe: bool,
    silk_touch: bool,
) -> CoralPlantDrop {
    if !silk_touch || (!state.live && !correct_pickaxe) {
        return CoralPlantDrop::None;
    }
    match state.form {
        CoralForm::Upright => CoralPlantDrop::Upright(state.color),
        CoralForm::FloorFan | CoralForm::WallFan => CoralPlantDrop::FloorFan(state.color),
    }
}
