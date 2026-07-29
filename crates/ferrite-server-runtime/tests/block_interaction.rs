use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::direction::Direction;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::state::{PlayerPose, Rotation, Vec3};
use ferrite_protocol::java_26_2::play::clientbound::packet::{BlockUpdate, PlayClientboundPacket};
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, MainHand, ParticleStatus, SessionId, SessionIdentity,
};
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_server_runtime::chunk::projection::JavaTerrainRegistryMap;
use ferrite_server_runtime::player::block::command::{BlockIntent, BlockInteractionCommand};
use ferrite_server_runtime::player::block::replication::{
    BlockCommandOutcome, project_committed_blocks,
};
use ferrite_server_runtime::player::logic::PlayerRegionLogic;
use ferrite_server_runtime::session::command::SessionJoinPayload;
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;

const TARGET: BlockPos = BlockPos::new(1, 65, 1);
const TARGET_TWO: BlockPos = BlockPos::new(3, 65, 1);

fn player() -> StableEntityId {
    StableEntityId::new(7).unwrap()
}

fn region() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

fn simulation() -> RegionSimulationState {
    let mut voxels = RegionVoxelState::new(
        region(),
        RegionMapping::V1,
        ChunkLayout::new(
            VerticalSectionRange::new(-4, 24).unwrap(),
            BlockStateId::new(0),
            BiomeId::new(0),
        ),
    )
    .unwrap();
    voxels.ensure_chunk(ChunkPos::new(0, 0)).unwrap();
    voxels.set_block(TARGET, BlockStateId::new(1)).unwrap();
    voxels.set_block(TARGET_TWO, BlockStateId::new(1)).unwrap();
    RegionSimulationState::new(voxels)
}

fn join(tick: GameTick) -> ferrite_simulation::command::RegionCommand {
    SessionJoinPayload {
        session: SessionId::new(1).unwrap(),
        player: player(),
        identity: SessionIdentity {
            profile_id: 7,
            name: "Builder".to_owned(),
        },
        settings: ClientSettings {
            language: "en_us".to_owned(),
            view_distance: 8,
            chat_visibility: ChatVisibility::Full,
            chat_colors: true,
            model_customization: 0xff,
            main_hand: MainHand::Right,
            text_filtering: false,
            allows_listing: true,
            particle_status: ParticleStatus::All,
        },
        transferred: false,
        spawn_pose: PlayerPose::new(Vec3::new(1.5, 64.0, 1.5), Rotation::default()),
    }
    .into_region_command(region(), tick, 1)
    .unwrap()
}

fn interaction(
    tick: GameTick,
    sequence: u64,
    intent: BlockIntent,
) -> ferrite_simulation::command::RegionCommand {
    BlockInteractionCommand {
        player: player(),
        intent,
        eye: Vec3::new(1.5, 65.62, 1.5),
        interaction_range: 4.5,
    }
    .into_region_command(region(), tick, sequence)
    .unwrap()
}

fn registry_map() -> JavaTerrainRegistryMap {
    let mut map = JavaTerrainRegistryMap::new(4, BlockStateId::new(0)).unwrap();
    map.insert_block_state(BlockStateId::new(0), 0).unwrap();
    map.insert_block_state(BlockStateId::new(1), 1).unwrap();
    map.insert_block_state(BlockStateId::new(2), 200).unwrap();
    map
}

#[test]
fn break_mutates_only_on_matching_stop_and_replicates_after_commit() {
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(simulation(), ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    runner.admit_command(join(GameTick::new(1))).unwrap();
    runner
        .admit_command(interaction(
            GameTick::new(1),
            2,
            BlockIntent::StartDestroy { position: TARGET },
        ))
        .unwrap();
    let mut logic = PlayerRegionLogic;
    let start = runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    let map = registry_map();
    let projected = project_committed_blocks(&start, player(), Some(&map)).unwrap();
    assert_eq!(projected.results[0].outcome, BlockCommandOutcome::Tracking);
    assert!(projected.packets.is_empty());
    assert_eq!(
        runner
            .region(&region())
            .unwrap()
            .state()
            .voxels()
            .block_state(TARGET)
            .unwrap(),
        BlockStateId::new(1)
    );

    runner
        .admit_command(interaction(
            GameTick::new(2),
            3,
            BlockIntent::StopDestroy { position: TARGET },
        ))
        .unwrap();
    let stop = runner.run_tick(GameTick::new(2), &mut logic).unwrap();
    let projected = project_committed_blocks(&stop, player(), Some(&map)).unwrap();
    assert_eq!(projected.results[0].outcome, BlockCommandOutcome::Applied);
    assert_eq!(
        projected.packets,
        [PlayClientboundPacket::BlockUpdate(BlockUpdate {
            position: TARGET,
            state: 0,
        })]
    );
}

#[test]
fn use_on_emits_two_direct_corrections_and_a_committed_delta() {
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(simulation(), ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    runner.admit_command(join(GameTick::new(1))).unwrap();
    runner
        .admit_command(interaction(
            GameTick::new(1),
            2,
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
        ))
        .unwrap();
    runner
        .admit_command(interaction(
            GameTick::new(1),
            3,
            BlockIntent::UseOn {
                position: TARGET_TWO,
                direction: Direction::East,
                offset_x: 1.0,
                offset_y: 0.5,
                offset_z: 0.5,
                inside: true,
                world_border_hit: false,
                interaction_allowed: true,
                placement_state: BlockStateId::new(2),
            },
        ))
        .unwrap();
    let report = runner
        .run_tick(GameTick::new(1), &mut PlayerRegionLogic)
        .unwrap();
    let map = registry_map();
    let projected = project_committed_blocks(&report, player(), Some(&map)).unwrap();
    assert!(
        projected
            .results
            .iter()
            .all(|result| result.outcome == BlockCommandOutcome::Applied)
    );
    assert!(
        projected
            .results
            .iter()
            .all(|result| result.corrections.len() == 2)
    );
    assert_eq!(projected.packets.len(), 1);
    let PlayClientboundPacket::SectionBlocksUpdate(section) = &projected.packets[0] else {
        panic!("two committed changes in one section must aggregate");
    };
    assert_eq!(section.changes.len(), 2);
    assert!(section.changes.iter().all(|change| change.state == 200));
    let adjacent = BlockPos::new(2, 65, 1);
    assert_eq!(
        runner
            .region(&region())
            .unwrap()
            .state()
            .voxels()
            .block_state(adjacent)
            .unwrap(),
        BlockStateId::new(2)
    );
}
