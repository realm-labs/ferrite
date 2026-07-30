//! Death loot gates, contexts, item spawning, equipment, and subtype overrides.

use crate::entity::runtime::ent_005::knockback::Vector3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootOwner {
    CommonLiving,
    Monster,
}

#[must_use]
pub const fn loot_gate(owner: LootOwner, adult: bool, mob_drops: bool) -> bool {
    mob_drops && (adult || matches!(owner, LootOwner::Monster))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LootContext {
    pub this_entity: bool,
    pub origin: bool,
    pub damage_source: bool,
    pub attacking_entity: bool,
    pub direct_attacking_entity: bool,
    pub last_damage_player: bool,
    pub last_damage_player_luck: Option<f32>,
    pub seed: u64,
}

#[must_use]
pub const fn loot_context(
    attacking_entity: bool,
    direct_attacking_entity: bool,
    recent_player_memory: bool,
    remembered_player_present: bool,
    remembered_player_luck: f32,
    seed: u64,
) -> LootContext {
    let last_damage_player = recent_player_memory && remembered_player_present;
    LootContext {
        this_entity: true,
        origin: true,
        damage_source: true,
        attacking_entity,
        direct_attacking_entity,
        last_damage_player,
        last_damage_player_luck: if last_damage_player {
            Some(remembered_player_luck)
        } else {
            None
        },
        seed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemSpawn {
    pub admitted: bool,
    pub position: Vector3,
    pub pickup_delay: u16,
    pub velocity: Vector3,
    pub construction_draws: u8,
}

#[must_use]
pub fn spawn_at_location(
    empty_stack: bool,
    position: Vector3,
    first_float: f32,
    second_float: f32,
    first_double: f64,
    second_double: f64,
    wither_rose_direct: bool,
) -> ItemSpawn {
    if empty_stack {
        return ItemSpawn {
            admitted: false,
            position,
            pickup_delay: 0,
            velocity: Vector3::ZERO,
            construction_draws: 0,
        };
    }
    let _bob = first_float;
    let _yaw = second_float;
    ItemSpawn {
        admitted: true,
        position,
        pickup_delay: if wither_rose_direct { 0 } else { 10 },
        velocity: Vector3::new(first_double * 0.2 - 0.1, 0.2, second_double * 0.2 - 0.1),
        construction_draws: 4,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand,
    Offhand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

pub const EQUIPMENT_ORDER: [EquipmentSlot; 8] = [
    EquipmentSlot::MainHand,
    EquipmentSlot::Offhand,
    EquipmentSlot::Feet,
    EquipmentSlot::Legs,
    EquipmentSlot::Chest,
    EquipmentSlot::Head,
    EquipmentSlot::Body,
    EquipmentSlot::Saddle,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquipmentDropInput {
    pub nonempty: bool,
    pub base_chance: Option<f32>,
    pub causing_living: bool,
    pub looting_level: u8,
    pub looting_holder_is_player: bool,
    pub prevent_equipment_drop: bool,
    pub killed_by_player: bool,
    pub chance_draw: f32,
    pub damageable: bool,
    pub maximum_damage: u32,
    pub inner_damage_draw: u32,
    pub outer_damage_draw: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquipmentDrop {
    pub adjusted_chance: f32,
    pub preserved: bool,
    pub chance_draw_consumed: bool,
    pub drop: bool,
    pub new_damage: Option<u32>,
    pub damage_draws_consumed: u8,
    pub clear_slot: bool,
}

#[must_use]
pub fn equipment_drop(input: EquipmentDropInput) -> EquipmentDrop {
    let mut chance = input.base_chance.unwrap_or(0.085);
    let preserved = chance > 1.0;
    if chance == 0.0
        || !input.nonempty
        || input.prevent_equipment_drop
        || (!input.killed_by_player && !preserved)
    {
        return EquipmentDrop {
            adjusted_chance: chance,
            preserved,
            chance_draw_consumed: false,
            drop: false,
            new_damage: None,
            damage_draws_consumed: 0,
            clear_slot: false,
        };
    }
    if input.causing_living && input.looting_holder_is_player {
        chance += 0.01 * f32::from(input.looting_level);
    }
    let drop = input.chance_draw < chance;
    if !drop {
        return EquipmentDrop {
            adjusted_chance: chance,
            preserved,
            chance_draw_consumed: true,
            drop: false,
            new_damage: None,
            damage_draws_consumed: 0,
            clear_slot: false,
        };
    }
    let new_damage = if !preserved && input.damageable {
        let inner_bound = input.maximum_damage.saturating_sub(3).max(1);
        let inner = input.inner_damage_draw % inner_bound;
        let outer = input.outer_damage_draw % (1 + inner);
        Some(input.maximum_damage.saturating_sub(outer))
    } else {
        None
    };
    EquipmentDrop {
        adjusted_chance: chance,
        preserved,
        chance_draw_consumed: true,
        drop: true,
        new_damage,
        damage_draws_consumed: u8::from(new_damage.is_some()) * 2,
        clear_slot: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentOverride {
    FoxMainHandBeforeBase,
    AllayInventoryThenMainHand,
    HorseInventory,
    ChestedHorseChestAfterInventory,
    CopperPreservedSlots,
    PiglinInventoryInsideLootGate,
    EndermanSilkTouchBlockLootInsideGate,
    WitherNetherStarInsideGate,
}

#[must_use]
pub const fn override_ignores_mob_drops(kind: EquipmentOverride) -> bool {
    matches!(
        kind,
        EquipmentOverride::FoxMainHandBeforeBase
            | EquipmentOverride::AllayInventoryThenMainHand
            | EquipmentOverride::HorseInventory
            | EquipmentOverride::ChestedHorseChestAfterInventory
            | EquipmentOverride::CopperPreservedSlots
    )
}

#[must_use]
pub const fn nether_star_age(entity_creation_succeeded: bool) -> Option<i32> {
    if entity_creation_succeeded {
        Some(-6_000)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerItemDrop {
    pub position: Vector3,
    pub pickup_delay: u16,
    pub velocity: Vector3,
    pub victim_draws_consumed: u8,
    pub thrower_present: bool,
}

#[must_use]
pub fn player_item_drop(
    x: f64,
    eye_y: f64,
    z: f64,
    speed_draw: f32,
    angle_draw: f32,
) -> PlayerItemDrop {
    let speed = speed_draw * 0.5;
    let angle = angle_draw * 6.283_185_5;
    PlayerItemDrop {
        position: Vector3::new(x, eye_y - f64::from(0.3_f32), z),
        pickup_delay: 40,
        velocity: Vector3::new(
            f64::from(-angle.sin() * speed),
            f64::from(0.2_f32),
            f64::from(angle.cos() * speed),
        ),
        victim_draws_consumed: 2,
        thrower_present: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerInventoryPlan {
    pub destroy_prevent_drop_first: bool,
    pub drop_ordinary_indices_ascending: bool,
    pub drop_equipment_enum_order: bool,
    pub clear_each_owner: bool,
}

#[must_use]
pub const fn player_inventory_plan(
    keep_inventory: bool,
    spectator: bool,
) -> Option<PlayerInventoryPlan> {
    if keep_inventory || spectator {
        None
    } else {
        Some(PlayerInventoryPlan {
            destroy_prevent_drop_first: true,
            drop_ordinary_indices_ascending: true,
            drop_equipment_enum_order: true,
            clear_each_owner: true,
        })
    }
}
