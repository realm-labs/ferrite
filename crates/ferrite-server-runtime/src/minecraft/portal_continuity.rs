use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::path::PathBuf;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::{DimensionId, StableEntityId};
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::state::{PlayerPose, Rotation, Vec3};
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, MainHand, ParticleStatus, PlayAdmission, PlayerSpawn,
    SessionId, SessionIdentity, VirtualHost,
};
use ferrite_region_runtime::local::LocalTickReport;
use ferrite_world::generation::border::state::WorldBorder;
use ferrite_world::id::{AIR, END_PORTAL, OBSIDIAN};
use ferrite_world::projection::ChunkSnapshot;
use tempfile::TempDir;

use crate::chunk::session::ChunkSessionLimits;
use crate::composite::gateway::{CompositeGatewayTickReport, CompositeRegionRouter};
use crate::config::ServerConfig;
use crate::minecraft::portal::PortalSessionState;
use crate::minecraft::{settings, world};
use crate::player::connection::JavaPlayerConnection;
use crate::player::session::PlayerSessionAction;
use crate::session::command::SessionJoinPayload;
use crate::session::router::RegionCommandRouter;
use crate::world_service::metadata::region_store_root;
use crate::world_service::runtime::WorldBlockWrite;

const OVERWORLD: &str = "minecraft:overworld";
const NETHER: &str = "minecraft:the_nether";
const END: &str = "minecraft:the_end";

struct PortalContinuityHarness {
    temporary: TempDir,
    config_text: String,
    runtime: world::WorldBootstrap,
    player: JavaPlayerConnection,
    portal: PortalSessionState,
    borders: BTreeMap<DimensionId, WorldBorder>,
    tick: ferrite_simulation::tick::GameTick,
    source_region: SimulationRegionKey,
    player_id: StableEntityId,
    end: DimensionId,
}

impl PortalContinuityHarness {
    fn new() -> Self {
        let temporary = TempDir::new().unwrap();
        let mut config = ServerConfig::development_node(1, 1, 30_000, temporary.path()).unwrap();
        config.world.view_distance = 2;
        config.world.simulation_distance = 2;
        config.world.dimensions = vec![OVERWORLD.to_owned(), NETHER.to_owned(), END.to_owned()];
        let config_text = config.to_toml().unwrap();
        let validated = ServerConfig::from_toml(&config_text).unwrap();
        let mut runtime = world::load(&validated).unwrap();
        let route = runtime
            .routes
            .resolve(&VirtualHost {
                host: "localhost".to_owned(),
                port: 25_565,
            })
            .clone();
        let source = runtime.respawn;
        let source_region =
            route
                .mapping
                .region_for_chunk(route.world, route.dimension.clone(), source.chunk());
        let player_id = StableEntityId::new(41).unwrap();
        let admission = PlayAdmission {
            session: SessionId::new(41).unwrap(),
            identity: SessionIdentity {
                profile_id: 41,
                name: "EndTraveler".to_owned(),
            },
            player: player_id,
            region: source_region.clone(),
            region_mapping: RegionMapping::V1,
            spawn_chunk: source.chunk(),
            spawn: PlayerSpawn {
                x: f64::from(source.x) + 0.5,
                y: f64::from(source.y),
                z: f64::from(source.z) + 0.5,
                yaw: 0.0,
                pitch: 0.0,
            },
            requested_view_distance: 2,
            transferred: false,
        };
        let tick = runtime.committed_tick.checked_next().unwrap();
        runtime
            .router
            .admit_world_blocks(
                &source_region,
                tick,
                vec![WorldBlockWrite {
                    position: source,
                    state: END_PORTAL,
                }],
            )
            .unwrap();
        RegionCommandRouter::route(
            &mut runtime.router,
            SessionJoinPayload {
                session: admission.session,
                player: admission.player,
                identity: admission.identity.clone(),
                settings: client_settings(),
                transferred: false,
                spawn_pose: PlayerPose::new(
                    Vec3::new(admission.spawn.x, admission.spawn.y, admission.spawn.z),
                    Rotation::default(),
                ),
            }
            .into_region_command(source_region.clone(), tick, 0)
            .unwrap(),
        )
        .unwrap();
        let baseline = runtime.router.run_tick(tick).unwrap();
        checkpoint(&mut runtime, &baseline);

        let protocol = settings::load(None, &runtime.dimensions).unwrap();
        let player = JavaPlayerConnection::new(
            admission,
            protocol.registries,
            2,
            2,
            ChunkSessionLimits {
                maximum_tracked_chunks: 25,
                maximum_tickets: 26,
                maximum_chunks_per_batch: 4,
            },
        )
        .unwrap();
        let borders = runtime
            .dimensions
            .iter()
            .map(|dimension| {
                (
                    dimension.clone(),
                    runtime.lifecycle.level(dimension).unwrap().border.clone(),
                )
            })
            .collect();
        Self {
            temporary,
            config_text,
            runtime,
            player,
            portal: PortalSessionState::default(),
            borders,
            tick,
            source_region,
            player_id,
            end: DimensionId::new(ResourceId::minecraft("the_end").unwrap()),
        }
    }

    fn control_file_lengths(&self) -> Vec<(PathBuf, u64)> {
        let control = &self
            .runtime
            .lifecycle
            .level(self.source_region.dimension())
            .unwrap()
            .control_region;
        let root = region_store_root(
            &ServerConfig::from_toml(&self.config_text)
                .unwrap()
                .config()
                .storage
                .root,
            control,
        )
        .unwrap();
        ["region-data.log", "region-index.log", "region-journal.log"]
            .into_iter()
            .map(|name| {
                let path = root.join(name);
                let length = path.metadata().unwrap().len();
                (path, length)
            })
            .collect()
    }

    fn travel_to_end(&mut self) -> CompositeGatewayTickReport {
        let snapshots = projectable_snapshots(&self.runtime.router, &self.runtime.dimensions);
        self.portal
            .observe_contact(
                Some(&self.player),
                &snapshots,
                &self.runtime.dimensions,
                &self.borders,
                self.runtime.respawn,
            )
            .unwrap();
        let tickets = self
            .portal
            .tickets(Some(&self.player), &self.runtime.router)
            .into_iter()
            .filter_map(|(dimension, ticket)| (dimension == self.end).then_some(ticket))
            .collect::<Vec<_>>();
        assert!(!tickets.is_empty());
        for _ in 0..32 {
            self.tick = self.tick.checked_next().unwrap();
            self.runtime
                .chunk_lifecycles
                .get_mut(&self.end)
                .unwrap()
                .drive(self.tick, tickets.clone(), &mut self.runtime.router)
                .unwrap();
            let snapshots = projectable_snapshots(&self.runtime.router, &self.runtime.dimensions);
            self.portal
                .stage_ready(
                    Some(&mut self.player),
                    self.tick,
                    &snapshots,
                    &self.borders,
                    self.runtime.respawn,
                    &mut self.runtime.router,
                )
                .unwrap();
            let report = self.runtime.router.run_tick(self.tick).unwrap();
            if self.player.player().transfer_pending()
                && committed_dimension_transfer(&mut self.player, report.local())
            {
                return report;
            }
        }
        panic!("End portal did not commit within the bounded lifecycle window")
    }

    fn checkpoint(&mut self, report: &CompositeGatewayTickReport) {
        checkpoint(&mut self.runtime, report);
    }

    fn truncate_control_to(&self, lengths: &[(PathBuf, u64)]) {
        for (path, length) in lengths {
            let file = OpenOptions::new().write(true).open(path).unwrap();
            file.set_len(*length).unwrap();
            file.sync_all().unwrap();
        }
    }

    fn restart(self) -> (TempDir, world::WorldBootstrap) {
        let Self {
            temporary,
            config_text,
            runtime,
            ..
        } = self;
        drop(runtime);
        let validated = ServerConfig::from_toml(&config_text).unwrap();
        (temporary, world::load(&validated).unwrap())
    }
}

#[test]
fn cross_region_end_platform_and_player_transfer_survive_one_published_checkpoint() {
    let mut harness = PortalContinuityHarness::new();
    let report = harness.travel_to_end();
    harness.checkpoint(&report);
    let target = RegionMapping::V1.region_for_chunk(
        harness.source_region.world(),
        harness.end.clone(),
        ChunkPos::new(6, 0),
    );
    let player = harness.player_id;
    let end = harness.end.clone();
    let (_temporary, restarted) = harness.restart();

    assert!(restarted.router.player_is_owned(&target, player));
    for x in 98..=102 {
        for z in -2..=2 {
            assert_eq!(
                restarted
                    .router
                    .world_block_state(&end, BlockPos::new(x, 48, z))
                    .unwrap(),
                Some(OBSIDIAN)
            );
            assert_eq!(
                restarted
                    .router
                    .world_block_state(&end, BlockPos::new(x, 49, z))
                    .unwrap(),
                Some(AIR)
            );
        }
    }
}

#[test]
fn unpublished_cross_region_portal_successor_rolls_back_to_the_control_checkpoint() {
    let mut harness = PortalContinuityHarness::new();
    let baseline_lengths = harness.control_file_lengths();
    let report = harness.travel_to_end();
    harness.checkpoint(&report);
    harness.truncate_control_to(&baseline_lengths);
    let source = harness.source_region.clone();
    let player = harness.player_id;
    let end = harness.end.clone();
    let (_temporary, restarted) = harness.restart();

    assert!(restarted.router.player_is_owned(&source, player));
    assert_ne!(
        restarted
            .router
            .world_block_state(&end, BlockPos::new(100, 48, 0))
            .unwrap(),
        Some(OBSIDIAN)
    );
}

fn checkpoint(runtime: &mut world::WorldBootstrap, report: &CompositeGatewayTickReport) {
    let generations = report
        .regions()
        .map(|(key, _)| {
            (
                key.clone(),
                runtime.router.activation_generation(key).unwrap(),
            )
        })
        .collect();
    runtime.persistence.capture(report, &generations).unwrap();
    for committed in runtime.persistence.flush().unwrap() {
        runtime
            .router
            .apply_world_save_receipt(committed.region(), committed.point(), committed.receipt())
            .unwrap();
    }
}

fn projectable_snapshots(
    router: &CompositeRegionRouter,
    dimensions: &[DimensionId],
) -> BTreeMap<DimensionId, BTreeMap<ChunkPos, ChunkSnapshot>> {
    dimensions
        .iter()
        .map(|dimension| {
            let positions = router.projectable_world_positions(dimension).unwrap();
            let snapshots = router
                .projectable_world_snapshots(dimension, positions)
                .unwrap();
            (dimension.clone(), snapshots)
        })
        .collect()
}

fn committed_dimension_transfer(
    player: &mut JavaPlayerConnection,
    report: &LocalTickReport,
) -> bool {
    player.observe_committed_tick(report).unwrap().player
        == PlayerSessionAction::DimensionTransferCommitted
}

fn client_settings() -> ClientSettings {
    ClientSettings {
        language: "en_us".to_owned(),
        view_distance: 2,
        chat_visibility: ChatVisibility::Full,
        chat_colors: true,
        model_customization: u8::MAX,
        main_hand: MainHand::Right,
        text_filtering: false,
        allows_listing: true,
        particle_status: ParticleStatus::All,
    }
}
