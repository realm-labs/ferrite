//! Multi-process development topology orchestration.

use crate::http;
use ferrite_server_runtime::config::ServerConfig;
use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(12);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

pub(crate) struct DevArguments {
    nodes: u16,
    base_port: u16,
    state_dir: Option<PathBuf>,
    server_bin: Option<PathBuf>,
    shutdown_after: Option<Duration>,
}

impl DevArguments {
    pub(crate) fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, Box<dyn Error>> {
        let mut nodes = None;
        let mut base_port = 27_000_u16;
        let mut state_dir = None;
        let mut server_bin = None;
        let mut shutdown_after = None;
        let mut arguments = arguments.peekable();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--nodes" => {
                    nodes = Some(parse_next::<u16>(&mut arguments, "--nodes")?);
                }
                "--base-port" => {
                    base_port = parse_next::<u16>(&mut arguments, "--base-port")?;
                }
                "--state-dir" => {
                    state_dir = Some(PathBuf::from(next_value(&mut arguments, "--state-dir")?));
                }
                "--server-bin" => {
                    server_bin = Some(PathBuf::from(next_value(&mut arguments, "--server-bin")?));
                }
                "--shutdown-after-ms" => {
                    let millis = parse_next::<u64>(&mut arguments, "--shutdown-after-ms")?;
                    shutdown_after = Some(Duration::from_millis(millis));
                }
                "--help" | "-h" => {
                    crate::print_help();
                    std::process::exit(0);
                }
                _ => return Err(format!("unknown dev argument: {argument}").into()),
            }
        }
        Ok(Self {
            nodes: nodes.ok_or("--nodes is required")?,
            base_port,
            state_dir,
            server_bin,
            shutdown_after,
        })
    }
}

pub(crate) fn run(arguments: DevArguments) -> Result<(), Box<dyn Error>> {
    let state = StateDirectory::create(arguments.state_dir)?;
    let nodes = build_nodes(arguments.nodes, arguments.base_port, state.path())?;
    let server = resolve_server_binary(arguments.server_bin)?;
    let mut children = spawn_nodes(&server, &nodes)?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&shutdown);
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;

    if let Err(error) = wait_ready(&mut children, &nodes) {
        force_stop(&mut children);
        return Err(error);
    }
    println!(
        "ferrite development cluster ready: nodes={} state={}",
        nodes.len(),
        state.path().display()
    );

    let automatic_deadline = arguments.shutdown_after.map(|after| Instant::now() + after);
    loop {
        if shutdown.load(Ordering::Acquire)
            || automatic_deadline.is_some_and(|deadline| Instant::now() >= deadline)
        {
            break;
        }
        fail_if_exited(&mut children)?;
        thread::sleep(POLL_INTERVAL);
    }
    graceful_stop(&mut children, &nodes)?;
    println!("ferrite development cluster stopped cleanly");
    Ok(())
}

struct NodeLaunch {
    config_path: PathBuf,
    management: SocketAddr,
}

fn build_nodes(
    count: u16,
    base_port: u16,
    state_root: &Path,
) -> Result<Vec<NodeLaunch>, Box<dyn Error>> {
    if count == 0 {
        return Err("development cluster must contain at least one node".into());
    }
    let config_root = state_root.join("config");
    fs::create_dir_all(&config_root)?;
    (1..=count)
        .map(|index| {
            let config = ServerConfig::development_node(index, count, base_port, state_root)?;
            let config_path = config_root.join(format!("node-{index}.toml"));
            fs::write(&config_path, config.to_toml()?)?;
            Ok(NodeLaunch {
                config_path,
                management: config.management.bind,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()
}

fn resolve_server_binary(explicit: Option<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(path) = explicit {
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("ferrite-server binary does not exist: {}", path.display()).into());
    }
    let executable_name = if cfg!(windows) {
        "ferrite-server.exe"
    } else {
        "ferrite-server"
    };
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("cannot resolve workspace root")?;
    if !workspace.join("Cargo.toml").is_file() {
        return std::env::current_exe()?
            .parent()
            .map(|parent| parent.join(executable_name))
            .filter(|path| path.is_file())
            .ok_or_else(|| "packaged ferrite-server sibling is missing".into());
    }
    let status = Command::new("cargo")
        .args(["build", "-q", "-p", "ferrite-server"])
        .current_dir(workspace)
        .status()?;
    if !status.success() {
        return Err("cargo build -p ferrite-server failed".into());
    }
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                workspace.join(path)
            }
        })
        .unwrap_or_else(|| workspace.join("target"));
    let binary = target.join("debug").join(executable_name);
    if !binary.is_file() {
        return Err(format!(
            "built ferrite-server binary is missing: {}",
            binary.display()
        )
        .into());
    }
    Ok(binary)
}

fn spawn_nodes(server: &Path, nodes: &[NodeLaunch]) -> Result<Vec<Child>, Box<dyn Error>> {
    let mut children = Vec::with_capacity(nodes.len());
    for node in nodes {
        match Command::new(server)
            .arg("--config")
            .arg(&node.config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(child) => children.push(child),
            Err(error) => {
                force_stop(&mut children);
                return Err(error.into());
            }
        }
    }
    Ok(children)
}

fn wait_ready(children: &mut [Child], nodes: &[NodeLaunch]) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        fail_if_exited(children)?;
        let ready = nodes
            .iter()
            .filter(|node| matches!(http::status(node.management, "/readyz"), Ok(200)))
            .count();
        if ready == nodes.len() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "development cluster readiness timed out: {ready}/{} nodes ready",
                nodes.len()
            )
            .into());
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn fail_if_exited(children: &mut [Child]) -> Result<(), Box<dyn Error>> {
    for child in children {
        if let Some(status) = child.try_wait()? {
            return Err(format!("ferrite-server process exited unexpectedly: {status}").into());
        }
    }
    Ok(())
}

fn graceful_stop(children: &mut [Child], nodes: &[NodeLaunch]) -> Result<(), Box<dyn Error>> {
    for node in nodes {
        match http::drain(node.management) {
            Ok(202) | Err(_) => {}
            Ok(status) => return Err(format!("node drain returned HTTP {status}").into()),
        }
    }
    let deadline = Instant::now() + SHUTDOWN_TIMEOUT;
    let mut complete = vec![false; children.len()];
    while complete.iter().any(|stopped| !stopped) && Instant::now() < deadline {
        for (index, child) in children.iter_mut().enumerate() {
            if !complete[index] && child.try_wait()?.is_some() {
                complete[index] = true;
            }
        }
        thread::sleep(POLL_INTERVAL);
    }
    if complete.iter().any(|stopped| !stopped) {
        force_stop(children);
        return Err("one or more nodes exceeded the graceful shutdown deadline".into());
    }
    Ok(())
}

fn force_stop(children: &mut [Child]) {
    for child in children {
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

enum StateDirectory {
    Temporary(TempDir),
    Explicit(PathBuf),
}

impl StateDirectory {
    fn create(path: Option<PathBuf>) -> Result<Self, std::io::Error> {
        match path {
            Some(path) => {
                fs::create_dir_all(&path)?;
                Ok(Self::Explicit(path))
            }
            None => tempfile::Builder::new()
                .prefix("ferrite-dev-")
                .tempdir()
                .map(Self::Temporary),
        }
    }

    fn path(&self) -> &Path {
        match self {
            Self::Temporary(directory) => directory.path(),
            Self::Explicit(path) => path,
        }
    }
}

fn next_value(
    arguments: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn Error>> {
    arguments
        .next()
        .ok_or_else(|| format!("{flag} requires a value").into())
}

fn parse_next<T>(
    arguments: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<T, Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
{
    Ok(next_value(arguments, flag)?.parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_server_runtime::config::ConfigError;

    #[test]
    fn development_node_count_and_port_range_are_validated() {
        let state = tempfile::tempdir().unwrap();
        assert!(build_nodes(0, 27_000, state.path()).is_err());
        assert!(matches!(
            ServerConfig::development_node(1, 65, 27_000, state.path()),
            Err(ConfigError::Invalid(_))
        ));
        assert!(build_nodes(3, u16::MAX - 2, state.path()).is_err());
    }
}
