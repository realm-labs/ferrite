#![forbid(unsafe_code)]

//! Local cluster launcher and deployment inspection entry point.

mod dev;
mod http;
mod topology;

use crate::dev::{DevArguments, run};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    match arguments.next().as_deref() {
        Some("dev") => run(DevArguments::parse(arguments)?)?,
        Some("verify-topology") => topology::verify(topology::VerifyArguments::parse(arguments)?)?,
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
         ferrite-cluster verify-topology [--ticks <N>]\n\
         Proves local, in-process Lattice-envelope, and three-process convergence.\n\
         Options: --base-port <PORT> --state-dir <PATH> --server-bin <PATH> \
         --shutdown-after-ms <MILLIS>"
    );
}
