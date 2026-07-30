//! Client-side ordered entity-state replacement projection.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_state::accessor_registry::{
    MetadataAccessorSchemaError, schema_for_hierarchy,
};
use crate::java_26_2::play::clientbound::entity_state::metadata::{
    MetadataEntry, MetadataSerializer, MetadataValue,
};
use crate::java_26_2::play::clientbound::entity_state::packet::{
    AttributeModifier, AttributeSnapshot, EquipmentSlot, SetEntityData, SetEntityLink,
    SetEquipment, SetPassengers, UpdateAttributes,
};
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntityStateCapabilities {
    pub living: bool,
    pub leashable: bool,
    pub boat: bool,
    pub local_player: bool,
    pub riding_allowed: bool,
}

impl EntityStateCapabilities {
    #[must_use]
    pub const fn living() -> Self {
        Self {
            living: true,
            riding_allowed: true,
            leashable: false,
            boat: false,
            local_player: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataSlotProjection {
    pub serializer: MetadataSerializer,
    pub default: MetadataValue,
    pub current: MetadataValue,
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeInstanceProjection {
    pub base: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub client_syncable: bool,
    pub modifiers: BTreeMap<Identifier, AttributeModifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityStateProjection {
    pub capabilities: EntityStateCapabilities,
    pub yaw: f32,
    pub old_yaw: f32,
    pub head_yaw: f32,
    pub metadata: BTreeMap<u8, MetadataSlotProjection>,
    pub metadata_callback_log: Vec<u8>,
    pub metadata_aggregate_callback_log: Vec<Vec<u8>>,
    pub metadata_pairing_snapshot: Vec<MetadataEntry>,
    pub attributes: BTreeMap<Identifier, AttributeInstanceProjection>,
    pub attributes_to_sync: BTreeSet<Identifier>,
    pub equipment: BTreeMap<EquipmentSlot, ItemStack>,
    pub passengers: Vec<i32>,
    pub vehicle: Option<i32>,
    pub delayed_leash_holder_id: Option<i32>,
}

impl EntityStateProjection {
    #[must_use]
    pub fn new(capabilities: EntityStateCapabilities) -> Self {
        Self {
            capabilities,
            yaw: 0.0,
            old_yaw: 0.0,
            head_yaw: 0.0,
            metadata: BTreeMap::new(),
            metadata_callback_log: Vec::new(),
            metadata_aggregate_callback_log: Vec::new(),
            metadata_pairing_snapshot: Vec::new(),
            attributes: BTreeMap::new(),
            attributes_to_sync: BTreeSet::new(),
            equipment: BTreeMap::new(),
            passengers: Vec::new(),
            vehicle: None,
            delayed_leash_holder_id: None,
        }
    }

    pub fn define_metadata(&mut self, slot: u8, default: MetadataValue) {
        self.metadata.insert(
            slot,
            MetadataSlotProjection {
                serializer: default.serializer(),
                current: default.clone(),
                default,
                dirty: false,
            },
        );
    }

    pub fn install_locked_metadata_hierarchy(
        &mut self,
        declaring_classes: &[&str],
        mut defaults: BTreeMap<u8, MetadataValue>,
    ) -> Result<(), EntityStateProjectionError> {
        let schema = schema_for_hierarchy(declaring_classes)?;
        for (slot, serializer) in schema {
            let default = defaults
                .remove(&slot)
                .ok_or(EntityStateProjectionError::MissingMetadataDefault { slot, serializer })?;
            let actual = default.serializer();
            if actual != serializer {
                return Err(EntityStateProjectionError::MetadataDefaultMismatch {
                    slot,
                    expected: serializer,
                    actual,
                });
            }
            self.define_metadata(slot, default);
        }
        if let Some((slot, _)) = defaults.pop_first() {
            return Err(EntityStateProjectionError::UnexpectedMetadataDefault { slot });
        }
        Ok(())
    }

    pub fn define_attribute(
        &mut self,
        identity: Identifier,
        base: f64,
        minimum: f64,
        maximum: f64,
        client_syncable: bool,
    ) {
        self.attributes.insert(
            identity,
            AttributeInstanceProjection {
                base,
                minimum,
                maximum,
                client_syncable,
                modifiers: BTreeMap::new(),
            },
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityStateAction {
    Ignored,
    MetadataApplied { entries: usize },
    AttributesApplied { snapshots: usize, skipped: usize },
    EquipmentApplied { entries: usize },
    PassengersApplied { passengers: usize },
    LinkApplied,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntityStateClientProjection {
    entities: BTreeMap<i32, EntityStateProjection>,
    removed_player_vehicle_id: Option<i32>,
    riding_onboarding_shown: bool,
    riding_onboarding_presentations: usize,
}

impl EntityStateClientProjection {
    pub fn insert_entity(&mut self, entity_id: i32, entity: EntityStateProjection) {
        self.entities.insert(entity_id, entity);
    }

    #[must_use]
    pub fn entity(&self, entity_id: i32) -> Option<&EntityStateProjection> {
        self.entities.get(&entity_id)
    }

    #[must_use]
    pub fn entity_mut(&mut self, entity_id: i32) -> Option<&mut EntityStateProjection> {
        self.entities.get_mut(&entity_id)
    }

    pub const fn set_removed_player_vehicle_id(&mut self, entity_id: Option<i32>) {
        self.removed_player_vehicle_id = entity_id;
    }

    #[must_use]
    pub const fn removed_player_vehicle_id(&self) -> Option<i32> {
        self.removed_player_vehicle_id
    }

    #[must_use]
    pub const fn riding_onboarding_presentations(&self) -> usize {
        self.riding_onboarding_presentations
    }

    #[must_use]
    pub fn resolved_leash_holder(&self, source_entity_id: i32) -> Option<i32> {
        let holder = self
            .entities
            .get(&source_entity_id)?
            .delayed_leash_holder_id?;
        self.entities.contains_key(&holder).then_some(holder)
    }

    pub fn set_local_metadata(
        &mut self,
        entity_id: i32,
        slot: u8,
        value: MetadataValue,
    ) -> Result<(), EntityStateProjectionError> {
        let entity = self
            .entities
            .get_mut(&entity_id)
            .ok_or(EntityStateProjectionError::MissingEntity { entity_id })?;
        let item = entity
            .metadata
            .get_mut(&slot)
            .ok_or(EntityStateProjectionError::MissingMetadataSlot { entity_id, slot })?;
        let actual = value.serializer();
        if item.serializer != actual {
            return Err(EntityStateProjectionError::MetadataSerializerMismatch {
                entity_id,
                slot,
                expected: item.serializer,
                actual,
            });
        }
        item.current = value;
        item.dirty = true;
        Ok(())
    }

    pub fn mark_attribute_to_sync(
        &mut self,
        entity_id: i32,
        attribute: Identifier,
    ) -> Result<(), EntityStateProjectionError> {
        let entity = self
            .entities
            .get_mut(&entity_id)
            .ok_or(EntityStateProjectionError::MissingEntity { entity_id })?;
        if !entity.attributes.contains_key(&attribute) {
            return Err(EntityStateProjectionError::MissingAttribute {
                entity_id,
                attribute,
            });
        }
        entity.attributes_to_sync.insert(attribute);
        Ok(())
    }

    #[must_use]
    pub fn metadata_pairing_values(&self, entity_id: i32) -> Vec<MetadataEntry> {
        self.entities
            .get(&entity_id)
            .map_or_else(Vec::new, nondefault_metadata)
    }

    #[must_use]
    pub fn pack_dirty_metadata(&mut self, entity_id: i32) -> Vec<MetadataEntry> {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return Vec::new();
        };
        let mut dirty = Vec::new();
        for (slot, item) in &mut entity.metadata {
            if item.dirty {
                item.dirty = false;
                dirty.push(MetadataEntry {
                    slot: *slot,
                    serializer: item.serializer,
                    value: item.current.clone(),
                });
            }
        }
        entity.metadata_pairing_snapshot = nondefault_metadata(entity);
        dirty
    }

    #[must_use]
    pub fn syncable_attributes(&self, entity_id: i32) -> Vec<AttributeSnapshot> {
        let Some(entity) = self.entities.get(&entity_id) else {
            return Vec::new();
        };
        entity
            .attributes
            .iter()
            .filter(|(_, instance)| instance.client_syncable)
            .map(|(identity, instance)| snapshot(identity, instance))
            .collect()
    }

    #[must_use]
    pub fn take_attributes_to_sync(&mut self, entity_id: i32) -> Vec<AttributeSnapshot> {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return Vec::new();
        };
        let identities = std::mem::take(&mut entity.attributes_to_sync);
        identities
            .into_iter()
            .filter_map(|identity| {
                entity
                    .attributes
                    .get(&identity)
                    .map(|instance| snapshot(&identity, instance))
            })
            .collect()
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
    ) -> Result<EntityStateAction, EntityStateProjectionError> {
        match packet {
            PlayClientboundPacket::SetEntityData(packet) => self.apply_metadata(packet),
            PlayClientboundPacket::UpdateAttributes(packet) => self.apply_attributes(packet),
            PlayClientboundPacket::SetEquipment(packet) => Ok(self.apply_equipment(packet)),
            PlayClientboundPacket::SetPassengers(packet) => Ok(self.apply_passengers(packet)),
            PlayClientboundPacket::SetEntityLink(packet) => Ok(self.apply_link(*packet)),
            _ => Ok(EntityStateAction::Ignored),
        }
    }

    fn apply_metadata(
        &mut self,
        packet: &SetEntityData,
    ) -> Result<EntityStateAction, EntityStateProjectionError> {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return Ok(EntityStateAction::Ignored);
        };
        let mut callback_batch = Vec::with_capacity(packet.values.len());
        for entry in &packet.values {
            let item = entity.metadata.get_mut(&entry.slot).ok_or(
                EntityStateProjectionError::MissingMetadataSlot {
                    entity_id: packet.entity_id,
                    slot: entry.slot,
                },
            )?;
            if item.serializer != entry.serializer {
                return Err(EntityStateProjectionError::MetadataSerializerMismatch {
                    entity_id: packet.entity_id,
                    slot: entry.slot,
                    expected: item.serializer,
                    actual: entry.serializer,
                });
            }
            item.current = entry.value.clone();
            entity.metadata_callback_log.push(entry.slot);
            callback_batch.push(entry.slot);
        }
        entity
            .metadata_aggregate_callback_log
            .push(callback_batch.clone());
        Ok(EntityStateAction::MetadataApplied {
            entries: callback_batch.len(),
        })
    }

    fn apply_attributes(
        &mut self,
        packet: &UpdateAttributes,
    ) -> Result<EntityStateAction, EntityStateProjectionError> {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return Ok(EntityStateAction::Ignored);
        };
        if !entity.capabilities.living {
            return Err(EntityStateProjectionError::AttributesRequireLiving {
                entity_id: packet.entity_id,
            });
        }
        let mut applied = 0;
        let mut skipped = 0;
        for wire in &packet.snapshots {
            let Some(instance) = entity.attributes.get_mut(&wire.attribute) else {
                skipped += 1;
                continue;
            };
            instance.base = wire.base.clamp(instance.minimum, instance.maximum);
            instance.modifiers.clear();
            for modifier in &wire.modifiers {
                if instance.modifiers.contains_key(&modifier.identity) {
                    return Err(EntityStateProjectionError::DuplicateAttributeModifier {
                        entity_id: packet.entity_id,
                        attribute: wire.attribute.clone(),
                        modifier: modifier.identity.clone(),
                    });
                }
                instance
                    .modifiers
                    .insert(modifier.identity.clone(), modifier.clone());
            }
            applied += 1;
        }
        Ok(EntityStateAction::AttributesApplied {
            snapshots: applied,
            skipped,
        })
    }

    fn apply_equipment(&mut self, packet: &SetEquipment) -> EntityStateAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityStateAction::Ignored;
        };
        if !entity.capabilities.living {
            return EntityStateAction::Ignored;
        }
        for entry in &packet.entries {
            entity.equipment.insert(entry.slot, entry.stack.clone());
        }
        EntityStateAction::EquipmentApplied {
            entries: packet.entries.len(),
        }
    }

    fn apply_passengers(&mut self, packet: &SetPassengers) -> EntityStateAction {
        if !self.entities.contains_key(&packet.vehicle_id) {
            return EntityStateAction::Ignored;
        }
        let local_player = self
            .entities
            .iter()
            .find_map(|(id, entity)| entity.capabilities.local_player.then_some(*id));
        let carried_before =
            local_player.is_some_and(|local| self.indirectly_carries(packet.vehicle_id, local));
        self.eject_passengers(packet.vehicle_id);

        let mut added = 0;
        for passenger_id in &packet.passenger_ids {
            if self.start_riding(*passenger_id, packet.vehicle_id) {
                added += 1;
                if Some(*passenger_id) == local_player {
                    self.removed_player_vehicle_id = None;
                    if !carried_before
                        && self
                            .entities
                            .get(&packet.vehicle_id)
                            .is_some_and(|vehicle| vehicle.capabilities.boat)
                    {
                        self.copy_boat_rotation(packet.vehicle_id, *passenger_id);
                        if !self.riding_onboarding_shown {
                            self.riding_onboarding_shown = true;
                            self.riding_onboarding_presentations += 1;
                        }
                    }
                }
            }
        }
        EntityStateAction::PassengersApplied { passengers: added }
    }

    fn apply_link(&mut self, packet: SetEntityLink) -> EntityStateAction {
        let Some(source) = self.entities.get_mut(&packet.source_entity_id) else {
            return EntityStateAction::Ignored;
        };
        if !source.capabilities.leashable {
            return EntityStateAction::Ignored;
        }
        source.delayed_leash_holder_id =
            (packet.destination_entity_id != 0).then_some(packet.destination_entity_id);
        EntityStateAction::LinkApplied
    }

    fn eject_passengers(&mut self, vehicle_id: i32) {
        let passengers = self
            .entities
            .get_mut(&vehicle_id)
            .map_or_else(Vec::new, |vehicle| std::mem::take(&mut vehicle.passengers));
        for passenger_id in passengers {
            if let Some(passenger) = self.entities.get_mut(&passenger_id)
                && passenger.vehicle == Some(vehicle_id)
            {
                passenger.vehicle = None;
            }
        }
    }

    fn start_riding(&mut self, passenger_id: i32, vehicle_id: i32) -> bool {
        let Some(passenger) = self.entities.get(&passenger_id) else {
            return false;
        };
        if passenger_id == vehicle_id
            || !passenger.capabilities.riding_allowed
            || passenger.vehicle == Some(vehicle_id)
            || self.indirectly_carries(passenger_id, vehicle_id)
        {
            return false;
        }
        if let Some(old_vehicle_id) = passenger.vehicle
            && let Some(old_vehicle) = self.entities.get_mut(&old_vehicle_id)
        {
            old_vehicle
                .passengers
                .retain(|existing| *existing != passenger_id);
        }
        self.entities
            .get_mut(&passenger_id)
            .expect("passenger presence checked")
            .vehicle = Some(vehicle_id);
        self.entities
            .get_mut(&vehicle_id)
            .expect("vehicle presence checked")
            .passengers
            .push(passenger_id);
        true
    }

    fn indirectly_carries(&self, root: i32, target: i32) -> bool {
        let mut pending = vec![root];
        let mut seen = BTreeSet::new();
        while let Some(entity_id) = pending.pop() {
            if !seen.insert(entity_id) {
                continue;
            }
            let Some(entity) = self.entities.get(&entity_id) else {
                continue;
            };
            for passenger_id in &entity.passengers {
                if *passenger_id == target {
                    return true;
                }
                pending.push(*passenger_id);
            }
        }
        false
    }

    fn copy_boat_rotation(&mut self, boat_id: i32, player_id: i32) {
        let yaw = self.entities.get(&boat_id).map_or(0.0, |boat| boat.yaw);
        if let Some(player) = self.entities.get_mut(&player_id) {
            player.yaw = yaw;
            player.old_yaw = yaw;
            player.head_yaw = yaw;
        }
    }
}

fn nondefault_metadata(entity: &EntityStateProjection) -> Vec<MetadataEntry> {
    entity
        .metadata
        .iter()
        .filter(|(_, item)| item.current != item.default)
        .map(|(slot, item)| MetadataEntry {
            slot: *slot,
            serializer: item.serializer,
            value: item.current.clone(),
        })
        .collect()
}

fn snapshot(identity: &Identifier, instance: &AttributeInstanceProjection) -> AttributeSnapshot {
    AttributeSnapshot {
        attribute: identity.clone(),
        base: instance.base,
        modifiers: instance.modifiers.values().cloned().collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EntityStateProjectionError {
    #[error(transparent)]
    AccessorSchema(#[from] MetadataAccessorSchemaError),
    #[error("entity {entity_id} is absent")]
    MissingEntity { entity_id: i32 },
    #[error("entity {entity_id} has no metadata slot {slot}")]
    MissingMetadataSlot { entity_id: i32, slot: u8 },
    #[error("locked metadata schema requires slot {slot} default with serializer {serializer:?}")]
    MissingMetadataDefault {
        slot: u8,
        serializer: MetadataSerializer,
    },
    #[error(
        "locked metadata slot {slot} expects default serializer {expected:?}, received {actual:?}"
    )]
    MetadataDefaultMismatch {
        slot: u8,
        expected: MetadataSerializer,
        actual: MetadataSerializer,
    },
    #[error("metadata default slot {slot} is absent from the selected locked hierarchy")]
    UnexpectedMetadataDefault { slot: u8 },
    #[error("entity {entity_id} metadata slot {slot} expects {expected:?}, received {actual:?}")]
    MetadataSerializerMismatch {
        entity_id: i32,
        slot: u8,
        expected: MetadataSerializer,
        actual: MetadataSerializer,
    },
    #[error("entity {entity_id} is nonliving and cannot accept attribute snapshots")]
    AttributesRequireLiving { entity_id: i32 },
    #[error("entity {entity_id} has no attribute {attribute}")]
    MissingAttribute {
        entity_id: i32,
        attribute: Identifier,
    },
    #[error("entity {entity_id} attribute {attribute} repeats transient modifier {modifier}")]
    DuplicateAttributeModifier {
        entity_id: i32,
        attribute: Identifier,
        modifier: Identifier,
    },
}
