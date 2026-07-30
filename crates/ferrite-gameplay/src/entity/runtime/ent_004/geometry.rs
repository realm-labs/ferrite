//! Shared projectile launch, ownership, collision, deflection, and hit ordering.

use std::f64::consts::PI;

pub const UNCERTAINTY_DEVIATION: f64 = 0.017_227_5;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    #[must_use]
    pub const fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    #[must_use]
    pub const fn squared_length(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[must_use]
    pub fn length(self) -> f64 {
        self.squared_length().sqrt()
    }

    #[must_use]
    pub fn normalize(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Self::ZERO
        } else {
            self.scale(1.0 / length)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Launch {
    pub velocity: Vector3,
    pub yaw: f32,
    pub pitch: f32,
}

#[must_use]
pub fn launch(
    direction: Vector3,
    power: f64,
    uncertainty: f64,
    triangular_draws: [f64; 6],
) -> Launch {
    let deviation = UNCERTAINTY_DEVIATION * uncertainty;
    let normalized = direction.normalize();
    let perturbed = Vector3::new(
        normalized.x + (triangular_draws[0] - triangular_draws[1]) * deviation,
        normalized.y + (triangular_draws[2] - triangular_draws[3]) * deviation,
        normalized.z + (triangular_draws[4] - triangular_draws[5]) * deviation,
    )
    .scale(power);
    rotation_for(perturbed, perturbed)
}

#[must_use]
pub fn shoot_from_rotation(
    pitch_degrees: f32,
    yaw_degrees: f32,
    power: f64,
    uncertainty: f64,
    triangular_draws: [f64; 6],
    source_velocity: Vector3,
    source_on_ground: bool,
) -> Launch {
    let pitch = f64::from(pitch_degrees) * PI / 180.0;
    let yaw = f64::from(yaw_degrees) * PI / 180.0;
    let mut shot = launch(
        Vector3::new(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        ),
        power,
        uncertainty,
        triangular_draws,
    );
    shot.velocity.x += source_velocity.x;
    shot.velocity.z += source_velocity.z;
    if !source_on_ground {
        shot.velocity.y += source_velocity.y;
    }
    shot
}

fn rotation_for(velocity: Vector3, fallback: Vector3) -> Launch {
    let horizontal = velocity.x.hypot(velocity.z);
    let facing = if velocity.squared_length() == 0.0 {
        fallback
    } else {
        velocity
    };
    Launch {
        velocity,
        yaw: (facing.x.atan2(facing.z) * 180.0 / PI) as f32,
        pitch: (facing.y.atan2(horizontal) * 180.0 / PI) as f32,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnStage {
    Shoot,
    AddEntity,
    ProjectileSpawnEnchantment,
}

pub const SPAWN_ORDER: [SpawnStage; 3] = [
    SpawnStage::Shoot,
    SpawnStage::AddEntity,
    SpawnStage::ProjectileSpawnEnchantment,
];

#[must_use]
pub const fn emits_shoot_event(tick_count: u32) -> bool {
    tick_count == 0
}

#[must_use]
pub const fn update_left_owner(
    already_left_owner: bool,
    swept_box_intersects_pickable_root_vehicle_member: bool,
) -> bool {
    already_left_owner || !swept_box_intersects_pickable_root_vehicle_member
}

#[must_use]
pub fn collision_margin(tick_count: u32) -> f64 {
    (f64::from(tick_count.saturating_sub(2)) / 20.0).clamp(0.0, 0.3)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitCandidate {
    pub entity_id: u64,
    pub squared_distance: f64,
    pub hittable: bool,
    pub shares_owner_vehicle: bool,
}

#[must_use]
pub fn first_entity_hit(
    candidates: &[HitCandidate],
    left_owner: bool,
    block_squared_distance: Option<f64>,
) -> Option<u64> {
    let mut selected = None;
    let mut nearest = block_squared_distance.unwrap_or(f64::INFINITY);
    for candidate in candidates {
        if !candidate.hittable || (!left_owner && candidate.shares_owner_vehicle) {
            continue;
        }
        if candidate.squared_distance < nearest {
            nearest = candidate.squared_distance;
            selected = Some(candidate.entity_id);
        }
    }
    selected
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deflection {
    RejectedSameDeflector,
    Applied,
}

#[must_use]
pub const fn deflect(last_deflector: Option<u64>, deflector: u64) -> Deflection {
    if matches!(last_deflector, Some(last) if last == deflector) {
        Deflection::RejectedSameDeflector
    } else {
        Deflection::Applied
    }
}

#[must_use]
pub const fn world_border_bounce(velocity: Vector3, subtype_bounces: bool) -> Option<Vector3> {
    if subtype_bounces {
        Some(velocity.scale(-0.2))
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityHitStage {
    RedirectTarget,
    SubtypeCallback,
    ProjectileLandEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockHitStage {
    BlockCallback,
    ProjectileLandEvent,
}

#[must_use]
pub const fn entity_hit_order(target_redirectable: bool) -> [Option<EntityHitStage>; 3] {
    [
        if target_redirectable {
            Some(EntityHitStage::RedirectTarget)
        } else {
            None
        },
        Some(EntityHitStage::SubtypeCallback),
        Some(EntityHitStage::ProjectileLandEvent),
    ]
}

pub const BLOCK_HIT_ORDER: [BlockHitStage; 2] = [
    BlockHitStage::BlockCallback,
    BlockHitStage::ProjectileLandEvent,
];
