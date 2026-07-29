use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_gameplay::block::targeting::{DEFAULT_BLOCK_INTERACTION_RANGE, eye_position};
use ferrite_gameplay::player::state::PlayerSessionState;
use ferrite_protocol::java_26_2::connection::driver::ServerConnection;
use ferrite_protocol::java_26_2::connection::error::ServerConnectionError;
use ferrite_protocol::java_26_2::play::clientbound::packet::{BlockUpdate, PlayClientboundPacket};
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    PlayServerboundEntryPacket, PlayerActionKind,
};
use ferrite_protocol::semantic::PlayAdmission;
use ferrite_region_runtime::local::LocalTickReport;
use ferrite_simulation::tick::GameTick;
use ferrite_world::id::BlockStateId;
use thiserror::Error;

use crate::chunk::projection::{JavaTerrainRegistryMap, TerrainProjectionError};
use crate::player::block::command::{BlockCommandError, BlockIntent, BlockInteractionCommand};
use crate::player::block::replication::{
    BlockCommandResult, BlockReplicationError, project_committed_blocks,
};
use crate::player::router::{PlayerRegionRouteError, PlayerRegionRouter};

#[derive(Debug, Clone)]
pub struct BlockInteractionSession {
    player: StableEntityId,
    mapping: RegionMapping,
    next_command_sequence: u64,
    interaction_range: f64,
    placement_state: BlockStateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockInteractionAction {
    None,
    DroppedBeforeClientLoaded,
    Routed { command_sequence: u64 },
    PredictionRegistered,
    Unsequenced,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockSessionUpdate {
    pub results: Vec<BlockCommandResult>,
    pub packets: Vec<PlayClientboundPacket>,
}

pub(crate) struct BlockPacketContext<'a, R> {
    pub player: &'a PlayerSessionState,
    pub player_region: &'a SimulationRegionKey,
    pub teleport_pending: bool,
    pub target_tick: GameTick,
    pub router: &'a mut R,
    pub connection: &'a mut ServerConnection,
}

impl BlockInteractionSession {
    #[must_use]
    pub const fn new(admission: &PlayAdmission) -> Self {
        Self {
            player: admission.player,
            mapping: admission.region_mapping,
            next_command_sequence: 1,
            interaction_range: DEFAULT_BLOCK_INTERACTION_RANGE,
            placement_state: BlockStateId::new(1),
        }
    }

    pub const fn set_interaction_range(&mut self, interaction_range: f64) {
        self.interaction_range = interaction_range;
    }

    pub const fn set_placement_state(&mut self, state: BlockStateId) {
        self.placement_state = state;
    }

    pub(crate) fn handle_packet<R: PlayerRegionRouter>(
        &mut self,
        packet: PlayServerboundEntryPacket,
        context: BlockPacketContext<'_, R>,
    ) -> Result<BlockInteractionAction, BlockSessionError> {
        let BlockPacketContext {
            player,
            player_region,
            teleport_pending,
            target_tick,
            router,
            connection,
        } = context;
        match packet {
            PlayServerboundEntryPacket::PlayerAction(packet) if packet.action.is_destroy() => {
                if !player.client_loaded() {
                    return Ok(BlockInteractionAction::DroppedBeforeClientLoaded);
                }
                let intent = match packet.action {
                    PlayerActionKind::StartDestroyBlock => BlockIntent::StartDestroy {
                        position: packet.position,
                    },
                    PlayerActionKind::AbortDestroyBlock => BlockIntent::AbortDestroy {
                        position: packet.position,
                    },
                    PlayerActionKind::StopDestroyBlock => BlockIntent::StopDestroy {
                        position: packet.position,
                    },
                    _ => unreachable!("destroy action guard excludes auxiliary actions"),
                };
                let action = self.route(intent, player, player_region, target_tick, router)?;
                connection.register_block_sequence(packet.sequence)?;
                Ok(action)
            }
            PlayServerboundEntryPacket::UseItemOn(packet) => {
                if !player.client_loaded() {
                    return Ok(BlockInteractionAction::DroppedBeforeClientLoaded);
                }
                connection.register_block_sequence(packet.sequence)?;
                self.route(
                    BlockIntent::UseOn {
                        position: packet.hit.position,
                        direction: packet.hit.direction,
                        offset_x: packet.hit.offset_x,
                        offset_y: packet.hit.offset_y,
                        offset_z: packet.hit.offset_z,
                        inside: packet.hit.inside,
                        world_border_hit: packet.hit.world_border_hit,
                        interaction_allowed: !teleport_pending,
                        placement_state: self.placement_state,
                    },
                    player,
                    player_region,
                    target_tick,
                    router,
                )
            }
            PlayServerboundEntryPacket::UseItem(packet) => {
                if !player.client_loaded() {
                    return Ok(BlockInteractionAction::DroppedBeforeClientLoaded);
                }
                connection.register_block_sequence(packet.sequence)?;
                Ok(BlockInteractionAction::PredictionRegistered)
            }
            PlayServerboundEntryPacket::PickItemFromBlock(_)
            | PlayServerboundEntryPacket::Swing(_)
            | PlayServerboundEntryPacket::PlayerAction(_) => {
                Ok(BlockInteractionAction::Unsequenced)
            }
            _ => Ok(BlockInteractionAction::None),
        }
    }

    pub fn observe_committed_tick(
        &self,
        report: &LocalTickReport,
        registries: Option<&JavaTerrainRegistryMap>,
    ) -> Result<BlockSessionUpdate, BlockSessionError> {
        let projection = project_committed_blocks(report, self.player, registries)?;
        let mut packets = Vec::new();
        for result in &projection.results {
            for update in &result.corrections {
                let registries = registries.ok_or(BlockSessionError::MissingRegistryMap)?;
                packets.push(PlayClientboundPacket::BlockUpdate(BlockUpdate {
                    position: update.position,
                    state: registries.block_state(update.state)?,
                }));
            }
        }
        packets.extend(projection.packets);
        Ok(BlockSessionUpdate {
            results: projection.results,
            packets,
        })
    }

    fn route(
        &mut self,
        intent: BlockIntent,
        player: &PlayerSessionState,
        player_region: &SimulationRegionKey,
        target_tick: GameTick,
        router: &mut impl PlayerRegionRouter,
    ) -> Result<BlockInteractionAction, BlockSessionError> {
        let sequence = self.next_command_sequence;
        let position = intent_position(intent);
        let target = self.mapping.region_for_chunk(
            player_region.world(),
            player_region.dimension().clone(),
            position.chunk(),
        );
        let command = BlockInteractionCommand {
            player: self.player,
            intent,
            eye: eye_position(player.pose().position),
            interaction_range: self.interaction_range,
        }
        .into_region_command(target, target_tick, sequence)?;
        router.route_player_command(command)?;
        self.next_command_sequence = sequence
            .checked_add(1)
            .ok_or(BlockSessionError::SequenceExhausted)?;
        Ok(BlockInteractionAction::Routed {
            command_sequence: sequence,
        })
    }
}

const fn intent_position(intent: BlockIntent) -> BlockPos {
    match intent {
        BlockIntent::StartDestroy { position }
        | BlockIntent::AbortDestroy { position }
        | BlockIntent::StopDestroy { position }
        | BlockIntent::UseOn { position, .. } => position,
    }
}

#[derive(Debug, Error)]
pub enum BlockSessionError {
    #[error(transparent)]
    Command(#[from] BlockCommandError),
    #[error(transparent)]
    Connection(#[from] ServerConnectionError),
    #[error(transparent)]
    Replication(#[from] BlockReplicationError),
    #[error(transparent)]
    Terrain(#[from] TerrainProjectionError),
    #[error(transparent)]
    Route(#[from] PlayerRegionRouteError),
    #[error("block command sequence is exhausted")]
    SequenceExhausted,
    #[error("Java terrain registry map is required for block-state projection")]
    MissingRegistryMap,
}
