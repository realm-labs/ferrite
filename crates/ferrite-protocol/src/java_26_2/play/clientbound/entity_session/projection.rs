use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_session::packet::{
    Animate, DamageEvent, HurtAnimation, SetCamera, TakeItemEntity,
};
use crate::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, PlayClientboundPacket, Vector3,
};
use crate::java_26_2::play::clientbound::session::Respawn;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEntityKind {
    Generic,
    Living,
    Player,
    Item,
    ExperienceOrb,
}

impl SessionEntityKind {
    const fn living(self) -> bool {
        matches!(self, Self::Living | Self::Player)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamagePresentation {
    pub damage_type: Identifier,
    pub source_position: Option<Vector3>,
    pub cause_resolved: bool,
    pub direct_resolved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionEntityProjection {
    pub kind: SessionEntityKind,
    pub removed: bool,
    pub item_count: i32,
    pub main_hand_swings: u32,
    pub off_hand_swings: u32,
    pub wakes: u32,
    pub critical_particles: u32,
    pub enchanted_particles: u32,
    pub hurt_yaw: Option<f32>,
    pub walk_animation_speed: f32,
    pub invulnerable_time: i32,
    pub hurt_time: i32,
    pub hurt_duration: i32,
    pub last_damage: Option<DamagePresentation>,
    pub last_damage_game_time: Option<i64>,
    pub pickup_particles: u32,
}

impl SessionEntityProjection {
    #[must_use]
    pub fn new(kind: SessionEntityKind) -> Self {
        Self {
            kind,
            removed: false,
            item_count: 0,
            main_hand_swings: 0,
            off_hand_swings: 0,
            wakes: 0,
            critical_particles: 0,
            enchanted_particles: 0,
            hurt_yaw: None,
            walk_animation_speed: 0.0,
            invulnerable_time: 0,
            hurt_time: 0,
            hurt_duration: 0,
            last_damage: None,
            last_damage_game_time: None,
            pickup_particles: 0,
        }
    }

    #[must_use]
    pub const fn item(count: i32) -> Self {
        Self {
            kind: SessionEntityKind::Item,
            removed: false,
            item_count: count,
            main_hand_swings: 0,
            off_hand_swings: 0,
            wakes: 0,
            critical_particles: 0,
            enchanted_particles: 0,
            hurt_yaw: None,
            walk_animation_speed: 0.0,
            invulnerable_time: 0,
            hurt_time: 0,
            hurt_duration: 0,
            last_damage: None,
            last_damage_game_time: None,
            pickup_particles: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedAttribute {
    pub base: f64,
    pub value: f64,
    pub modifiers: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RespawnPlayerProjection {
    pub entity_id: i32,
    pub position: Vector3,
    pub motion: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub last_input: u8,
    pub sprinting: bool,
    pub nondefault_entity_data: BTreeMap<u8, i32>,
    pub attributes: BTreeMap<Identifier, ProjectedAttribute>,
    pub stats_generation: u64,
    pub recipe_book_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelWaitReason {
    Respawn,
    DimensionChange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RespawnSessionProjection {
    pub spawn: CommonSpawnInfo,
    pub player: RespawnPlayerProjection,
    pub camera_entity_id: Option<i32>,
    pub container_open: bool,
    pub client_loaded: bool,
    pub level_generation: u64,
    pub debug_subscriptions_installed: bool,
    pub wait_reason: Option<LevelWaitReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitySessionAction {
    Ignored,
    Applied,
    CameraChanged,
    PickupProjected,
    SourceRemoved,
    Respawned { dimension_changed: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntitySessionClientProjection {
    entities: BTreeMap<i32, SessionEntityProjection>,
    local_player_entity_id: i32,
    camera_entity_id: Option<i32>,
    game_time: i64,
    respawn: Option<RespawnSessionProjection>,
}

impl EntitySessionClientProjection {
    #[must_use]
    pub const fn new(local_player_entity_id: i32) -> Self {
        Self {
            entities: BTreeMap::new(),
            local_player_entity_id,
            camera_entity_id: Some(local_player_entity_id),
            game_time: 0,
            respawn: None,
        }
    }

    pub fn track_entity(&mut self, entity_id: i32, entity: SessionEntityProjection) {
        self.entities.insert(entity_id, entity);
    }

    pub const fn set_game_time(&mut self, game_time: i64) {
        self.game_time = game_time;
    }

    pub fn install_respawn_session(&mut self, session: RespawnSessionProjection) {
        self.camera_entity_id = session.camera_entity_id;
        self.local_player_entity_id = session.player.entity_id;
        self.respawn = Some(session);
    }

    #[must_use]
    pub fn entity(&self, entity_id: i32) -> Option<&SessionEntityProjection> {
        self.entities.get(&entity_id)
    }

    #[must_use]
    pub const fn camera_entity_id(&self) -> Option<i32> {
        self.camera_entity_id
    }

    #[must_use]
    pub const fn respawn_session(&self) -> Option<&RespawnSessionProjection> {
        self.respawn.as_ref()
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
    ) -> Result<EntitySessionAction, EntitySessionProjectionError> {
        match packet {
            PlayClientboundPacket::Animate(packet) => self.apply_animate(*packet),
            PlayClientboundPacket::DamageEvent(packet) => Ok(self.apply_damage(packet)),
            PlayClientboundPacket::HurtAnimation(packet) => Ok(self.apply_hurt(*packet)),
            PlayClientboundPacket::Respawn(packet) => Ok(self.apply_respawn(packet)),
            PlayClientboundPacket::SetCamera(packet) => Ok(self.apply_camera(*packet)),
            PlayClientboundPacket::TakeItemEntity(packet) => self.apply_take(*packet),
            _ => Ok(EntitySessionAction::Ignored),
        }
    }

    fn apply_animate(
        &mut self,
        packet: Animate,
    ) -> Result<EntitySessionAction, EntitySessionProjectionError> {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return Ok(EntitySessionAction::Ignored);
        };
        match packet.action {
            0 => {
                require_living(entity, packet.entity_id, "main-hand swing")?;
                entity.main_hand_swings = entity.main_hand_swings.saturating_add(1);
            }
            3 => {
                require_living(entity, packet.entity_id, "off-hand swing")?;
                entity.off_hand_swings = entity.off_hand_swings.saturating_add(1);
            }
            2 => {
                if entity.kind != SessionEntityKind::Player {
                    return Err(EntitySessionProjectionError::WrongEntityType {
                        entity_id: packet.entity_id,
                        operation: "wake",
                    });
                }
                entity.wakes = entity.wakes.saturating_add(1);
            }
            4 => {
                entity.critical_particles = entity.critical_particles.saturating_add(1);
            }
            5 => {
                entity.enchanted_particles = entity.enchanted_particles.saturating_add(1);
            }
            _ => return Ok(EntitySessionAction::Ignored),
        }
        Ok(EntitySessionAction::Applied)
    }

    fn apply_hurt(&mut self, packet: HurtAnimation) -> EntitySessionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntitySessionAction::Ignored;
        };
        entity.hurt_yaw = Some(packet.yaw);
        EntitySessionAction::Applied
    }

    fn apply_damage(&mut self, packet: &DamageEvent) -> EntitySessionAction {
        let Some(target) = self.entities.get(&packet.entity_id) else {
            return EntitySessionAction::Ignored;
        };
        if !target.kind.living() {
            return EntitySessionAction::Applied;
        }
        let source_position = packet.source_position;
        let (cause_resolved, direct_resolved) = if source_position.is_some() {
            (false, false)
        } else {
            (
                self.entities.contains_key(&packet.cause_entity_id),
                self.entities.contains_key(&packet.direct_entity_id),
            )
        };
        let target = self
            .entities
            .get_mut(&packet.entity_id)
            .expect("target existence was checked");
        target.walk_animation_speed = 1.5;
        target.invulnerable_time = 20;
        target.hurt_time = 10;
        target.hurt_duration = 10;
        target.last_damage = Some(DamagePresentation {
            damage_type: packet.damage_type.clone(),
            source_position,
            cause_resolved,
            direct_resolved,
        });
        target.last_damage_game_time = Some(self.game_time);
        EntitySessionAction::Applied
    }

    fn apply_camera(&mut self, packet: SetCamera) -> EntitySessionAction {
        if !self.entities.contains_key(&packet.entity_id) {
            return EntitySessionAction::Ignored;
        }
        self.camera_entity_id = Some(packet.entity_id);
        if let Some(respawn) = &mut self.respawn {
            respawn.camera_entity_id = Some(packet.entity_id);
        }
        EntitySessionAction::CameraChanged
    }

    fn apply_take(
        &mut self,
        packet: TakeItemEntity,
    ) -> Result<EntitySessionAction, EntitySessionProjectionError> {
        let collector = self
            .entities
            .get(&packet.collector_entity_id)
            .map(|entity| (packet.collector_entity_id, entity.kind))
            .unwrap_or_else(|| {
                (
                    self.local_player_entity_id,
                    self.entities
                        .get(&self.local_player_entity_id)
                        .map_or(SessionEntityKind::Player, |entity| entity.kind),
                )
            });
        if !collector.1.living() {
            return Err(EntitySessionProjectionError::WrongEntityType {
                entity_id: collector.0,
                operation: "pickup collector",
            });
        }
        let Some(source) = self.entities.get_mut(&packet.source_entity_id) else {
            return Ok(EntitySessionAction::Ignored);
        };
        source.pickup_particles = source.pickup_particles.saturating_add(1);
        match source.kind {
            SessionEntityKind::Item => {
                source.item_count = source.item_count.wrapping_sub(packet.amount);
                if source.item_count <= 0 {
                    source.removed = true;
                    Ok(EntitySessionAction::SourceRemoved)
                } else {
                    Ok(EntitySessionAction::PickupProjected)
                }
            }
            SessionEntityKind::ExperienceOrb => Ok(EntitySessionAction::PickupProjected),
            _ => {
                source.removed = true;
                Ok(EntitySessionAction::SourceRemoved)
            }
        }
    }

    fn apply_respawn(&mut self, packet: &Respawn) -> EntitySessionAction {
        let Some(session) = &mut self.respawn else {
            return EntitySessionAction::Ignored;
        };
        let retention = packet.retention();
        let dimension_changed = session.spawn.dimension != packet.spawn.dimension;
        let old = session.player.clone();
        let mut player = RespawnPlayerProjection {
            entity_id: old.entity_id,
            position: Vector3::default(),
            motion: Vector3::default(),
            yaw: -180.0,
            pitch: 0.0,
            last_input: 0,
            sprinting: false,
            nondefault_entity_data: BTreeMap::new(),
            attributes: old
                .attributes
                .iter()
                .map(|(identity, attribute)| {
                    (
                        identity.clone(),
                        ProjectedAttribute {
                            base: attribute.base,
                            value: attribute.base,
                            modifiers: Vec::new(),
                        },
                    )
                })
                .collect(),
            stats_generation: old.stats_generation,
            recipe_book_generation: old.recipe_book_generation,
        };
        if retention.entity_data {
            player.position = old.position;
            player.motion = old.motion;
            player.yaw = old.yaw;
            player.pitch = old.pitch;
            player.last_input = old.last_input;
            player.sprinting = old.sprinting;
            player.nondefault_entity_data = old.nondefault_entity_data;
        }
        if retention.attributes {
            player.attributes = old.attributes;
        }
        if dimension_changed {
            session.level_generation = session.level_generation.saturating_add(1);
            session.debug_subscriptions_installed = false;
        }
        session.spawn = packet.spawn.clone();
        session.player = player;
        session.camera_entity_id = Some(session.player.entity_id);
        session.container_open = false;
        session.client_loaded = false;
        session.wait_reason = Some(if dimension_changed {
            LevelWaitReason::DimensionChange
        } else {
            LevelWaitReason::Respawn
        });
        self.camera_entity_id = session.camera_entity_id;
        EntitySessionAction::Respawned { dimension_changed }
    }
}

fn require_living(
    entity: &SessionEntityProjection,
    entity_id: i32,
    operation: &'static str,
) -> Result<(), EntitySessionProjectionError> {
    if entity.kind.living() {
        Ok(())
    } else {
        Err(EntitySessionProjectionError::WrongEntityType {
            entity_id,
            operation,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EntitySessionProjectionError {
    #[error("entity {entity_id} has the wrong runtime type for {operation}")]
    WrongEntityType {
        entity_id: i32,
        operation: &'static str,
    },
}
