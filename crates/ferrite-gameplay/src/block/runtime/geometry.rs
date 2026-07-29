//! Shared axis, orientation, light, and block-shape rules.

use ferrite_foundation::direction::{Axis, Direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarterTurn {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

pub const fn rotate_axis(axis: Axis, rotation: QuarterTurn) -> Axis {
    match (axis, rotation) {
        (Axis::X, QuarterTurn::Clockwise90 | QuarterTurn::CounterClockwise90) => Axis::Z,
        (Axis::Z, QuarterTurn::Clockwise90 | QuarterTurn::CounterClockwise90) => Axis::X,
        _ => axis,
    }
}

pub const fn rotate_horizontal(direction: Direction, rotation: QuarterTurn) -> Direction {
    match rotation {
        QuarterTurn::None => direction,
        QuarterTurn::Clockwise90 => clockwise(direction),
        QuarterTurn::Clockwise180 => clockwise(clockwise(direction)),
        QuarterTurn::CounterClockwise90 => counter_clockwise(direction),
    }
}

const fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        direction => direction,
    }
}

const fn counter_clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::West,
        Direction::West => Direction::South,
        Direction::South => Direction::East,
        Direction::East => Direction::North,
        direction => direction,
    }
}

pub fn banner_rotation(player_yaw: f32) -> u8 {
    (((player_yaw + 180.0) * 16.0 / 360.0).round() as i32 & 15) as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

impl DyeColor {
    pub const ALL: [Self; 16] = [
        Self::White,
        Self::Orange,
        Self::Magenta,
        Self::LightBlue,
        Self::Yellow,
        Self::Lime,
        Self::Pink,
        Self::Gray,
        Self::LightGray,
        Self::Cyan,
        Self::Purple,
        Self::Blue,
        Self::Brown,
        Self::Green,
        Self::Red,
        Self::Black,
    ];

    pub const fn diffuse_rgb(self) -> u32 {
        match self {
            Self::White => 0xF9FFFE,
            Self::Orange => 0xF9801D,
            Self::Magenta => 0xC74EBD,
            Self::LightBlue => 0x3AB3DA,
            Self::Yellow => 0xFED83D,
            Self::Lime => 0x80C71F,
            Self::Pink => 0xF38BAA,
            Self::Gray => 0x474F52,
            Self::LightGray => 0x9D9D97,
            Self::Cyan => 0x169C9C,
            Self::Purple => 0x8932B8,
            Self::Blue => 0x3C44AA,
            Self::Brown => 0x835432,
            Self::Green => 0x5E7C16,
            Self::Red => 0xB02E26,
            Self::Black => 0x1D1D21,
        }
    }

    pub const fn beacon_rgb(self) -> [f32; 3] {
        let rgb = self.diffuse_rgb();
        [
            ((rgb >> 16) & 0xff) as f32 / 255.0,
            ((rgb >> 8) & 0xff) as f32 / 255.0,
            (rgb & 0xff) as f32 / 255.0,
        ]
    }
}

pub fn extend_beacon_color(previous: Option<[f32; 3]>, color: DyeColor) -> [f32; 3] {
    let color = color.beacon_rgb();
    match previous {
        None => color,
        Some(previous) => [
            (previous[0] + color[0]) / 2.0,
            (previous[1] + color[1]) / 2.0,
            (previous[2] + color[2]) / 2.0,
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockPhysics {
    pub hardness: f32,
    pub resistance: f32,
    pub speed_factor: f32,
    pub jump_factor: f32,
    pub friction: f32,
    pub light_emission: u8,
    pub light_dampening: u8,
    pub full_collision: bool,
    pub replaceable: bool,
}

pub fn exceptional_physics(path: &str) -> Option<BlockPhysics> {
    let profile = match path {
        "air" | "cave_air" | "void_air" => physics(0.0, 0.0, [1.0, 1.0, 0.6], [0, 0], false, true),
        "bedrock" => physics(-1.0, 3_600_000.0, [1.0, 1.0, 0.6], [0, 15], true, false),
        "reinforced_deepslate" => physics(55.0, 1_200.0, [1.0, 1.0, 0.6], [0, 15], true, false),
        "glass" => physics(0.3, 0.3, [1.0, 1.0, 0.6], [0, 0], true, false),
        "tinted_glass" => physics(0.3, 0.3, [1.0, 1.0, 0.6], [0, 15], true, false),
        "slime_block" => physics(0.0, 0.0, [1.0, 1.0, 0.8], [0, 1], true, false),
        "honey_block" => physics(0.0, 0.0, [0.4, 0.5, 0.6], [0, 1], false, false),
        "soul_sand" => physics(0.5, 0.5, [0.4, 1.0, 0.6], [0, 15], false, false),
        "magma_block" => physics(0.5, 0.5, [1.0, 1.0, 0.6], [3, 15], true, false),
        "lava_cauldron" => physics(2.0, 2.0, [1.0, 1.0, 0.6], [15, 0], false, false),
        "structure_block" | "jigsaw" => {
            physics(-1.0, 3_600_000.0, [1.0, 1.0, 0.6], [0, 15], true, false)
        }
        "structure_void" => physics(0.0, 0.0, [1.0, 1.0, 0.6], [0, 0], false, true),
        _ => return None,
    };
    Some(profile)
}

const fn physics(
    hardness: f32,
    resistance: f32,
    movement: [f32; 3],
    light: [u8; 2],
    full_collision: bool,
    replaceable: bool,
) -> BlockPhysics {
    BlockPhysics {
        hardness,
        resistance,
        speed_factor: movement[0],
        jump_factor: movement[1],
        friction: movement[2],
        light_emission: light[0],
        light_dampening: light[1],
        full_collision,
        replaceable,
    }
}
