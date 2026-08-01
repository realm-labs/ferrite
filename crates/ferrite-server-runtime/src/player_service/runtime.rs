use std::collections::{BTreeMap, VecDeque};

use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_gameplay::item::runtime::menu_sync::MAX_STATE_ID;
use ferrite_persistence::snapshot::SnapshotRecord;
use thiserror::Error;

use crate::player_service::continuity::{
    ContinuityError, decode_player, encode_player, validate_state,
};
use crate::player_service::model::{
    ActionOutcome, MenuLease, PlayerActionHeader, PlayerMutation, PlayerPersistentState,
    PlayerProjection, ProjectionKind, ResyncReason,
};

#[derive(Debug, Clone)]
struct OwnedPlayer {
    persistent: PlayerPersistentState,
    session_epoch: u64,
    menu: Option<MenuLease>,
    projections: VecDeque<PlayerProjection>,
}

#[derive(Debug, Clone)]
pub struct PlayerServiceRegionRuntime {
    key: SimulationRegionKey,
    generation: ActivationGeneration,
    player_capacity: usize,
    projection_capacity_per_player: usize,
    next_projection_revision: u64,
    players: BTreeMap<StableEntityId, OwnedPlayer>,
}

impl PlayerServiceRegionRuntime {
    pub fn new(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        player_capacity: usize,
        projection_capacity_per_player: usize,
    ) -> Result<Self, PlayerServiceRuntimeError> {
        validate_capacities(player_capacity, projection_capacity_per_player)?;
        Ok(Self {
            key,
            generation,
            player_capacity,
            projection_capacity_per_player,
            next_projection_revision: 1,
            players: BTreeMap::new(),
        })
    }

    pub fn restore(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        player_capacity: usize,
        projection_capacity_per_player: usize,
        records: &[SnapshotRecord],
    ) -> Result<Self, PlayerServiceRuntimeError> {
        let mut runtime = Self::new(
            key,
            generation,
            player_capacity,
            projection_capacity_per_player,
        )?;
        for record in records {
            let Some((player, mut persistent)) = decode_player(record)? else {
                continue;
            };
            if runtime.players.len() == runtime.player_capacity {
                return Err(PlayerServiceRuntimeError::PlayerCapacity {
                    capacity: runtime.player_capacity,
                });
            }
            if runtime.players.contains_key(&player) {
                return Err(PlayerServiceRuntimeError::DuplicatePlayer(player));
            }
            let session_epoch = persistent
                .last_session_epoch
                .checked_add(1)
                .ok_or(PlayerServiceRuntimeError::SessionEpochExhausted(player))?;
            persistent.last_session_epoch = session_epoch;
            let inventory_revision = persistent.inventory_revision;
            let revision = runtime.allocate_projection_revision()?;
            runtime.players.insert(
                player,
                OwnedPlayer {
                    persistent,
                    session_epoch,
                    menu: None,
                    projections: VecDeque::from([PlayerProjection {
                        revision,
                        player,
                        session_epoch,
                        kind: ProjectionKind::FullState {
                            reason: ResyncReason::Reload,
                            inventory_revision,
                            menu: None,
                        },
                    }]),
                },
            );
        }
        Ok(runtime)
    }

    pub fn join(
        &mut self,
        player: StableEntityId,
        mut persistent: PlayerPersistentState,
    ) -> Result<u64, PlayerServiceRuntimeError> {
        validate_state(&persistent)?;
        if self.players.contains_key(&player) {
            return Err(PlayerServiceRuntimeError::DuplicatePlayer(player));
        }
        if self.players.len() == self.player_capacity {
            return Err(PlayerServiceRuntimeError::PlayerCapacity {
                capacity: self.player_capacity,
            });
        }
        let session_epoch = persistent
            .last_session_epoch
            .checked_add(1)
            .ok_or(PlayerServiceRuntimeError::SessionEpochExhausted(player))?;
        persistent.last_session_epoch = session_epoch;
        let inventory_revision = persistent.inventory_revision;
        let revision = self.allocate_projection_revision()?;
        self.players.insert(
            player,
            OwnedPlayer {
                persistent,
                session_epoch,
                menu: None,
                projections: VecDeque::from([PlayerProjection {
                    revision,
                    player,
                    session_epoch,
                    kind: ProjectionKind::FullState {
                        reason: ResyncReason::Join,
                        inventory_revision,
                        menu: None,
                    },
                }]),
            },
        );
        Ok(session_epoch)
    }

    pub fn open_menu(
        &mut self,
        header: &PlayerActionHeader,
        container_id: u8,
    ) -> Result<(), PlayerServiceRuntimeError> {
        self.validate_header(header)?;
        let player = self
            .players
            .get_mut(&header.player)
            .expect("validated player remains present");
        player.menu = Some(MenuLease {
            container_id,
            state_id: 0,
        });
        Ok(())
    }

    pub fn close_menu(
        &mut self,
        header: &PlayerActionHeader,
    ) -> Result<(), PlayerServiceRuntimeError> {
        self.validate_header(header)?;
        self.players
            .get_mut(&header.player)
            .expect("validated player remains present")
            .menu = None;
        Ok(())
    }

    pub fn apply_player_action(
        &mut self,
        header: &PlayerActionHeader,
        mutation: PlayerMutation,
    ) -> Result<ActionOutcome, PlayerServiceRuntimeError> {
        self.validate_header(header)?;
        let admission = self.sequence_admission(header)?;
        if let Some(outcome) = admission {
            return Ok(outcome);
        }
        let persistent = self
            .players
            .get(&header.player)
            .expect("validated player remains present")
            .persistent
            .clone();
        if mutation.expected_inventory_revision != persistent.inventory_revision {
            return self.reject_with_full_resync(
                header,
                ResyncReason::InventoryRevision,
                persistent.inventory_revision,
            );
        }
        let next_inventory_revision = persistent.inventory_revision.checked_add(1).ok_or(
            PlayerServiceRuntimeError::InventoryRevisionExhausted(header.player),
        )?;
        let candidate = candidate_state(
            persistent,
            mutation,
            next_inventory_revision,
            header.sequence,
        )?;
        let revision = self.preflight_projection(header.player)?;
        let player = self
            .players
            .get_mut(&header.player)
            .expect("validated player remains present");
        player.persistent = candidate;
        player.projections.push_back(PlayerProjection {
            revision,
            player: header.player,
            session_epoch: header.session_epoch,
            kind: ProjectionKind::InventoryDelta {
                inventory_revision: next_inventory_revision,
            },
        });
        Ok(ActionOutcome::Committed {
            projection_revision: revision,
            full_resync: false,
        })
    }

    pub fn apply_menu_action(
        &mut self,
        header: &PlayerActionHeader,
        container_id: u8,
        state_id: u16,
        mutation: PlayerMutation,
    ) -> Result<ActionOutcome, PlayerServiceRuntimeError> {
        self.validate_header(header)?;
        if let Some(outcome) = self.sequence_admission(header)? {
            return Ok(outcome);
        }
        let (menu, inventory_revision) = {
            let player = self
                .players
                .get(&header.player)
                .expect("validated player remains present");
            (player.menu, player.persistent.inventory_revision)
        };
        let Some(menu) = menu else {
            self.record_ignored_sequence(header);
            return Ok(ActionOutcome::IgnoredWrongContainer);
        };
        if menu.container_id != container_id {
            self.record_ignored_sequence(header);
            return Ok(ActionOutcome::IgnoredWrongContainer);
        }
        if mutation.expected_inventory_revision != inventory_revision {
            return self.reject_with_full_resync(
                header,
                ResyncReason::InventoryRevision,
                inventory_revision,
            );
        }

        let full_resync = menu.state_id != state_id;
        let next_inventory_revision = inventory_revision.checked_add(1).ok_or(
            PlayerServiceRuntimeError::InventoryRevisionExhausted(header.player),
        )?;
        let persistent = self
            .players
            .get(&header.player)
            .expect("validated player remains present")
            .persistent
            .clone();
        let candidate = candidate_state(
            persistent,
            mutation,
            next_inventory_revision,
            header.sequence,
        )?;
        let revision = self.preflight_projection(header.player)?;
        let player = self
            .players
            .get_mut(&header.player)
            .expect("validated player remains present");
        let menu = player
            .menu
            .as_mut()
            .expect("validated menu remains present");
        menu.state_id = menu.state_id.wrapping_add(1) & MAX_STATE_ID;
        player.persistent = candidate;
        let kind = if full_resync {
            ProjectionKind::FullState {
                reason: ResyncReason::MenuState,
                inventory_revision: next_inventory_revision,
                menu: Some(*menu),
            }
        } else {
            ProjectionKind::MenuDelta {
                container_id,
                state_id: menu.state_id,
                inventory_revision: next_inventory_revision,
            }
        };
        player.projections.push_back(PlayerProjection {
            revision,
            player: header.player,
            session_epoch: header.session_epoch,
            kind,
        });
        Ok(ActionOutcome::Committed {
            projection_revision: revision,
            full_resync,
        })
    }

    pub fn capture_continuity(&self) -> Result<Vec<SnapshotRecord>, PlayerServiceRuntimeError> {
        self.players
            .iter()
            .map(|(&player, owned)| encode_player(player, &owned.persistent).map_err(Into::into))
            .collect()
    }

    pub fn drain_projections(
        &mut self,
        player: StableEntityId,
        maximum: usize,
    ) -> Result<Vec<PlayerProjection>, PlayerServiceRuntimeError> {
        let queue = &mut self
            .players
            .get_mut(&player)
            .ok_or(PlayerServiceRuntimeError::UnknownPlayer(player))?
            .projections;
        let count = maximum.min(queue.len());
        Ok(queue.drain(..count).collect())
    }

    #[must_use]
    pub fn state(&self, player: StableEntityId) -> Option<PlayerPersistentState> {
        self.players
            .get(&player)
            .map(|owned| owned.persistent.clone())
    }

    #[must_use]
    pub fn session_epoch(&self, player: StableEntityId) -> Option<u64> {
        self.players.get(&player).map(|owned| owned.session_epoch)
    }

    #[must_use]
    pub fn menu(&self, player: StableEntityId) -> Option<MenuLease> {
        self.players.get(&player).and_then(|owned| owned.menu)
    }

    #[must_use]
    pub fn projection_len(&self, player: StableEntityId) -> Option<usize> {
        self.players
            .get(&player)
            .map(|owned| owned.projections.len())
    }

    fn validate_header(
        &self,
        header: &PlayerActionHeader,
    ) -> Result<(), PlayerServiceRuntimeError> {
        if header.region != self.key {
            return Err(PlayerServiceRuntimeError::WrongRegion);
        }
        if header.generation != self.generation {
            return Err(PlayerServiceRuntimeError::StaleGeneration {
                expected: self.generation,
                actual: header.generation,
            });
        }
        let player = self
            .players
            .get(&header.player)
            .ok_or(PlayerServiceRuntimeError::UnknownPlayer(header.player))?;
        if header.session_epoch != player.session_epoch {
            return Err(PlayerServiceRuntimeError::StaleSession {
                expected: player.session_epoch,
                actual: header.session_epoch,
            });
        }
        Ok(())
    }

    fn sequence_admission(
        &self,
        header: &PlayerActionHeader,
    ) -> Result<Option<ActionOutcome>, PlayerServiceRuntimeError> {
        let last = self
            .players
            .get(&header.player)
            .expect("validated player remains present")
            .persistent
            .last_action_sequence;
        if header.sequence <= last {
            return Ok(Some(ActionOutcome::AlreadyApplied));
        }
        let expected =
            last.checked_add(1)
                .ok_or(PlayerServiceRuntimeError::ActionSequenceExhausted(
                    header.player,
                ))?;
        if header.sequence != expected {
            return Err(PlayerServiceRuntimeError::ActionSequenceGap {
                expected,
                actual: header.sequence,
            });
        }
        Ok(None)
    }

    fn reject_with_full_resync(
        &mut self,
        header: &PlayerActionHeader,
        reason: ResyncReason,
        inventory_revision: u64,
    ) -> Result<ActionOutcome, PlayerServiceRuntimeError> {
        let revision = self.preflight_projection(header.player)?;
        let player = self
            .players
            .get_mut(&header.player)
            .expect("validated player remains present");
        player.persistent.last_action_sequence = header.sequence;
        player.projections.push_back(PlayerProjection {
            revision,
            player: header.player,
            session_epoch: header.session_epoch,
            kind: ProjectionKind::FullState {
                reason,
                inventory_revision,
                menu: player.menu,
            },
        });
        Ok(ActionOutcome::RejectedAndResynchronized {
            reason,
            projection_revision: revision,
        })
    }

    fn record_ignored_sequence(&mut self, header: &PlayerActionHeader) {
        self.players
            .get_mut(&header.player)
            .expect("validated player remains present")
            .persistent
            .last_action_sequence = header.sequence;
    }

    fn preflight_projection(
        &mut self,
        player: StableEntityId,
    ) -> Result<u64, PlayerServiceRuntimeError> {
        let queue = &self
            .players
            .get(&player)
            .expect("validated player remains present")
            .projections;
        if queue.len() == self.projection_capacity_per_player {
            return Err(PlayerServiceRuntimeError::ProjectionCapacity {
                player,
                capacity: self.projection_capacity_per_player,
            });
        }
        self.allocate_projection_revision()
    }

    fn allocate_projection_revision(&mut self) -> Result<u64, PlayerServiceRuntimeError> {
        let revision = self.next_projection_revision;
        self.next_projection_revision = revision
            .checked_add(1)
            .ok_or(PlayerServiceRuntimeError::ProjectionRevisionExhausted)?;
        Ok(revision)
    }
}

fn candidate_state(
    state: PlayerPersistentState,
    mutation: PlayerMutation,
    inventory_revision: u64,
    action_sequence: u64,
) -> Result<PlayerPersistentState, PlayerServiceRuntimeError> {
    let candidate = PlayerPersistentState {
        inventory_revision,
        inventory: mutation.inventory,
        selected_slot: mutation.selected_slot,
        experience_points: mutation.experience_points,
        experience_level: mutation.experience_level,
        food_level: mutation.food_level,
        saturation_bits: mutation.saturation_bits,
        exhaustion_bits: mutation.exhaustion_bits,
        progression: mutation.progression,
        last_action_sequence: action_sequence,
        last_session_epoch: state.last_session_epoch,
    };
    validate_state(&candidate)?;
    Ok(candidate)
}

fn validate_capacities(
    player_capacity: usize,
    projection_capacity_per_player: usize,
) -> Result<(), PlayerServiceRuntimeError> {
    if player_capacity == 0 {
        return Err(PlayerServiceRuntimeError::ZeroPlayerCapacity);
    }
    if projection_capacity_per_player == 0 {
        return Err(PlayerServiceRuntimeError::ZeroProjectionCapacity);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum PlayerServiceRuntimeError {
    #[error("player-service capacity cannot be zero")]
    ZeroPlayerCapacity,
    #[error("player-service per-player projection capacity cannot be zero")]
    ZeroProjectionCapacity,
    #[error("player-service Region is at its {capacity}-player capacity")]
    PlayerCapacity { capacity: usize },
    #[error("player {0} is duplicated in the player-service Region")]
    DuplicatePlayer(StableEntityId),
    #[error("player {0} is not owned by the player-service Region")]
    UnknownPlayer(StableEntityId),
    #[error("player-service action targets a different Region")]
    WrongRegion,
    #[error("player-service generation {actual:?} is stale; expected {expected:?}")]
    StaleGeneration {
        expected: ActivationGeneration,
        actual: ActivationGeneration,
    },
    #[error("player-service session epoch {actual} is stale; expected {expected}")]
    StaleSession { expected: u64, actual: u64 },
    #[error("player-service action sequence gap: expected {expected}, got {actual}")]
    ActionSequenceGap { expected: u64, actual: u64 },
    #[error("player {0} exhausted action sequences")]
    ActionSequenceExhausted(StableEntityId),
    #[error("player {0} exhausted inventory revisions")]
    InventoryRevisionExhausted(StableEntityId),
    #[error("player {0} exhausted session epochs")]
    SessionEpochExhausted(StableEntityId),
    #[error("player-service projection revisions are exhausted")]
    ProjectionRevisionExhausted,
    #[error("player {player} reached its {capacity}-projection capacity")]
    ProjectionCapacity {
        player: StableEntityId,
        capacity: usize,
    },
    #[error(transparent)]
    Continuity(#[from] ContinuityError),
}
