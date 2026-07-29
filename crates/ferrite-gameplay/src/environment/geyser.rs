//! Potent-sulfur state, countdown, gas, plume, and launch transactions.

use super::fluid::{BLOCK_UPDATE_FLAGS, FluidRandom};

pub const BLOCK_ID: u16 = 999;
pub const ITEM_ID: u16 = 27;
pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 48;
pub const FIRST_STATE_ID: u32 = 24_688;
pub const STATE_COUNT: u8 = 5;
pub const HARDNESS: f32 = 1.5;
pub const RESISTANCE: f32 = 6.0;
pub const REQUIRES_CORRECT_TOOL: bool = true;
pub const SULFUR_BUBBLE_PARTICLE_ID: u16 = 4;
pub const NOXIOUS_GAS_CLOUD_PARTICLE_ID: u16 = 6;
pub const GEYSER_PARTICLE_ID: u16 = 7;
pub const ERUPTION_START_SOUND_ID: u16 = 1_922;
pub const ERUPTION_ACTIVE_SOUND_ID: u16 = 1_923;
pub const CONTINUOUS_START_SOUND_ID: u16 = 1_924;
pub const CONTINUOUS_ACTIVE_SOUND_ID: u16 = 1_925;
pub const NOXIOUS_GAS_SOUND_ID: u16 = 1_962;
pub const ENDER_DRAGON_ENTITY_TYPE_ID: u16 = 43;
pub const GEYSER_SALT: i64 = -904_011_478;
pub const MAX_WATER_BLOCKS: usize = 4;
pub const COLUMN_PROBE_COUNT: usize = 5;
pub const COUNTDOWN_FREQUENCY: i64 = 20;
pub const NAUSEA_FREQUENCY: i64 = 10;
pub const NAUSEA_DURATION: u16 = 80;
pub const NAUSEA_AMPLIFIER: u8 = 0;
pub const NAUSEA_RADIUS_SQUARED: f64 = 9.0;
pub const CLOUD_FREQUENCY: i64 = 20;
pub const PLUME_FREQUENCY: i64 = 20;
pub const ACTIVE_SOUND_FREQUENCY: i64 = 40;
pub const LAUNCH_HEIGHT_MULTIPLIER: usize = 6;
pub const LAUNCH_BASE_SPEED: f64 = 0.3_f32 as f64;
pub const LAUNCH_FORCE: f64 = 0.2_f32 as f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotentSulfurState {
    Dry,
    Wet,
    Dormant,
    Erupting,
    Continuous,
}

impl PotentSulfurState {
    pub const fn state_id(self) -> u32 {
        FIRST_STATE_ID
            + match self {
                Self::Dry => 0,
                Self::Wet => 1,
                Self::Dormant => 2,
                Self::Erupting => 3,
                Self::Continuous => 4,
            }
    }

    pub const fn active(self) -> bool {
        matches!(self, Self::Erupting | Self::Continuous)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportFluid {
    Empty,
    Source,
    Flowing,
}

impl SupportFluid {
    const fn source_if_fluid(self) -> bool {
        matches!(self, Self::Empty | Self::Source)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeyserSupport {
    pub continuous_tag: bool,
    pub periodic_tag: bool,
    pub fluid: SupportFluid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeyserDerivation {
    pub state: PotentSulfurState,
    pub reset_countdown: bool,
}

pub fn derive_potent_sulfur(
    current: PotentSulfurState,
    immediate_above_source_water: bool,
    support: GeyserSupport,
    matching_block_entity: bool,
) -> GeyserDerivation {
    if !immediate_above_source_water {
        return GeyserDerivation {
            state: PotentSulfurState::Dry,
            reset_countdown: false,
        };
    }
    if support.continuous_tag && support.fluid.source_if_fluid() {
        return GeyserDerivation {
            state: PotentSulfurState::Continuous,
            reset_countdown: false,
        };
    }
    if support.periodic_tag && support.fluid.source_if_fluid() {
        let retained = matches!(
            current,
            PotentSulfurState::Dormant | PotentSulfurState::Erupting
        );
        return GeyserDerivation {
            state: if current == PotentSulfurState::Erupting {
                current
            } else {
                PotentSulfurState::Dormant
            },
            reset_countdown: !retained && matching_block_entity,
        };
    }
    GeyserDerivation {
        state: PotentSulfurState::Wet,
        reset_countdown: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeyserGameEvent {
    BlockActivate,
    BlockDeactivate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeyserPlacementEffects {
    pub queue_event: Option<(u8, u8)>,
    pub sound_id: Option<u16>,
    pub game_event: Option<GeyserGameEvent>,
}

pub const fn geyser_on_place(state: PotentSulfurState) -> GeyserPlacementEffects {
    match state {
        PotentSulfurState::Erupting => GeyserPlacementEffects {
            queue_event: Some((0, 0)),
            sound_id: Some(ERUPTION_START_SOUND_ID),
            game_event: Some(GeyserGameEvent::BlockActivate),
        },
        PotentSulfurState::Continuous => GeyserPlacementEffects {
            queue_event: Some((0, 0)),
            sound_id: Some(CONTINUOUS_START_SOUND_ID),
            game_event: Some(GeyserGameEvent::BlockActivate),
        },
        PotentSulfurState::Dry | PotentSulfurState::Wet | PotentSulfurState::Dormant => {
            GeyserPlacementEffects {
                queue_event: None,
                sound_id: None,
                game_event: None,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeyserTickStage {
    Countdown,
    Nausea,
    GasCloud,
    Plume,
    Launch,
}

pub fn geyser_tick_stages(state: PotentSulfurState, client: bool) -> Vec<GeyserTickStage> {
    match (state, client) {
        (PotentSulfurState::Dry, _) => Vec::new(),
        (PotentSulfurState::Wet, false) => vec![GeyserTickStage::Nausea],
        (PotentSulfurState::Wet, true) => vec![GeyserTickStage::GasCloud],
        (PotentSulfurState::Dormant, false) => {
            vec![GeyserTickStage::Countdown, GeyserTickStage::Nausea]
        }
        (PotentSulfurState::Dormant, true) => vec![GeyserTickStage::GasCloud],
        (PotentSulfurState::Erupting, false) => {
            vec![GeyserTickStage::Launch, GeyserTickStage::Countdown]
        }
        (PotentSulfurState::Erupting, true) => {
            vec![GeyserTickStage::Plume, GeyserTickStage::Launch]
        }
        (PotentSulfurState::Continuous, false) => vec![GeyserTickStage::Launch],
        (PotentSulfurState::Continuous, true) => {
            vec![GeyserTickStage::Plume, GeyserTickStage::Launch]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnCell {
    pub source_water: bool,
    pub water_block: bool,
    pub air: bool,
    pub empty_collision: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GasSource {
    pub source_offset_y: u8,
    pub water_blocks: u8,
}

pub fn find_gas_source(cells: &[ColumnCell; COLUMN_PROBE_COUNT]) -> Option<GasSource> {
    for (index, cell) in cells.iter().enumerate() {
        let passable = cell.air || cell.water_block || cell.empty_collision;
        if cell.source_water {
            if !cell.water_block && !passable {
                return None;
            }
            continue;
        }
        if !cell.air && !passable {
            return None;
        }
        return Some(GasSource {
            source_offset_y: index as u8 + 1,
            water_blocks: index as u8,
        });
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeyserRuntime {
    pub waiting_countdown: i32,
    pub eruption_tick: i64,
}

impl Default for GeyserRuntime {
    fn default() -> Self {
        Self {
            waiting_countdown: -1,
            eruption_tick: -1,
        }
    }
}

impl GeyserRuntime {
    pub const fn load_countdown(stored: Option<i32>) -> Self {
        Self {
            waiting_countdown: match stored {
                Some(value) => value,
                None => -1,
            },
            eruption_tick: -1,
        }
    }

    pub const fn saved_countdown(self) -> i32 {
        self.waiting_countdown
    }

    pub fn set_level(&mut self, game_time: i64) {
        if self.eruption_tick == -1 {
            self.eruption_tick = game_time;
        }
    }

    pub fn trigger_event(&mut self, matching_block_entity: bool, game_time: i64) -> bool {
        if matching_block_entity {
            self.eruption_tick = game_time;
        }
        true
    }

    pub fn reset_countdown(&mut self) {
        self.waiting_countdown = -1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountdownOutcome {
    pub ran: bool,
    pub initialized: bool,
    pub discarded_unbounded_draw: bool,
    pub countdown: i32,
    pub state_write: Option<PotentSulfurState>,
    pub write_flags: Option<u16>,
    pub game_event: Option<GeyserGameEvent>,
}

pub fn countdown_tick<R: FluidRandom>(
    state: PotentSulfurState,
    game_time: i64,
    source: Option<GasSource>,
    runtime: &mut GeyserRuntime,
    random: &mut R,
) -> CountdownOutcome {
    if game_time.rem_euclid(COUNTDOWN_FREQUENCY) != 0
        || !matches!(
            state,
            PotentSulfurState::Dormant | PotentSulfurState::Erupting
        )
        || source.is_none()
    {
        return CountdownOutcome {
            ran: false,
            initialized: false,
            discarded_unbounded_draw: false,
            countdown: runtime.waiting_countdown,
            state_write: None,
            write_flags: None,
            game_event: None,
        };
    }
    let source = source.expect("source checked");
    let mut initialized = false;
    let mut discarded = false;
    if runtime.waiting_countdown <= 0 {
        initialized = true;
        runtime.waiting_countdown = match state {
            PotentSulfurState::Dormant => {
                10 * (i32::from(source.water_blocks) - 1) + 15 + random.next_int(16) as i32
            }
            PotentSulfurState::Erupting => {
                random.next_unbounded_int();
                discarded = true;
                i32::from(source.water_blocks) - 1 + 1 + random.next_int(2) as i32
            }
            PotentSulfurState::Dry | PotentSulfurState::Wet | PotentSulfurState::Continuous => {
                unreachable!("state filtered")
            }
        };
    }
    if runtime.waiting_countdown > 0 {
        runtime.waiting_countdown -= 1;
    }
    let state_write = (runtime.waiting_countdown == 0).then_some(match state {
        PotentSulfurState::Dormant => PotentSulfurState::Erupting,
        PotentSulfurState::Erupting => PotentSulfurState::Dormant,
        _ => unreachable!("state filtered"),
    });
    CountdownOutcome {
        ran: true,
        initialized,
        discarded_unbounded_draw: discarded,
        countdown: runtime.waiting_countdown,
        state_write,
        write_flags: state_write.map(|_| BLOCK_UPDATE_FLAGS),
        game_event: (state_write == Some(PotentSulfurState::Dormant))
            .then_some(GeyserGameEvent::BlockDeactivate),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasEntity {
    pub id: u64,
    pub alive: bool,
    pub spectator: bool,
    pub intersects_horizontal_query: bool,
    pub eye_cell_passable: bool,
    pub eye_distance_squared: f64,
    pub source_water_below_eye: bool,
    pub collider_clip_hit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NauseaApplication {
    pub entity_id: u64,
    pub duration: u16,
    pub amplifier: u8,
    pub ambient: bool,
    pub visible: bool,
}

pub fn nausea_applications(
    game_time: i64,
    source: Option<GasSource>,
    entities: &[GasEntity],
) -> Vec<NauseaApplication> {
    if game_time.rem_euclid(NAUSEA_FREQUENCY) != 0 || source.is_none() {
        return Vec::new();
    }
    entities
        .iter()
        .filter(|entity| {
            entity.alive
                && !entity.spectator
                && entity.intersects_horizontal_query
                && entity.eye_cell_passable
                && entity.eye_distance_squared <= NAUSEA_RADIUS_SQUARED
                && entity.source_water_below_eye
                && !entity.collider_clip_hit
        })
        .map(|entity| NauseaApplication {
            entity_id: entity.id,
            duration: NAUSEA_DURATION,
            amplifier: NAUSEA_AMPLIFIER,
            ambient: true,
            visible: true,
        })
        .collect()
}

pub const fn client_gas_cloud(game_time: i64, source: Option<GasSource>) -> bool {
    game_time.rem_euclid(CLOUD_FREQUENCY) == 0 && source.is_some()
}

pub fn unobstructed_count(water_blocks: u8, passable: &[bool]) -> usize {
    let maximum = LAUNCH_HEIGHT_MULTIPLIER * usize::from(water_blocks);
    passable
        .iter()
        .take(maximum)
        .position(|passable| !passable)
        .unwrap_or(maximum)
}

pub const fn launch_query_expand_y(unobstructed_count: usize) -> i32 {
    unobstructed_count as i32 - 1
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchEntity {
    pub id: u64,
    pub alive: bool,
    pub spectator: bool,
    pub can_simulate_movement: bool,
    pub flying_player: bool,
    pub passenger: bool,
    pub immune_to_geysers: bool,
    pub vertical_velocity: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchPlan {
    pub threshold: f64,
    pub fall_distance_entity_ids: Vec<u64>,
    pub launched_entity_ids: Vec<u64>,
    pub velocity_addition: f64,
    pub mark_sync: bool,
}

pub fn geyser_launch(water_blocks: u8, entities: &[LaunchEntity]) -> LaunchPlan {
    let threshold = LAUNCH_BASE_SPEED + f64::from(water_blocks) * 0.1;
    let candidates: Vec<_> = entities
        .iter()
        .filter(|entity| entity.alive && !entity.spectator)
        .collect();
    let fall_distance_entity_ids = candidates.iter().map(|entity| entity.id).collect();
    let launched_entity_ids = candidates
        .into_iter()
        .filter(|entity| {
            entity.can_simulate_movement
                && !entity.flying_player
                && !entity.passenger
                && !entity.immune_to_geysers
                && entity.vertical_velocity < threshold
        })
        .map(|entity| entity.id)
        .collect();
    LaunchPlan {
        threshold,
        fall_distance_entity_ids,
        launched_entity_ids,
        velocity_addition: LAUNCH_FORCE,
        mark_sync: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlumePlan {
    pub geyser_particle: bool,
    pub particle_water_blocks: Option<u8>,
    pub play_active_sound: bool,
    pub sound_id: Option<u16>,
}

pub fn client_plume(
    state: PotentSulfurState,
    game_time: i64,
    eruption_tick: i64,
    source: Option<GasSource>,
) -> PlumePlan {
    let Some(source) = source else {
        return PlumePlan {
            geyser_particle: false,
            particle_water_blocks: None,
            play_active_sound: false,
            sound_id: None,
        };
    };
    if !state.active() {
        return PlumePlan {
            geyser_particle: false,
            particle_water_blocks: None,
            play_active_sound: false,
            sound_id: None,
        };
    }
    let elapsed = game_time.wrapping_sub(eruption_tick);
    let particle = elapsed.rem_euclid(PLUME_FREQUENCY) == 0;
    let sound = elapsed.rem_euclid(ACTIVE_SOUND_FREQUENCY) == 0;
    PlumePlan {
        geyser_particle: particle,
        particle_water_blocks: particle.then_some(source.water_blocks),
        play_active_sound: sound,
        sound_id: sound.then_some(match state {
            PotentSulfurState::Erupting => ERUPTION_ACTIVE_SOUND_ID,
            PotentSulfurState::Continuous => CONTINUOUS_ACTIVE_SOUND_ID,
            _ => unreachable!("active checked"),
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeyserDisplay {
    pub bubble_positions: Vec<[f32; 3]>,
    pub play_noxious_sound: bool,
    pub sound_position_at_integer_corner: bool,
}

pub fn geyser_display_tick<R: FluidRandom>(
    state: PotentSulfurState,
    immediate_above_source_water: bool,
    random: &mut R,
) -> GeyserDisplay {
    if state == PotentSulfurState::Dry || !immediate_above_source_water {
        return GeyserDisplay {
            bubble_positions: Vec::new(),
            play_noxious_sound: false,
            sound_position_at_integer_corner: true,
        };
    }
    let bubble_positions = (0..2)
        .map(|_| {
            [
                random.next_float(),
                1.0 + random.next_float(),
                random.next_float(),
            ]
        })
        .collect();
    GeyserDisplay {
        bubble_positions,
        play_noxious_sound: random.next_int(10) == 0,
        sound_position_at_integer_corner: true,
    }
}

pub const fn potent_sulfur_loot(survives_explosion: bool) -> bool {
    survives_explosion
}
