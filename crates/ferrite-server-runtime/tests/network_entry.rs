use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use ferrite_protocol::java_26_2::catalog::PROTOCOL_VERSION;
use ferrite_protocol::java_26_2::handshake::codec as handshake_codec;
use ferrite_protocol::java_26_2::handshake::packet::{ClientIntention, ClientIntentionPacket};
use ferrite_protocol::java_26_2::status::clientbound::codec as status_clientbound;
use ferrite_protocol::java_26_2::status::clientbound::packet::StatusClientboundPacket;
use ferrite_protocol::java_26_2::status::serverbound::codec as status_serverbound;
use ferrite_protocol::java_26_2::status::serverbound::packet::StatusServerboundPacket;
use ferrite_protocol::java_26_2::wire::compression::CompressionMode;
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::stream::{PacketStreamDecoder, PacketStreamEncoder};
use ferrite_server_runtime::config::{AdvertisedAddress, DiscoveryConfig, ServerConfig};
use ferrite_server_runtime::lifecycle::NodePhase;
use ferrite_server_runtime::process::{NodeProcess, ProcessPoll};

const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn formal_network_entry_serves_status_holds_sessions_and_drains() {
    let state = tempfile::tempdir().unwrap();
    let [remoting, management, minecraft] = free_addresses();
    let mut config = ServerConfig::development_node(1, 1, 30_000, state.path()).unwrap();
    config.remoting.bind = remoting;
    config.remoting.advertise = AdvertisedAddress {
        host: remoting.ip().to_string(),
        port: remoting.port(),
    };
    config.discovery = DiscoveryConfig::DevelopmentStatic {
        peers: vec![config.remoting.advertise.clone()],
        minimum_members: 1,
    };
    config.management.bind = management;
    config.minecraft.bind = minecraft;
    let validated = ServerConfig::from_toml(&config.to_toml().unwrap()).unwrap();
    let mut process = NodeProcess::start(validated).unwrap();
    poll_until(&mut process, |process| {
        process.lifecycle().snapshot().unwrap().phase == NodePhase::Ready
    });
    assert_eq!(process.minecraft_address(), Some(minecraft));
    assert_eq!(
        process
            .lifecycle()
            .snapshot()
            .unwrap()
            .active_region_authorities,
        25
    );

    let status = thread::spawn(move || status_round_trip(minecraft));
    poll_until(&mut process, |_| status.is_finished());
    status.join().unwrap().unwrap();
    poll_until(&mut process, |process| {
        process.lifecycle().snapshot().unwrap().active_sessions == 0
    });

    let held = TcpStream::connect(minecraft).unwrap();
    poll_until(&mut process, |process| {
        process.lifecycle().snapshot().unwrap().active_sessions == 1
    });
    let hold_until = Instant::now() + Duration::from_millis(150);
    while Instant::now() < hold_until {
        assert_eq!(process.poll().unwrap(), ProcessPoll::Running);
        assert_eq!(process.lifecycle().snapshot().unwrap().active_sessions, 1);
        thread::sleep(Duration::from_millis(5));
    }

    process.begin_drain().unwrap();
    let drain_deadline = Instant::now() + TIMEOUT;
    while process.poll().unwrap() != ProcessPoll::Drained {
        assert!(Instant::now() < drain_deadline, "server drain timed out");
        thread::sleep(Duration::from_millis(5));
    }
    drop(held);
    assert_eq!(
        process.lifecycle().snapshot().unwrap().phase,
        NodePhase::Drained
    );
    process.stop().unwrap();
}

fn status_round_trip(address: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect(address)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    let encoder = PacketStreamEncoder::new(FrameLimits::default(), CompressionMode::Disabled);
    let intention = handshake_codec::encode_packet(&ClientIntentionPacket {
        protocol_version: PROTOCOL_VERSION as i32,
        host: "localhost".to_owned(),
        port: address.port(),
        intention: ClientIntention::Status,
    })?;
    stream.write_all(&encoder.encode(&intention)?)?;
    let request = status_serverbound::encode_packet(StatusServerboundPacket::Request)?;
    stream.write_all(&encoder.encode(&request)?)?;

    let mut decoder = PacketStreamDecoder::new(FrameLimits::default(), CompressionMode::Disabled);
    let response = read_packet(&mut stream, &mut decoder)?;
    assert!(matches!(
        status_clientbound::decode_packet(&response)?,
        StatusClientboundPacket::Response(_)
    ));
    let token = 0x0102_0304_0506_0708;
    let ping = status_serverbound::encode_packet(StatusServerboundPacket::Ping(token))?;
    stream.write_all(&encoder.encode(&ping)?)?;
    let pong = read_packet(&mut stream, &mut decoder)?;
    assert_eq!(
        status_clientbound::decode_packet(&pong)?,
        StatusClientboundPacket::Pong(token)
    );
    Ok(())
}

fn read_packet(
    stream: &mut TcpStream,
    decoder: &mut PacketStreamDecoder,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = [0u8; 16 * 1024];
    loop {
        if let Some(body) = decoder.next_packet()? {
            return Ok(body);
        }
        let length = stream.read(&mut buffer)?;
        if length == 0 {
            return Err("server closed before the next status packet".into());
        }
        decoder.push(&buffer[..length])?;
    }
}

fn poll_until(process: &mut NodeProcess, done: impl Fn(&NodeProcess) -> bool) {
    let deadline = Instant::now() + TIMEOUT;
    while !done(process) {
        assert!(Instant::now() < deadline, "server polling timed out");
        assert_eq!(process.poll().unwrap(), ProcessPoll::Running);
        thread::sleep(Duration::from_millis(5));
    }
}

fn free_addresses() -> [SocketAddr; 3] {
    let listeners = [
        TcpListener::bind("127.0.0.1:0").unwrap(),
        TcpListener::bind("127.0.0.1:0").unwrap(),
        TcpListener::bind("127.0.0.1:0").unwrap(),
    ];
    listeners.map(|listener| listener.local_addr().unwrap())
}
