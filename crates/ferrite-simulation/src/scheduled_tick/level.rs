//! Loaded-chunk registration, due-head merging, and execution snapshots.

use crate::scheduled_tick::container::ChunkTickContainer;
use crate::scheduled_tick::record::{SavedTick, ScheduledTick, TickPriority};
use ferrite_foundation::bounds::BlockBounds;
use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashSet, VecDeque};
use std::hash::Hash;

pub const SCHEDULED_TICK_CAP: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleOutcome {
    Queued,
    Duplicate,
    UnregisteredChunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelTickAdmission {
    pub runs_normally: bool,
    pub debug_level: bool,
}

impl LevelTickAdmission {
    pub const fn admits_scheduled_ticks(self) -> bool {
        self.runs_normally && !self.debug_level
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LevelDrainCounts {
    pub blocks: usize,
    pub fluids: usize,
}

#[derive(Debug)]
struct ContainerHead {
    chunk: ChunkPos,
    priority: TickPriority,
    sub_tick_order: i64,
}

impl ContainerHead {
    fn from_tick<I>(chunk: ChunkPos, tick: &ScheduledTick<I>) -> Self {
        Self {
            chunk,
            priority: tick.priority,
            sub_tick_order: tick.sub_tick_order,
        }
    }
}

impl PartialEq for ContainerHead {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
            && self.sub_tick_order == other.sub_tick_order
            && self.chunk == other.chunk
    }
}

impl Eq for ContainerHead {}

impl PartialOrd for ContainerHead {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContainerHead {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| other.sub_tick_order.cmp(&self.sub_tick_order))
            .then_with(|| other.chunk.cmp(&self.chunk))
    }
}

#[derive(Debug)]
pub struct ScheduledTickQueue<I> {
    containers: BTreeMap<ChunkPos, ChunkTickContainer<I>>,
    next_tick_by_chunk: BTreeMap<ChunkPos, i64>,
    to_run_this_tick: VecDeque<ScheduledTick<I>>,
    already_run_this_tick: Vec<ScheduledTick<I>>,
    to_run_this_tick_set: HashSet<(I, BlockPos)>,
}

impl<I> Default for ScheduledTickQueue<I>
where
    I: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<I> ScheduledTickQueue<I>
where
    I: Clone + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            containers: BTreeMap::new(),
            next_tick_by_chunk: BTreeMap::new(),
            to_run_this_tick: VecDeque::new(),
            already_run_this_tick: Vec::new(),
            to_run_this_tick_set: HashSet::new(),
        }
    }

    pub fn register_container(
        &mut self,
        chunk: ChunkPos,
        container: ChunkTickContainer<I>,
    ) -> Option<ChunkTickContainer<I>> {
        self.refresh_next_tick(chunk, &container);
        self.containers.insert(chunk, container)
    }

    pub fn unregister_container(&mut self, chunk: ChunkPos) -> Option<ChunkTickContainer<I>> {
        self.next_tick_by_chunk.remove(&chunk);
        self.containers.remove(&chunk)
    }

    pub fn unpack_container(&mut self, chunk: ChunkPos, current_tick: i64) -> bool {
        let Some(container) = self.containers.get_mut(&chunk) else {
            return false;
        };
        container.unpack(current_tick);
        let next_tick = container.peek().map(|tick| tick.trigger_tick);
        self.set_next_tick(chunk, next_tick);
        true
    }

    pub fn container(&self, chunk: ChunkPos) -> Option<&ChunkTickContainer<I>> {
        self.containers.get(&chunk)
    }

    pub fn registered_chunks(&self) -> impl ExactSizeIterator<Item = ChunkPos> + '_ {
        self.containers.keys().copied()
    }

    pub fn pack_container(&self, chunk: ChunkPos, current_tick: i64) -> Option<Vec<SavedTick<I>>> {
        self.containers
            .get(&chunk)
            .map(|container| container.pack(current_tick))
    }

    pub fn schedule(&mut self, tick: ScheduledTick<I>) -> ScheduleOutcome {
        let chunk = tick.position.chunk();
        let Some(container) = self.containers.get_mut(&chunk) else {
            return ScheduleOutcome::UnregisteredChunk;
        };
        if !container.schedule(tick) {
            return ScheduleOutcome::Duplicate;
        }
        let next_tick = container.peek().map(|head| head.trigger_tick);
        self.set_next_tick(chunk, next_tick);
        ScheduleOutcome::Queued
    }

    pub fn has_scheduled_tick(&self, position: BlockPos, type_identity: &I) -> bool {
        self.containers
            .get(&position.chunk())
            .is_some_and(|container| container.has_scheduled_tick(position, type_identity))
    }

    pub fn will_tick_this_tick(&mut self, position: BlockPos, type_identity: &I) -> bool {
        if self.to_run_this_tick_set.is_empty() && !self.to_run_this_tick.is_empty() {
            self.to_run_this_tick_set.extend(
                self.to_run_this_tick
                    .iter()
                    .map(|tick| (tick.type_identity.clone(), tick.position)),
            );
        }
        self.to_run_this_tick_set
            .contains(&(type_identity.clone(), position))
    }

    pub fn tick(
        &mut self,
        current_tick: i64,
        max_ticks: usize,
        mut in_block_ticking_range: impl FnMut(ChunkPos) -> bool,
        mut output: impl FnMut(&mut Self, ScheduledTick<I>),
    ) -> usize {
        self.collect_ticks(current_tick, max_ticks, &mut in_block_ticking_range);
        let collected = self.to_run_this_tick.len();
        while let Some(tick) = self.to_run_this_tick.pop_front() {
            if !self.to_run_this_tick_set.is_empty() {
                self.to_run_this_tick_set
                    .remove(&(tick.type_identity.clone(), tick.position));
            }
            self.already_run_this_tick.push(tick.clone());
            output(self, tick);
        }
        self.cleanup_after_tick();
        collected
    }

    pub fn tick_matching(
        &mut self,
        current_tick: i64,
        max_ticks: usize,
        in_block_ticking_range: impl FnMut(ChunkPos) -> bool,
        mut type_matches: impl FnMut(BlockPos, &I) -> bool,
        mut output: impl FnMut(&mut Self, ScheduledTick<I>),
    ) -> usize {
        self.tick(
            current_tick,
            max_ticks,
            in_block_ticking_range,
            |scheduler, tick| {
                if type_matches(tick.position, &tick.type_identity) {
                    output(scheduler, tick);
                }
            },
        )
    }

    pub fn clear_area(&mut self, area: BlockBounds) {
        let affected: Vec<_> = self
            .containers
            .keys()
            .copied()
            .filter(|chunk| chunk_intersects(*chunk, area))
            .collect();
        for chunk in affected {
            let container = self
                .containers
                .get_mut(&chunk)
                .expect("affected chunk came from the container map");
            container.remove_if(|tick| area.contains(tick.position));
            let next_tick = container.peek().map(|tick| tick.trigger_tick);
            self.set_next_tick(chunk, next_tick);
        }
        self.already_run_this_tick
            .retain(|tick| !area.contains(tick.position));
        self.to_run_this_tick
            .retain(|tick| !area.contains(tick.position));
    }

    pub fn copy_area(&mut self, area: BlockBounds, offset: BlockPos) {
        let ticks = self.ticks_in_area(area);
        self.schedule_copies(ticks, offset);
    }

    pub fn copy_area_from(&mut self, source: &Self, area: BlockBounds, offset: BlockPos) {
        self.schedule_copies(source.ticks_in_area(area), offset);
    }

    pub fn count(&self) -> usize {
        self.containers
            .values()
            .map(ChunkTickContainer::count)
            .sum()
    }

    fn collect_ticks(
        &mut self,
        current_tick: i64,
        max_ticks: usize,
        in_block_ticking_range: &mut impl FnMut(ChunkPos) -> bool,
    ) {
        let mut eligible = BinaryHeap::new();
        let due_chunks: Vec<_> = self
            .next_tick_by_chunk
            .iter()
            .filter_map(|(chunk, trigger)| (*trigger <= current_tick).then_some(*chunk))
            .collect();
        for chunk in due_chunks {
            let Some(container) = self.containers.get(&chunk) else {
                self.next_tick_by_chunk.remove(&chunk);
                continue;
            };
            let Some(head) = container.peek() else {
                self.next_tick_by_chunk.remove(&chunk);
                continue;
            };
            if head.trigger_tick > current_tick {
                self.next_tick_by_chunk.insert(chunk, head.trigger_tick);
            } else if in_block_ticking_range(chunk) {
                self.next_tick_by_chunk.remove(&chunk);
                eligible.push(ContainerHead::from_tick(chunk, head));
            }
        }
        self.drain_containers(current_tick, max_ticks, &mut eligible);
        while let Some(head) = eligible.pop() {
            self.update_next_from_container(head.chunk);
        }
    }

    fn drain_containers(
        &mut self,
        current_tick: i64,
        max_ticks: usize,
        eligible: &mut BinaryHeap<ContainerHead>,
    ) {
        while self.to_run_this_tick.len() < max_ticks {
            let Some(current) = eligible.pop() else {
                break;
            };
            let tick = self
                .containers
                .get_mut(&current.chunk)
                .and_then(ChunkTickContainer::poll)
                .expect("eligible container has a head");
            self.to_run_this_tick.push_back(tick);
            self.drain_current_container(current.chunk, current_tick, max_ticks, eligible);
            let Some(next) = self
                .containers
                .get(&current.chunk)
                .and_then(ChunkTickContainer::peek)
            else {
                continue;
            };
            if next.trigger_tick <= current_tick && self.to_run_this_tick.len() < max_ticks {
                eligible.push(ContainerHead::from_tick(current.chunk, next));
            } else {
                self.next_tick_by_chunk
                    .insert(current.chunk, next.trigger_tick);
            }
        }
    }

    fn drain_current_container(
        &mut self,
        chunk: ChunkPos,
        current_tick: i64,
        max_ticks: usize,
        eligible: &BinaryHeap<ContainerHead>,
    ) {
        while self.to_run_this_tick.len() < max_ticks {
            let Some(current) = self
                .containers
                .get(&chunk)
                .and_then(ChunkTickContainer::peek)
            else {
                break;
            };
            if current.trigger_tick > current_tick {
                break;
            }
            if let Some(other) = eligible.peek()
                && current
                    .priority
                    .cmp(&other.priority)
                    .then_with(|| current.sub_tick_order.cmp(&other.sub_tick_order))
                    == Ordering::Greater
            {
                break;
            }
            let tick = self
                .containers
                .get_mut(&chunk)
                .and_then(ChunkTickContainer::poll)
                .expect("current container head was just observed");
            self.to_run_this_tick.push_back(tick);
        }
    }

    fn ticks_in_area(&self, area: BlockBounds) -> Vec<ScheduledTick<I>> {
        let mut ticks = Vec::new();
        ticks.extend(
            self.already_run_this_tick
                .iter()
                .filter(|tick| area.contains(tick.position))
                .cloned(),
        );
        ticks.extend(
            self.to_run_this_tick
                .iter()
                .filter(|tick| area.contains(tick.position))
                .cloned(),
        );
        for (chunk, container) in &self.containers {
            if !chunk_intersects(*chunk, area) {
                continue;
            }
            ticks.extend(
                container
                    .all_ticks()
                    .into_iter()
                    .filter(|tick| area.contains(tick.position)),
            );
        }
        ticks
    }

    fn schedule_copies(&mut self, ticks: Vec<ScheduledTick<I>>, offset: BlockPos) {
        let Some(minimum) = ticks.iter().map(|tick| tick.sub_tick_order).min() else {
            return;
        };
        let maximum = ticks
            .iter()
            .map(|tick| tick.sub_tick_order)
            .max()
            .expect("nonempty tick list has a maximum");
        for tick in ticks {
            let copied = ScheduledTick::new(
                tick.type_identity,
                wrapping_offset(tick.position, offset),
                tick.trigger_tick,
                tick.priority,
                tick.sub_tick_order
                    .wrapping_sub(minimum)
                    .wrapping_add(maximum)
                    .wrapping_add(1),
            );
            self.schedule(copied);
        }
    }

    fn cleanup_after_tick(&mut self) {
        self.to_run_this_tick.clear();
        self.already_run_this_tick.clear();
        self.to_run_this_tick_set.clear();
    }

    fn update_next_from_container(&mut self, chunk: ChunkPos) {
        let next_tick = self
            .containers
            .get(&chunk)
            .and_then(ChunkTickContainer::peek)
            .map(|tick| tick.trigger_tick);
        self.set_next_tick(chunk, next_tick);
    }

    fn refresh_next_tick(&mut self, chunk: ChunkPos, container: &ChunkTickContainer<I>) {
        self.set_next_tick(chunk, container.peek().map(|tick| tick.trigger_tick));
    }

    fn set_next_tick(&mut self, chunk: ChunkPos, next_tick: Option<i64>) {
        if let Some(trigger_tick) = next_tick {
            self.next_tick_by_chunk.insert(chunk, trigger_tick);
        } else {
            self.next_tick_by_chunk.remove(&chunk);
        }
    }
}

#[derive(Debug)]
pub struct LevelScheduledTicks<B, F> {
    pub blocks: ScheduledTickQueue<B>,
    pub fluids: ScheduledTickQueue<F>,
}

impl<B, F> Default for LevelScheduledTicks<B, F>
where
    B: Clone + Eq + Hash,
    F: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self {
            blocks: ScheduledTickQueue::new(),
            fluids: ScheduledTickQueue::new(),
        }
    }
}

impl<B, F> LevelScheduledTicks<B, F>
where
    B: Clone + Eq + Hash,
    F: Clone + Eq + Hash,
{
    pub fn tick(
        &mut self,
        admission: LevelTickAdmission,
        game_time: i64,
        in_block_ticking_range: &mut impl FnMut(ChunkPos) -> bool,
        block_output: &mut impl FnMut(&mut ScheduledTickQueue<B>, ScheduledTick<B>),
        fluid_output: &mut impl FnMut(&mut ScheduledTickQueue<F>, ScheduledTick<F>),
    ) -> LevelDrainCounts {
        if !admission.admits_scheduled_ticks() {
            return LevelDrainCounts::default();
        }
        LevelDrainCounts {
            blocks: self.blocks.tick(
                game_time,
                SCHEDULED_TICK_CAP,
                &mut *in_block_ticking_range,
                block_output,
            ),
            fluids: self.fluids.tick(
                game_time,
                SCHEDULED_TICK_CAP,
                in_block_ticking_range,
                fluid_output,
            ),
        }
    }
}

fn chunk_intersects(chunk: ChunkPos, area: BlockBounds) -> bool {
    chunk.x >= area.minimum().chunk().x
        && chunk.x <= area.maximum().chunk().x
        && chunk.z >= area.minimum().chunk().z
        && chunk.z <= area.maximum().chunk().z
}

const fn wrapping_offset(position: BlockPos, offset: BlockPos) -> BlockPos {
    BlockPos::new(
        position.x.wrapping_add(offset.x),
        position.y.wrapping_add(offset.y),
        position.z.wrapping_add(offset.z),
    )
}
