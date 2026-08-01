use ferrite_gameplay::player::collision::CollisionWorld;
use ferrite_gameplay::player::movement::{MovementContext, MovementOutcome};
use ferrite_protocol::java_26_2::connection::driver::ServerConnection;
use ferrite_protocol::java_26_2::connection::error::ServerConnectionError;
use ferrite_protocol::java_26_2::connection::output::PlayDisconnectReason;
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_protocol::semantic::PlayAdmission;
use ferrite_region_runtime::local::LocalTickReport;
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::chunk::projection::{
    JavaTerrainRegistryMap, TerrainProjectionError, project_stream_events,
};
use crate::chunk::session::{ChunkSessionLimits, ClientChunkSession, ClientChunkSessionError};
use crate::chunk::stream::ChunkStreamEvent;
use crate::player::block::replication::BlockCommandResult;
use crate::player::block::session::{
    BlockInteractionAction, BlockInteractionSession, BlockPacketContext, BlockSessionError,
};
use crate::player::dispatch::{
    ServerboundDispatchOutcome, ServerboundDisposition, route as dispatch_route,
};
use crate::player::router::PlayerRegionRouter;
use crate::player::session::{PlayerSession, PlayerSessionAction, PlayerSessionError};
use ferrite_world::terrain::MinimalTerrain;

#[derive(Debug, Clone)]
pub struct JavaPlayerConnection {
    player: PlayerSession,
    blocks: BlockInteractionSession,
    chunks: ClientChunkSession,
    registries: PlayRegistries,
    terrain_registries: Option<JavaTerrainRegistryMap>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerConnectionUpdate {
    pub player: PlayerSessionAction,
    pub block: BlockInteractionAction,
    pub block_results: Vec<BlockCommandResult>,
    pub block_packets: Vec<PlayClientboundPacket>,
    pub chunk_events: Vec<ChunkStreamEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerDispatchReport {
    pub outcome: ServerboundDispatchOutcome,
    pub update: Option<PlayerConnectionUpdate>,
}

pub struct PlayerDispatchContext<'a, R, C> {
    pub teleport_pending: bool,
    pub connection: &'a mut ServerConnection,
    pub movement: MovementContext,
    pub collision: &'a C,
    pub target_tick: GameTick,
    pub router: &'a mut R,
}

impl JavaPlayerConnection {
    pub fn new(
        admission: PlayAdmission,
        registries: PlayRegistries,
        server_view_distance: u16,
        simulation_distance: u16,
        chunk_limits: ChunkSessionLimits,
    ) -> Result<Self, PlayerConnectionError> {
        let chunks = ClientChunkSession::join(
            &admission,
            server_view_distance,
            simulation_distance,
            chunk_limits,
        )?;
        Ok(Self {
            blocks: BlockInteractionSession::new(&admission),
            player: PlayerSession::new(admission),
            chunks,
            registries,
            terrain_registries: None,
        })
    }

    #[must_use]
    pub const fn player(&self) -> &PlayerSession {
        &self.player
    }

    #[must_use]
    pub const fn chunks(&self) -> &ClientChunkSession {
        &self.chunks
    }

    pub fn install_terrain_registry_map(&mut self, registries: JavaTerrainRegistryMap) {
        self.terrain_registries = Some(registries);
    }

    /// Installs interest metadata and marks every initial view chunk ready for
    /// deterministic projection from the authoritative terrain provider.
    pub fn enqueue_initial_terrain(
        &mut self,
        connection: &mut ServerConnection,
        terrain: &MinimalTerrain,
    ) -> Result<(), PlayerConnectionError> {
        let registries = self
            .terrain_registries
            .as_ref()
            .ok_or(PlayerConnectionError::MissingTerrainRegistries)?;
        let packets = project_stream_events(self.chunks.initial_events().to_vec(), registries)?;
        connection.enqueue_play(&packets, &self.registries)?;
        let positions = self
            .chunks
            .stream()
            .interest()
            .view()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        for position in positions {
            self.chunks.mark_ready(position)?;
        }
        self.enqueue_next_terrain_batch(connection, terrain)?;
        Ok(())
    }

    /// Projects one flow-controlled terrain batch when the client has room.
    pub fn enqueue_next_terrain_batch(
        &mut self,
        connection: &mut ServerConnection,
        terrain: &MinimalTerrain,
    ) -> Result<bool, PlayerConnectionError> {
        let Some(prepared) = self
            .chunks
            .prepare_next_batch(|position| terrain.snapshot(position).ok())?
        else {
            return Ok(false);
        };
        let registries = self
            .terrain_registries
            .as_ref()
            .ok_or(PlayerConnectionError::MissingTerrainRegistries)?;
        let packets = project_stream_events(prepared.events().to_vec(), registries)?;
        connection.enqueue_play(&packets, &self.registries)?;
        self.chunks.commit_prepared_batch(prepared)?;
        Ok(true)
    }

    /// Queues the authoritative initial correction after the rest of the entry projection.
    pub fn finish_play_installation(
        &mut self,
        connection: &mut ServerConnection,
    ) -> Result<i32, PlayerConnectionError> {
        let pose = self.player.state().pose();
        let challenge = connection.issue_player_correction(
            Vector3 {
                x: pose.position.x,
                y: pose.position.y,
                z: pose.position.z,
            },
            pose.rotation.yaw,
            pose.rotation.pitch,
            &self.registries,
        )?;
        connection.complete_play_installation()?;
        Ok(challenge)
    }

    pub fn begin_server_tick(&mut self) {
        self.player.begin_server_tick();
    }

    pub fn finish_server_tick(
        &mut self,
        connection: &mut ServerConnection,
        gravity: f64,
        floating_exempt: bool,
    ) -> Result<Option<PlayerConnectionUpdate>, PlayerConnectionError> {
        let Some(action) = self.player.finish_server_tick(gravity, floating_exempt) else {
            return Ok(None);
        };
        self.project_player_action(connection, action).map(Some)
    }

    pub fn dispatch_serverbound(
        &mut self,
        packet: PlayServerboundEntryPacket,
        context: PlayerDispatchContext<'_, impl PlayerRegionRouter, impl CollisionWorld>,
    ) -> Result<PlayerDispatchReport, PlayerConnectionError> {
        let PlayerDispatchContext {
            teleport_pending,
            connection,
            movement,
            collision,
            target_tick,
            router,
        } = context;
        let route = dispatch_route(&packet);
        if !route.is_application_supported() {
            return Ok(PlayerDispatchReport {
                outcome: route.default_outcome(),
                update: None,
            });
        }
        if route.is_block() {
            let action = self.blocks.handle_packet(
                packet,
                BlockPacketContext {
                    player: self.player.state(),
                    player_region: self.player.region(),
                    teleport_pending,
                    target_tick,
                    router,
                    connection,
                },
            )?;
            return Ok(PlayerDispatchReport {
                outcome: ServerboundDispatchOutcome::from_block(route, action),
                update: Some(PlayerConnectionUpdate {
                    player: PlayerSessionAction::None,
                    block: action,
                    block_results: Vec::new(),
                    block_packets: Vec::new(),
                    chunk_events: Vec::new(),
                }),
            });
        }
        let action = self.player.handle_packet(
            packet,
            teleport_pending,
            movement,
            collision,
            target_tick,
            router,
        )?;
        let outcome = ServerboundDispatchOutcome::from_player(route, action);
        let update = self.project_player_action(connection, action)?;
        debug_assert!(!matches!(
            outcome.disposition(),
            ServerboundDisposition::Unsupported
        ));
        Ok(PlayerDispatchReport {
            outcome,
            update: Some(update),
        })
    }

    pub fn observe_committed_tick(
        &mut self,
        report: &LocalTickReport,
    ) -> Result<PlayerConnectionUpdate, PlayerConnectionError> {
        let action = self.player.observe_committed_tick(report);
        let block = self
            .blocks
            .observe_committed_tick(report, self.terrain_registries.as_ref())?;
        let center = match action {
            PlayerSessionAction::RegionTransferCommitted => {
                let position = self.player.committed_state().pose().position;
                Some(ferrite_foundation::coordinate::ChunkPos::new(
                    (position.x / 16.0).floor() as i32,
                    (position.z / 16.0).floor() as i32,
                ))
            }
            PlayerSessionAction::StateCommitted { recenter } => recenter,
            _ => None,
        };
        let chunk_events = center
            .map(|center| self.chunks.recenter(center, true))
            .transpose()?
            .unwrap_or_default();
        Ok(PlayerConnectionUpdate {
            player: action,
            block: BlockInteractionAction::None,
            block_results: block.results,
            block_packets: block.packets,
            chunk_events,
        })
    }

    pub fn observe_committed_tick_and_project(
        &mut self,
        report: &LocalTickReport,
        connection: &mut ServerConnection,
    ) -> Result<PlayerConnectionUpdate, PlayerConnectionError> {
        let update = self.observe_committed_tick(report)?;
        connection.enqueue_play(&update.block_packets, &self.registries)?;
        if !update.chunk_events.is_empty() {
            let registries = self
                .terrain_registries
                .as_ref()
                .ok_or(PlayerConnectionError::MissingTerrainRegistries)?;
            let packets = project_stream_events(update.chunk_events.clone(), registries)?;
            connection.enqueue_play(&packets, &self.registries)?;
        }
        Ok(update)
    }

    fn project_player_action(
        &mut self,
        connection: &mut ServerConnection,
        action: PlayerSessionAction,
    ) -> Result<PlayerConnectionUpdate, PlayerConnectionError> {
        let chunk_events = Vec::new();
        match action {
            PlayerSessionAction::ChunkBatchFeedback(desired) => {
                self.chunks.acknowledge_batch(desired)?;
            }
            PlayerSessionAction::Movement(MovementOutcome::Correct { authoritative_pose }) => {
                connection.issue_player_correction(
                    Vector3 {
                        x: authoritative_pose.position.x,
                        y: authoritative_pose.position.y,
                        z: authoritative_pose.position.z,
                    },
                    authoritative_pose.rotation.yaw,
                    authoritative_pose.rotation.pitch,
                    &self.registries,
                )?;
            }
            PlayerSessionAction::Movement(MovementOutcome::DisconnectInvalidMovement) => {
                connection.disconnect_play(
                    PlayDisconnectReason::InvalidPlayerMovement,
                    &self.registries,
                )?;
            }
            PlayerSessionAction::Movement(MovementOutcome::DisconnectFlying) => {
                connection.disconnect_play(PlayDisconnectReason::Flying, &self.registries)?;
            }
            PlayerSessionAction::Movement(MovementOutcome::Accepted { .. }) => {}
            _ => {}
        }
        Ok(PlayerConnectionUpdate {
            player: action,
            block: BlockInteractionAction::None,
            block_results: Vec::new(),
            block_packets: Vec::new(),
            chunk_events,
        })
    }
}

#[derive(Debug, Error)]
pub enum PlayerConnectionError {
    #[error("player connection has no installed terrain registry map")]
    MissingTerrainRegistries,
    #[error(transparent)]
    Connection(#[from] ServerConnectionError),
    #[error(transparent)]
    Player(#[from] PlayerSessionError),
    #[error(transparent)]
    Chunk(#[from] ClientChunkSessionError),
    #[error(transparent)]
    Block(#[from] BlockSessionError),
    #[error(transparent)]
    TerrainProjection(#[from] TerrainProjectionError),
}
