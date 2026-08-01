//! Process-boundary coordinator for the C2 playable conformance scenario.

use ferrite_server_runtime::conformance::playable::{
    PlayableScenarioEvidence, PlayableTopology, run_playable_scenario,
};
use ferrite_server_runtime::conformance::playable_replay::{
    PlayableReplayEvidence, verify_playable_replay,
};
use std::error::Error;
use std::process::{Command, Stdio};

const PROCESS_RUNNERS: usize = 3;

pub(crate) fn verify(arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    reject_arguments(arguments)?;
    let local = run_playable_scenario(PlayableTopology::Local)?;
    let lattice = run_playable_scenario(PlayableTopology::LatticeInProcess)?;
    require_equal(&local, &lattice, "in-process Lattice")?;
    for index in 0..PROCESS_RUNNERS {
        let process = run_process_worker()?;
        require_equal(
            &local,
            &process,
            &format!("process-isolated Lattice runner {index}"),
        )?;
    }
    println!(
        "playable equivalence verified: ticks={} packets={} processes={} state={} trace={}",
        local.committed_tick,
        local.packet_trace.len(),
        PROCESS_RUNNERS,
        local.committed_hash,
        local.packet_trace_digest
    );
    Ok(())
}

pub(crate) fn worker(arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    reject_arguments(arguments)?;
    let evidence = run_playable_scenario(PlayableTopology::LatticeInProcess)?;
    serde_json::to_writer(std::io::stdout().lock(), &evidence)?;
    Ok(())
}

pub(crate) fn verify_replay(arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    reject_arguments(arguments)?;
    let local = verify_playable_replay(PlayableTopology::Local)?;
    let lattice = verify_playable_replay(PlayableTopology::LatticeInProcess)?;
    if local != lattice {
        return Err("local and in-process playable replay evidence diverged".into());
    }
    for index in 0..PROCESS_RUNNERS {
        let process = run_replay_worker()?;
        if process != local {
            return Err(format!("process-isolated replay runner {index} diverged").into());
        }
    }
    println!(
        "playable replay equivalence verified: frames={} bytes={} processes={} log={} state={}",
        local.frames,
        local.encoded_bytes,
        PROCESS_RUNNERS,
        local.log_digest,
        local.final_world_hash
    );
    Ok(())
}

pub(crate) fn replay_worker(arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    reject_arguments(arguments)?;
    let evidence = verify_playable_replay(PlayableTopology::LatticeInProcess)?;
    serde_json::to_writer(std::io::stdout().lock(), &evidence)?;
    Ok(())
}

fn run_process_worker() -> Result<PlayableScenarioEvidence, Box<dyn Error>> {
    let output = Command::new(std::env::current_exe()?)
        .arg("playable-worker")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "playable worker exited with {}; stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn run_replay_worker() -> Result<PlayableReplayEvidence, Box<dyn Error>> {
    let output = Command::new(std::env::current_exe()?)
        .arg("replay-worker")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "replay worker exited with {}; stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn require_equal(
    expected: &PlayableScenarioEvidence,
    actual: &PlayableScenarioEvidence,
    runner: &str,
) -> Result<(), Box<dyn Error>> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{runner} playable committed state or packet trace diverged").into())
    }
}

fn reject_arguments(mut arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    match arguments.next() {
        Some(argument) => Err(format!("unexpected playable argument: {argument}").into()),
        None => Ok(()),
    }
}
