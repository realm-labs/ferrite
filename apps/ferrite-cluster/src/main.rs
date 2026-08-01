#![forbid(unsafe_code)]

//! Local cluster launcher and deployment inspection entry point.

mod capacity;
mod dev;
mod http;
mod playable;
mod topology;

use crate::dev::{DevArguments, run};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    match arguments.next().as_deref() {
        Some("dev") => run(DevArguments::parse(arguments)?)?,
        Some("capacity") => capacity::run(arguments)?,
        Some("capacity-worker") => capacity::worker(arguments)?,
        Some("verify-playable") => playable::verify(arguments)?,
        Some("verify-replay") => playable::verify_replay(arguments)?,
        Some("playable-worker") => playable::worker(arguments)?,
        Some("replay-worker") => playable::replay_worker(arguments)?,
        Some("verify-topology") => topology::verify(topology::VerifyArguments::parse(arguments)?)?,
        Some("verify-faults") => topology::verify_faults()?,
        Some("topology-worker") => topology::worker(arguments)?,
        Some("--help" | "-h") => print_help(),
        Some(command) => return Err(format!("unknown ferrite-cluster command: {command}").into()),
        None => return Err("a ferrite-cluster command is required; use --help".into()),
    }
    Ok(())
}

fn print_help() {
    println!(
        "Usage: ferrite-cluster dev --nodes <N> [options]\n\
         \n\
         Starts N ferrite-server processes from ephemeral local configuration and drains them on Ctrl+C.\n\
         \n\
         ferrite-cluster capacity <verify|benchmark> [--profile <NAME>] [--output <PATH>]\n\
         Validates named capacity profiles or records their synthetic Region benchmark report.\n\
         \n\
         ferrite-cluster verify-topology [--ticks <N>]\n\
         Proves local, in-process Lattice-envelope, and three-process convergence.\n\
         \n\
         ferrite-cluster verify-faults\n\
         Runs the three-process crash, network, control-plane, handoff, restart, and upgrade matrix.\n\
         \n\
         ferrite-cluster verify-playable\n\
         Proves equal C2 committed hashes and packet traces across local, Lattice, and process boundaries.\n\
         \n\
         ferrite-cluster verify-replay\n\
         Replays one canonical C2 log across local, Lattice, and process boundaries.\n\
         Options: --base-port <PORT> --state-dir <PATH> --server-bin <PATH> \
         --shutdown-after-ms <MILLIS>"
    );
}
