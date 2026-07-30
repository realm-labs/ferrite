//! Entity query bounds, exposure samples, damage, knockback, and notification routing.

use crate::redstone::explosion::math::{Aabb, Vec3};

pub const ENTITY_PHASE_MIN_RADIUS: f32 = 1.0e-5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityQueryBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
    pub excludes_direct_source: bool,
}

pub fn entity_query_bounds(center: Vec3, radius: f32) -> Option<EntityQueryBounds> {
    if radius < ENTITY_PHASE_MIN_RADIUS {
        return None;
    }
    let double_radius = f64::from(radius * 2.0_f32);
    Some(EntityQueryBounds {
        min_x: (center.x - double_radius - 1.0).floor() as i32,
        min_y: (center.y - double_radius - 1.0).floor() as i32,
        min_z: (center.z - double_radius - 1.0).floor() as i32,
        max_x: (center.x + double_radius + 1.0).floor() as i32,
        max_y: (center.y + double_radius + 1.0).floor() as i32,
        max_z: (center.z + double_radius + 1.0).floor() as i32,
        excludes_direct_source: true,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExposureTrace {
    pub misses: u32,
    pub samples: u32,
}

impl ExposureTrace {
    pub fn seen_percent(self) -> f32 {
        if self.samples == 0 {
            0.0
        } else {
            self.misses as f32 / self.samples as f32
        }
    }
}

pub fn calculate_exposure(
    center: Vec3,
    bounds: Aabb,
    mut collider_miss: impl FnMut(Vec3, Vec3) -> bool,
) -> ExposureTrace {
    let extent = bounds.extent();
    let x_step = 1.0 / (extent.x * 2.0 + 1.0);
    let y_step = 1.0 / (extent.y * 2.0 + 1.0);
    let z_step = 1.0 / (extent.z * 2.0 + 1.0);
    let x_offset = (1.0 - (1.0 / x_step).floor() * x_step) / 2.0;
    let z_offset = (1.0 - (1.0 / z_step).floor() * z_step) / 2.0;
    if x_step < 0.0 || y_step < 0.0 || z_step < 0.0 {
        return ExposureTrace {
            misses: 0,
            samples: 0,
        };
    }

    let mut misses = 0;
    let mut samples = 0;
    let mut x = 0.0;
    while x <= 1.0 {
        let mut y = 0.0;
        while y <= 1.0 {
            let mut z = 0.0;
            while z <= 1.0 {
                let from = Vec3::new(
                    Vec3::lerp(x, bounds.min.x, bounds.max.x) + x_offset,
                    Vec3::lerp(y, bounds.min.y, bounds.max.y),
                    Vec3::lerp(z, bounds.min.z, bounds.max.z) + z_offset,
                );
                if collider_miss(from, center) {
                    misses += 1;
                }
                samples += 1;
                z += z_step;
            }
            y += y_step;
        }
        x += x_step;
    }
    ExposureTrace { misses, samples }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerState {
    pub spectator: bool,
    pub creative: bool,
    pub flying: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityEffectInput {
    pub position: Vec3,
    pub effect_origin: Vec3,
    pub ignore_explosion: bool,
    pub should_damage: bool,
    pub knockback_multiplier: f32,
    pub exposure: f32,
    pub living_knockback_resistance: Option<f64>,
    pub redirectable_projectile: bool,
    pub player: Option<PlayerState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostPushRouting {
    RedirectProjectileToDamageSource,
    RecordPlayerVelocity,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityEffectStage {
    Damage,
    Push,
    RedirectOrRecordPlayer,
    OnExplosionHit,
}

pub const ENTITY_EFFECT_ORDER: [EntityEffectStage; 4] = [
    EntityEffectStage::Damage,
    EntityEffectStage::Push,
    EntityEffectStage::RedirectOrRecordPlayer,
    EntityEffectStage::OnExplosionHit,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityEffectPlan {
    pub normalized_distance: f64,
    pub exposure_was_required: bool,
    pub exposure: f32,
    pub damage: Option<f32>,
    pub knockback: Vec3,
    pub push_even_if_zero: bool,
    pub routing: PostPushRouting,
    pub call_on_explosion_hit: bool,
    pub order: [EntityEffectStage; 4],
}

pub fn plan_entity_effect(
    center: Vec3,
    radius: f32,
    input: EntityEffectInput,
) -> Option<EntityEffectPlan> {
    if radius < ENTITY_PHASE_MIN_RADIUS || input.ignore_explosion {
        return None;
    }
    let double_radius = radius * 2.0_f32;
    let normalized_distance =
        input.position.distance_squared(center).sqrt() / f64::from(double_radius);
    if normalized_distance > 1.0 {
        return None;
    }

    let exposure_was_required = input.should_damage || input.knockback_multiplier != 0.0;
    let exposure = if exposure_was_required {
        input.exposure
    } else {
        0.0
    };
    let damage = input
        .should_damage
        .then(|| default_damage(normalized_distance, exposure, double_radius));
    let resistance = input.living_knockback_resistance.unwrap_or(0.0);
    let knockback_power = (1.0 - normalized_distance)
        * f64::from(exposure)
        * f64::from(input.knockback_multiplier)
        * (1.0 - resistance);
    let knockback = input
        .effect_origin
        .subtract(center)
        .normalize()
        .scale(knockback_power);

    Some(EntityEffectPlan {
        normalized_distance,
        exposure_was_required,
        exposure,
        damage,
        knockback,
        push_even_if_zero: true,
        routing: routing(input),
        call_on_explosion_hit: true,
        order: ENTITY_EFFECT_ORDER,
    })
}

pub fn default_damage(normalized_distance: f64, exposure: f32, double_radius: f32) -> f32 {
    let power = (1.0 - normalized_distance) * f64::from(exposure);
    ((power * power + power) / 2.0 * 7.0 * f64::from(double_radius) + 1.0) as f32
}

fn routing(input: EntityEffectInput) -> PostPushRouting {
    if input.redirectable_projectile {
        return PostPushRouting::RedirectProjectileToDamageSource;
    }
    match input.player {
        Some(player) if !player.spectator && !(player.creative && player.flying) => {
            PostPushRouting::RecordPlayerVelocity
        }
        _ => PostPushRouting::None,
    }
}
