use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::direction::Direction;
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
use ferrite_server_runtime::composite::gateway::{CompositeGatewayError, CompositeRegionRouter};
use ferrite_server_runtime::composite::projection::{SessionProjectionAction, decode_projection};
use ferrite_server_runtime::composite::runtime::CompositeRuntimeConfig;
use ferrite_server_runtime::composite::services::{
    CompositeProductionRegionRuntime, CompositeProductionRuntimeConfig, CompositeServiceAction,
    CompositeServiceCommand,
};
use ferrite_server_runtime::entity_service::runtime::EntityServiceRuntimeLimits;
use ferrite_server_runtime::player::block::authority::AuthoritativeBlockError;
use ferrite_server_runtime::player::block::command::{BlockIntent, BlockInteractionCommand};
use ferrite_server_runtime::player::block::replication::{
    BlockCommandOutcome, project_committed_blocks,
};
use ferrite_server_runtime::session::command::{SessionJoinPayload, SessionLeavePayload};
use ferrite_server_runtime::session::router::RegionCommandRouter;
use ferrite_server_runtime::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
use ferrite_server_runtime::simulation::runtime::SimulationRuntimeConfig;
use ferrite_server_runtime::world_service::model::WorldServiceRuntimeConfig;
use ferrite_simulation::command::{CommandSource, RegionCommand};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, ChunkRevision, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;

fn key() -> SimulationRegionKey {
    key_at(0)
}

fn key_at(region_x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(region_x, 0),
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

fn two_region_router(
    initial_blocks: &[(SimulationRegionKey, BlockPos, BlockStateId)],
) -> CompositeRegionRouter {
    let regions = [
        (key_at(0), ChunkPos::new(7, 0)),
        (key_at(1), ChunkPos::new(8, 0)),
    ];
    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    let mut runtimes = Vec::new();
    for (region, chunk) in regions {
        let voxels = RegionVoxelState::new(region.clone(), RegionMapping::V1, layout()).unwrap();
        runner
            .insert_region(
                RegionSimulationState::new(voxels),
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
            )
            .unwrap();
        let mut runtime = CompositeProductionRegionRuntime::new(
            region.clone(),
            ActivationGeneration::INITIAL,
            GameTick::ZERO,
            0,
            [chunk],
            composite_config(),
        )
        .unwrap();
        for (_, position, state) in initial_blocks
            .iter()
            .filter(|(owner, _, _)| owner == &region)
        {
            runtime
                .admit_command(CompositeServiceCommand::new(
                    GameTick::new(1),
                    1,
                    CompositeServiceAction::SetWorldBlock {
                        expected_revision: ChunkRevision::INITIAL,
                        position: *position,
                        state: *state,
                    },
                ))
                .unwrap();
        }
        runtimes.push(runtime);
    }
    CompositeRegionRouter::new(runner, runtimes).unwrap()
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

#[test]
fn unavailable_authoritative_block_rejects_without_terminating_gateway() {
    let player = StableEntityId::new(9).unwrap();
    let mut router = router();
    let position = BlockPos::new(16, 65, 0);
    RegionCommandRouter::route(
        &mut router,
        BlockInteractionCommand {
            player,
            intent: BlockIntent::StartDestroy { position },
            eye: Vec3::new(16.5, 66.62, 0.5),
            interaction_range: 4.5,
        }
        .into_region_command(key(), GameTick::new(1), 10)
        .unwrap(),
    )
    .unwrap();

    let report = router.run_tick(GameTick::new(1)).unwrap();
    let projection = project_committed_blocks(report.local(), player, None).unwrap();

    assert_eq!(projection.results.len(), 1);
    assert_eq!(projection.results[0].outcome, BlockCommandOutcome::Rejected);
    assert!(projection.results[0].corrections.is_empty());
    assert!(router.run_tick(GameTick::new(2)).is_ok());
}

#[test]
fn destroy_across_player_region_uses_authoritative_target_region() {
    let player = StableEntityId::new(10).unwrap();
    let target = BlockPos::new(128, 65, 8);
    let mut router = two_region_router(&[(key_at(1), target, BlockStateId::new(1))]);
    RegionCommandRouter::route(
        &mut router,
        join(player)
            .into_region_command(key_at(0), GameTick::new(1), 20)
            .unwrap(),
    )
    .unwrap();
    router.run_tick(GameTick::new(1)).unwrap();

    for (tick, sequence, intent) in [
        (
            GameTick::new(2),
            21,
            BlockIntent::StartDestroy { position: target },
        ),
        (
            GameTick::new(3),
            22,
            BlockIntent::StopDestroy { position: target },
        ),
    ] {
        RegionCommandRouter::route(
            &mut router,
            BlockInteractionCommand {
                player,
                intent,
                eye: Vec3::new(126.5, 66.62, 8.5),
                interaction_range: 4.5,
            }
            .into_region_command(key_at(1), tick, sequence)
            .unwrap(),
        )
        .unwrap();
        let report = router.run_tick(tick).unwrap();
        if tick == GameTick::new(3) {
            assert!(
                report
                    .region(&key_at(1))
                    .unwrap()
                    .projections
                    .iter()
                    .map(decode_projection)
                    .any(|projection| matches!(
                        projection.unwrap().action(),
                        SessionProjectionAction::Block(update)
                            if update.position == target && update.state == BlockStateId::new(0)
                    ))
            );
        }
    }
    assert!(router.player_is_owned(&key_at(0), player));
    assert!(router.run_tick(GameTick::new(4)).is_ok());
}

#[test]
fn placement_crossing_region_boundary_commits_in_adjacent_authority() {
    let player = StableEntityId::new(11).unwrap();
    let hit = BlockPos::new(127, 65, 8);
    let placed = BlockPos::new(128, 65, 8);
    let mut router = two_region_router(&[(key_at(0), hit, BlockStateId::new(1))]);
    router.run_tick(GameTick::new(1)).unwrap();
    RegionCommandRouter::route(
        &mut router,
        BlockInteractionCommand {
            player,
            intent: BlockIntent::UseOn {
                position: hit,
                direction: Direction::East,
                offset_x: 1.0,
                offset_y: 0.5,
                offset_z: 0.5,
                inside: false,
                world_border_hit: false,
                interaction_allowed: true,
                placement_state: BlockStateId::new(2),
            },
            eye: Vec3::new(126.5, 66.62, 8.5),
            interaction_range: 4.5,
        }
        .into_region_command(key_at(0), GameTick::new(2), 30)
        .unwrap(),
    )
    .unwrap();

    let report = router.run_tick(GameTick::new(2)).unwrap();

    assert!(
        report
            .region(&key_at(1))
            .unwrap()
            .projections
            .iter()
            .map(decode_projection)
            .any(|projection| matches!(
                projection.unwrap().action(),
                SessionProjectionAction::Block(update)
                    if update.position == placed && update.state == BlockStateId::new(2)
            ))
    );
    assert!(router.run_tick(GameTick::new(3)).is_ok());
}

#[test]
fn malformed_internal_block_command_preserves_detailed_gateway_error() {
    let player = StableEntityId::new(12).unwrap();
    let mut router = router();
    RegionCommandRouter::route(
        &mut router,
        RegionCommand::new(
            key(),
            GameTick::new(1),
            CommandSource::Player(player),
            40,
            ResourceId::new("ferrite", "player/block_interaction").unwrap(),
            b"invalid-block-command".to_vec(),
        )
        .unwrap(),
    )
    .unwrap();

    assert!(matches!(
        router.run_tick(GameTick::new(1)),
        Err(CompositeGatewayError::BlockInteraction { source, .. })
            if matches!(*source, AuthoritativeBlockError::Command(_))
    ));
}
