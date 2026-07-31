//! Client entity, damage, level-event, and game-event dispatch.

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityEventAction {
    IgnoredMissingEntity,
    GuardianAttackSound,
    TotemActivation {
        emitter_ticks: u8,
        sound: bool,
        local_activation_display: bool,
    },
    SnifferSound,
    EntityHandler(i8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageEventAction {
    IgnoredMissingEntity,
    EntityDamageHandler(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelEventAction {
    OrdinaryHandler { event: i32, data: i32 },
    GlobalHandler { event: i32, data: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationAction {
    Entity(EntityEventAction),
    Damage(DamageEventAction),
    Level(LevelEventAction),
    GameEventHasNoClientPresentation,
    LocalCallSiteEffect { call_site: u64 },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventDispatcher {
    entities: BTreeSet<i32>,
    pub actions: Vec<PresentationAction>,
}

impl EventDispatcher {
    pub fn track_entity(&mut self, entity_id: i32) {
        self.entities.insert(entity_id);
    }

    pub fn remove_entity(&mut self, entity_id: i32) {
        self.entities.remove(&entity_id);
    }

    pub fn entity_event(
        &mut self,
        entity_id: i32,
        event: i8,
        local_player: bool,
    ) -> EntityEventAction {
        let action = if !self.entities.contains(&entity_id) {
            EntityEventAction::IgnoredMissingEntity
        } else {
            match event {
                21 => EntityEventAction::GuardianAttackSound,
                35 => EntityEventAction::TotemActivation {
                    emitter_ticks: 30,
                    sound: true,
                    local_activation_display: local_player,
                },
                63 => EntityEventAction::SnifferSound,
                other => EntityEventAction::EntityHandler(other),
            }
        };
        self.actions.push(PresentationAction::Entity(action));
        action
    }

    pub fn damage_event(&mut self, entity_id: i32) -> DamageEventAction {
        let action = if self.entities.contains(&entity_id) {
            DamageEventAction::EntityDamageHandler(entity_id)
        } else {
            DamageEventAction::IgnoredMissingEntity
        };
        self.actions.push(PresentationAction::Damage(action));
        action
    }

    pub fn level_event(&mut self, event: i32, data: i32, global: bool) -> LevelEventAction {
        let action = if global {
            LevelEventAction::GlobalHandler { event, data }
        } else {
            LevelEventAction::OrdinaryHandler { event, data }
        };
        self.actions.push(PresentationAction::Level(action));
        action
    }

    pub fn game_event(&mut self) {
        self.actions
            .push(PresentationAction::GameEventHasNoClientPresentation);
    }

    pub fn local_call_site_effect(&mut self, call_site: u64) {
        self.actions
            .push(PresentationAction::LocalCallSiteEffect { call_site });
    }
}
