//! Ordinary spawner countdown, batch retry, and spawn-egg semantics.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_simulation::random::DeterministicRng;
use std::num::NonZeroU64;

pub const SPAWNER_EVENT: i32 = 1;
pub const SPAWN_EFFECT_EVENT: i32 = 2004;
pub const SELECTED_DATA_UPDATE_FLAGS: u16 = 260;
pub const SPAWN_EGG_UPDATE_FLAGS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnerConfig {
    pub minimum_delay: i32,
    pub maximum_delay: i32,
    pub spawn_count: i32,
    pub maximum_nearby_entities: i32,
    pub required_player_range: i32,
    pub spawn_range: i32,
}

impl Default for SpawnerConfig {
    fn default() -> Self {
        Self {
            minimum_delay: 200,
            maximum_delay: 800,
            spawn_count: 4,
            maximum_nearby_entities: 6,
            required_player_range: 16,
            spawn_range: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedSpawnerSettings {
    pub delay: i16,
    pub minimum_delay: i16,
    pub maximum_delay: i16,
    pub spawn_count: i16,
    pub maximum_nearby_entities: i16,
    pub required_player_range: i16,
    pub spawn_range: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrdinarySpawner {
    pub delay: i32,
    pub selected_data_present: bool,
}

impl Default for OrdinarySpawner {
    fn default() -> Self {
        Self {
            delay: 20,
            selected_data_present: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnAttempt {
    SkipCollision,
    SkipPositionRule,
    SkipMobRuleOrObstruction,
    AbortNullLoad,
    AbortNearbyLimit,
    AbortAdmission,
    SuccessNonMob,
    SuccessMob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnerEffect {
    SelectPotential,
    PublishSelectedData { flags: u16 },
    BroadcastBlockEvent(i32),
    DecrementDelay,
    Attempt(i32),
    SpawnLevelEvent(i32),
    EntityPlaceGameEvent,
    MobSpawnAnimation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnerTick {
    pub effects: Vec<SpawnerEffect>,
    pub successes: u32,
    pub read_rule: bool,
}

#[derive(Debug)]
pub struct SpawnerTickInputs<'a> {
    pub has_required_player: bool,
    pub spawners_work: bool,
    pub potentials_available: bool,
    pub selected_entity_type_valid: bool,
    pub attempts: &'a [SpawnAttempt],
}

impl OrdinarySpawner {
    pub const fn save_settings(self, config: SpawnerConfig) -> SavedSpawnerSettings {
        SavedSpawnerSettings {
            delay: self.delay as i16,
            minimum_delay: config.minimum_delay as i16,
            maximum_delay: config.maximum_delay as i16,
            spawn_count: config.spawn_count as i16,
            maximum_nearby_entities: config.maximum_nearby_entities as i16,
            required_player_range: config.required_player_range as i16,
            spawn_range: config.spawn_range as i16,
        }
    }

    pub const fn load_settings(
        saved: SavedSpawnerSettings,
        selected_data_present: bool,
    ) -> (Self, SpawnerConfig) {
        (
            Self {
                delay: saved.delay as i32,
                selected_data_present,
            },
            SpawnerConfig {
                minimum_delay: saved.minimum_delay as i32,
                maximum_delay: saved.maximum_delay as i32,
                spawn_count: saved.spawn_count as i32,
                maximum_nearby_entities: saved.maximum_nearby_entities as i32,
                required_player_range: saved.required_player_range as i32,
                spawn_range: saved.spawn_range as i32,
            },
        )
    }

    pub fn server_tick(
        &mut self,
        config: SpawnerConfig,
        inputs: SpawnerTickInputs<'_>,
        random: &mut DeterministicRng,
    ) -> SpawnerTick {
        if !inputs.has_required_player {
            return SpawnerTick {
                effects: Vec::new(),
                successes: 0,
                read_rule: false,
            };
        }
        if !inputs.spawners_work {
            return SpawnerTick {
                effects: Vec::new(),
                successes: 0,
                read_rule: true,
            };
        }

        let mut effects = Vec::new();
        if self.delay == -1 {
            self.reset_delay(config, inputs.potentials_available, random, &mut effects);
        }
        if self.delay > 0 {
            self.delay -= 1;
            effects.push(SpawnerEffect::DecrementDelay);
            return SpawnerTick {
                effects,
                successes: 0,
                read_rule: true,
            };
        }
        self.ensure_selected(inputs.potentials_available, &mut effects);
        if !inputs.selected_entity_type_valid {
            self.reset_delay(config, inputs.potentials_available, random, &mut effects);
            return SpawnerTick {
                effects,
                successes: 0,
                read_rule: true,
            };
        }

        let mut successes = 0;
        for attempt_index in 0..config.spawn_count.max(0) {
            effects.push(SpawnerEffect::Attempt(attempt_index));
            let outcome = inputs
                .attempts
                .get(attempt_index as usize)
                .copied()
                .unwrap_or(SpawnAttempt::SkipCollision);
            match outcome {
                SpawnAttempt::SkipCollision
                | SpawnAttempt::SkipPositionRule
                | SpawnAttempt::SkipMobRuleOrObstruction => {}
                SpawnAttempt::AbortNullLoad
                | SpawnAttempt::AbortNearbyLimit
                | SpawnAttempt::AbortAdmission => {
                    self.reset_delay(config, inputs.potentials_available, random, &mut effects);
                    break;
                }
                SpawnAttempt::SuccessNonMob | SpawnAttempt::SuccessMob => {
                    successes += 1;
                    effects.extend([
                        SpawnerEffect::SpawnLevelEvent(SPAWN_EFFECT_EVENT),
                        SpawnerEffect::EntityPlaceGameEvent,
                    ]);
                    if outcome == SpawnAttempt::SuccessMob {
                        effects.push(SpawnerEffect::MobSpawnAnimation);
                    }
                }
            }
        }
        if successes > 0 && self.delay <= 0 {
            self.reset_delay(config, inputs.potentials_available, random, &mut effects);
        }
        SpawnerTick {
            effects,
            successes,
            read_rule: true,
        }
    }

    fn ensure_selected(&mut self, potentials_available: bool, effects: &mut Vec<SpawnerEffect>) {
        if self.selected_data_present {
            return;
        }
        if potentials_available {
            effects.push(SpawnerEffect::SelectPotential);
        }
        self.selected_data_present = true;
        effects.push(SpawnerEffect::PublishSelectedData {
            flags: SELECTED_DATA_UPDATE_FLAGS,
        });
    }

    fn reset_delay(
        &mut self,
        config: SpawnerConfig,
        potentials_available: bool,
        random: &mut DeterministicRng,
        effects: &mut Vec<SpawnerEffect>,
    ) {
        self.delay = if config.maximum_delay <= config.minimum_delay {
            config.minimum_delay
        } else {
            let width = (config.maximum_delay - config.minimum_delay) as u64;
            let upper = NonZeroU64::new(width).expect("positive delay width");
            config.minimum_delay + random.uniform_u64(upper) as i32
        };
        if potentials_available {
            effects.push(SpawnerEffect::SelectPotential);
            self.selected_data_present = true;
            effects.push(SpawnerEffect::PublishSelectedData {
                flags: SELECTED_DATA_UPDATE_FLAGS,
            });
        }
        effects.push(SpawnerEffect::BroadcastBlockEvent(SPAWNER_EVENT));
    }
}

pub const fn player_in_required_range(
    squared_distance_to_center: f64,
    required_range: i32,
) -> bool {
    required_range < 0
        || squared_distance_to_center < (required_range as f64) * (required_range as f64)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandidatePositionDraws {
    pub first_x: f64,
    pub second_x: f64,
    pub y_offset: i32,
    pub first_z: f64,
    pub second_z: f64,
}

pub fn candidate_position(
    spawner: BlockPos,
    spawn_range: i32,
    explicit: Option<[f64; 3]>,
    draws: CandidatePositionDraws,
) -> [f64; 3] {
    explicit.unwrap_or([
        f64::from(spawner.x) + (draws.first_x - draws.second_x) * f64::from(spawn_range) + 0.5,
        f64::from(spawner.y.wrapping_add(draws.y_offset).wrapping_sub(1)),
        f64::from(spawner.z) + (draws.first_z - draws.second_z) * f64::from(spawn_range) + 0.5,
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnerKind {
    Ordinary,
    Trial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnEggEffect {
    FailureMessage,
    SelectPotential,
    OverrideOrdinarySelectedData,
    ResetTrialEncounterAndConfigs,
    MarkChanged,
    SameStateUpdate { flags: u16 },
    BlockChangeGameEvent,
    ShrinkStack,
}

pub fn plan_spawn_egg_edit(
    kind: SpawnerKind,
    server_player: bool,
    spawners_work: bool,
    selected_data_present: bool,
) -> (bool, Vec<SpawnEggEffect>) {
    if !spawners_work {
        return (
            false,
            if server_player {
                vec![SpawnEggEffect::FailureMessage]
            } else {
                Vec::new()
            },
        );
    }
    let mut effects = Vec::new();
    match kind {
        SpawnerKind::Ordinary => {
            if !selected_data_present {
                effects.push(SpawnEggEffect::SelectPotential);
            }
            effects.extend([
                SpawnEggEffect::OverrideOrdinarySelectedData,
                SpawnEggEffect::MarkChanged,
            ]);
        }
        SpawnerKind::Trial => effects.extend([
            SpawnEggEffect::ResetTrialEncounterAndConfigs,
            SpawnEggEffect::MarkChanged,
        ]),
    }
    effects.extend([
        SpawnEggEffect::SameStateUpdate {
            flags: SPAWN_EGG_UPDATE_FLAGS,
        },
        SpawnEggEffect::BlockChangeGameEvent,
        SpawnEggEffect::ShrinkStack,
    ]);
    (true, effects)
}

pub fn client_spin(previous: f64, delay: i32) -> f64 {
    (previous + 1_000.0 / f64::from(delay + 200)).rem_euclid(360.0)
}

pub fn display_scale(width: f32, height: f32) -> f32 {
    let maximum = width.max(height);
    if maximum > 1.0 {
        0.53125 / maximum
    } else {
        0.53125
    }
}
