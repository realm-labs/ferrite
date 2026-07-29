//! Trial-spawner encounter counters, timing, and six-state transition kernel.

use ferrite_foundation::coordinate::BlockPos;

pub const PLAYER_SCAN_PERIOD: i64 = 20;
pub const TRACKING_DISTANCE_SQUARED: i32 = 47 * 47;
pub const INITIAL_SPAWN_BUFFER: i64 = 40;
pub const REWARD_OPEN_DELAY: i64 = 40;
pub const REWARD_EJECTION_PERIOD: i64 = 30;
pub const OMINOUS_ITEM_PERIOD: i64 = 160;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialSpawnerState {
    Inactive,
    WaitingForPlayers,
    Active,
    WaitingForRewardEjection,
    EjectingReward,
    Cooldown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrialSpawnerConfig {
    pub total_mobs: f32,
    pub simultaneous_mobs: f32,
    pub added_total_per_player: f32,
    pub added_simultaneous_per_player: f32,
    pub ticks_between_spawn: i64,
    pub required_player_range: f64,
    pub target_cooldown: i64,
}

impl Default for TrialSpawnerConfig {
    fn default() -> Self {
        Self {
            total_mobs: 6.0,
            simultaneous_mobs: 2.0,
            added_total_per_player: 2.0,
            added_simultaneous_per_player: 1.0,
            ticks_between_spawn: 40,
            required_player_range: 14.0,
            target_cooldown: 36_000,
        }
    }
}

impl TrialSpawnerConfig {
    pub fn target_total(self, registered_players: usize) -> i32 {
        target(
            self.total_mobs,
            self.added_total_per_player,
            registered_players,
        )
    }

    pub fn target_simultaneous(self, registered_players: usize) -> i32 {
        target(
            self.simultaneous_mobs,
            self.added_simultaneous_per_player,
            registered_players,
        )
    }
}

fn target(base: f32, added: f32, registered_players: usize) -> i32 {
    let additional = registered_players.saturating_sub(1) as f32;
    (base + added * additional).floor() as i32
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialSpawner {
    pub state: TrialSpawnerState,
    pub ominous: bool,
    pub registered_players: usize,
    pub current_mobs: usize,
    pub total_spawned: i32,
    pub next_mob_spawns_at: i64,
    pub cooldown_ends_at: i64,
    pub dispensing_players: usize,
    pub ejection_table_fixed: bool,
}

impl Default for TrialSpawner {
    fn default() -> Self {
        Self {
            state: TrialSpawnerState::Inactive,
            ominous: false,
            registered_players: 0,
            current_mobs: 0,
            total_spawned: 0,
            next_mob_spawns_at: 0,
            cooldown_ends_at: 0,
            dispensing_players: 0,
            ejection_table_fixed: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrialTickInputs {
    pub now: i64,
    pub encounters_enabled: bool,
    pub selected_entity_usable: bool,
    pub tracked_entities_removed: usize,
    pub newly_registered_players: usize,
    pub converted_to_ominous: bool,
    pub mob_attempt_succeeded: bool,
    pub reward_result_nonempty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialEffect {
    PostponeSpawnAfterPrune,
    RegisterPlayers(usize),
    OmenConversion,
    DiscardTrackedEntities,
    ClearEncounter,
    AttemptMob,
    SpawnMob,
    SelectAndPublishNextPotential,
    OpenShutter,
    FixEjectionTable,
    EvaluateReward,
    EjectReward,
    RewardEvent,
    CloseShutter,
    ResetCooldown,
    StateWrite(TrialSpawnerState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialTick {
    pub effects: Vec<TrialEffect>,
}

impl TrialSpawner {
    pub fn server_tick(
        &mut self,
        config: TrialSpawnerConfig,
        inputs: TrialTickInputs,
    ) -> TrialTick {
        let mut effects = Vec::new();
        let registered_before_scan = self.registered_players;
        if inputs.tracked_entities_removed > 0 {
            self.current_mobs = self
                .current_mobs
                .saturating_sub(inputs.tracked_entities_removed);
            self.next_mob_spawns_at = inputs.now + config.ticks_between_spawn;
            effects.push(TrialEffect::PostponeSpawnAfterPrune);
        }
        let converted_to_ominous = inputs.encounters_enabled && inputs.converted_to_ominous;
        if converted_to_ominous {
            self.ominous = true;
            self.current_mobs = 0;
            self.total_spawned = 0;
            self.next_mob_spawns_at = inputs.now + config.ticks_between_spawn;
            effects.extend([
                TrialEffect::OmenConversion,
                TrialEffect::DiscardTrackedEntities,
                TrialEffect::ClearEncounter,
            ]);
        }
        if inputs.encounters_enabled && inputs.newly_registered_players > 0 {
            self.registered_players = self
                .registered_players
                .saturating_add(inputs.newly_registered_players);
            self.next_mob_spawns_at = self
                .next_mob_spawns_at
                .max(inputs.now + INITIAL_SPAWN_BUFFER);
            effects.push(TrialEffect::RegisterPlayers(
                inputs.newly_registered_players,
            ));
        }

        let next = match self.state {
            TrialSpawnerState::Inactive => {
                if inputs.selected_entity_usable {
                    TrialSpawnerState::WaitingForPlayers
                } else {
                    TrialSpawnerState::Inactive
                }
            }
            TrialSpawnerState::WaitingForPlayers => {
                if !inputs.encounters_enabled {
                    self.clear_encounter_statistics();
                    effects.push(TrialEffect::ClearEncounter);
                    TrialSpawnerState::WaitingForPlayers
                } else if !inputs.selected_entity_usable {
                    TrialSpawnerState::Inactive
                } else if self.registered_players > 0 {
                    TrialSpawnerState::Active
                } else {
                    TrialSpawnerState::WaitingForPlayers
                }
            }
            TrialSpawnerState::Active => {
                self.tick_active(config, inputs, registered_before_scan, &mut effects)
            }
            TrialSpawnerState::WaitingForRewardEjection => {
                let cooldown_started_at =
                    self.cooldown_ends_at.wrapping_sub(config.target_cooldown);
                if (inputs.now as f32) >= (cooldown_started_at as f32) + REWARD_OPEN_DELAY as f32 {
                    effects.push(TrialEffect::OpenShutter);
                    TrialSpawnerState::EjectingReward
                } else {
                    TrialSpawnerState::WaitingForRewardEjection
                }
            }
            TrialSpawnerState::EjectingReward => self.tick_ejection(config, inputs, &mut effects),
            TrialSpawnerState::Cooldown => {
                if converted_to_ominous && self.registered_players > 0 {
                    self.total_spawned = 0;
                    self.next_mob_spawns_at = 0;
                    TrialSpawnerState::Active
                } else if inputs.now >= self.cooldown_ends_at {
                    self.ominous = false;
                    self.registered_players = 0;
                    self.clear_encounter_statistics();
                    effects.push(TrialEffect::ResetCooldown);
                    TrialSpawnerState::WaitingForPlayers
                } else {
                    TrialSpawnerState::Cooldown
                }
            }
        };
        if next != self.state {
            self.state = next;
            effects.push(TrialEffect::StateWrite(next));
        }
        TrialTick { effects }
    }

    fn tick_active(
        &mut self,
        config: TrialSpawnerConfig,
        inputs: TrialTickInputs,
        target_registered_players: usize,
        effects: &mut Vec<TrialEffect>,
    ) -> TrialSpawnerState {
        if !inputs.encounters_enabled {
            self.clear_encounter_statistics();
            effects.push(TrialEffect::ClearEncounter);
            return TrialSpawnerState::WaitingForPlayers;
        }
        if !inputs.selected_entity_usable {
            return TrialSpawnerState::Inactive;
        }
        let total_target = config.target_total(target_registered_players);
        if self.total_spawned >= total_target {
            if self.current_mobs == 0 {
                self.cooldown_ends_at = inputs.now + config.target_cooldown;
                self.total_spawned = 0;
                self.next_mob_spawns_at = 0;
                self.dispensing_players = self.registered_players;
                return TrialSpawnerState::WaitingForRewardEjection;
            }
            return TrialSpawnerState::Active;
        }
        let simultaneous_target = config.target_simultaneous(target_registered_players);
        if inputs.now >= self.next_mob_spawns_at && (self.current_mobs as i32) < simultaneous_target
        {
            effects.push(TrialEffect::AttemptMob);
            if inputs.mob_attempt_succeeded {
                self.current_mobs += 1;
                self.total_spawned += 1;
                self.next_mob_spawns_at = inputs.now + config.ticks_between_spawn;
                effects.extend([
                    TrialEffect::SpawnMob,
                    TrialEffect::SelectAndPublishNextPotential,
                ]);
            }
        }
        TrialSpawnerState::Active
    }

    fn tick_ejection(
        &mut self,
        config: TrialSpawnerConfig,
        inputs: TrialTickInputs,
        effects: &mut Vec<TrialEffect>,
    ) -> TrialSpawnerState {
        let cooldown_started_at = self.cooldown_ends_at.wrapping_sub(config.target_cooldown);
        let elapsed = inputs.now.wrapping_sub(cooldown_started_at) as f32;
        if elapsed % REWARD_EJECTION_PERIOD as f32 != 0.0 {
            return TrialSpawnerState::EjectingReward;
        }
        if self.dispensing_players == 0 {
            self.ejection_table_fixed = false;
            effects.push(TrialEffect::CloseShutter);
            return TrialSpawnerState::Cooldown;
        }
        if !self.ejection_table_fixed {
            self.ejection_table_fixed = true;
            effects.push(TrialEffect::FixEjectionTable);
        }
        effects.push(TrialEffect::EvaluateReward);
        if inputs.reward_result_nonempty {
            effects.extend([TrialEffect::EjectReward, TrialEffect::RewardEvent]);
        }
        self.dispensing_players -= 1;
        TrialSpawnerState::EjectingReward
    }

    fn clear_encounter_statistics(&mut self) {
        self.registered_players = 0;
        self.current_mobs = 0;
        self.total_spawned = 0;
        self.next_mob_spawns_at = 0;
    }
}

pub const fn player_scan_due(position: BlockPos, game_time: i64) -> bool {
    position_as_long(position).wrapping_add(game_time) % PLAYER_SCAN_PERIOD == 0
}

pub fn player_in_detection_range(squared_block_distance: f64, required_range: f64) -> bool {
    squared_block_distance < required_range * required_range
}

pub const fn tracked_entity_retained(
    alive: bool,
    same_dimension: bool,
    squared_block_distance: i32,
) -> bool {
    alive && same_dimension && squared_block_distance <= TRACKING_DISTANCE_SQUARED
}

pub const fn trial_omen_duration(bad_omen_amplifier: u8) -> i32 {
    18_000 * (bad_omen_amplifier as i32 + 1)
}

const fn position_as_long(position: BlockPos) -> i64 {
    ((position.x as i64 & 0x3ff_ffff) << 38)
        | ((position.z as i64 & 0x3ff_ffff) << 12)
        | (position.y as i64 & 0xfff)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OminousItemInputs {
    pub weighted_item_present: bool,
    pub timer_due: bool,
    pub chosen_side_has_targets: bool,
    pub target_count: usize,
    pub geometry_clear: bool,
    pub admission_succeeded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OminousItemEffect {
    SelectWeightedItem,
    ChooseMobOrPlayer,
    ChooseTargetIndex,
    OfferItemSpawner,
    PlayBeginSound,
    AdvanceTimer,
}

pub fn plan_ominous_item_attempt(inputs: OminousItemInputs) -> Vec<OminousItemEffect> {
    let mut effects = vec![OminousItemEffect::SelectWeightedItem];
    if !inputs.weighted_item_present || !inputs.timer_due {
        return effects;
    }
    effects.push(OminousItemEffect::ChooseMobOrPlayer);
    if !inputs.chosen_side_has_targets {
        return effects;
    }
    if inputs.target_count > 1 {
        effects.push(OminousItemEffect::ChooseTargetIndex);
    }
    if !inputs.geometry_clear {
        return effects;
    }
    effects.push(OminousItemEffect::OfferItemSpawner);
    let _ = inputs.admission_succeeded;
    effects.extend([
        OminousItemEffect::PlayBeginSound,
        OminousItemEffect::AdvanceTimer,
    ]);
    effects
}
