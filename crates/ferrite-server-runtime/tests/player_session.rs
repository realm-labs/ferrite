use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::collision::NoCollision;
use ferrite_gameplay::player::movement::{MovementContext, MovementOutcome};
use ferrite_gameplay::player::state::{PlayerPose, PlayerSessionState, Rotation, Vec3};
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    MovePlayerPosition, MovementFlags, PlayServerboundEntryPacket, PlayerPosition,
};
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, MainHand, ParticleStatus, PlayAdmission, PlayerSpawn,
    SessionId, SessionIdentity,
};
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_server_runtime::player::logic::PlayerRegionLogic;
use ferrite_server_runtime::player::session::{PlayerSession, PlayerSessionAction};
use ferrite_server_runtime::session::command::SessionJoinPayload;
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;

fn dimension() -> DimensionId {
    DimensionId::new(ResourceId::minecraft("overworld").unwrap())
}

fn region(x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        dimension(),
        RegionCoord::new(x, 0),
        RegionMappingVersion::V1,
    )
}

fn state(x: i32) -> RegionSimulationState {
    let voxels = RegionVoxelState::new(
        region(x),
        RegionMapping::V1,
        ChunkLayout::new(
            VerticalSectionRange::new(-4, 24).unwrap(),
            BlockStateId::new(0),
            BiomeId::new(0),
        ),
    )
    .unwrap();
    RegionSimulationState::new(voxels)
}

fn settings() -> ClientSettings {
    ClientSettings {
        language: "en_us".to_owned(),
        view_distance: 8,
        chat_visibility: ChatVisibility::Full,
        chat_colors: true,
        model_customization: 0xff,
        main_hand: MainHand::Right,
        text_filtering: false,
        allows_listing: true,
        particle_status: ParticleStatus::All,
    }
}

fn admission() -> PlayAdmission {
    PlayAdmission {
        session: SessionId::new(1).unwrap(),
        identity: SessionIdentity {
            profile_id: 7,
            name: "RegionWalker".to_owned(),
        },
        player: StableEntityId::new(7).unwrap(),
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
        requested_view_distance: 8,
        transferred: false,
    }
}

fn join_payload(admission: &PlayAdmission) -> SessionJoinPayload {
    SessionJoinPayload {
        session: admission.session,
        player: admission.player,
        identity: admission.identity.clone(),
        settings: settings(),
        transferred: admission.transferred,
        spawn_pose: PlayerPose::new(
            Vec3::new(admission.spawn.x, admission.spawn.y, admission.spawn.z),
            Rotation {
                yaw: admission.spawn.yaw,
                pitch: admission.spawn.pitch,
            },
        ),
    }
}

fn move_position(x: f64) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::MovePlayerPosition(MovePlayerPosition {
        position: PlayerPosition { x, y: 65.0, z: 8.5 },
        flags: MovementFlags {
            on_ground: true,
            horizontal_collision: false,
        },
    })
}

#[test]
fn player_spawns_and_region_owner_switches_only_after_transfer_commit() {
    let admission = admission();
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(state(0), ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    runner
        .insert_region(state(1), ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    runner
        .admit_command(
            join_payload(&admission)
                .into_region_command(region(0), GameTick::new(1), 0)
                .unwrap(),
        )
        .unwrap();
    let mut logic = PlayerRegionLogic;
    runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    assert!(
        runner
            .region(&region(0))
            .unwrap()
            .state()
            .entities()
            .component::<PlayerSessionState>(admission.player)
            .is_some()
    );

    let mut session = PlayerSession::new(admission.clone());
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
    assert_eq!(
        session
            .handle_packet(
                move_position(128.5),
                false,
                MovementContext::default(),
                &NoCollision,
                GameTick::new(2),
                &mut runner,
            )
            .unwrap(),
        PlayerSessionAction::RegionTransferStaged
    );
    assert_eq!(session.region(), &region(0));
    assert!(session.transfer_pending());
    assert_eq!(
        session
            .handle_packet(
                move_position(129.5),
                false,
                MovementContext::default(),
                &NoCollision,
                GameTick::new(2),
                &mut runner,
            )
            .unwrap(),
        PlayerSessionAction::AwaitingRegionTransfer
    );

    let report = runner.run_tick(GameTick::new(2), &mut logic).unwrap();
    assert_eq!(session.region(), &region(0));
    assert_eq!(
        session.observe_committed_tick(&report),
        PlayerSessionAction::RegionTransferCommitted
    );
    assert_eq!(session.region(), &region(1));
    assert!(!session.transfer_pending());
    assert!(
        !runner
            .region(&region(0))
            .unwrap()
            .state()
            .entities()
            .contains(admission.player)
    );
    let target_state = runner
        .region(&region(1))
        .unwrap()
        .state()
        .entities()
        .component::<PlayerSessionState>(admission.player)
        .unwrap();
    assert_eq!(target_state.pose().position, Vec3::new(128.5, 65.0, 8.5));
}

#[test]
fn failed_region_route_rolls_back_the_connection_side_movement_transaction() {
    let admission = admission();
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(state(0), ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    let mut session = PlayerSession::new(admission);
    session
        .handle_packet(
            PlayServerboundEntryPacket::PlayerLoaded,
            false,
            MovementContext::default(),
            &NoCollision,
            GameTick::new(1),
            &mut runner,
        )
        .unwrap();
    let before = session.state().clone();
    assert!(
        session
            .handle_packet(
                move_position(128.5),
                false,
                MovementContext::default(),
                &NoCollision,
                GameTick::new(1),
                &mut runner,
            )
            .is_err()
    );
    assert_eq!(session.state(), &before);
    assert!(!session.transfer_pending());
}

#[test]
fn same_region_movement_projects_to_region_ecs_on_the_target_tick() {
    let admission = admission();
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(state(0), ActivationGeneration::INITIAL, GameTick::ZERO)
        .unwrap();
    runner
        .admit_command(
            join_payload(&admission)
                .into_region_command(region(0), GameTick::new(1), 0)
                .unwrap(),
        )
        .unwrap();
    let mut logic = PlayerRegionLogic;
    runner.run_tick(GameTick::new(1), &mut logic).unwrap();
    let mut session = PlayerSession::new(admission.clone());
    session
        .handle_packet(
            PlayServerboundEntryPacket::PlayerLoaded,
            false,
            MovementContext::default(),
            &NoCollision,
            GameTick::new(2),
            &mut runner,
        )
        .unwrap();
    let action = session
        .handle_packet(
            move_position(121.5),
            false,
            MovementContext::default(),
            &NoCollision,
            GameTick::new(2),
            &mut runner,
        )
        .unwrap();
    assert!(matches!(
        action,
        PlayerSessionAction::Movement(MovementOutcome::Accepted { .. })
    ));
    let report = runner.run_tick(GameTick::new(2), &mut logic).unwrap();
    assert_eq!(
        session.observe_committed_tick(&report),
        PlayerSessionAction::StateCommitted {
            recenter: Some(ChunkPos::new(7, 0)),
        }
    );
    assert_eq!(
        session.committed_state().pose().position,
        Vec3::new(121.5, 65.0, 8.5)
    );
    let state = runner
        .region(&region(0))
        .unwrap()
        .state()
        .entities()
        .component::<PlayerSessionState>(admission.player)
        .unwrap();
    assert_eq!(state.pose().position, Vec3::new(121.5, 65.0, 8.5));
}
