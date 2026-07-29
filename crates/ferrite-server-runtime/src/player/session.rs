use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use ferrite_gameplay::player::collision::CollisionWorld;
use ferrite_gameplay::player::movement::{
    MovementContext, MovementOutcome, PlayerMove, validate_movement,
};
use ferrite_gameplay::player::state::{PlayerPose, PlayerSessionState, Rotation, Vec3};
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    MovePlayerPosition, MovePlayerRotation, PlayServerboundEntryPacket, PlayerPosition,
    PlayerRotation,
};
use ferrite_protocol::semantic::PlayAdmission;
use ferrite_region_runtime::local::LocalTickReport;
use ferrite_region_runtime::transfer::{
    EntityTransfer, EntityTransferError, EntityTransferHeader, TransferRole,
};
use ferrite_simulation::tick::GameTick;
use std::collections::VecDeque;
use thiserror::Error;

use crate::player::command::{PlayerCommandError, state_update_command};
use crate::player::router::{PlayerRegionRouteError, PlayerRegionRouter};

#[derive(Debug, Clone)]
pub struct PlayerSession {
    admission: PlayAdmission,
    state: PlayerSessionState,
    committed_state: PlayerSessionState,
    region: SimulationRegionKey,
    next_sequence: u64,
    pending_state_updates: VecDeque<PendingStateUpdate>,
    pending_transfer: Option<PendingTransfer>,
}

const MAX_PENDING_STATE_UPDATES: usize = 64;

#[derive(Debug, Clone, PartialEq)]
struct PendingStateUpdate {
    tick: GameTick,
    region: SimulationRegionKey,
    sequence: u64,
    state: PlayerSessionState,
    recenter: Option<ChunkPos>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingTransfer {
    tick: GameTick,
    source: SimulationRegionKey,
    target: SimulationRegionKey,
    source_generation: ActivationGeneration,
    target_generation: ActivationGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerSessionAction {
    None,
    PlayerLoaded,
    ClientTickEnded,
    ChunkBatchFeedback(f32),
    Movement(MovementOutcome),
    RegionTransferStaged,
    RegionTransferCommitted,
    StateCommitted { recenter: Option<ChunkPos> },
    AwaitingRegionTransfer,
}

impl PlayerSession {
    #[must_use]
    pub fn new(admission: PlayAdmission) -> Self {
        let spawn = admission.spawn;
        let pose = PlayerPose::new(
            Vec3::new(spawn.x, spawn.y, spawn.z),
            Rotation {
                yaw: spawn.yaw,
                pitch: spawn.pitch,
            },
        );
        let state = PlayerSessionState::new(pose);
        Self {
            region: admission.region.clone(),
            admission,
            committed_state: state.clone(),
            state,
            next_sequence: 1,
            pending_state_updates: VecDeque::new(),
            pending_transfer: None,
        }
    }

    #[must_use]
    pub const fn state(&self) -> &PlayerSessionState {
        &self.state
    }

    #[must_use]
    pub const fn committed_state(&self) -> &PlayerSessionState {
        &self.committed_state
    }

    #[must_use]
    pub const fn region(&self) -> &SimulationRegionKey {
        &self.region
    }

    #[must_use]
    pub const fn transfer_pending(&self) -> bool {
        self.pending_transfer.is_some()
    }

    pub fn begin_server_tick(&mut self) {
        self.state.begin_server_tick();
    }

    pub fn finish_server_tick(
        &mut self,
        gravity: f64,
        floating_exempt: bool,
    ) -> Option<PlayerSessionAction> {
        self.state
            .finish_server_tick(gravity, floating_exempt)
            .map(PlayerSessionAction::Movement)
    }

    pub fn handle_packet(
        &mut self,
        packet: PlayServerboundEntryPacket,
        teleport_pending: bool,
        mut context: MovementContext,
        collision: &impl CollisionWorld,
        target_tick: GameTick,
        router: &mut impl PlayerRegionRouter,
    ) -> Result<PlayerSessionAction, PlayerSessionError> {
        match packet {
            PlayServerboundEntryPacket::ChunkBatchReceived(feedback) => Ok(
                PlayerSessionAction::ChunkBatchFeedback(feedback.desired_chunks_per_tick),
            ),
            PlayServerboundEntryPacket::ClientTickEnd => {
                let previous = self.state.clone();
                self.state.finish_client_tick();
                self.route_state_mutation(previous, target_tick, router)?;
                Ok(PlayerSessionAction::ClientTickEnded)
            }
            PlayServerboundEntryPacket::PlayerLoaded => {
                let previous = self.state.clone();
                self.state.accept_player_loaded();
                self.route_state_mutation(previous, target_tick, router)?;
                Ok(PlayerSessionAction::PlayerLoaded)
            }
            packet if self.pending_transfer.is_some() && is_movement(packet) => {
                Ok(PlayerSessionAction::AwaitingRegionTransfer)
            }
            packet => {
                let Some(movement) = normalize_movement(packet) else {
                    return Ok(PlayerSessionAction::None);
                };
                context.teleport_pending = teleport_pending;
                self.apply_movement(movement, context, collision, target_tick, router)
            }
        }
    }

    pub fn observe_committed_tick(&mut self, report: &LocalTickReport) -> PlayerSessionAction {
        let mut committed_recenter = None;
        let mut committed_state = None;
        let mut any_state_committed = false;
        self.pending_state_updates.retain(|pending| {
            let committed = report.committed_commands().iter().any(|command| {
                command.tick == pending.tick
                    && command.target == pending.region
                    && command.sequence == pending.sequence
                    && command.source
                        == ferrite_simulation::command::CommandSource::Player(self.admission.player)
                    && command.kind.namespace() == "ferrite"
                    && command.kind.path() == crate::player::command::PLAYER_STATE_PATH
            });
            if committed {
                any_state_committed = true;
                committed_recenter = pending.recenter.or(committed_recenter);
                committed_state = Some(pending.state.clone());
            }
            !committed
        });
        if let Some(state) = committed_state {
            self.committed_state = state;
        }
        let Some(pending) = &self.pending_transfer else {
            return if any_state_committed {
                PlayerSessionAction::StateCommitted {
                    recenter: committed_recenter,
                }
            } else {
                PlayerSessionAction::None
            };
        };
        let committed = report.committed_entity_transfers().iter().any(|transfer| {
            transfer.tick == pending.tick
                && transfer.stable_id == self.admission.player
                && transfer.source == pending.source
                && transfer.target == pending.target
                && transfer.source_generation == pending.source_generation
                && transfer.target_generation == pending.target_generation
                && transfer.role == TransferRole::Player
        });
        if !committed {
            return PlayerSessionAction::None;
        }
        self.region = pending.target.clone();
        self.committed_state = self.state.clone();
        self.pending_transfer = None;
        PlayerSessionAction::RegionTransferCommitted
    }

    fn apply_movement(
        &mut self,
        movement: PlayerMove,
        context: MovementContext,
        collision: &impl CollisionWorld,
        target_tick: GameTick,
        router: &mut impl PlayerRegionRouter,
    ) -> Result<PlayerSessionAction, PlayerSessionError> {
        let previous = self.state.clone();
        let outcome = validate_movement(&mut self.state, movement, context, collision);
        let MovementOutcome::Accepted { pose, .. } = outcome else {
            self.route_state_mutation(previous, target_tick, router)?;
            return Ok(PlayerSessionAction::Movement(outcome));
        };
        let target = self.region_for_position(pose.position);
        let route_result = if target == self.region {
            self.route_state_update(target_tick, router, Some(chunk_for_position(pose.position)))
                .map(|()| PlayerSessionAction::Movement(outcome))
        } else {
            self.stage_transfer(target, target_tick, router)
                .map(|()| PlayerSessionAction::RegionTransferStaged)
        };
        if route_result.is_err() {
            self.state = previous;
        }
        route_result
    }

    fn route_state_mutation(
        &mut self,
        previous: PlayerSessionState,
        tick: GameTick,
        router: &mut impl PlayerRegionRouter,
    ) -> Result<(), PlayerSessionError> {
        if self.state == previous {
            return Ok(());
        }
        if let Err(error) = self.route_state_update(tick, router, None) {
            self.state = previous;
            return Err(error);
        }
        Ok(())
    }

    fn route_state_update(
        &mut self,
        tick: GameTick,
        router: &mut impl PlayerRegionRouter,
        recenter: Option<ChunkPos>,
    ) -> Result<(), PlayerSessionError> {
        if self.pending_state_updates.len() == MAX_PENDING_STATE_UPDATES {
            return Err(PlayerSessionError::PendingStateUpdatesFull {
                capacity: MAX_PENDING_STATE_UPDATES,
            });
        }
        let sequence = self.current_sequence()?;
        let command = state_update_command(
            self.region.clone(),
            tick,
            sequence,
            self.admission.player,
            &self.state,
        )?;
        router.route_player_command(command)?;
        self.advance_sequence();
        self.pending_state_updates.push_back(PendingStateUpdate {
            tick,
            region: self.region.clone(),
            sequence,
            state: self.state.clone(),
            recenter,
        });
        Ok(())
    }

    fn stage_transfer(
        &mut self,
        target: SimulationRegionKey,
        tick: GameTick,
        router: &mut impl PlayerRegionRouter,
    ) -> Result<(), PlayerSessionError> {
        let source_generation = router.activation_generation(&self.region)?;
        let target_generation = router.activation_generation(&target)?;
        let source_sequence = self.current_sequence()?;
        let transfer = EntityTransfer::new(
            EntityTransferHeader {
                tick,
                source: self.region.clone(),
                target: target.clone(),
                source_generation,
                target_generation,
                source_sequence,
                stable_id: self.admission.player,
                role: TransferRole::Player,
            },
            ResourceId::minecraft("player")?,
            self.state.encode_transfer(),
        )?;
        router.route_player_transfer(transfer)?;
        self.advance_sequence();
        self.pending_transfer = Some(PendingTransfer {
            tick,
            source: self.region.clone(),
            target,
            source_generation,
            target_generation,
        });
        Ok(())
    }

    fn region_for_position(&self, position: Vec3) -> SimulationRegionKey {
        self.admission.region_mapping.region_for_chunk(
            self.region.world(),
            self.region.dimension().clone(),
            chunk_for_position(position),
        )
    }

    fn current_sequence(&self) -> Result<u64, PlayerSessionError> {
        self.next_sequence
            .checked_add(1)
            .map(|_| self.next_sequence)
            .ok_or(PlayerSessionError::SequenceExhausted)
    }

    fn advance_sequence(&mut self) {
        self.next_sequence += 1;
    }
}

fn chunk_for_position(position: Vec3) -> ChunkPos {
    ChunkPos::new(
        (position.x / 16.0).floor() as i32,
        (position.z / 16.0).floor() as i32,
    )
}

fn is_movement(packet: PlayServerboundEntryPacket) -> bool {
    matches!(
        packet,
        PlayServerboundEntryPacket::MovePlayerPosition(_)
            | PlayServerboundEntryPacket::MovePlayerPositionRotation(_)
            | PlayServerboundEntryPacket::MovePlayerRotation(_)
            | PlayServerboundEntryPacket::MovePlayerStatusOnly(_)
    )
}

fn normalize_movement(packet: PlayServerboundEntryPacket) -> Option<PlayerMove> {
    match packet {
        PlayServerboundEntryPacket::MovePlayerPosition(packet) => Some(position_movement(packet)),
        PlayServerboundEntryPacket::MovePlayerPositionRotation(packet) => Some(PlayerMove {
            position: Some(position(packet.position)),
            rotation: Some(rotation(packet.rotation)),
            on_ground: packet.flags.on_ground,
            horizontal_collision: packet.flags.horizontal_collision,
        }),
        PlayServerboundEntryPacket::MovePlayerRotation(packet) => Some(rotation_movement(packet)),
        PlayServerboundEntryPacket::MovePlayerStatusOnly(packet) => Some(PlayerMove {
            position: None,
            rotation: None,
            on_ground: packet.flags.on_ground,
            horizontal_collision: packet.flags.horizontal_collision,
        }),
        _ => None,
    }
}

fn position_movement(packet: MovePlayerPosition) -> PlayerMove {
    PlayerMove {
        position: Some(position(packet.position)),
        rotation: None,
        on_ground: packet.flags.on_ground,
        horizontal_collision: packet.flags.horizontal_collision,
    }
}

fn rotation_movement(packet: MovePlayerRotation) -> PlayerMove {
    PlayerMove {
        position: None,
        rotation: Some(rotation(packet.rotation)),
        on_ground: packet.flags.on_ground,
        horizontal_collision: packet.flags.horizontal_collision,
    }
}

const fn position(position: PlayerPosition) -> Vec3 {
    Vec3::new(position.x, position.y, position.z)
}

const fn rotation(rotation: PlayerRotation) -> Rotation {
    Rotation {
        yaw: rotation.yaw,
        pitch: rotation.pitch,
    }
}

#[derive(Debug, Error)]
pub enum PlayerSessionError {
    #[error(transparent)]
    Command(#[from] PlayerCommandError),
    #[error(transparent)]
    Route(#[from] PlayerRegionRouteError),
    #[error(transparent)]
    Transfer(#[from] EntityTransferError),
    #[error(transparent)]
    Resource(#[from] ResourceIdError),
    #[error("player command sequence space is exhausted")]
    SequenceExhausted,
    #[error("player session reached its {capacity}-update pending bound")]
    PendingStateUpdatesFull { capacity: usize },
}
