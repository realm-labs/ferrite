//! Schooling, air, age, puffing, and aquatic subtype transitions.

pub const FISH_MAX_AIR: i16 = 300;
pub const DOLPHIN_MAX_AIR: i16 = 4_800;
pub const DOLPHIN_MAX_MOISTNESS: i32 = 2_400;
pub const GLOW_SQUID_DARK_TICKS: i32 = 100;
pub const TADPOLE_DEFAULT_CONVERSION_AGE: i32 = 24_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishKind {
    Cod,
    Salmon,
    Tropical,
}

impl FishKind {
    #[must_use]
    pub const fn school_capacity(self) -> u8 {
        match self {
            Self::Cod | Self::Tropical => 8,
            Self::Salmon => 5,
        }
    }

    #[must_use]
    pub const fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Cod => (500, 300),
            Self::Salmon => (700, 400),
            Self::Tropical => (500, 400),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchoolState {
    pub school_size: u8,
    pub has_leader: bool,
}

#[must_use]
pub const fn may_join_school(
    kind: FishKind,
    follower: SchoolState,
    leader: SchoolState,
    squared_distance: u16,
) -> bool {
    !follower.has_leader && leader.school_size < kind.school_capacity() && squared_distance <= 121
}

#[must_use]
pub fn school_search_due(tick: u16) -> bool {
    (200..=219).contains(&tick)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AirStep {
    pub next_air: i16,
    pub damage: u8,
}

#[must_use]
pub const fn fish_air_step(in_water: bool, air: i16) -> AirStep {
    if in_water {
        AirStep {
            next_air: FISH_MAX_AIR,
            damage: 0,
        }
    } else if air == -20 {
        AirStep {
            next_air: 0,
            damage: 2,
        }
    } else {
        AirStep {
            next_air: air - 1,
            damage: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SalmonSize {
    Small,
    Medium,
    Large,
}

impl SalmonSize {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Small,
            2 => Self::Large,
            _ => Self::Medium,
        }
    }

    #[must_use]
    pub const fn scale_millis(self) -> u16 {
        match self {
            Self::Small => 500,
            Self::Medium => 1_000,
            Self::Large => 1_500,
        }
    }
}

#[must_use]
pub const fn salmon_size_roll(draw_95: u8) -> SalmonSize {
    match draw_95 {
        0..=29 => SalmonSize::Small,
        30..=79 => SalmonSize::Medium,
        _ => SalmonSize::Large,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TropicalVariant {
    pub pattern: u16,
    pub base_color: u8,
    pub pattern_color: u8,
}

impl TropicalVariant {
    #[must_use]
    pub fn packed(self) -> u32 {
        u32::from(self.pattern)
            | (u32::from(self.base_color) << 16)
            | (u32::from(self.pattern_color) << 24)
    }

    #[must_use]
    pub const fn unpack(raw: u32) -> Self {
        Self {
            pattern: (raw & 0xffff) as u16,
            base_color: ((raw >> 16) & 0xff) as u8,
            pattern_color: ((raw >> 24) & 0xff) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TropicalSelection {
    Common { index: u8 },
    Rare(TropicalVariant),
}

#[must_use]
pub const fn tropical_selection(
    common_draw: u8,
    common_index: u8,
    rare: TropicalVariant,
) -> TropicalSelection {
    if common_draw < 90 {
        TropicalSelection::Common {
            index: common_index % 22,
        }
    } else {
        TropicalSelection::Rare(rare)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuffState {
    Small,
    Mid,
    Full,
    Other(i32),
}

impl PuffState {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Small,
            1 => Self::Mid,
            2 => Self::Full,
            value => Self::Other(value),
        }
    }

    #[must_use]
    pub const fn scale_millis(self) -> u16 {
        match self {
            Self::Small => 500,
            Self::Mid => 700,
            Self::Full | Self::Other(_) => 1_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PuffStep {
    pub state: PuffState,
    pub inflate_counter: u16,
    pub deflate_counter: u16,
}

#[must_use]
pub const fn puff_step(
    current: PuffState,
    inflate_counter: u16,
    deflate_counter: u16,
    scary_entity_nearby: bool,
) -> PuffStep {
    let mut state = current;
    let mut next_inflate = inflate_counter;
    let mut next_deflate = deflate_counter;
    if inflate_counter > 0 {
        state = match (current, inflate_counter) {
            (PuffState::Small, _) => PuffState::Mid,
            (PuffState::Mid, 41..) => PuffState::Full,
            _ => current,
        };
        next_inflate = inflate_counter.saturating_add(1);
    } else if !matches!(current, PuffState::Small) {
        state = match (current, deflate_counter) {
            (PuffState::Full, 61..) => PuffState::Mid,
            (PuffState::Mid, 101..) => PuffState::Small,
            _ => current,
        };
        next_deflate = deflate_counter.saturating_add(1);
    }

    if scary_entity_nearby && inflate_counter == 0 {
        next_inflate = 1;
        next_deflate = 0;
    } else if !scary_entity_nearby && inflate_counter > 0 {
        next_inflate = 0;
    }
    PuffStep {
        state,
        inflate_counter: next_inflate,
        deflate_counter: next_deflate,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sting {
    pub damage: i32,
    pub poison_ticks: i32,
}

#[must_use]
pub const fn pufferfish_sting(raw_state: i32, damage_admitted: bool) -> Sting {
    if raw_state <= 0 {
        return Sting {
            damage: 0,
            poison_ticks: 0,
        };
    }
    Sting {
        damage: 1 + raw_state,
        poison_ticks: if damage_admitted { 60 * raw_state } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SquidInk {
    pub packets: u8,
    pub position_float_draws: u16,
}

#[must_use]
pub const fn squid_ink(damage_admitted: bool, attacker_is_mob: bool) -> SquidInk {
    if damage_admitted && attacker_is_mob {
        SquidInk {
            packets: 30,
            position_float_draws: 90,
        }
    } else {
        SquidInk {
            packets: 0,
            position_float_draws: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlowInk {
    pub packets: u8,
    pub server_position_draws: u8,
    pub client_glow_requests: u8,
}

#[must_use]
pub const fn glow_squid_ink(damage_admitted: bool, attacker_is_mob: bool) -> GlowInk {
    if damage_admitted && attacker_is_mob {
        GlowInk {
            packets: 30,
            server_position_draws: 3,
            client_glow_requests: 1,
        }
    } else {
        GlowInk {
            packets: 0,
            server_position_draws: 0,
            client_glow_requests: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DolphinMoistureStep {
    pub moisture: i32,
    pub damage: u8,
    pub flop: bool,
}

#[must_use]
pub const fn dolphin_moisture_step(
    no_ai: bool,
    wet: bool,
    on_ground: bool,
    moisture: i32,
) -> DolphinMoistureStep {
    if no_ai || wet {
        return DolphinMoistureStep {
            moisture: DOLPHIN_MAX_MOISTNESS,
            damage: 0,
            flop: false,
        };
    }
    let moisture = moisture - 1;
    DolphinMoistureStep {
        moisture,
        damage: if moisture <= 0 { 1 } else { 0 },
        flop: on_ground,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TadpoleAgeStep {
    pub age: i32,
    pub converts: bool,
}

#[must_use]
pub const fn tadpole_age_step(age: i32, age_locked: bool, conversion_age: i32) -> TadpoleAgeStep {
    if age_locked {
        TadpoleAgeStep {
            age,
            converts: age >= conversion_age,
        }
    } else {
        let age = age.wrapping_add(1);
        TadpoleAgeStep {
            age,
            converts: age >= conversion_age,
        }
    }
}

#[must_use]
pub const fn tadpole_food_acceleration(age: i32, conversion_age: i32) -> i32 {
    let remaining = conversion_age.wrapping_sub(age);
    (remaining / 20) / 10 * 20
}
