//! Vault key admission, rewarded exclusion, and reverse reward ejection.

use std::collections::VecDeque;

pub const VAULT_SCAN_PERIOD: i64 = 20;
pub const UNLOCK_DELAY: i64 = 14;
pub const EJECTION_DELAY: i64 = 20;
pub const FAILURE_SOUND_BUFFER: i64 = 15;
pub const MAX_REWARDED_PLAYERS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultState {
    Inactive,
    Active,
    Unlocking,
    Ejecting,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VaultServerData {
    pub rewarded_players: VecDeque<u128>,
    pub state_updating_resumes_at: i64,
    pub pending_rewards: Vec<u64>,
    pub total_ejections: usize,
    pub last_failure_time: i64,
}

impl VaultServerData {
    pub fn load_from(&mut self, decoded: &Self) {
        self.rewarded_players = decoded.rewarded_players.clone();
        self.state_updating_resumes_at = decoded.state_updating_resumes_at;
        self.pending_rewards = decoded.pending_rewards.clone();
    }

    pub fn append_rewarded(&mut self, player: u128) {
        if self.rewarded_players.contains(&player) {
            return;
        }
        self.rewarded_players.push_back(player);
        if self.rewarded_players.len() > MAX_REWARDED_PLAYERS {
            self.rewarded_players.pop_front();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vault {
    pub state: VaultState,
    pub display_item: Option<u64>,
    pub connected_players: usize,
    pub server: VaultServerData,
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            state: VaultState::Inactive,
            display_item: None,
            connected_players: 0,
            server: VaultServerData::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultUseResult {
    TryWithEmptyHand,
    SuccessServer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultKeyInputs {
    pub logical_server: bool,
    pub block_entity_present: bool,
    pub config_key_empty: bool,
    pub key_matches_exactly: bool,
    pub sufficient_count: bool,
    pub player_already_rewarded: bool,
    pub infinite_materials: bool,
    pub now: i64,
    pub player_id: u128,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VaultEffect {
    CycleDisplay,
    InsertFailureSound,
    RejectRewardedSound,
    AwardItemUsed,
    ConsumeConfiguredKey,
    ResolveReward,
    RecomputeConnections,
    SetBlockState(VaultState),
    ExitState(VaultState),
    EnterState(VaultState),
    InsertSound,
    OpenShutterSound,
    CloseShutterSound,
    FillDisplayIfEmpty,
    ActivationEvent,
    DeactivationEvent,
    EjectReward {
        stack: u64,
        progress: f32,
        pitch: f32,
    },
    EjectionEvent,
}

impl Vault {
    pub fn use_key(
        &mut self,
        held_stack_empty: bool,
        inputs: VaultKeyInputs,
        resolved_rewards: &[u64],
    ) -> (VaultUseResult, Vec<VaultEffect>) {
        if held_stack_empty || self.state != VaultState::Active {
            return (VaultUseResult::TryWithEmptyHand, Vec::new());
        }
        if !inputs.logical_server {
            return (VaultUseResult::SuccessServer, Vec::new());
        }
        if !inputs.block_entity_present
            || inputs.config_key_empty
            || self.state != VaultState::Active
        {
            return (VaultUseResult::SuccessServer, Vec::new());
        }

        if !inputs.key_matches_exactly || !inputs.sufficient_count {
            return (
                VaultUseResult::SuccessServer,
                self.failure_effect(inputs.now, VaultEffect::InsertFailureSound),
            );
        }
        if inputs.player_already_rewarded {
            return (
                VaultUseResult::SuccessServer,
                self.failure_effect(inputs.now, VaultEffect::RejectRewardedSound),
            );
        }

        let mut effects = vec![VaultEffect::ResolveReward];
        if resolved_rewards.is_empty() {
            return (VaultUseResult::SuccessServer, effects);
        }
        effects.push(VaultEffect::AwardItemUsed);
        if !inputs.infinite_materials {
            effects.push(VaultEffect::ConsumeConfiguredKey);
        }
        self.server.pending_rewards = resolved_rewards.to_vec();
        self.server.total_ejections = resolved_rewards.len();
        self.display_item = resolved_rewards.last().copied();
        self.server.state_updating_resumes_at = inputs.now + UNLOCK_DELAY;
        self.transition(VaultState::Unlocking, &mut effects);
        self.server.append_rewarded(inputs.player_id);
        effects.push(VaultEffect::RecomputeConnections);
        (VaultUseResult::SuccessServer, effects)
    }

    pub fn server_tick(
        &mut self,
        now: i64,
        detected_players: usize,
        cycled_display: Option<u64>,
    ) -> Vec<VaultEffect> {
        let captured = self.state;
        let mut effects = Vec::new();
        if captured == VaultState::Active && now % VAULT_SCAN_PERIOD == 0 {
            self.display_item = cycled_display;
            effects.push(VaultEffect::CycleDisplay);
        }
        if now < self.server.state_updating_resumes_at {
            return effects;
        }

        let next = match captured {
            VaultState::Inactive | VaultState::Active => {
                self.connected_players = detected_players;
                self.server.state_updating_resumes_at = now + VAULT_SCAN_PERIOD;
                if detected_players == 0 {
                    VaultState::Inactive
                } else {
                    VaultState::Active
                }
            }
            VaultState::Unlocking => {
                self.server.state_updating_resumes_at = now + EJECTION_DELAY;
                VaultState::Ejecting
            }
            VaultState::Ejecting => {
                if let Some(stack) = self.server.pending_rewards.pop() {
                    let progress = ejection_progress(
                        self.server.pending_rewards.len() + 1,
                        self.server.total_ejections,
                    );
                    self.display_item = self.server.pending_rewards.last().copied();
                    self.server.state_updating_resumes_at = now + EJECTION_DELAY;
                    effects.extend([
                        VaultEffect::EjectReward {
                            stack,
                            progress,
                            pitch: ejection_pitch(progress),
                        },
                        VaultEffect::EjectionEvent,
                    ]);
                    VaultState::Ejecting
                } else {
                    self.server.total_ejections = 0;
                    self.connected_players = detected_players;
                    self.server.state_updating_resumes_at = now + VAULT_SCAN_PERIOD;
                    if detected_players == 0 {
                        VaultState::Inactive
                    } else {
                        VaultState::Active
                    }
                }
            }
        };
        if next != captured {
            self.transition(next, &mut effects);
        }
        effects
    }

    fn failure_effect(&mut self, now: i64, effect: VaultEffect) -> Vec<VaultEffect> {
        if now >= self.server.last_failure_time + FAILURE_SOUND_BUFFER {
            self.server.last_failure_time = now;
            vec![effect]
        } else {
            Vec::new()
        }
    }

    fn transition(&mut self, next: VaultState, effects: &mut Vec<VaultEffect>) {
        let old = self.state;
        self.state = next;
        effects.extend([
            VaultEffect::SetBlockState(next),
            VaultEffect::ExitState(old),
        ]);
        if old == VaultState::Ejecting {
            effects.push(VaultEffect::CloseShutterSound);
        }
        effects.push(VaultEffect::EnterState(next));
        match next {
            VaultState::Inactive => {
                self.display_item = None;
                effects.push(VaultEffect::DeactivationEvent);
            }
            VaultState::Active => {
                effects.extend([
                    VaultEffect::FillDisplayIfEmpty,
                    VaultEffect::ActivationEvent,
                ]);
            }
            VaultState::Unlocking => effects.push(VaultEffect::InsertSound),
            VaultState::Ejecting => effects.push(VaultEffect::OpenShutterSound),
        }
    }
}

pub fn ejection_progress(current_size: usize, total_ejections: usize) -> f32 {
    if total_ejections == 1 {
        1.0
    } else {
        1.0 - (current_size as f32 - 1.0) / (total_ejections as f32 - 1.0)
    }
}

pub const fn ejection_pitch(progress: f32) -> f32 {
    0.8 + 0.4 * progress
}

pub fn within_strict_scan_radius(squared_block_distance: f64, radius: f64) -> bool {
    squared_block_distance < radius * radius
}

pub fn within_particle_radius(squared_block_distance: f64, radius: f64) -> bool {
    squared_block_distance <= radius * radius
}

pub fn client_spin(previous: f32) -> f32 {
    (previous + 10.0).rem_euclid(360.0)
}
