use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_motion::packet::decode_rotation;
use crate::java_26_2::play::clientbound::entity_spawn::packet::{AddEntity, RemoveEntities};
use crate::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnAdmission {
    pub required_features_enabled: bool,
    pub allowed_in_peaceful: bool,
    pub factory_available: bool,
}

impl Default for SpawnAdmission {
    fn default() -> Self {
        Self {
            required_features_enabled: true,
            allowed_in_peaceful: true,
            factory_available: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentDirection {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl AttachmentDirection {
    fn from_data(data: i32) -> Self {
        match (data % 6).abs() {
            0 => Self::Down,
            1 => Self::Up,
            2 => Self::North,
            3 => Self::South,
            4 => Self::West,
            _ => Self::East,
        }
    }

    const fn horizontal(self) -> bool {
        matches!(self, Self::North | Self::South | Self::West | Self::East)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnSound {
    MinecartRolling,
    BeeNonaggressiveFlying,
    BeeAggressiveFlying,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnedEntityProjection {
    pub entity_id: i32,
    pub uuid: u128,
    pub entity_type: Identifier,
    pub position: Vector3,
    pub old_position: Vector3,
    pub packet_position_base: Vector3,
    pub motion: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub old_yaw: f32,
    pub old_pitch: f32,
    pub body_yaw: f32,
    pub old_body_yaw: f32,
    pub head_yaw: f32,
    pub old_head_yaw: f32,
    pub living: bool,
    pub player: bool,
    pub discarded: bool,
    pub owner_entity_id: Option<i32>,
    pub owner_uuid_seen_during_recreation: Option<u128>,
    pub attachment_direction: Option<AttachmentDirection>,
    pub block_state: Option<i32>,
    pub block_anchor: Option<(i32, i32, i32)>,
    pub emerging_warden: bool,
    pub dragon_part_ids: Vec<i32>,
    pub llama_spit_particle_multipliers: Vec<f32>,
    pub movement_reapplied_after_construction: bool,
    pub minecart_initial_motion: Option<Vector3>,
    pub post_add_sound: Option<SpawnSound>,
    pub uuid_registered: bool,
    pub vehicle: Option<i32>,
    pub passengers: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitySpawnAction {
    Ignored,
    SkippedConstruction,
    Inserted {
        replaced_same_id: bool,
        uuid_registered: bool,
    },
    Removed {
        count: usize,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntitySpawnClientProjection {
    entities: BTreeMap<i32, SpawnedEntityProjection>,
    uuid_to_id: BTreeMap<u128, i32>,
    player_info: BTreeSet<u128>,
    admissions: BTreeMap<Identifier, SpawnAdmission>,
    seen_players: BTreeSet<u128>,
    dragon_parts: BTreeSet<i32>,
    debug_subscriptions: BTreeSet<i32>,
    local_player_entity_id: Option<i32>,
    removed_player_vehicle_id: Option<i32>,
}

impl EntitySpawnClientProjection {
    pub const fn set_local_player_entity_id(&mut self, entity_id: i32) {
        self.local_player_entity_id = Some(entity_id);
    }

    pub fn add_player_info(&mut self, uuid: u128) {
        self.player_info.insert(uuid);
    }

    pub fn set_admission(&mut self, entity_type: Identifier, admission: SpawnAdmission) {
        self.admissions.insert(entity_type, admission);
    }

    pub fn mark_debug_subscription(&mut self, entity_id: i32) {
        self.debug_subscriptions.insert(entity_id);
    }

    pub fn set_passengers(&mut self, vehicle_id: i32, passenger_ids: Vec<i32>) {
        let old = self
            .entities
            .get(&vehicle_id)
            .map_or_else(Vec::new, |entity| entity.passengers.clone());
        for passenger_id in old {
            if let Some(passenger) = self.entities.get_mut(&passenger_id)
                && passenger.vehicle == Some(vehicle_id)
            {
                passenger.vehicle = None;
            }
        }
        if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
            vehicle.passengers = passenger_ids.clone();
        }
        for passenger_id in passenger_ids {
            if let Some(passenger) = self.entities.get_mut(&passenger_id) {
                passenger.vehicle = Some(vehicle_id);
            }
        }
    }

    #[must_use]
    pub fn entity(&self, entity_id: i32) -> Option<&SpawnedEntityProjection> {
        self.entities.get(&entity_id)
    }

    #[must_use]
    pub fn entity_id_by_uuid(&self, uuid: u128) -> Option<i32> {
        self.uuid_to_id.get(&uuid).copied()
    }

    #[must_use]
    pub fn seen_player(&self, uuid: u128) -> bool {
        self.seen_players.contains(&uuid)
    }

    #[must_use]
    pub const fn removed_player_vehicle_id(&self) -> Option<i32> {
        self.removed_player_vehicle_id
    }

    #[must_use]
    pub fn has_debug_subscription(&self, entity_id: i32) -> bool {
        self.debug_subscriptions.contains(&entity_id)
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
    ) -> Result<EntitySpawnAction, EntitySpawnProjectionError> {
        match packet {
            PlayClientboundPacket::AddEntity(packet) => self.apply_add(packet),
            PlayClientboundPacket::RemoveEntities(packet) => Ok(self.apply_remove(packet)),
            _ => Ok(EntitySpawnAction::Ignored),
        }
    }

    fn apply_add(
        &mut self,
        packet: &AddEntity,
    ) -> Result<EntitySpawnAction, EntitySpawnProjectionError> {
        if self.removed_player_vehicle_id == Some(packet.entity_id) {
            self.removed_player_vehicle_id = None;
        }
        let path = packet.entity_type.path();
        if path == "player" && !self.player_info.contains(&packet.uuid) {
            return Ok(EntitySpawnAction::SkippedConstruction);
        }
        let admission = self
            .admissions
            .get(&packet.entity_type)
            .copied()
            .unwrap_or_default();
        if path != "player"
            && (!admission.required_features_enabled
                || !admission.allowed_in_peaceful
                || !admission.factory_available)
        {
            return Ok(EntitySpawnAction::SkippedConstruction);
        }

        let mut entity = self.recreate(packet)?;
        let replaced_same_id = self.entities.contains_key(&packet.entity_id);
        if replaced_same_id {
            self.discard_without_vehicle_retention(packet.entity_id);
        }
        let uuid_registered = match self.uuid_to_id.get(&packet.uuid) {
            Some(existing) if *existing != packet.entity_id => false,
            _ => {
                self.uuid_to_id.insert(packet.uuid, packet.entity_id);
                true
            }
        };
        entity.uuid_registered = uuid_registered;
        for part_id in &entity.dragon_part_ids {
            self.dragon_parts.insert(*part_id);
        }
        if entity.player {
            self.seen_players.insert(entity.uuid);
        }
        self.entities.insert(packet.entity_id, entity);
        Ok(EntitySpawnAction::Inserted {
            replaced_same_id,
            uuid_registered,
        })
    }

    fn recreate(
        &self,
        packet: &AddEntity,
    ) -> Result<SpawnedEntityProjection, EntitySpawnProjectionError> {
        let path = packet.entity_type.path();
        let living = is_living(path);
        let player = path == "player";
        let pitch = decode_rotation(packet.pitch);
        let yaw = decode_rotation(packet.yaw);
        let head_yaw = decode_rotation(packet.head_yaw);
        let position = if living {
            Vector3 {
                x: packet.position.x.clamp(-30_000_000.0, 30_000_000.0),
                y: packet.position.y,
                z: packet.position.z.clamp(-30_000_000.0, 30_000_000.0),
            }
        } else {
            packet.position
        };
        let pitch = if living {
            pitch.clamp(-90.0, 90.0)
        } else {
            pitch
        };
        let mut entity = SpawnedEntityProjection {
            entity_id: packet.entity_id,
            uuid: packet.uuid,
            entity_type: packet.entity_type.clone(),
            position,
            old_position: if player { position } else { Vector3::default() },
            packet_position_base: position,
            motion: packet.motion,
            yaw,
            pitch,
            old_yaw: if player { yaw } else { 0.0 },
            old_pitch: if player { pitch } else { 0.0 },
            body_yaw: if living { head_yaw } else { 0.0 },
            old_body_yaw: if living { head_yaw } else { 0.0 },
            head_yaw: if living { head_yaw } else { 0.0 },
            old_head_yaw: if living { head_yaw } else { 0.0 },
            living,
            player,
            discarded: false,
            owner_entity_id: None,
            owner_uuid_seen_during_recreation: None,
            attachment_direction: None,
            block_state: None,
            block_anchor: None,
            emerging_warden: false,
            dragon_part_ids: Vec::new(),
            llama_spit_particle_multipliers: Vec::new(),
            movement_reapplied_after_construction: false,
            minecart_initial_motion: None,
            post_add_sound: None,
            uuid_registered: false,
            vehicle: None,
            passengers: Vec::new(),
        };
        self.apply_spawn_data(&mut entity, packet.data)?;
        self.apply_specializations(&mut entity);
        Ok(entity)
    }

    fn apply_spawn_data(
        &self,
        entity: &mut SpawnedEntityProjection,
        data: i32,
    ) -> Result<(), EntitySpawnProjectionError> {
        let path = entity.entity_type.path();
        if matches!(path, "item_frame" | "glow_item_frame") {
            entity.attachment_direction = Some(AttachmentDirection::from_data(data));
            entity.block_anchor = Some(block_anchor(entity.position));
        } else if path == "painting" {
            let direction = AttachmentDirection::from_data(data);
            if !direction.horizontal() {
                return Err(EntitySpawnProjectionError::VerticalPaintingDirection { direction });
            }
            entity.attachment_direction = Some(direction);
            entity.block_anchor = Some(block_anchor(entity.position));
        } else if path == "falling_block" {
            entity.block_state = Some(if (0..=32_365).contains(&data) {
                data
            } else {
                0
            });
            entity.block_anchor = Some(block_anchor(entity.position));
        } else if path == "leash_knot" {
            entity.block_anchor = Some(block_anchor(entity.position));
        } else if path == "warden" {
            entity.emerging_warden = data == 1;
        }
        if is_projectile(path)
            && let Some(owner) = self.entities.get(&data)
        {
            entity.owner_entity_id = Some(data);
            entity.owner_uuid_seen_during_recreation = Some(owner.uuid);
        }
        if path == "fishing_bobber"
            && !entity
                .owner_entity_id
                .and_then(|owner_id| self.entities.get(&owner_id))
                .is_some_and(|owner| owner.player)
        {
            entity.discarded = true;
        }
        Ok(())
    }

    fn apply_specializations(&self, entity: &mut SpawnedEntityProjection) {
        let path = entity.entity_type.path();
        if path == "ender_dragon" {
            entity.dragon_part_ids = (1..=8)
                .map(|offset| entity.entity_id.wrapping_add(offset))
                .collect();
        }
        if path == "shulker" {
            entity.body_yaw = 0.0;
            entity.old_body_yaw = 0.0;
        }
        if path == "llama_spit" {
            entity.llama_spit_particle_multipliers =
                (4..=10).map(|value| value as f32 / 10.0).collect();
        }
        if matches!(path, "llama_spit" | "shulker_bullet") {
            entity.movement_reapplied_after_construction = true;
        }
        if is_minecart(path) {
            entity.minecart_initial_motion = Some(entity.motion);
            entity.post_add_sound = Some(SpawnSound::MinecartRolling);
        } else if path == "bee" {
            entity.post_add_sound = Some(SpawnSound::BeeNonaggressiveFlying);
        }
    }

    fn apply_remove(&mut self, packet: &RemoveEntities) -> EntitySpawnAction {
        let mut count = 0;
        for entity_id in &packet.entity_ids {
            if !self.entities.contains_key(entity_id) {
                continue;
            }
            if self
                .local_player_entity_id
                .is_some_and(|local| self.indirectly_carries(*entity_id, local))
            {
                self.removed_player_vehicle_id = Some(*entity_id);
            }
            self.discard_without_vehicle_retention(*entity_id);
            self.debug_subscriptions.remove(entity_id);
            count += 1;
        }
        EntitySpawnAction::Removed { count }
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
            for passenger in &entity.passengers {
                if *passenger == target {
                    return true;
                }
                pending.push(*passenger);
            }
        }
        false
    }

    fn discard_without_vehicle_retention(&mut self, entity_id: i32) {
        let Some(entity) = self.entities.remove(&entity_id) else {
            return;
        };
        if let Some(vehicle_id) = entity.vehicle
            && let Some(vehicle) = self.entities.get_mut(&vehicle_id)
        {
            vehicle
                .passengers
                .retain(|passenger| *passenger != entity_id);
        }
        for passenger_id in &entity.passengers {
            if let Some(passenger) = self.entities.get_mut(passenger_id)
                && passenger.vehicle == Some(entity_id)
            {
                passenger.vehicle = None;
            }
        }
        if self.uuid_to_id.get(&entity.uuid) == Some(&entity_id) {
            self.uuid_to_id.remove(&entity.uuid);
        }
        for part_id in entity.dragon_part_ids {
            self.dragon_parts.remove(&part_id);
        }
        self.debug_subscriptions.remove(&entity_id);
    }
}

fn is_living(path: &str) -> bool {
    !is_nonliving(path)
}

fn is_nonliving(path: &str) -> bool {
    path.ends_with("_boat")
        || path.ends_with("_raft")
        || is_minecart(path)
        || is_projectile(path)
        || matches!(
            path,
            "area_effect_cloud"
                | "block_display"
                | "end_crystal"
                | "experience_orb"
                | "falling_block"
                | "interaction"
                | "item"
                | "item_display"
                | "item_frame"
                | "glow_item_frame"
                | "leash_knot"
                | "lightning_bolt"
                | "marker"
                | "ominous_item_spawner"
                | "painting"
                | "text_display"
                | "tnt"
        )
}

fn is_projectile(path: &str) -> bool {
    matches!(
        path,
        "arrow"
            | "breeze_wind_charge"
            | "dragon_fireball"
            | "egg"
            | "ender_pearl"
            | "experience_bottle"
            | "fireball"
            | "firework_rocket"
            | "fishing_bobber"
            | "lingering_potion"
            | "llama_spit"
            | "shulker_bullet"
            | "small_fireball"
            | "snowball"
            | "spectral_arrow"
            | "splash_potion"
            | "trident"
            | "wind_charge"
            | "wither_skull"
    )
}

fn is_minecart(path: &str) -> bool {
    path == "minecart" || path.ends_with("_minecart")
}

fn block_anchor(position: Vector3) -> (i32, i32, i32) {
    (
        position.x.floor() as i32,
        position.y.floor() as i32,
        position.z.floor() as i32,
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EntitySpawnProjectionError {
    #[error("painting attachment direction {direction:?} is vertical")]
    VerticalPaintingDirection { direction: AttachmentDirection },
}
