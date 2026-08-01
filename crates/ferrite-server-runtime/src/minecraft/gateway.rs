use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_gameplay::player::movement::MovementContext;
use ferrite_persistence::snapshot::SnapshotRecord;
use ferrite_protocol::java_26_2::connection::driver::ServerConnection;
use ferrite_protocol::java_26_2::connection::output::{
    OutboundFrame, PlayDisconnectReason, ServerConnectionEvent, ServerConnectionStage,
};
use ferrite_protocol::java_26_2::connection::settings::ServerConnectionSettings;
use ferrite_protocol::java_26_2::login::serverbound::session::AdmissionSnapshot;
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::semantic::{PlayerSpawn, SessionEgress, SessionId};
use ferrite_simulation::tick::GameTick;
use ferrite_world::generation::border::state::WorldBorder;
use ferrite_world::projection::ChunkSnapshot;
use thiserror::Error;

use crate::chunk::projection::JavaTerrainRegistryMap;
use crate::composite::gateway::{CompositeGatewayTickReport, CompositeRegionRouter};
use crate::composite::projection::{SessionProjection, SessionProjectionQueue, decode_projection};
use crate::config::ValidatedServerConfig;
use crate::lifecycle::{NodeLifecycle, NodePhase};
use crate::minecraft::collision::AuthoritativePlayerCollision;
use crate::minecraft::entry;
use crate::minecraft::settings;
use crate::minecraft::world;
use crate::player::block::replication::BlockCommandOutcome;
use crate::player::connection::{JavaPlayerConnection, PlayerDispatchContext};
use crate::player::dispatch::{ServerboundDispatchOutcome, ServerboundDisposition};
use crate::player::session::PlayerSessionAction;
use crate::runtime_status::{
    BlockResultStatus, MinecraftRuntimeStatus, MinecraftSessionStatus, ServerboundDispatchStatus,
};
use crate::session::admission::AllowAll;
use crate::session::bridge::SessionBridge;
use crate::world_service::environment::{EnvironmentProjection, LevelEnvironment};
use crate::world_service::formal_lifecycle::FormalChunkLifecycle;
use crate::world_service::formal_persistence::FormalWorldPersistence;
use crate::world_service::lifecycle::WorldLifecycleRuntime;
use crate::world_service::spawn::resolve_respawn;

type DynError = Box<dyn Error + Send + Sync>;

const SERVER_TICK: Duration = Duration::from_millis(50);
const MAX_ACCEPTS_PER_POLL: usize = 64;
const MAX_READS_PER_POLL: usize = 64;
const MAX_DRIVE_PASSES: usize = 8;
const MAX_TICK_CATCH_UP: usize = 4;
const READ_BUFFER_BYTES: usize = 64 * 1024;
const CHUNK_BATCH_SIZE: usize = 4;
const SESSION_PROJECTION_BATCH_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GatewayPoll {
    pub(crate) active_sessions: usize,
    pub(crate) committed_tick: GameTick,
}

pub(crate) struct MinecraftGateway {
    listener: Option<TcpListener>,
    local_address: SocketAddr,
    lifecycle: Arc<NodeLifecycle>,
    bridge: SessionBridge<CompositeRegionRouter>,
    settings: ServerConnectionSettings,
    registries: PlayRegistries,
    terrain_registries: JavaTerrainRegistryMap,
    chunk_lifecycles: BTreeMap<ferrite_foundation::identity::DimensionId, FormalChunkLifecycle>,
    persistence: FormalWorldPersistence,
    world_lifecycle: WorldLifecycleRuntime,
    world_metadata_record: SnapshotRecord,
    sessions: BTreeMap<SessionId, NetworkSession>,
    maximum_sessions: usize,
    view_distance: u16,
    simulation_distance: u16,
    world_spawn: BlockPos,
    respawn_position: BlockPos,
    dimensions: Vec<ferrite_foundation::identity::DimensionId>,
    projection_capacity: usize,
    next_session: u64,
    committed_tick: GameTick,
    next_tick: Instant,
    draining: bool,
    authorities_released: bool,
    shutdown_capture_pending: bool,
    last_session_error: Option<String>,
    last_dispatch: Option<ServerboundDispatchOutcome>,
    composite_region_commits: usize,
}

impl MinecraftGateway {
    pub(crate) fn bind(
        config: &ValidatedServerConfig,
        lifecycle: Arc<NodeLifecycle>,
    ) -> Result<Self, MinecraftGatewayError> {
        Self::bind_inner(config, lifecycle).map_err(MinecraftGatewayError::new)
    }

    fn bind_inner(
        config: &ValidatedServerConfig,
        lifecycle: Arc<NodeLifecycle>,
    ) -> Result<Self, DynError> {
        let minecraft = &config.config().minecraft;
        let listener = TcpListener::bind(minecraft.bind)?;
        listener.set_nonblocking(true)?;
        let local_address = listener.local_addr()?;
        let world = world::load(config)?;
        let world::WorldBootstrap {
            routes,
            router,
            chunk_lifecycles,
            persistence,
            lifecycle: world_lifecycle,
            metadata_record: world_metadata_record,
            committed_tick,
            view_distance,
            simulation_distance,
            world_spawn,
            respawn,
            dimensions,
        } = world;
        let protocol = settings::load(minecraft.registry_report.as_deref(), &dimensions)?;
        let region_authorities = router.len();
        let bridge = SessionBridge::new(
            routes,
            Arc::clone(&lifecycle),
            config.config().limits.max_sessions,
            router,
        )?;
        let gateway = Self {
            listener: Some(listener),
            local_address,
            lifecycle: Arc::clone(&lifecycle),
            bridge,
            settings: protocol.settings,
            registries: protocol.registries,
            terrain_registries: protocol.terrain_registries,
            chunk_lifecycles,
            persistence,
            world_lifecycle,
            world_metadata_record,
            sessions: BTreeMap::new(),
            maximum_sessions: config.config().limits.max_sessions,
            view_distance,
            simulation_distance,
            world_spawn,
            respawn_position: respawn,
            dimensions,
            projection_capacity: config.config().limits.max_region_mailbox,
            next_session: 1,
            committed_tick,
            next_tick: Instant::now() + SERVER_TICK,
            draining: false,
            authorities_released: false,
            shutdown_capture_pending: false,
            last_session_error: None,
            last_dispatch: None,
            composite_region_commits: 0,
        };
        lifecycle.set_active_region_authorities(region_authorities)?;
        Ok(gateway)
    }

    pub(crate) const fn local_address(&self) -> SocketAddr {
        self.local_address
    }

    pub(crate) fn last_session_error(&self) -> Option<&str> {
        self.last_session_error.as_deref()
    }

    pub(crate) const fn last_dispatch(&self) -> Option<ServerboundDispatchOutcome> {
        self.last_dispatch
    }

    pub(crate) const fn committed_tick(&self) -> GameTick {
        self.committed_tick
    }

    pub(crate) fn runtime_status(&self) -> MinecraftRuntimeStatus {
        MinecraftRuntimeStatus {
            committed_tick: self.committed_tick.get(),
            composite_region_commits: self.composite_region_commits,
            sessions: self.sessions.values().map(NetworkSession::status).collect(),
        }
    }

    pub(crate) fn poll(&mut self, phase: NodePhase) -> Result<GatewayPoll, MinecraftGatewayError> {
        self.poll_inner(phase).map_err(MinecraftGatewayError::new)
    }

    fn poll_inner(&mut self, phase: NodePhase) -> Result<GatewayPoll, DynError> {
        if matches!(phase, NodePhase::Draining | NodePhase::Drained) {
            self.begin_drain()?;
        }
        if phase == NodePhase::Ready && !self.draining {
            self.accept_connections()?;
        }
        self.poll_sessions()?;
        self.run_due_ticks()?;
        self.poll_sessions()?;
        if self.draining
            && self.sessions.is_empty()
            && self.shutdown_capture_pending
            && !self.authorities_released
        {
            self.run_one_tick()?;
        }
        if self.draining && self.sessions.is_empty() && !self.authorities_released {
            self.flush_persistence()?;
            let close_results = self
                .world_lifecycle
                .dimensions()
                .iter()
                .cloned()
                .map(|dimension| (dimension, true))
                .collect();
            self.world_lifecycle.finish_shutdown(&close_results)?;
            self.lifecycle.set_active_region_authorities(0)?;
            self.authorities_released = true;
        }
        Ok(GatewayPoll {
            active_sessions: self.sessions.len(),
            committed_tick: self.committed_tick,
        })
    }

    fn begin_drain(&mut self) -> Result<(), DynError> {
        if self.draining {
            return Ok(());
        }
        self.draining = true;
        self.listener.take();
        self.world_lifecycle.begin_shutdown(self.sessions.len())?;
        self.refresh_world_auxiliary()?;
        self.shutdown_capture_pending = true;
        for session in self.sessions.values_mut() {
            session.begin_drain(&self.registries);
        }
        Ok(())
    }

    fn accept_connections(&mut self) -> Result<(), DynError> {
        let Some(listener) = &self.listener else {
            return Ok(());
        };
        for _ in 0..MAX_ACCEPTS_PER_POLL {
            let (stream, peer) = match listener.accept() {
                Ok(connection) => connection,
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(error) => return Err(error.into()),
            };
            stream.set_nonblocking(true)?;
            stream.set_nodelay(true)?;
            let session = SessionId::new(self.next_session)?;
            self.next_session = self
                .next_session
                .checked_add(1)
                .ok_or("Minecraft session identity exhausted")?;
            if self.bridge.register(session, peer).is_err() {
                let _ = stream.shutdown(Shutdown::Both);
                continue;
            }
            self.sessions.insert(
                session,
                NetworkSession::new(
                    session,
                    stream,
                    self.settings.clone(),
                    self.projection_capacity,
                )?,
            );
        }
        Ok(())
    }

    fn poll_sessions(&mut self) -> Result<(), DynError> {
        let ids = self.sessions.keys().copied().collect::<Vec<_>>();
        let target_tick = self
            .committed_tick
            .checked_next()
            .unwrap_or(self.committed_tick);
        let now_millis = unix_millis();
        let environment = self.overworld_environment()?;
        let border = self.overworld_border()?;
        let mut disconnect = Vec::new();
        for id in ids {
            let Some(mut session) = self.sessions.remove(&id) else {
                continue;
            };
            let result = if session.terminal {
                Ok(Vec::new())
            } else {
                session.poll(SessionContext {
                    now_millis,
                    target_tick,
                    maximum_sessions: self.maximum_sessions,
                    bridge: &mut self.bridge,
                    registries: &self.registries,
                    terrain_registries: &self.terrain_registries,
                    environment,
                    border: border.clone(),
                    view_distance: self.view_distance,
                    simulation_distance: self.simulation_distance,
                    respawn_position: self.respawn_position,
                    world_spawn: self.world_spawn,
                    dimensions: &self.dimensions,
                })
            };
            match result {
                Ok(actions) => disconnect.extend(actions),
                Err(error) => {
                    self.last_session_error = Some(format!("session {}: {error}", id.get()));
                    session.terminate();
                }
            }
            if let Some(outcome) = session.last_dispatch() {
                self.last_dispatch = Some(outcome);
            }
            if session.terminal {
                let current_region = session
                    .player
                    .as_ref()
                    .map(|player| player.player().region().clone());
                if !self.close_registered_session(id, current_region.as_ref()) {
                    self.sessions.insert(id, session);
                }
            } else {
                self.sessions.insert(id, session);
            }
        }
        for id in disconnect {
            if let Some(session) = self.sessions.get_mut(&id) {
                session.begin_drain(&self.registries);
            }
        }
        Ok(())
    }

    fn close_registered_session(
        &mut self,
        id: SessionId,
        current_region: Option<&ferrite_foundation::region::SimulationRegionKey>,
    ) -> bool {
        let target_tick = self
            .committed_tick
            .checked_next()
            .unwrap_or(self.committed_tick);
        let result = match current_region {
            Some(region) => self.bridge.unregister_from_region(id, target_tick, region),
            None => self.bridge.unregister(id, target_tick),
        };
        match result {
            Ok(()) => true,
            Err(error) => {
                self.last_session_error =
                    Some(format!("session {} unregister failed: {error}", id.get()));
                false
            }
        }
    }

    fn run_due_ticks(&mut self) -> Result<(), DynError> {
        let now = Instant::now();
        let mut caught_up = 0;
        while now >= self.next_tick && caught_up < MAX_TICK_CATCH_UP {
            self.run_one_tick()?;
            self.next_tick += SERVER_TICK;
            caught_up += 1;
        }
        if caught_up == MAX_TICK_CATCH_UP && now >= self.next_tick {
            self.next_tick = now + SERVER_TICK;
        }
        Ok(())
    }

    fn run_one_tick(&mut self) -> Result<(), DynError> {
        let tick = self.committed_tick.checked_next()?;
        let mut failed = Vec::new();
        for (id, session) in &mut self.sessions {
            if let Err(error) = session.finish_server_tick() {
                failed.push((*id, error.to_string()));
                session.terminate();
            }
        }
        self.record_session_failures(&failed);
        let mut tickets_by_dimension = BTreeMap::<_, Vec<_>>::new();
        for player in self
            .sessions
            .values()
            .filter_map(|session| session.player.as_ref())
        {
            tickets_by_dimension
                .entry(player.player().region().dimension().clone())
                .or_default()
                .extend(player.chunks().tickets().tickets().cloned());
        }
        for (dimension, lifecycle) in &mut self.chunk_lifecycles {
            lifecycle.drive(
                tick,
                tickets_by_dimension.remove(dimension).unwrap_or_default(),
                self.bridge.router_mut(),
            )?;
        }
        let overworld = self.overworld_dimension()?;
        let mut environment = None;
        for dimension in self.dimensions.clone() {
            self.world_lifecycle.tick_border(&dimension)?;
            let projection = self.world_lifecycle.tick_environment(&dimension)?;
            if dimension == overworld {
                environment = Some(projection);
            }
        }
        let environment = environment.ok_or("formal overworld environment did not tick")?;
        self.refresh_world_auxiliary()?;
        let report = self.bridge.router_mut().run_tick(tick)?;
        let generations = report
            .regions()
            .map(|(key, _)| {
                self.bridge
                    .router_mut()
                    .activation_generation(key)
                    .map(|generation| (key.clone(), generation))
                    .ok_or_else(|| format!("formal Region generation disappeared for {key:?}"))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        self.persistence.capture(&report, &generations)?;
        self.shutdown_capture_pending = false;
        if self.persistence.autosave_due(tick) {
            self.flush_persistence()?;
        }
        self.composite_region_commits = report.regions().count();
        self.route_composite_projections(&report)?;
        let requested_snapshots = self
            .sessions
            .values()
            .filter_map(|session| session.player.as_ref())
            .flat_map(|player| player.chunks().stream().interest().view().iter().copied())
            .collect::<BTreeSet<_>>();
        let terrain_snapshots = self
            .bridge
            .router_mut()
            .projectable_world_snapshots(&overworld, requested_snapshots)?;
        let border = self.overworld_border()?;
        if let Some(position) = resolve_respawn(self.world_spawn, &border, &terrain_snapshots) {
            self.respawn_position = position;
        }
        failed.clear();
        for (id, session) in &mut self.sessions {
            if let Err(error) = session.observe_tick(
                &report,
                &terrain_snapshots,
                &self.terrain_registries,
                &self.registries,
                &environment,
            ) {
                failed.push((*id, error.to_string()));
                session.terminate();
            }
            session.begin_server_tick();
        }
        self.record_session_failures(&failed);
        self.committed_tick = tick;
        Ok(())
    }

    fn overworld_dimension(&self) -> Result<ferrite_foundation::identity::DimensionId, DynError> {
        self.world_lifecycle
            .dimensions()
            .first()
            .cloned()
            .ok_or_else(|| "formal world has no overworld level".into())
    }

    fn overworld_environment(&self) -> Result<LevelEnvironment, DynError> {
        let dimension = self.overworld_dimension()?;
        self.world_lifecycle
            .level(&dimension)
            .map(|level| level.environment)
            .ok_or_else(|| "formal overworld has no control state".into())
    }

    fn overworld_border(&self) -> Result<WorldBorder, DynError> {
        let dimension = self.overworld_dimension()?;
        self.world_lifecycle
            .level(&dimension)
            .map(|level| level.border.clone())
            .ok_or_else(|| "formal overworld has no control state".into())
    }

    fn refresh_world_auxiliary(&mut self) -> Result<(), DynError> {
        let overworld = self.overworld_dimension()?;
        for dimension in self.dimensions.clone() {
            let control_region = self
                .world_lifecycle
                .level(&dimension)
                .ok_or_else(|| format!("formal dimension {dimension} has no control state"))?
                .control_region
                .clone();
            let generation = self
                .bridge
                .router_mut()
                .activation_generation(&control_region)
                .ok_or_else(|| format!("formal {dimension} control Region is not active"))?;
            let level_record = self
                .world_lifecycle
                .level_record(&control_region, generation)?;
            let mut records = vec![level_record];
            if dimension == overworld {
                records.insert(0, self.world_metadata_record.clone());
            }
            self.bridge
                .router_mut()
                .replace_world_auxiliary_records(&control_region, records)?;
        }
        Ok(())
    }

    fn flush_persistence(&mut self) -> Result<(), DynError> {
        let pending = self.persistence.pending_commit_count();
        if pending == 0 {
            return Ok(());
        }
        self.lifecycle.set_pending_commits(pending)?;
        let result = self.persistence.flush();
        self.lifecycle.set_pending_commits(0)?;
        for committed in result? {
            self.bridge.router_mut().apply_world_save_receipt(
                committed.region(),
                committed.point(),
                committed.receipt(),
            )?;
        }
        Ok(())
    }

    fn route_composite_projections(
        &mut self,
        report: &CompositeGatewayTickReport,
    ) -> Result<(), DynError> {
        let mut projections = Vec::new();
        for (key, region) in report.regions() {
            for projection in &region.projections {
                projections.push(decode_projection(projection)?.scoped_to_region(key.clone()));
            }
        }
        let mut failed = Vec::new();
        for (id, session) in &mut self.sessions {
            if let Err(error) = session.admit_projections(&projections) {
                failed.push((*id, error.to_string()));
                session.terminate();
            }
        }
        self.record_session_failures(&failed);
        Ok(())
    }

    fn record_session_failures(&mut self, failures: &[(SessionId, String)]) {
        if let Some((id, error)) = failures.last() {
            self.last_session_error = Some(format!("session {}: {error}", id.get()));
        }
    }
}

struct SessionContext<'a> {
    now_millis: i64,
    target_tick: GameTick,
    maximum_sessions: usize,
    bridge: &'a mut SessionBridge<CompositeRegionRouter>,
    registries: &'a PlayRegistries,
    terrain_registries: &'a JavaTerrainRegistryMap,
    environment: LevelEnvironment,
    border: WorldBorder,
    view_distance: u16,
    simulation_distance: u16,
    respawn_position: BlockPos,
    world_spawn: BlockPos,
    dimensions: &'a [ferrite_foundation::identity::DimensionId],
}

struct NetworkSession {
    id: SessionId,
    stream: TcpStream,
    connection: ServerConnection,
    admission: Option<AdmissionSnapshot>,
    player: Option<JavaPlayerConnection>,
    pending_write: Option<PendingWrite>,
    registry_selection_seen: bool,
    spawn_ready_sent: bool,
    drain_started: bool,
    terminal: bool,
    last_dispatch: Option<ServerboundDispatchOutcome>,
    last_unsupported_dispatch: Option<ServerboundDispatchOutcome>,
    last_block_result: Option<BlockResultStatus>,
    region_transfers: u64,
    projections: SessionProjectionQueue,
    deferred_projections: usize,
}

impl NetworkSession {
    fn new(
        id: SessionId,
        stream: TcpStream,
        settings: ServerConnectionSettings,
        projection_capacity: usize,
    ) -> Result<Self, crate::composite::projection::SessionProjectionError> {
        Ok(Self {
            id,
            stream,
            connection: ServerConnection::new(settings),
            admission: None,
            player: None,
            pending_write: None,
            registry_selection_seen: false,
            spawn_ready_sent: false,
            drain_started: false,
            terminal: false,
            last_dispatch: None,
            last_unsupported_dispatch: None,
            last_block_result: None,
            region_transfers: 0,
            projections: SessionProjectionQueue::new(projection_capacity)?,
            deferred_projections: 0,
        })
    }

    fn poll(&mut self, mut context: SessionContext<'_>) -> Result<Vec<SessionId>, DynError> {
        let mut disconnect = Vec::new();
        for _ in 0..MAX_DRIVE_PASSES {
            self.read_available(context.now_millis)?;
            self.prepare_admission(context.bridge)?;
            self.connection.tick(
                self.admission
                    .clone()
                    .unwrap_or_else(AdmissionSnapshot::allowed),
                u128::from(self.id.get()),
                context.now_millis,
                false,
            )?;
            self.drain_events(&mut context, &mut disconnect)?;
            self.flush_available(context.now_millis)?;
            self.drain_events(&mut context, &mut disconnect)?;
            if self.registry_selection_seen
                && !self.spawn_ready_sent
                && self.connection.pending_outbound() == 0
                && self.connection.stage() == ServerConnectionStage::Configuration
            {
                self.connection.spawn_ready()?;
                self.spawn_ready_sent = true;
                continue;
            }
            if self.connection.pending_outbound() == 0 {
                break;
            }
        }
        if matches!(
            self.connection.stage(),
            ServerConnectionStage::Closed | ServerConnectionStage::Faulted
        ) {
            self.terminate();
        }
        Ok(disconnect)
    }

    fn read_available(&mut self, now_millis: i64) -> Result<(), DynError> {
        let mut buffer = [0u8; READ_BUFFER_BYTES];
        for _ in 0..MAX_READS_PER_POLL {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    self.terminal = true;
                    return Ok(());
                }
                Ok(length) => self
                    .connection
                    .receive(&buffer[..length], now_millis, false)?,
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    fn prepare_admission(
        &mut self,
        bridge: &mut SessionBridge<CompositeRegionRouter>,
    ) -> Result<(), DynError> {
        if self.connection.stage() == ServerConnectionStage::Login
            && let Some(profile) = self.connection.profile()
        {
            self.admission = Some(bridge.login_admission(self.id, profile, &mut AllowAll)?);
        }
        Ok(())
    }

    fn drain_events(
        &mut self,
        context: &mut SessionContext<'_>,
        disconnect: &mut Vec<SessionId>,
    ) -> Result<(), DynError> {
        while let Some(event) = self.connection.take_event() {
            match event {
                ServerConnectionEvent::RegistrySelection { .. } => {
                    self.registry_selection_seen = true;
                }
                ServerConnectionEvent::PlayInstallationRequested(request) => {
                    let egress = context.bridge.apply_java_event(
                        self.id,
                        ServerConnectionEvent::PlayInstallationRequested(request.clone()),
                        context.target_tick,
                        &mut AllowAll,
                    )?;
                    let Some(SessionEgress::CompletePlayInstallation(admission)) = egress else {
                        return Err("play installation was not admitted".into());
                    };
                    self.install_play(request.profile, admission, context)?;
                }
                event @ ServerConnectionEvent::PlayPacket { .. } => {
                    if let Some(player) = self.player.as_mut() {
                        let ServerConnectionEvent::PlayPacket {
                            packet,
                            teleport_pending,
                        } = event
                        else {
                            unreachable!("guarded Play packet event changed variant")
                        };
                        let collision = AuthoritativePlayerCollision::capture(
                            &*context.bridge.router_mut(),
                            player.player().region().dimension(),
                            &context.border,
                            player.player().state(),
                            &packet,
                        )?;
                        let report = player.dispatch_serverbound(
                            packet,
                            PlayerDispatchContext {
                                teleport_pending,
                                connection: &mut self.connection,
                                movement: MovementContext::default(),
                                collision: &collision,
                                target_tick: context.target_tick,
                                router: context.bridge.router_mut(),
                            },
                        )?;
                        self.last_dispatch = Some(report.outcome);
                        if matches!(
                            report.outcome.disposition(),
                            ServerboundDisposition::Unsupported
                        ) {
                            self.last_unsupported_dispatch = Some(report.outcome);
                        }
                    }
                }
                ServerConnectionEvent::Closed(_) => {
                    self.terminal = true;
                }
                event => {
                    if let Some(egress) = context.bridge.apply_java_event(
                        self.id,
                        event,
                        context.target_tick,
                        &mut AllowAll,
                    )? {
                        match egress {
                            SessionEgress::Disconnect { session, .. } => disconnect.push(session),
                            SessionEgress::CompletePlayInstallation(_) => {
                                return Err("unexpected duplicate play installation".into());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn install_play(
        &mut self,
        profile: ferrite_protocol::java_26_2::login::profile::GameProfile,
        mut admission: ferrite_protocol::semantic::PlayAdmission,
        context: &SessionContext<'_>,
    ) -> Result<(), DynError> {
        let respawn_chunk = context.respawn_position.chunk();
        let respawn_region = admission.region_mapping.region_for_chunk(
            admission.region.world(),
            admission.region.dimension().clone(),
            respawn_chunk,
        );
        if respawn_region == admission.region {
            admission.spawn_chunk = respawn_chunk;
            admission.spawn = PlayerSpawn {
                x: f64::from(context.respawn_position.x) + 0.5,
                y: f64::from(context.respawn_position.y),
                z: f64::from(context.respawn_position.z) + 0.5,
                yaw: 0.0,
                pitch: 0.0,
            };
        }
        self.connection.enqueue_play(
            &entry::before_position(
                &profile,
                &admission,
                context.maximum_sessions,
                context.simulation_distance,
                context.dimensions,
            )?,
            context.registries,
        )?;
        let maximum_tracked_chunks = maximum_tracked_chunks(context.view_distance)?;
        let mut player = JavaPlayerConnection::new(
            admission.clone(),
            context.registries.clone(),
            context.view_distance,
            context.simulation_distance,
            crate::chunk::session::ChunkSessionLimits {
                maximum_tracked_chunks,
                maximum_tickets: maximum_tracked_chunks.saturating_add(1),
                maximum_chunks_per_batch: CHUNK_BATCH_SIZE,
            },
        )?;
        player.install_terrain_registry_map(context.terrain_registries.clone());
        player.begin_server_tick();
        player.finish_play_installation(&mut self.connection)?;
        self.connection.enqueue_play(
            &entry::after_position(
                &admission,
                context.environment,
                &context.border,
                context.world_spawn,
            )?,
            context.registries,
        )?;
        player.enqueue_initial_terrain(&mut self.connection)?;
        self.player = Some(player);
        Ok(())
    }

    fn flush_available(&mut self, now_millis: i64) -> Result<(), DynError> {
        loop {
            if self.pending_write.is_none() {
                self.pending_write = self
                    .connection
                    .take_outbound()
                    .map(|frame| PendingWrite { frame, offset: 0 });
            }
            let Some(pending) = self.pending_write.as_mut() else {
                return Ok(());
            };
            match self.stream.write(&pending.frame.bytes[pending.offset..]) {
                Ok(0) => return Err(std::io::Error::from(ErrorKind::WriteZero).into()),
                Ok(length) => pending.offset += length,
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            }
            if pending.offset == pending.frame.bytes.len() {
                let sequence = pending.frame.sequence;
                self.pending_write = None;
                self.connection.outbound_sent(sequence, now_millis, false)?;
            }
        }
    }

    fn finish_server_tick(&mut self) -> Result<(), DynError> {
        if self.connection.stage() == ServerConnectionStage::Play
            && let Some(player) = self.player.as_mut()
        {
            player.finish_server_tick(&mut self.connection, 0.08, false)?;
        }
        Ok(())
    }

    fn observe_tick(
        &mut self,
        report: &CompositeGatewayTickReport,
        terrain: &BTreeMap<ChunkPos, ChunkSnapshot>,
        terrain_registries: &JavaTerrainRegistryMap,
        registries: &PlayRegistries,
        environment: &EnvironmentProjection,
    ) -> Result<(), DynError> {
        if self.connection.stage() == ServerConnectionStage::Play
            && let Some(player) = self.player.as_mut()
        {
            let update =
                player.observe_committed_tick_and_project(report.local(), &mut self.connection)?;
            if matches!(update.player, PlayerSessionAction::RegionTransferCommitted) {
                self.region_transfers = self.region_transfers.saturating_add(1);
            }
            if let Some(result) = update.block_results.last() {
                self.last_block_result = Some(BlockResultStatus {
                    command_sequence: result.command_sequence,
                    outcome: block_outcome_name(result.outcome),
                    corrections: result.corrections.len(),
                });
            }
            let projected = self
                .projections
                .project(SESSION_PROJECTION_BATCH_SIZE, terrain_registries)?;
            self.connection
                .enqueue_play(&projected.packets, registries)?;
            self.connection.enqueue_play(
                &crate::minecraft::environment::tick_packets(environment)?,
                registries,
            )?;
            self.deferred_projections = self
                .deferred_projections
                .saturating_add(projected.deferred.len());
            player.enqueue_next_terrain_batch(&mut self.connection, |position| {
                terrain.get(&position).cloned()
            })?;
        }
        Ok(())
    }

    fn admit_projections(
        &mut self,
        projections: &[SessionProjection],
    ) -> Result<usize, crate::composite::projection::SessionProjectionError> {
        let Some(player) = self.player.as_ref() else {
            return Ok(0);
        };
        self.projections
            .admit(player.stable_id(), player.player().region(), projections)
    }

    fn begin_server_tick(&mut self) {
        if self.connection.stage() == ServerConnectionStage::Play
            && let Some(player) = self.player.as_mut()
        {
            player.begin_server_tick();
        }
    }

    const fn last_dispatch(&self) -> Option<ServerboundDispatchOutcome> {
        self.last_dispatch
    }

    fn status(&self) -> MinecraftSessionStatus {
        let player = self.player.as_ref();
        let region = player.map(|connection| connection.player().region().coordinate());
        MinecraftSessionStatus {
            session_id: self.id.get(),
            player: player.map(|connection| connection.stable_id().to_string()),
            region_x: region.map(ferrite_foundation::region::RegionCoord::x),
            region_z: region.map(ferrite_foundation::region::RegionCoord::z),
            region_transfers: self.region_transfers,
            last_dispatch: self.last_dispatch.map(dispatch_status),
            last_unsupported_dispatch: self.last_unsupported_dispatch.map(dispatch_status),
            last_block_result: self.last_block_result,
        }
    }

    fn begin_drain(&mut self, registries: &PlayRegistries) {
        if self.drain_started {
            return;
        }
        self.drain_started = true;
        if self.connection.stage() == ServerConnectionStage::Play {
            if self
                .connection
                .disconnect_play(PlayDisconnectReason::ServerError, registries)
                .is_err()
            {
                self.terminate();
            }
        } else {
            self.terminate();
        }
    }

    fn terminate(&mut self) {
        self.terminal = true;
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

fn maximum_tracked_chunks(view_distance: u16) -> Result<usize, DynError> {
    let diameter = usize::from(view_distance)
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .ok_or("view-distance diameter overflow")?;
    diameter
        .checked_mul(diameter)
        .ok_or_else(|| "view-distance area overflow".into())
}

struct PendingWrite {
    frame: OutboundFrame,
    offset: usize,
}

fn unix_millis() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    i64::try_from(millis).unwrap_or(i64::MAX)
}

const fn dispatch_status(outcome: ServerboundDispatchOutcome) -> ServerboundDispatchStatus {
    ServerboundDispatchStatus {
        packet: outcome.packet(),
        responsibility: outcome.responsibility_name(),
        disposition: outcome.disposition_name(),
        detail: outcome.disposition_detail(),
    }
}

const fn block_outcome_name(outcome: BlockCommandOutcome) -> &'static str {
    match outcome {
        BlockCommandOutcome::Applied => "applied",
        BlockCommandOutcome::Rejected => "rejected",
        BlockCommandOutcome::Tracking => "tracking",
        BlockCommandOutcome::Cleared => "cleared",
    }
}

#[derive(Debug, Error)]
#[error("Minecraft gateway failed: {source}")]
pub(crate) struct MinecraftGatewayError {
    #[source]
    source: DynError,
}

impl MinecraftGatewayError {
    fn new(source: DynError) -> Self {
        Self { source }
    }
}
