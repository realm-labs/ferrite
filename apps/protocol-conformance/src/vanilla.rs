use std::fs::{self, File};
use std::io::Read;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use sha1::{Digest, Sha1};

use crate::DynError;
use crate::fixture::{CLIENT_JAR_SHA1, vanilla_settings};
use crate::smoke::{ConnectionObservation, serve_connection, serve_playable_connection};

const CLIENT_JAR_SIZE: u64 = 39_193_383;

pub(crate) struct VanillaProbe {
    pub(crate) client_jar: PathBuf,
    pub(crate) registry_report: PathBuf,
    pub(crate) bind: String,
    pub(crate) timeout: Duration,
    pub(crate) evidence: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VanillaProbeReport {
    schema: &'static str,
    minecraft_version: &'static str,
    client_jar_sha1: &'static str,
    endpoint: String,
    status_observed: bool,
    login_configuration_observed: bool,
    play_teleport_acknowledged: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct VanillaPlayableProbeReport {
    schema: &'static str,
    minecraft_version: &'static str,
    client_jar_sha1: &'static str,
    endpoint: String,
    login_configuration_observed: bool,
    play_teleport_acknowledged: bool,
    chunk_batch_feedback_observed: bool,
    player_loaded_observed: bool,
    movement_observed: bool,
    client_tick_end_observed: bool,
}

impl VanillaProbeReport {
    pub(crate) fn summary(&self) -> String {
        format!(
            "unmodified 26.2 probe passed at {}: status={}, login/config={}, play-ack={}",
            self.endpoint,
            self.status_observed,
            self.login_configuration_observed,
            self.play_teleport_acknowledged
        )
    }
}

impl VanillaPlayableProbeReport {
    pub(crate) fn summary(&self) -> String {
        format!(
            "unmodified 26.2 C2 probe passed at {}: login/config={}, play-ack={}, batch={}, loaded={}, movement={}, tick-end={}",
            self.endpoint,
            self.login_configuration_observed,
            self.play_teleport_acknowledged,
            self.chunk_batch_feedback_observed,
            self.player_loaded_observed,
            self.movement_observed,
            self.client_tick_end_observed
        )
    }
}

pub(crate) fn run(probe: VanillaProbe) -> Result<VanillaProbeReport, DynError> {
    let (endpoint, combined) = observe(&probe, false)?;
    if !combined.play_acknowledged {
        return Err("exact 26.2 client did not acknowledge the Play entry teleport".into());
    }
    let report = VanillaProbeReport {
        schema: "ferrite-unmodified-client-smoke-v1",
        minecraft_version: "26.2",
        client_jar_sha1: CLIENT_JAR_SHA1,
        endpoint,
        status_observed: combined.status_complete,
        login_configuration_observed: combined.login_complete,
        play_teleport_acknowledged: combined.play_acknowledged,
    };
    write_evidence(&probe.evidence, &report)?;
    Ok(report)
}

pub(crate) fn run_playable(probe: VanillaProbe) -> Result<VanillaPlayableProbeReport, DynError> {
    let (endpoint, combined) = observe(&probe, true)?;
    if !playable_complete(&combined) {
        return Err(
            format!("exact 26.2 client did not complete the C2 observation: {combined:?}").into(),
        );
    }
    let report = VanillaPlayableProbeReport {
        schema: "ferrite-unmodified-client-c2-smoke-v1",
        minecraft_version: "26.2",
        client_jar_sha1: CLIENT_JAR_SHA1,
        endpoint,
        login_configuration_observed: combined.login_complete,
        play_teleport_acknowledged: combined.play_acknowledged,
        chunk_batch_feedback_observed: combined.chunk_batch_received,
        player_loaded_observed: combined.player_loaded,
        movement_observed: combined.movement_observed,
        client_tick_end_observed: combined.client_tick_end,
    };
    write_evidence(&probe.evidence, &report)?;
    Ok(report)
}

fn observe(
    probe: &VanillaProbe,
    playable: bool,
) -> Result<(String, ConnectionObservation), DynError> {
    verify_client(&probe.client_jar)?;
    let settings = vanilla_settings(&probe.registry_report)?;
    let listener = TcpListener::bind(&probe.bind)?;
    listener.set_nonblocking(true)?;
    let endpoint = listener.local_addr()?.to_string();
    println!(
        "verified unmodified client.jar SHA-1; connect Minecraft 26.2 to {endpoint} \
         within {} seconds",
        probe.timeout.as_secs()
    );

    let deadline = Instant::now() + probe.timeout;
    let mut combined = ConnectionObservation::default();
    while Instant::now() < deadline
        && if playable {
            !playable_complete(&combined)
        } else {
            !combined.play_acknowledged
        }
    {
        match listener.accept() {
            Ok((stream, _)) => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                let observation = if playable {
                    serve_playable_connection(stream, settings.clone(), remaining)?
                } else {
                    serve_connection(stream, settings.clone(), true, remaining)?
                };
                println!("observed client connection: {observation:?}");
                if !observation.status_complete
                    && (observation.peer_closed || observation.close_reason.is_some())
                {
                    return Err(format!(
                        "exact 26.2 client closed before the required Play observation: {observation:?}"
                    )
                    .into());
                }
                merge_observation(&mut combined, observation);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok((endpoint, combined))
}

fn merge_observation(combined: &mut ConnectionObservation, observation: ConnectionObservation) {
    combined.status_complete |= observation.status_complete;
    combined.login_complete |= observation.login_complete;
    combined.play_acknowledged |= observation.play_acknowledged;
    combined.chunk_batch_received |= observation.chunk_batch_received;
    combined.player_loaded |= observation.player_loaded;
    combined.movement_observed |= observation.movement_observed;
    combined.client_tick_end |= observation.client_tick_end;
}

fn playable_complete(observation: &ConnectionObservation) -> bool {
    observation.login_complete
        && observation.play_acknowledged
        && observation.chunk_batch_received
        && observation.player_loaded
        && observation.movement_observed
        && observation.client_tick_end
}

fn verify_client(path: &PathBuf) -> Result<(), DynError> {
    let metadata = fs::metadata(path)?;
    if metadata.len() != CLIENT_JAR_SIZE {
        return Err(format!(
            "client.jar size mismatch: expected {CLIENT_JAR_SIZE}, found {}",
            metadata.len()
        )
        .into());
    }
    let mut file = File::open(path)?;
    let mut digest = Sha1::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        digest.update(&buffer[..length]);
    }
    let actual = format!("{:x}", digest.finalize());
    if actual != CLIENT_JAR_SHA1 {
        return Err(format!(
            "client.jar SHA-1 mismatch: expected {CLIENT_JAR_SHA1}, found {actual}"
        )
        .into());
    }
    Ok(())
}

fn write_evidence(path: &PathBuf, report: &impl Serialize) -> Result<(), DynError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, toml::to_string_pretty(report)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrong_client_artifact_fails_before_listener_start() {
        let path =
            std::env::temp_dir().join(format!("ferrite-wrong-client-{}.jar", std::process::id()));
        fs::write(&path, b"not a client").unwrap();
        assert!(verify_client(&path).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn locked_client_and_registry_report_build_the_probe_fixture() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let client = workspace.join("target/mc-reference/26.2/client.jar");
        let registries =
            workspace.join("target/mc-reference/26.2/generated/reports/registries.json");
        verify_client(&client).unwrap();
        vanilla_settings(&registries).unwrap();
    }
}
