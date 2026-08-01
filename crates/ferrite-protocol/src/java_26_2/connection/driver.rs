use std::collections::VecDeque;

use crate::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use crate::java_26_2::configuration::clientbound::codec as configuration_clientbound_codec;
use crate::java_26_2::configuration::clientbound::packet::ConfigurationClientboundPacket;
use crate::java_26_2::configuration::serverbound::codec as configuration_serverbound_codec;
use crate::java_26_2::configuration::serverbound::session::{
    ConfigurationServerSession, ServerAction,
};
use crate::java_26_2::connection::error::ServerConnectionError;
use crate::java_26_2::connection::output::{
    ConnectionCloseReason, OutboundFrame, PlayDisconnectReason, PlayInstallationRequest,
    ServerConnectionEvent, ServerConnectionStage,
};
use crate::java_26_2::connection::settings::ServerConnectionSettings;
use crate::java_26_2::handshake::codec as handshake_codec;
use crate::java_26_2::handshake::transition::{
    HandshakeSession, HandshakeStep, LoginRefusal, RoutingContext,
};
use crate::java_26_2::login::clientbound::codec as login_clientbound_codec;
use crate::java_26_2::login::clientbound::packet::LoginClientboundPacket;
use crate::java_26_2::login::component_json::LoginDisconnectReason;
use crate::java_26_2::login::profile::GameProfile;
use crate::java_26_2::login::serverbound::codec as login_serverbound_codec;
use crate::java_26_2::login::serverbound::session::{
    AdmissionSnapshot, ConfigurationTransitionStep, LoginDisconnect, LoginServerAction,
    LoginServerSession,
};
use crate::java_26_2::play::clientbound::codec as play_clientbound_codec;
use crate::java_26_2::play::clientbound::packet::{
    BlockChangedAck, PlayClientboundPacket, PlayerPosition, Vector3,
};
use crate::java_26_2::play::registry::PlayRegistries;
use crate::java_26_2::play::serverbound::codec as play_serverbound_codec;
use crate::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use crate::java_26_2::play::serverbound::session::{
    PlayServerSession, PlaySessionAction, PlayerCorrectionChallenge,
};
use crate::java_26_2::status::clientbound::codec as status_clientbound_codec;
use crate::java_26_2::status::clientbound::packet::StatusClientboundPacket;
use crate::java_26_2::status::serverbound::codec as status_serverbound_codec;
use crate::java_26_2::status::serverbound::session::{StatusServerAction, StatusServerSession};
use crate::java_26_2::value::nbt::TextComponentNbt;
use crate::java_26_2::wire::compression::CompressionMode;
use crate::java_26_2::wire::primitive::WireReader;
use crate::java_26_2::wire::stream::{PacketStreamDecoder, PacketStreamEncoder};

const MAX_PENDING_OUTBOUND_FRAMES: usize = 128;
const MAX_PENDING_EVENTS: usize = 64;

#[derive(Debug)]
pub struct ServerConnection {
    settings: ServerConnectionSettings,
    stage: ServerConnectionStage,
    serverbound_state: ConnectionState,
    clientbound_state: ConnectionState,
    decoder: PacketStreamDecoder,
    encoder: PacketStreamEncoder,
    handshake: Option<HandshakeSession>,
    status: Option<StatusServerSession>,
    login: Option<LoginServerSession>,
    configuration: Option<ConfigurationServerSession>,
    play: Option<PlayServerSession>,
    routing_context: Option<RoutingContext>,
    profile: Option<GameProfile>,
    transferred: bool,
    next_sequence: u64,
    outbound: VecDeque<QueuedFrame>,
    in_flight: Option<InFlight>,
    events: VecDeque<ServerConnectionEvent>,
}

impl ServerConnection {
    #[must_use]
    pub fn new(mut settings: ServerConnectionSettings) -> Self {
        settings.handshake_policy.cached_status_available = settings.status.is_some();
        let limits = settings.frame_limits;
        Self {
            handshake: Some(HandshakeSession::new(settings.handshake_policy)),
            settings,
            stage: ServerConnectionStage::Handshake,
            serverbound_state: ConnectionState::Handshake,
            clientbound_state: ConnectionState::Handshake,
            decoder: PacketStreamDecoder::new(limits, CompressionMode::Disabled),
            encoder: PacketStreamEncoder::new(limits, CompressionMode::Disabled),
            status: None,
            login: None,
            configuration: None,
            play: None,
            routing_context: None,
            profile: None,
            transferred: false,
            next_sequence: 0,
            outbound: VecDeque::new(),
            in_flight: None,
            events: VecDeque::new(),
        }
    }

    #[must_use]
    pub const fn stage(&self) -> ServerConnectionStage {
        self.stage
    }

    #[must_use]
    pub const fn serverbound_state(&self) -> ConnectionState {
        self.serverbound_state
    }

    #[must_use]
    pub const fn clientbound_state(&self) -> ConnectionState {
        self.clientbound_state
    }

    #[must_use]
    pub const fn compression(&self) -> CompressionMode {
        self.decoder.compression()
    }

    #[must_use]
    pub fn routing_context(&self) -> Option<&RoutingContext> {
        self.routing_context.as_ref()
    }

    #[must_use]
    pub fn profile(&self) -> Option<&GameProfile> {
        self.profile.as_ref()
    }

    #[must_use]
    pub const fn transferred(&self) -> bool {
        self.transferred
    }

    #[must_use]
    pub fn pending_outbound(&self) -> usize {
        self.outbound.len() + usize::from(self.in_flight.is_some())
    }

    pub fn receive(
        &mut self,
        bytes: &[u8],
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        self.require_live()?;
        let result = self.receive_inner(bytes, now_millis, is_singleplayer_owner);
        self.fault_on_error(result)
    }

    pub fn tick(
        &mut self,
        admission: AdmissionSnapshot,
        server_session_id: u128,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        self.require_live()?;
        let result = self.tick_inner(
            admission,
            server_session_id,
            now_millis,
            is_singleplayer_owner,
        );
        self.fault_on_error(result)
    }

    pub fn spawn_ready(&mut self) -> Result<(), ServerConnectionError> {
        self.require_live()?;
        let result = self.spawn_ready_inner();
        self.fault_on_error(result)
    }

    pub fn complete_play_installation(&mut self) -> Result<(), ServerConnectionError> {
        if matches!(
            self.stage,
            ServerConnectionStage::Closed | ServerConnectionStage::Faulted
        ) {
            return Err(ServerConnectionError::TerminalStage { stage: self.stage });
        }
        let result = self.complete_play_installation_inner();
        self.fault_on_error(result)
    }

    pub fn enqueue_play(
        &mut self,
        packets: &[PlayClientboundPacket],
        registries: &PlayRegistries,
    ) -> Result<(), ServerConnectionError> {
        if !matches!(
            self.stage,
            ServerConnectionStage::InstallingPlay | ServerConnectionStage::Play
        ) {
            return Err(ServerConnectionError::UnexpectedStage {
                operation: "enqueue play projection",
                expected: ServerConnectionStage::InstallingPlay,
                actual: self.stage,
            });
        }
        self.ensure_outbound_capacity(packets.len())?;
        self.next_sequence
            .checked_add(packets.len() as u64)
            .ok_or(ServerConnectionError::SequenceExhausted)?;
        let encoded = packets
            .iter()
            .map(|packet| {
                Ok((
                    play_clientbound_codec::packet_identity(packet),
                    play_clientbound_codec::encode_packet(packet, registries)?,
                ))
            })
            .collect::<Result<Vec<_>, ServerConnectionError>>()?;
        for (identity, body) in encoded {
            self.queue_frame(ConnectionState::Play, identity, body, Completion::None)?;
        }
        Ok(())
    }

    pub fn issue_player_correction(
        &mut self,
        authoritative_position: Vector3,
        yaw: f32,
        pitch: f32,
        registries: &PlayRegistries,
    ) -> Result<i32, ServerConnectionError> {
        if !matches!(
            self.stage,
            ServerConnectionStage::InstallingPlay | ServerConnectionStage::Play
        ) {
            return Err(ServerConnectionError::UnexpectedStage {
                operation: "player correction",
                expected: ServerConnectionStage::InstallingPlay,
                actual: self.stage,
            });
        }
        let mut candidate = *self
            .play
            .as_ref()
            .ok_or(ServerConnectionError::MissingStateOwner("play"))?;
        let correction = candidate.issue_correction(authoritative_position, yaw, pitch);
        let challenge = correction.teleport.challenge;
        self.queue_player_correction(correction, registries)?;
        self.play = Some(candidate);
        Ok(challenge)
    }

    pub fn register_block_sequence(&mut self, sequence: i32) -> Result<(), ServerConnectionError> {
        self.require_live()?;
        let result = if self.stage != ServerConnectionStage::Play {
            Err(ServerConnectionError::UnexpectedStage {
                operation: "register block prediction sequence",
                expected: ServerConnectionStage::Play,
                actual: self.stage,
            })
        } else {
            self.play
                .as_mut()
                .ok_or(ServerConnectionError::MissingStateOwner("play"))?
                .register_block_sequence(sequence)
                .map_err(ServerConnectionError::from)
        };
        self.fault_on_error(result)
    }

    pub fn disconnect_play(
        &mut self,
        reason: PlayDisconnectReason,
        registries: &PlayRegistries,
    ) -> Result<(), ServerConnectionError> {
        if self.stage != ServerConnectionStage::Play {
            return Err(ServerConnectionError::UnexpectedStage {
                operation: "play disconnect",
                expected: ServerConnectionStage::Play,
                actual: self.stage,
            });
        }
        let message = self.play_disconnect_message(reason);
        self.queue_play(
            PlayClientboundPacket::Disconnect(message),
            registries,
            Completion::Close(ConnectionCloseReason::Play(reason)),
        )?;
        self.stage = ServerConnectionStage::Closing;
        Ok(())
    }

    pub fn take_outbound(&mut self) -> Option<OutboundFrame> {
        if self.in_flight.is_some() {
            return None;
        }
        let queued = self.outbound.pop_front()?;
        let frame = queued.frame;
        self.in_flight = Some(InFlight {
            sequence: frame.sequence,
            completion: queued.completion,
        });
        Some(frame)
    }

    pub fn outbound_sent(
        &mut self,
        sequence: u64,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        let result = self.outbound_sent_inner(sequence, now_millis, is_singleplayer_owner);
        self.fault_on_error(result)
    }

    pub fn take_event(&mut self) -> Option<ServerConnectionEvent> {
        self.events.pop_front()
    }

    fn receive_inner(
        &mut self,
        bytes: &[u8],
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        self.decoder.push(bytes)?;
        self.drive_inbound(now_millis, is_singleplayer_owner)
    }

    fn drive_inbound(
        &mut self,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        while !self.has_input_barrier()
            && !matches!(
                self.stage,
                ServerConnectionStage::Closing
                    | ServerConnectionStage::Closed
                    | ServerConnectionStage::Faulted
                    | ServerConnectionStage::InstallingPlay
            )
        {
            let Some(body) = self.decoder.next_packet()? else {
                break;
            };
            match self.stage {
                ServerConnectionStage::Handshake => self.handle_handshake(&body)?,
                ServerConnectionStage::Status => self.handle_status(&body)?,
                ServerConnectionStage::Login => self.handle_login(&body, now_millis)?,
                ServerConnectionStage::Configuration => {
                    self.handle_configuration(&body, now_millis, is_singleplayer_owner)?;
                }
                ServerConnectionStage::Play => {
                    self.handle_play(&body, now_millis, is_singleplayer_owner)?;
                }
                _ => unreachable!("terminal and installing stages are excluded by the loop"),
            }
        }
        Ok(())
    }

    fn handle_handshake(&mut self, body: &[u8]) -> Result<(), ServerConnectionError> {
        let packet = handshake_codec::decode_packet(body)?;
        let plan = self
            .handshake
            .as_mut()
            .ok_or(ServerConnectionError::MissingStateOwner("handshake"))?
            .route(packet)?;
        self.routing_context = Some(plan.routing_context.clone());
        self.push_event(ServerConnectionEvent::Routed(plan.routing_context))?;

        for step in plan.steps {
            match step {
                HandshakeStep::InstallStatusClientbound => {
                    self.clientbound_state = ConnectionState::Status;
                }
                HandshakeStep::InstallStatusServerbound => {
                    self.serverbound_state = ConnectionState::Status;
                    let snapshot = self
                        .settings
                        .status
                        .clone()
                        .ok_or(ServerConnectionError::MissingStatusSnapshot)?;
                    self.status = Some(StatusServerSession::new(snapshot));
                    self.stage = ServerConnectionStage::Status;
                }
                HandshakeStep::InstallLoginClientbound => {
                    self.clientbound_state = ConnectionState::Login;
                }
                HandshakeStep::InstallLoginServerbound { transferred } => {
                    self.serverbound_state = ConnectionState::Login;
                    self.transferred = transferred;
                    self.login = Some(LoginServerSession::new(self.settings.login_policy.clone()));
                    self.stage = ServerConnectionStage::Login;
                }
                HandshakeStep::SendLoginDisconnect(refusal) => {
                    let reason = self.handshake_refusal_message(refusal);
                    self.queue_login(
                        LoginClientboundPacket::Disconnect(reason),
                        Completion::Close(ConnectionCloseReason::HandshakeRefused(refusal)),
                    )?;
                    self.stage = ServerConnectionStage::Closing;
                }
                HandshakeStep::Close if self.stage != ServerConnectionStage::Closing => {
                    self.close_now(ConnectionCloseReason::StatusUnavailable)?;
                }
                HandshakeStep::Close => {}
            }
        }
        Ok(())
    }

    fn handle_status(&mut self, body: &[u8]) -> Result<(), ServerConnectionError> {
        let packet = status_serverbound_codec::decode_packet(body)?;
        let action = self
            .status
            .as_mut()
            .ok_or(ServerConnectionError::MissingStateOwner("status"))?
            .apply(packet)?;
        match action {
            StatusServerAction::Send(packet @ StatusClientboundPacket::Pong(_)) => {
                self.queue_status(packet, Completion::StatusPong)?;
                self.stage = ServerConnectionStage::Closing;
            }
            StatusServerAction::Send(packet) => self.queue_status(packet, Completion::None)?,
            StatusServerAction::CloseRequestHandled => {
                self.close_now(ConnectionCloseReason::StatusRequestHandled)?;
            }
        }
        Ok(())
    }

    fn handle_login(&mut self, body: &[u8], now_millis: i64) -> Result<(), ServerConnectionError> {
        let packet = login_serverbound_codec::decode_packet(body)?;
        let action = self
            .login
            .as_mut()
            .ok_or(ServerConnectionError::MissingStateOwner("login"))?
            .apply(packet)?;
        self.apply_login_action(action, None, now_millis)
    }

    fn handle_configuration(
        &mut self,
        body: &[u8],
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        let packet = configuration_serverbound_codec::decode_packet(body)?;
        let action = self
            .configuration
            .as_mut()
            .ok_or(ServerConnectionError::MissingStateOwner("configuration"))?
            .apply(packet, now_millis, is_singleplayer_owner)?;
        self.apply_configuration_action(action)
    }

    fn handle_play(
        &mut self,
        body: &[u8],
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        if discard_base_play_custom_payload(body)? {
            return Ok(());
        }
        let packet = play_serverbound_codec::decode_packet_with_registries(
            body,
            &self.settings.play_registries,
        )?;
        match packet {
            PlayServerboundEntryPacket::AcceptTeleportation(packet) => {
                let acknowledgement = self
                    .play
                    .as_mut()
                    .ok_or(ServerConnectionError::MissingStateOwner("play"))?
                    .acknowledge_teleport(packet.challenge);
                if acknowledgement
                    == crate::java_26_2::play::serverbound::teleport::TeleportAcknowledgement::DisconnectInvalidMovement
                {
                    return self.disconnect_play(
                        PlayDisconnectReason::InvalidPlayerMovement,
                        &PlayRegistries::default(),
                    );
                }
                self.push_event(ServerConnectionEvent::TeleportAcknowledged(acknowledgement))?;
            }
            PlayServerboundEntryPacket::KeepAlive(packet) => {
                let action = self
                    .play
                    .as_mut()
                    .ok_or(ServerConnectionError::MissingStateOwner("play"))?
                    .accept_keep_alive(packet, now_millis, is_singleplayer_owner);
                self.apply_play_action(action)?;
            }
            packet => {
                let teleport_pending = self
                    .play
                    .as_ref()
                    .ok_or(ServerConnectionError::MissingStateOwner("play"))?
                    .teleport_pending();
                self.push_event(ServerConnectionEvent::PlayPacket {
                    packet,
                    teleport_pending,
                })?;
            }
        }
        Ok(())
    }

    fn tick_inner(
        &mut self,
        admission: AdmissionSnapshot,
        server_session_id: u128,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        self.drive_inbound(now_millis, is_singleplayer_owner)?;
        match self.stage {
            ServerConnectionStage::Login => {
                let action = self
                    .login
                    .as_mut()
                    .ok_or(ServerConnectionError::MissingStateOwner("login"))?
                    .tick(admission, server_session_id)?;
                self.apply_login_action(action, Some(server_session_id), now_millis)
            }
            ServerConnectionStage::Configuration => {
                let action = self
                    .configuration
                    .as_mut()
                    .ok_or(ServerConnectionError::MissingStateOwner("configuration"))?
                    .poll_liveness(now_millis, is_singleplayer_owner)?;
                self.apply_configuration_action(action)
            }
            ServerConnectionStage::Play => {
                let (block_ack, correction, action) = {
                    let play = self
                        .play
                        .as_mut()
                        .ok_or(ServerConnectionError::MissingStateOwner("play"))?;
                    (
                        play.take_block_sequence_ack(),
                        play.advance_listener_tick(),
                        play.poll_liveness(now_millis, is_singleplayer_owner),
                    )
                };
                if let Some(sequence) = block_ack {
                    self.queue_play(
                        PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence }),
                        &PlayRegistries::default(),
                        Completion::None,
                    )?;
                }
                if let Some(correction) = correction {
                    self.queue_player_correction(correction, &PlayRegistries::default())?;
                }
                self.apply_play_action(action)
            }
            _ => Ok(()),
        }
    }

    fn apply_login_action(
        &mut self,
        action: LoginServerAction,
        server_session_id: Option<u128>,
        now_millis: i64,
    ) -> Result<(), ServerConnectionError> {
        match action {
            LoginServerAction::None => Ok(()),
            LoginServerAction::Disconnect(reason) => {
                let packet_reason = self.login_disconnect_message(&reason);
                let close_reason = ConnectionCloseReason::LoginRejected(reason);
                self.queue_login(
                    LoginClientboundPacket::Disconnect(packet_reason),
                    Completion::Close(close_reason),
                )?;
                self.stage = ServerConnectionStage::Closing;
                Ok(())
            }
            LoginServerAction::DisconnectExistingAndWait { profile_id } => {
                self.push_event(ServerConnectionEvent::DisconnectExisting { profile_id })
            }
            LoginServerAction::SendCompressionUncompressed(threshold) => {
                let server_session_id =
                    server_session_id.ok_or(ServerConnectionError::MissingServerSessionId)?;
                let threshold = i32::try_from(threshold.get())
                    .map_err(|_| ServerConnectionError::CompressionThresholdOutOfRange)?;
                self.queue_login(
                    LoginClientboundPacket::Compression(threshold),
                    Completion::InstallCompression(server_session_id),
                )
            }
            LoginServerAction::SendFinished(finished) => self.queue_login(
                LoginClientboundPacket::Finished(finished),
                Completion::LoginFinished,
            ),
            LoginServerAction::BeginConfiguration(transition) => {
                let expected = [
                    ConfigurationTransitionStep::InstallConfigurationClientbound,
                    ConfigurationTransitionStep::BuildNormalizedConnectionCookie,
                    ConfigurationTransitionStep::InstallConfigurationServerbound,
                    ConfigurationTransitionStep::StartConfigurationTasks,
                ];
                if transition.steps != expected {
                    return Err(ServerConnectionError::InvalidConfigurationTransition);
                }
                self.clientbound_state = ConnectionState::Configuration;
                self.profile = Some(transition.profile.clone());
                self.serverbound_state = ConnectionState::Configuration;
                self.configuration = Some(ConfigurationServerSession::new(
                    self.settings.configuration.offered_packs().to_vec(),
                    self.settings.initial_client_information.clone(),
                    now_millis,
                    self.settings.initial_latency_millis,
                ));
                self.stage = ServerConnectionStage::Configuration;
                self.ensure_outbound_capacity(3)?;
                for packet in self.settings.configuration.initial_packets() {
                    self.queue_configuration(packet, Completion::None)?;
                }
                self.push_event(ServerConnectionEvent::ConfigurationStarted {
                    profile: transition.profile,
                })
            }
        }
    }

    fn apply_configuration_action(
        &mut self,
        action: ServerAction,
    ) -> Result<(), ServerConnectionError> {
        match action {
            ServerAction::None => Ok(()),
            ServerAction::KeepAliveAccepted { latency_millis } => {
                self.push_event(ServerConnectionEvent::LatencyUpdated { latency_millis })
            }
            ServerAction::SendKeepAlive(token) => self.queue_configuration(
                ConfigurationClientboundPacket::KeepAlive(token),
                Completion::None,
            ),
            ServerAction::RegistrySelection(selection) => {
                let packets = self
                    .settings
                    .configuration
                    .synchronization_packets(selection.exact_offer_match);
                self.ensure_outbound_capacity(packets.len())?;
                for packet in packets {
                    self.queue_configuration(packet, Completion::None)?;
                }
                self.push_event(ServerConnectionEvent::RegistrySelection {
                    selected_packs: selection.selected_packs,
                    exact_offer_match: selection.exact_offer_match,
                })
            }
            ServerAction::BeginPlayInstallation(installation) => {
                self.clientbound_state = ConnectionState::Play;
                self.play = Some(PlayServerSession::new(
                    Vector3::default(),
                    self.configuration
                        .as_ref()
                        .ok_or(ServerConnectionError::MissingStateOwner("configuration"))?
                        .latency_millis(),
                ));
                self.stage = ServerConnectionStage::InstallingPlay;
                let profile = self
                    .profile
                    .clone()
                    .ok_or(ServerConnectionError::MissingNormalizedProfile)?;
                self.push_event(ServerConnectionEvent::PlayInstallationRequested(
                    PlayInstallationRequest {
                        profile,
                        client_information: installation.client_information,
                        transferred: self.transferred,
                    },
                ))
            }
            ServerAction::DisconnectTimeout => {
                self.queue_configuration(
                    ConfigurationClientboundPacket::Disconnect(
                        self.settings
                            .disconnect_messages
                            .configuration_timeout
                            .clone(),
                    ),
                    Completion::Close(ConnectionCloseReason::ConfigurationTimeout),
                )?;
                self.stage = ServerConnectionStage::Closing;
                Ok(())
            }
        }
    }

    fn apply_play_action(
        &mut self,
        action: PlaySessionAction,
    ) -> Result<(), ServerConnectionError> {
        match action {
            PlaySessionAction::None => Ok(()),
            PlaySessionAction::KeepAliveAccepted { latency_millis } => {
                self.push_event(ServerConnectionEvent::LatencyUpdated { latency_millis })
            }
            PlaySessionAction::SendKeepAlive(challenge) => self.queue_play(
                PlayClientboundPacket::KeepAlive(
                    crate::java_26_2::play::clientbound::packet::KeepAlive { challenge },
                ),
                &PlayRegistries::default(),
                Completion::None,
            ),
            PlaySessionAction::DisconnectTimeout => {
                self.disconnect_play(PlayDisconnectReason::Timeout, &PlayRegistries::default())
            }
        }
    }

    fn spawn_ready_inner(&mut self) -> Result<(), ServerConnectionError> {
        if self.stage != ServerConnectionStage::Configuration {
            return Err(ServerConnectionError::UnexpectedStage {
                operation: "spawn readiness",
                expected: ServerConnectionStage::Configuration,
                actual: self.stage,
            });
        }
        self.ensure_outbound_capacity(1)?;
        self.configuration
            .as_mut()
            .ok_or(ServerConnectionError::MissingStateOwner("configuration"))?
            .spawn_ready_and_finish_sent()?;
        self.queue_configuration(
            ConfigurationClientboundPacket::FinishConfiguration,
            Completion::None,
        )
    }

    fn complete_play_installation_inner(&mut self) -> Result<(), ServerConnectionError> {
        if self.stage != ServerConnectionStage::InstallingPlay {
            return Err(ServerConnectionError::UnexpectedStage {
                operation: "play installation completion",
                expected: ServerConnectionStage::InstallingPlay,
                actual: self.stage,
            });
        }
        self.configuration
            .as_mut()
            .ok_or(ServerConnectionError::MissingStateOwner("configuration"))?
            .play_installation_completed()?;
        self.serverbound_state = ConnectionState::Play;
        self.stage = ServerConnectionStage::Play;
        Ok(())
    }

    fn outbound_sent_inner(
        &mut self,
        sequence: u64,
        now_millis: i64,
        is_singleplayer_owner: bool,
    ) -> Result<(), ServerConnectionError> {
        let in_flight = self
            .in_flight
            .take()
            .ok_or(ServerConnectionError::NoOutboundInFlight)?;
        if in_flight.sequence != sequence {
            let expected = in_flight.sequence;
            self.in_flight = Some(in_flight);
            return Err(ServerConnectionError::UnexpectedOutboundSequence {
                expected,
                actual: sequence,
            });
        }
        match in_flight.completion {
            Completion::None => {}
            Completion::Close(reason) => self.close_now(reason)?,
            Completion::StatusPong => {
                let action = self
                    .status
                    .as_mut()
                    .ok_or(ServerConnectionError::MissingStateOwner("status"))?
                    .pong_sent()?;
                if action != StatusServerAction::CloseRequestHandled {
                    return Err(ServerConnectionError::InvalidStatusCompletion);
                }
                self.close_now(ConnectionCloseReason::StatusRequestHandled)?;
            }
            Completion::InstallCompression(server_session_id) => {
                let action = self
                    .login
                    .as_mut()
                    .ok_or(ServerConnectionError::MissingStateOwner("login"))?
                    .compression_send_completed(server_session_id)?;
                let mode = self
                    .login
                    .as_ref()
                    .ok_or(ServerConnectionError::MissingStateOwner("login"))?
                    .compression();
                self.decoder.set_compression(mode)?;
                self.encoder.set_compression(mode);
                self.apply_login_action(action, None, now_millis)?;
            }
            Completion::LoginFinished => {}
        }
        if !matches!(
            self.stage,
            ServerConnectionStage::Closed | ServerConnectionStage::Faulted
        ) {
            self.drive_inbound(now_millis, is_singleplayer_owner)?;
        }
        Ok(())
    }

    fn queue_status(
        &mut self,
        packet: StatusClientboundPacket,
        completion: Completion,
    ) -> Result<(), ServerConnectionError> {
        let identity = match &packet {
            StatusClientboundPacket::Response(_) => "minecraft:status_response",
            StatusClientboundPacket::Pong(_) => "minecraft:pong_response",
        };
        let body = status_clientbound_codec::encode_packet(&packet)?;
        self.queue_frame(ConnectionState::Status, identity, body, completion)
    }

    fn queue_login(
        &mut self,
        packet: LoginClientboundPacket,
        completion: Completion,
    ) -> Result<(), ServerConnectionError> {
        let identity = match &packet {
            LoginClientboundPacket::Disconnect(_) => "minecraft:login_disconnect",
            LoginClientboundPacket::Finished(_) => "minecraft:login_finished",
            LoginClientboundPacket::Compression(_) => "minecraft:login_compression",
        };
        let body = login_clientbound_codec::encode_packet(&packet)?;
        self.queue_frame(ConnectionState::Login, identity, body, completion)
    }

    fn queue_configuration(
        &mut self,
        packet: ConfigurationClientboundPacket,
        completion: Completion,
    ) -> Result<(), ServerConnectionError> {
        let identity = configuration_identity(&packet);
        let body = configuration_clientbound_codec::encode_packet(&packet)?;
        self.queue_frame(ConnectionState::Configuration, identity, body, completion)
    }

    fn queue_play(
        &mut self,
        packet: PlayClientboundPacket,
        registries: &PlayRegistries,
        completion: Completion,
    ) -> Result<(), ServerConnectionError> {
        let identity = play_clientbound_codec::packet_identity(&packet);
        let body = play_clientbound_codec::encode_packet(&packet, registries)?;
        self.queue_frame(ConnectionState::Play, identity, body, completion)
    }

    fn queue_player_correction(
        &mut self,
        correction: PlayerCorrectionChallenge,
        registries: &PlayRegistries,
    ) -> Result<(), ServerConnectionError> {
        self.queue_play(
            PlayClientboundPacket::PlayerPosition(PlayerPosition {
                teleport_id: correction.teleport.challenge,
                position: correction.teleport.authoritative_position,
                motion: Vector3::default(),
                yaw: correction.yaw,
                pitch: correction.pitch,
                relative_flags: 0,
            }),
            registries,
            Completion::None,
        )
    }

    fn queue_frame(
        &mut self,
        state: ConnectionState,
        identity: &'static str,
        body: Vec<u8>,
        completion: Completion,
    ) -> Result<(), ServerConnectionError> {
        if self.clientbound_state != state {
            return Err(ServerConnectionError::WrongClientboundState {
                expected: state,
                actual: self.clientbound_state,
            });
        }
        self.ensure_outbound_capacity(1)?;
        let sequence = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(ServerConnectionError::SequenceExhausted)?;
        let bytes = self.encoder.encode(&body)?;
        self.outbound.push_back(QueuedFrame {
            frame: OutboundFrame {
                sequence,
                state,
                identity,
                bytes,
            },
            completion,
        });
        Ok(())
    }

    fn ensure_outbound_capacity(&self, additional: usize) -> Result<(), ServerConnectionError> {
        let pending = self.pending_outbound().checked_add(additional).ok_or(
            ServerConnectionError::OutboundQueueFull {
                maximum: MAX_PENDING_OUTBOUND_FRAMES,
            },
        )?;
        if pending > MAX_PENDING_OUTBOUND_FRAMES {
            Err(ServerConnectionError::OutboundQueueFull {
                maximum: MAX_PENDING_OUTBOUND_FRAMES,
            })
        } else {
            Ok(())
        }
    }

    fn push_event(&mut self, event: ServerConnectionEvent) -> Result<(), ServerConnectionError> {
        if self.events.len() >= MAX_PENDING_EVENTS {
            Err(ServerConnectionError::EventQueueFull {
                maximum: MAX_PENDING_EVENTS,
            })
        } else {
            self.events.push_back(event);
            Ok(())
        }
    }

    fn close_now(&mut self, reason: ConnectionCloseReason) -> Result<(), ServerConnectionError> {
        self.stage = ServerConnectionStage::Closed;
        self.push_event(ServerConnectionEvent::Closed(reason))
    }

    fn has_input_barrier(&self) -> bool {
        self.in_flight
            .as_ref()
            .is_some_and(|frame| frame.completion.blocks_input())
            || self
                .outbound
                .iter()
                .any(|frame| frame.completion.blocks_input())
    }

    fn handshake_refusal_message(&self, refusal: LoginRefusal) -> LoginDisconnectReason {
        match refusal {
            LoginRefusal::OutdatedClient => {
                self.settings.disconnect_messages.outdated_client.clone()
            }
            LoginRefusal::IncompatibleVersion => self
                .settings
                .disconnect_messages
                .incompatible_version
                .clone(),
            LoginRefusal::TransfersDisabled => {
                self.settings.disconnect_messages.transfers_disabled.clone()
            }
        }
    }

    fn login_disconnect_message(&self, reason: &LoginDisconnect) -> LoginDisconnectReason {
        match reason {
            LoginDisconnect::InvalidName => self.settings.disconnect_messages.invalid_name.clone(),
            LoginDisconnect::AdmissionPolicy(reason) => reason.clone(),
            LoginDisconnect::IntendedProfileMismatch { .. } => {
                self.settings.disconnect_messages.invalid_profile.clone()
            }
            LoginDisconnect::SlowLogin => self.settings.disconnect_messages.slow_login.clone(),
        }
    }

    fn play_disconnect_message(&self, reason: PlayDisconnectReason) -> TextComponentNbt {
        let messages = &self.settings.disconnect_messages;
        match reason {
            PlayDisconnectReason::Timeout => messages.play_timeout.clone(),
            PlayDisconnectReason::InvalidPlayerMovement => messages.invalid_player_movement.clone(),
            PlayDisconnectReason::Flying => messages.flying.clone(),
            PlayDisconnectReason::RegionUnavailable => {
                TextComponentNbt::literal("Region unavailable")
                    .unwrap_or_else(|_| messages.play_timeout.clone())
            }
            PlayDisconnectReason::ServerError => TextComponentNbt::literal("Internal server error")
                .unwrap_or_else(|_| messages.play_timeout.clone()),
        }
    }

    fn require_live(&self) -> Result<(), ServerConnectionError> {
        if matches!(
            self.stage,
            ServerConnectionStage::Closed | ServerConnectionStage::Faulted
        ) {
            Err(ServerConnectionError::TerminalStage { stage: self.stage })
        } else {
            Ok(())
        }
    }

    fn fault_on_error<T>(
        &mut self,
        result: Result<T, ServerConnectionError>,
    ) -> Result<T, ServerConnectionError> {
        if result.is_err() {
            self.stage = ServerConnectionStage::Faulted;
            self.outbound.clear();
            self.in_flight = None;
            self.events.clear();
        }
        result
    }
}

#[derive(Debug)]
struct QueuedFrame {
    frame: OutboundFrame,
    completion: Completion,
}

#[derive(Debug)]
struct InFlight {
    sequence: u64,
    completion: Completion,
}

#[derive(Debug)]
enum Completion {
    None,
    Close(ConnectionCloseReason),
    StatusPong,
    InstallCompression(u128),
    LoginFinished,
}

impl Completion {
    const fn blocks_input(&self) -> bool {
        matches!(self, Self::InstallCompression(_) | Self::LoginFinished)
    }
}

fn configuration_identity(packet: &ConfigurationClientboundPacket) -> &'static str {
    match packet {
        ConfigurationClientboundPacket::CustomPayload(_) => "minecraft:custom_payload",
        ConfigurationClientboundPacket::Disconnect(_) => "minecraft:disconnect",
        ConfigurationClientboundPacket::FinishConfiguration => "minecraft:finish_configuration",
        ConfigurationClientboundPacket::KeepAlive(_) => "minecraft:keep_alive",
        ConfigurationClientboundPacket::Ping(_) => "minecraft:ping",
        ConfigurationClientboundPacket::RegistryData(_) => "minecraft:registry_data",
        ConfigurationClientboundPacket::UpdateEnabledFeatures(_) => {
            "minecraft:update_enabled_features"
        }
        ConfigurationClientboundPacket::UpdateTags(_) => "minecraft:update_tags",
        ConfigurationClientboundPacket::SelectKnownPacks(_) => "minecraft:select_known_packs",
    }
}

fn discard_base_play_custom_payload(body: &[u8]) -> Result<bool, ServerConnectionError> {
    let mut reader = WireReader::new(body);
    let wire_id = reader.read_var_i32()?;
    let custom_payload =
        PacketCatalog::by_wire_id(ConnectionState::Play, PacketDirection::Serverbound, wire_id)
            .is_some_and(|descriptor| descriptor.identity() == "minecraft:custom_payload");
    if !custom_payload {
        return Ok(false);
    }
    configuration_serverbound_codec::decode_custom_payload_body(&mut reader)?;
    reader.finish()?;
    Ok(true)
}
