//! Boat placement, interaction, chest storage, removal, and dispenser transactions.

use crate::item::runtime::container_storage::{
    CONTAINER_MAXIMUM, LootCaller, PendingLoot, RandomizableStorage,
};
use crate::item::runtime::inventory::Slot;
use crate::item::runtime::random::{GameplayRandom, GameplayRandomError, checked_int};
use crate::item::runtime::stack::ItemStack;

pub const BOAT_WIDTH: f64 = 1.375;
pub const BOAT_HEIGHT: f64 = 0.5625;
pub const BOAT_TRACKING_RANGE: u32 = 10;
pub const BOAT_VIEW_RANGE: f64 = 5.0;
pub const BOAT_SWEEP_INFLATION: f64 = 1.0;
pub const CHEST_BOAT_SLOTS: usize = 27;
pub const ORDINARY_BOAT_PASSENGERS: usize = 2;
pub const CHEST_BOAT_PASSENGERS: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoatHit {
    Miss,
    Block(Position),
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatUseResult {
    Pass,
    Fail,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackConfigurationStep {
    ImplicitComponents,
    ExplicitEntityData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoatUseOutcome {
    pub result: BoatUseResult,
    pub entity_created: bool,
    pub position: Option<Position>,
    pub yaw: Option<f32>,
    pub configuration: Vec<StackConfigurationStep>,
    pub spawn_attempted: bool,
    pub admitted: bool,
    pub placement_event: bool,
    pub consumed: u8,
    pub awarded_item_used_stat: bool,
}

impl BoatUseOutcome {
    fn terminal(result: BoatUseResult) -> Self {
        Self {
            result,
            entity_created: false,
            position: None,
            yaw: None,
            configuration: Vec::new(),
            spawn_attempted: false,
            admitted: false,
            placement_event: false,
            consumed: 0,
            awarded_item_used_stat: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoatUseInput {
    pub hit: BoatHit,
    pub eye_inside_pickable_box: bool,
    pub factory_created: bool,
    pub collision_free: bool,
    pub server_side: bool,
    pub admission_accepted: bool,
    pub player_yaw: f32,
}

pub fn use_boat(input: BoatUseInput) -> BoatUseOutcome {
    if matches!(input.hit, BoatHit::Miss) || input.eye_inside_pickable_box {
        return BoatUseOutcome::terminal(BoatUseResult::Pass);
    }
    let BoatHit::Block(position) = input.hit else {
        return BoatUseOutcome::terminal(BoatUseResult::Pass);
    };
    if !input.factory_created {
        return BoatUseOutcome::terminal(BoatUseResult::Fail);
    }
    let mut outcome = BoatUseOutcome {
        result: BoatUseResult::Fail,
        entity_created: true,
        position: Some(position),
        yaw: Some(input.player_yaw),
        configuration: if input.server_side {
            vec![
                StackConfigurationStep::ImplicitComponents,
                StackConfigurationStep::ExplicitEntityData,
            ]
        } else {
            Vec::new()
        },
        spawn_attempted: false,
        admitted: false,
        placement_event: false,
        consumed: 0,
        awarded_item_used_stat: false,
    };
    if !input.collision_free {
        return outcome;
    }
    outcome.result = BoatUseResult::Success;
    outcome.awarded_item_used_stat = true;
    if input.server_side {
        outcome.spawn_attempted = true;
        outcome.admitted = input.admission_accepted;
        outcome.placement_event = true;
        outcome.consumed = 1;
    }
    outcome
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleBaseInteraction {
    Pass,
    Success,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatInteractionAction {
    Pass,
    Mount,
    OpenContainer,
    BaseSuccess,
    BaseFail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatInteractionOutcome {
    pub action: BoatInteractionAction,
    pub result: BoatUseResult,
    pub container_open_event: bool,
    pub anger_piglins: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoatInteractionInput {
    pub base: VehicleBaseInteraction,
    pub chest: bool,
    pub secondary_use: bool,
    pub out_of_control_ticks: f32,
    pub client_side: bool,
    pub start_riding_succeeds: bool,
    pub can_add_passenger: bool,
}

pub fn interact_boat(input: BoatInteractionInput) -> BoatInteractionOutcome {
    let terminal = |action, result| BoatInteractionOutcome {
        action,
        result,
        container_open_event: false,
        anger_piglins: false,
    };
    match input.base {
        VehicleBaseInteraction::Success => {
            return terminal(BoatInteractionAction::BaseSuccess, BoatUseResult::Success);
        }
        VehicleBaseInteraction::Fail => {
            return terminal(BoatInteractionAction::BaseFail, BoatUseResult::Fail);
        }
        VehicleBaseInteraction::Pass => {}
    }
    if !input.secondary_use
        && input.out_of_control_ticks < 60.0
        && (input.client_side || input.start_riding_succeeds)
    {
        return terminal(BoatInteractionAction::Mount, BoatUseResult::Success);
    }
    if input.chest && (!input.can_add_passenger || input.secondary_use) {
        return BoatInteractionOutcome {
            action: BoatInteractionAction::OpenContainer,
            result: BoatUseResult::Success,
            container_open_event: !input.client_side,
            anger_piglins: !input.client_side,
        };
    }
    terminal(BoatInteractionAction::Pass, BoatUseResult::Pass)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChestBoatStorage {
    pub storage: RandomizableStorage<CHEST_BOAT_SLOTS>,
}

impl ChestBoatStorage {
    pub fn empty() -> Self {
        Self {
            storage: RandomizableStorage::empty(),
        }
    }

    pub fn materialize(
        &mut self,
        caller: LootCaller,
        mut fill: impl FnMut(&PendingLoot, &mut [Slot]),
    ) -> bool {
        let Some(pending) = self.storage.pending_loot.take() else {
            return false;
        };
        self.storage.materialized_by = Some(caller);
        fill(&pending, &mut self.storage.inventory.slots);
        true
    }

    pub fn set(
        &mut self,
        slot: usize,
        stack: ItemStack,
        fill: impl FnMut(&PendingLoot, &mut [Slot]),
    ) -> bool {
        self.materialize(LootCaller::NullPlayer, fill);
        let Some(target) = self.storage.inventory.slots.get_mut(slot) else {
            return false;
        };
        let mut stack = stack;
        stack.count = stack.count.min(CONTAINER_MAXIMUM).min(stack.maximum);
        target.stack = stack.normalized();
        true
    }

    pub fn open(
        &mut self,
        spectator: bool,
        player_fingerprint: u64,
        luck: f32,
        fill: impl FnMut(&PendingLoot, &mut [Slot]),
    ) -> bool {
        if spectator && self.storage.pending_loot.is_some() {
            return false;
        }
        self.materialize(
            LootCaller::Player {
                player_fingerprint,
                luck_bits: luck.to_bits(),
            },
            fill,
        );
        true
    }

    pub fn still_valid(removed: bool, distance_squared: f64, interaction_range: f64) -> bool {
        !removed && distance_squared < (interaction_range + 4.0).powi(2)
    }

    pub fn save(&self) -> ChestBoatSave {
        if let Some(pending) = &self.storage.pending_loot {
            ChestBoatSave::PendingLoot {
                table_fingerprint: pending.table_fingerprint,
                seed: (pending.seed != 0).then_some(pending.seed),
            }
        } else {
            ChestBoatSave::Items(
                self.storage
                    .inventory
                    .slots
                    .iter()
                    .map(|slot| slot.stack.clone())
                    .collect(),
            )
        }
    }

    pub fn load(save: ChestBoatSave) -> Self {
        let mut value = Self::empty();
        match save {
            ChestBoatSave::PendingLoot {
                table_fingerprint,
                seed,
            } => {
                value.storage.pending_loot = Some(PendingLoot {
                    table_fingerprint,
                    seed: seed.unwrap_or(0),
                });
            }
            ChestBoatSave::Items(items) => {
                for (slot, stack) in value.storage.inventory.slots.iter_mut().zip(items) {
                    slot.stack = stack;
                }
            }
        }
        value
    }
}

impl Default for ChestBoatStorage {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChestBoatSave {
    PendingLoot {
        table_fingerprint: u64,
        seed: Option<i64>,
    },
    Items(Vec<ItemStack>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalReason {
    Killed,
    Discarded,
    Unloaded,
    ChangedDimension,
}

impl RemovalReason {
    const fn should_destroy(self) -> bool {
        matches!(self, Self::Killed | Self::Discarded)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoatRemovalOutcome {
    pub scattered_contents: Vec<ItemStack>,
    pub matching_vehicle_item: bool,
    pub anger_piglins: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatRemovalContext {
    pub reason: RemovalReason,
    pub server_side: bool,
    pub entity_drops: bool,
    pub itemize_vehicle: bool,
    pub direct_player_damage: bool,
}

pub fn remove_boat(
    chest: Option<&mut ChestBoatStorage>,
    context: BoatRemovalContext,
    random: &mut dyn GameplayRandom,
    mut next_identity: impl FnMut() -> u64,
    mut fill: impl FnMut(&PendingLoot, &mut [Slot]),
) -> Result<BoatRemovalOutcome, GameplayRandomError> {
    let chest_present = chest.is_some();
    let mut scattered_contents = Vec::new();
    if context.server_side
        && context.reason.should_destroy()
        && let Some(chest) = chest
    {
        chest.materialize(LootCaller::NullPlayer, &mut fill);
        for slot in &mut chest.storage.inventory.slots {
            while !slot.stack.is_empty() {
                let count = checked_int(random, 21)? as i32 + 10;
                scattered_contents.push(slot.stack.split(count, next_identity()));
            }
        }
    }
    Ok(BoatRemovalOutcome {
        scattered_contents,
        matching_vehicle_item: context.itemize_vehicle && context.entity_drops,
        anger_piglins: context.itemize_vehicle
            && chest_present
            && context.entity_drops
            && context.direct_player_damage,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispenserTerrain {
    Water,
    AirOverWater,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DispenserPlacement {
    pub fallback: bool,
    pub position: Position,
    pub yaw: f32,
    pub consume_after_creation: bool,
}

pub fn dispense_boat(
    origin: Position,
    facing_x: i32,
    facing_y: i32,
    facing_z: i32,
    facing_yaw: f32,
    terrain: DispenserTerrain,
    factory_created: bool,
) -> DispenserPlacement {
    let vertical_offset = match terrain {
        DispenserTerrain::Water => 1.0,
        DispenserTerrain::AirOverWater => 0.0,
        DispenserTerrain::Fallback => {
            return DispenserPlacement {
                fallback: true,
                position: origin,
                yaw: 0.0,
                consume_after_creation: false,
            };
        }
    };
    let horizontal = 0.5625 + BOAT_WIDTH / 2.0;
    DispenserPlacement {
        fallback: false,
        position: Position {
            x: origin.x + f64::from(facing_x) * horizontal,
            y: origin.y + f64::from(facing_y) * 1.125 + vertical_offset,
            z: origin.z + f64::from(facing_z) * horizontal,
        },
        yaw: facing_yaw,
        consume_after_creation: factory_created,
    }
}

pub const fn passenger_ride_height(passenger_height: f32, raft: bool) -> f32 {
    if raft {
        passenger_height * 0.888_888_9
    } else {
        passenger_height / 3.0
    }
}

pub const fn qualifies_goat_boat_advancement(chest: bool, has_goat_passenger: bool) -> bool {
    !chest && has_goat_passenger
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatRecipeKind {
    ShapedFromPlanks,
    ShapelessChestAndBoat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatRecipeProfile {
    pub kind: BoatRecipeKind,
    pub output_count: u8,
    pub copies_source_components: bool,
}

pub const fn boat_recipe(chest: bool) -> BoatRecipeProfile {
    BoatRecipeProfile {
        kind: if chest {
            BoatRecipeKind::ShapelessChestAndBoat
        } else {
            BoatRecipeKind::ShapedFromPlanks
        },
        output_count: 1,
        copies_source_components: false,
    }
}

pub fn destruction_item_custom_name(custom_name: Option<&str>) -> Option<String> {
    custom_name.map(str::to_owned)
}
