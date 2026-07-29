//! Scheduled fluid recomputation, spread, mixing, fire, and evaporation decisions.

use ferrite_foundation::direction::Direction;

pub const FLUID_PROTOCOL_IDS: [u8; 5] = [0, 1, 2, 3, 4];
pub const WATER_BLOCK_ID: u16 = 35;
pub const LAVA_BLOCK_ID: u16 = 36;
pub const WATER_FIRST_STATE_ID: u32 = 86;
pub const LAVA_FIRST_STATE_ID: u32 = 102;
pub const OBSIDIAN_STATE_ID: u32 = 3_369;
pub const COBBLESTONE_STATE_ID: u32 = 14;
pub const STONE_STATE_ID: u32 = 1;
pub const BASE_FIRE_STATE_ID: u32 = 3_406;
pub const BLOCK_UPDATE_FLAGS: u16 = 3;
pub const LIQUID_MIX_EVENT: u16 = 1_501;
pub const WATER_TICK_DELAY: u16 = 5;
pub const BUBBLE_COLUMN_CHECK_DELAY: u16 = 20;
pub const SCHEDULED_TICK_CAP: usize = 65_536;
pub const SIMPLE_WATERLOGGED_BLOCK_COUNT: usize = 429;
pub const WATER_SOURCE_CONVERSION_DEFAULT: bool = true;
pub const LAVA_SOURCE_CONVERSION_DEFAULT: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelTickQueue {
    Block,
    Fluid,
}

pub const LEVEL_TICK_ORDER: [LevelTickQueue; 2] = [LevelTickQueue::Block, LevelTickQueue::Fluid];

pub fn explicitly_unholdable(path: &str) -> bool {
    path.ends_with("_door")
        || path.ends_with("_sign")
        || matches!(
            path,
            "ladder"
                | "sugar_cane"
                | "bubble_column"
                | "nether_portal"
                | "end_portal"
                | "end_gateway"
                | "structure_void"
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidFamily {
    Water,
    Lava,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidState {
    Empty,
    Source(FluidFamily),
    Flowing {
        family: FluidFamily,
        amount: u8,
        falling: bool,
    },
}

impl FluidState {
    pub const fn family(self) -> Option<FluidFamily> {
        match self {
            Self::Empty => None,
            Self::Source(family) | Self::Flowing { family, .. } => Some(family),
        }
    }

    pub const fn amount(self) -> u8 {
        match self {
            Self::Empty => 0,
            Self::Source(_) => 8,
            Self::Flowing { amount, .. } => amount,
        }
    }

    pub const fn is_source(self) -> bool {
        matches!(self, Self::Source(_))
    }

    pub const fn falling(self) -> bool {
        matches!(self, Self::Flowing { falling: true, .. })
    }

    pub const fn protocol_id(self) -> u8 {
        match self {
            Self::Empty => 0,
            Self::Flowing {
                family: FluidFamily::Water,
                ..
            } => 1,
            Self::Source(FluidFamily::Water) => 2,
            Self::Flowing {
                family: FluidFamily::Lava,
                ..
            } => 3,
            Self::Source(FluidFamily::Lava) => 4,
        }
    }

    pub fn legacy_level(self) -> Option<u8> {
        match self {
            Self::Empty => None,
            Self::Source(_) => Some(0),
            Self::Flowing {
                amount, falling, ..
            } => Some(8 - amount.min(8) + if falling { 8 } else { 0 }),
        }
    }

    pub fn block_state_id(self) -> Option<u32> {
        let level = u32::from(self.legacy_level()?);
        match self.family() {
            Some(FluidFamily::Water) => Some(WATER_FIRST_STATE_ID + level),
            Some(FluidFamily::Lava) => Some(LAVA_FIRST_STATE_ID + level),
            None => None,
        }
    }

    pub fn own_height(self, same_family_above: bool) -> f32 {
        if same_family_above {
            1.0
        } else {
            f32::from(self.amount()) / 9.0
        }
    }
}

pub fn fluid_from_legacy(family: FluidFamily, level: u8) -> FluidState {
    match level.min(15) {
        0 => FluidState::Source(family),
        1..=7 => FluidState::Flowing {
            family,
            amount: 8 - level,
            falling: false,
        },
        _ => FluidState::Flowing {
            family,
            amount: 8,
            falling: true,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidParameters {
    pub drop_off: u8,
    pub slope_range: u8,
    pub tick_delay: u16,
}

pub const fn fluid_parameters(family: FluidFamily, fast_lava: bool) -> FluidParameters {
    match (family, fast_lava) {
        (FluidFamily::Water, _) => FluidParameters {
            drop_off: 1,
            slope_range: 4,
            tick_delay: 5,
        },
        (FluidFamily::Lava, false) => FluidParameters {
            drop_off: 2,
            slope_range: 2,
            tick_delay: 30,
        },
        (FluidFamily::Lava, true) => FluidParameters {
            drop_off: 1,
            slope_range: 4,
            tick_delay: 10,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpreadDelay {
    pub delay: u16,
    pub slowdown_draw_consumed: bool,
}

pub fn spread_delay(
    old: FluidState,
    new: FluidState,
    fast_lava: bool,
    slowdown_draw: u32,
) -> SpreadDelay {
    let base = match old.family() {
        Some(family) => fluid_parameters(family, fast_lava).tick_delay,
        None => 0,
    };
    let lava_slowdown = matches!(old.family(), Some(FluidFamily::Lava))
        && !matches!(old, FluidState::Empty)
        && !matches!(new, FluidState::Empty)
        && !old.falling()
        && !new.falling()
        && new.own_height(false) > old.own_height(false);
    SpreadDelay {
        delay: if lava_slowdown && slowdown_draw != 0 {
            base * 4
        } else {
            base
        },
        slowdown_draw_consumed: lava_slowdown,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorizontalFluid {
    pub state: FluidState,
    pub face_passes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalFluidInputs {
    pub family: FluidFamily,
    pub horizontal: [HorizontalFluid; 4],
    pub same_family_above_admitted: bool,
    pub below_solid: bool,
    pub below_same_family_source: bool,
    pub source_conversion: bool,
    pub drop_off: u8,
}

pub fn recompute_local_fluid(inputs: LocalFluidInputs) -> FluidState {
    let mut maximum = 0;
    let mut sources = 0;
    for neighbour in inputs.horizontal {
        if !neighbour.face_passes || neighbour.state.family() != Some(inputs.family) {
            continue;
        }
        maximum = maximum.max(neighbour.state.amount());
        sources += usize::from(neighbour.state.is_source());
    }
    if sources >= 2
        && inputs.source_conversion
        && (inputs.below_solid || inputs.below_same_family_source)
    {
        return FluidState::Source(inputs.family);
    }
    if inputs.same_family_above_admitted {
        return FluidState::Flowing {
            family: inputs.family,
            amount: 8,
            falling: true,
        };
    }
    let amount = maximum.saturating_sub(inputs.drop_off);
    if amount == 0 {
        FluidState::Empty
    } else {
        FluidState::Flowing {
            family: inputs.family,
            amount,
            falling: false,
        }
    }
}

pub fn fluid_can_be_replaced(
    current: FluidState,
    incoming: FluidFamily,
    direction: Direction,
) -> bool {
    match current {
        FluidState::Empty => true,
        FluidState::Source(FluidFamily::Water)
        | FluidState::Flowing {
            family: FluidFamily::Water,
            ..
        } => direction == Direction::Down && incoming != FluidFamily::Water,
        FluidState::Source(FluidFamily::Lava)
        | FluidState::Flowing {
            family: FluidFamily::Lava,
            ..
        } => incoming == FluidFamily::Water && current.amount() >= 4,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    None,
    SimpleWaterlogged { waterlogged: bool },
    IntrinsicAquatic,
    Other { accepts: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidDestination {
    pub current_fluid: FluidState,
    pub generally_holdable: bool,
    pub joined_face_passes: bool,
    pub container: ContainerKind,
}

pub fn destination_admits(
    destination: FluidDestination,
    incoming: FluidState,
    direction: Direction,
) -> bool {
    let Some(family) = incoming.family() else {
        return false;
    };
    if destination.current_fluid == FluidState::Source(family)
        || !destination.generally_holdable
        || !destination.joined_face_passes
        || !fluid_can_be_replaced(destination.current_fluid, family, direction)
    {
        return false;
    }
    match destination.container {
        ContainerKind::None => true,
        ContainerKind::SimpleWaterlogged { waterlogged } => {
            !waterlogged && incoming == FluidState::Source(FluidFamily::Water)
        }
        ContainerKind::IntrinsicAquatic => false,
        ContainerKind::Other { accepts } => accepts,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerPlacement {
    pub accepted: bool,
    pub write_waterlogged: bool,
    pub write_flags: Option<u16>,
    pub schedule_delay: Option<u16>,
}

pub fn place_simple_waterlogged(
    side_is_client: bool,
    already_waterlogged: bool,
    incoming: FluidState,
) -> ContainerPlacement {
    let accepted = !already_waterlogged && incoming == FluidState::Source(FluidFamily::Water);
    ContainerPlacement {
        accepted,
        write_waterlogged: accepted && !side_is_client,
        write_flags: if accepted && !side_is_client {
            Some(BLOCK_UPDATE_FLAGS)
        } else {
            None
        },
        schedule_delay: if accepted && !side_is_client {
            Some(WATER_TICK_DELAY)
        } else {
            None
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpreadCandidate {
    pub direction: Direction,
    pub admitted: bool,
    pub hole_distance: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidSpreadRequest {
    pub origin: FluidState,
    pub drop_off: u8,
    pub downward_admitted: bool,
    pub downward_state: FluidState,
    pub below_is_open_hole: bool,
    pub horizontal_source_neighbours: u8,
    pub candidates: [SpreadCandidate; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidSpreadPlan {
    pub downward: Option<FluidState>,
    pub horizontal_state: Option<FluidState>,
    pub horizontal_directions: Vec<Direction>,
}

pub fn fluid_spread_plan(request: FluidSpreadRequest) -> FluidSpreadPlan {
    if request.downward_admitted {
        let side = (request.horizontal_source_neighbours >= 3)
            .then(|| side_spread(request.origin, request.drop_off, request.candidates))
            .flatten();
        return FluidSpreadPlan {
            downward: Some(request.downward_state),
            horizontal_state: side.as_ref().map(|selection| selection.0),
            horizontal_directions: side.map(|selection| selection.1).unwrap_or_default(),
        };
    }
    if !request.origin.is_source() && request.below_is_open_hole {
        return FluidSpreadPlan {
            downward: None,
            horizontal_state: None,
            horizontal_directions: Vec::new(),
        };
    }
    let side = side_spread(request.origin, request.drop_off, request.candidates);
    FluidSpreadPlan {
        downward: None,
        horizontal_state: side.as_ref().map(|selection| selection.0),
        horizontal_directions: side.map(|selection| selection.1).unwrap_or_default(),
    }
}

fn side_spread(
    origin: FluidState,
    drop_off: u8,
    candidates: [SpreadCandidate; 4],
) -> Option<(FluidState, Vec<Direction>)> {
    let family = origin.family()?;
    let amount = if origin.falling() {
        7
    } else {
        origin.amount().saturating_sub(drop_off)
    };
    if amount == 0 {
        return None;
    }
    let minimum = candidates
        .iter()
        .filter(|candidate| candidate.admitted)
        .filter_map(|candidate| candidate.hole_distance)
        .min()?;
    let mut directions: Vec<_> = candidates
        .into_iter()
        .filter(|candidate| candidate.admitted && candidate.hole_distance == Some(minimum))
        .map(|candidate| candidate.direction)
        .collect();
    directions.sort_by_key(|direction| horizontal_commit_rank(*direction));
    Some((
        FluidState::Flowing {
            family,
            amount,
            falling: false,
        },
        directions,
    ))
}

const fn horizontal_commit_rank(direction: Direction) -> u8 {
    match direction {
        Direction::North => 0,
        Direction::South => 1,
        Direction::West => 2,
        Direction::East => 3,
        Direction::Down | Direction::Up => 4,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidTickWrite {
    None,
    Air {
        flags: u16,
    },
    Fluid {
        state: FluidState,
        flags: u16,
        schedule_delay: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidTickPlan {
    pub write: FluidTickWrite,
    pub spread_state: FluidState,
}

pub fn fluid_tick_plan(old: FluidState, recomputed: FluidState, delay: u16) -> FluidTickPlan {
    if old.is_source() {
        return FluidTickPlan {
            write: FluidTickWrite::None,
            spread_state: old,
        };
    }
    let write = if recomputed == old {
        FluidTickWrite::None
    } else if matches!(recomputed, FluidState::Empty) {
        FluidTickWrite::Air {
            flags: BLOCK_UPDATE_FLAGS,
        }
    } else {
        FluidTickWrite::Fluid {
            state: recomputed,
            flags: BLOCK_UPDATE_FLAGS,
            schedule_delay: delay,
        }
    };
    FluidTickPlan {
        write,
        spread_state: recomputed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LavaMixNeighbour {
    pub water_tagged: bool,
    pub blue_ice: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LavaMixProduct {
    Obsidian,
    Cobblestone,
    Basalt,
}

impl LavaMixProduct {
    pub const fn state_id(self) -> u32 {
        match self {
            Self::Obsidian => OBSIDIAN_STATE_ID,
            Self::Cobblestone => COBBLESTONE_STATE_ID,
            Self::Basalt => 7_001,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LavaMix {
    pub product: LavaMixProduct,
    pub neighbour: Direction,
    pub level_event: u16,
    pub abort_schedule: bool,
}

pub fn lava_mix_before_schedule(
    lava_is_source: bool,
    soul_soil_below: bool,
    neighbours: [LavaMixNeighbour; 5],
) -> Option<LavaMix> {
    let directions = [
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];
    for (direction, neighbour) in directions.into_iter().zip(neighbours) {
        let product = if neighbour.water_tagged {
            Some(if lava_is_source {
                LavaMixProduct::Obsidian
            } else {
                LavaMixProduct::Cobblestone
            })
        } else if soul_soil_below && neighbour.blue_ice {
            Some(LavaMixProduct::Basalt)
        } else {
            None
        };
        if let Some(product) = product {
            return Some(LavaMix {
                product,
                neighbour: direction,
                level_event: LIQUID_MIX_EVENT,
                abort_schedule: true,
            });
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownwardLavaWater {
    pub write_stone: bool,
    pub stone_state_id: Option<u32>,
    pub fizz_event: u16,
    pub generic_placement: bool,
}

pub const fn downward_lava_into_water(target_is_liquid_block: bool) -> DownwardLavaWater {
    DownwardLavaWater {
        write_stone: target_is_liquid_block,
        stone_state_id: if target_is_liquid_block {
            Some(STONE_STATE_ID)
        } else {
            None
        },
        fizz_event: LIQUID_MIX_EVENT,
        generic_placement: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpreadCommitPlan {
    pub drop_target_resources: bool,
    pub fizz_before_write: bool,
    pub call_container_place: bool,
    pub ignore_container_result: bool,
    pub write_legacy_liquid: bool,
    pub write_flags: Option<u16>,
}

pub const fn spread_commit_plan(
    incoming: FluidFamily,
    container: bool,
    target_nonair: bool,
) -> SpreadCommitPlan {
    if container {
        return SpreadCommitPlan {
            drop_target_resources: false,
            fizz_before_write: false,
            call_container_place: true,
            ignore_container_result: true,
            write_legacy_liquid: false,
            write_flags: None,
        };
    }
    SpreadCommitPlan {
        drop_target_resources: target_nonair && matches!(incoming, FluidFamily::Water),
        fizz_before_write: matches!(incoming, FluidFamily::Lava),
        call_container_place: false,
        ignore_container_result: false,
        write_legacy_liquid: true,
        write_flags: Some(BLOCK_UPDATE_FLAGS),
    }
}

pub const fn shape_update_schedules(current_source: bool, changed_neighbour_source: bool) -> bool {
    current_source || changed_neighbour_source
}

pub trait FluidRandom {
    fn next_int(&mut self, bound: u32) -> u32;
    fn next_unbounded_int(&mut self) -> i32;
    fn next_float(&mut self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RisingFireProbe {
    pub loaded: bool,
    pub air: bool,
    pub ignited_neighbour: bool,
    pub motion_blocking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorizontalFireProbe {
    pub loaded: bool,
    pub base_ignited: bool,
    pub above_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LavaFirePlan {
    pub sampled_offsets: Vec<[i8; 2]>,
    pub fire_probe_indices: Vec<usize>,
    pub aborted_unloaded: bool,
}

pub fn lava_random_fire<R: FluidRandom>(
    can_spread_fire: bool,
    rising: &[RisingFireProbe],
    horizontal: &[HorizontalFireProbe; 3],
    random: &mut R,
) -> LavaFirePlan {
    if !can_spread_fire {
        return LavaFirePlan {
            sampled_offsets: Vec::new(),
            fire_probe_indices: Vec::new(),
            aborted_unloaded: false,
        };
    }
    let steps = random.next_int(3) as usize;
    let mut plan = LavaFirePlan {
        sampled_offsets: Vec::new(),
        fire_probe_indices: Vec::new(),
        aborted_unloaded: false,
    };
    if steps > 0 {
        for index in 0..steps {
            let offset = [random.next_int(3) as i8 - 1, random.next_int(3) as i8 - 1];
            plan.sampled_offsets.push(offset);
            let Some(probe) = rising.get(index) else {
                plan.aborted_unloaded = true;
                return plan;
            };
            if !probe.loaded {
                plan.aborted_unloaded = true;
                return plan;
            }
            if probe.air && probe.ignited_neighbour {
                plan.fire_probe_indices.push(index);
                return plan;
            }
            if !probe.air && probe.motion_blocking {
                return plan;
            }
        }
        return plan;
    }
    for (index, probe) in horizontal.iter().enumerate() {
        let offset = [random.next_int(3) as i8 - 1, random.next_int(3) as i8 - 1];
        plan.sampled_offsets.push(offset);
        if !probe.loaded {
            plan.aborted_unloaded = true;
            return plan;
        }
        if probe.base_ignited && probe.above_empty {
            plan.fire_probe_indices.push(index);
        }
    }
    plan
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaterEvaporation {
    pub success: bool,
    pub evaporated: bool,
    pub write_fluid: bool,
    pub ordinary_sound: bool,
    pub fluid_place_event: bool,
    pub extinguish_pitch: Option<f32>,
    pub smoke_samples: Vec<[f32; 3]>,
}

pub fn water_evaporation<R: FluidRandom>(
    placement_admitted: bool,
    water_tagged: bool,
    water_evaporates: bool,
    random: &mut R,
) -> WaterEvaporation {
    if !placement_admitted {
        return WaterEvaporation {
            success: false,
            evaporated: false,
            write_fluid: false,
            ordinary_sound: false,
            fluid_place_event: false,
            extinguish_pitch: None,
            smoke_samples: Vec::new(),
        };
    }
    if !water_tagged || !water_evaporates {
        return WaterEvaporation {
            success: true,
            evaporated: false,
            write_fluid: true,
            ordinary_sound: true,
            fluid_place_event: true,
            extinguish_pitch: None,
            smoke_samples: Vec::new(),
        };
    }
    let pitch = 2.6 + (random.next_float() - random.next_float()) * 0.8;
    let smoke_samples = (0..8)
        .map(|_| {
            [
                random.next_float(),
                random.next_float(),
                random.next_float(),
            ]
        })
        .collect();
    WaterEvaporation {
        success: true,
        evaporated: true,
        write_fluid: false,
        ordinary_sound: false,
        fluid_place_event: false,
        extinguish_pitch: Some(pitch),
        smoke_samples,
    }
}
