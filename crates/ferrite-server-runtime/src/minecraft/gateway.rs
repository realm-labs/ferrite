use std::collections::BTreeMap;
use std::error::Error;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ferrite_gameplay::player::collision::FlatWorldCollision;
use ferrite_gameplay::player::movement::MovementContext;
use ferrite_protocol::java_26_2::connection::driver::ServerConnection;
use ferrite_protocol::java_26_2::connection::output::{
    OutboundFrame, PlayDisconnectReason, ServerConnectionEvent, ServerConnectionStage,
};
use ferrite_protocol::java_26_2::connection::settings::ServerConnectionSettings;
use ferrite_protocol::java_26_2::login::serverbound::session::AdmissionSnapshot;
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::semantic::{SessionEgress, SessionId};
use ferrite_simulation::tick::GameTick;
use ferrite_world::terrain::MinimalTerrain;
use thiserror::Error;

use crate::chunk::projection::JavaTerrainRegistryMap;
use crate::composite::gateway::{CompositeGatewayTickReport, CompositeRegionRouter};
use crate::config::ValidatedServerConfig;
use crate::lifecycle::{NodeLifecycle, NodePhase};
use crate::minecraft::entry;
use crate::minecraft::settings;
use crate::minecraft::world;
use crate::player::connection::JavaPlayerConnection;
use crate::session::admission::AllowAll;
use crate::session::bridge::SessionBridge;

type DynError = Box<dyn Error + Send + Sync>;

const SERVER_TICK: Duration = Duration::from_millis(50);
const MAX_ACCEPTS_PER_POLL: usize = 64;
const MAX_READS_PER_POLL: usize = 64;
const MAX_DRIVE_PASSES: usize = 8;
const MAX_TICK_CATCH_UP: usize = 4;
const READ_BUFFER_BYTES: usize = 64 * 1024;
const CHUNK_BATCH_SIZE: usize = 4;
const SPAWN_GROUND_Y: f64 = 64.0;

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
    terrain: MinimalTerrain,
    sessions: BTreeMap<SessionId, NetworkSession>,
    maximum_sessions: usize,
    next_session: u64,
    committed_tick: GameTick,
    next_tick: Instant,
    draining: bool,
    authorities_released: bool,
    last_session_error: Option<String>,
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
        let protocol = settings::load(minecraft.registry_report.as_deref())?;
        let world = world::load(
            config.config().limits.max_region_mailbox,
            config.config().limits.max_sessions,
        )?;
        let region_authorities = world.router.len();
        let bridge = SessionBridge::new(
            world.routes,
            Arc::clone(&lifecycle),
            config.config().limits.max_sessions,
            world.router,
        )?;
        let gateway = Self {
            listener: Some(listener),
            local_address,
            lifecycle: Arc::clone(&lifecycle),
            bridge,
            settings: protocol.settings,
            registries: protocol.registries,
            terrain_registries: protocol.terrain_registries,
            terrain: world.terrain,
            sessions: BTreeMap::new(),
            maximum_sessions: config.config().limits.max_sessions,
            next_session: 1,
            committed_tick: GameTick::ZERO,
            next_tick: Instant::now() + SERVER_TICK,
            draining: false,
            authorities_released: false,
            last_session_error: None,
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

    pub(crate) fn poll(&mut self, phase: NodePhase) -> Result<GatewayPoll, MinecraftGatewayError> {
        self.poll_inner(phase).map_err(MinecraftGatewayError::new)
    }

    fn poll_inner(&mut self, phase: NodePhase) -> Result<GatewayPoll, DynError> {
        if matches!(phase, NodePhase::Draining | NodePhase::Drained) {
            self.begin_drain();
        }
        if phase == NodePhase::Ready && !self.draining {
            self.accept_connections()?;
        }
        self.poll_sessions();
        self.run_due_ticks()?;
        self.poll_sessions();
        if self.draining && self.sessions.is_empty() && !self.authorities_released {
            self.lifecycle.set_active_region_authorities(0)?;
            self.authorities_released = true;
        }
        Ok(GatewayPoll {
            active_sessions: self.sessions.len(),
            committed_tick: self.committed_tick,
        })
    }

    fn begin_drain(&mut self) {
        if self.draining {
            return;
        }
        self.draining = true;
        self.listener.take();
        for session in self.sessions.values_mut() {
            session.begin_drain(&self.registries);
        }
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
                NetworkSession::new(session, stream, self.settings.clone()),
            );
        }
        Ok(())
    }

    fn poll_sessions(&mut self) {
        let ids = self.sessions.keys().copied().collect::<Vec<_>>();
        let target_tick = self
            .committed_tick
            .checked_next()
            .unwrap_or(self.committed_tick);
        let now_millis = unix_millis();
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
                    terrain: &self.terrain,
                })
            };
            match result {
                Ok(actions) => disconnect.extend(actions),
                Err(error) => {
                    self.last_session_error = Some(format!("session {}: {error}", id.get()));
                    session.terminate();
                }
            }
            if session.terminal {
                if !self.close_registered_session(id) {
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
    }

    fn close_registered_session(&mut self, id: SessionId) -> bool {
        let target_tick = self
            .committed_tick
            .checked_next()
            .unwrap_or(self.committed_tick);
        match self.bridge.unregister(id, target_tick) {
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
        let report = self.bridge.router_mut().run_tick(tick)?;
        failed.clear();
        for (id, session) in &mut self.sessions {
            if let Err(error) = session.observe_tick(&report, &self.terrain) {
                failed.push((*id, error.to_string()));
                session.terminate();
            }
            session.begin_server_tick();
        }
        self.record_session_failures(&failed);
        self.committed_tick = tick;
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
    terrain: &'a MinimalTerrain,
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
}

impl NetworkSession {
    fn new(id: SessionId, stream: TcpStream, settings: ServerConnectionSettings) -> Self {
        Self {
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
        }
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
                        player.handle_java_event(
                            event,
                            &mut self.connection,
                            MovementContext::default(),
                            &FlatWorldCollision {
                                ground_y: SPAWN_GROUND_Y,
                            },
                            context.target_tick,
                            context.bridge.router_mut(),
                        )?;
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
        admission: ferrite_protocol::semantic::PlayAdmission,
        context: &SessionContext<'_>,
    ) -> Result<(), DynError> {
        self.connection.enqueue_play(
            &entry::before_position(&profile, &admission, context.maximum_sessions)?,
            context.registries,
        )?;
        let mut player = JavaPlayerConnection::new(
            admission.clone(),
            context.registries.clone(),
            8,
            10,
            crate::chunk::session::ChunkSessionLimits {
                maximum_tracked_chunks: 289,
                maximum_tickets: 290,
                maximum_chunks_per_batch: CHUNK_BATCH_SIZE,
            },
        )?;
        player.install_terrain_registry_map(context.terrain_registries.clone());
        player.begin_server_tick();
        player.finish_play_installation(&mut self.connection)?;
        self.connection
            .enqueue_play(&entry::after_position(&admission)?, context.registries)?;
        player.enqueue_initial_terrain(&mut self.connection, context.terrain)?;
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
        terrain: &MinimalTerrain,
    ) -> Result<(), DynError> {
        if self.connection.stage() == ServerConnectionStage::Play
            && let Some(player) = self.player.as_mut()
        {
            player.observe_committed_tick_and_project(report.local(), &mut self.connection)?;
            player.enqueue_next_terrain_batch(&mut self.connection, terrain)?;
        }
        Ok(())
    }

    fn begin_server_tick(&mut self) {
        if self.connection.stage() == ServerConnectionStage::Play
            && let Some(player) = self.player.as_mut()
        {
            player.begin_server_tick();
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
