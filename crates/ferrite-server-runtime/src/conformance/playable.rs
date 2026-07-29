//! Deterministic C2 scenario shared by local and Lattice-backed acceptance.

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::direction::Direction;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::block::targeting::eye_position;
use ferrite_gameplay::player::collision::NoCollision;
use ferrite_gameplay::player::movement::MovementContext;
use ferrite_gameplay::player::state::{PlayerPose, PlayerSessionState, Rotation, Vec3};
use ferrite_protocol::java_26_2::play::clientbound::codec;
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    BlockChangedAck, BlockUpdate, PlayClientboundPacket,
};
use ferrite_protocol::java_26_2::play::registry::{BIOME, PlayRegistries};
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    MovePlayerPosition, MovementFlags, PlayServerboundEntryPacket, PlayerPosition,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, MainHand, ParticleStatus, PlayAdmission, PlayerSpawn,
    SessionId, SessionIdentity,
};
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig, LocalTickReport};
use ferrite_region_runtime::transfer::EntityTransfer;
use ferrite_replay::hash::StateHash;
use ferrite_replay::projection::{
    BlockStateProjection, EntityStateProjection, RegionStateProjection, hash_projected_world,
};
use ferrite_simulation::command::RegionCommand;
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;
use ferrite_world::terrain::MinimalTerrain;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

use crate::chunk::projection::{JavaTerrainRegistryMap, project_stream_events};
use crate::chunk::session::{ChunkSessionLimits, ClientChunkSession};
use crate::player::block::command::{BlockIntent, BlockInteractionCommand};
use crate::player::block::replication::project_committed_blocks;
use crate::player::logic::PlayerRegionLogic;
use crate::player::router::{
    LatticePlayerRegionRouter, PlayerRegionRouteError, PlayerRegionRouter,
};
use crate::player::session::{PlayerSession, PlayerSessionAction};
use crate::session::command::SessionJoinPayload;

const FRAME_LIMIT: usize = 4 * 1024 * 1024;
const FINAL_TICK: GameTick = GameTick::new(7);
const TARGET: BlockPos = BlockPos::new(129, 65, 8);
const SECOND_TARGET: BlockPos = BlockPos::new(131, 65, 8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayableTopology {
    Local,
    LatticeInProcess,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayableScenarioEvidence {
    pub committed_tick: u64,
    pub committed_hash: String,
    pub packet_trace_digest: String,
    pub packet_trace: Vec<PacketTraceRecord>,
    pub final_region_x: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketTraceRecord {
    pub ordinal: u32,
    pub wire_id: i32,
    pub body_bytes: usize,
    pub body_digest: String,
}

pub fn run_playable_scenario(
    topology: PlayableTopology,
) -> Result<PlayableScenarioEvidence, PlayableScenarioError> {
    let admission = admission();
    let registries = protocol_registries();
    let terrain_map = terrain_registry_map()?;
    let mut trace = PacketTrace::new(registries);
    let mut chunks = stream_initial_terrain(&admission, &terrain_map, &mut trace)?;
    let mut runner = ScenarioRunner::new(topology)?;
    let mut logic = PlayerRegionLogic;

    runner.route_player_command(join_command(&admission, GameTick::new(1))?)?;
    runner.run_tick(GameTick::new(1), &mut logic)?;

    let mut player = PlayerSession::new(admission.clone());
    let loaded = player.handle_packet(
        PlayServerboundEntryPacket::PlayerLoaded,
        false,
        MovementContext::default(),
        &NoCollision,
        GameTick::new(2),
        &mut runner,
    )?;
    require(
        loaded == PlayerSessionAction::PlayerLoaded,
        "player-loaded action",
    )?;
    let report = runner.run_tick(GameTick::new(2), &mut logic)?;
    require(
        matches!(
            player.observe_committed_tick(&report),
            PlayerSessionAction::StateCommitted { .. }
        ),
        "player-loaded state commit",
    )?;

    route_movement(&mut player, &mut runner, 125.5, GameTick::new(3))?;
    let report = runner.run_tick(GameTick::new(3), &mut logic)?;
    require(
        matches!(
            player.observe_committed_tick(&report),
            PlayerSessionAction::StateCommitted { .. }
        ),
        "same-Region movement commit",
    )?;

    let transfer = route_movement(&mut player, &mut runner, 128.5, GameTick::new(4))?;
    require(
        transfer == PlayerSessionAction::RegionTransferStaged,
        "cross-Region transfer stage",
    )?;
    let report = runner.run_tick(GameTick::new(4), &mut logic)?;
    require(
        player.observe_committed_tick(&report) == PlayerSessionAction::RegionTransferCommitted,
        "cross-Region transfer commit",
    )?;
    let recenter = chunks.recenter(ChunkPos::new(8, 0), true)?;
    trace.extend(project_stream_events(recenter, &terrain_map)?)?;

    route_block_pair(&player, &mut runner, GameTick::new(5))?;
    let report = runner.run_tick(GameTick::new(5), &mut logic)?;
    trace.push(PlayClientboundPacket::BlockChangedAck(BlockChangedAck {
        sequence: 11,
    }))?;
    trace_block_projection(&report, admission.player, &terrain_map, &mut trace)?;

    route_break(
        &player,
        &mut runner,
        GameTick::new(6),
        102,
        BlockIntent::StartDestroy { position: TARGET },
    )?;
    let start = runner.run_tick(GameTick::new(6), &mut logic)?;
    trace.push(PlayClientboundPacket::BlockChangedAck(BlockChangedAck {
        sequence: 12,
    }))?;
    trace_block_projection(&start, admission.player, &terrain_map, &mut trace)?;

    route_break(
        &player,
        &mut runner,
        FINAL_TICK,
        103,
        BlockIntent::StopDestroy { position: TARGET },
    )?;
    let stop = runner.run_tick(FINAL_TICK, &mut logic)?;
    trace.push(PlayClientboundPacket::BlockChangedAck(BlockChangedAck {
        sequence: 13,
    }))?;
    trace_block_projection(&stop, admission.player, &terrain_map, &mut trace)?;

    verify_final_state(&runner, admission.player)?;
    let committed_hash = hash_runner(&runner, FINAL_TICK)?;
    let (packet_trace, packet_trace_digest) = trace.finish();
    Ok(PlayableScenarioEvidence {
        committed_tick: FINAL_TICK.get(),
        committed_hash: committed_hash.to_string(),
        packet_trace_digest,
        packet_trace,
        final_region_x: player.region().coordinate().x(),
    })
}

struct ScenarioRunner {
    topology: PlayableTopology,
    ingress: SimulationRegionKey,
    runner: LocalRegionRunner,
}

impl ScenarioRunner {
    fn new(topology: PlayableTopology) -> Result<Self, PlayableScenarioError> {
        let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing())?;
        runner.insert_region(
            region_state(0)?,
            ActivationGeneration::INITIAL,
            GameTick::ZERO,
        )?;
        runner.insert_region(
            region_state(1)?,
            ActivationGeneration::INITIAL,
            GameTick::ZERO,
        )?;
        Ok(Self {
            topology,
            ingress: region(9),
            runner,
        })
    }

    fn run_tick(
        &mut self,
        tick: GameTick,
        logic: &mut PlayerRegionLogic,
    ) -> Result<LocalTickReport, PlayableScenarioError> {
        Ok(self.runner.run_tick(tick, logic)?)
    }
}

impl PlayerRegionRouter for ScenarioRunner {
    fn route_player_command(
        &mut self,
        command: RegionCommand,
    ) -> Result<(), PlayerRegionRouteError> {
        match self.topology {
            PlayableTopology::Local => self.runner.route_player_command(command),
            PlayableTopology::LatticeInProcess => LatticePlayerRegionRouter::new(
                &mut self.runner,
                self.ingress.clone(),
                ActivationGeneration::INITIAL,
                FRAME_LIMIT,
            )?
            .route_player_command(command),
        }
    }

    fn route_player_transfer(
        &mut self,
        transfer: EntityTransfer,
    ) -> Result<(), PlayerRegionRouteError> {
        match self.topology {
            PlayableTopology::Local => self.runner.route_player_transfer(transfer),
            PlayableTopology::LatticeInProcess => LatticePlayerRegionRouter::new(
                &mut self.runner,
                self.ingress.clone(),
                ActivationGeneration::INITIAL,
                FRAME_LIMIT,
            )?
            .route_player_transfer(transfer),
        }
    }

    fn activation_generation(
        &self,
        region: &SimulationRegionKey,
    ) -> Result<ActivationGeneration, PlayerRegionRouteError> {
        self.runner
            .activation_generation(region)
            .map_err(Into::into)
    }
}

fn route_movement(
    player: &mut PlayerSession,
    runner: &mut ScenarioRunner,
    x: f64,
    tick: GameTick,
) -> Result<PlayerSessionAction, PlayableScenarioError> {
    Ok(player.handle_packet(
        PlayServerboundEntryPacket::MovePlayerPosition(MovePlayerPosition {
            position: PlayerPosition { x, y: 65.0, z: 8.5 },
            flags: MovementFlags {
                on_ground: true,
                horizontal_collision: false,
            },
        }),
        false,
        MovementContext::default(),
        &NoCollision,
        tick,
        runner,
    )?)
}

fn route_block_pair(
    player: &PlayerSession,
    runner: &mut ScenarioRunner,
    tick: GameTick,
) -> Result<(), PlayableScenarioError> {
    let eye = eye_position(player.state().pose().position);
    for (sequence, intent) in [
        (
            100,
            BlockIntent::UseOn {
                position: TARGET,
                direction: Direction::East,
                offset_x: 1.0,
                offset_y: 0.5,
                offset_z: 0.5,
                inside: false,
                world_border_hit: false,
                interaction_allowed: true,
                placement_state: BlockStateId::new(2),
            },
        ),
        (
            101,
            BlockIntent::UseOn {
                position: SECOND_TARGET,
                direction: Direction::East,
                offset_x: 1.0,
                offset_y: 0.5,
                offset_z: 0.5,
                inside: false,
                world_border_hit: false,
                interaction_allowed: false,
                placement_state: BlockStateId::new(2),
            },
        ),
    ] {
        runner.route_player_command(
            BlockInteractionCommand {
                player: player_id(),
                intent,
                eye,
                interaction_range: 4.5,
            }
            .into_region_command(region(1), tick, sequence)?,
        )?;
    }
    Ok(())
}

fn route_break(
    player: &PlayerSession,
    runner: &mut ScenarioRunner,
    tick: GameTick,
    sequence: u64,
    intent: BlockIntent,
) -> Result<(), PlayableScenarioError> {
    runner.route_player_command(
        BlockInteractionCommand {
            player: player_id(),
            intent,
            eye: eye_position(player.state().pose().position),
            interaction_range: 4.5,
        }
        .into_region_command(region(1), tick, sequence)?,
    )?;
    Ok(())
}

fn trace_block_projection(
    report: &LocalTickReport,
    player: StableEntityId,
    terrain_map: &JavaTerrainRegistryMap,
    trace: &mut PacketTrace,
) -> Result<(), PlayableScenarioError> {
    let projection = project_committed_blocks(report, player, Some(terrain_map))?;
    for result in &projection.results {
        for correction in &result.corrections {
            trace.push(PlayClientboundPacket::BlockUpdate(BlockUpdate {
                position: correction.position,
                state: terrain_map.block_state(correction.state)?,
            }))?;
        }
    }
    trace.extend(projection.packets)
}

fn stream_initial_terrain(
    admission: &PlayAdmission,
    terrain_map: &JavaTerrainRegistryMap,
    trace: &mut PacketTrace,
) -> Result<ClientChunkSession, PlayableScenarioError> {
    let limits = ChunkSessionLimits {
        maximum_tracked_chunks: 25,
        maximum_tickets: 26,
        maximum_chunks_per_batch: 2,
    };
    let mut chunks = ClientChunkSession::join(admission, 2, 10, limits)?;
    trace.extend(project_stream_events(
        chunks.initial_events().to_vec(),
        terrain_map,
    )?)?;
    chunks.mark_ready(admission.spawn_chunk)?;
    let terrain = minimal_terrain()?;
    let prepared = chunks
        .prepare_next_batch(|position| terrain.snapshot(position).ok())?
        .ok_or_else(|| failure("initial terrain did not produce a batch"))?;
    let packets = project_stream_events(prepared.events().to_vec(), terrain_map)?;
    chunks.commit_prepared_batch(prepared)?;
    trace.extend(packets)?;
    chunks.acknowledge_batch(9.0)?;
    Ok(chunks)
}

fn verify_final_state(
    runner: &ScenarioRunner,
    player: StableEntityId,
) -> Result<(), PlayableScenarioError> {
    let left = runner
        .runner
        .region(&region(0))
        .ok_or_else(|| failure("left Region disappeared"))?;
    let right = runner
        .runner
        .region(&region(1))
        .ok_or_else(|| failure("right Region disappeared"))?;
    require(
        !left.state().entities().contains(player) && right.state().entities().contains(player),
        "final player ownership",
    )?;
    require(
        right.state().voxels().block_state(TARGET)? == BlockStateId::new(0),
        "destroyed block state",
    )?;
    require(
        right
            .state()
            .voxels()
            .block_state(BlockPos::new(130, 65, 8))?
            == BlockStateId::new(2),
        "placed block state",
    )
}

fn hash_runner(
    runner: &ScenarioRunner,
    committed_tick: GameTick,
) -> Result<StateHash, PlayableScenarioError> {
    let projections = [region(0), region(1)]
        .into_iter()
        .map(|key| project_region(runner, key))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(hash_projected_world(
        WorldId::new(1).map_err(|error| detail("world identity", error))?,
        committed_tick,
        StateHash::from_bytes([0x26; 32]),
        &projections,
    )?)
}

fn project_region(
    runner: &ScenarioRunner,
    key: SimulationRegionKey,
) -> Result<RegionStateProjection, PlayableScenarioError> {
    let view = runner
        .runner
        .region(&key)
        .ok_or_else(|| failure("Region missing during hash projection"))?
        .state();
    let mut blocks = Vec::new();
    for (chunk_pos, chunk) in view.voxels().chunks() {
        let sections = chunk.layout().sections();
        for y in sections.minimum() * 16..sections.maximum_exclusive() * 16 {
            for local_z in 0..16 {
                for local_x in 0..16 {
                    let position =
                        BlockPos::new(chunk_pos.x * 16 + local_x, y, chunk_pos.z * 16 + local_z);
                    let state = chunk.block_state(position)?;
                    if state != BlockStateId::new(0) {
                        blocks.push(BlockStateProjection::new(position, block_identity(state)?));
                    }
                }
            }
        }
    }
    let mut entities = Vec::new();
    for stable_id in view.entities().stable_ids() {
        let state = view
            .entities()
            .component::<PlayerSessionState>(stable_id)
            .ok_or_else(|| failure("playable entity lacks player state"))?;
        entities.push(EntityStateProjection::new(
            stable_id,
            ResourceId::minecraft("player")?,
            state.encode_transfer(),
        )?);
    }
    Ok(RegionStateProjection::new(
        key,
        RegionMapping::V1,
        blocks,
        entities,
        Vec::new(),
    )?)
}

fn block_identity(state: BlockStateId) -> Result<ResourceId, PlayableScenarioError> {
    match state.get() {
        1 => Ok(ResourceId::minecraft("stone")?),
        2 => Ok(ResourceId::minecraft("oak_planks")?),
        value => Err(failure(format!("unmapped block state {value}"))),
    }
}

struct PacketTrace {
    registries: PlayRegistries,
    hasher: blake3::Hasher,
    records: Vec<PacketTraceRecord>,
}

impl PacketTrace {
    fn new(registries: PlayRegistries) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"ferrite-playable-packet-trace-v1");
        Self {
            registries,
            hasher,
            records: Vec::new(),
        }
    }

    fn push(&mut self, packet: PlayClientboundPacket) -> Result<(), PlayableScenarioError> {
        let body = codec::encode_packet(&packet, &self.registries)?;
        let wire_id = read_var_i32(&body)?;
        let ordinal = u32::try_from(self.records.len())
            .map_err(|error| detail("packet trace ordinal", error))?;
        self.hasher.update(&ordinal.to_be_bytes());
        self.hasher.update(&(body.len() as u64).to_be_bytes());
        self.hasher.update(&body);
        self.records.push(PacketTraceRecord {
            ordinal,
            wire_id,
            body_bytes: body.len(),
            body_digest: blake3::hash(&body).to_hex().to_string(),
        });
        Ok(())
    }

    fn extend(
        &mut self,
        packets: impl IntoIterator<Item = PlayClientboundPacket>,
    ) -> Result<(), PlayableScenarioError> {
        for packet in packets {
            self.push(packet)?;
        }
        Ok(())
    }

    fn finish(self) -> (Vec<PacketTraceRecord>, String) {
        (self.records, self.hasher.finalize().to_hex().to_string())
    }
}

fn read_var_i32(bytes: &[u8]) -> Result<i32, PlayableScenarioError> {
    let mut value = 0_u32;
    for (index, byte) in bytes.iter().copied().take(5).enumerate() {
        value |= u32::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            return Ok(value as i32);
        }
    }
    Err(failure("invalid packet identity VarInt"))
}

fn admission() -> PlayAdmission {
    PlayAdmission {
        session: SessionId::new(1).expect("locked session identity is valid"),
        identity: SessionIdentity {
            profile_id: 7,
            name: "TopologyWalker".to_owned(),
        },
        player: player_id(),
        region: region(0),
        region_mapping: RegionMapping::V1,
        spawn_chunk: ChunkPos::new(7, 0),
        spawn: PlayerSpawn {
            x: 120.5,
            y: 65.0,
            z: 8.5,
            yaw: 0.0,
            pitch: 0.0,
        },
        requested_view_distance: 2,
        transferred: false,
    }
}

fn join_command(
    admission: &PlayAdmission,
    tick: GameTick,
) -> Result<RegionCommand, PlayableScenarioError> {
    Ok(SessionJoinPayload {
        session: admission.session,
        player: admission.player,
        identity: admission.identity.clone(),
        settings: ClientSettings {
            language: "en_us".to_owned(),
            view_distance: 2,
            chat_visibility: ChatVisibility::Full,
            chat_colors: true,
            model_customization: 0xff,
            main_hand: MainHand::Right,
            text_filtering: false,
            allows_listing: true,
            particle_status: ParticleStatus::All,
        },
        transferred: false,
        spawn_pose: PlayerPose::new(
            Vec3::new(admission.spawn.x, admission.spawn.y, admission.spawn.z),
            Rotation::default(),
        ),
    }
    .into_region_command(region(0), tick, 0)?)
}

fn region_state(x: i32) -> Result<RegionSimulationState, PlayableScenarioError> {
    let mut voxels = RegionVoxelState::new(region(x), RegionMapping::V1, chunk_layout())?;
    let chunk = if x == 0 {
        ChunkPos::new(7, 0)
    } else {
        ChunkPos::new(8, 0)
    };
    voxels.ensure_chunk(chunk)?;
    if x == 1 {
        voxels.set_block(TARGET, BlockStateId::new(1))?;
        voxels.set_block(SECOND_TARGET, BlockStateId::new(1))?;
    }
    Ok(RegionSimulationState::new(voxels))
}

fn minimal_terrain() -> Result<MinimalTerrain, PlayableScenarioError> {
    Ok(MinimalTerrain::new(
        chunk_layout(),
        BlockStateId::new(0),
        BlockStateId::new(1),
        BiomeId::new(0),
        63,
    )?)
}

fn chunk_layout() -> ChunkLayout {
    ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).expect("locked vertical range is valid"),
        BlockStateId::new(0),
        BiomeId::new(0),
    )
}

fn terrain_registry_map() -> Result<JavaTerrainRegistryMap, PlayableScenarioError> {
    let mut map = JavaTerrainRegistryMap::new(4, BlockStateId::new(0))?;
    map.insert_block_state(BlockStateId::new(0), 0)?;
    map.insert_block_state(BlockStateId::new(1), 1)?;
    map.insert_block_state(BlockStateId::new(2), 2)?;
    map.insert_biome(BiomeId::new(0), 0)?;
    Ok(map)
}

fn protocol_registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        Identifier::parse(BIOME).expect("locked biome registry identity is valid"),
        vec![Identifier::parse("minecraft:plains").expect("locked biome identity is valid")],
    );
    registries
}

fn region(x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).expect("locked world identity is valid"),
        DimensionId::new(
            ResourceId::minecraft("overworld").expect("locked dimension identity is valid"),
        ),
        RegionCoord::new(x, 0),
        RegionMappingVersion::V1,
    )
}

fn player_id() -> StableEntityId {
    StableEntityId::new(7).expect("locked player identity is valid")
}

fn require(condition: bool, expectation: &'static str) -> Result<(), PlayableScenarioError> {
    if condition {
        Ok(())
    } else {
        Err(failure(format!(
            "playable scenario did not satisfy {expectation}"
        )))
    }
}

fn failure(detail: impl Into<String>) -> PlayableScenarioError {
    PlayableScenarioError::Failure {
        detail: detail.into(),
    }
}

fn detail(context: &'static str, error: impl Display) -> PlayableScenarioError {
    failure(format!("{context}: {error}"))
}

#[derive(Debug, Error)]
pub enum PlayableScenarioError {
    #[error("{detail}")]
    Failure { detail: String },
    #[error(transparent)]
    Local(#[from] ferrite_region_runtime::local::LocalRunnerError),
    #[error(transparent)]
    Route(#[from] PlayerRegionRouteError),
    #[error(transparent)]
    Player(#[from] crate::player::session::PlayerSessionError),
    #[error(transparent)]
    SessionCommand(#[from] crate::session::command::SessionCommandError),
    #[error(transparent)]
    BlockCommand(#[from] crate::player::block::command::BlockCommandError),
    #[error(transparent)]
    BlockReplication(#[from] crate::player::block::replication::BlockReplicationError),
    #[error(transparent)]
    ChunkSession(#[from] crate::chunk::session::ClientChunkSessionError),
    #[error(transparent)]
    TerrainProjection(#[from] crate::chunk::projection::TerrainProjectionError),
    #[error(transparent)]
    Protocol(
        #[from] ferrite_protocol::java_26_2::play::clientbound::codec::PlayClientboundCodecError,
    ),
    #[error(transparent)]
    Resource(#[from] ferrite_foundation::resource::ResourceIdError),
    #[error(transparent)]
    RegionVoxel(#[from] ferrite_world::region::RegionVoxelError),
    #[error(transparent)]
    ChunkAccess(#[from] ferrite_world::chunk::ChunkAccessError),
    #[error(transparent)]
    Terrain(#[from] ferrite_world::terrain::MinimalTerrainError),
    #[error(transparent)]
    Projection(#[from] ferrite_replay::projection::ProjectionError),
    #[error(transparent)]
    ProjectionHash(#[from] ferrite_replay::projection::ProjectionHashError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_and_lattice_playable_evidence_is_identical() {
        let local = run_playable_scenario(PlayableTopology::Local).unwrap();
        let lattice = run_playable_scenario(PlayableTopology::LatticeInProcess).unwrap();
        assert_eq!(local, lattice);
        assert_eq!(local.committed_tick, 7);
        assert_eq!(local.final_region_x, 1);
        assert_eq!(
            local.committed_hash,
            "1e7c50dbf4463c858fcd779f4db59a08418e54cab7ae0e502821bba95ad0a858"
        );
        assert_eq!(
            local.packet_trace_digest,
            "8328cdaa1bf165640fc44b8db0be6727c5445c302983dff9bcbfa36e16fcf95e"
        );
        assert_eq!(local.packet_trace.len(), 16);
        let wire_ids = local
            .packet_trace
            .iter()
            .map(|record| record.wire_id)
            .collect::<Vec<_>>();
        assert!(wire_ids.contains(&45), "full terrain packet is present");
        assert_eq!(
            wire_ids.iter().filter(|wire_id| **wire_id == 4).count(),
            3,
            "three cumulative prediction acknowledgements are present"
        );
        assert!(
            wire_ids.iter().filter(|wire_id| **wire_id == 8).count() >= 3,
            "authoritative corrections and committed block updates are present"
        );
    }
}
