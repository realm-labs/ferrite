use crate::java_26_2::play::clientbound::entity_effects::packet::{
    Explosion, RemoveMobEffect, UpdateMobEffect,
};
use crate::java_26_2::play::clientbound::entity_effects::particle::Particle;
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::value::identifier::Identifier;

#[must_use]
pub const fn explosion_recipient(distance_squared: f64) -> bool {
    distance_squared < 4_096.0
}

#[must_use]
pub fn selected_explosion_particle(
    radius: f32,
    interacts_with_blocks: bool,
    small: Particle,
    large: Particle,
) -> Particle {
    if radius < 2.0 || !interacts_with_blocks {
        small
    } else {
        large
    }
}

#[must_use]
pub fn publish_explosion(explosion: Explosion) -> PlayClientboundPacket {
    PlayClientboundPacket::Explosion(Box::new(explosion))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectUpdatePublication {
    pub entity_id: i32,
    pub effect: Identifier,
    pub amplifier: i32,
    pub duration: i32,
    pub ambient: bool,
    pub visible: bool,
    pub show_icon: bool,
    pub self_player_new_effect: bool,
}

#[must_use]
pub fn publish_effect_update(publication: EffectUpdatePublication) -> PlayClientboundPacket {
    let mut flags = 0;
    if publication.ambient {
        flags |= 0x01;
    }
    if publication.visible {
        flags |= 0x02;
    }
    if publication.show_icon {
        flags |= 0x04;
    }
    if publication.self_player_new_effect {
        flags |= 0x08;
    }
    PlayClientboundPacket::UpdateMobEffect(UpdateMobEffect {
        entity_id: publication.entity_id,
        effect: publication.effect,
        amplifier: publication.amplifier,
        duration: publication.duration,
        flags,
    })
}

#[must_use]
pub fn publish_effect_removal(entity_id: i32, effect: Identifier) -> PlayClientboundPacket {
    PlayClientboundPacket::RemoveMobEffect(RemoveMobEffect { entity_id, effect })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectAudience {
    pub direct_player_passengers: bool,
    pub self_player: bool,
    pub indirect_passengers: bool,
    pub ordinary_tracking_viewers: bool,
}

pub const EFFECT_AUDIENCE: EffectAudience = EffectAudience {
    direct_player_passengers: true,
    self_player: true,
    indirect_passengers: false,
    ordinary_tracking_viewers: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectUpdateCause {
    Added,
    Replaced,
    PeriodicDurationRefresh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectPacketRecipient {
    DirectPlayerPassenger(usize),
    SelfPlayer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectPublicationStep {
    MarkParticleMetadataDirty,
    AddAttributeModifiers,
    RefreshAttributeModifiers,
    RemoveAttributeModifiers,
    SendUpdate {
        recipient: EffectPacketRecipient,
        blend: bool,
    },
    SendRemoval {
        recipient: EffectPacketRecipient,
    },
    RefreshAffectedAttributes,
}

#[must_use]
pub fn effect_update_plan(
    cause: EffectUpdateCause,
    direct_player_passengers: usize,
    self_player: bool,
) -> Vec<EffectPublicationStep> {
    let mut plan = vec![EffectPublicationStep::MarkParticleMetadataDirty];
    match cause {
        EffectUpdateCause::Added => plan.push(EffectPublicationStep::AddAttributeModifiers),
        EffectUpdateCause::Replaced => {
            plan.push(EffectPublicationStep::RefreshAttributeModifiers);
        }
        EffectUpdateCause::PeriodicDurationRefresh => {}
    }
    plan.extend(
        (0..direct_player_passengers).map(|index| EffectPublicationStep::SendUpdate {
            recipient: EffectPacketRecipient::DirectPlayerPassenger(index),
            blend: false,
        }),
    );
    if self_player {
        plan.push(EffectPublicationStep::SendUpdate {
            recipient: EffectPacketRecipient::SelfPlayer,
            blend: cause == EffectUpdateCause::Added,
        });
    }
    plan
}

#[must_use]
pub fn effect_removal_plan(
    direct_player_passengers: usize,
    self_player: bool,
) -> Vec<EffectPublicationStep> {
    let mut plan = vec![EffectPublicationStep::RemoveAttributeModifiers];
    plan.extend(
        (0..direct_player_passengers).map(|index| EffectPublicationStep::SendRemoval {
            recipient: EffectPacketRecipient::DirectPlayerPassenger(index),
        }),
    );
    plan.push(EffectPublicationStep::RefreshAffectedAttributes);
    if self_player {
        plan.push(EffectPublicationStep::SendRemoval {
            recipient: EffectPacketRecipient::SelfPlayer,
        });
    }
    plan
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RidingEffectStep {
    PositionAndChallenge,
    UpdateVehicleEffect { effect: Identifier, blend: bool },
    RemoveVehicleEffect { effect: Identifier },
    SendPassengerList,
}

#[must_use]
pub fn initial_self_effect_replay(effects_in_iteration_order: &[Identifier]) -> Vec<Identifier> {
    effects_in_iteration_order.to_vec()
}

#[must_use]
pub fn mount_effect_plan(effects_in_iteration_order: &[Identifier]) -> Vec<RidingEffectStep> {
    let mut plan = vec![RidingEffectStep::PositionAndChallenge];
    plan.extend(effects_in_iteration_order.iter().cloned().map(|effect| {
        RidingEffectStep::UpdateVehicleEffect {
            effect,
            blend: false,
        }
    }));
    plan.push(RidingEffectStep::SendPassengerList);
    plan
}

#[must_use]
pub fn dismount_effect_plan(effects_in_iteration_order: &[Identifier]) -> Vec<RidingEffectStep> {
    let mut plan = effects_in_iteration_order
        .iter()
        .cloned()
        .map(|effect| RidingEffectStep::RemoveVehicleEffect { effect })
        .collect::<Vec<_>>();
    plan.push(RidingEffectStep::SendPassengerList);
    plan
}
