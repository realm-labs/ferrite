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
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_server_runtime::composite::gateway::CompositeRegionRouter;
use ferrite_server_runtime::composite::runtime::CompositeRuntimeConfig;
use ferrite_server_runtime::composite::services::{
    CompositeProductionRegionRuntime, CompositeProductionRuntimeConfig,
};
use ferrite_server_runtime::entity_service::runtime::EntityServiceRuntimeLimits;
use ferrite_server_runtime::session::command::{SessionJoinPayload, SessionLeavePayload};
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

fn composite_config() -> CompositeProductionRuntimeConfig {
    CompositeProductionRuntimeConfig {
        coordinator: CompositeRuntimeConfig {
            command_capacity: 32,
            event_capacity: 32,
            projection_capacity: 32,
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
            gameplay_random_seed: 3,
        },
        entities: EntityServiceRuntimeLimits::new(8, 8, 8, 8),
        world: WorldServiceRuntimeConfig {
            mapping: RegionMapping::V1,
            layout: layout(),
            region_side_chunks: 8,
            chunk_capacity: 8,
            event_capacity: 8,
            content_manifest: [3; 32],
        },
        player_capacity: 8,
        projection_capacity_per_player: 8,
    }
}

fn router() -> CompositeRegionRouter {
    let region = key();
    let voxels = RegionVoxelState::new(region.clone(), RegionMapping::V1, layout()).unwrap();
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
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
        composite_config(),
    )
    .unwrap();
    CompositeRegionRouter::new(runner, [composite]).unwrap()
}

fn join(player: StableEntityId) -> SessionJoinPayload {
    SessionJoinPayload {
        session: SessionId::new(1).unwrap(),
        player,
        identity: SessionIdentity {
            profile_id: player.get(),
            name: "CompositeWalker".to_owned(),
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
        spawn_pose: PlayerPose::new(
            Vec3::new(0.5, 65.0, 0.5),
            Rotation {
                yaw: 0.0,
                pitch: 0.0,
            },
        ),
    }
}

#[test]
fn formal_route_commits_session_join_through_composite_authority() {
    let player = StableEntityId::new(7).unwrap();
    let mut router = router();
    RegionCommandRouter::route(
        &mut router,
        join(player)
            .into_region_command(key(), GameTick::new(1), 0)
            .unwrap(),
    )
    .unwrap();

    let report = router.run_tick(GameTick::new(1)).unwrap();

    assert_eq!(report.local().tick(), GameTick::new(1));
    assert_eq!(report.regions().count(), 1);
    assert_eq!(report.region(&key()).unwrap().commit.tick, GameTick::new(1));
    assert!(router.player_is_owned(&key(), player));
    assert_eq!(router.last_commit(&key()).unwrap().tick, GameTick::new(1));
}

#[test]
fn formal_route_removes_disconnected_player_from_composite_authority() {
    let player = StableEntityId::new(8).unwrap();
    let mut router = router();
    RegionCommandRouter::route(
        &mut router,
        join(player)
            .into_region_command(key(), GameTick::new(1), 0)
            .unwrap(),
    )
    .unwrap();
    router.run_tick(GameTick::new(1)).unwrap();
    RegionCommandRouter::route(
        &mut router,
        SessionLeavePayload {
            session: SessionId::new(1).unwrap(),
            player,
        }
        .into_region_command(key(), GameTick::new(2), 1)
        .unwrap(),
    )
    .unwrap();

    let report = router.run_tick(GameTick::new(2)).unwrap();

    assert_eq!(report.region(&key()).unwrap().commit.tick, GameTick::new(2));
    assert!(!router.player_is_owned(&key(), player));
}

#[test]
fn formal_route_commits_every_region_even_without_commands() {
    let mut router = router();

    let report = router.run_tick(GameTick::new(1)).unwrap();

    let region = report.region(&key()).unwrap();
    assert_eq!(region.commit.tick, GameTick::new(1));
    assert_eq!(region.events.len(), 9);
    assert!(region.projections.is_empty());
}
