use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::state::{PlayerPose, Rotation, Vec3};
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, MainHand, ParticleStatus, SessionId, SessionIdentity,
};
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig, LocalRunnerError};
use ferrite_server_runtime::composite::gateway::{CompositeGatewayError, CompositeRegionRouter};
use ferrite_server_runtime::composite::projection::{
    DeferredProjectionKind, SessionProjectionQueue, decode_projection,
};
use ferrite_server_runtime::composite::replay::ProductionTickReplayEvidence;
use ferrite_server_runtime::composite::runtime::CompositeRuntimeConfig;
use ferrite_server_runtime::composite::services::{
    CompositeProductionRegionRuntime, CompositeProductionRuntimeConfig,
    CompositeServiceRuntimeError,
};
use ferrite_server_runtime::entity_service::runtime::EntityServiceRuntimeLimits;
use ferrite_server_runtime::session::command::SessionJoinPayload;
use ferrite_server_runtime::session::router::RegionCommandRouter;
use ferrite_server_runtime::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
use ferrite_server_runtime::simulation::runtime::SimulationRuntimeConfig;
use ferrite_server_runtime::world_service::model::WorldServiceRuntimeConfig;
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;

fn key() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

fn layout() -> ChunkLayout {
    ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).unwrap(),
        BlockStateId::new(0),
        BiomeId::new(0),
    )
}

fn config(projection_capacity: usize) -> CompositeProductionRuntimeConfig {
    CompositeProductionRuntimeConfig {
        coordinator: CompositeRuntimeConfig {
            command_capacity: 32,
            event_capacity: 32,
            projection_capacity,
            continuity_record_capacity: 64,
            maximum_future_ticks: 4,
            maximum_payload_bytes: 1024 * 1024,
        },
        simulation: SimulationRuntimeConfig {
            mapping: RegionMapping::V1,
            budget: SimulationQueueBudget::new([
                (SimulationQueueKind::ScheduledBlocks, 32),
                (SimulationQueueKind::ScheduledFluids, 32),
                (SimulationQueueKind::BoundaryTransactions, 32),
                (SimulationQueueKind::ImmediateNeighbors, 32),
                (SimulationQueueKind::Fluids, 32),
                (SimulationQueueKind::Redstone, 32),
                (SimulationQueueKind::Lighting, 32),
                (SimulationQueueKind::ProjectionPositions, 32),
            ])
            .unwrap(),
            projection_capacity: 32,
            receipt_capacity: 32,
            gameplay_random_seed: 17,
        },
        entities: EntityServiceRuntimeLimits::new(8, 8, 8, 8),
        world: WorldServiceRuntimeConfig {
            mapping: RegionMapping::V1,
            layout: layout(),
            region_side_chunks: 8,
            chunk_capacity: 8,
            event_capacity: 8,
            content_manifest: [17; 32],
        },
        player_capacity: 8,
        projection_capacity_per_player: 8,
    }
}

fn router(projection_capacity: usize) -> CompositeRegionRouter {
    let region = key();
    let voxels = RegionVoxelState::new(region.clone(), RegionMapping::V1, layout()).unwrap();
    let mut local = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    local
        .insert_region(
            RegionSimulationState::new(voxels),
            ActivationGeneration::INITIAL,
            GameTick::ZERO,
        )
        .unwrap();
    let composite = CompositeProductionRegionRuntime::new(
        region,
        ActivationGeneration::INITIAL,
        GameTick::ZERO,
        0,
        [ChunkPos::new(0, 0)],
        config(projection_capacity),
    )
    .unwrap();
    CompositeRegionRouter::new(local, [composite]).unwrap()
}

fn join(player: u128, session: u64, sequence: u64) -> ferrite_simulation::command::RegionCommand {
    let player = StableEntityId::new(player).unwrap();
    SessionJoinPayload {
        session: SessionId::new(session).unwrap(),
        player,
        identity: SessionIdentity {
            profile_id: player.get(),
            name: format!("Replay{session}"),
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
        spawn_pose: PlayerPose::new(Vec3::new(session as f64, 65.0, 0.5), Rotation::default()),
    }
    .into_region_command(key(), GameTick::new(1), sequence)
    .unwrap()
}

#[test]
fn ingress_order_converges_through_composite_continuity_and_session_projection() {
    let first = join(1, 1, 1);
    let second = join(2, 2, 1);
    let mut left = router(8);
    let mut right = router(8);
    RegionCommandRouter::route(&mut left, second.clone()).unwrap();
    RegionCommandRouter::route(&mut left, first.clone()).unwrap();
    RegionCommandRouter::route(&mut right, first).unwrap();
    RegionCommandRouter::route(&mut right, second).unwrap();

    let left_report = left.run_tick(GameTick::new(1)).unwrap();
    let right_report = right.run_tick(GameTick::new(1)).unwrap();
    let left_evidence = ProductionTickReplayEvidence::capture(&left_report).unwrap();
    let right_evidence = ProductionTickReplayEvidence::capture(&right_report).unwrap();

    assert_eq!(left_evidence, right_evidence);
    assert_ne!(left_evidence.ingress_digest, [0; 32]);
    assert_ne!(left_evidence.projection_digest, [0; 32]);
    assert_ne!(left_evidence.digest, [0; 32]);
    assert!(left_evidence.regions[0].continuity_record_count >= 2);

    let projections = left_report
        .region(&key())
        .unwrap()
        .projections
        .iter()
        .map(decode_projection)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let owner = StableEntityId::new(1).unwrap();
    let mut queue = SessionProjectionQueue::new(2).unwrap();
    assert_eq!(queue.admit(owner, &key(), &projections).unwrap(), 1);
    let projected = queue
        .project(
            2,
            &ferrite_server_runtime::chunk::projection::JavaTerrainRegistryMap::new(
                1,
                BlockStateId::new(0),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        projected.deferred[0].kind,
        DeferredProjectionKind::PlayerService
    );
}

#[test]
fn projection_backpressure_fails_before_composite_commit_and_poisons_retry() {
    let mut runtime = router(1);
    RegionCommandRouter::route(&mut runtime, join(1, 1, 1)).unwrap();
    RegionCommandRouter::route(&mut runtime, join(2, 2, 1)).unwrap();

    assert!(matches!(
        runtime.run_tick(GameTick::new(1)),
        Err(CompositeGatewayError::Composite { source, .. })
            if matches!(*source, CompositeServiceRuntimeError::ProjectionBackpressure { .. })
    ));
    assert!(runtime.last_commit(&key()).is_none());
    assert!(matches!(
        runtime.run_tick(GameTick::new(1)),
        Err(CompositeGatewayError::Local(LocalRunnerError::Poisoned))
    ));
}
