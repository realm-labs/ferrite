//! Blocking use, angle reduction, durability, retaliation, and disable transactions.

use std::cmp::Ordering;
use std::f64::consts::PI;

use crate::entity::runtime::ent_005::knockback::Vector3;

pub const MISSING_SOURCE_ANGLE: f64 = 3.141_592_741_012_573_2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartBlocking {
    pub admitted: bool,
    pub use_duration: u32,
    pub set_using_flag: bool,
    pub set_offhand_flag: bool,
    pub emit_interact_start: bool,
}

#[must_use]
pub const fn start_blocking(
    empty_stack: bool,
    already_using: bool,
    server_side: bool,
    offhand: bool,
    interact_vibrations: bool,
) -> StartBlocking {
    let admitted = !empty_stack && !already_using;
    StartBlocking {
        admitted,
        use_duration: if admitted { 72_000 } else { 0 },
        set_using_flag: admitted && server_side,
        set_offhand_flag: admitted && server_side && offhand,
        emit_interact_start: admitted && server_side && interact_vibrations,
    }
}

#[must_use]
pub fn blocking_stack_mature(
    using_item: bool,
    component_still_present: bool,
    use_duration: u32,
    use_remaining: u32,
    block_delay_seconds: f32,
) -> bool {
    let elapsed = use_duration.saturating_sub(use_remaining);
    let delay = (block_delay_seconds * 20.0).round() as u32;
    using_item && component_still_present && elapsed >= delay
}

#[must_use]
pub fn blocking_amount_admitted(amount: f32) -> bool {
    !matches!(
        amount.partial_cmp(&0.0),
        Some(Ordering::Less | Ordering::Equal)
    )
}

#[must_use]
pub fn incidence_angle(
    source_position: Option<Vector3>,
    victim_position: Vector3,
    victim_head_yaw_degrees: f32,
) -> f64 {
    let Some(source) = source_position else {
        return MISSING_SOURCE_ANGLE;
    };
    let horizontal = Vector3::new(
        source.x - victim_position.x,
        0.0,
        source.z - victim_position.z,
    );
    let direction = horizontal.normalize();
    let yaw = f64::from(victim_head_yaw_degrees) * PI / 180.0;
    let view = Vector3::new(-yaw.sin(), 0.0, yaw.cos());
    (direction.x * view.x + direction.z * view.z)
        .clamp(-1.0, 1.0)
        .acos()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageReduction {
    pub horizontal_angle_degrees: f32,
    pub damage_type_matches: bool,
    pub base: f32,
    pub factor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockingResolution {
    pub blocked_amount: f32,
    pub item_used_stat: bool,
    pub requested_durability: i32,
    pub retaliate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolveBlockingInput<'a> {
    pub amount: f32,
    pub mature_stack: bool,
    pub bypassed_by_damage_type: bool,
    pub piercing_arrow: bool,
    pub angle: f64,
    pub reductions: &'a [DamageReduction],
    pub player_victim: bool,
    pub durability_threshold: f32,
    pub durability_base: f32,
    pub durability_factor: f32,
    pub projectile_damage_type: bool,
    pub living_direct_attacker: bool,
}

#[must_use]
pub fn resolve_blocking(input: ResolveBlockingInput<'_>) -> BlockingResolution {
    if !blocking_amount_admitted(input.amount)
        || !input.mature_stack
        || input.bypassed_by_damage_type
        || input.piercing_arrow
    {
        return BlockingResolution {
            blocked_amount: 0.0,
            item_used_stat: false,
            requested_durability: 0,
            retaliate: false,
        };
    }
    let mut sum = 0.0_f32;
    for reduction in input.reductions {
        let maximum_angle = f64::from(0.017_453_292_f32 * reduction.horizontal_angle_degrees);
        let contribution = if input.angle > maximum_angle || !reduction.damage_type_matches {
            0.0
        } else {
            java_clamp(
                reduction.base + reduction.factor * input.amount,
                0.0,
                input.amount,
            )
        };
        sum += contribution;
    }
    let blocked_amount = java_clamp(sum, 0.0, input.amount);
    let below_threshold =
        blocked_amount.partial_cmp(&input.durability_threshold) == Some(Ordering::Less);
    let requested_durability = if input.player_victim && !below_threshold {
        (input.durability_base + input.durability_factor * blocked_amount).floor() as i32
    } else {
        0
    };
    BlockingResolution {
        blocked_amount,
        item_used_stat: input.player_victim,
        requested_durability,
        retaliate: blocked_amount > 0.0
            && !input.projectile_damage_type
            && input.living_direct_attacker,
    }
}

fn java_clamp(value: f32, minimum: f32, maximum: f32) -> f32 {
    if value.is_nan() || minimum.is_nan() || maximum.is_nan() {
        f32::NAN
    } else {
        value.max(minimum).min(maximum)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackerKind {
    Default,
    BabyHoglin,
    AdultHoglin,
    BabyZoglin,
    AdultZoglin,
    Ravager,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Retaliation {
    DefaultKnockback {
        strength: f64,
        submitted_amount: f32,
    },
    HoglinThrow,
    RavagerStun {
        stunned_ticks: u8,
        event: u8,
        push_victim: bool,
        dirty: bool,
    },
    RavagerPush {
        velocity: Vector3,
    },
    None,
}

#[must_use]
pub fn retaliation(
    kind: AttackerKind,
    submitted_amount: f32,
    ravager_roar_ticks: u8,
    ravager_draw: f64,
    attacker_minus_victim: Vector3,
) -> Retaliation {
    match kind {
        AttackerKind::BabyHoglin | AttackerKind::BabyZoglin => Retaliation::None,
        AttackerKind::AdultHoglin | AttackerKind::AdultZoglin => Retaliation::HoglinThrow,
        AttackerKind::Ravager if ravager_roar_ticks != 0 => Retaliation::None,
        AttackerKind::Ravager if ravager_draw < 0.5 => Retaliation::RavagerStun {
            stunned_ticks: 40,
            event: 39,
            push_victim: true,
            dirty: true,
        },
        AttackerKind::Ravager => {
            let divisor = (attacker_minus_victim.x * attacker_minus_victim.x
                + attacker_minus_victim.z * attacker_minus_victim.z)
                .max(0.001);
            Retaliation::RavagerPush {
                velocity: Vector3::new(
                    4.0 * attacker_minus_victim.x / divisor,
                    0.2,
                    4.0 * attacker_minus_victim.z / divisor,
                ),
            }
        }
        AttackerKind::Default => Retaliation::DefaultKnockback {
            strength: 0.5,
            submitted_amount,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinThrow {
    pub velocity: Vector3,
    pub dirty: bool,
    pub draws_consumed: u8,
}

#[must_use]
pub fn hoglin_throw(
    attack_knockback: f64,
    victim_resistance: f64,
    attacker_to_victim: Vector3,
    rotation_draw_twenty_one: i32,
    horizontal_draw: f32,
    vertical_draw: f32,
) -> HoglinThrow {
    let strength = attack_knockback - victim_resistance;
    if strength <= 0.0 {
        return HoglinThrow {
            velocity: Vector3::ZERO,
            dirty: false,
            draws_consumed: 0,
        };
    }
    let direction = Vector3::new(attacker_to_victim.x, 0.0, attacker_to_victim.z).normalize();
    let angle = f64::from(rotation_draw_twenty_one.rem_euclid(21) - 10) * PI / 180.0;
    let (sin, cos) = angle.sin_cos();
    let rotated_x = direction.x * cos - direction.z * sin;
    let rotated_z = direction.x * sin + direction.z * cos;
    let horizontal = strength * (0.2 + 0.5 * f64::from(horizontal_draw));
    HoglinThrow {
        velocity: Vector3::new(
            rotated_x * horizontal,
            strength * f64::from(vertical_draw) * 0.5,
            rotated_z * horizontal,
        ),
        dirty: true,
        draws_consumed: 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisableBlocking {
    pub cooldown_ticks: u32,
    pub stop_use: bool,
    pub emit_interact_finish: bool,
    pub play_disable_sound_after_stop: bool,
}

#[must_use]
pub fn disable_blocking(
    seconds: f32,
    cooldown_scale: f32,
    blocking_component_still_present: bool,
    interact_vibrations: bool,
    disable_sound_present: bool,
) -> DisableBlocking {
    let product = seconds * cooldown_scale * 20.0;
    let cooldown_ticks = if seconds > 0.0 && blocking_component_still_present && product > 0.0 {
        product.round() as u32
    } else {
        0
    };
    DisableBlocking {
        cooldown_ticks,
        stop_use: cooldown_ticks > 0,
        emit_interact_finish: cooldown_ticks > 0 && interact_vibrations,
        play_disable_sound_after_stop: cooldown_ticks > 0 && disable_sound_present,
    }
}

#[must_use]
pub const fn attacker_disable_seconds(
    warden: bool,
    main_hand_disable_seconds: Option<f32>,
    main_hand_is_exact_active_stack: bool,
) -> f32 {
    if warden {
        5.0
    } else if main_hand_is_exact_active_stack {
        match main_hand_disable_seconds {
            Some(seconds) => seconds,
            None => 0.0,
        }
    } else {
        0.0
    }
}

#[must_use]
pub fn block_sound_pitch(draw: f32) -> f32 {
    0.8 + 0.4 * draw
}
