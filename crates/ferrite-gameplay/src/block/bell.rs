//! Bell state, ingress, cached hearing, resonance, and rendering decisions.

use ferrite_foundation::direction::{Axis, Direction};

pub const BLOCK_ID: u16 = 848;
pub const ITEM_ID: u16 = 1_393;
pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 30;
pub const DEFAULT_STATE_ID: u32 = 20_806;
pub const FIRST_STATE_ID: u32 = 20_805;
pub const STATE_COUNT: u32 = 32;
pub const MAX_STACK: u8 = 64;
pub const HARDNESS: f32 = 5.0;
pub const RESISTANCE: f32 = 5.0;
pub const FORCED_SOLID: bool = true;
pub const DESTROYED_BY_PISTON: bool = true;
pub const RING_SOUND_ID: u16 = 167;
pub const RESONATE_SOUND_ID: u16 = 168;
pub const ENTITY_EFFECT_PARTICLE_ID: u16 = 28;
pub const HEARD_BELL_MEMORY_ID: u16 = 29;
pub const BELL_RING_STAT_ID: u16 = 70;
pub const BLOCK_UPDATE_FLAGS: u16 = 3;
pub const RING_EVENT_ID: u8 = 1;
pub const HIT_Y_LIMIT: f64 = 0.8124_f32 as f64;
pub const CACHE_RADIUS: f64 = 48.0;
pub const HEARING_RADIUS: f64 = 32.0;
pub const CACHE_INTERVAL: u64 = 60;
pub const SHAKE_TICKS: u16 = 50;
pub const RESONANCE_DELAY: u16 = 5;
pub const RESONANCE_TICKS: u16 = 40;
pub const GLOW_TICKS: u16 = 60;
pub const GLOW_AMPLIFIER: u8 = 0;
pub const RAIDER_ENTITY_TYPE_IDS: [u16; 6] = [46, 103, 109, 141, 68, 145];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BellAttachment {
    Floor,
    Ceiling,
    SingleWall,
    DoubleWall,
}

impl BellAttachment {
    const fn state_offset(self) -> u32 {
        match self {
            Self::Floor => 0,
            Self::Ceiling => 8,
            Self::SingleWall => 16,
            Self::DoubleWall => 24,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellState {
    pub facing: Direction,
    pub attachment: BellAttachment,
    pub powered: bool,
}

impl Default for BellState {
    fn default() -> Self {
        Self {
            facing: Direction::North,
            attachment: BellAttachment::Floor,
            powered: false,
        }
    }
}

impl BellState {
    pub const fn state_id(self) -> Option<u32> {
        let facing = match self.facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            Direction::Down | Direction::Up => return None,
        };
        Some(
            FIRST_STATE_ID
                + self.attachment.state_offset()
                + facing * 2
                + if self.powered { 0 } else { 1 },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BellRotation {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BellMirror {
    None,
    LeftRight,
    FrontBack,
}

pub const fn rotate_bell(mut state: BellState, rotation: BellRotation) -> BellState {
    state.facing = match (state.facing, rotation) {
        (facing, BellRotation::None) => facing,
        (Direction::North, BellRotation::Clockwise90) => Direction::East,
        (Direction::East, BellRotation::Clockwise90) => Direction::South,
        (Direction::South, BellRotation::Clockwise90) => Direction::West,
        (Direction::West, BellRotation::Clockwise90) => Direction::North,
        (Direction::North, BellRotation::Clockwise180) => Direction::South,
        (Direction::South, BellRotation::Clockwise180) => Direction::North,
        (Direction::West, BellRotation::Clockwise180) => Direction::East,
        (Direction::East, BellRotation::Clockwise180) => Direction::West,
        (Direction::North, BellRotation::CounterClockwise90) => Direction::West,
        (Direction::West, BellRotation::CounterClockwise90) => Direction::South,
        (Direction::South, BellRotation::CounterClockwise90) => Direction::East,
        (Direction::East, BellRotation::CounterClockwise90) => Direction::North,
        (Direction::Down | Direction::Up, _) => state.facing,
    };
    state
}

pub const fn mirror_bell(state: BellState, mirror: BellMirror) -> BellState {
    let rotation = match (mirror, state.facing.axis()) {
        (BellMirror::LeftRight, Axis::Z) | (BellMirror::FrontBack, Axis::X) => {
            BellRotation::Clockwise180
        }
        _ => BellRotation::None,
    };
    rotate_bell(state, rotation)
}

pub const fn bell_use_without_item_admitted(
    main_hand_attempt: bool,
    secondary_use: bool,
    main_hand_nonempty: bool,
    off_hand_nonempty: bool,
) -> bool {
    main_hand_attempt && !(secondary_use && (main_hand_nonempty || off_hand_nonempty))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BellSupports {
    pub below_top: bool,
    pub above_center: bool,
    pub above_unstable_bottom_center: bool,
    pub north: bool,
    pub south: bool,
    pub west: bool,
    pub east: bool,
}

impl BellSupports {
    pub const fn sturdy_toward(self, direction: Direction) -> bool {
        match direction {
            Direction::Down => self.below_top,
            Direction::Up => self.above_center && !self.above_unstable_bottom_center,
            Direction::North => self.north,
            Direction::South => self.south,
            Direction::West => self.west,
            Direction::East => self.east,
        }
    }
}

pub fn bell_placement(
    clicked_face: Direction,
    player_horizontal: Direction,
    supports: BellSupports,
) -> Option<BellState> {
    let facing = horizontal_or_north(player_horizontal);
    if clicked_face.axis() == Axis::Y {
        let attachment = if clicked_face == Direction::Down {
            BellAttachment::Ceiling
        } else {
            BellAttachment::Floor
        };
        let state = BellState {
            facing,
            attachment,
            powered: false,
        };
        return bell_survives(state, supports).then_some(state);
    }

    let facing = clicked_face.opposite();
    let double_attached =
        supports.sturdy_toward(facing) && supports.sturdy_toward(facing.opposite());
    let wall = BellState {
        facing,
        attachment: if double_attached {
            BellAttachment::DoubleWall
        } else {
            BellAttachment::SingleWall
        },
        powered: false,
    };
    if bell_survives(wall, supports) {
        return Some(wall);
    }

    let fallback = BellState {
        attachment: if supports.below_top {
            BellAttachment::Floor
        } else {
            BellAttachment::Ceiling
        },
        ..wall
    };
    bell_survives(fallback, supports).then_some(fallback)
}

pub const fn bell_survives(state: BellState, supports: BellSupports) -> bool {
    match state.attachment {
        BellAttachment::Floor => supports.sturdy_toward(Direction::Down),
        BellAttachment::Ceiling => supports.sturdy_toward(Direction::Up),
        BellAttachment::SingleWall | BellAttachment::DoubleWall => {
            supports.sturdy_toward(state.facing)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BellSupportUpdate {
    Unchanged,
    State(BellState),
    Air,
}

pub fn bell_support_update(
    state: BellState,
    changed_direction: Direction,
    neighbour_face_sturdy: bool,
    supports: BellSupports,
) -> BellSupportUpdate {
    let connected = match state.attachment {
        BellAttachment::Floor => Direction::Down,
        BellAttachment::Ceiling => Direction::Up,
        BellAttachment::SingleWall | BellAttachment::DoubleWall => state.facing,
    };
    if changed_direction == connected
        && state.attachment != BellAttachment::DoubleWall
        && !bell_survives(state, supports)
    {
        return BellSupportUpdate::Air;
    }
    if changed_direction.axis() != state.facing.axis() {
        return BellSupportUpdate::Unchanged;
    }
    match state.attachment {
        BellAttachment::DoubleWall if !neighbour_face_sturdy => {
            BellSupportUpdate::State(BellState {
                facing: changed_direction.opposite(),
                attachment: BellAttachment::SingleWall,
                ..state
            })
        }
        BellAttachment::SingleWall
            if changed_direction == state.facing.opposite() && neighbour_face_sturdy =>
        {
            BellSupportUpdate::State(BellState {
                attachment: BellAttachment::DoubleWall,
                ..state
            })
        }
        _ => BellSupportUpdate::Unchanged,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb16 {
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}

impl Aabb16 {
    const fn new(min_x: f32, min_y: f32, min_z: f32, max_x: f32, max_y: f32, max_z: f32) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}

pub fn bell_shape(state: BellState) -> Vec<Aabb16> {
    match state.attachment {
        BellAttachment::Floor => {
            if state.facing.axis() == Axis::X {
                vec![Aabb16::new(4.0, 0.0, 0.0, 12.0, 16.0, 16.0)]
            } else {
                vec![Aabb16::new(0.0, 0.0, 4.0, 16.0, 16.0, 12.0)]
            }
        }
        BellAttachment::Ceiling => vec![
            bell_lower_shape(),
            bell_upper_shape(),
            Aabb16::new(7.0, 13.0, 7.0, 9.0, 16.0, 9.0),
        ],
        BellAttachment::DoubleWall => {
            let bracket = if state.facing.axis() == Axis::X {
                Aabb16::new(0.0, 13.0, 7.0, 16.0, 15.0, 9.0)
            } else {
                Aabb16::new(7.0, 13.0, 0.0, 9.0, 15.0, 16.0)
            };
            vec![bell_lower_shape(), bell_upper_shape(), bracket]
        }
        BellAttachment::SingleWall => vec![
            bell_lower_shape(),
            bell_upper_shape(),
            single_wall_bracket(state.facing),
        ],
    }
}

const fn bell_lower_shape() -> Aabb16 {
    Aabb16::new(5.0, 6.0, 5.0, 11.0, 13.0, 11.0)
}

const fn bell_upper_shape() -> Aabb16 {
    Aabb16::new(4.0, 4.0, 4.0, 12.0, 6.0, 12.0)
}

const fn single_wall_bracket(facing: Direction) -> Aabb16 {
    match facing {
        Direction::North => Aabb16::new(7.0, 13.0, 0.0, 9.0, 15.0, 13.0),
        Direction::South => Aabb16::new(7.0, 13.0, 3.0, 9.0, 15.0, 16.0),
        Direction::West => Aabb16::new(0.0, 13.0, 7.0, 13.0, 15.0, 9.0),
        Direction::East => Aabb16::new(3.0, 13.0, 7.0, 16.0, 15.0, 9.0),
        Direction::Down | Direction::Up => Aabb16::new(7.0, 13.0, 0.0, 9.0, 15.0, 13.0),
    }
}

pub fn proper_bell_hit(state: BellState, clicked_face: Direction, local_y: f64) -> bool {
    if clicked_face.axis() == Axis::Y || local_y > HIT_Y_LIMIT {
        return false;
    }
    match state.attachment {
        BellAttachment::Floor => state.facing.axis() == clicked_face.axis(),
        BellAttachment::SingleWall | BellAttachment::DoubleWall => {
            state.facing.axis() != clicked_face.axis()
        }
        BellAttachment::Ceiling => true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BellLiving {
    pub id: u64,
    pub position: [f64; 3],
    pub alive: bool,
    pub removed: bool,
    pub raider: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellQueuedEvent {
    pub event_id: u8,
    pub parameter: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BellEventOutcome {
    pub handled: bool,
    pub refreshed_cache: bool,
    pub heard_entity_ids: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BellSide {
    Server,
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellRingOutcome {
    pub admitted_hit: bool,
    pub rang: bool,
    pub queue: Option<BellQueuedEvent>,
    pub play_sound: bool,
    pub emit_block_change: bool,
    pub award_stat: bool,
}

impl BellRingOutcome {
    const fn rejected() -> Self {
        Self {
            admitted_hit: false,
            rang: false,
            queue: None,
            play_sound: false,
            emit_block_change: false,
            award_stat: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BellParticle {
    pub entity_id: u64,
    pub color: u32,
    pub position: [f64; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct BellTickOutcome {
    pub resonance_sound_call: bool,
    pub audible_resonance_sound: bool,
    pub glowing_entity_ids: Vec<u64>,
    pub particles: Vec<BellParticle>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BellRuntime {
    pub last_ring_timestamp: u64,
    pub ticks: u16,
    pub shaking: bool,
    pub click_direction: Option<Direction>,
    pub resonating: bool,
    pub resonance_ticks: u16,
    nearby_entities: Option<Vec<BellLiving>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BellHit {
    pub side: BellSide,
    pub matching_block_entity: bool,
    pub state: BellState,
    pub clicked_face: Direction,
    pub local_y: f64,
    pub require_correct_side: bool,
    pub player_source: bool,
}

impl BellRuntime {
    pub fn cached_entity_ids(&self) -> Option<Vec<u64>> {
        self.nearby_entities
            .as_ref()
            .map(|entities| entities.iter().map(|entity| entity.id).collect())
    }

    pub fn on_hit(&mut self, direction: Direction) -> BellQueuedEvent {
        self.click_direction = Some(direction);
        if self.shaking {
            self.ticks = 0;
        } else {
            self.shaking = true;
        }
        BellQueuedEvent {
            event_id: RING_EVENT_ID,
            parameter: direction_data_value(direction),
        }
    }

    pub fn attempt_to_ring(
        &mut self,
        side: BellSide,
        matching_block_entity: bool,
        state: BellState,
        direction: Option<Direction>,
        player_source: bool,
    ) -> BellRingOutcome {
        if side == BellSide::Client || !matching_block_entity {
            return BellRingOutcome {
                admitted_hit: true,
                ..BellRingOutcome::rejected()
            };
        }
        let queue = self.on_hit(direction.unwrap_or(state.facing));
        BellRingOutcome {
            admitted_hit: true,
            rang: true,
            queue: Some(queue),
            play_sound: true,
            emit_block_change: true,
            award_stat: player_source,
        }
    }

    pub fn hit(&mut self, hit: BellHit) -> BellRingOutcome {
        if hit.require_correct_side && !proper_bell_hit(hit.state, hit.clicked_face, hit.local_y) {
            return BellRingOutcome::rejected();
        }
        self.attempt_to_ring(
            hit.side,
            hit.matching_block_entity,
            hit.state,
            Some(hit.clicked_face),
            hit.player_source,
        )
    }

    pub fn trigger_event(
        &mut self,
        event_id: u8,
        parameter: i32,
        game_time: u64,
        side: BellSide,
        queried_entities: &[BellLiving],
    ) -> BellEventOutcome {
        if event_id != RING_EVENT_ID {
            return BellEventOutcome {
                handled: false,
                refreshed_cache: false,
                heard_entity_ids: Vec::new(),
            };
        }
        let refresh = self.nearby_entities.is_none()
            || game_time > self.last_ring_timestamp.saturating_add(CACHE_INTERVAL);
        if refresh {
            self.last_ring_timestamp = game_time;
            self.nearby_entities = Some(queried_entities.to_vec());
        }
        let current = self.current_cached_entities(queried_entities);
        let heard_entity_ids = if side == BellSide::Server {
            current
                .iter()
                .filter(|entity| {
                    entity.alive
                        && !entity.removed
                        && within_center(entity.position, HEARING_RADIUS)
                })
                .map(|entity| entity.id)
                .collect()
        } else {
            Vec::new()
        };
        self.resonance_ticks = 0;
        self.click_direction = Some(direction_from_data_value(parameter));
        self.ticks = 0;
        self.shaking = true;
        BellEventOutcome {
            handled: true,
            refreshed_cache: refresh,
            heard_entity_ids,
        }
    }

    pub fn tick(&mut self, side: BellSide, current_entities: &[BellLiving]) -> BellTickOutcome {
        if self.shaking {
            self.ticks += 1;
        }
        if self.ticks >= SHAKE_TICKS {
            self.shaking = false;
            self.ticks = 0;
        }

        let current = self.current_cached_entities(current_entities);
        let raider_nearby = current.iter().any(|entity| {
            eligible_raider(entity) && within_center(entity.position, HEARING_RADIUS)
        });
        let mut resonance_sound_call = false;
        if self.ticks >= RESONANCE_DELAY && self.resonance_ticks == 0 && raider_nearby {
            self.resonating = true;
            resonance_sound_call = true;
        }

        let mut glowing_entity_ids = Vec::new();
        let mut particles = Vec::new();
        if self.resonating {
            if self.resonance_ticks < RESONANCE_TICKS {
                self.resonance_ticks += 1;
            } else {
                match side {
                    BellSide::Server => {
                        glowing_entity_ids = current
                            .iter()
                            .filter(|entity| {
                                eligible_raider(entity)
                                    && within_center(entity.position, CACHE_RADIUS)
                            })
                            .map(|entity| entity.id)
                            .collect();
                    }
                    BellSide::Client => {
                        particles = bell_particles(&current);
                    }
                }
                self.resonating = false;
            }
        }
        BellTickOutcome {
            resonance_sound_call,
            audible_resonance_sound: resonance_sound_call && side == BellSide::Server,
            glowing_entity_ids,
            particles,
        }
    }

    fn current_cached_entities(&self, current: &[BellLiving]) -> Vec<BellLiving> {
        self.nearby_entities
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|cached| {
                current
                    .iter()
                    .find(|candidate| candidate.id == cached.id)
                    .copied()
                    .unwrap_or(*cached)
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellNeighbourOutcome {
    pub state: BellState,
    pub write_state: bool,
    pub update_flags: Option<u16>,
    pub ring: Option<BellRingOutcome>,
}

pub fn bell_neighbour_signal(
    runtime: &mut BellRuntime,
    state: BellState,
    has_signal: bool,
    matching_block_entity: bool,
) -> BellNeighbourOutcome {
    if state.powered == has_signal {
        return BellNeighbourOutcome {
            state,
            write_state: false,
            update_flags: None,
            ring: None,
        };
    }
    let ring = has_signal.then(|| {
        runtime.attempt_to_ring(BellSide::Server, matching_block_entity, state, None, false)
    });
    BellNeighbourOutcome {
        state: BellState {
            powered: has_signal,
            ..state
        },
        write_state: true,
        update_flags: Some(BLOCK_UPDATE_FLAGS),
        ring,
    }
}

pub fn bell_explosion(
    runtime: &mut BellRuntime,
    state: BellState,
    can_trigger_blocks: bool,
    matching_block_entity: bool,
) -> Option<BellRingOutcome> {
    can_trigger_blocks.then(|| {
        runtime.attempt_to_ring(BellSide::Server, matching_block_entity, state, None, false)
    })
}

pub fn bell_render_rotation(runtime: &BellRuntime, partial_ticks: f32) -> [f32; 2] {
    let Some(direction) = runtime.click_direction.filter(|_| runtime.shaking) else {
        return [0.0, 0.0];
    };
    let render_time = f32::from(runtime.ticks) + partial_ticks;
    let rotation = (render_time / std::f32::consts::PI).sin() / (4.0 + render_time / 3.0);
    match direction {
        Direction::North => [-rotation, 0.0],
        Direction::South => [rotation, 0.0],
        Direction::West => [0.0, -rotation],
        Direction::East => [0.0, rotation],
        Direction::Down | Direction::Up => [0.0, 0.0],
    }
}

pub const fn bell_loot_survives_explosion(survives_explosion: bool) -> bool {
    survives_explosion
}

fn bell_particles(entities: &[BellLiving]) -> Vec<BellParticle> {
    let nearby_count = entities
        .iter()
        .filter(|entity| within_center(entity.position, CACHE_RADIUS))
        .count() as i32;
    let count = ((nearby_count - 21) / -2).clamp(3, 15);
    let mut color = 16_700_985_u32;
    let mut particles = Vec::new();
    for entity in entities
        .iter()
        .filter(|entity| eligible_raider(entity) && within_center(entity.position, CACHE_RADIUS))
    {
        let x = entity.position[0];
        let z = entity.position[2];
        let distance = (x * x + z * z).sqrt();
        let particle_x = 0.5 + x / distance;
        let particle_z = 0.5 + z / distance;
        for _ in 0..count {
            color += 5;
            particles.push(BellParticle {
                entity_id: entity.id,
                color,
                position: [particle_x, 0.5, particle_z],
            });
        }
    }
    particles
}

const fn horizontal_or_north(direction: Direction) -> Direction {
    if direction.is_horizontal() {
        direction
    } else {
        Direction::North
    }
}

const fn direction_data_value(direction: Direction) -> u8 {
    match direction {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

fn direction_from_data_value(value: i32) -> Direction {
    match value.rem_euclid(6) {
        0 => Direction::Down,
        1 => Direction::Up,
        2 => Direction::North,
        3 => Direction::South,
        4 => Direction::West,
        _ => Direction::East,
    }
}

const fn eligible_raider(entity: &BellLiving) -> bool {
    entity.alive && !entity.removed && entity.raider
}

fn within_center(position: [f64; 3], radius: f64) -> bool {
    let dx = position[0] - 0.5;
    let dy = position[1] - 0.5;
    let dz = position[2] - 0.5;
    dx * dx + dy * dy + dz * dz < radius * radius
}
