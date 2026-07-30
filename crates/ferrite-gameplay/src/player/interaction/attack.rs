//! Client attack dispatch and independent server attack admission.

use crate::player::interaction::{Hand, HitTarget};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientAttackContext {
    pub miss_delay_remaining: u8,
    pub hit: Option<HitTarget>,
    pub hands_busy: bool,
    pub item_feature_enabled: bool,
    pub cannot_attack_with_item: bool,
    pub spectator: bool,
    pub piercing_weapon: bool,
    pub custom_range_present: bool,
    pub custom_range_admits_hit: bool,
    pub block_is_air: bool,
    pub block_became_air_during_start: bool,
    pub game_mode_uses_miss_time: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientAttackEffect {
    Spectate(u64),
    SpectatorNoAction,
    PiercingAttack,
    AttackEntity(u64),
    StartBlockBreak,
    InstantBlockAttack,
    InstallMissDelay(u8),
    ResetAttackStrength,
    Swing(Hand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAttackPlan {
    pub admitted: bool,
    pub instant_block_attack: bool,
    pub effects: Vec<ClientAttackEffect>,
}

#[must_use]
pub fn plan_client_attack(context: ClientAttackContext) -> ClientAttackPlan {
    if context.miss_delay_remaining > 0
        || context.hit.is_none()
        || context.hands_busy
        || !context.item_feature_enabled
        || context.cannot_attack_with_item
    {
        return ClientAttackPlan {
            admitted: false,
            instant_block_attack: false,
            effects: Vec::new(),
        };
    }
    let hit = context.hit.expect("the early return checks the hit");
    if context.spectator {
        let effect = match hit {
            HitTarget::Entity(entity) => ClientAttackEffect::Spectate(entity.entity_id),
            HitTarget::Block(_) | HitTarget::Miss { .. } => ClientAttackEffect::SpectatorNoAction,
        };
        return ClientAttackPlan {
            admitted: true,
            instant_block_attack: false,
            effects: vec![effect],
        };
    }
    if context.piercing_weapon {
        return ClientAttackPlan {
            admitted: true,
            instant_block_attack: false,
            effects: vec![
                ClientAttackEffect::PiercingAttack,
                ClientAttackEffect::Swing(Hand::Main),
            ],
        };
    }

    let mut effects = Vec::new();
    let mut instant_block_attack = false;
    match hit {
        HitTarget::Entity(entity) => {
            if !context.custom_range_present || context.custom_range_admits_hit {
                effects.push(ClientAttackEffect::AttackEntity(entity.entity_id));
            }
        }
        HitTarget::Block(_) if !context.block_is_air => {
            effects.push(ClientAttackEffect::StartBlockBreak);
            if context.block_became_air_during_start {
                effects.push(ClientAttackEffect::InstantBlockAttack);
                instant_block_attack = true;
            }
        }
        HitTarget::Block(_) | HitTarget::Miss { .. } => {
            if context.game_mode_uses_miss_time {
                effects.push(ClientAttackEffect::InstallMissDelay(10));
            }
            effects.push(ClientAttackEffect::ResetAttackStrength);
        }
    }
    effects.push(ClientAttackEffect::Swing(Hand::Main));
    ClientAttackPlan {
        admitted: true,
        instant_block_attack,
        effects,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackTargetKind {
    Ordinary,
    ItemEntity,
    ExperienceOrb,
    SelfPlayer,
    NonAttackableArrow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerAttackContext {
    pub target_current: bool,
    pub inside_world_border: bool,
    pub distance_to_bounds_squared: f64,
    pub attack_range: f64,
    pub target_kind: AttackTargetKind,
    pub item_feature_enabled: bool,
    pub cannot_attack_with_item: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerAttackOutcome {
    Ignored,
    DisconnectInvalidTarget,
    RejectedItem,
    Attack,
}

#[must_use]
pub fn admit_server_attack(context: ServerAttackContext) -> ServerAttackOutcome {
    if !context.target_current || !context.inside_world_border {
        return ServerAttackOutcome::Ignored;
    }
    let admitted_range = context.attack_range + 3.0;
    if context.distance_to_bounds_squared >= admitted_range * admitted_range {
        return ServerAttackOutcome::Ignored;
    }
    if context.target_kind != AttackTargetKind::Ordinary {
        return ServerAttackOutcome::DisconnectInvalidTarget;
    }
    if !context.item_feature_enabled || context.cannot_attack_with_item {
        return ServerAttackOutcome::RejectedItem;
    }
    ServerAttackOutcome::Attack
}

#[cfg(test)]
mod tests {
    use crate::player::interaction::EntityHit;
    use crate::player::state::Vec3;

    use super::*;

    fn entity_attack() -> ClientAttackContext {
        ClientAttackContext {
            miss_delay_remaining: 0,
            hit: Some(HitTarget::Entity(EntityHit {
                entity_id: 9,
                location: Vec3::ZERO,
                relative_location: Vec3::ZERO,
            })),
            hands_busy: false,
            item_feature_enabled: true,
            cannot_attack_with_item: false,
            spectator: false,
            piercing_weapon: false,
            custom_range_present: false,
            custom_range_admits_hit: true,
            block_is_air: false,
            block_became_air_during_start: false,
            game_mode_uses_miss_time: true,
        }
    }

    #[test]
    fn custom_range_rejection_still_swings_without_attacking() {
        let plan = plan_client_attack(ClientAttackContext {
            custom_range_present: true,
            custom_range_admits_hit: false,
            ..entity_attack()
        });
        assert_eq!(plan.effects, vec![ClientAttackEffect::Swing(Hand::Main)]);
    }

    #[test]
    fn server_range_uses_strict_plus_three_boundary() {
        assert_eq!(
            admit_server_attack(ServerAttackContext {
                target_current: true,
                inside_world_border: true,
                distance_to_bounds_squared: 64.0,
                attack_range: 5.0,
                target_kind: AttackTargetKind::Ordinary,
                item_feature_enabled: true,
                cannot_attack_with_item: false,
            }),
            ServerAttackOutcome::Ignored
        );
    }
}
