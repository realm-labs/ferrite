use ferrite_protocol::java_26_2::catalog::{ConnectionState, PROTOCOL_VERSION};
use ferrite_protocol::java_26_2::configuration::registry::SYNCHRONIZED_REGISTRY_IDENTITIES;
use ferrite_protocol::java_26_2::configuration::serverbound::codec as configuration_codec;
use ferrite_protocol::java_26_2::configuration::serverbound::packet::{
    ClientInformation, ConfigurationServerboundPacket, CustomPayload,
};
use ferrite_protocol::java_26_2::connection::driver::ServerConnection;
use ferrite_protocol::java_26_2::connection::output::{
    ServerConnectionEvent, ServerConnectionStage,
};
use ferrite_protocol::java_26_2::handshake::packet::ClientIntention;
use ferrite_protocol::java_26_2::login::clientbound::codec as login_clientbound_codec;
use ferrite_protocol::java_26_2::login::clientbound::packet::LoginClientboundPacket;
use ferrite_protocol::java_26_2::login::serverbound::codec as login_serverbound_codec;
use ferrite_protocol::java_26_2::login::serverbound::packet::{LoginHello, LoginServerboundPacket};
use ferrite_protocol::java_26_2::login::serverbound::session::AdmissionSnapshot;
use ferrite_protocol::java_26_2::wire::compression::CompressionMode;
use ferrite_testkit::malformed::{MalformedCase, MalformedCorpus};

use crate::DynError;
use crate::fixture::{
    SERVER_SESSION_ID, compact_settings, core_pack, frame, frame_body, intention,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConformanceReport {
    pub(crate) golden_vectors: usize,
    pub(crate) malformed_sessions: usize,
    pub(crate) transition_checks: usize,
    pub(crate) ordered_packets: usize,
}

impl ConformanceReport {
    pub(crate) fn summary(self) -> String {
        format!(
            "C0/C1 headless conformance passed: {} goldens, {} malformed sessions, \
             {} half-duplex checks, {} ordered packets",
            self.golden_vectors,
            self.malformed_sessions,
            self.transition_checks,
            self.ordered_packets
        )
    }
}

pub(crate) fn run() -> Result<ConformanceReport, DynError> {
    let golden_vectors = verify_status_trace()?;
    let malformed_sessions = verify_malformed_sessions()?;
    let (transition_checks, ordered_packets) = verify_login_configuration_trace()?;
    Ok(ConformanceReport {
        golden_vectors,
        malformed_sessions,
        transition_checks,
        ordered_packets,
    })
}

fn verify_status_trace() -> Result<usize, DynError> {
    let expected_intention = crate::fixture::decode_hex("10008806096c6f63616c686f737463dd01")?;
    require(
        intention(ClientIntention::Status, PROTOCOL_VERSION as i32)? == expected_intention,
        "status intention differs from the independent golden",
    )?;

    let mut connection = ServerConnection::new(compact_settings()?);
    for byte in expected_intention {
        connection.receive(&[byte], 0, false)?;
    }
    require(
        connection.stage() == ServerConnectionStage::Status,
        "fragmented intention did not enter status",
    )?;
    require(
        matches!(
            connection.take_event(),
            Some(ServerConnectionEvent::Routed(_))
        ),
        "status intention did not emit routing",
    )?;
    connection.receive(&crate::fixture::decode_hex("0100")?, 1, false)?;
    let response = send_next(&mut connection, CompressionMode::Disabled, 2)?;
    require(
        response.0 == "minecraft:status_response",
        "status response identity changed",
    )?;

    let ping = crate::fixture::decode_hex("09010102030405060708")?;
    connection.receive(&ping, 3, false)?;
    let pong = connection
        .take_outbound()
        .ok_or("status ping did not queue a pong")?;
    require(
        pong.bytes == ping,
        "status pong did not echo exact token bits",
    )?;
    require(
        connection.stage() == ServerConnectionStage::Closing,
        "status closed before pong send completion",
    )?;
    connection.outbound_sent(pong.sequence, 4, false)?;
    require(
        connection.stage() == ServerConnectionStage::Closed,
        "status did not close after pong send completion",
    )?;
    Ok(4)
}

fn verify_malformed_sessions() -> Result<usize, DynError> {
    let handshake_cases = MalformedCorpus::from_cases(vec![
        MalformedCase::new("zero-frame", vec![0])?,
        MalformedCase::new("wide-frame", vec![0x80, 0x80, 0x80, 0])?,
    ])?;
    for case in handshake_cases.cases() {
        let mut connection = ServerConnection::new(compact_settings()?);
        require(
            connection.receive(case.bytes(), 0, false).is_err(),
            &format!("{} was accepted", case.name()),
        )?;
        require(
            connection.stage() == ServerConnectionStage::Faulted,
            &format!("{} did not fault terminally", case.name()),
        )?;
    }

    for malformed in [
        crate::fixture::decode_hex("0102")?,
        crate::fixture::decode_hex("020000")?,
        crate::fixture::decode_hex("080100000000000000")?,
    ] {
        let mut connection = status_connection()?;
        require(
            connection.receive(&malformed, 1, false).is_err(),
            "malformed status packet was accepted",
        )?;
        require(
            connection.stage() == ServerConnectionStage::Faulted,
            "malformed status packet did not fault terminally",
        )?;
        require(
            connection.pending_outbound() == 0,
            "malformed status packet retained output",
        )?;
    }

    let hello = login_hello_frame()?;
    let mut duplicate = login_connection()?;
    duplicate.receive(&hello, 1, false)?;
    require(
        duplicate.receive(&hello, 2, false).is_err(),
        "duplicate login hello was accepted",
    )?;
    require(
        duplicate.stage() == ServerConnectionStage::Faulted,
        "duplicate login hello did not fault",
    )?;

    let mut early_ack = login_connection()?;
    let acknowledgement =
        login_serverbound_codec::encode_packet(&LoginServerboundPacket::Acknowledged)?;
    require(
        early_ack
            .receive(
                &frame(&acknowledgement, CompressionMode::Disabled)?,
                1,
                false,
            )
            .is_err(),
        "early login acknowledgement was accepted",
    )?;
    Ok(handshake_cases.cases().len() + 5)
}

fn verify_login_configuration_trace() -> Result<(usize, usize), DynError> {
    let mut connection = login_connection()?;
    connection.receive(&login_hello_frame()?, 1, false)?;
    connection.tick(AdmissionSnapshot::allowed(), SERVER_SESSION_ID, 2, false)?;

    let compression = connection
        .take_outbound()
        .ok_or("login did not queue compression")?;
    let compression_body = frame_body(&compression.bytes, CompressionMode::Disabled)?;
    require(
        login_clientbound_codec::decode_packet(&compression_body)?
            == LoginClientboundPacket::Compression(256),
        "login compression packet changed",
    )?;
    require(
        connection.clientbound_state() == ConnectionState::Login
            && connection.serverbound_state() == ConnectionState::Login,
        "compression switched a direction before send completion",
    )?;
    connection.outbound_sent(compression.sequence, 3, false)?;
    let compressed = CompressionMode::enabled(256)?;
    require(
        connection.compression() == compressed,
        "compression did not install after send completion",
    )?;

    let finished = connection
        .take_outbound()
        .ok_or("login did not queue finished")?;
    let finished_body = frame_body(&finished.bytes, compressed)?;
    require(
        matches!(
            login_clientbound_codec::decode_packet(&finished_body)?,
            LoginClientboundPacket::Finished(_)
        ),
        "login finished packet changed",
    )?;
    connection.outbound_sent(finished.sequence, 4, false)?;
    require(
        connection.clientbound_state() == ConnectionState::Login
            && connection.serverbound_state() == ConnectionState::Login,
        "login finished switched protocol before its acknowledgement",
    )?;

    let acknowledgement =
        login_serverbound_codec::encode_packet(&LoginServerboundPacket::Acknowledged)?;
    connection.receive(&frame(&acknowledgement, compressed)?, 5, false)?;
    require(
        connection.clientbound_state() == ConnectionState::Configuration
            && connection.serverbound_state() == ConnectionState::Configuration,
        "login acknowledgement did not finish the directional transition",
    )?;
    require(
        matches!(
            connection.take_event(),
            Some(ServerConnectionEvent::ConfigurationStarted { .. })
        ),
        "configuration start event was not emitted",
    )?;

    let mut ordered = Vec::new();
    for _ in 0..3 {
        ordered.push(send_next(&mut connection, compressed, 6)?.0);
    }
    require(
        ordered
            == [
                "minecraft:custom_payload",
                "minecraft:update_enabled_features",
                "minecraft:select_known_packs",
            ],
        "configuration prelude order changed",
    )?;
    for packet in [
        ConfigurationServerboundPacket::CustomPayload(CustomPayload::Brand("vanilla".to_owned())),
        ConfigurationServerboundPacket::ClientInformation(ClientInformation::default()),
        ConfigurationServerboundPacket::SelectKnownPacks(vec![core_pack()]),
    ] {
        let body = configuration_codec::encode_packet(&packet)?;
        connection.receive(&frame(&body, compressed)?, 7, false)?;
    }
    require(
        matches!(
            connection.take_event(),
            Some(ServerConnectionEvent::RegistrySelection {
                exact_offer_match: true,
                ..
            })
        ),
        "exact known-pack response was not retained",
    )?;

    for _ in 0..SYNCHRONIZED_REGISTRY_IDENTITIES.len() {
        ordered.push(send_next(&mut connection, compressed, 8)?.0);
    }
    ordered.push(send_next(&mut connection, compressed, 8)?.0);
    require(
        ordered[3..ordered.len() - 1]
            .iter()
            .all(|identity| *identity == "minecraft:registry_data"),
        "registry projection order contains another packet family",
    )?;
    require(
        ordered.last() == Some(&"minecraft:update_tags"),
        "registry projection did not end in tags",
    )?;

    connection.spawn_ready()?;
    ordered.push(send_next(&mut connection, compressed, 9)?.0);
    require(
        ordered.last() == Some(&"minecraft:finish_configuration"),
        "configuration finish order changed",
    )?;
    let finish =
        configuration_codec::encode_packet(&ConfigurationServerboundPacket::FinishConfiguration)?;
    connection.receive(&frame(&finish, compressed)?, 10, false)?;
    require(
        connection.clientbound_state() == ConnectionState::Play
            && connection.serverbound_state() == ConnectionState::Configuration
            && connection.stage() == ServerConnectionStage::InstallingPlay,
        "configuration finish did not preserve its half-duplex transition",
    )?;
    require(
        matches!(
            connection.take_event(),
            Some(ServerConnectionEvent::PlayInstallationRequested(_))
        ),
        "configuration finish did not reach semantic play admission",
    )?;
    connection.complete_play_installation()?;
    require(
        connection.clientbound_state() == ConnectionState::Play
            && connection.serverbound_state() == ConnectionState::Play
            && connection.stage() == ServerConnectionStage::Play,
        "semantic admission did not complete serverbound Play installation",
    )?;
    Ok((7, ordered.len()))
}

fn status_connection() -> Result<ServerConnection, DynError> {
    let mut connection = ServerConnection::new(compact_settings()?);
    connection.receive(
        &intention(ClientIntention::Status, PROTOCOL_VERSION as i32)?,
        0,
        false,
    )?;
    connection.take_event();
    Ok(connection)
}

fn login_connection() -> Result<ServerConnection, DynError> {
    let mut connection = ServerConnection::new(compact_settings()?);
    connection.receive(
        &intention(ClientIntention::Login, PROTOCOL_VERSION as i32)?,
        0,
        false,
    )?;
    connection.take_event();
    Ok(connection)
}

fn login_hello_frame() -> Result<Vec<u8>, DynError> {
    let body =
        login_serverbound_codec::encode_packet(&LoginServerboundPacket::Hello(LoginHello {
            name: "Player".to_owned(),
            supplied_profile_id: 0,
        }))?;
    frame(&body, CompressionMode::Disabled)
}

fn send_next(
    connection: &mut ServerConnection,
    compression: CompressionMode,
    now_millis: i64,
) -> Result<(&'static str, Vec<u8>), DynError> {
    let outbound = connection
        .take_outbound()
        .ok_or("expected one queued outbound frame")?;
    let body = frame_body(&outbound.bytes, compression)?;
    connection.outbound_sent(outbound.sequence, now_millis, false)?;
    Ok((outbound.identity, body))
}

fn require(condition: bool, message: &str) -> Result<(), DynError> {
    if condition {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_headless_c0_c1_suites_pass() {
        let report = run().unwrap();
        assert_eq!(report.golden_vectors, 4);
        assert_eq!(report.malformed_sessions, 7);
        assert_eq!(report.transition_checks, 7);
        assert_eq!(
            report.ordered_packets,
            SYNCHRONIZED_REGISTRY_IDENTITIES.len() + 5
        );
    }
}
