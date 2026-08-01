//! Three-process topology conformance coordinator and worker protocol.

mod fault;

use ferrite_region_runtime::lattice::remoting::{LatticeRemotingAdapter, LatticeTransportFrame};
use ferrite_region_runtime::topology::cluster::{
    TopologyCluster, TopologyTransport, digest_snapshots,
};
use ferrite_region_runtime::topology::layout::TopologyLayout;
use ferrite_region_runtime::topology::partition::{
    TopologyPartition, TopologyPartitionSnapshot, TopologyWireMessage,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const REGION_COUNT: u16 = 12;
const NODE_COUNT: u16 = 3;
const MAILBOX_CAPACITY: usize = 32;
const FRAME_LIMIT: usize = 16 * 1024;

pub(crate) struct VerifyArguments {
    ticks: u64,
}

impl VerifyArguments {
    pub(crate) fn parse(
        mut arguments: impl Iterator<Item = String>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut ticks = 10_000;
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--ticks" => {
                    ticks = arguments
                        .next()
                        .ok_or("--ticks requires a value")?
                        .parse()?;
                }
                _ => return Err(format!("unknown verify-topology argument: {argument}").into()),
            }
        }
        if ticks == 0 {
            return Err("--ticks must be positive".into());
        }
        Ok(Self { ticks })
    }
}

pub(crate) fn verify(arguments: VerifyArguments) -> Result<(), Box<dyn Error>> {
    let distributed_layout = TopologyLayout::ring(REGION_COUNT, NODE_COUNT)?;
    let local_layout = distributed_layout.with_all_on_node(0)?;
    let mut local = TopologyCluster::seeded(
        local_layout,
        MAILBOX_CAPACITY,
        TopologyTransport::SemanticLocal,
    )?;
    let mut in_process = TopologyCluster::seeded(
        distributed_layout.clone(),
        MAILBOX_CAPACITY,
        TopologyTransport::LatticeInProcess,
    )?;
    local.run_to(arguments.ticks)?;
    in_process.run_to(arguments.ticks)?;

    let multi_process = run_multi_process(&distributed_layout, arguments.ticks)?;
    let local_digest = local.digest();
    if local_digest != in_process.digest() || local_digest != multi_process {
        return Err("local, in-process, and multi-process topology hashes diverged".into());
    }
    println!(
        "topology equivalence verified: ticks={} regions={} nodes={} digest={}",
        arguments.ticks,
        REGION_COUNT,
        NODE_COUNT,
        hex(&local_digest)
    );
    Ok(())
}

pub(crate) fn verify_faults() -> Result<(), Box<dyn Error>> {
    fault::verify()
}

fn run_multi_process(layout: &TopologyLayout, final_tick: u64) -> Result<[u8; 32], Box<dyn Error>> {
    let directory = tempfile::Builder::new()
        .prefix("ferrite-topology-")
        .tempdir()?;
    let layout_path = directory.path().join("layout.json");
    fs::write(&layout_path, serde_json::to_vec(layout)?)?;
    let mut workers = (0..layout.node_count())
        .map(|node| NodeWorker::spawn(node, &layout_path, 1))
        .collect::<Result<Vec<_>, _>>()?;
    let adapter = LatticeRemotingAdapter::new(FRAME_LIMIT)?;

    for value in 1..=final_tick {
        let mut messages = Vec::with_capacity(layout.len());
        for worker in &mut workers {
            match worker.request(WorkerRequest::Emit { tick: value })? {
                WorkerResponse::Messages(emitted) => messages.extend(emitted),
                response => return Err(unexpected("messages", response)),
            }
        }
        messages.reverse();
        let mut routed = BTreeMap::<u16, Vec<TopologyWireMessage>>::new();
        for message in messages {
            let frame = LatticeTransportFrame::from_transport_payload(message.bytes.clone());
            let envelope = adapter.decode(&frame)?;
            let node = layout.descriptor(envelope.target())?.node;
            routed.entry(node).or_default().push(message);
        }
        for worker in &mut workers {
            let messages = routed.remove(&worker.node).unwrap_or_default();
            expect_ack(worker.request(WorkerRequest::Admit {
                tick: value,
                messages,
            })?)?;
        }
        for worker in &mut workers {
            expect_ack(worker.request(WorkerRequest::Preflight { tick: value })?)?;
        }
        for worker in &mut workers {
            expect_ack(worker.request(WorkerRequest::Commit { tick: value })?)?;
        }
    }

    let mut snapshots = Vec::with_capacity(workers.len());
    for worker in &mut workers {
        match worker.request(WorkerRequest::Snapshot)? {
            WorkerResponse::Snapshot(snapshot) => snapshots.push(snapshot),
            response => return Err(unexpected("snapshot", response)),
        }
    }
    for worker in &mut workers {
        expect_ack(worker.request(WorkerRequest::Shutdown)?)?;
    }
    for worker in workers {
        worker.wait()?;
    }
    Ok(digest_snapshots(&snapshots))
}

pub(crate) fn worker(mut arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut node = None;
    let mut layout = None;
    let mut runtime_version = 1;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--node" => node = Some(arguments.next().ok_or("--node requires a value")?.parse()?),
            "--layout" => {
                layout = Some(PathBuf::from(
                    arguments.next().ok_or("--layout requires a path")?,
                ));
            }
            "--runtime-version" => {
                runtime_version = arguments
                    .next()
                    .ok_or("--runtime-version requires a value")?
                    .parse()?;
            }
            _ => return Err(format!("unknown topology-worker argument: {argument}").into()),
        }
    }
    let node = node.ok_or("--node is required")?;
    let layout: TopologyLayout =
        serde_json::from_slice(&fs::read(layout.ok_or("--layout is required")?)?)?;
    let adapter = LatticeRemotingAdapter::new(FRAME_LIMIT)?;
    let mut partition = TopologyPartition::seeded(node, layout, MAILBOX_CAPACITY)?;
    let input = std::io::stdin();
    let mut output = BufWriter::new(std::io::stdout().lock());
    for line in input.lock().lines() {
        let request: WorkerRequest = serde_json::from_str(&line?)?;
        let (response, shutdown) =
            handle_request(&mut partition, &adapter, runtime_version, request);
        serde_json::to_writer(&mut output, &response)?;
        output.write_all(b"\n")?;
        output.flush()?;
        if shutdown {
            return Ok(());
        }
    }
    Err("topology worker input closed without shutdown".into())
}

fn handle_request(
    partition: &mut TopologyPartition,
    adapter: &LatticeRemotingAdapter,
    runtime_version: u16,
    request: WorkerRequest,
) -> (WorkerResponse, bool) {
    let shutdown = matches!(&request, WorkerRequest::Shutdown);
    let response = match request {
        WorkerRequest::Emit { tick } => partition
            .emit_tick(tick, adapter)
            .map(WorkerResponse::Messages),
        WorkerRequest::Admit { tick, messages } => messages
            .into_iter()
            .try_for_each(|message| partition.admit_tick(message, tick, adapter).map(|_| ()))
            .map(|()| WorkerResponse::Ack),
        WorkerRequest::Preflight { tick } => partition
            .can_commit_tick(tick)
            .map(|()| WorkerResponse::Ack),
        WorkerRequest::Commit { tick } => partition.commit_tick(tick).map(|()| WorkerResponse::Ack),
        WorkerRequest::Snapshot => Ok(WorkerResponse::Snapshot(partition.snapshot())),
        WorkerRequest::Restore { layout, snapshot } => {
            TopologyPartition::restore(snapshot, layout, MAILBOX_CAPACITY).map(|restored| {
                *partition = restored;
                WorkerResponse::Ack
            })
        }
        WorkerRequest::Drain { target, move_id } => partition
            .begin_drain(target, u128::from(move_id))
            .map(|()| WorkerResponse::Ack),
        WorkerRequest::RuntimeVersion => Ok(WorkerResponse::RuntimeVersion(runtime_version)),
        WorkerRequest::Shutdown => Ok(WorkerResponse::Ack),
    }
    .unwrap_or_else(|error| WorkerResponse::Error(error.to_string()));
    (response, shutdown)
}

struct NodeWorker {
    node: u16,
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
}

impl NodeWorker {
    fn spawn(node: u16, layout: &Path, runtime_version: u16) -> Result<Self, Box<dyn Error>> {
        let mut child = Command::new(std::env::current_exe()?)
            .args(["topology-worker", "--node"])
            .arg(node.to_string())
            .arg("--layout")
            .arg(layout)
            .arg("--runtime-version")
            .arg(runtime_version.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child
            .stdin
            .take()
            .ok_or("topology worker stdin is missing")?;
        let output = child
            .stdout
            .take()
            .ok_or("topology worker stdout is missing")?;
        Ok(Self {
            node,
            child,
            input: BufWriter::new(input),
            output: BufReader::new(output),
        })
    }

    fn request(&mut self, request: WorkerRequest) -> Result<WorkerResponse, Box<dyn Error>> {
        serde_json::to_writer(&mut self.input, &request)?;
        self.input.write_all(b"\n")?;
        self.input.flush()?;
        let mut line = String::new();
        if self.output.read_line(&mut line)? == 0 {
            return Err(format!("topology worker {} closed its output", self.node).into());
        }
        Ok(serde_json::from_str(&line)?)
    }

    fn wait(mut self) -> Result<(), Box<dyn Error>> {
        let status = self.child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("topology worker {} exited with {status}", self.node).into())
        }
    }

    fn shutdown(mut self) -> Result<(), Box<dyn Error>> {
        expect_ack(self.request(WorkerRequest::Shutdown)?)?;
        self.wait()
    }

    fn crash(mut self) -> Result<(), Box<dyn Error>> {
        self.child.kill()?;
        let status = self.child.wait()?;
        if status.success() {
            Err(format!("topology worker {} did not crash", self.node).into())
        } else {
            Ok(())
        }
    }
}

impl Drop for NodeWorker {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case")]
enum WorkerRequest {
    Emit {
        tick: u64,
    },
    Admit {
        tick: u64,
        messages: Vec<TopologyWireMessage>,
    },
    Preflight {
        tick: u64,
    },
    Commit {
        tick: u64,
    },
    Snapshot,
    Restore {
        layout: TopologyLayout,
        snapshot: TopologyPartitionSnapshot,
    },
    Drain {
        target: u16,
        move_id: u64,
    },
    RuntimeVersion,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "result", content = "value", rename_all = "kebab-case")]
enum WorkerResponse {
    Messages(Vec<TopologyWireMessage>),
    Snapshot(TopologyPartitionSnapshot),
    RuntimeVersion(u16),
    Ack,
    Error(String),
}

fn expect_ack(response: WorkerResponse) -> Result<(), Box<dyn Error>> {
    match response {
        WorkerResponse::Ack => Ok(()),
        response => Err(unexpected("acknowledgement", response)),
    }
}

fn unexpected(expected: &str, response: WorkerResponse) -> Box<dyn Error> {
    match response {
        WorkerResponse::Error(error) => error.into(),
        response => format!("expected worker {expected}, got {response:?}").into(),
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[usize::from(byte >> 4)] as char);
        output.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    output
}
