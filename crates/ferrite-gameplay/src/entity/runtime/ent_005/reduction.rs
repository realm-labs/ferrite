//! Armor, magic, absorption, health, combat, and subtype damage reduction.

use std::cmp::Ordering;

use crate::entity::runtime::ent_005::admission::FLOAT_STAT_UPPER_BOUND;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorOwner {
    OrdinaryLiving,
    Player,
    Horse,
    Wolf,
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

pub const PLAYER_ARMOR_ORDER: [EquipmentSlot; 4] = [
    EquipmentSlot::Feet,
    EquipmentSlot::Legs,
    EquipmentSlot::Chest,
    EquipmentSlot::Head,
];

pub const PROTECTION_ORDER: [EquipmentSlot; 8] = [
    EquipmentSlot::MainHand,
    EquipmentSlot::Offhand,
    EquipmentSlot::Feet,
    EquipmentSlot::Legs,
    EquipmentSlot::Chest,
    EquipmentSlot::Head,
    EquipmentSlot::Body,
    EquipmentSlot::Saddle,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmorDurability {
    pub request_per_selected_slot: i32,
    pub selected_slots: &'static [EquipmentSlot],
}

#[must_use]
pub fn armor_durability(owner: ArmorOwner, selected_amount: f32) -> ArmorDurability {
    let selected_slots: &'static [EquipmentSlot] = match owner {
        ArmorOwner::OrdinaryLiving => &[],
        ArmorOwner::Player => &PLAYER_ARMOR_ORDER,
        ArmorOwner::Horse | ArmorOwner::Wolf => &[EquipmentSlot::Body],
    };
    ArmorDurability {
        request_per_selected_slot: (selected_amount / 4.0).max(1.0) as i32,
        selected_slots,
    }
}

#[must_use]
pub const fn armor_slot_takes_durability(
    equippable_damage_on_hurt: bool,
    damageable: bool,
    damage_resistant_to_source: bool,
) -> bool {
    equippable_damage_on_hurt && damageable && !damage_resistant_to_source
}

#[must_use]
pub fn armor_reduction(
    selected_amount: f32,
    armor_attribute: f64,
    toughness: f64,
    breach_levels_in_stored_order: &[u8],
) -> f32 {
    let armor = armor_attribute.floor() as f32;
    let toughness = toughness as f32;
    let divisor = 2.0 + toughness / 4.0;
    let effective = (armor - selected_amount / divisor).clamp(armor * 0.2, 20.0);
    let mut effectiveness = effective / 25.0;
    for level in breach_levels_in_stored_order {
        effectiveness += -0.15 * f32::from(*level);
    }
    selected_amount * (1.0 - effectiveness.clamp(0.0, 1.0))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResistanceResult {
    pub amount: f32,
    pub resisted: f32,
    pub victim_stat: Option<u32>,
    pub attacker_stat: Option<u32>,
}

#[must_use]
pub fn resistance(
    amount: f32,
    amplifier: u8,
    victim_is_server_player: bool,
    nonplayer_with_server_player_attacker: bool,
) -> ResistanceResult {
    let factor = 25_i32 - (i32::from(amplifier) + 1) * 5;
    let reduced = (amount * factor as f32 / 25.0).max(0.0);
    let resisted = amount - reduced;
    let stat = stat_value(resisted);
    ResistanceResult {
        amount: reduced,
        resisted,
        victim_stat: victim_is_server_player.then_some(stat).flatten(),
        attacker_stat: (!victim_is_server_player && nonplayer_with_server_player_attacker)
            .then_some(stat)
            .flatten(),
    }
}

#[must_use]
pub fn protection_reduction(amount: f32, ordered_contributions: &[f32]) -> f32 {
    if !jvm_positive(amount) {
        return 0.0;
    }
    let protection = ordered_contributions
        .iter()
        .fold(0.0_f32, |total, value| total + value)
        .clamp(0.0, 20.0);
    amount * (1.0 - protection / 25.0)
}

#[must_use]
pub fn magic_reduction(input: MagicReductionInput<'_>) -> ResistanceResult {
    if input.bypasses_effects {
        return ResistanceResult {
            amount: input.amount,
            resisted: 0.0,
            victim_stat: None,
            attacker_stat: None,
        };
    }
    let mut result = if let Some(amplifier) = input.resistance_amplifier {
        if input.bypasses_resistance {
            ResistanceResult {
                amount: input.amount,
                resisted: 0.0,
                victim_stat: None,
                attacker_stat: None,
            }
        } else {
            resistance(
                input.amount,
                amplifier,
                input.victim_is_server_player,
                input.nonplayer_with_server_player_attacker,
            )
        }
    } else {
        ResistanceResult {
            amount: input.amount,
            resisted: 0.0,
            victim_stat: None,
            attacker_stat: None,
        }
    };
    if result.amount > 0.0 && !input.bypasses_enchantments {
        result.amount = protection_reduction(result.amount, input.protection_contributions);
    } else if !jvm_positive(result.amount) {
        result.amount = 0.0;
    }
    result
}

#[derive(Debug, Clone, Copy)]
pub struct MagicReductionInput<'a> {
    pub amount: f32,
    pub bypasses_effects: bool,
    pub resistance_amplifier: Option<u8>,
    pub bypasses_resistance: bool,
    pub bypasses_enchantments: bool,
    pub protection_contributions: &'a [f32],
    pub victim_is_server_player: bool,
    pub nonplayer_with_server_player_attacker: bool,
}

#[must_use]
pub fn witch_reduction(
    amount: f32,
    source_entity_is_same_witch: bool,
    witch_resistant_type: bool,
) -> f32 {
    let amount = if source_entity_is_same_witch {
        0.0
    } else {
        amount
    };
    if witch_resistant_type {
        amount * 0.15
    } else {
        amount
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthReductionInput {
    pub defended_amount: f32,
    pub absorption: f32,
    pub maximum_absorption: f32,
    pub health: f32,
    pub maximum_health: f32,
    pub player_victim: bool,
    pub ability_invulnerable: bool,
    pub source_exhaustion: f32,
    pub current_exhaustion: f32,
    pub causing_server_player: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthReduction {
    pub absorption: f32,
    pub health_damage: f32,
    pub health: f32,
    pub exhaustion: f32,
    pub absorbed_stat: Option<u32>,
    pub damage_taken_stat: Option<u32>,
    pub attacker_absorbed_stat: Option<u32>,
    pub record_combat: bool,
    pub emit_entity_damage: bool,
    pub nonplayer_second_absorption_write: bool,
}

#[must_use]
pub fn apply_absorption_and_health(input: HealthReductionInput) -> HealthReduction {
    let health_damage = (input.defended_amount - input.absorption).max(0.0);
    let absorbed = input.defended_amount - health_damage;
    let absorption = (input.absorption - absorbed).clamp(0.0, input.maximum_absorption);
    let absorbed_stat = stat_value(absorbed);
    if health_damage == 0.0 {
        return HealthReduction {
            absorption,
            health_damage,
            health: input.health,
            exhaustion: input.current_exhaustion,
            absorbed_stat: input.player_victim.then_some(absorbed_stat).flatten(),
            damage_taken_stat: None,
            attacker_absorbed_stat: (!input.player_victim && input.causing_server_player)
                .then_some(absorbed_stat)
                .flatten(),
            record_combat: false,
            emit_entity_damage: false,
            nonplayer_second_absorption_write: false,
        };
    }
    let exhaustion = if input.player_victim && !input.ability_invulnerable {
        (input.current_exhaustion + input.source_exhaustion).min(40.0)
    } else {
        input.current_exhaustion
    };
    let final_absorption = if input.player_victim {
        absorption
    } else {
        (absorption - health_damage).clamp(0.0, input.maximum_absorption)
    };
    HealthReduction {
        absorption: final_absorption,
        health_damage,
        health: (input.health - health_damage).clamp(0.0, input.maximum_health),
        exhaustion,
        absorbed_stat: input.player_victim.then_some(absorbed_stat).flatten(),
        damage_taken_stat: input
            .player_victim
            .then_some(stat_value(health_damage))
            .flatten(),
        attacker_absorbed_stat: (!input.player_victim && input.causing_server_player)
            .then_some(absorbed_stat)
            .flatten(),
        record_combat: true,
        emit_entity_damage: true,
        nonplayer_second_absorption_write: !input.player_victim,
    }
}

fn stat_value(amount: f32) -> Option<u32> {
    (amount > 0.0 && amount < FLOAT_STAT_UPPER_BOUND).then(|| (amount * 10.0).round() as u32)
}

fn jvm_positive(amount: f32) -> bool {
    matches!(amount.partial_cmp(&0.0), Some(Ordering::Greater) | None)
}

#[must_use]
pub const fn combat_expired(
    taking_damage: bool,
    in_combat: bool,
    tick_count: u32,
    last_damage_time: u32,
    dead: bool,
) -> bool {
    taking_damage
        && (dead || tick_count.saturating_sub(last_damage_time) > if in_combat { 300 } else { 100 })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WolfArmorCrack {
    None,
    Low,
    Medium,
    High,
}

#[must_use]
pub fn wolf_armor_crack(remaining_fraction: Option<f32>) -> WolfArmorCrack {
    match remaining_fraction {
        Some(value) if value < 0.32 => WolfArmorCrack::High,
        Some(value) if value < 0.69 => WolfArmorCrack::Medium,
        Some(value) if value < 0.95 => WolfArmorCrack::Low,
        _ => WolfArmorCrack::None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WolfArmorOutcome {
    pub intercepts_common_damage: bool,
    pub requested_durability: i32,
    pub crack_changed: bool,
    pub particles: u8,
}

#[must_use]
pub fn wolf_armor_outcome(
    selected_amount: f32,
    exact_wolf_armor: bool,
    bypasses_wolf_armor: bool,
    before: WolfArmorCrack,
    after: WolfArmorCrack,
) -> WolfArmorOutcome {
    let intercepts = exact_wolf_armor && !bypasses_wolf_armor;
    let crack_changed = intercepts && before != after;
    WolfArmorOutcome {
        intercepts_common_damage: intercepts,
        requested_durability: if intercepts {
            selected_amount.ceil() as i32
        } else {
            0
        },
        crack_changed,
        particles: if crack_changed { 20 } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmadilloAction {
    None,
    RememberDanger { ticks: u8 },
    RollUp { ticks: u8 },
    RollOut,
}

#[must_use]
pub const fn armadillo_post_damage(input: ArmadilloInput) -> ArmadilloAction {
    if input.no_ai || input.dead_or_dying {
        ArmadilloAction::None
    } else if input.causing_living {
        if input.panicking
            || input.in_liquid
            || input.leashed
            || input.passenger
            || input.vehicle
            || input.already_scared
        {
            ArmadilloAction::RememberDanger { ticks: 80 }
        } else {
            ArmadilloAction::RollUp { ticks: 80 }
        }
    } else if input.environmental_panic && input.already_scared {
        ArmadilloAction::RollOut
    } else {
        ArmadilloAction::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmadilloInput {
    pub no_ai: bool,
    pub dead_or_dying: bool,
    pub causing_living: bool,
    pub panicking: bool,
    pub in_liquid: bool,
    pub leashed: bool,
    pub passenger: bool,
    pub vehicle: bool,
    pub already_scared: bool,
    pub environmental_panic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrePostHooks {
    pub camel_stands: bool,
    pub camel_pose_tick_offset: u8,
    pub clear_animal_love_before_defense: bool,
    pub copper_golem_idle_after_defense: bool,
}

#[must_use]
pub const fn subtype_hooks(camel: bool, animal: bool, copper_golem: bool) -> PrePostHooks {
    PrePostHooks {
        camel_stands: camel,
        camel_pose_tick_offset: if camel { 53 } else { 0 },
        clear_animal_love_before_defense: animal,
        copper_golem_idle_after_defense: copper_golem,
    }
}
