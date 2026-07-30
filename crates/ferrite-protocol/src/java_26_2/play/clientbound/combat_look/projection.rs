use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::combat_look::packet::{
    EntityAnchor, LookPosition, PlayerCombatKill, PlayerLookAt,
};
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackedEntityPosition {
    pub feet: LookPosition,
    pub eye_height: f32,
    pub current_local_player: bool,
}

impl TrackedEntityPosition {
    fn anchor(self, anchor: EntityAnchor) -> LookPosition {
        match anchor {
            EntityAnchor::Feet => self.feet,
            EntityAnchor::Eyes => LookPosition {
                y: self.feet.y + f64::from(self.eye_height),
                ..self.feet
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerRotations {
    pub pitch: f32,
    pub yaw: f32,
    pub head_yaw: f32,
    pub previous_pitch: f32,
    pub previous_yaw: f32,
    pub body_yaw: f32,
    pub previous_body_yaw: f32,
    pub previous_head_yaw: f32,
}

impl Default for PlayerRotations {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            head_yaw: 0.0,
            previous_pitch: 0.0,
            previous_yaw: 0.0,
            body_yaw: 0.0,
            previous_body_yaw: 0.0,
            previous_head_yaw: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeathScreenProjection {
    pub message: TextComponentNbt,
    pub hardcore: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatLookAction {
    Ignored,
    DeathScreenInstalled(DeathScreenProjection),
    RespawnRequestedAndToggleKeysReset,
    Rotated(PlayerRotations),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatLookClientProjection {
    local_player_entity_id: i32,
    show_death_screen: bool,
    hardcore: bool,
    living_player: bool,
    entities: BTreeMap<i32, TrackedEntityPosition>,
    rotations: PlayerRotations,
    death_screen: Option<DeathScreenProjection>,
    respawn_requests: u32,
    toggle_key_resets: u32,
}

impl CombatLookClientProjection {
    #[must_use]
    pub fn new(
        local_player_entity_id: i32,
        local_position: TrackedEntityPosition,
        show_death_screen: bool,
        hardcore: bool,
        living_player: bool,
    ) -> Self {
        let mut entities = BTreeMap::new();
        entities.insert(local_player_entity_id, local_position);
        Self {
            local_player_entity_id,
            show_death_screen,
            hardcore,
            living_player,
            entities,
            rotations: PlayerRotations::default(),
            death_screen: None,
            respawn_requests: 0,
            toggle_key_resets: 0,
        }
    }

    pub fn track_entity(&mut self, entity_id: i32, position: TrackedEntityPosition) {
        self.entities.insert(entity_id, position);
    }

    pub fn remove_entity(&mut self, entity_id: i32) {
        self.entities.remove(&entity_id);
    }

    pub fn apply(&mut self, packet: &PlayClientboundPacket) -> CombatLookAction {
        match packet {
            PlayClientboundPacket::PlayerCombatEnter
            | PlayClientboundPacket::PlayerCombatEnd(_) => CombatLookAction::Ignored,
            PlayClientboundPacket::PlayerCombatKill(packet) => self.apply_kill(packet),
            PlayClientboundPacket::PlayerLookAt(packet) => self.apply_look(*packet),
            _ => CombatLookAction::Ignored,
        }
    }

    #[must_use]
    pub const fn rotations(&self) -> PlayerRotations {
        self.rotations
    }

    #[must_use]
    pub const fn death_screen(&self) -> Option<&DeathScreenProjection> {
        self.death_screen.as_ref()
    }

    #[must_use]
    pub const fn respawn_requests(&self) -> u32 {
        self.respawn_requests
    }

    #[must_use]
    pub const fn toggle_key_resets(&self) -> u32 {
        self.toggle_key_resets
    }

    fn apply_kill(&mut self, packet: &PlayerCombatKill) -> CombatLookAction {
        let current_local = self
            .entities
            .get(&packet.player_entity_id)
            .is_some_and(|entity| entity.current_local_player);
        if packet.player_entity_id != self.local_player_entity_id || !current_local {
            return CombatLookAction::Ignored;
        }
        if self.show_death_screen {
            let screen = DeathScreenProjection {
                message: packet.message.clone(),
                hardcore: self.hardcore,
            };
            self.death_screen = Some(screen.clone());
            CombatLookAction::DeathScreenInstalled(screen)
        } else {
            self.respawn_requests = self.respawn_requests.saturating_add(1);
            self.toggle_key_resets = self.toggle_key_resets.saturating_add(1);
            CombatLookAction::RespawnRequestedAndToggleKeysReset
        }
    }

    fn apply_look(&mut self, packet: PlayerLookAt) -> CombatLookAction {
        let target = packet
            .entity
            .and_then(|entity| {
                self.entities
                    .get(&entity.entity_id)
                    .map(|position| position.anchor(entity.anchor))
            })
            .unwrap_or(packet.fallback);
        let Some(local) = self.entities.get(&self.local_player_entity_id).copied() else {
            return CombatLookAction::Ignored;
        };
        let origin = local.anchor(packet.from_anchor);
        let dx = target.x - origin.x;
        let dy = target.y - origin.y;
        let dz = target.z - origin.z;
        let horizontal = (dx * dx + dz * dz).sqrt();
        let pitch = wrap_degrees((-dy.atan2(horizontal) * 57.295_776_367_187_5) as f32);
        let yaw = wrap_degrees((dz.atan2(dx) * 57.295_776_367_187_5) as f32 - 90.0);
        self.rotations.pitch = pitch;
        self.rotations.yaw = yaw;
        self.rotations.head_yaw = yaw;
        self.rotations.previous_pitch = pitch;
        self.rotations.previous_yaw = yaw;
        self.rotations.previous_head_yaw = yaw;
        if self.living_player {
            self.rotations.body_yaw = yaw;
            self.rotations.previous_body_yaw = yaw;
        }
        CombatLookAction::Rotated(self.rotations)
    }
}

fn wrap_degrees(value: f32) -> f32 {
    let wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped - 360.0
    } else if wrapped < -180.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}
