use ferrite_gameplay::player::collision::CollisionWorld;
use ferrite_gameplay::player::movement::{MovementContext, MovementOutcome};
use ferrite_protocol::java_26_2::connection::driver::ServerConnection;
use ferrite_protocol::java_26_2::connection::error::ServerConnectionError;
use ferrite_protocol::java_26_2::connection::output::{
    PlayDisconnectReason, ServerConnectionEvent,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_protocol::semantic::PlayAdmission;
use ferrite_region_runtime::local::LocalTickReport;
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::chunk::projection::JavaTerrainRegistryMap;
use crate::chunk::session::{ChunkSessionLimits, ClientChunkSession, ClientChunkSessionError};
use crate::chunk::stream::ChunkStreamEvent;
use crate::player::block::replication::BlockCommandResult;
use crate::player::block::session::{
    BlockInteractionAction, BlockInteractionSession, BlockPacketContext, BlockSessionError,
};
use crate::player::router::PlayerRegionRouter;
use crate::player::session::{PlayerSession, PlayerSessionAction, PlayerSessionError};

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

    pub fn handle_java_event(
        &mut self,
        event: ServerConnectionEvent,
        connection: &mut ServerConnection,
        movement_context: MovementContext,
        collision: &impl CollisionWorld,
        target_tick: GameTick,
        router: &mut impl PlayerRegionRouter,
    ) -> Result<Option<PlayerConnectionUpdate>, PlayerConnectionError> {
        let ServerConnectionEvent::PlayPacket {
            packet,
            teleport_pending,
        } = event
        else {
            return Ok(None);
        };
        if is_block_interaction(&packet) {
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
            return Ok(Some(PlayerConnectionUpdate {
                player: PlayerSessionAction::None,
                block: action,
                block_results: Vec::new(),
                block_packets: Vec::new(),
                chunk_events: Vec::new(),
            }));
        }
        let action = self.player.handle_packet(
            packet,
            teleport_pending,
            movement_context,
            collision,
            target_tick,
            router,
        )?;
        self.project_player_action(connection, action).map(Some)
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
    #[error(transparent)]
    Connection(#[from] ServerConnectionError),
    #[error(transparent)]
    Player(#[from] PlayerSessionError),
    #[error(transparent)]
    Chunk(#[from] ClientChunkSessionError),
    #[error(transparent)]
    Block(#[from] BlockSessionError),
}

fn is_block_interaction(packet: &PlayServerboundEntryPacket) -> bool {
    matches!(
        packet,
        PlayServerboundEntryPacket::PickItemFromBlock(_)
            | PlayServerboundEntryPacket::PlayerAction(_)
            | PlayServerboundEntryPacket::Swing(_)
            | PlayServerboundEntryPacket::UseItemOn(_)
            | PlayServerboundEntryPacket::UseItem(_)
    )
}
