//! Server publication audience and ordering for mutable entity state.

use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::entity_state::packet::{EquipmentEntry, EquipmentSlot};
use crate::java_26_2::play::item::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityStateAudience {
    TrackingPlayers,
    TrackingPlayersAndSelf,
}

pub const METADATA_AUDIENCE: EntityStateAudience = EntityStateAudience::TrackingPlayersAndSelf;
pub const ATTRIBUTE_AUDIENCE: EntityStateAudience = EntityStateAudience::TrackingPlayersAndSelf;
pub const EQUIPMENT_AUDIENCE: EntityStateAudience = EntityStateAudience::TrackingPlayers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityStatePairingStep {
    UpdateDataBeforeSync,
    NondefaultMetadata,
    SyncableAttributes,
    NonemptyEquipment,
    OwnPassengers,
    VehiclePassengers,
    Leash,
}

pub const ENTITY_STATE_PAIRING_ORDER: [EntityStatePairingStep; 7] = [
    EntityStatePairingStep::UpdateDataBeforeSync,
    EntityStatePairingStep::NondefaultMetadata,
    EntityStatePairingStep::SyncableAttributes,
    EntityStatePairingStep::NonemptyEquipment,
    EntityStatePairingStep::OwnPassengers,
    EntityStatePairingStep::VehiclePassengers,
    EntityStatePairingStep::Leash,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentPublication {
    pub hand_swap_event: bool,
    pub changes: Vec<EquipmentEntry>,
}

#[must_use]
pub fn collect_equipment_changes(
    current: &BTreeMap<EquipmentSlot, ItemStack>,
    remembered: &mut BTreeMap<EquipmentSlot, ItemStack>,
) -> EquipmentPublication {
    let old_main = stack_at(remembered, EquipmentSlot::Mainhand);
    let old_off = stack_at(remembered, EquipmentSlot::Offhand);
    let new_main = stack_at(current, EquipmentSlot::Mainhand);
    let new_off = stack_at(current, EquipmentSlot::Offhand);
    let hand_swap_event =
        old_main == new_off && old_off == new_main && (old_main != new_main || old_off != new_off);

    let mut changes = Vec::new();
    for ordinal in 0..=7 {
        let slot = EquipmentSlot::from_ordinal(ordinal).expect("locked ordinal is valid");
        let new = stack_at(current, slot);
        let old = stack_at(remembered, slot);
        if new != old && !(hand_swap_event && ordinal <= 1) {
            changes.push(EquipmentEntry {
                slot,
                stack: new.clone(),
            });
        }
        remembered.insert(slot, new.clone());
    }
    EquipmentPublication {
        hand_swap_event,
        changes,
    }
}

#[must_use]
pub fn pairing_equipment(equipment: &BTreeMap<EquipmentSlot, ItemStack>) -> Vec<EquipmentEntry> {
    (0..=7)
        .filter_map(|ordinal| {
            let slot = EquipmentSlot::from_ordinal(ordinal).expect("locked ordinal is valid");
            let stack = stack_at(equipment, slot);
            (!stack.is_empty()).then(|| EquipmentEntry {
                slot,
                stack: stack.clone(),
            })
        })
        .collect()
}

#[must_use]
pub fn passenger_tracker_receives_broadcast(
    viewer_entity_id: i32,
    old_passengers: &[i32],
    new_passengers: &[i32],
) -> bool {
    old_passengers.contains(&viewer_entity_id) == new_passengers.contains(&viewer_entity_id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiderTransitionStep {
    PositionRider,
    PlayerPositionChallenge,
    LivingVehicleEffects,
    FullPassengerList,
}

pub const RIDER_START_ORDER: [RiderTransitionStep; 4] = [
    RiderTransitionStep::PositionRider,
    RiderTransitionStep::PlayerPositionChallenge,
    RiderTransitionStep::LivingVehicleEffects,
    RiderTransitionStep::FullPassengerList,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeashPublicationStep {
    MutateRelation,
    BroadcastLink,
}

pub const LEASH_PUBLICATION_ORDER: [LeashPublicationStep; 2] = [
    LeashPublicationStep::MutateRelation,
    LeashPublicationStep::BroadcastLink,
];

fn stack_at(equipment: &BTreeMap<EquipmentSlot, ItemStack>, slot: EquipmentSlot) -> &ItemStack {
    static EMPTY: ItemStack = ItemStack::Empty;
    equipment.get(&slot).unwrap_or(&EMPTY)
}
