//! Dispenser/dropper trigger, selection, behavior dispatch, and wrapper events.

use crate::item::runtime::inventory::{
    Inventory, SelectionError, Slot, TransferPolicy, move_item_stack_to, select_random_occupied,
    transfer_one,
};
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::direction::Direction;
use ferrite_foundation::resource::ResourceId;

pub const DISPENSER_SLOTS: usize = 9;
pub const TRIGGER_DELAY: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerTransition {
    pub schedule_after: Option<u32>,
    pub offered_triggered: Option<bool>,
}

pub const fn neighbor_trigger(powered: bool, captured_triggered: bool) -> TriggerTransition {
    if powered && !captured_triggered {
        TriggerTransition {
            schedule_after: Some(TRIGGER_DELAY),
            offered_triggered: Some(true),
        }
    } else if !powered && captured_triggered {
        TriggerTransition {
            schedule_after: None,
            offered_triggered: Some(false),
        }
    } else {
        TriggerTransition {
            schedule_after: None,
            offered_triggered: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispenseBehavior {
    Noop,
    Projectile,
    FireworkProjectile,
    FireChargeProjectile,
    WindChargeProjectile,
    ArmorStand,
    HorseChest,
    Boat,
    FilledBucket,
    EmptyBucket,
    FlintAndSteel,
    BoneMeal,
    Tnt,
    WitherSkull,
    CarvedPumpkin,
    ShulkerBox,
    GlassBottle,
    Glowstone,
    Shears,
    Brush,
    Honeycomb,
    Potion,
    Minecart,
    Equipment,
    SulfurCubeEquipment,
    SpawnEgg,
    DefaultEjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DynamicDispenseComponents {
    pub feature_enabled: bool,
    pub equippable: bool,
    pub sulfur_swallowable: bool,
    pub spawn_egg_with_entity_data: bool,
}

pub fn resolve_behavior(
    item: &ResourceId,
    components: DynamicDispenseComponents,
) -> DispenseBehavior {
    if !components.feature_enabled {
        return DispenseBehavior::DefaultEjection;
    }
    if let Some(explicit) = explicit_behavior(item) {
        return explicit;
    }
    if components.equippable {
        DispenseBehavior::Equipment
    } else if components.sulfur_swallowable {
        DispenseBehavior::SulfurCubeEquipment
    } else if components.spawn_egg_with_entity_data {
        DispenseBehavior::SpawnEgg
    } else {
        DispenseBehavior::DefaultEjection
    }
}

pub fn explicit_behavior(item: &ResourceId) -> Option<DispenseBehavior> {
    if item.namespace() != "minecraft" {
        return None;
    }
    let path = item.path();
    match path {
        "firework_rocket" => return Some(DispenseBehavior::FireworkProjectile),
        "fire_charge" => return Some(DispenseBehavior::FireChargeProjectile),
        "wind_charge" => return Some(DispenseBehavior::WindChargeProjectile),
        _ => {}
    }
    if PROJECTILES.contains(&path) {
        return Some(DispenseBehavior::Projectile);
    }
    if BOATS.contains(&path) {
        return Some(DispenseBehavior::Boat);
    }
    if FILLED_BUCKETS.contains(&path) {
        return Some(DispenseBehavior::FilledBucket);
    }
    if SHULKER_BOXES.contains(&path) {
        return Some(DispenseBehavior::ShulkerBox);
    }
    if MINECARTS.contains(&path) {
        return Some(DispenseBehavior::Minecart);
    }
    match path {
        "armor_stand" => Some(DispenseBehavior::ArmorStand),
        "chest" => Some(DispenseBehavior::HorseChest),
        "bucket" => Some(DispenseBehavior::EmptyBucket),
        "flint_and_steel" => Some(DispenseBehavior::FlintAndSteel),
        "bone_meal" => Some(DispenseBehavior::BoneMeal),
        "tnt" => Some(DispenseBehavior::Tnt),
        "wither_skeleton_skull" => Some(DispenseBehavior::WitherSkull),
        "carved_pumpkin" => Some(DispenseBehavior::CarvedPumpkin),
        "glass_bottle" => Some(DispenseBehavior::GlassBottle),
        "glowstone" => Some(DispenseBehavior::Glowstone),
        "shears" => Some(DispenseBehavior::Shears),
        "brush" => Some(DispenseBehavior::Brush),
        "honeycomb" => Some(DispenseBehavior::Honeycomb),
        "potion" => Some(DispenseBehavior::Potion),
        _ => None,
    }
}

pub const EXPLICIT_BEHAVIOR_COUNT: usize = PROJECTILES.len()
    + BOATS.len()
    + FILLED_BUCKETS.len()
    + SHULKER_BOXES.len()
    + MINECARTS.len()
    + 14;

const PROJECTILES: [&str; 13] = [
    "arrow",
    "tipped_arrow",
    "spectral_arrow",
    "egg",
    "blue_egg",
    "brown_egg",
    "snowball",
    "experience_bottle",
    "splash_potion",
    "lingering_potion",
    "firework_rocket",
    "fire_charge",
    "wind_charge",
];
const BOATS: [&str; 20] = [
    "oak_boat",
    "spruce_boat",
    "birch_boat",
    "jungle_boat",
    "dark_oak_boat",
    "acacia_boat",
    "cherry_boat",
    "mangrove_boat",
    "pale_oak_boat",
    "bamboo_raft",
    "oak_chest_boat",
    "spruce_chest_boat",
    "birch_chest_boat",
    "jungle_chest_boat",
    "dark_oak_chest_boat",
    "acacia_chest_boat",
    "cherry_chest_boat",
    "mangrove_chest_boat",
    "pale_oak_chest_boat",
    "bamboo_chest_raft",
];
const FILLED_BUCKETS: [&str; 10] = [
    "lava_bucket",
    "water_bucket",
    "powder_snow_bucket",
    "salmon_bucket",
    "cod_bucket",
    "pufferfish_bucket",
    "tropical_fish_bucket",
    "axolotl_bucket",
    "sulfur_cube_bucket",
    "tadpole_bucket",
];
const SHULKER_BOXES: [&str; 17] = [
    "shulker_box",
    "white_shulker_box",
    "orange_shulker_box",
    "magenta_shulker_box",
    "light_blue_shulker_box",
    "yellow_shulker_box",
    "lime_shulker_box",
    "pink_shulker_box",
    "gray_shulker_box",
    "light_gray_shulker_box",
    "cyan_shulker_box",
    "purple_shulker_box",
    "blue_shulker_box",
    "brown_shulker_box",
    "green_shulker_box",
    "red_shulker_box",
    "black_shulker_box",
];
const MINECARTS: [&str; 6] = [
    "minecart",
    "chest_minecart",
    "furnace_minecart",
    "tnt_minecart",
    "hopper_minecart",
    "command_block_minecart",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelEvent {
    Dispense,
    Fail,
    Launch,
    Firework,
    FireCharge,
    WindCharge,
    Animate(Direction),
}

pub fn wrapper_events(
    behavior: DispenseBehavior,
    success: bool,
    facing: Direction,
    nested_default_pairs: u8,
) -> Vec<LevelEvent> {
    if matches!(behavior, DispenseBehavior::Noop) {
        return Vec::new();
    }
    let mut events = Vec::with_capacity(2 + usize::from(nested_default_pairs) * 2);
    for _ in 0..nested_default_pairs {
        events.push(LevelEvent::Dispense);
        events.push(LevelEvent::Animate(facing));
    }
    let sound = match behavior {
        DispenseBehavior::Projectile => LevelEvent::Launch,
        DispenseBehavior::FireworkProjectile => LevelEvent::Firework,
        DispenseBehavior::FireChargeProjectile => LevelEvent::FireCharge,
        DispenseBehavior::WindChargeProjectile => LevelEvent::WindCharge,
        _ if success => LevelEvent::Dispense,
        _ => LevelEvent::Fail,
    };
    events.push(sound);
    events.push(LevelEvent::Animate(facing));
    events
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionalBehaviorState {
    pub brush_success: bool,
    pub tnt_success: bool,
}

impl Default for OptionalBehaviorState {
    fn default() -> Self {
        Self {
            brush_success: true,
            tnt_success: true,
        }
    }
}

impl OptionalBehaviorState {
    pub fn brush(&mut self, action_succeeded: bool) -> bool {
        if !action_succeeded {
            self.brush_success = false;
        }
        self.brush_success
    }

    pub fn tnt(&mut self, action: TntAction) -> bool {
        match action {
            TntAction::GameRuleDisabled => self.tnt_success = false,
            TntAction::OrdinaryPrime => self.tnt_success = true,
            TntAction::SulfurCubeAccepted => {}
        }
        self.tnt_success
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TntAction {
    GameRuleDisabled,
    OrdinaryPrime,
    SulfurCubeAccepted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemainderOutcome {
    pub selected_stack: ItemStack,
    pub ejected_remainder: Option<ItemStack>,
    pub extra_events: Vec<LevelEvent>,
}

pub fn consume_with_remainder(
    mut selected_stack: ItemStack,
    remainder: ItemStack,
    source: &mut Inventory,
    facing: Direction,
) -> RemainderOutcome {
    selected_stack.shrink(1);
    if selected_stack.is_empty() {
        return RemainderOutcome {
            selected_stack: remainder,
            ejected_remainder: None,
            extra_events: Vec::new(),
        };
    }
    let mut remainder = remainder;
    let slot_count = source.slots.len();
    move_item_stack_to(&mut remainder, &mut source.slots, 0..slot_count, false);
    if remainder.is_empty() {
        RemainderOutcome {
            selected_stack,
            ejected_remainder: None,
            extra_events: Vec::new(),
        }
    } else {
        RemainderOutcome {
            selected_stack,
            ejected_remainder: Some(remainder),
            extra_events: vec![LevelEvent::Dispense, LevelEvent::Animate(facing)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropperDispatch {
    MissingBlockEntity,
    Empty {
        events: Vec<LevelEvent>,
    },
    Target {
        selected_slot: usize,
        inserted: bool,
    },
    Ejected {
        selected_slot: usize,
        stack: ItemStack,
        events: Vec<LevelEvent>,
        random_double_draws: u8,
    },
}

pub fn dispatch_dropper(
    source: &mut Inventory,
    draws: &[usize],
    target: Option<&mut Inventory>,
    captured_facing: Direction,
    split_identity: u64,
) -> Result<DropperDispatch, SelectionError> {
    let Some(selected_slot) = select_random_occupied(&source.slots, draws)? else {
        return Ok(DropperDispatch::Empty {
            events: vec![LevelEvent::Fail],
        });
    };
    if let Some(target) = target {
        let slots = (0..target.slots.len()).collect::<Vec<_>>();
        let inserted = transfer_one(
            source,
            selected_slot,
            target,
            &slots,
            TransferPolicy::default(),
            split_identity,
        );
        source.changed_calls += 1;
        return Ok(DropperDispatch::Target {
            selected_slot,
            inserted,
        });
    }
    let ejected = source.slots[selected_slot].stack.split(1, split_identity);
    source.changed_calls += 1;
    Ok(DropperDispatch::Ejected {
        selected_slot,
        stack: ejected,
        events: vec![LevelEvent::Dispense, LevelEvent::Animate(captured_facing)],
        random_double_draws: 7,
    })
}

pub fn explicit_items() -> impl Iterator<Item = &'static str> {
    PROJECTILES
        .into_iter()
        .chain(BOATS)
        .chain(FILLED_BUCKETS)
        .chain(SHULKER_BOXES)
        .chain(MINECARTS)
        .chain([
            "armor_stand",
            "chest",
            "bucket",
            "flint_and_steel",
            "bone_meal",
            "tnt",
            "wither_skeleton_skull",
            "carved_pumpkin",
            "glass_bottle",
            "glowstone",
            "shears",
            "brush",
            "honeycomb",
            "potion",
        ])
}

pub fn empty_dispenser() -> Inventory {
    Inventory {
        slots: (0..DISPENSER_SLOTS).map(|_| Slot::empty()).collect(),
        changed_calls: 0,
    }
}
