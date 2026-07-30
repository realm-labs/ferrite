//! Data-driven enchantment hook ordering, selection, offers, and menu commit.

use crate::item::runtime::random::{
    GameplayRandom, GameplayRandomError, checked_float, checked_int,
};
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

impl EquipmentSlot {
    pub const ALL: [Self; 8] = [
        Self::MainHand,
        Self::OffHand,
        Self::Feet,
        Self::Legs,
        Self::Chest,
        Self::Head,
        Self::Body,
        Self::Saddle,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantmentCost {
    pub base: i32,
    pub per_level_above_first: i32,
}

impl EnchantmentCost {
    pub fn at_level(self, level: u8) -> i32 {
        self.base.saturating_add(
            self.per_level_above_first
                .saturating_mul(i32::from(level.saturating_sub(1))),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantmentDefinition {
    pub key: ResourceId,
    pub weight: u32,
    pub minimum_level: u8,
    pub maximum_level: u8,
    pub minimum_cost: EnchantmentCost,
    pub maximum_cost: EnchantmentCost,
    pub primary_items: BTreeSet<ResourceId>,
    pub exclusive_with: BTreeSet<ResourceId>,
    pub matching_slots: BTreeSet<EquipmentSlot>,
    pub effects: Vec<ResourceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveEnchantment {
    pub key: ResourceId,
    pub level: u8,
    pub matching_slots: BTreeSet<EquipmentSlot>,
    pub effects: Vec<ResourceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantedItem {
    pub stack: ItemStack,
    pub enchantable: Option<u32>,
    pub ordinary_book: bool,
    pub active: Vec<ActiveEnchantment>,
    pub stored: Vec<ActiveEnchantment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquippedEnchantments {
    pub slot: EquipmentSlot,
    pub item: EnchantedItem,
}

pub fn compose_item_value(
    item: &EnchantedItem,
    component: &ResourceId,
    mut value: f32,
    mut apply: impl FnMut(&ActiveEnchantment, usize, f32) -> f32,
) -> f32 {
    for enchantment in &item.active {
        for (index, effect) in enchantment.effects.iter().enumerate() {
            if effect == component {
                value = apply(enchantment, index, value);
            }
        }
    }
    value
}

pub fn compose_item_int(
    item: &EnchantedItem,
    component: &ResourceId,
    value: i32,
    minimum_zero: bool,
    apply: impl FnMut(&ActiveEnchantment, usize, f32) -> f32,
) -> i32 {
    let value = compose_item_value(item, component, value as f32, apply) as i32;
    if minimum_zero { value.max(0) } else { value }
}

pub fn visit_equipment(
    equipment: &[EquippedEnchantments],
    mut visitor: impl FnMut(EquipmentSlot, &ActiveEnchantment),
) {
    for slot in EquipmentSlot::ALL {
        let Some(equipped) = equipment.iter().find(|equipped| equipped.slot == slot) else {
            continue;
        };
        if equipped.item.stack.is_empty() {
            continue;
        }
        for enchantment in &equipped.item.active {
            if enchantment.matching_slots.contains(&slot) {
                visitor(slot, enchantment);
            }
        }
    }
}

pub fn equipment_immunity(
    equipment: &[EquippedEnchantments],
    mut invoke: impl FnMut(EquipmentSlot, &ActiveEnchantment) -> bool,
) -> bool {
    let mut immune = false;
    visit_equipment(equipment, |slot, enchantment| {
        immune = invoke(slot, enchantment) || immune;
    });
    immune
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostAttackTarget {
    Victim,
    Attacker,
}

pub fn visit_post_attack(
    victim_equipment: &[EquippedEnchantments],
    attacker_weapon: Option<&EnchantedItem>,
    mut visitor: impl FnMut(PostAttackTarget, Option<EquipmentSlot>, &ActiveEnchantment),
) {
    visit_equipment(victim_equipment, |slot, enchantment| {
        visitor(PostAttackTarget::Victim, Some(slot), enchantment);
    });
    if let Some(weapon) = attacker_weapon {
        for enchantment in &weapon.active {
            if enchantment
                .matching_slots
                .contains(&EquipmentSlot::MainHand)
            {
                visitor(
                    PostAttackTarget::Attacker,
                    Some(EquipmentSlot::MainHand),
                    enchantment,
                );
            }
        }
    }
}

pub fn enchantments_compatible(
    first: &EnchantmentDefinition,
    second: &EnchantmentDefinition,
) -> bool {
    first.key != second.key
        && !first.exclusive_with.contains(&second.key)
        && !second.exclusive_with.contains(&first.key)
}

pub fn enchanting_cost(
    random: &mut dyn GameplayRandom,
    slot: usize,
    bookcases: i32,
    enchantable: Option<u32>,
) -> Result<i32, EnchantmentError> {
    if enchantable.is_none() {
        return Ok(0);
    }
    let bookcases = bookcases.min(15);
    let second_bound = bookcases
        .checked_add(1)
        .filter(|bound| *bound > 0)
        .ok_or(EnchantmentError::InvalidBookshelfCount(bookcases))?;
    let selected = checked_int(random, 8)? as i32
        + 1
        + (bookcases >> 1)
        + checked_int(random, second_bound as u32)? as i32;
    Ok(match slot {
        0 => (selected / 3).max(1),
        1 => selected * 2 / 3 + 1,
        _ => selected.max(bookcases * 2),
    })
}

pub fn offer_costs(
    random: &mut dyn GameplayRandom,
    bookcases: i32,
    enchantable: Option<u32>,
) -> Result<[i32; 3], EnchantmentError> {
    let mut costs = [0; 3];
    for (slot, cost) in costs.iter_mut().enumerate() {
        *cost = enchanting_cost(random, slot, bookcases, enchantable)?;
        if *cost < slot as i32 + 1 {
            *cost = 0;
        }
    }
    Ok(costs)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedEnchantment {
    pub key: ResourceId,
    pub level: u8,
}

pub fn select_enchantments(
    random: &mut dyn GameplayRandom,
    item: &EnchantedItem,
    enchantment_cost: i32,
    definitions: &[EnchantmentDefinition],
) -> Result<Vec<SelectedEnchantment>, EnchantmentError> {
    let Some(enchantable) = item.enchantable else {
        return Ok(Vec::new());
    };
    let adjustment_bound = enchantable / 4 + 1;
    let mut cost = enchantment_cost
        .saturating_add(1)
        .saturating_add(checked_int(random, adjustment_bound)? as i32)
        .saturating_add(checked_int(random, adjustment_bound)? as i32);
    let span = (checked_float(random)? + checked_float(random)? - 1.0) * 0.15;
    cost = ((cost as f32 * (1.0 + span)).round() as i32).max(1);
    let mut candidates = available_enchantments(cost, item, definitions);
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let first = weighted_select(random, &candidates, definitions)?;
    let mut selected = vec![first.clone()];
    while checked_int(random, 50)? as i32 <= cost {
        let last_definition = definition(
            selected.last().expect("selection is known to be nonempty"),
            definitions,
        )?;
        candidates.retain(|candidate| {
            definition(candidate, definitions).is_ok_and(|candidate_definition| {
                enchantments_compatible(last_definition, candidate_definition)
            })
        });
        if candidates.is_empty() {
            break;
        }
        selected.push(weighted_select(random, &candidates, definitions)?);
        cost /= 2;
    }
    if item.ordinary_book && selected.len() > 1 {
        let removed = checked_int(random, selected.len() as u32)? as usize;
        selected.remove(removed);
    }
    Ok(selected)
}

fn available_enchantments(
    cost: i32,
    item: &EnchantedItem,
    definitions: &[EnchantmentDefinition],
) -> Vec<SelectedEnchantment> {
    definitions
        .iter()
        .filter(|definition| {
            item.ordinary_book
                || item
                    .stack
                    .item
                    .as_ref()
                    .is_some_and(|item| definition.primary_items.contains(item))
        })
        .filter_map(|definition| {
            (definition.minimum_level..=definition.maximum_level)
                .rev()
                .find(|level| {
                    cost >= definition.minimum_cost.at_level(*level)
                        && cost <= definition.maximum_cost.at_level(*level)
                })
                .map(|level| SelectedEnchantment {
                    key: definition.key.clone(),
                    level,
                })
        })
        .collect()
}

fn weighted_select(
    random: &mut dyn GameplayRandom,
    candidates: &[SelectedEnchantment],
    definitions: &[EnchantmentDefinition],
) -> Result<SelectedEnchantment, EnchantmentError> {
    let total = candidates.iter().try_fold(0_u32, |total, candidate| {
        let weight = definition(candidate, definitions)?.weight;
        total
            .checked_add(weight)
            .ok_or(EnchantmentError::WeightOverflow)
    })?;
    if total == 0 {
        return Err(EnchantmentError::ZeroWeight);
    }
    let mut draw = checked_int(random, total)?;
    for candidate in candidates {
        let weight = definition(candidate, definitions)?.weight;
        if draw < weight {
            return Ok(candidate.clone());
        }
        draw -= weight;
    }
    Err(EnchantmentError::ZeroWeight)
}

fn definition<'a>(
    selected: &SelectedEnchantment,
    definitions: &'a [EnchantmentDefinition],
) -> Result<&'a EnchantmentDefinition, EnchantmentError> {
    definitions
        .iter()
        .find(|definition| definition.key == selected.key)
        .ok_or_else(|| EnchantmentError::MissingDefinition(selected.key.clone()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantingClue {
    pub selected: SelectedEnchantment,
    pub offer_slot: usize,
}

pub fn enchanting_clue(
    random: &mut dyn GameplayRandom,
    item: &EnchantedItem,
    slot: usize,
    cost: i32,
    definitions: &[EnchantmentDefinition],
) -> Result<Option<EnchantingClue>, EnchantmentError> {
    let selected = select_enchantments(random, item, cost, definitions)?;
    if selected.is_empty() {
        return Ok(None);
    }
    let selected_index = checked_int(random, selected.len() as u32)? as usize;
    Ok(Some(EnchantingClue {
        selected: selected[selected_index].clone(),
        offer_slot: slot,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantingPlayer {
    pub experience_levels: u32,
    pub enchantment_seed: i32,
    pub infinite_materials: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantCommit {
    pub admitted: bool,
    pub committed: bool,
    pub transmuted_book: bool,
    pub selected: Vec<SelectedEnchantment>,
    pub levels_spent: u32,
    pub lapis_spent: u32,
    pub award_stat: bool,
    pub trigger_criterion: bool,
    pub recompute_offers: bool,
    pub play_sound: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantOfferRequest {
    pub slot: usize,
    pub displayed_cost: i32,
    pub refreshed_seed: i32,
}

pub fn commit_enchanting_offer(
    item: &mut EnchantedItem,
    player: &mut EnchantingPlayer,
    lapis: &mut u32,
    request: EnchantOfferRequest,
    definitions: &[EnchantmentDefinition],
    random: &mut dyn GameplayRandom,
) -> Result<EnchantCommit, EnchantmentError> {
    if request.slot >= 3 {
        return Ok(empty_commit());
    }
    let resource_cost = request.slot as u32 + 1;
    let has_resources = player.infinite_materials
        || (*lapis >= resource_cost
            && player.experience_levels >= resource_cost
            && player.experience_levels >= request.displayed_cost.max(0) as u32);
    if request.displayed_cost <= 0 || item.stack.is_empty() || !has_resources {
        return Ok(empty_commit());
    }

    let selected = select_enchantments(random, item, request.displayed_cost, definitions)?;
    if selected.is_empty() {
        return Ok(EnchantCommit {
            admitted: true,
            ..empty_commit()
        });
    }
    if !player.infinite_materials {
        player.experience_levels -= resource_cost;
        *lapis -= resource_cost;
    }
    player.enchantment_seed = request.refreshed_seed;
    let transmuted_book = item.ordinary_book;
    if transmuted_book {
        item.stack.item = Some(minecraft("enchanted_book"));
        item.ordinary_book = false;
    }
    if transmuted_book {
        merge_selected(&mut item.stored, &selected, definitions)?;
    } else {
        merge_selected(&mut item.active, &selected, definitions)?;
    }
    Ok(EnchantCommit {
        admitted: true,
        committed: true,
        transmuted_book,
        selected,
        levels_spent: if player.infinite_materials {
            0
        } else {
            resource_cost
        },
        lapis_spent: if player.infinite_materials {
            0
        } else {
            resource_cost
        },
        award_stat: true,
        trigger_criterion: true,
        recompute_offers: true,
        play_sound: true,
    })
}

fn merge_selected(
    entries: &mut Vec<ActiveEnchantment>,
    selected: &[SelectedEnchantment],
    definitions: &[EnchantmentDefinition],
) -> Result<(), EnchantmentError> {
    for selection in selected {
        if let Some(existing) = entries.iter_mut().find(|entry| entry.key == selection.key) {
            existing.level = existing.level.max(selection.level);
            continue;
        }
        let definition = definition(selection, definitions)?;
        entries.push(ActiveEnchantment {
            key: selection.key.clone(),
            level: selection.level,
            matching_slots: definition.matching_slots.clone(),
            effects: definition.effects.clone(),
        });
    }
    Ok(())
}

fn empty_commit() -> EnchantCommit {
    EnchantCommit {
        admitted: false,
        committed: false,
        transmuted_book: false,
        selected: Vec::new(),
        levels_spent: 0,
        lapis_spent: 0,
        award_stat: false,
        trigger_criterion: false,
        recompute_offers: false,
        play_sound: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnchantmentError {
    Random(GameplayRandomError),
    InvalidBookshelfCount(i32),
    InvalidSlot(usize),
    MissingDefinition(ResourceId),
    WeightOverflow,
    ZeroWeight,
}

impl From<GameplayRandomError> for EnchantmentError {
    fn from(value: GameplayRandomError) -> Self {
        Self::Random(value)
    }
}

fn minecraft(path: &str) -> ResourceId {
    ResourceId::minecraft(path).expect("locked enchantment item identifier")
}
