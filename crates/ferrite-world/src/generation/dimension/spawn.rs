//! Dimension-owned spawn, darkness, bed, and anchor decisions.

use super::DimensionType;
use super::environment::{BedRule, SleepRule, SpawnRule};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LightSample {
    pub sky_light: u8,
    pub block_light: u8,
    pub local_raw_light: u8,
    pub thunder_local_raw_light: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonsterDarknessResult {
    pub allowed: bool,
    pub random_draws: u8,
    pub sampled_spawn_limit: Option<u8>,
}

/// Evaluates the exact branch-local monster light gates.
pub fn monster_dark_enough(
    dimension: &DimensionType,
    lights: LightSample,
    weather_capable: bool,
    thundering: bool,
    mut next_bounded: impl FnMut(u32) -> u32,
) -> MonsterDarknessResult {
    let sky_roll = next_bounded(32);
    if sky_roll < u32::from(lights.sky_light) {
        return MonsterDarknessResult {
            allowed: false,
            random_draws: 1,
            sampled_spawn_limit: None,
        };
    }
    if dimension.monster_spawn_block_light_limit < 15
        && lights.block_light > dimension.monster_spawn_block_light_limit
    {
        return MonsterDarknessResult {
            allowed: false,
            random_draws: 1,
            sampled_spawn_limit: None,
        };
    }
    let mut provider_draws = 0;
    let spawn_limit = dimension.monster_spawn_light_level.sample(|bound| {
        provider_draws += 1;
        next_bounded(bound)
    });
    let local = if weather_capable && thundering {
        lights.thunder_local_raw_light
    } else {
        lights.local_raw_light
    };
    MonsterDarknessResult {
        allowed: local <= spawn_limit,
        random_draws: 1 + provider_draws,
        sampled_spawn_limit: Some(spawn_limit),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpawnColumnHeight {
    GeneratorSpawnHeight,
    MotionBlocking,
}

pub fn initial_spawn_column_height(dimension: &DimensionType) -> SpawnColumnHeight {
    if dimension.has_ceiling {
        SpawnColumnHeight::GeneratorSpawnHeight
    } else {
        SpawnColumnHeight::MotionBlocking
    }
}

pub fn natural_spawn_requires_air_descent(dimension: &DimensionType) -> bool {
    dimension.has_ceiling
}

pub fn map_sample_radius(dimension: &DimensionType, ordinary_radius: u32) -> u32 {
    if dimension.has_ceiling {
        ordinary_radius / 2
    } else {
        ordinary_radius
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SpawnCandidate {
    pub x: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialSpawnPlan {
    pub radius: u32,
    pub candidates: Vec<SpawnCandidate>,
    pub fallback: SpawnCandidate,
    pub ticket_radius: u8,
    pub ticket_kind: &'static str,
}

/// Builds the bounded candidate permutation. The callback models the sole
/// thread-local-random bounded draw used by the vanilla search.
pub fn initial_spawn_plan(
    suggestion: SpawnCandidate,
    respawn_radius: i32,
    border_distance: f64,
    adventure_mode: bool,
    mut next_bounded: impl FnMut(u32) -> u32,
) -> InitialSpawnPlan {
    if adventure_mode {
        return InitialSpawnPlan {
            radius: 0,
            candidates: Vec::new(),
            fallback: suggestion,
            ticket_radius: 0,
            ticket_kind: "spawn_search",
        };
    }
    let configured = respawn_radius.max(0) as u32;
    let radius = if border_distance <= 1.0 {
        1
    } else {
        configured.min(border_distance.floor().max(0.0) as u32)
    };
    let width = u64::from(radius) * 2 + 1;
    let full_count = width.saturating_mul(width);
    let count = full_count.min(1_024) as u32;
    let mut candidates = Vec::with_capacity(count as usize);
    if count > 0 {
        let offset = next_bounded(count);
        let step = if count <= 16 {
            count.saturating_sub(1).max(1)
        } else {
            17
        };
        for attempt in 0..count {
            let index =
                (u64::from(offset) + u64::from(attempt) * u64::from(step)) % u64::from(count);
            let local_x = (index % width) as i64 - i64::from(radius);
            let local_z = (index / width) as i64 - i64::from(radius);
            candidates.push(SpawnCandidate {
                x: suggestion.x.saturating_add(local_x as i32),
                z: suggestion.z.saturating_add(local_z as i32),
            });
        }
    }
    InitialSpawnPlan {
        radius,
        candidates,
        fallback: suggestion,
        ticket_radius: 0,
        ticket_kind: "spawn_search",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SleepDenial {
    NeverAllowed,
    NotDark,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BedInteraction {
    pub remove_both_halves: bool,
    pub explosion: Option<BadRespawnExplosion>,
    pub spawn_recorded: bool,
    pub starts_sleeping: bool,
    pub denial: Option<SleepDenial>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadRespawnExplosion {
    pub power: f32,
    pub causes_fire: bool,
    pub block_interaction: bool,
}

impl BadRespawnExplosion {
    pub const fn vanilla() -> Self {
        Self {
            power: 5.0,
            causes_fire: true,
            block_interaction: true,
        }
    }
}

pub fn interact_with_bed(rule: &BedRule, is_dark: bool) -> BedInteraction {
    if rule.explodes {
        return BedInteraction {
            remove_both_halves: true,
            explosion: Some(BadRespawnExplosion::vanilla()),
            spawn_recorded: false,
            starts_sleeping: false,
            denial: None,
        };
    }
    let spawn_recorded = rule.can_set_spawn == SpawnRule::Always;
    let (starts_sleeping, denial) = match rule.can_sleep {
        SleepRule::Always => (true, None),
        SleepRule::WhenDark if is_dark => (true, None),
        SleepRule::WhenDark => (false, Some(SleepDenial::NotDark)),
        SleepRule::Never => (false, Some(SleepDenial::NeverAllowed)),
    };
    BedInteraction {
        remove_both_halves: false,
        explosion: None,
        spawn_recorded,
        starts_sleeping,
        denial,
    }
}

pub fn sleeping_player_remains_asleep(rule: &BedRule, is_dark: bool) -> bool {
    match rule.can_sleep {
        SleepRule::Always => true,
        SleepRule::WhenDark => is_dark,
        SleepRule::Never => false,
    }
}

pub fn retained_bed_respawn_allowed(rule: &BedRule) -> bool {
    rule.can_set_spawn == SpawnRule::Always
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorInteraction {
    SetSpawn,
    Explode(BadRespawnExplosion),
    Uncharged,
}

pub fn interact_with_respawn_anchor(works_here: bool, charged: bool) -> AnchorInteraction {
    match (works_here, charged) {
        (true, true) => AnchorInteraction::SetSpawn,
        (false, true) => AnchorInteraction::Explode(BadRespawnExplosion::vanilla()),
        (_, false) => AnchorInteraction::Uncharged,
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SpawnSurfaceChecks {
    pub at_or_above_min_y: bool,
    pub valid_surface_stack: bool,
    pub full_support: bool,
    pub liquid_free: bool,
    pub collision_free: bool,
}

impl SpawnSurfaceChecks {
    pub fn accepted(self) -> bool {
        self.at_or_above_min_y
            && self.valid_surface_stack
            && self.full_support
            && self.liquid_free
            && self.collision_free
    }
}
