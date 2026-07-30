use thiserror::Error;

use crate::java_26_2::play::serverbound::entity_session::model::{
    EntitySessionAction, EntitySessionDisposition, EntitySessionPlayer, SessionEntityProjection,
    SessionLevelProjection,
};
use crate::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;

#[derive(Debug, Clone, PartialEq)]
pub struct EntitySessionProjection {
    pub(super) player: EntitySessionPlayer,
    pub(super) levels: Vec<SessionLevelProjection>,
    pub(super) actions: Vec<EntitySessionAction>,
}

impl EntitySessionProjection {
    pub fn new(
        player: EntitySessionPlayer,
        levels: Vec<SessionLevelProjection>,
    ) -> Result<Self, EntitySessionProjectionError> {
        if player.current_level >= levels.len() {
            return Err(EntitySessionProjectionError::CurrentLevelOutOfBounds {
                index: player.current_level,
                levels: levels.len(),
            });
        }
        if player.selected_hotbar >= player.hotbar.len() {
            return Err(EntitySessionProjectionError::SelectedHotbarOutOfBounds {
                slot: player.selected_hotbar,
                slots: player.hotbar.len(),
            });
        }
        Ok(Self {
            player,
            levels,
            actions: Vec::new(),
        })
    }

    #[must_use]
    pub const fn player(&self) -> &EntitySessionPlayer {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut EntitySessionPlayer {
        &mut self.player
    }

    #[must_use]
    pub fn levels(&self) -> &[SessionLevelProjection] {
        &self.levels
    }

    pub fn levels_mut(&mut self) -> &mut [SessionLevelProjection] {
        &mut self.levels
    }

    #[must_use]
    pub fn actions(&self) -> &[EntitySessionAction] {
        &self.actions
    }

    pub fn take_actions(&mut self) -> Vec<EntitySessionAction> {
        std::mem::take(&mut self.actions)
    }

    pub fn handle(
        &mut self,
        packet: PlayServerboundEntryPacket,
    ) -> Result<EntitySessionDisposition, EntitySessionProjectionError> {
        match packet {
            PlayServerboundEntryPacket::Attack(packet) => Ok(self.handle_attack(packet)),
            PlayServerboundEntryPacket::ClientCommand(packet) => {
                Ok(self.handle_client_command(packet))
            }
            PlayServerboundEntryPacket::Interact(packet) => Ok(self.handle_interact(packet)),
            PlayServerboundEntryPacket::PickItemFromEntity(packet) => Ok(self.handle_pick(packet)),
            PlayServerboundEntryPacket::SpectatorAction(packet) => {
                Ok(self.handle_spectator_action(packet))
            }
            PlayServerboundEntryPacket::TeleportToEntity(packet) => {
                Ok(self.handle_teleport_to_entity(packet))
            }
            _ => Ok(EntitySessionDisposition::Ignored),
        }
    }

    pub(super) fn current_entity(&self, entity_id: i32) -> Option<&SessionEntityProjection> {
        self.levels[self.player.current_level]
            .entities
            .iter()
            .find(|entity| entity.entity_id == entity_id)
    }

    pub(super) fn uuid_entity(&self, uuid: u128) -> Option<(usize, SessionEntityProjection)> {
        self.levels.iter().enumerate().find_map(|(level, state)| {
            state
                .entities
                .iter()
                .find(|entity| entity.uuid == uuid)
                .cloned()
                .map(|entity| (level, entity))
        })
    }

    pub(super) fn reset_idle(&mut self) {
        self.player.idle_resets = self.player.idle_resets.wrapping_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EntitySessionProjectionError {
    #[error("current level index {index} is outside {levels} installed levels")]
    CurrentLevelOutOfBounds { index: usize, levels: usize },
    #[error("selected hotbar slot {slot} is outside {slots} installed slots")]
    SelectedHotbarOutOfBounds { slot: usize, slots: usize },
}
