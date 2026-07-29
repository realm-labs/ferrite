//! Conduit water/frame refresh, effects, targeting, and ambient clocks.

pub const WATER_CELL_COUNT: usize = 27;
pub const FRAME_CELL_COUNT: usize = 42;
pub const REFRESH_INTERVAL: u64 = 40;
pub const AMBIENT_INTERVAL: u64 = 80;
pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameResult {
    pub water_complete: bool,
    pub frame_count: u8,
    pub active: bool,
    pub hunting: bool,
}

pub fn scan_frame(water_cells: &[bool], frame_cells: &[bool]) -> FrameResult {
    let water_complete = water_cells.iter().take(WATER_CELL_COUNT).all(|cell| *cell)
        && water_cells.len() >= WATER_CELL_COUNT;
    let frame_count = if water_complete {
        frame_cells
            .iter()
            .take(FRAME_CELL_COUNT)
            .filter(|cell| **cell)
            .count() as u8
    } else {
        0
    };
    FrameResult {
        water_complete,
        frame_count,
        active: frame_count >= 16,
        hunting: frame_count >= 42,
    }
}

pub const fn effect_radius(frame_count: u8) -> u16 {
    16 * (frame_count / 7) as u16
}

pub const fn player_receives_power(
    frame_count: u8,
    block_distance_squared: u32,
    wet: bool,
) -> bool {
    let radius = effect_radius(frame_count) as u32;
    frame_count >= 16 && wet && block_distance_squared <= radius * radius
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetSnapshot {
    pub alive: bool,
    pub block_distance_squared: u16,
}

pub const fn retain_target(target: TargetSnapshot) -> bool {
    target.alive && target.block_distance_squared < 64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetRefresh {
    InactiveRetain,
    PartialClear,
    Retain,
    ClearWithoutReselect,
    SelectCandidate,
}

pub const fn target_refresh(
    active: bool,
    hunting: bool,
    current: Option<TargetSnapshot>,
) -> TargetRefresh {
    if !active {
        return TargetRefresh::InactiveRetain;
    }
    if !hunting {
        return TargetRefresh::PartialClear;
    }
    match current {
        Some(target) if retain_target(target) => TargetRefresh::Retain,
        Some(_) => TargetRefresh::ClearWithoutReselect,
        None => TargetRefresh::SelectCandidate,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackResult {
    pub play_sound: bool,
    pub damage: u8,
    pub consult_damage_result: bool,
}

pub const fn attack() -> AttackResult {
    AttackResult {
        play_sound: true,
        damage: 4,
        consult_damage_result: false,
    }
}

pub const fn next_short_ambient_deadline(game_time: u64, next_int_40: u32) -> u64 {
    game_time + 60 + if next_int_40 < 40 { next_int_40 } else { 39 } as u64
}

pub const fn short_ambient_due(game_time: u64, deadline: u64) -> bool {
    game_time > deadline
}

pub const fn periodic_refresh_due(game_time: u64) -> bool {
    game_time.is_multiple_of(REFRESH_INTERVAL)
}

pub const fn long_ambient_due(game_time: u64) -> bool {
    game_time.is_multiple_of(AMBIENT_INTERVAL)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientParticleDraws {
    pub frame_gate_draws: u8,
    pub frame_float_draws: u8,
    pub target_float_draws: u8,
}

pub const fn client_particle_draws(
    frame_gate_hit: bool,
    target_present: bool,
) -> ClientParticleDraws {
    ClientParticleDraws {
        frame_gate_draws: 1,
        frame_float_draws: if frame_gate_hit { 3 } else { 0 },
        target_float_draws: if target_present { 3 } else { 0 },
    }
}
