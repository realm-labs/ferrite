//! Minimal bounded HTTP management listener for health, readiness, status, and drain.

use crate::config::ManagementConfig;
use crate::lifecycle::NodeLifecycle;
use serde::Serialize;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

const ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

pub struct ManagementServer {
    local_address: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ManagementServer {
    pub fn bind(
        config: &ManagementConfig,
        maximum_request_bytes: usize,
        lifecycle: Arc<NodeLifecycle>,
    ) -> Result<Self, ManagementError> {
        let listener = TcpListener::bind(config.bind)?;
        listener.set_nonblocking(true)?;
        let local_address = listener.local_addr()?;
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let allow_remote_drain = config.allow_remote_drain;
        let thread = thread::Builder::new()
            .name("ferrite-management".to_owned())
            .spawn(move || {
                run_listener(
                    listener,
                    maximum_request_bytes,
                    allow_remote_drain,
                    lifecycle,
                    thread_stop,
                );
            })?;
        Ok(Self {
            local_address,
            stop,
            thread: Some(thread),
        })
    }

    pub const fn local_address(&self) -> SocketAddr {
        self.local_address
    }

    pub fn stop(mut self) -> Result<(), ManagementError> {
        self.stop.store(true, Ordering::Release);
        self.join()
    }

    fn join(&mut self) -> Result<(), ManagementError> {
        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| ManagementError::ThreadPanicked)?;
        }
        Ok(())
    }
}

impl Drop for ManagementServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.join();
    }
}

fn run_listener(
    listener: TcpListener,
    maximum_request_bytes: usize,
    allow_remote_drain: bool,
    lifecycle: Arc<NodeLifecycle>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, peer)) => {
                let _ = handle_connection(
                    stream,
                    peer,
                    maximum_request_bytes,
                    allow_remote_drain,
                    &lifecycle,
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(ACCEPT_POLL_INTERVAL);
            }
            Err(_) => break,
        }
    }
}

fn handle_connection(
    mut stream: TcpStream,
    peer: SocketAddr,
    maximum_request_bytes: usize,
    allow_remote_drain: bool,
    lifecycle: &NodeLifecycle,
) -> Result<(), ManagementError> {
    stream.set_read_timeout(Some(REQUEST_TIMEOUT))?;
    stream.set_write_timeout(Some(REQUEST_TIMEOUT))?;
    let request = read_request(&mut stream, maximum_request_bytes)?;
    let response = route_request(&request, peer, allow_remote_drain, lifecycle);
    stream.write_all(&response)?;
    stream.flush()?;
    Ok(())
}

fn read_request(
    stream: &mut TcpStream,
    maximum_request_bytes: usize,
) -> Result<Vec<u8>, ManagementError> {
    let mut request = Vec::with_capacity(maximum_request_bytes.min(1_024));
    let mut chunk = [0_u8; 512];
    loop {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Err(ManagementError::MalformedRequest);
        }
        if request.len().saturating_add(count) > maximum_request_bytes {
            return Err(ManagementError::RequestTooLarge);
        }
        request.extend_from_slice(&chunk[..count]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            return Ok(request);
        }
    }
}

fn route_request(
    request: &[u8],
    peer: SocketAddr,
    allow_remote_drain: bool,
    lifecycle: &NodeLifecycle,
) -> Vec<u8> {
    let Some((method, path)) = parse_request_line(request) else {
        return response(400, "Bad Request", &ErrorBody::new("malformed request"));
    };
    match (method, path) {
        ("GET", "/healthz") => lifecycle_response(lifecycle, ResponseKind::Health),
        ("GET", "/readyz") => lifecycle_response(lifecycle, ResponseKind::Ready),
        ("GET", "/status") => lifecycle_response(lifecycle, ResponseKind::Status),
        ("POST", "/drain") if allow_remote_drain || peer.ip().is_loopback() => {
            match lifecycle.begin_drain() {
                Ok(()) => lifecycle_response(lifecycle, ResponseKind::Drain),
                Err(error) => response(409, "Conflict", &ErrorBody::new(error.to_string())),
            }
        }
        ("POST", "/drain") => response(
            403,
            "Forbidden",
            &ErrorBody::new("remote drain is disabled"),
        ),
        (_, "/healthz" | "/readyz" | "/status" | "/drain") => response(
            405,
            "Method Not Allowed",
            &ErrorBody::new("method not allowed"),
        ),
        _ => response(404, "Not Found", &ErrorBody::new("not found")),
    }
}

fn lifecycle_response(lifecycle: &NodeLifecycle, kind: ResponseKind) -> Vec<u8> {
    let snapshot = match lifecycle.snapshot() {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return response(
                500,
                "Internal Server Error",
                &ErrorBody::new(error.to_string()),
            );
        }
    };
    let (status, reason) = match kind {
        ResponseKind::Health if !snapshot.healthy => (503, "Service Unavailable"),
        ResponseKind::Ready if !snapshot.ready => (503, "Service Unavailable"),
        ResponseKind::Drain => (202, "Accepted"),
        _ => (200, "OK"),
    };
    response(status, reason, &snapshot)
}

fn parse_request_line(request: &[u8]) -> Option<(&str, &str)> {
    let request = std::str::from_utf8(request).ok()?;
    let line = request.split_once("\r\n")?.0;
    let mut parts = line.split(' ');
    let method = parts.next()?;
    let path = parts.next()?;
    let version = parts.next()?;
    if parts.next().is_some() || version != "HTTP/1.1" {
        return None;
    }
    Some((method, path))
}

fn response(status: u16, reason: &str, body: &impl Serialize) -> Vec<u8> {
    let body = serde_json::to_vec(body)
        .unwrap_or_else(|_| br#"{"error":"response serialization failed"}"#.to_vec());
    let headers = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let mut response = headers.into_bytes();
    response.extend_from_slice(&body);
    response
}

#[derive(Debug, Clone, Copy)]
enum ResponseKind {
    Health,
    Ready,
    Status,
    Drain,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl ErrorBody {
    fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ManagementError {
    #[error("management I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("management request exceeded its configured bound")]
    RequestTooLarge,
    #[error("management request was malformed")]
    MalformedRequest,
    #[error("management listener thread panicked")]
    ThreadPanicked,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::NodePhase;
    use std::collections::BTreeSet;

    #[test]
    fn health_precedes_readiness_and_local_drain_closes_it() {
        let lifecycle = Arc::new(NodeLifecycle::new(BTreeSet::from([
            "ferrite-region-v1".to_owned()
        ])));
        let server = ManagementServer::bind(
            &ManagementConfig {
                bind: "127.0.0.1:0".parse().unwrap(),
                allow_remote_drain: false,
            },
            4_096,
            Arc::clone(&lifecycle),
        )
        .unwrap();

        assert!(request(server.local_address(), "GET /healthz").starts_with("HTTP/1.1 200"));
        assert!(request(server.local_address(), "GET /readyz").starts_with("HTTP/1.1 503"));
        lifecycle.mark_membership_ready().unwrap();
        assert_eq!(
            lifecycle.snapshot().unwrap().phase,
            NodePhase::AwaitingPlacement
        );
        lifecycle
            .mark_placement_domain_ready("ferrite-region-v1")
            .unwrap();
        assert!(request(server.local_address(), "GET /readyz").starts_with("HTTP/1.1 200"));
        assert!(request(server.local_address(), "POST /drain").starts_with("HTTP/1.1 202"));
        assert!(request(server.local_address(), "GET /readyz").starts_with("HTTP/1.1 503"));
        server.stop().unwrap();
    }

    fn request(address: SocketAddr, line: &str) -> String {
        let mut stream = TcpStream::connect(address).unwrap();
        write!(stream, "{line} HTTP/1.1\r\nHost: ferrite\r\n\r\n").unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }
}
