use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use ferrite_persistence::store::RegionFileStore;
use ferrite_server_runtime::config::{AdvertisedAddress, DiscoveryConfig, ServerConfig};
use ferrite_server_runtime::lifecycle::NodePhase;
use ferrite_server_runtime::process::{NodeProcess, ProcessPoll};

const TIMEOUT: Duration = Duration::from_secs(10);

#[test]
fn clean_shutdown_flushes_formal_regions_and_restart_resumes_the_checkpoint() {
    let temporary = tempfile::tempdir().unwrap();
    let config = configured_server(temporary.path(), 2);
    let config_text = config.to_toml().unwrap();

    let saved_tick = run_until_saved_and_stop(&config_text);
    let store = RegionFileStore::open(control_store(&config)).unwrap();
    let point = store
        .load_named(1, "minecraft:overworld", 0, 0, 1)
        .unwrap()
        .unwrap();
    assert_eq!(point.committed_tick(), saved_tick);
    let domains = point
        .snapshot()
        .records()
        .iter()
        .map(|record| record.domain().to_string())
        .collect::<Vec<_>>();
    assert!(
        domains
            .iter()
            .any(|domain| domain == "ferrite:world-service/world_v1")
    );
    assert!(
        domains
            .iter()
            .any(|domain| domain == "ferrite:world-service/level_v1")
    );
    assert!(
        domains
            .iter()
            .any(|domain| domain == "ferrite:simulation/runtime_v1")
    );

    let validated = ServerConfig::from_toml(&config_text).unwrap();
    let mut restarted = NodeProcess::start(validated).unwrap();
    assert_eq!(restarted.minecraft_committed_tick(), Some(saved_tick));
    poll_until(&mut restarted, |process| {
        process.lifecycle().snapshot().unwrap().phase == NodePhase::Ready
    });
    drain_and_stop(restarted);
}

#[test]
fn corrupt_formal_control_store_prevents_restart() {
    let temporary = tempfile::tempdir().unwrap();
    let config = configured_server(temporary.path(), 2);
    let config_text = config.to_toml().unwrap();
    run_until_saved_and_stop(&config_text);

    let data = control_store(&config).join("region-data.log");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(data)
        .unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    file.write_all(b"X").unwrap();
    file.sync_all().unwrap();

    let validated = ServerConfig::from_toml(&config_text).unwrap();
    let error = match NodeProcess::start(validated) {
        Ok(_) => panic!("corrupt durable world must not start"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("Minecraft local-world bootstrap failed")
    );
}

fn configured_server(root: &Path, autosave_ticks: u64) -> ServerConfig {
    let [remoting, management, minecraft] = free_addresses();
    let mut config = ServerConfig::development_node(1, 1, 30_000, root).unwrap();
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
    config.world.save.autosave_interval_ticks = autosave_ticks;
    config
}

fn run_until_saved_and_stop(config_text: &str) -> u64 {
    let validated = ServerConfig::from_toml(config_text).unwrap();
    let mut process = NodeProcess::start(validated).unwrap();
    poll_until(&mut process, |process| {
        process.lifecycle().snapshot().unwrap().phase == NodePhase::Ready
    });
    poll_until(&mut process, |process| {
        process
            .minecraft_committed_tick()
            .is_some_and(|tick| tick >= 2)
    });
    process.begin_drain().unwrap();
    while process.poll().unwrap() != ProcessPoll::Drained {
        thread::sleep(Duration::from_millis(5));
    }
    let tick = process.minecraft_committed_tick().unwrap();
    process.stop().unwrap();
    tick
}

fn drain_and_stop(mut process: NodeProcess) {
    process.begin_drain().unwrap();
    let deadline = Instant::now() + TIMEOUT;
    while process.poll().unwrap() != ProcessPoll::Drained {
        assert!(Instant::now() < deadline, "server drain timed out");
        thread::sleep(Duration::from_millis(5));
    }
    process.stop().unwrap();
}

fn poll_until(process: &mut NodeProcess, done: impl Fn(&NodeProcess) -> bool) {
    let deadline = Instant::now() + TIMEOUT;
    while !done(process) {
        assert!(Instant::now() < deadline, "server polling timed out");
        assert_eq!(process.poll().unwrap(), ProcessPoll::Running);
        thread::sleep(Duration::from_millis(5));
    }
}

fn control_store(config: &ServerConfig) -> PathBuf {
    config
        .storage
        .root
        .join("worlds/00000000000000000000000000000001")
        .join("dimensions/minecraft/overworld/regions/r.0.0")
}

fn free_addresses() -> [SocketAddr; 3] {
    let listeners = [
        TcpListener::bind("127.0.0.1:0").unwrap(),
        TcpListener::bind("127.0.0.1:0").unwrap(),
        TcpListener::bind("127.0.0.1:0").unwrap(),
    ];
    listeners.map(|listener| listener.local_addr().unwrap())
}
