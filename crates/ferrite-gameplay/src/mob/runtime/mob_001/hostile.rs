//! Hostile-spawn policy propagation and direct live-rule consumers.

pub const SPAWN_MONSTERS_DEFAULT: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    #[must_use]
    pub const fn id(self) -> u32 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostilePolicy {
    pub live: bool,
    pub replace_chunk_cache: bool,
}

#[must_use]
pub const fn refresh_hostile_policy(spawn_mobs: bool, spawn_monsters: bool) -> HostilePolicy {
    HostilePolicy {
        live: spawn_mobs && spawn_monsters,
        replace_chunk_cache: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaturalCategory {
    Monster,
    Creature,
    Ambient,
    Axolotls,
    UndergroundWaterCreature,
    WaterCreature,
    WaterAmbient,
}

#[must_use]
pub const fn category_survives_hostile_cache(
    category: NaturalCategory,
    cached_spawn_enemies: bool,
) -> bool {
    cached_spawn_enemies || !matches!(category, NaturalCategory::Monster)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomSpawnerPolicy {
    pub patrol: bool,
    pub phantom: bool,
    pub village_siege_done: bool,
    pub clear_village_siege_setup: bool,
    pub cat: bool,
    pub wandering_trader: bool,
}

#[must_use]
pub const fn custom_spawner_policy(
    cached_spawn_enemies: bool,
    daylight: bool,
) -> CustomSpawnerPolicy {
    CustomSpawnerPolicy {
        patrol: cached_spawn_enemies,
        phantom: cached_spawn_enemies,
        village_siege_done: daylight || !cached_spawn_enemies,
        clear_village_siege_setup: daylight || !cached_spawn_enemies,
        cat: true,
        wandering_trader: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndermiteAttempt {
    pub chance_draw_consumed: bool,
    pub construct: bool,
    pub copy_owner_position_and_rotation: bool,
    pub reason_triggered: bool,
    pub retry_on_failure: bool,
    pub continue_pearl_transaction: bool,
}

#[must_use]
pub const fn endermite_attempt(
    accepted_server_impact: bool,
    connected_server_player_owner: bool,
    chance_draw: f32,
    live_hostile_policy: bool,
    difficulty: Difficulty,
) -> EndermiteAttempt {
    let draw = accepted_server_impact && connected_server_player_owner;
    let construct = draw
        && chance_draw < 0.05
        && live_hostile_policy
        && !matches!(difficulty, Difficulty::Peaceful);
    EndermiteAttempt {
        chance_draw_consumed: draw,
        construct,
        copy_owner_position_and_rotation: construct,
        reason_triggered: construct,
        retry_on_failure: false,
        continue_pearl_transaction: accepted_server_impact,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReinforcementAdmission {
    pub chance_draw_consumed: bool,
    pub construct: bool,
    pub attempts: u8,
    pub draws_per_attempt: u8,
}

#[must_use]
pub const fn reinforcement_admission(
    superclass_damage_succeeded: bool,
    target_or_living_source_present: bool,
    difficulty: Difficulty,
    chance_draw: f32,
    reinforcement_chance: f32,
    live_hostile_policy: bool,
) -> ReinforcementAdmission {
    let hard = superclass_damage_succeeded
        && target_or_living_source_present
        && matches!(difficulty, Difficulty::Hard);
    let construct = hard && chance_draw < reinforcement_chance && live_hostile_policy;
    ReinforcementAdmission {
        chance_draw_consumed: hard,
        construct,
        attempts: if construct { 50 } else { 0 },
        draws_per_attempt: if construct { 6 } else { 0 },
    }
}

#[must_use]
pub const fn reinforcement_offset(distance_draw: u32, sign_draw: u32) -> i32 {
    let distance = 7 + (distance_draw % 34) as i32;
    let sign = (sign_draw % 3) as i32 - 1;
    distance * sign
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReinforcementCandidateFailure {
    PlacementPosition,
    RegisteredRule,
    NearbyAlivePlayer,
    Obstructed,
    Collision,
    Liquid,
}

pub const fn reinforcement_candidate(
    valid_position: bool,
    registered_rule: bool,
    alive_player_within_seven: bool,
    unobstructed_aabb: bool,
    collision_free: bool,
    subtype_can_spawn_in_liquid: bool,
    contains_liquid: bool,
) -> Result<(), ReinforcementCandidateFailure> {
    if !valid_position {
        Err(ReinforcementCandidateFailure::PlacementPosition)
    } else if !registered_rule {
        Err(ReinforcementCandidateFailure::RegisteredRule)
    } else if alive_player_within_seven {
        Err(ReinforcementCandidateFailure::NearbyAlivePlayer)
    } else if !unobstructed_aabb {
        Err(ReinforcementCandidateFailure::Obstructed)
    } else if !collision_free {
        Err(ReinforcementCandidateFailure::Collision)
    } else if !subtype_can_spawn_in_liquid && contains_liquid {
        Err(ReinforcementCandidateFailure::Liquid)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReinforcementSuccess {
    pub finalize_and_insert_with_passengers: bool,
    pub caller_permanent_add: f64,
    pub callee_fixed_permanent_add: bool,
    pub rollback_modifiers_on_insert_failure: bool,
}

#[must_use]
pub fn reinforcement_success(existing_caller_add: Option<f64>) -> ReinforcementSuccess {
    ReinforcementSuccess {
        finalize_and_insert_with_passengers: true,
        caller_permanent_add: existing_caller_add.unwrap_or(0.0) - 0.05,
        callee_fixed_permanent_add: true,
        rollback_modifiers_on_insert_failure: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreakingProtectorAttempt {
    pub ticker_draw_consumed: bool,
    pub ticker: u8,
    pub query_nearest_player: bool,
    pub call_spawn_util: bool,
    pub spawn_attempts: u8,
    pub horizontal_range: u8,
    pub vertical_range: u8,
}

#[must_use]
pub const fn creaking_protector_attempt(
    ticker_draw: u8,
    uprooted: bool,
    tracks_protector: bool,
    awake: bool,
    live_hostile_policy: bool,
    difficulty: Difficulty,
    nearest_player_present: bool,
) -> CreakingProtectorAttempt {
    let eligible = !uprooted
        && !tracks_protector
        && awake
        && live_hostile_policy
        && !matches!(difficulty, Difficulty::Peaceful);
    CreakingProtectorAttempt {
        ticker_draw_consumed: true,
        ticker: 20 + ticker_draw % 5,
        query_nearest_player: eligible,
        call_spawn_util: eligible && nearest_player_present,
        spawn_attempts: if eligible && nearest_player_present {
            5
        } else {
            0
        },
        horizontal_range: 16,
        vertical_range: 8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalPiglinAttempt {
    pub chance_draw_consumed: bool,
    pub descend_portal_column: bool,
    pub construct: bool,
    pub set_entity_and_vehicle_cooldown: bool,
}

#[must_use]
pub const fn portal_piglin_attempt(input: PortalPiglinInput) -> PortalPiglinAttempt {
    let gated = input.live_hostile_policy
        && !matches!(input.difficulty, Difficulty::Peaceful)
        && input.environment_attribute;
    let chance = gated && input.chance_draw_below_2000 < input.difficulty.id();
    let construct =
        chance && input.player_close_enough && input.valid_ground && input.creation_succeeded;
    PortalPiglinAttempt {
        chance_draw_consumed: gated,
        descend_portal_column: chance && input.player_close_enough,
        construct,
        set_entity_and_vehicle_cooldown: construct,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalPiglinInput {
    pub live_hostile_policy: bool,
    pub difficulty: Difficulty,
    pub environment_attribute: bool,
    pub chance_draw_below_2000: u32,
    pub player_close_enough: bool,
    pub valid_ground: bool,
    pub creation_succeeded: bool,
}
