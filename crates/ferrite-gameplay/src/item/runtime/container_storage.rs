//! Randomizable storage, open counters, barrels, chests, and player Ender storage.

use crate::item::runtime::inventory::{Inventory, Slot};
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::direction::Direction;

pub const CONTAINER_MAXIMUM: i32 = 99;
pub const OPENER_RECOUNT_DELAY: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingLoot {
    pub table_fingerprint: u64,
    pub seed: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootCaller {
    Player {
        player_fingerprint: u64,
        luck_bits: u32,
    },
    NullPlayer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomizableStorage<const N: usize> {
    pub inventory: Inventory,
    pub pending_loot: Option<PendingLoot>,
    pub materialized_by: Option<LootCaller>,
    pub custom_name: Option<String>,
    pub lock_fingerprint: Option<u64>,
}

impl<const N: usize> RandomizableStorage<N> {
    pub fn empty() -> Self {
        Self {
            inventory: Inventory::empty(N),
            pending_loot: None,
            materialized_by: None,
            custom_name: None,
            lock_fingerprint: None,
        }
    }

    pub fn public_access(&mut self, caller: LootCaller) {
        if self.pending_loot.take().is_some() {
            self.materialized_by = Some(caller);
        }
    }

    pub fn player_open(&mut self, player_fingerprint: u64, luck: f32) {
        self.public_access(LootCaller::Player {
            player_fingerprint,
            luck_bits: luck.to_bits(),
        });
    }

    pub fn get(&mut self, slot: usize) -> Option<&ItemStack> {
        self.public_access(LootCaller::NullPlayer);
        self.inventory.slots.get(slot).map(|slot| &slot.stack)
    }

    pub fn set(&mut self, slot: usize, stack: ItemStack) -> bool {
        self.public_access(LootCaller::NullPlayer);
        let Some(target) = self.inventory.slots.get_mut(slot) else {
            return false;
        };
        let mut stack = stack;
        stack.count = stack.count.min(CONTAINER_MAXIMUM).min(stack.maximum);
        target.stack = stack.normalized();
        self.inventory.changed_calls += 1;
        true
    }

    pub fn clear_raw(&mut self) {
        self.inventory.slots = (0..N).map(|_| Slot::empty()).collect();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenUser {
    pub spectator: bool,
    pub interaction_range: f64,
    pub reports_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OpenEffects {
    pub opened_boundary: bool,
    pub closed_boundary: bool,
    pub block_event_count: Option<i32>,
    pub schedule_recount: bool,
    pub sourced_game_event: bool,
    pub source_less_game_event: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenerCounter {
    pub count: i32,
    pub maximum_interaction_range: f64,
}

impl OpenerCounter {
    pub const fn new() -> Self {
        Self {
            count: 0,
            maximum_interaction_range: 0.0,
        }
    }

    pub fn start_open(
        &mut self,
        removed: bool,
        user: OpenUser,
        sends_block_event: bool,
    ) -> OpenEffects {
        if removed || user.spectator {
            return OpenEffects::default();
        }
        let old = self.count;
        self.count = self.count.saturating_add(1);
        self.maximum_interaction_range = self.maximum_interaction_range.max(user.interaction_range);
        OpenEffects {
            opened_boundary: old == 0,
            block_event_count: sends_block_event.then_some(self.count),
            schedule_recount: old == 0,
            sourced_game_event: old == 0,
            ..OpenEffects::default()
        }
    }

    pub fn stop_open(
        &mut self,
        removed: bool,
        user: OpenUser,
        sends_block_event: bool,
    ) -> OpenEffects {
        if removed || user.spectator {
            return OpenEffects::default();
        }
        let old = self.count;
        self.count = self.count.saturating_sub(1);
        if self.count == 0 {
            self.maximum_interaction_range = 0.0;
        }
        OpenEffects {
            closed_boundary: old == 1,
            block_event_count: sends_block_event.then_some(self.count),
            sourced_game_event: old == 1,
            ..OpenEffects::default()
        }
    }

    pub fn recount(&mut self, users: &[OpenUser], sends_block_event: bool) -> OpenEffects {
        let old = self.count;
        self.maximum_interaction_range = 0.0;
        self.count = 0;
        for user in users
            .iter()
            .filter(|user| !user.spectator && user.reports_open)
        {
            self.count += 1;
            self.maximum_interaction_range =
                self.maximum_interaction_range.max(user.interaction_range);
        }
        OpenEffects {
            opened_boundary: old == 0 && self.count > 0,
            closed_boundary: old > 0 && self.count == 0,
            block_event_count: sends_block_event.then_some(self.count),
            schedule_recount: self.count > 0,
            source_less_game_event: (old == 0) != (self.count == 0),
            ..OpenEffects::default()
        }
    }

    pub fn still_valid(&self, distance_squared: f64, interaction_range: f64) -> bool {
        distance_squared < (interaction_range + 4.0).powi(2)
    }
}

impl Default for OpenerCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Barrel {
    pub facing: Direction,
    pub open_state: bool,
    pub storage: RandomizableStorage<27>,
    pub openers: OpenerCounter,
}

impl Barrel {
    pub fn empty(facing: Direction) -> Self {
        Self {
            facing,
            open_state: false,
            storage: RandomizableStorage::empty(),
            openers: OpenerCounter::new(),
        }
    }

    pub fn start_open(&mut self, removed: bool, user: OpenUser) -> OpenEffects {
        let effects = self.openers.start_open(removed, user, false);
        if effects.opened_boundary {
            self.open_state = true;
        }
        effects
    }

    pub fn stop_open(&mut self, removed: bool, user: OpenUser) -> OpenEffects {
        let effects = self.openers.stop_open(removed, user, false);
        if effects.closed_boundary {
            self.open_state = false;
        }
        effects
    }

    pub fn recount(&mut self, users: &[OpenUser]) -> OpenEffects {
        let effects = self.openers.recount(users, false);
        if effects.opened_boundary {
            self.open_state = true;
        } else if effects.closed_boundary {
            self.open_state = false;
        }
        effects
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestIdentity {
    Ordinary,
    Trapped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestSide {
    Single,
    Right,
    Left,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChestHalf {
    pub identity: ChestIdentity,
    pub side: ChestSide,
    pub facing: Direction,
    pub blocked: bool,
    pub storage: RandomizableStorage<27>,
    pub openers: OpenerCounter,
}

impl ChestHalf {
    pub fn empty(identity: ChestIdentity, side: ChestSide, facing: Direction) -> Self {
        Self {
            identity,
            side,
            facing,
            blocked: false,
            storage: RandomizableStorage::empty(),
            openers: OpenerCounter::new(),
        }
    }

    pub fn weak_signal(&self) -> u8 {
        if matches!(self.identity, ChestIdentity::Trapped) {
            self.openers.count.clamp(0, 15) as u8
        } else {
            0
        }
    }

    pub fn direct_signal(&self, query: Direction) -> u8 {
        if query == Direction::Up {
            self.weak_signal()
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoubleChest {
    pub right: ChestHalf,
    pub left: ChestHalf,
}

impl DoubleChest {
    pub fn combine(right: ChestHalf, left: ChestHalf) -> Result<Self, ChestPairError> {
        if right.identity != left.identity {
            return Err(ChestPairError::DifferentIdentity);
        }
        if right.facing != left.facing {
            return Err(ChestPairError::DifferentFacing);
        }
        if right.side != ChestSide::Right || left.side != ChestSide::Left {
            return Err(ChestPairError::WrongSides);
        }
        Ok(Self { right, left })
    }

    pub fn is_blocked(&self) -> bool {
        self.right.blocked || self.left.blocked
    }

    pub fn get(&mut self, index: usize, ignore_obstruction: bool) -> Option<&ItemStack> {
        if self.is_blocked() && !ignore_obstruction {
            return None;
        }
        if index < 27 {
            self.right.storage.get(index)
        } else {
            self.left.storage.get(index - 27)
        }
    }

    pub fn comparator_output(&mut self) -> u8 {
        if self.is_blocked() {
            return 0;
        }
        self.right.storage.public_access(LootCaller::NullPlayer);
        self.left.storage.public_access(LootCaller::NullPlayer);
        let fullness = self
            .right
            .storage
            .inventory
            .slots
            .iter()
            .chain(self.left.storage.inventory.slots.iter())
            .filter(|slot| !slot.stack.is_empty())
            .map(|slot| slot.stack.count as f32 / slot.stack.maximum.min(99) as f32)
            .sum::<f32>()
            / 54.0;
        if fullness == 0.0 {
            0
        } else {
            (fullness * 14.0).floor() as u8 + 1
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestPairError {
    DifferentIdentity,
    DifferentFacing,
    WrongSides,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerEnderStorage {
    pub storage: RandomizableStorage<27>,
    pub active_block_entity: Option<u64>,
}

impl PlayerEnderStorage {
    pub fn empty() -> Self {
        Self {
            storage: RandomizableStorage::empty(),
            active_block_entity: None,
        }
    }

    pub fn load_slots(&mut self, entries: &[(usize, ItemStack)]) {
        self.storage.clear_raw();
        for (slot, stack) in entries {
            self.storage.set(*slot, stack.clone());
        }
    }

    pub fn saved_slots(&self) -> Vec<(usize, ItemStack)> {
        self.storage
            .inventory
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.stack.is_empty())
            .map(|(index, slot)| (index, slot.stack.clone()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnderChestPresentation {
    pub identity: u64,
    pub openers: OpenerCounter,
    pub lid_previous: f32,
    pub lid_current: f32,
    pub lid_target_open: bool,
}

impl EnderChestPresentation {
    pub fn new(identity: u64) -> Self {
        Self {
            identity,
            openers: OpenerCounter::new(),
            lid_previous: 0.0,
            lid_current: 0.0,
            lid_target_open: false,
        }
    }

    pub fn apply_block_event(&mut self, event_id: u8, count: i32) -> bool {
        if event_id != 1 {
            return false;
        }
        self.lid_target_open = count > 0;
        true
    }

    pub fn animate_lid(&mut self) {
        self.lid_previous = self.lid_current;
        let delta = if self.lid_target_open { 0.1 } else { -0.1 };
        self.lid_current = (self.lid_current + delta).clamp(0.0, 1.0);
    }

    pub fn eased_lid(&self, partial_tick: f32) -> f32 {
        let open = self.lid_previous + (self.lid_current - self.lid_previous) * partial_tick;
        1.0 - (1.0 - open).powi(3)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContainerDropPlan {
    pub chunks: Vec<ItemStack>,
    pub position_double_draws: usize,
    pub bounded_integer_draws: usize,
    pub velocity_double_draws: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerDropError {
    MissingBoundedDraw,
    BoundedDrawOutOfRange(u8),
}

pub fn plan_removal_drops(
    inventory: &mut Inventory,
    suppress_side_effects: bool,
    bounded_draws: &[u8],
) -> Result<ContainerDropPlan, ContainerDropError> {
    if suppress_side_effects {
        return Ok(ContainerDropPlan::default());
    }
    let position_double_draws = inventory.slots.len() * 3;
    let mut draw_index = 0;
    let mut chunks = Vec::new();
    for slot in &mut inventory.slots {
        while !slot.stack.is_empty() {
            let Some(&draw) = bounded_draws.get(draw_index) else {
                return Err(ContainerDropError::MissingBoundedDraw);
            };
            if draw > 20 {
                return Err(ContainerDropError::BoundedDrawOutOfRange(draw));
            }
            draw_index += 1;
            let count = slot.stack.count.min(i32::from(10 + draw));
            let mut chunk = slot.stack.clone();
            chunk.count = count;
            chunks.push(chunk);
            slot.stack.shrink(count);
        }
    }
    Ok(ContainerDropPlan {
        chunks,
        position_double_draws,
        bounded_integer_draws: draw_index,
        velocity_double_draws: draw_index * 6,
    })
}
