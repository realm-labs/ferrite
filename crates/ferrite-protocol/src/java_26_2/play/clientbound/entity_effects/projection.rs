use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_effects::packet::{
    Explosion, RemoveMobEffect, UpdateMobEffect,
};
use crate::java_26_2::play::clientbound::entity_effects::particle::Particle;
use crate::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectFlags {
    pub ambient: bool,
    pub visible: bool,
    pub show_icon: bool,
    pub blend: bool,
}

impl EffectFlags {
    #[must_use]
    pub const fn from_byte(flags: u8) -> Self {
        Self {
            ambient: flags & 0x01 != 0,
            visible: flags & 0x02 != 0,
            show_icon: flags & 0x04 != 0,
            blend: flags & 0x08 != 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectBlendState {
    pub blending: bool,
    pub replacement_copies: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectInstanceProjection {
    pub amplifier: u8,
    pub duration: i32,
    pub infinite: bool,
    pub has_remaining_tick: bool,
    pub flags: EffectFlags,
    pub hidden_effect: bool,
    pub blend_state: EffectBlendState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackedEffectEntity {
    pub living: bool,
    pub can_be_affected: bool,
    pub effects: BTreeMap<Identifier, EffectInstanceProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionPresentation {
    pub sound_pitch: f32,
    pub sound_volume: f32,
    pub primary_particle: Particle,
    pub primary_velocity: Vector3,
    pub tracker_queued: bool,
    pub resulting_player_motion: Vector3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackerTick {
    pub attempted_samples: u16,
    pub cleared: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityEffectsAction {
    Ignored,
    EffectAdded,
    EffectReplaced,
    EffectRemoved,
    ExplosionPresented(ExplosionPresentation),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntityEffectsClientProjection {
    entities: BTreeMap<i32, TrackedEffectEntity>,
    player_motion: Vector3,
    queued_explosion_block_counts: Vec<i32>,
}

impl EntityEffectsClientProjection {
    pub fn track_entity(&mut self, entity_id: i32, entity: TrackedEffectEntity) {
        self.entities.insert(entity_id, entity);
    }

    pub const fn set_player_motion(&mut self, motion: Vector3) {
        self.player_motion = motion;
    }

    #[must_use]
    pub const fn player_motion(&self) -> Vector3 {
        self.player_motion
    }

    #[must_use]
    pub fn entity(&self, entity_id: i32) -> Option<&TrackedEffectEntity> {
        self.entities.get(&entity_id)
    }

    #[must_use]
    pub fn queued_explosions(&self) -> usize {
        self.queued_explosion_block_counts.len()
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
        explosion_sound_draws: (f32, f32),
    ) -> EntityEffectsAction {
        match packet {
            PlayClientboundPacket::Explosion(explosion) => {
                self.apply_explosion(explosion, explosion_sound_draws)
            }
            PlayClientboundPacket::RemoveMobEffect(effect) => self.apply_remove(effect),
            PlayClientboundPacket::UpdateMobEffect(effect) => self.apply_update(effect),
            _ => EntityEffectsAction::Ignored,
        }
    }

    pub fn tick_tracker(
        &mut self,
        particle_setting_all: bool,
    ) -> Result<TrackerTick, ExplosionTrackerError> {
        let total = self
            .queued_explosion_block_counts
            .iter()
            .try_fold(0_i32, |sum, count| sum.checked_add(*count))
            .ok_or(ExplosionTrackerError::BlockCountOverflow)?;
        let attempts = if particle_setting_all && total > 0 {
            total.min(512) as u16
        } else {
            0
        };
        self.queued_explosion_block_counts.clear();
        Ok(TrackerTick {
            attempted_samples: attempts,
            cleared: true,
        })
    }

    fn apply_update(&mut self, update: &UpdateMobEffect) -> EntityEffectsAction {
        let Some(entity) = self.entities.get_mut(&update.entity_id) else {
            return EntityEffectsAction::Ignored;
        };
        if !entity.living || !entity.can_be_affected {
            return EntityEffectsAction::Ignored;
        }
        let flags = EffectFlags::from_byte(update.flags);
        let fresh_blend = EffectBlendState {
            blending: flags.blend,
            replacement_copies: 0,
        };
        let previous = entity.effects.get(&update.effect);
        let blend_state = previous.map_or(fresh_blend, |instance| EffectBlendState {
            blending: instance.blend_state.blending,
            replacement_copies: instance.blend_state.replacement_copies.saturating_add(1),
        });
        let instance = EffectInstanceProjection {
            amplifier: update.amplifier.clamp(0, 255) as u8,
            duration: update.duration,
            infinite: update.duration == -1,
            has_remaining_tick: update.duration == -1 || update.duration > 0,
            flags,
            hidden_effect: false,
            blend_state,
        };
        let replaced = entity
            .effects
            .insert(update.effect.clone(), instance)
            .is_some();
        if replaced {
            EntityEffectsAction::EffectReplaced
        } else {
            EntityEffectsAction::EffectAdded
        }
    }

    fn apply_remove(&mut self, remove: &RemoveMobEffect) -> EntityEffectsAction {
        let Some(entity) = self.entities.get_mut(&remove.entity_id) else {
            return EntityEffectsAction::Ignored;
        };
        if !entity.living {
            return EntityEffectsAction::Ignored;
        }
        if entity.effects.remove(&remove.effect).is_some() {
            EntityEffectsAction::EffectRemoved
        } else {
            EntityEffectsAction::Ignored
        }
    }

    fn apply_explosion(
        &mut self,
        explosion: &Explosion,
        sound_draws: (f32, f32),
    ) -> EntityEffectsAction {
        let sound_pitch = (1.0 + (sound_draws.0 - sound_draws.1) * 0.2) * 0.7;
        let total_recipe_weight = explosion
            .block_particles
            .iter()
            .fold(0_i64, |sum, recipe| sum + i64::from(recipe.weight));
        let tracker_queued = total_recipe_weight > 0;
        if tracker_queued {
            self.queued_explosion_block_counts
                .push(explosion.block_count);
        }
        if let Some(knockback) = explosion.knockback {
            self.player_motion.x += knockback.x;
            self.player_motion.y += knockback.y;
            self.player_motion.z += knockback.z;
        }
        EntityEffectsAction::ExplosionPresented(ExplosionPresentation {
            sound_pitch,
            sound_volume: 4.0,
            primary_particle: explosion.particle.clone(),
            primary_velocity: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            tracker_queued,
            resulting_player_motion: self.player_motion,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExplosionTrackerError {
    #[error("queued explosion block-count total exceeds signed int")]
    BlockCountOverflow,
}
