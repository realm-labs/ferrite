//! Small mushrooms, huge-mushroom faces/features, and Nether fungi.

use ferrite_foundation::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MushroomKind {
    Brown,
    Red,
}

impl MushroomKind {
    pub fn state_id(self) -> u32 {
        match self {
            Self::Brown => 2_336,
            Self::Red => 2_337,
        }
    }

    pub fn preliminary_growth_height(self) -> u8 {
        match self {
            Self::Brown => 7,
            Self::Red => 6,
        }
    }
}

pub fn mushroom_survives(
    override_substrate: bool,
    raw_brightness: u8,
    below_solid_render: bool,
) -> bool {
    override_substrate || (raw_brightness < 13 && below_solid_render)
}

pub fn mushroom_spread_admitted(next_int_25: u8, same_identity_in_box: u8) -> bool {
    next_int_25 == 0 && same_identity_in_box <= 4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MushroomWalkStep {
    pub dx: i8,
    pub dy: i8,
    pub dz: i8,
}

pub fn mushroom_walk_step(
    x_draw: u8,
    y_positive_draw: u8,
    y_negative_draw: u8,
    z_draw: u8,
) -> MushroomWalkStep {
    MushroomWalkStep {
        dx: x_draw.min(2) as i8 - 1,
        dy: y_positive_draw.min(1) as i8 - y_negative_draw.min(1) as i8,
        dz: z_draw.min(2) as i8 - 1,
    }
}

pub fn mushroom_bone_meal_succeeds(next_float: f32) -> bool {
    next_float < 0.4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetherFungus {
    Crimson,
    Warped,
}

impl NetherFungus {
    pub fn state_id(self) -> u32 {
        match self {
            Self::Crimson => 20_975,
            Self::Warped => 20_958,
        }
    }

    pub fn block_id(self) -> u16 {
        match self {
            Self::Crimson => 876,
            Self::Warped => 867,
        }
    }

    pub fn item_id(self) -> u16 {
        match self {
            Self::Crimson => 277,
            Self::Warped => 278,
        }
    }

    pub fn potted_state_id(self) -> u32 {
        match self {
            Self::Crimson => 21_826,
            Self::Warped => 21_827,
        }
    }

    pub fn valid_bone_meal_target(
        self,
        below_exact_required_nylium: bool,
        above_in_build_height: bool,
    ) -> bool {
        below_exact_required_nylium && above_in_build_height
    }
}

pub fn fungus_bone_meal_succeeds(next_float: f32) -> bool {
    next_float < 0.4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugeMushroomKind {
    Brown,
    Red,
    Stem,
}

impl HugeMushroomKind {
    pub fn state_base(self) -> u32 {
        match self {
            Self::Brown => 7_766,
            Self::Red => 7_830,
            Self::Stem => 7_894,
        }
    }

    pub fn block_id(self) -> u16 {
        match self {
            Self::Brown => 338,
            Self::Red => 339,
            Self::Stem => 340,
        }
    }

    pub fn item_id(self) -> u16 {
        match self {
            Self::Brown => 415,
            Self::Red => 416,
            Self::Stem => 417,
        }
    }

    pub fn compost_chance(self) -> f32 {
        match self {
            Self::Stem => 0.65,
            Self::Brown | Self::Red => 0.85,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HugeMushroomFaces {
    pub down: bool,
    pub east: bool,
    pub north: bool,
    pub south: bool,
    pub up: bool,
    pub west: bool,
}

impl Default for HugeMushroomFaces {
    fn default() -> Self {
        Self {
            down: true,
            east: true,
            north: true,
            south: true,
            up: true,
            west: true,
        }
    }
}

impl HugeMushroomFaces {
    pub fn state_id(self, kind: HugeMushroomKind) -> u32 {
        kind.state_base()
            + 32 * u32::from(!self.down)
            + 16 * u32::from(!self.east)
            + 8 * u32::from(!self.north)
            + 4 * u32::from(!self.south)
            + 2 * u32::from(!self.up)
            + u32::from(!self.west)
    }

    pub fn hide(mut self, direction: Direction) -> Self {
        match direction {
            Direction::Down => self.down = false,
            Direction::Up => self.up = false,
            Direction::North => self.north = false,
            Direction::South => self.south = false,
            Direction::West => self.west = false,
            Direction::East => self.east = false,
        }
        self
    }
}

pub fn huge_mushroom_placement(
    kind: HugeMushroomKind,
    matching_neighbors: &[(Direction, bool)],
) -> HugeMushroomFaces {
    let mut state = HugeMushroomFaces::default();
    for direction in Direction::ALL {
        if matching_neighbors
            .iter()
            .any(|(neighbor_direction, same_kind)| *neighbor_direction == direction && *same_kind)
        {
            state = state.hide(direction);
        }
    }
    let _ = kind;
    state
}

pub fn huge_mushroom_height(next_int_3: u8, next_int_12: u8) -> u8 {
    let base = 4 + next_int_3.min(2);
    if next_int_12 == 0 { base * 2 } else { base }
}

pub fn huge_mushroom_write_count(kind: MushroomKind, height: u8) -> u16 {
    match kind {
        MushroomKind::Brown | MushroomKind::Red => 45 + height as u16,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugeMushroomDrop {
    Nothing,
    Block,
    SmallMushrooms(u8),
}

pub fn huge_mushroom_drop(
    kind: HugeMushroomKind,
    silk_touch: bool,
    count_draw_minus_6_to_2: i8,
) -> HugeMushroomDrop {
    if silk_touch {
        return HugeMushroomDrop::Block;
    }
    match kind {
        HugeMushroomKind::Stem => HugeMushroomDrop::Nothing,
        HugeMushroomKind::Brown | HugeMushroomKind::Red => {
            let count = if count_draw_minus_6_to_2 > 0 {
                count_draw_minus_6_to_2.min(2) as u8
            } else {
                0
            };
            HugeMushroomDrop::SmallMushrooms(count)
        }
    }
}
