//! Random-work admission, chunk/section traversal, and callback ordering.

use crate::random_tick::position::RandomPositionStream;
use crate::random_tick::tracker::{SimulationChunkTracker, is_entity_ticking};
use ferrite_foundation::coordinate::{BlockPos, ChunkPos};

pub const DEFAULT_RANDOM_TICK_SPEED: i32 = 3;
pub const MINIMUM_RANDOM_TICK_SPEED: i32 = 0;
pub const MAXIMUM_RANDOM_TICK_SPEED: i32 = i32::MAX;
pub const PRECIPITATION_ATTEMPT_BOUND: i32 = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCacheTickPlan {
    pub purge_stale_tickets: bool,
    pub run_distance_updates: bool,
    pub update_inhabited_time: bool,
    pub run_random_chunk_work: bool,
}

pub const fn chunk_cache_tick_plan(runs_normally: bool, debug_level: bool) -> ChunkCacheTickPlan {
    ChunkCacheTickPlan {
        purge_stale_tickets: runs_normally,
        run_distance_updates: true,
        update_inhabited_time: true,
        run_random_chunk_work: runs_normally && !debug_level,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayChunkStage {
    ConstructNaturalSpawnState,
    ReadSpawnAndRandomTickRules,
    ShuffleSpawningChunks,
    NaturalSpawning,
    Thunder,
    PrecipitationAndRandomTicks,
}

pub const GAMEPLAY_CHUNK_STAGES: [GameplayChunkStage; 6] = [
    GameplayChunkStage::ConstructNaturalSpawnState,
    GameplayChunkStage::ReadSpawnAndRandomTickRules,
    GameplayChunkStage::ShuffleSpawningChunks,
    GameplayChunkStage::NaturalSpawning,
    GameplayChunkStage::Thunder,
    GameplayChunkStage::PrecipitationAndRandomTicks,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolderAccess {
    MissingVisibleHolder,
    MissingTickingChunk,
    TickingChunk,
}

pub fn random_tick_chunk_order(
    tracker: &SimulationChunkTracker,
    mut holder_access: impl FnMut(ChunkPos) -> HolderAccess,
) -> Vec<ChunkPos> {
    tracker
        .compatibility_order()
        .into_iter()
        .filter_map(|(chunk, level)| {
            (is_entity_ticking(level) && holder_access(chunk) == HolderAccess::TickingChunk)
                .then_some(chunk)
        })
        .collect()
}

pub trait BoundedRandom {
    fn next_i32(&mut self, upper_exclusive: i32) -> i32;
}

pub trait RandomTickState: Clone {
    fn block_is_randomly_ticking(&self) -> bool;
    fn fluid_is_randomly_ticking(&self) -> bool;
}

pub trait RandomTickChunk {
    type State: RandomTickState;

    fn chunk_position(&self) -> ChunkPos;
    fn section_count(&self) -> usize;
    fn section_bottom_y(&self, section_index: usize) -> i32;
    fn section_is_randomly_ticking(&self, section_index: usize) -> bool;
    fn state_at(&self, section_index: usize, local_x: u8, local_y: u8, local_z: u8) -> Self::State;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RandomTickReport {
    pub precipitation_draws: u64,
    pub precipitation_callbacks: u64,
    pub admitted_sections: u64,
    pub position_samples: u64,
    pub block_callbacks: u64,
    pub fluid_callbacks: u64,
}

pub fn tick_chunk<C, R>(
    chunk: &mut C,
    speed: i32,
    position_stream: &mut RandomPositionStream,
    gameplay_random: &mut R,
    mut precipitation: impl FnMut(&mut C, BlockPos, &mut R),
    mut block_callback: impl FnMut(&mut C, BlockPos, &C::State, &mut R),
    mut fluid_callback: impl FnMut(&mut C, BlockPos, &C::State, &mut R),
) -> RandomTickReport
where
    C: RandomTickChunk,
    R: BoundedRandom,
{
    let chunk_position = chunk.chunk_position();
    let minimum_x = chunk_position.x.wrapping_mul(16);
    let minimum_z = chunk_position.z.wrapping_mul(16);
    let mut report = RandomTickReport::default();

    for _ in 0..speed {
        report.precipitation_draws += 1;
        if gameplay_random.next_i32(PRECIPITATION_ATTEMPT_BOUND) != 0 {
            continue;
        }
        let position = position_stream.next(BlockPos::new(minimum_x, 0, minimum_z), 15);
        report.precipitation_callbacks += 1;
        report.position_samples += 1;
        precipitation(chunk, position, gameplay_random);
    }

    if speed <= 0 {
        return report;
    }
    for section_index in 0..chunk.section_count() {
        if !chunk.section_is_randomly_ticking(section_index) {
            continue;
        }
        report.admitted_sections += 1;
        let minimum_y = chunk.section_bottom_y(section_index);
        for _ in 0..speed {
            let position = position_stream.next(BlockPos::new(minimum_x, minimum_y, minimum_z), 15);
            report.position_samples += 1;
            let state = chunk.state_at(
                section_index,
                position.x.wrapping_sub(minimum_x) as u8,
                position.y.wrapping_sub(minimum_y) as u8,
                position.z.wrapping_sub(minimum_z) as u8,
            );
            if state.block_is_randomly_ticking() {
                report.block_callbacks += 1;
                block_callback(chunk, position, &state, gameplay_random);
            }
            if state.fluid_is_randomly_ticking() {
                report.fluid_callbacks += 1;
                fluid_callback(chunk, position, &state, gameplay_random);
            }
        }
    }
    report
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SectionRandomTickCounts {
    block: i16,
    fluid: i16,
}

impl SectionRandomTickCounts {
    pub const fn from_raw(block: i16, fluid: i16) -> Self {
        Self { block, fluid }
    }

    pub const fn block(self) -> i16 {
        self.block
    }

    pub const fn fluid(self) -> i16 {
        self.fluid
    }

    pub const fn is_randomly_ticking(self) -> bool {
        self.block > 0 || self.fluid > 0
    }

    pub fn replace(
        &mut self,
        previous_block: bool,
        previous_fluid: bool,
        next_block: bool,
        next_fluid: bool,
    ) {
        self.block = adjust_count(self.block, previous_block, next_block);
        self.fluid = adjust_count(self.fluid, previous_fluid, next_fluid);
    }

    pub fn recalculate<S: RandomTickState>(&mut self, states: impl IntoIterator<Item = S>) {
        let mut block = 0_i32;
        let mut fluid = 0_i32;
        for state in states {
            if state.block_is_randomly_ticking() {
                block += 1;
            }
            if state.fluid_is_randomly_ticking() {
                fluid += 1;
            }
        }
        self.block = block as i16;
        self.fluid = fluid as i16;
    }
}

pub const fn planned_attempts(speed: i32, eligible_sections: usize) -> (u64, u64) {
    if speed <= 0 {
        return (0, 0);
    }
    (
        speed as u64,
        (speed as u64).saturating_mul(eligible_sections as u64),
    )
}

const fn adjust_count(current: i16, previous: bool, next: bool) -> i16 {
    match (previous, next) {
        (true, false) => current.wrapping_sub(1),
        (false, true) => current.wrapping_add(1),
        _ => current,
    }
}
