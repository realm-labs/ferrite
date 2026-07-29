use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use ferrite_protocol::java_26_2::catalog::{
    ConnectionState, PROTOCOL_VERSION, PacketCatalog, PacketDirection,
};
use ferrite_protocol::java_26_2::configuration::clientbound::codec as configuration_clientbound;
use ferrite_protocol::java_26_2::configuration::clientbound::packet::ConfigurationClientboundPacket;
use ferrite_protocol::java_26_2::configuration::serverbound::codec as configuration_serverbound;
use ferrite_protocol::java_26_2::configuration::serverbound::packet::{
    ClientInformation, ConfigurationServerboundPacket, CustomPayload,
};
use ferrite_protocol::java_26_2::connection::driver::ServerConnection;
use ferrite_protocol::java_26_2::connection::output::{
    ConnectionCloseReason, ServerConnectionEvent, ServerConnectionStage,
};
use ferrite_protocol::java_26_2::connection::settings::ServerConnectionSettings;
use ferrite_protocol::java_26_2::handshake::packet::ClientIntention;
use ferrite_protocol::java_26_2::login::clientbound::codec as login_clientbound;
use ferrite_protocol::java_26_2::login::clientbound::packet::LoginClientboundPacket;
use ferrite_protocol::java_26_2::login::serverbound::codec as login_serverbound;
use ferrite_protocol::java_26_2::login::serverbound::packet::{LoginHello, LoginServerboundPacket};
use ferrite_protocol::java_26_2::login::serverbound::session::AdmissionSnapshot;
use ferrite_protocol::java_26_2::play::serverbound::codec as play_serverbound;
use ferrite_protocol::java_26_2::play::serverbound::packet::{
    AcceptTeleportation, ChunkBatchReceived, MovePlayerStatusOnly, MovementFlags,
    PlayServerboundEntryPacket,
};
use ferrite_protocol::java_26_2::status::clientbound::codec as status_clientbound;
use ferrite_protocol::java_26_2::status::clientbound::packet::StatusClientboundPacket;
use ferrite_protocol::java_26_2::status::serverbound::codec as status_serverbound;
use ferrite_protocol::java_26_2::status::serverbound::packet::StatusServerboundPacket;
use ferrite_protocol::java_26_2::wire::compression::CompressionMode;
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::primitive::WireReader;
use ferrite_protocol::java_26_2::wire::stream::{PacketStreamDecoder, PacketStreamEncoder};

use crate::DynError;
use crate::fixture::{
    SERVER_SESSION_ID, compact_settings, core_pack, frame, intention, play_entry_frames,
    playable_terrain_frames,
};

const IO_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SmokeReport {
    status_complete: bool,
    login_complete: bool,
    play_acknowledged: bool,
}

impl SmokeReport {
    pub(crate) fn summary(self) -> String {
        format!(
            "C0/C1 loopback TCP smoke passed: status={}, login={}, play-ack={}",
            self.status_complete, self.login_complete, self.play_acknowledged
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlayableSmokeReport {
    login_complete: bool,
    play_acknowledged: bool,
    chunk_batch_received: bool,
    player_loaded: bool,
    movement_observed: bool,
    client_tick_end: bool,
}

impl PlayableSmokeReport {
    pub(crate) fn summary(self) -> String {
        format!(
            "C2 loopback TCP smoke passed: login={}, play-ack={}, batch={}, loaded={}, movement={}, tick-end={}",
            self.login_complete,
            self.play_acknowledged,
            self.chunk_batch_received,
            self.player_loaded,
            self.movement_observed,
            self.client_tick_end
        )
    }
}

pub(crate) fn run_loopback() -> Result<SmokeReport, DynError> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let server = thread::spawn(move || -> Result<SmokeReport, DynError> {
        let (status, _) = listener.accept()?;
        let status =
            serve_connection(status, compact_settings()?, false, IO_TIMEOUT).map_err(|error| {
                let message = format!("status connection: {error}");
                DynError::from(message)
            })?;
        let (login, _) = listener.accept()?;
        let login =
            serve_connection(login, compact_settings()?, true, IO_TIMEOUT).map_err(|error| {
                let message = format!("login connection: {error}");
                DynError::from(message)
            })?;
        Ok(SmokeReport {
            status_complete: status.status_complete,
            login_complete: login.login_complete,
            play_acknowledged: login.play_acknowledged,
        })
    });

    let status_result = run_status_client(TcpStream::connect(address)?).map_err(|error| {
        let message = format!("loopback status client failed: {error}");
        DynError::from(message)
    });
    let login_result = run_login_client(TcpStream::connect(address)?).map_err(|error| {
        let message = format!("loopback login client failed: {error}");
        DynError::from(message)
    });
    let report = server
        .join()
        .map_err(|_| "loopback conformance server panicked")?
        .map_err(|error| {
            let message = format!("loopback conformance server failed: {error}");
            DynError::from(message)
        })?;
    status_result?;
    login_result?;
    if report
        != (SmokeReport {
            status_complete: true,
            login_complete: true,
            play_acknowledged: true,
        })
    {
        return Err("loopback conformance did not reach every C0/C1 boundary".into());
    }
    Ok(report)
}

pub(crate) fn run_playable_loopback() -> Result<PlayableSmokeReport, DynError> {
    let baseline = run_playable_loopback_once(false)?;
    let delayed = run_playable_loopback_once(true)?;
    if baseline != delayed {
        return Err(format!(
            "fragmented delayed C2 feedback diverged: baseline={baseline:?}, delayed={delayed:?}"
        )
        .into());
    }
    Ok(baseline)
}

fn run_playable_loopback_once(adverse: bool) -> Result<PlayableSmokeReport, DynError> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let server = thread::spawn(move || -> Result<ConnectionObservation, DynError> {
        let (login, _) = listener.accept()?;
        serve_playable_connection(login, compact_settings()?, IO_TIMEOUT)
    });
    let client = run_login_client_to(TcpStream::connect(address)?, PlayTarget::Playable, adverse);
    let observation = server
        .join()
        .map_err(|_| "C2 loopback conformance server panicked")??;
    client?;
    let report = PlayableSmokeReport {
        login_complete: observation.login_complete,
        play_acknowledged: observation.play_acknowledged,
        chunk_batch_received: observation.chunk_batch_received,
        player_loaded: observation.player_loaded,
        movement_observed: observation.movement_observed,
        client_tick_end: observation.client_tick_end,
    };
    if report
        != (PlayableSmokeReport {
            login_complete: true,
            play_acknowledged: true,
            chunk_batch_received: true,
            player_loaded: true,
            movement_observed: true,
            client_tick_end: true,
        })
    {
        return Err(format!("C2 loopback stopped at {report:?}").into());
    }
    Ok(report)
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ConnectionObservation {
    pub(crate) status_complete: bool,
    pub(crate) configuration_started: bool,
    pub(crate) registry_selection: bool,
    pub(crate) login_complete: bool,
    pub(crate) play_acknowledged: bool,
    pub(crate) chunk_batch_received: bool,
    pub(crate) player_loaded: bool,
    pub(crate) movement_observed: bool,
    pub(crate) client_tick_end: bool,
    pub(crate) peer_closed: bool,
    pub(crate) close_reason: Option<ConnectionCloseReason>,
    pub(crate) stages: Vec<ServerConnectionStage>,
    pub(crate) outbound: Vec<&'static str>,
}

pub(crate) fn serve_connection(
    stream: TcpStream,
    settings: ServerConnectionSettings,
    send_play_entry: bool,
    timeout: Duration,
) -> Result<ConnectionObservation, DynError> {
    serve_connection_to(
        stream,
        settings,
        send_play_entry.then_some(PlayTarget::Entry),
        timeout,
    )
}

pub(crate) fn serve_playable_connection(
    stream: TcpStream,
    settings: ServerConnectionSettings,
    timeout: Duration,
) -> Result<ConnectionObservation, DynError> {
    serve_connection_to(stream, settings, Some(PlayTarget::Playable), timeout)
}

fn serve_connection_to(
    mut stream: TcpStream,
    settings: ServerConnectionSettings,
    play_target: Option<PlayTarget>,
    timeout: Duration,
) -> Result<ConnectionObservation, DynError> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    stream.set_write_timeout(Some(timeout))?;
    let started = Instant::now();
    let mut connection = ServerConnection::new(settings);
    let mut observation = ConnectionObservation::default();
    let mut selection_seen = false;
    let mut spawn_ready = false;
    let mut play_decoder = None;
    let mut buffer = [0u8; 16 * 1024];
    record_stage(&mut observation, connection.stage());

    while started.elapsed() < timeout {
        let now = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
        if matches!(
            connection.stage(),
            ServerConnectionStage::Login | ServerConnectionStage::Configuration
        ) {
            connection.tick(AdmissionSnapshot::allowed(), SERVER_SESSION_ID, now, false)?;
        }
        record_stage(&mut observation, connection.stage());
        flush_connection(&mut stream, &mut connection, &mut observation, now)?;
        drain_events(
            &mut stream,
            &mut connection,
            &mut observation,
            &mut selection_seen,
            &mut play_decoder,
            play_target,
        )?;
        if selection_seen && !spawn_ready && connection.pending_outbound() == 0 {
            connection.spawn_ready()?;
            spawn_ready = true;
            continue;
        }
        if observation.status_complete
            || reached_play_target(&observation, play_target)
            || observation.close_reason.is_some()
        {
            let _ = stream.shutdown(Shutdown::Both);
            return Ok(observation);
        }
        match stream.read(&mut buffer) {
            Ok(0) => {
                observation.peer_closed = true;
                return Ok(observation);
            }
            Ok(length) => {
                if let Some(decoder) = play_decoder.as_mut() {
                    decoder.push(&buffer[..length])?;
                    while let Some(body) = decoder.next_packet()? {
                        observe_play_packet(&body, &mut observation)?;
                    }
                } else {
                    connection.receive(&buffer[..length], now, false)?;
                    record_stage(&mut observation, connection.stage());
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Err("TCP conformance connection timed out before its terminal observation".into())
}

fn flush_connection(
    stream: &mut TcpStream,
    connection: &mut ServerConnection,
    observation: &mut ConnectionObservation,
    now_millis: i64,
) -> Result<(), DynError> {
    while let Some(outbound) = connection.take_outbound() {
        observation.outbound.push(outbound.identity);
        stream.write_all(&outbound.bytes)?;
        connection.outbound_sent(outbound.sequence, now_millis, false)?;
    }
    Ok(())
}

fn record_stage(observation: &mut ConnectionObservation, stage: ServerConnectionStage) {
    if observation.stages.last() != Some(&stage) {
        observation.stages.push(stage);
    }
}

fn drain_events(
    stream: &mut TcpStream,
    connection: &mut ServerConnection,
    observation: &mut ConnectionObservation,
    selection_seen: &mut bool,
    play_decoder: &mut Option<PacketStreamDecoder>,
    play_target: Option<PlayTarget>,
) -> Result<(), DynError> {
    while let Some(event) = connection.take_event() {
        match event {
            ServerConnectionEvent::RegistrySelection { .. } => {
                observation.registry_selection = true;
                *selection_seen = true;
            }
            ServerConnectionEvent::ConfigurationStarted { .. } => {
                observation.configuration_started = true;
            }
            ServerConnectionEvent::PlayInstallationRequested(request) => {
                connection.complete_play_installation()?;
                observation.login_complete = true;
                if let Some(target) = play_target {
                    for frame in play_entry_frames(&request.profile)? {
                        stream.write_all(&frame)?;
                    }
                    if target == PlayTarget::Playable {
                        for frame in playable_terrain_frames()? {
                            stream.write_all(&frame)?;
                        }
                    }
                    *play_decoder = Some(PacketStreamDecoder::new(
                        FrameLimits::default(),
                        connection.compression(),
                    ));
                }
            }
            ServerConnectionEvent::Closed(reason) => {
                observation.status_complete =
                    matches!(reason, ConnectionCloseReason::StatusRequestHandled);
                observation.close_reason = Some(reason);
            }
            ServerConnectionEvent::Routed(_)
            | ServerConnectionEvent::DisconnectExisting { .. }
            | ServerConnectionEvent::LatencyUpdated { .. }
            | ServerConnectionEvent::PlayPacket { .. }
            | ServerConnectionEvent::TeleportAcknowledged(_) => {}
        }
    }
    Ok(())
}

fn run_status_client(mut stream: TcpStream) -> Result<(), DynError> {
    configure_client_stream(&stream)?;
    stream.write_all(&intention(
        ClientIntention::Status,
        PROTOCOL_VERSION as i32,
    )?)?;
    let request = status_serverbound::encode_packet(StatusServerboundPacket::Request)?;
    stream.write_all(&frame(&request, CompressionMode::Disabled)?)?;

    let mut decoder = PacketStreamDecoder::new(FrameLimits::default(), CompressionMode::Disabled);
    let response = read_packet(&mut stream, &mut decoder)?;
    if !matches!(
        status_clientbound::decode_packet(&response)?,
        StatusClientboundPacket::Response(_)
    ) {
        return Err("TCP status response had the wrong identity".into());
    }
    let ping =
        status_serverbound::encode_packet(StatusServerboundPacket::Ping(0x0102_0304_0506_0708))?;
    stream.write_all(&frame(&ping, CompressionMode::Disabled)?)?;
    let pong = read_packet(&mut stream, &mut decoder)?;
    if status_clientbound::decode_packet(&pong)?
        != StatusClientboundPacket::Pong(0x0102_0304_0506_0708)
    {
        return Err("TCP status pong changed the token".into());
    }
    Ok(())
}

fn run_login_client(stream: TcpStream) -> Result<(), DynError> {
    run_login_client_to(stream, PlayTarget::Entry, false)
}

fn run_login_client_to(
    mut stream: TcpStream,
    play_target: PlayTarget,
    adverse: bool,
) -> Result<(), DynError> {
    configure_client_stream(&stream)?;
    stream.write_all(&intention(ClientIntention::Login, PROTOCOL_VERSION as i32)?)?;
    let hello = login_serverbound::encode_packet(&LoginServerboundPacket::Hello(LoginHello {
        name: "Player".to_owned(),
        supplied_profile_id: 0,
    }))?;
    stream.write_all(&frame(&hello, CompressionMode::Disabled)?)?;

    let limits = FrameLimits::default();
    let mut decoder = PacketStreamDecoder::new(limits, CompressionMode::Disabled);
    let mut encoder = PacketStreamEncoder::new(limits, CompressionMode::Disabled);
    let mut state = ClientState::Login;
    loop {
        let body = read_packet(&mut stream, &mut decoder)?;
        match state {
            ClientState::Login => match login_clientbound::decode_packet(&body)? {
                LoginClientboundPacket::Compression(threshold) => {
                    let compression = CompressionMode::enabled(usize::try_from(threshold)?)?;
                    decoder.set_compression(compression)?;
                    encoder.set_compression(compression);
                }
                LoginClientboundPacket::Finished(_) => {
                    let acknowledgement =
                        login_serverbound::encode_packet(&LoginServerboundPacket::Acknowledged)?;
                    stream.write_all(&encoder.encode(&acknowledgement)?)?;
                    state = ClientState::Configuration;
                }
                LoginClientboundPacket::Disconnect(_) => {
                    return Err("loopback login was disconnected".into());
                }
            },
            ClientState::Configuration => {
                let packet = configuration_clientbound::decode_packet(&body)?;
                match packet {
                    ConfigurationClientboundPacket::CustomPayload(_) => {
                        for response in [
                            ConfigurationServerboundPacket::CustomPayload(CustomPayload::Brand(
                                "vanilla".to_owned(),
                            )),
                            ConfigurationServerboundPacket::ClientInformation(
                                ClientInformation::default(),
                            ),
                        ] {
                            let response = configuration_serverbound::encode_packet(&response)?;
                            stream.write_all(&encoder.encode(&response)?)?;
                        }
                    }
                    ConfigurationClientboundPacket::SelectKnownPacks(_) => {
                        let response = configuration_serverbound::encode_packet(
                            &ConfigurationServerboundPacket::SelectKnownPacks(vec![core_pack()]),
                        )?;
                        stream.write_all(&encoder.encode(&response)?)?;
                    }
                    ConfigurationClientboundPacket::FinishConfiguration => {
                        let response = configuration_serverbound::encode_packet(
                            &ConfigurationServerboundPacket::FinishConfiguration,
                        )?;
                        stream.write_all(&encoder.encode(&response)?)?;
                        state = ClientState::Play;
                    }
                    _ => {}
                }
            }
            ClientState::Play => {
                if packet_id(&body)? == 72 {
                    let acknowledgement = play_serverbound::encode_packet(
                        PlayServerboundEntryPacket::AcceptTeleportation(AcceptTeleportation {
                            challenge: 1,
                        }),
                    )?;
                    stream.write_all(&encoder.encode(&acknowledgement)?)?;
                    if play_target == PlayTarget::Entry {
                        stream.shutdown(Shutdown::Write)?;
                        drain_until_peer_close(&mut stream)?;
                        return Ok(());
                    }
                } else if play_target == PlayTarget::Playable && packet_id(&body)? == 11 {
                    let mut frames = Vec::new();
                    for packet in [
                        PlayServerboundEntryPacket::ChunkBatchReceived(ChunkBatchReceived {
                            desired_chunks_per_tick: 9.0,
                        }),
                        PlayServerboundEntryPacket::PlayerLoaded,
                        PlayServerboundEntryPacket::MovePlayerStatusOnly(MovePlayerStatusOnly {
                            flags: MovementFlags {
                                on_ground: true,
                                horizontal_collision: false,
                            },
                        }),
                        PlayServerboundEntryPacket::ClientTickEnd,
                    ] {
                        let packet = play_serverbound::encode_packet(packet)?;
                        frames.push(encoder.encode(&packet)?);
                    }
                    if adverse {
                        thread::sleep(Duration::from_millis(25));
                        let bytes = frames.concat();
                        for chunk in bytes.chunks(3) {
                            stream.write_all(chunk)?;
                            thread::sleep(Duration::from_millis(1));
                        }
                    } else {
                        for frame in frames {
                            stream.write_all(&frame)?;
                        }
                    }
                    stream.shutdown(Shutdown::Write)?;
                    drain_until_peer_close(&mut stream)?;
                    return Ok(());
                }
            }
        }
    }
}

fn observe_play_packet(
    body: &[u8],
    observation: &mut ConnectionObservation,
) -> Result<(), DynError> {
    let wire_id = packet_id(body)?;
    let Some(descriptor) =
        PacketCatalog::by_wire_id(ConnectionState::Play, PacketDirection::Serverbound, wire_id)
    else {
        return Err(format!("client sent unknown Play packet ID {wire_id}").into());
    };
    match descriptor.identity() {
        "minecraft:accept_teleportation"
        | "minecraft:chunk_batch_received"
        | "minecraft:client_tick_end"
        | "minecraft:move_player_pos"
        | "minecraft:move_player_pos_rot"
        | "minecraft:move_player_rot"
        | "minecraft:move_player_status_only"
        | "minecraft:player_loaded" => match play_serverbound::decode_packet(body)? {
            PlayServerboundEntryPacket::AcceptTeleportation(packet) => {
                observation.play_acknowledged |= packet.challenge == 1;
            }
            PlayServerboundEntryPacket::ChunkBatchReceived(_) => {
                observation.chunk_batch_received = true;
            }
            PlayServerboundEntryPacket::ClientTickEnd => {
                observation.client_tick_end = true;
            }
            PlayServerboundEntryPacket::MovePlayerPosition(_)
            | PlayServerboundEntryPacket::MovePlayerPositionRotation(_)
            | PlayServerboundEntryPacket::MovePlayerRotation(_)
            | PlayServerboundEntryPacket::MovePlayerStatusOnly(_) => {
                observation.movement_observed = true;
            }
            PlayServerboundEntryPacket::PlayerLoaded => {
                observation.player_loaded = true;
            }
            _ => unreachable!("identity filter contains only observed C2 packets"),
        },
        _ => {}
    }
    Ok(())
}

fn reached_play_target(observation: &ConnectionObservation, target: Option<PlayTarget>) -> bool {
    match target {
        None => false,
        Some(PlayTarget::Entry) => observation.play_acknowledged,
        Some(PlayTarget::Playable) => {
            observation.play_acknowledged
                && observation.chunk_batch_received
                && observation.player_loaded
                && observation.movement_observed
                && observation.client_tick_end
        }
    }
}

fn drain_until_peer_close(stream: &mut TcpStream) -> Result<(), DynError> {
    let mut buffer = [0u8; 16 * 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::ConnectionReset => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
}

fn configure_client_stream(stream: &TcpStream) -> Result<(), DynError> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    Ok(())
}

fn read_packet(
    stream: &mut TcpStream,
    decoder: &mut PacketStreamDecoder,
) -> Result<Vec<u8>, DynError> {
    let mut buffer = [0u8; 16 * 1024];
    loop {
        if let Some(body) = decoder.next_packet()? {
            return Ok(body);
        }
        let length = stream.read(&mut buffer)?;
        if length == 0 {
            return Err("TCP peer closed before the next complete packet".into());
        }
        decoder.push(&buffer[..length])?;
    }
}

fn packet_id(body: &[u8]) -> Result<i32, DynError> {
    Ok(WireReader::new(body).read_var_i32()?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClientState {
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayTarget {
    Entry,
    Playable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_tcp_reaches_status_login_and_play_acknowledgement() {
        assert_eq!(
            run_loopback().unwrap(),
            SmokeReport {
                status_complete: true,
                login_complete: true,
                play_acknowledged: true,
            }
        );
    }

    #[test]
    fn loopback_tcp_reaches_playable_terrain_and_c2_feedback() {
        assert!(run_playable_loopback().is_ok());
    }
}
