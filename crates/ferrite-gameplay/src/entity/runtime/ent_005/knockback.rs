//! Damage-source direction, common resistance, sulfur-cube, and indication mechanics.

use std::f64::consts::PI;

pub const DIRECTION_EPSILON_SQUARED: f64 = 9.999_999_747_378_752e-6;

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
    pub const fn subtract(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    #[must_use]
    pub const fn scale(self, scale: f64) -> Self {
        Self::new(self.x * scale, self.y * scale, self.z * scale)
    }

    #[must_use]
    pub fn normalize(self) -> Self {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if length < DIRECTION_EPSILON_SQUARED {
            Self::ZERO
        } else {
            self.scale(1.0 / length)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileDirectionKind {
    Ordinary,
    FireworkOrPotion,
}

#[must_use]
pub const fn projectile_direction(
    kind: ProjectileDirectionKind,
    projectile_position: Vector3,
    projectile_velocity: Vector3,
    victim_position: Vector3,
) -> (f64, f64) {
    match kind {
        ProjectileDirectionKind::Ordinary => (-projectile_velocity.x, -projectile_velocity.z),
        ProjectileDirectionKind::FireworkOrPotion => (
            -(victim_position.x - projectile_position.x),
            -(victim_position.z - projectile_position.z),
        ),
    }
}

#[must_use]
pub const fn positioned_source_direction(
    source_position: Option<Vector3>,
    victim_position: Vector3,
) -> (f64, f64) {
    match source_position {
        Some(source) => (source.x - victim_position.x, source.z - victim_position.z),
        None => (0.0, 0.0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommonKnockback {
    pub velocity: Vector3,
    pub dirty: bool,
    pub draws_consumed: usize,
    pub completed: bool,
}

#[must_use]
pub fn common_knockback(
    strength: f64,
    resistance: f64,
    mut direction_x: f64,
    mut direction_z: f64,
    old_velocity: Vector3,
    on_ground: bool,
    retry_draws: &[[f64; 4]],
) -> CommonKnockback {
    let effective = strength * (1.0 - resistance);
    if effective <= 0.0 {
        return CommonKnockback {
            velocity: old_velocity,
            dirty: false,
            draws_consumed: 0,
            completed: true,
        };
    }
    let mut retries = 0;
    while direction_x * direction_x + direction_z * direction_z < DIRECTION_EPSILON_SQUARED {
        let Some(draws) = retry_draws.get(retries) else {
            return CommonKnockback {
                velocity: old_velocity,
                dirty: false,
                draws_consumed: retries * 4,
                completed: false,
            };
        };
        direction_x = (draws[0] - draws[1]) * 0.01;
        direction_z = (draws[2] - draws[3]) * 0.01;
        retries += 1;
    }
    let length = direction_x.hypot(direction_z);
    let push_x = direction_x / length * effective;
    let push_z = direction_z / length * effective;
    CommonKnockback {
        velocity: Vector3::new(
            old_velocity.x / 2.0 - push_x,
            if on_ground {
                (old_velocity.y / 2.0 + effective).min(0.4)
            } else {
                old_velocity.y
            },
            old_velocity.z / 2.0 - push_z,
        ),
        dirty: true,
        draws_consumed: retries * 4,
        completed: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiveArgumentGate {
    Common,
    CreakingImmobile,
    DragonSitting,
}

#[must_use]
pub const fn five_argument_admitted(gate: FiveArgumentGate) -> bool {
    matches!(gate, FiveArgumentGate::Common)
}

#[must_use]
pub const fn uses_sulfur_special(body_item_present: bool, causing_entity_present: bool) -> bool {
    body_item_present && causing_entity_present
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageIndication {
    pub hurt_direction: f32,
    pub send_hurt_animation: bool,
}

#[must_use]
pub fn damage_indication(
    blocked: bool,
    server_player: bool,
    direction_x: f64,
    direction_z: f64,
    yaw: f32,
) -> Option<DamageIndication> {
    if blocked || !server_player {
        return None;
    }
    Some(DamageIndication {
        hurt_direction: (direction_z.atan2(direction_x) * 180.0 / PI) as f32 - yaw,
        send_hurt_animation: true,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SulfurArchetype {
    Bouncy,
    Explosive,
    FastFlat,
    FastSliding,
    HighResistance,
    Hot,
    Light,
    Regular,
    SlowBouncy,
    SlowFlat,
    SlowSliding,
    Sticky,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SulfurSettings {
    pub horizontal: f32,
    pub vertical: f32,
    pub sound_suffix: &'static str,
}

#[must_use]
pub const fn sulfur_settings(archetype: SulfurArchetype) -> SulfurSettings {
    let (horizontal, vertical, sound_suffix) = match archetype {
        SulfurArchetype::Bouncy => (0.4125, 0.105, "bouncy.hit"),
        SulfurArchetype::Explosive => (0.4125, 0.09, "explosive.hit"),
        SulfurArchetype::FastFlat => (0.9125, 0.09, "fast_flat.hit"),
        SulfurArchetype::FastSliding => (0.6625, 0.09, "fast_sliding.hit"),
        SulfurArchetype::HighResistance => (0.4125, 0.09, "high_resistance.hit"),
        SulfurArchetype::Hot => (0.4125, 0.09, "hot.hit"),
        SulfurArchetype::Light => (0.4125, 0.18, "light.hit"),
        SulfurArchetype::Regular => (0.4125, 0.09, "regular.hit"),
        SulfurArchetype::SlowBouncy => (0.4125, 0.24, "slow_bouncy.hit"),
        SulfurArchetype::SlowFlat => (0.4125, 0.105, "slow_flat.hit"),
        SulfurArchetype::SlowSliding => (0.4125, 0.09, "slow_sliding.hit"),
        SulfurArchetype::Sticky => (0.4125, 0.09, "sticky.hit"),
    };
    SulfurSettings {
        horizontal,
        vertical,
        sound_suffix,
    }
}

#[must_use]
pub fn active_sulfur_settings(matches_in_registry_order: &[SulfurArchetype]) -> SulfurSettings {
    matches_in_registry_order
        .last()
        .copied()
        .map(sulfur_settings)
        .unwrap_or(SulfurSettings {
            horizontal: 0.33,
            vertical: 0.06,
            sound_suffix: "regular.hit",
        })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SulfurKnockbackInput {
    pub old_velocity: Vector3,
    pub cube_position: Vector3,
    pub cube_center: Vector3,
    pub cube_height: f32,
    pub attacker_position: Vector3,
    pub attacker_eye: Vector3,
    pub attacker_look: Vector3,
    pub direction_x: f64,
    pub direction_z: f64,
    pub amount: f32,
    pub strength: f64,
    pub final_boolean: bool,
    pub resistance: f64,
    pub settings: SulfurSettings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SulfurKnockback {
    pub velocity: Vector3,
    pub dirty: bool,
    pub sound_suffix: &'static str,
}

#[must_use]
pub fn sulfur_knockback(input: SulfurKnockbackInput) -> SulfurKnockback {
    let look = input.attacker_look.normalize();
    let toward_center = input.cube_center.subtract(input.attacker_eye).normalize();
    let horizontal_angle = (look.x * toward_center.z - look.z * toward_center.x)
        .atan2(look.x * toward_center.x + look.z * toward_center.z)
        as f32;
    let direction = rotate(
        (input.direction_x as f32, input.direction_z as f32),
        1.6 * horizontal_angle,
    );

    let half_height = 0.5 * input.cube_height;
    let upper = input
        .cube_center
        .add(Vector3::new(0.0, f64::from(half_height), 0.0))
        .subtract(input.attacker_eye)
        .normalize();
    let lower = input
        .cube_center
        .add(Vector3::new(0.0, -f64::from(half_height), 0.0))
        .subtract(input.attacker_eye)
        .normalize();
    let aim = clamped_map(look.y as f32, upper.y as f32, lower.y as f32, -1.0, 1.0);
    let mut transfer = (aim * 0.5).abs();
    if aim < 0.0 {
        transfer = -transfer;
    }
    let mut horizontal = input.settings.horizontal * (1.0 - transfer);
    let mut vertical = input.settings.vertical * (1.0 + transfer);

    let relative = input.cube_position.subtract(input.attacker_position);
    let elevation = (-relative.y).atan2(relative.x.hypot(relative.z)) as f32;
    (horizontal, vertical) = rotate((horizontal, vertical), -0.8 * elevation);
    let envelope = (if input.settings.horizontal > 0.0 {
        (horizontal / input.settings.horizontal).abs()
    } else {
        0.0
    })
    .max(if input.settings.vertical > 0.0 {
        (vertical / input.settings.vertical).abs()
    } else {
        0.0
    });
    if envelope > 1.0 {
        horizontal /= envelope;
        vertical /= envelope;
    }

    let direct_scale = if input.final_boolean {
        input.strength as f32 * 0.25
    } else {
        1.0
    };
    let power = input.amount.sqrt() * direct_scale * (1.0 - input.resistance as f32);
    horizontal = (horizontal * power * 0.4).clamp(-128.0, 128.0);
    vertical = (vertical * power).clamp(-128.0, 128.0);
    let direction_length = direction.0.hypot(direction.1);
    let (normal_x, normal_z) = if direction_length < DIRECTION_EPSILON_SQUARED as f32 {
        (0.0, 0.0)
    } else {
        (
            direction.0 / direction_length,
            direction.1 / direction_length,
        )
    };
    SulfurKnockback {
        velocity: Vector3::new(
            input.old_velocity.x - f64::from(normal_x * horizontal),
            input.old_velocity.y + f64::from(vertical * 1.2),
            input.old_velocity.z - f64::from(normal_z * horizontal),
        ),
        dirty: true,
        sound_suffix: input.settings.sound_suffix,
    }
}

fn rotate(pair: (f32, f32), angle: f32) -> (f32, f32) {
    let (sin, cos) = angle.sin_cos();
    (pair.0 * cos - pair.1 * sin, pair.0 * sin + pair.1 * cos)
}

fn clamped_map(value: f32, from_start: f32, from_end: f32, to_start: f32, to_end: f32) -> f32 {
    if from_start == from_end {
        return to_start;
    }
    let ratio = ((value - from_start) / (from_end - from_start)).clamp(0.0, 1.0);
    to_start + ratio * (to_end - to_start)
}
