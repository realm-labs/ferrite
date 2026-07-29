//! Sculk-sensor vibration selection, travel, activation, and signals.

use ferrite_foundation::direction::Direction;

pub const ORDINARY_RADIUS: u8 = 8;
pub const CALIBRATED_RADIUS: u8 = 16;
pub const ORDINARY_ACTIVE_TICKS: u64 = 30;
pub const CALIBRATED_ACTIVE_TICKS: u64 = 10;
pub const COOLDOWN_TICKS: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorPhase {
    Inactive,
    Active,
    Cooldown,
}

impl SensorPhase {
    const fn index(self) -> u32 {
        match self {
            Self::Inactive => 0,
            Self::Active => 1,
            Self::Cooldown => 2,
        }
    }
}

pub const fn ordinary_state_id(power: u8, phase: SensorPhase, waterlogged: bool) -> Option<u32> {
    if power > 15 {
        return None;
    }
    Some(27_163 + 6 * power as u32 + 2 * phase.index() + if waterlogged { 0 } else { 1 })
}

pub const fn calibrated_state_id(
    facing: Direction,
    power: u8,
    phase: SensorPhase,
    waterlogged: bool,
) -> Option<u32> {
    if power > 15 {
        return None;
    }
    let facing = match facing {
        Direction::North => 0,
        Direction::South => 1,
        Direction::West => 2,
        Direction::East => 3,
        Direction::Down | Direction::Up => return None,
    };
    Some(
        27_259
            + 96 * facing
            + 6 * power as u32
            + 2 * phase.index()
            + if waterlogged { 0 } else { 1 },
    )
}

pub fn vibration_frequency(event: &str) -> u8 {
    if let Some(value) = event
        .strip_prefix("resonate_")
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|value| (1..=15).contains(value))
    {
        return value;
    }
    match event {
        "step" | "swim" | "flap" => 1,
        "projectile_land" | "hit_ground" | "splash" | "bounce" => 2,
        "item_interact_finish" | "projectile_shoot" | "instrument_play" => 3,
        "entity_action" | "elytra_glide" | "unequip" => 4,
        "entity_dismount" | "equip" => 5,
        "entity_interact" | "shear" | "entity_mount" => 6,
        "entity_damage" => 7,
        "drink" | "eat" => 8,
        "container_close" | "block_close" | "block_deactivate" | "block_detach" => 9,
        "container_open" | "block_open" | "block_activate" | "block_attach" | "prime_fuse"
        | "note_block_play" => 10,
        "block_change" => 11,
        "block_destroy" | "fluid_pickup" => 12,
        "block_place" | "fluid_place" => 13,
        "entity_place" | "lightning_strike" | "teleport" => 14,
        "entity_die" | "explode" => 15,
        _ => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VibrationCandidate {
    pub tick: u64,
    pub exact_distance: f64,
    pub frequency: u8,
}

pub const fn select_candidate(
    retained: Option<VibrationCandidate>,
    offered: VibrationCandidate,
) -> Option<VibrationCandidate> {
    match retained {
        None => Some(offered),
        Some(current) if current.tick != offered.tick => Some(current),
        Some(current) if offered.exact_distance < current.exact_distance => Some(offered),
        Some(current)
            if offered.exact_distance == current.exact_distance
                && offered.frequency > current.frequency =>
        {
            Some(offered)
        }
        Some(current) => Some(current),
    }
}

pub const fn selection_ready(candidate_tick: u64, game_time: u64) -> bool {
    candidate_tick < game_time
}

pub fn travel_delay(exact_distance: f64) -> u32 {
    exact_distance.floor().max(0.0) as u32
}

pub fn arrival_power(block_distance: f64, radius: u8) -> u8 {
    let attenuation = (15.0 * block_distance / f64::from(radius)).floor() as u8;
    15_u8.saturating_sub(attenuation).max(1)
}

pub const fn calibrated_admits(back_signal: u8, frequency: u8) -> bool {
    back_signal == 0 || back_signal == frequency
}

pub const fn chunks_admit_delivery(should_tick_and_loaded: [bool; 9]) -> bool {
    let mut index = 0;
    while index < should_tick_and_loaded.len() {
        if !should_tick_and_loaded[index] {
            return false;
        }
        index += 1;
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Activation {
    pub power: u8,
    pub active_ticks: u64,
    pub cooldown_ticks: u64,
    pub write_flags: u16,
    pub notify_neighbor_positions: u8,
    pub dry_sound_float_draws: u8,
}

pub const fn activation(power: u8, calibrated: bool, waterlogged: bool) -> Activation {
    Activation {
        power,
        active_ticks: if calibrated {
            CALIBRATED_ACTIVE_TICKS
        } else {
            ORDINARY_ACTIVE_TICKS
        },
        cooldown_ticks: COOLDOWN_TICKS,
        write_flags: 3,
        notify_neighbor_positions: 2,
        dry_sound_float_draws: if waterlogged { 0 } else { 1 },
    }
}

pub fn weak_signal(power: u8, calibrated_facing: Option<Direction>, query: Direction) -> u8 {
    if calibrated_facing == Some(query) {
        0
    } else {
        power
    }
}

pub const fn direct_signal(power: u8, query: Direction) -> u8 {
    if matches!(query, Direction::Up) {
        power
    } else {
        0
    }
}

pub const fn comparator_signal(phase: SensorPhase, entity_present: bool, frequency: u8) -> u8 {
    if matches!(phase, SensorPhase::Active) && entity_present {
        frequency
    } else {
        0
    }
}

pub const RESONANCE_NOTES: [u8; 15] = [0, 2, 4, 6, 7, 9, 10, 12, 14, 15, 18, 19, 21, 22, 24];

pub const fn resonance_note(frequency: u8) -> Option<u8> {
    if frequency == 0 || frequency > 15 {
        None
    } else {
        Some(RESONANCE_NOTES[frequency as usize - 1])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedListener {
    pub last_frequency: u8,
    pub event_delay: u32,
    pub has_current_vibration: bool,
    pub reload_particle_requested: bool,
}

impl PersistedListener {
    pub const fn decode(last_frequency: u8, event_delay: u32, has_current: bool) -> Self {
        Self {
            last_frequency,
            event_delay,
            has_current_vibration: has_current,
            reload_particle_requested: true,
        }
    }
}
