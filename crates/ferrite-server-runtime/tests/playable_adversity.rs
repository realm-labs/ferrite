use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::collision::NoCollision;
use ferrite_gameplay::player::movement::MovementContext;
use ferrite_gameplay::player::state::{PlayerPose, PlayerSessionState, Rotation, Vec3};
use ferrite_protocol::java_26_2::play::serverbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    MovePlayerPosition, MovementFlags, PlayServerboundEntryPacket, PlayerPosition,
};
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, MainHand, ParticleStatus, PlayAdmission, PlayerSpawn,
    SessionId, SessionIdentity,
};
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig, LocalRunnerError};
use ferrite_server_runtime::chunk::session::{ChunkSessionLimits, ClientChunkSession};
use ferrite_server_runtime::conformance::playable::{PlayableTopology, run_playable_scenario};
use ferrite_server_runtime::player::logic::PlayerRegionLogic;
use ferrite_server_runtime::player::session::{PlayerSession, PlayerSessionAction};
use ferrite_server_runtime::session::command::SessionJoinPayload;
use ferrite_simulation::command::{CommandError, CommandSource, RegionCommand};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;
use ferrite_world::terrain::MinimalTerrain;

fn region() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

fn player() -> StableEntityId {
    StableEntityId::new(7).unwrap()
}

fn admission() -> PlayAdmission {
    PlayAdmission {
        session: SessionId::new(1).unwrap(),
        identity: SessionIdentity {
            profile_id: 7,
            name: "BackpressureWalker".to_owned(),
        },
        player: player(),
        region: region(),
        region_mapping: RegionMapping::V1,
        spawn_chunk: ChunkPos::new(0, 0),
        spawn: PlayerSpawn {
            x: 8.5,
            y: 65.0,
            z: 8.5,
            yaw: 0.0,
            pitch: 0.0,
        },
        requested_view_distance: 2,
        transferred: false,
    }
}

fn settings() -> ClientSettings {
    ClientSettings {
        language: "en_us".to_owned(),
        view_distance: 2,
        chat_visibility: ChatVisibility::Full,
        chat_colors: true,
        model_customization: 0xff,
        main_hand: MainHand::Right,
        text_filtering: false,
        allows_listing: true,
        particle_status: ParticleStatus::All,
    }
}

fn join(tick: GameTick) -> RegionCommand {
    let admission = admission();
    SessionJoinPayload {
        session: admission.session,
        player: admission.player,
        identity: admission.identity,
        settings: settings(),
        transferred: false,
        spawn_pose: PlayerPose::new(
            Vec3::new(admission.spawn.x, admission.spawn.y, admission.spawn.z),
            Rotation::default(),
        ),
    }
    .into_region_command(region(), tick, 0)
    .unwrap()
}

fn simulation() -> RegionSimulationState {
    RegionSimulationState::new(
        RegionVoxelState::new(region(), RegionMapping::V1, layout()).unwrap(),
    )
}

fn layout() -> ChunkLayout {
    ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).unwrap(),
        BlockStateId::new(0),
        BiomeId::new(0),
    )
}

fn terrain() -> MinimalTerrain {
    MinimalTerrain::new(
        layout(),
        BlockStateId::new(0),
        BlockStateId::new(1),
        BiomeId::new(0),
        63,
    )
    .unwrap()
}

#[test]
fn delayed_chunk_feedback_holds_and_then_releases_bounded_streaming() {
    let mut chunks = ClientChunkSession::join(
        &admission(),
        2,
        10,
        ChunkSessionLimits {
            maximum_tracked_chunks: 25,
            maximum_tickets: 26,
            maximum_chunks_per_batch: 1,
        },
    )
    .unwrap();
    for position in [
        ChunkPos::new(0, 0),
        ChunkPos::new(1, 0),
        ChunkPos::new(0, 1),
    ] {
        chunks.mark_ready(position).unwrap();
    }
    let terrain = terrain();
    let first = chunks
        .prepare_next_batch(|position| terrain.snapshot(position).ok())
        .unwrap()
        .unwrap();
    chunks.commit_prepared_batch(first).unwrap();
    assert_eq!(chunks.stream().unacknowledged_batches(), 1);
    assert!(
        chunks
            .prepare_next_batch(|position| terrain.snapshot(position).ok())
            .unwrap()
            .is_none(),
        "the initial one-batch window applies while feedback is delayed"
    );

    chunks.acknowledge_batch(64.0).unwrap();
    let second = chunks
        .prepare_next_batch(|position| terrain.snapshot(position).ok())
        .unwrap()
        .expect("feedback reopens the bounded stream window");
    chunks.commit_prepared_batch(second).unwrap();
    assert_eq!(chunks.stream().desired_chunks_per_tick(), 64.0);
}

#[test]
fn command_backpressure_rolls_back_connection_movement_without_dropping_admitted_work() {
    let mut config = LocalRunnerConfig::testing();
    config.command_capacity = 1;
    let mut runner = LocalRegionRunner::new(config).unwrap();
    runner
        .insert_region(simulation(), ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    runner.admit_command(join(GameTick::new(1))).unwrap();
    runner
        .run_tick(GameTick::new(1), &mut PlayerRegionLogic)
        .unwrap();

    let mut session = PlayerSession::new(admission());
    assert_eq!(
        session
            .handle_packet(
                PlayServerboundEntryPacket::PlayerLoaded,
                false,
                MovementContext::default(),
                &NoCollision,
                GameTick::new(2),
                &mut runner,
            )
            .unwrap(),
        PlayerSessionAction::PlayerLoaded
    );
    let loaded = runner
        .run_tick(GameTick::new(2), &mut PlayerRegionLogic)
        .unwrap();
    session.observe_committed_tick(&loaded);

    let blocker = RegionCommand::new(
        region(),
        GameTick::new(3),
        CommandSource::System(ResourceId::new("ferrite", "adversity/blocker").unwrap()),
        0,
        ResourceId::new("ferrite", "adversity/blocker").unwrap(),
        Vec::new(),
    )
    .unwrap();
    runner.admit_command(blocker.clone()).unwrap();
    let before = session.state().clone();
    let error = session
        .handle_packet(
            PlayServerboundEntryPacket::MovePlayerPosition(MovePlayerPosition {
                position: PlayerPosition {
                    x: 9.5,
                    y: 65.0,
                    z: 8.5,
                },
                flags: MovementFlags {
                    on_ground: true,
                    horizontal_collision: false,
                },
            }),
            false,
            MovementContext::default(),
            &NoCollision,
            GameTick::new(3),
            &mut runner,
        )
        .unwrap_err();
    assert!(matches!(
        error,
        ferrite_server_runtime::player::session::PlayerSessionError::Route(
            ferrite_server_runtime::player::router::PlayerRegionRouteError::Local(
                LocalRunnerError::Command(CommandError::Full { capacity: 1 })
            )
        )
    ));
    assert_eq!(session.state(), &before);

    let report = runner
        .run_tick(GameTick::new(3), &mut PlayerRegionLogic)
        .unwrap();
    assert_eq!(report.committed_commands().len(), 1);
    assert_eq!(report.committed_commands()[0].kind, blocker.kind().clone());
    let authoritative = runner
        .region(&region())
        .unwrap()
        .state()
        .entities()
        .component::<PlayerSessionState>(player())
        .unwrap();
    assert_eq!(authoritative.pose().position, before.pose().position);
}

#[test]
fn malformed_c2_bodies_fail_and_cross_region_evidence_stays_topology_equal() {
    for mut body in [
        encode_packet(PlayServerboundEntryPacket::PlayerLoaded).unwrap(),
        encode_packet(PlayServerboundEntryPacket::MovePlayerPosition(
            MovePlayerPosition {
                position: PlayerPosition {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                flags: MovementFlags {
                    on_ground: false,
                    horizontal_collision: false,
                },
            },
        ))
        .unwrap(),
    ] {
        body.push(0);
        assert!(decode_packet(&body).is_err());
        body.truncate(body.len().saturating_sub(2));
        assert!(decode_packet(&body).is_err());
    }

    let local = run_playable_scenario(PlayableTopology::Local).unwrap();
    let lattice = run_playable_scenario(PlayableTopology::LatticeInProcess).unwrap();
    assert_eq!(local, lattice);
    assert_eq!(local.final_region_x, 1);
}
