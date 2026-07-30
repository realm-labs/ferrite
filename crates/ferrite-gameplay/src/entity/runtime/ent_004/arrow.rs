//! Arrow-family in-ground, flight, piercing, damage, block-hit, and loyalty transitions.

use crate::entity::runtime::ent_004::geometry::Vector3;

pub const ARROW_DESPAWN_TICKS: u16 = 1_200;
pub const POTION_CONTENT_LOSS_TICKS: u16 = 600;
pub const SPECTRAL_GLOW_TICKS: u16 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowTickStage {
    DetectInBlock,
    ResolveGroundState,
    ApplyFlightMotion,
    ClipBlockEndpoint,
    GatherEntities,
    StableDistanceSort,
    ResolveHits,
}

pub const ARROW_TICK_ORDER: [ArrowTickStage; 7] = [
    ArrowTickStage::DetectInBlock,
    ArrowTickStage::ResolveGroundState,
    ArrowTickStage::ApplyFlightMotion,
    ArrowTickStage::ClipBlockEndpoint,
    ArrowTickStage::GatherEntities,
    ArrowTickStage::StableDistanceSort,
    ArrowTickStage::ResolveHits,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InGroundStep {
    pub in_ground_time: u16,
    pub velocity: Vector3,
    pub released: bool,
    pub discard: bool,
}

#[must_use]
pub fn in_ground_step(
    in_ground_time: u16,
    block_changed: bool,
    collision_free_radius_point_zero_six: bool,
    velocity: Vector3,
    release_draws: [f32; 3],
) -> InGroundStep {
    if block_changed && collision_free_radius_point_zero_six {
        return InGroundStep {
            in_ground_time: 0,
            velocity: Vector3::new(
                velocity.x * f64::from(release_draws[0]) * 0.2,
                velocity.y * f64::from(release_draws[1]) * 0.2,
                velocity.z * f64::from(release_draws[2]) * 0.2,
            ),
            released: true,
            discard: false,
        };
    }
    let in_ground_time = in_ground_time.saturating_add(1);
    InGroundStep {
        in_ground_time,
        velocity,
        released: false,
        discard: in_ground_time >= ARROW_DESPAWN_TICKS,
    }
}

#[must_use]
pub const fn arrow_flight_velocity(velocity: Vector3, in_water: bool, no_gravity: bool) -> Vector3 {
    let inertia = if in_water { 0.6 } else { 0.99 };
    let dragged = velocity.scale(inertia);
    Vector3::new(
        dragged.x,
        if no_gravity {
            dragged.y
        } else {
            dragged.y - 0.05
        },
        dragged.z,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowCandidate {
    pub entity_id: u64,
    pub squared_distance_to_position: f64,
    pub admitted: bool,
}

#[must_use]
pub fn piercing_targets(
    candidates: &[ArrowCandidate],
    block_squared_distance: f64,
    pierce_level: u8,
    prior_hit_ids: &[u64],
) -> Vec<u64> {
    let mut ordered: Vec<(usize, ArrowCandidate)> = candidates
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, candidate)| {
            candidate.admitted
                && candidate.squared_distance_to_position <= block_squared_distance
                && !prior_hit_ids.contains(&candidate.entity_id)
        })
        .collect();
    ordered.sort_by(|left, right| {
        left.1
            .squared_distance_to_position
            .total_cmp(&right.1.squared_distance_to_position)
            .then_with(|| left.0.cmp(&right.0))
    });
    ordered
        .into_iter()
        .take(usize::from(pierce_level) + 1)
        .map(|(_, candidate)| candidate.entity_id)
        .collect()
}

#[must_use]
pub fn arrow_damage(speed: f64, modified_base_damage: f64, critical_draw: Option<u32>) -> u32 {
    let base = (speed * modified_base_damage)
        .clamp(0.0, f64::from(i32::MAX))
        .ceil() as u32;
    match critical_draw {
        Some(draw) => base.saturating_add(draw % (base / 2 + 2)),
        None => base,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityHitResult {
    pub discard: bool,
    pub restored_fire: bool,
    pub velocity: Vector3,
    pub drop_arrow: bool,
}

#[must_use]
pub const fn resolve_entity_damage(
    damage_succeeded: bool,
    pierce_level: u8,
    prior_velocity: Vector3,
    pickup_allowed: bool,
) -> EntityHitResult {
    if damage_succeeded {
        return EntityHitResult {
            discard: pierce_level == 0,
            restored_fire: false,
            velocity: prior_velocity,
            drop_arrow: false,
        };
    }
    let velocity = prior_velocity.scale(-0.2);
    let stopped = velocity.squared_length() < 1.0e-7;
    EntityHitResult {
        discard: stopped,
        restored_fire: true,
        velocity,
        drop_arrow: stopped && pickup_allowed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockHit {
    pub backed_up: Vector3,
    pub velocity: Vector3,
    pub in_ground: bool,
    pub shake_time: u8,
    pub critical: bool,
    pub pierce_level: u8,
    pub clear_hit_sets: bool,
}

#[must_use]
pub fn block_hit(position: Vector3, movement: Vector3) -> BlockHit {
    let signs = Vector3::new(
        nonzero_sign(movement.x),
        nonzero_sign(movement.y),
        nonzero_sign(movement.z),
    );
    BlockHit {
        backed_up: position.add(signs.scale(-0.05)),
        velocity: Vector3::ZERO,
        in_ground: true,
        shake_time: 7,
        critical: false,
        pierce_level: 0,
        clear_hit_sets: true,
    }
}

fn nonzero_sign(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value.signum() }
}

#[must_use]
pub const fn potion_arrow_loses_contents(in_ground_time: u16) -> bool {
    in_ground_time >= POTION_CONTENT_LOSS_TICKS
}

#[must_use]
pub const fn spectral_glow_duration() -> u16 {
    SPECTRAL_GLOW_TICKS
}

#[must_use]
pub const fn trident_damage() -> f32 {
    8.0
}

#[must_use]
pub const fn trident_target_limit() -> u8 {
    1
}

#[must_use]
pub const fn trident_marks_dealt(in_ground_time: u16) -> bool {
    in_ground_time >= 5
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoyaltyReturn {
    pub velocity: Vector3,
    pub vertical_adjustment: f64,
}

#[must_use]
pub fn loyalty_return(
    velocity: Vector3,
    delta_to_owner_eyes: Vector3,
    loyalty_level: u8,
) -> LoyaltyReturn {
    let level = f64::from(loyalty_level);
    let vertical_adjustment = 0.015 * level;
    LoyaltyReturn {
        velocity: velocity
            .scale(0.95)
            .add(delta_to_owner_eyes.normalize().scale(0.05 * level)),
        vertical_adjustment,
    }
}

#[must_use]
pub const fn trident_water_inertia() -> f64 {
    0.99
}
