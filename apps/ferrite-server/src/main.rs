#![forbid(unsafe_code)]

//! Ferrite server process entry point.

use ferrite_server_runtime::config::ServerConfig;
use ferrite_server_runtime::lifecycle::NodePhase;
use ferrite_server_runtime::process::{NodeProcess, ProcessPoll};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::parse(std::env::args().skip(1))?;
    let config = ServerConfig::load(&arguments.config)?;
    let identity = config.node_identity()?;
    if arguments.check {
        println!(
            "configuration valid: cluster={} node={} advertise={}:{}",
            config.config().cluster.name,
            identity.node_id(),
            identity.host(),
            identity.port()
        );
        return Ok(());
    }

    let shutdown = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&shutdown);
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let drain_timeout = Duration::from_millis(config.config().shutdown.drain_timeout_millis);
    let node_id = config.config().node.id.clone();
    let mut process = NodeProcess::start(config)?;
    let minecraft = process
        .minecraft_address()
        .map_or_else(|| "disabled".to_owned(), |address| address.to_string());
    println!(
        "ferrite-server node={node_id} management={} minecraft={minecraft}",
        process.management_address()?
    );

    let mut drain_deadline = None;
    loop {
        if shutdown.swap(false, Ordering::AcqRel) {
            process.begin_drain()?;
        }
        match process.poll()? {
            ProcessPoll::Drained => break,
            ProcessPoll::Running => {}
        }
        let phase = process.lifecycle().snapshot()?.phase;
        if phase == NodePhase::Draining {
            let deadline = drain_deadline.get_or_insert_with(|| Instant::now() + drain_timeout);
            if Instant::now() >= *deadline {
                process
                    .lifecycle()
                    .fail("graceful drain deadline exceeded")?;
                return Err("graceful drain deadline exceeded".into());
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    process.stop()?;
    println!("ferrite-server node={node_id} stopped");
    Ok(())
}

struct Arguments {
    config: PathBuf,
    check: bool,
}

impl Arguments {
    fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, Box<dyn Error>> {
        let mut config = None;
        let mut check = false;
        let mut arguments = arguments.peekable();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--config" => {
                    let value = arguments.next().ok_or("--config requires a path")?;
                    config = Some(PathBuf::from(value));
                }
                "--check-config" => check = true,
                "--help" | "-h" => {
                    println!(
                        "Usage: ferrite-server --config <path> [--check-config]\n\
                         Runs the immutable Ferrite node process described by the versioned TOML file."
                    );
                    std::process::exit(0);
                }
                _ => return Err(format!("unknown argument: {argument}").into()),
            }
        }
        Ok(Self {
            config: config.ok_or("--config is required")?,
            check,
        })
    }
}
