//! Immutable server-process shell around deployment endpoints and lifecycle gates.

use crate::config::{DiscoveryConfig, NodeRole, ValidatedServerConfig};
use crate::lifecycle::{LifecycleError, NodeLifecycle, NodePhase};
use crate::management::{ManagementError, ManagementServer};
use std::fs;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

const DEVELOPMENT_CONNECT_TIMEOUT: Duration = Duration::from_millis(30);
const MAX_PROBE_ACCEPTS_PER_POLL: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessPoll {
    Running,
    Drained,
}

pub struct NodeProcess {
    config: ValidatedServerConfig,
    lifecycle: Arc<NodeLifecycle>,
    management: Option<ManagementServer>,
    remoting_reservation: TcpListener,
    minecraft_listener: Option<TcpListener>,
}

impl NodeProcess {
    pub fn start(config: ValidatedServerConfig) -> Result<Self, ProcessError> {
        fs::create_dir_all(&config.config().storage.root)?;
        let remoting_reservation = TcpListener::bind(config.config().remoting.bind)?;
        remoting_reservation.set_nonblocking(true)?;
        let minecraft_listener = config
            .config()
            .minecraft
            .enabled
            .then(|| TcpListener::bind(config.config().minecraft.bind))
            .transpose()?;
        if let Some(listener) = &minecraft_listener {
            listener.set_nonblocking(true)?;
        }
        let lifecycle = Arc::new(NodeLifecycle::new(config.required_domains()));
        let management = ManagementServer::bind(
            &config.config().management,
            config.config().limits.max_management_request_bytes,
            Arc::clone(&lifecycle),
        )?;
        Ok(Self {
            config,
            lifecycle,
            management: Some(management),
            remoting_reservation,
            minecraft_listener,
        })
    }

    pub fn lifecycle(&self) -> Arc<NodeLifecycle> {
        Arc::clone(&self.lifecycle)
    }

    pub fn management_address(&self) -> Result<SocketAddr, ProcessError> {
        self.management
            .as_ref()
            .map(ManagementServer::local_address)
            .ok_or(ProcessError::AlreadyStopped)
    }

    pub fn poll(&mut self) -> Result<ProcessPoll, ProcessError> {
        self.drain_probe_connections()?;
        let phase = self.lifecycle.snapshot()?.phase;
        if phase == NodePhase::AwaitingMembership {
            self.try_bootstrap()?;
        }
        let phase = self.lifecycle.snapshot()?.phase;
        if matches!(phase, NodePhase::Draining | NodePhase::Drained) {
            self.minecraft_listener.take();
        }
        if phase == NodePhase::Drained {
            Ok(ProcessPoll::Drained)
        } else {
            Ok(ProcessPoll::Running)
        }
    }

    pub fn begin_drain(&self) -> Result<(), ProcessError> {
        self.lifecycle.begin_drain()?;
        Ok(())
    }

    pub fn stop(mut self) -> Result<(), ProcessError> {
        if self.lifecycle.snapshot()?.phase != NodePhase::Drained {
            self.lifecycle.begin_drain()?;
        }
        if self.lifecycle.snapshot()?.phase != NodePhase::Drained {
            return Err(ProcessError::DrainIncomplete);
        }
        self.minecraft_listener.take();
        if let Some(management) = self.management.take() {
            management.stop()?;
        }
        self.lifecycle.mark_stopped()?;
        Ok(())
    }

    fn drain_probe_connections(&self) -> Result<(), ProcessError> {
        for _ in 0..MAX_PROBE_ACCEPTS_PER_POLL {
            match self.remoting_reservation.accept() {
                Ok((_stream, _peer)) => {}
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(error) => return Err(ProcessError::Io(error)),
            }
        }
        Ok(())
    }

    fn try_bootstrap(&self) -> Result<(), ProcessError> {
        let (addresses, minimum_members) = match &self.config.config().discovery {
            DiscoveryConfig::DevelopmentStatic {
                peers,
                minimum_members,
            } => {
                let addresses = peers
                    .iter()
                    .flat_map(|peer| (peer.host.as_str(), peer.port).to_socket_addrs())
                    .flatten()
                    .collect::<Vec<_>>();
                (addresses, *minimum_members)
            }
            DiscoveryConfig::Kubernetes {
                namespace,
                service,
                remoting_port,
                minimum_members,
            } => {
                let host = format!("{service}.{namespace}.svc");
                let addresses = (host.as_str(), *remoting_port)
                    .to_socket_addrs()
                    .map_err(ProcessError::Io)?
                    .collect::<Vec<_>>();
                (addresses, *minimum_members)
            }
        };
        let reachable = addresses
            .into_iter()
            .filter(|address| {
                TcpStream::connect_timeout(address, DEVELOPMENT_CONNECT_TIMEOUT).is_ok()
            })
            .count();
        if reachable < minimum_members {
            return Ok(());
        }
        self.lifecycle.mark_membership_ready()?;
        if self
            .config
            .config()
            .node
            .roles
            .contains(&NodeRole::RegionWorker)
        {
            for domain in &self.config.config().placement.required_domains {
                self.lifecycle.mark_placement_domain_ready(domain)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("server process I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("server lifecycle failed: {0}")]
    Lifecycle(#[from] LifecycleError),
    #[error("management server failed: {0}")]
    Management(#[from] ManagementError),
    #[error("server process is already stopped")]
    AlreadyStopped,
    #[error("server process cannot stop before sessions, Region authorities, and commits drain")]
    DrainIncomplete,
}
