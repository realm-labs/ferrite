use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::entity_effects::packet::SoundEventHolder;
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::clientbound::sound::packet::{
    SoundAtEntity, SoundAtPosition, SoundSource, StopSound,
};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackedSoundEntity {
    pub object_token: u64,
    pub position: Vector3,
    pub silent: bool,
    pub removed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SoundInstanceBinding {
    Positional,
    Entity { entity_id: i32, object_token: u64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedSoundInstance {
    pub sound: Identifier,
    pub source: SoundSource,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
    pub position: Vector3,
    pub binding: SoundInstanceBinding,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SoundProjectionAction {
    Played(ProjectedSoundInstance),
    MissingEntity,
    SilentEntity,
    Stopped { count: usize },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SoundProjection {
    instances: Vec<ProjectedSoundInstance>,
}

impl SoundProjection {
    #[must_use]
    pub fn instances(&self) -> &[ProjectedSoundInstance] {
        &self.instances
    }

    pub fn apply_position(&mut self, packet: SoundAtPosition) -> SoundProjectionAction {
        let instance = ProjectedSoundInstance {
            sound: sound_identity(&packet.sound),
            source: packet.source,
            volume: packet.volume,
            pitch: packet.pitch,
            seed: packet.seed,
            position: packet.position(),
            binding: SoundInstanceBinding::Positional,
        };
        self.instances.push(instance.clone());
        SoundProjectionAction::Played(instance)
    }

    pub fn apply_entity(
        &mut self,
        packet: SoundAtEntity,
        entities: &BTreeMap<i32, TrackedSoundEntity>,
    ) -> SoundProjectionAction {
        let Some(entity) = entities
            .get(&packet.entity_id)
            .filter(|entity| !entity.removed)
        else {
            return SoundProjectionAction::MissingEntity;
        };
        if entity.silent {
            return SoundProjectionAction::SilentEntity;
        }
        let instance = ProjectedSoundInstance {
            sound: sound_identity(&packet.sound),
            source: packet.source,
            volume: packet.volume,
            pitch: packet.pitch,
            seed: packet.seed,
            position: float_rounded_position(entity.position),
            binding: SoundInstanceBinding::Entity {
                entity_id: packet.entity_id,
                object_token: entity.object_token,
            },
        };
        self.instances.push(instance.clone());
        SoundProjectionAction::Played(instance)
    }

    pub fn apply_stop(&mut self, packet: &StopSound) -> SoundProjectionAction {
        let before = self.instances.len();
        self.instances.retain(|instance| {
            let source_matches = packet.source.is_none_or(|source| source == instance.source);
            let sound_matches = packet
                .sound
                .as_ref()
                .is_none_or(|sound| sound == &instance.sound);
            !(source_matches && sound_matches)
        });
        SoundProjectionAction::Stopped {
            count: before - self.instances.len(),
        }
    }

    pub fn tick_entity_bindings(&mut self, entities: &BTreeMap<i32, TrackedSoundEntity>) -> usize {
        let before = self.instances.len();
        self.instances.retain_mut(|instance| {
            let SoundInstanceBinding::Entity {
                entity_id,
                object_token,
            } = instance.binding
            else {
                return true;
            };
            let Some(entity) = entities
                .get(&entity_id)
                .filter(|entity| !entity.removed && entity.object_token == object_token)
            else {
                return false;
            };
            instance.position = float_rounded_position(entity.position);
            true
        });
        before - self.instances.len()
    }
}

#[must_use]
pub fn sound_identity(holder: &SoundEventHolder) -> Identifier {
    match holder {
        SoundEventHolder::Direct { identity, .. } | SoundEventHolder::Registered(identity) => {
            identity.clone()
        }
    }
}

fn float_rounded_position(position: Vector3) -> Vector3 {
    Vector3 {
        x: f64::from(position.x as f32),
        y: f64::from(position.y as f32),
        z: f64::from(position.z as f32),
    }
}
