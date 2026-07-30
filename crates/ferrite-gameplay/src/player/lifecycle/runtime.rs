//! Region-neutral lifecycle transaction owner.

use std::collections::BTreeMap;

use ferrite_foundation::identity::StableEntityId;
use thiserror::Error;

use crate::player::lifecycle::model::{
    GameMode, JoinProjection, LifecycleEffect, LifecycleSnapshot, PermissionLevel,
    PlayerLifecycleState, RespawnOutcome, RespawnProjection, RespawnRequest,
};

#[derive(Debug)]
pub struct PlayerLifecycle {
    capacity: usize,
    players: BTreeMap<StableEntityId, PlayerLifecycleState>,
}

impl PlayerLifecycle {
    pub fn new(capacity: usize) -> Result<Self, LifecycleError> {
        if capacity == 0 {
            return Err(LifecycleError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            players: BTreeMap::new(),
        })
    }

    pub fn join(
        &mut self,
        player: StableEntityId,
        session_epoch: u64,
        transferred: bool,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        if self.players.contains_key(&player) {
            return Err(LifecycleError::DuplicatePlayer(player));
        }
        if self.players.len() == self.capacity {
            return Err(LifecycleError::Capacity {
                capacity: self.capacity,
            });
        }
        let effects = join_effects(transferred);
        self.players
            .insert(player, PlayerLifecycleState::initial(player, session_epoch));
        Ok(effects)
    }

    pub fn mark_won_game(&mut self, player: StableEntityId) -> Result<(), LifecycleError> {
        self.player_mut(player)?.won_game = true;
        Ok(())
    }

    pub fn die(&mut self, player: StableEntityId) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        let state = self.player_mut(player)?;
        state.health = 0;
        state.waiting_for_respawn = true;
        Ok(vec![
            LifecycleEffect::LastDeathStored,
            LifecycleEffect::WaitingForRespawnSet,
        ])
    }

    pub fn respawn(
        &mut self,
        player: StableEntityId,
        request: RespawnRequest,
    ) -> Result<RespawnOutcome, LifecycleError> {
        let state = self.player_mut(player)?;
        let keep_all = if state.won_game {
            state.won_game = false;
            true
        } else if state.health > 0 {
            return Ok(RespawnOutcome::IgnoredAlive);
        } else {
            false
        };
        state.incarnation = state
            .incarnation
            .checked_add(1)
            .ok_or(LifecycleError::IncarnationExhausted(player))?;
        state.health = 20;
        state.waiting_for_respawn = false;
        if request.hardcore {
            state.previous_mode = Some(state.mode);
            state.mode = GameMode::Spectator;
        }
        let mut effects = respawn_effects(keep_all, request.keep_inventory);
        if keep_all {
            effects.insert(0, LifecycleEffect::WonGameCleared);
        }
        if request.hardcore {
            effects.push(LifecycleEffect::SpectatorModeForced);
        }
        Ok(RespawnOutcome::Replaced { keep_all, effects })
    }

    pub fn teleport(
        &mut self,
        player: StableEntityId,
        cross_dimension: bool,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        let state = self.player_mut(player)?;
        let mut effects = Vec::new();
        if state.sleeping {
            state.sleeping = false;
            effects.push(LifecycleEffect::SleepStopped);
        }
        if cross_dimension {
            effects.extend([
                LifecycleEffect::VehicleRemoved,
                LifecycleEffect::DimensionChangeMarked,
                LifecycleEffect::RespawnProjection(RespawnProjection::Respawn { keep_mask: 1 }),
                LifecycleEffect::RespawnProjection(RespawnProjection::Difficulty),
                LifecycleEffect::RespawnProjection(RespawnProjection::PermissionAndCommands),
                LifecycleEffect::OldLevelMembershipRemoved,
                LifecycleEffect::RespawnProjection(RespawnProjection::Teleport),
                LifecycleEffect::LevelMembershipAdded,
                LifecycleEffect::ItemUseStopped,
                LifecycleEffect::AbilitiesProjected,
                LifecycleEffect::RespawnProjection(RespawnProjection::LevelInfo),
                LifecycleEffect::RespawnProjection(RespawnProjection::Effects),
                LifecycleEffect::SentMirrorsInvalidated,
            ]);
        } else {
            effects.push(LifecycleEffect::RespawnProjection(
                RespawnProjection::Teleport,
            ));
        }
        Ok(effects)
    }

    pub fn set_game_mode(
        &mut self,
        player: StableEntityId,
        mode: GameMode,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        let state = self.player_mut(player)?;
        if state.mode == mode {
            return Ok(Vec::new());
        }
        let previous = state.mode;
        state.previous_mode = Some(previous);
        state.mode = mode;
        let mut effects = vec![LifecycleEffect::GameModeEvent(mode)];
        if mode == GameMode::Spectator {
            effects.extend([
                LifecycleEffect::ShoulderEntitiesRemoved,
                LifecycleEffect::RidingStopped,
                LifecycleEffect::ItemUseStopped,
                LifecycleEffect::LocationEffectsRemoved,
            ]);
        } else if previous == GameMode::Spectator {
            effects.extend([
                LifecycleEffect::CameraReset,
                LifecycleEffect::LocationEffectsRefreshed,
            ]);
        }
        effects.extend([
            LifecycleEffect::AbilitiesProjected,
            LifecycleEffect::InvisibilityRecomputed,
        ]);
        Ok(effects)
    }

    pub fn set_permission(
        &self,
        player: StableEntityId,
        level: PermissionLevel,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        self.player(player)?;
        Ok(vec![
            LifecycleEffect::PermissionEvent(level.entity_event()),
            LifecycleEffect::CommandTreeRebuilt,
        ])
    }

    pub fn disconnect(
        &mut self,
        player: StableEntityId,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        self.players
            .remove(&player)
            .ok_or(LifecycleError::UnknownPlayer(player))?;
        Ok(disconnect_effects())
    }

    #[must_use]
    pub fn state(&self, player: StableEntityId) -> Option<PlayerLifecycleState> {
        self.players.get(&player).copied()
    }

    #[must_use]
    pub fn snapshot(&self) -> LifecycleSnapshot {
        LifecycleSnapshot {
            players: self.players.values().copied().collect(),
        }
    }

    fn player(&self, player: StableEntityId) -> Result<&PlayerLifecycleState, LifecycleError> {
        self.players
            .get(&player)
            .ok_or(LifecycleError::UnknownPlayer(player))
    }

    fn player_mut(
        &mut self,
        player: StableEntityId,
    ) -> Result<&mut PlayerLifecycleState, LifecycleError> {
        self.players
            .get_mut(&player)
            .ok_or(LifecycleError::UnknownPlayer(player))
    }
}

fn join_effects(transferred: bool) -> Vec<LifecycleEffect> {
    let mut effects = vec![
        LifecycleEffect::ProfileCached,
        LifecycleEffect::PlayListenerInstalled,
        LifecycleEffect::FlushSuspended,
    ];
    effects.extend(
        [
            JoinProjection::Login,
            JoinProjection::Difficulty,
            JoinProjection::Abilities,
            JoinProjection::HeldSlot,
            JoinProjection::Recipes,
            JoinProjection::PermissionAndCommands,
            JoinProjection::Statistics,
            JoinProjection::RecipeBook,
            JoinProjection::Scoreboard,
        ]
        .map(LifecycleEffect::JoinProjection),
    );
    effects.extend([
        LifecycleEffect::StatusInvalidated,
        LifecycleEffect::JoinBroadcast,
        LifecycleEffect::JoinProjection(JoinProjection::Teleport),
    ]);
    if !transferred {
        effects.push(LifecycleEffect::JoinProjection(
            JoinProjection::NonTransferStatus,
        ));
    }
    effects.extend([
        LifecycleEffect::OldPlayerListQueued,
        LifecycleEffect::LiveListAdded,
        LifecycleEffect::UuidIndexAdded,
        LifecycleEffect::JoinProjection(JoinProjection::LevelInfo),
        LifecycleEffect::LevelMembershipAdded,
        LifecycleEffect::BossEventsJoined,
        LifecycleEffect::JoinProjection(JoinProjection::Effects),
        LifecycleEffect::JoinProjection(JoinProjection::Inventory),
        LifecycleEffect::IntegrationHookJoined,
        LifecycleEffect::FlushResumed,
    ]);
    effects
}

fn respawn_effects(keep_all: bool, keep_inventory: bool) -> Vec<LifecycleEffect> {
    let keep_mask = u8::from(keep_all);
    vec![
        LifecycleEffect::OldLiveListRemoved,
        LifecycleEffect::OldLevelMembershipRemoved,
        LifecycleEffect::ReplacementConstructed,
        LifecycleEffect::ConnectionTransferred,
        LifecycleEffect::StateRestored {
            keep_all,
            inventory_retained: keep_all || keep_inventory,
        },
        LifecycleEffect::RespawnProjection(RespawnProjection::Respawn { keep_mask }),
        LifecycleEffect::RespawnProjection(RespawnProjection::Teleport),
        LifecycleEffect::RespawnProjection(RespawnProjection::LevelSpawn),
        LifecycleEffect::RespawnProjection(RespawnProjection::Difficulty),
        LifecycleEffect::RespawnProjection(RespawnProjection::Experience),
        LifecycleEffect::RespawnProjection(RespawnProjection::Effects),
        LifecycleEffect::RespawnProjection(RespawnProjection::LevelInfo),
        LifecycleEffect::RespawnProjection(RespawnProjection::PermissionAndCommands),
        LifecycleEffect::LevelMembershipAdded,
        LifecycleEffect::LiveListAdded,
        LifecycleEffect::UuidIndexAdded,
        LifecycleEffect::RespawnProjection(RespawnProjection::Inventory),
        LifecycleEffect::RespawnProjection(RespawnProjection::Health),
    ]
}

fn disconnect_effects() -> Vec<LifecycleEffect> {
    vec![
        LifecycleEffect::ChatChainClosed,
        LifecycleEffect::StatusInvalidated,
        LifecycleEffect::LeaveBroadcast,
        LifecycleEffect::DisconnectedMarked,
        LifecycleEffect::PassengersEjected,
        LifecycleEffect::SleepStopped,
        LifecycleEffect::LeaveCriterionAwarded,
        LifecycleEffect::PlayerSaved,
        LifecycleEffect::StatisticsSaved,
        LifecycleEffect::AdvancementsSaved,
        LifecycleEffect::RootVehicleRemoved,
        LifecycleEffect::RidingStopped,
        LifecycleEffect::OwnedPearlsRemoved,
        LifecycleEffect::OldLevelMembershipRemoved,
        LifecycleEffect::AdvancementTriggersRemoved,
        LifecycleEffect::OldLiveListRemoved,
        LifecycleEffect::BossEventsDisconnected,
        LifecycleEffect::UuidIndexRemoved,
        LifecycleEffect::PlayerInfoRemoved,
        LifecycleEffect::TextFilterLeft,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LifecycleError {
    #[error("player lifecycle capacity cannot be zero")]
    ZeroCapacity,
    #[error("player lifecycle reached its {capacity}-player capacity")]
    Capacity { capacity: usize },
    #[error("player {0} is already live")]
    DuplicatePlayer(StableEntityId),
    #[error("player {0} is not live")]
    UnknownPlayer(StableEntityId),
    #[error("player {0} exhausted its lifecycle incarnation")]
    IncarnationExhausted(StableEntityId),
}
