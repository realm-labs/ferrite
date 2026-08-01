//! Deterministic multi-process fault injection over the topology worker protocol.

use super::{
    FRAME_LIMIT, MAILBOX_CAPACITY, NODE_COUNT, NodeWorker, WorkerRequest, WorkerResponse,
    expect_ack, hex, unexpected,
};
use ferrite_region_runtime::lattice::remoting::{LatticeRemotingAdapter, LatticeTransportFrame};
use ferrite_region_runtime::topology::cluster::{
    TopologyCluster, TopologyTransport, digest_snapshots, repartition_snapshots,
};
use ferrite_region_runtime::topology::layout::TopologyLayout;
use ferrite_region_runtime::topology::partition::{TopologyPartitionSnapshot, TopologyWireMessage};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const FAULT_REGION_COUNT: u16 = 12;
const FINAL_TICK: u64 = 64;
const OLD_RUNTIME_VERSION: u16 = 1;
const NEW_RUNTIME_VERSION: u16 = 2;

pub(super) fn verify() -> Result<(), Box<dyn Error>> {
    let initial_layout = TopologyLayout::ring(FAULT_REGION_COUNT, NODE_COUNT)?;
    let mut baseline = TopologyCluster::seeded(
        initial_layout.clone(),
        MAILBOX_CAPACITY,
        TopologyTransport::LatticeInProcess,
    )?;
    baseline.run_to(FINAL_TICK)?;

    let mut faulted = ProcessFaultCluster::spawn(initial_layout)?;
    stage(
        "duplicate and reordering",
        faulted.run_duplicate_and_reordering(1),
    )?;
    stage(
        "network partition and loss",
        faulted.run_partition_and_loss(2),
    )?;
    stage("pre-outage ticks", faulted.run_to(4))?;

    faulted.set_control_plane_available(false);
    let frozen_layout = faulted.layout.clone();
    let frozen_snapshots = faulted.snapshots()?;
    assert_error_contains(
        faulted.reconfigure_from(frozen_layout, frozen_snapshots),
        "control plane is unavailable",
    )?;
    stage("control-plane outage", faulted.run_to(8))?;
    faulted.set_control_plane_available(true);

    stage(
        "owner crash recovery",
        faulted.crash_owner_and_recover(1, 2),
    )?;
    stage("post-crash ticks", faulted.run_to(12))?;
    stage(
        "durable restart",
        faulted.replace_worker(2, OLD_RUNTIME_VERSION),
    )?;
    stage("post-restart ticks", faulted.run_to(16))?;
    stage("drain and handoff", faulted.drain_and_handoff(0, 1))?;
    stage("post-handoff ticks", faulted.run_to(24))?;

    for node in [1, 2, 0] {
        stage(
            "rolling replacement",
            faulted.replace_worker(node, NEW_RUNTIME_VERSION),
        )?;
        stage("mixed-version tick", faulted.run_tick(faulted.tick + 1))?;
    }
    faulted.run_to(FINAL_TICK)?;

    let digest = faulted.digest()?;
    if digest != baseline.digest() {
        return Err("faulted multi-process topology diverged from uninterrupted execution".into());
    }
    faulted.shutdown()?;
    println!(
        "multi-node fault injection verified: ticks={FINAL_TICK} regions={FAULT_REGION_COUNT} \
         nodes={NODE_COUNT} faults=8 digest={}",
        hex(&digest)
    );
    Ok(())
}

struct ProcessFaultCluster {
    _directory: TempDir,
    layout_path: PathBuf,
    layout: TopologyLayout,
    workers: BTreeMap<u16, NodeWorker>,
    versions: BTreeMap<u16, u16>,
    adapter: LatticeRemotingAdapter,
    control_plane_available: bool,
    tick: u64,
}

impl ProcessFaultCluster {
    fn spawn(layout: TopologyLayout) -> Result<Self, Box<dyn Error>> {
        let directory = tempfile::Builder::new()
            .prefix("ferrite-faults-")
            .tempdir()?;
        let layout_path = directory.path().join("layout.json");
        write_layout(&layout_path, &layout)?;
        let workers = (0..layout.node_count())
            .map(|node| {
                Ok((
                    node,
                    NodeWorker::spawn(node, &layout_path, OLD_RUNTIME_VERSION)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, Box<dyn Error>>>()?;
        Ok(Self {
            _directory: directory,
            layout_path,
            versions: (0..layout.node_count())
                .map(|node| (node, OLD_RUNTIME_VERSION))
                .collect(),
            layout,
            workers,
            adapter: LatticeRemotingAdapter::new(FRAME_LIMIT)?,
            control_plane_available: true,
            tick: 0,
        })
    }

    fn set_control_plane_available(&mut self, available: bool) {
        self.control_plane_available = available;
    }

    fn run_to(&mut self, final_tick: u64) -> Result<(), Box<dyn Error>> {
        for value in self.tick + 1..=final_tick {
            self.run_tick(value)?;
        }
        Ok(())
    }

    fn run_tick(&mut self, tick: u64) -> Result<(), Box<dyn Error>> {
        let mut messages = self.emit(tick)?;
        messages.reverse();
        self.admit_all(tick, messages)?;
        self.preflight_all(tick)?;
        self.commit_all(tick)
    }

    fn run_duplicate_and_reordering(&mut self, tick: u64) -> Result<(), Box<dyn Error>> {
        let mut messages = self.emit(tick)?;
        messages.reverse();
        messages.extend(messages.iter().take(3).cloned().collect::<Vec<_>>());
        self.admit_all(tick, messages)?;
        self.preflight_all(tick)?;
        self.commit_all(tick)
    }

    fn run_partition_and_loss(&mut self, tick: u64) -> Result<(), Box<dyn Error>> {
        let before = self.snapshots()?;
        let mut messages = self.emit(tick)?;
        let missing_index = messages
            .iter()
            .position(|message| self.target_node(message).ok() == Some(1))
            .ok_or("fault matrix found no message for partitioned node")?;
        let missing = messages.remove(missing_index);
        self.admit_all(tick, messages)?;
        assert_error_contains(self.preflight_all(tick), "missing required boundary")?;
        if self.snapshots()? != before {
            return Err("failed global preflight mutated committed state".into());
        }
        self.admit_all(tick, vec![missing])?;
        self.preflight_all(tick)?;
        self.commit_all(tick)
    }

    fn crash_owner_and_recover(
        &mut self,
        failed: u16,
        survivor: u16,
    ) -> Result<(), Box<dyn Error>> {
        let durable = self.snapshots()?;
        let stale_messages = self.emit(self.tick + 1)?;
        self.workers
            .remove(&failed)
            .ok_or_else(|| format!("missing worker {failed}"))?
            .crash()?;
        let next_layout = self.layout.recover_node(failed, survivor)?;
        self.reconfigure_from(next_layout, durable)?;
        let stale = stale_messages
            .into_iter()
            .find(|message| self.message_is_stale(message).unwrap_or(false))
            .ok_or("owner crash did not produce a stale-generation message")?;
        assert_error_contains(
            self.admit_all(self.tick + 1, vec![stale]),
            "stale source or target generation",
        )?;
        self.run_tick(self.tick + 1)
    }

    fn drain_and_handoff(&mut self, source: u16, target: u16) -> Result<(), Box<dyn Error>> {
        let move_id = self.tick + 1;
        expect_ack(
            self.worker_mut(source)?
                .request(WorkerRequest::Drain { target, move_id })?,
        )?;
        let next_tick = self.tick + 1;
        let response = self
            .worker_mut(source)?
            .request(WorkerRequest::Emit { tick: next_tick })?;
        assert_response_error_contains(response, "claim is closed")?;
        let durable = self.snapshots()?;
        let next_layout = self.layout.recover_node(source, target)?;
        self.reconfigure_from(next_layout, durable)?;
        self.run_tick(next_tick)
    }

    fn replace_worker(&mut self, node: u16, version: u16) -> Result<(), Box<dyn Error>> {
        let snapshot = self.snapshot_node(node)?;
        self.workers
            .remove(&node)
            .ok_or_else(|| format!("missing worker {node}"))?
            .shutdown()?;
        write_layout(&self.layout_path, &self.layout)?;
        let mut replacement = NodeWorker::spawn(node, &self.layout_path, version)?;
        expect_ack(replacement.request(WorkerRequest::Restore {
            layout: self.layout.clone(),
            snapshot,
        })?)?;
        match replacement.request(WorkerRequest::RuntimeVersion)? {
            WorkerResponse::RuntimeVersion(actual) if actual == version => {}
            response => return Err(unexpected("runtime version", response)),
        }
        self.versions.insert(node, version);
        self.workers.insert(node, replacement);
        Ok(())
    }

    fn reconfigure_from(
        &mut self,
        layout: TopologyLayout,
        snapshots: Vec<TopologyPartitionSnapshot>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.control_plane_available {
            return Err("control plane is unavailable".into());
        }
        let snapshots = repartition_snapshots(&snapshots, &layout)?;
        write_layout(&self.layout_path, &layout)?;
        for snapshot in snapshots {
            let node = snapshot.node;
            if !self.workers.contains_key(&node) {
                let version = self
                    .versions
                    .get(&node)
                    .copied()
                    .unwrap_or(OLD_RUNTIME_VERSION);
                self.workers
                    .insert(node, NodeWorker::spawn(node, &self.layout_path, version)?);
            }
            expect_ack(self.worker_mut(node)?.request(WorkerRequest::Restore {
                layout: layout.clone(),
                snapshot,
            })?)?;
        }
        self.layout = layout;
        Ok(())
    }

    fn emit(&mut self, tick: u64) -> Result<Vec<TopologyWireMessage>, Box<dyn Error>> {
        let mut messages = Vec::with_capacity(self.layout.len());
        for worker in self.workers.values_mut() {
            match worker.request(WorkerRequest::Emit { tick })? {
                WorkerResponse::Messages(emitted) => messages.extend(emitted),
                response => return Err(unexpected("messages", response)),
            }
        }
        Ok(messages)
    }

    fn admit_all(
        &mut self,
        tick: u64,
        messages: Vec<TopologyWireMessage>,
    ) -> Result<(), Box<dyn Error>> {
        let mut routed = BTreeMap::<u16, Vec<TopologyWireMessage>>::new();
        for message in messages {
            routed
                .entry(self.target_node(&message)?)
                .or_default()
                .push(message);
        }
        for (node, worker) in &mut self.workers {
            expect_ack(worker.request(WorkerRequest::Admit {
                tick,
                messages: routed.remove(node).unwrap_or_default(),
            })?)?;
        }
        if !routed.is_empty() {
            return Err("fault router targeted an absent worker".into());
        }
        Ok(())
    }

    fn preflight_all(&mut self, tick: u64) -> Result<(), Box<dyn Error>> {
        for worker in self.workers.values_mut() {
            expect_ack(worker.request(WorkerRequest::Preflight { tick })?)?;
        }
        Ok(())
    }

    fn commit_all(&mut self, tick: u64) -> Result<(), Box<dyn Error>> {
        for worker in self.workers.values_mut() {
            expect_ack(worker.request(WorkerRequest::Commit { tick })?)?;
        }
        self.tick = tick;
        Ok(())
    }

    fn snapshots(&mut self) -> Result<Vec<TopologyPartitionSnapshot>, Box<dyn Error>> {
        let mut snapshots = Vec::with_capacity(self.workers.len());
        for worker in self.workers.values_mut() {
            match worker.request(WorkerRequest::Snapshot)? {
                WorkerResponse::Snapshot(snapshot) => snapshots.push(snapshot),
                response => return Err(unexpected("snapshot", response)),
            }
        }
        Ok(snapshots)
    }

    fn snapshot_node(&mut self, node: u16) -> Result<TopologyPartitionSnapshot, Box<dyn Error>> {
        match self.worker_mut(node)?.request(WorkerRequest::Snapshot)? {
            WorkerResponse::Snapshot(snapshot) => Ok(snapshot),
            response => Err(unexpected("snapshot", response)),
        }
    }

    fn digest(&mut self) -> Result<[u8; 32], Box<dyn Error>> {
        Ok(digest_snapshots(&self.snapshots()?))
    }

    fn target_node(&self, message: &TopologyWireMessage) -> Result<u16, Box<dyn Error>> {
        let frame = LatticeTransportFrame::from_transport_payload(message.bytes.clone());
        let envelope = self.adapter.decode(&frame)?;
        Ok(self.layout.descriptor(envelope.target())?.node)
    }

    fn message_is_stale(&self, message: &TopologyWireMessage) -> Result<bool, Box<dyn Error>> {
        let frame = LatticeTransportFrame::from_transport_payload(message.bytes.clone());
        let envelope = self.adapter.decode(&frame)?;
        let source = self.layout.descriptor(envelope.source())?;
        let target = self.layout.descriptor(envelope.target())?;
        Ok(source.generation != envelope.source_generation()
            || target.generation != envelope.target_generation())
    }

    fn worker_mut(&mut self, node: u16) -> Result<&mut NodeWorker, Box<dyn Error>> {
        self.workers
            .get_mut(&node)
            .ok_or_else(|| format!("missing worker {node}").into())
    }

    fn shutdown(mut self) -> Result<(), Box<dyn Error>> {
        for worker in std::mem::take(&mut self.workers).into_values() {
            worker.shutdown()?;
        }
        Ok(())
    }
}

fn write_layout(path: &Path, layout: &TopologyLayout) -> Result<(), Box<dyn Error>> {
    fs::write(path, serde_json::to_vec(layout)?)?;
    Ok(())
}

fn assert_error_contains(
    result: Result<(), Box<dyn Error>>,
    expected: &str,
) -> Result<(), Box<dyn Error>> {
    match result {
        Ok(()) => Err(format!("expected error containing {expected:?}").into()),
        Err(error) if error.to_string().contains(expected) => Ok(()),
        Err(error) => Err(format!("expected error containing {expected:?}, got {error}").into()),
    }
}

fn assert_response_error_contains(
    response: WorkerResponse,
    expected: &str,
) -> Result<(), Box<dyn Error>> {
    match response {
        WorkerResponse::Error(error) if error.contains(expected) => Ok(()),
        response => {
            Err(format!("expected worker error containing {expected:?}, got {response:?}").into())
        }
    }
}

fn stage<T>(name: &str, result: Result<T, Box<dyn Error>>) -> Result<T, Box<dyn Error>> {
    result.map_err(|error| format!("{name}: {error}").into())
}
