//! Skull identity, power, durable data, animation, note sound, and wither summon.

use ferrite_foundation::direction::Direction;

pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 16;
pub const TYPE_COUNT: usize = 7;
pub const STATE_COUNT: usize = 280;
pub const WITHER_PATTERN_CELLS: u8 = 9;
pub const WITHER_ORIENTATIONS: u8 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkullType {
    Skeleton,
    WitherSkeleton,
    Zombie,
    Player,
    Creeper,
    Dragon,
    Piglin,
}

impl SkullType {
    const fn index(self) -> u32 {
        match self {
            Self::Skeleton => 0,
            Self::WitherSkeleton => 1,
            Self::Zombie => 2,
            Self::Player => 3,
            Self::Creeper => 4,
            Self::Dragon => 5,
            Self::Piglin => 6,
        }
    }

    pub const fn floor_state_id(self, rotation: u8, powered: bool) -> Option<u32> {
        if rotation > 15 {
            return None;
        }
        Some(10_915 + self.index() * 40 + rotation as u32 * 2 + powered as u32)
    }

    pub const fn wall_state_id(self, facing: Direction, powered: bool) -> Option<u32> {
        let facing = match facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            Direction::Down | Direction::Up => return None,
        };
        Some(10_947 + self.index() * 40 + facing * 2 + powered as u32)
    }

    pub const fn client_animates(self) -> bool {
        matches!(self, Self::Dragon | Self::Piglin)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkullData {
    pub profile: Option<String>,
    pub note_block_sound: Option<String>,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkullAnimation {
    pub counter: i32,
    pub active: bool,
}

impl SkullAnimation {
    pub const fn tick(mut self, powered: bool) -> Self {
        self.active = powered;
        if powered {
            self.counter = self.counter.wrapping_add(1);
        }
        self
    }

    pub fn sample(self, partial_tick: f32) -> f32 {
        self.counter as f32 + if self.active { partial_tick } else { 0.0 }
    }
}

pub fn dragon_jaw_rotation(animation: f32) -> f32 {
    (f32::sin(animation * std::f32::consts::PI * 0.2) + 1.0) * 0.2
}

pub fn piglin_ear_rotations(animation: f32) -> (f32, f32) {
    let phase = animation * std::f32::consts::PI * 0.2;
    (
        -(f32::cos(phase * 1.2) + 2.5) * 0.2,
        (f32::cos(phase) + 2.5) * 0.2,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerUpdate {
    pub changed: bool,
    pub powered: bool,
    pub flags: Option<u16>,
}

pub const fn neighbor_power(stored: bool, live_signal: bool, server: bool) -> PowerUpdate {
    let changed = server && stored != live_signal;
    PowerUpdate {
        changed,
        powered: if changed { live_signal } else { stored },
        flags: if changed { Some(2) } else { None },
    }
}

pub const fn floor_note_sound(skull_type: SkullType, note_identifier_present: bool) -> NoteSound {
    if matches!(skull_type, SkullType::Player) {
        if note_identifier_present {
            NoteSound::Custom {
                volume: 3,
                pitch: 1,
                consumes_random_long: true,
            }
        } else {
            NoteSound::None
        }
    } else {
        NoteSound::Fixed(skull_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteSound {
    None,
    Fixed(SkullType),
    Custom {
        volume: u8,
        pitch: u8,
        consumes_random_long: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitherAdmission {
    pub server: bool,
    pub correct_skull: bool,
    pub at_or_above_minimum_y: bool,
    pub peaceful: bool,
    pub pattern_matches: bool,
    pub entity_created: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitherResult {
    NoOp,
    Created {
        cleared_cells: u8,
        break_events: u8,
        criterion_before_entity_admission: bool,
        entity_admission_result_ignored: bool,
        neighbor_updates: u8,
    },
}

pub const fn summon_wither(admission: WitherAdmission) -> WitherResult {
    if !admission.server
        || !admission.correct_skull
        || !admission.at_or_above_minimum_y
        || admission.peaceful
        || !admission.pattern_matches
        || !admission.entity_created
    {
        return WitherResult::NoOp;
    }
    WitherResult::Created {
        cleared_cells: WITHER_PATTERN_CELLS,
        break_events: WITHER_PATTERN_CELLS,
        criterion_before_entity_admission: true,
        entity_admission_result_ignored: true,
        neighbor_updates: WITHER_PATTERN_CELLS,
    }
}
