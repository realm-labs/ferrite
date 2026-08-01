//! Local and in-process distributed orchestration over the same Region partitions.

use crate::lattice::remoting::LatticeRemotingAdapter;
use crate::topology::TopologyError;
use crate::topology::layout::TopologyLayout;
use crate::topology::partition::{
    AdmissionOutcome, TopologyPartition, TopologyPartitionSnapshot, TopologyRegionSnapshot,
    TopologyWireMessage,
};
use ferrite_persistence::recovery::RegionHandoffState;
use ferrite_persistence::snapshot::RegionRecoveryPoint;
use ferrite_simulation::tick::GameTick;
use std::collections::BTreeMap;

const TOPOLOGY_FRAME_LIMIT: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyTransport {
    SemanticLocal,
    LatticeInProcess,
}

pub struct TopologyCluster {
    layout: TopologyLayout,
    partitions: BTreeMap<u16, TopologyPartition>,
    adapter: LatticeRemotingAdapter,
    transport: TopologyTransport,
    mailbox_capacity: usize,
}

impl TopologyCluster {
    pub fn seeded(
        layout: TopologyLayout,
        mailbox_capacity: usize,
        transport: TopologyTransport,
    ) -> Result<Self, TopologyError> {
        let partitions = (0..layout.node_count())
            .map(|node| {
                Ok((
                    node,
                    TopologyPartition::seeded(node, layout.clone(), mailbox_capacity)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, TopologyError>>()?;
        Ok(Self {
            layout,
            partitions,
            adapter: LatticeRemotingAdapter::new(TOPOLOGY_FRAME_LIMIT)?,
            transport,
            mailbox_capacity,
        })
    }

    pub fn run_to(&mut self, final_tick: u64) -> Result<(), TopologyError> {
        let current = self.committed_tick()?;
        for value in current + 1..=final_tick {
            self.run_tick(GameTick::new(value))?;
        }
        Ok(())
    }

    pub fn run_tick(&mut self, tick: GameTick) -> Result<(), TopologyError> {
        let mut messages = self.emit_tick(tick)?;
        if self.transport == TopologyTransport::LatticeInProcess {
            messages.reverse();
        }
        self.admit_all(tick, messages)?;
        self.commit_all(tick)
    }

    pub fn emit_tick(&self, tick: GameTick) -> Result<Vec<TopologyWireMessage>, TopologyError> {
        let mut messages = Vec::with_capacity(self.layout.len());
        for partition in self.partitions.values() {
            messages.extend(partition.emit(tick, &self.adapter)?);
        }
        Ok(messages)
    }

    pub fn admit_all(
        &mut self,
        tick: GameTick,
        messages: Vec<TopologyWireMessage>,
    ) -> Result<(), TopologyError> {
        for message in messages {
            let frame = crate::lattice::remoting::LatticeTransportFrame::from_transport_payload(
                message.bytes.clone(),
            );
            let envelope = self.adapter.decode(&frame)?;
            let node = self.layout.descriptor(envelope.target())?.node;
            let partition = self
                .partitions
                .get_mut(&node)
                .ok_or(TopologyError::UnknownNode(node))?;
            let outcome = partition.admit(message, tick, &self.adapter)?;
            if outcome == AdmissionOutcome::Duplicate {
                continue;
            }
        }
        Ok(())
    }

    pub fn commit_all(&mut self, tick: GameTick) -> Result<(), TopologyError> {
        for partition in self.partitions.values() {
            partition.can_commit(tick)?;
        }
        for partition in self.partitions.values_mut() {
            partition.commit(tick)?;
        }
        Ok(())
    }

    pub fn snapshots(&self) -> Vec<TopologyPartitionSnapshot> {
        self.partitions
            .values()
            .map(TopologyPartition::snapshot)
            .collect()
    }

    pub fn digest(&self) -> [u8; 32] {
        digest_snapshots(&self.snapshots())
    }

    pub fn committed_tick(&self) -> Result<u64, TopologyError> {
        let mut ticks = self
            .partitions
            .values()
            .flat_map(|partition| partition.snapshot().regions)
            .map(|region| region.committed_tick);
        let first = ticks.next().ok_or(TopologyError::EmptyLayout)?;
        if ticks.any(|tick| tick != first) {
            return Err(TopologyError::SnapshotLayoutMismatch);
        }
        Ok(first)
    }

    pub fn recover_node(&mut self, failed: u16, survivor: u16) -> Result<(), TopologyError> {
        let layout = self.layout.recover_node(failed, survivor)?;
        self.reconfigure(layout)
    }

    pub fn reconfigure(&mut self, layout: TopologyLayout) -> Result<(), TopologyError> {
        let snapshots = repartition_snapshots(&self.snapshots(), &layout)?;
        let partitions = snapshots
            .into_iter()
            .map(|snapshot| {
                Ok((
                    snapshot.node,
                    TopologyPartition::restore(snapshot, layout.clone(), self.mailbox_capacity)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, TopologyError>>()?;
        self.layout = layout;
        self.partitions = partitions;
        Ok(())
    }
}

pub fn repartition_snapshots(
    snapshots: &[TopologyPartitionSnapshot],
    layout: &TopologyLayout,
) -> Result<Vec<TopologyPartitionSnapshot>, TopologyError> {
    let mut regions = BTreeMap::<_, TopologyRegionSnapshot>::new();
    for region in snapshots
        .iter()
        .flat_map(|snapshot| snapshot.regions.iter())
        .cloned()
    {
        if regions.insert(region.key.clone(), region).is_some() {
            return Err(TopologyError::SnapshotLayoutMismatch);
        }
    }
    let partitions = (0..layout.node_count())
        .map(|node| {
            let mut node_regions = Vec::new();
            for descriptor in layout
                .descriptors()
                .filter(|descriptor| descriptor.node == node)
            {
                let mut region = regions
                    .remove(&descriptor.key)
                    .ok_or_else(|| TopologyError::UnknownRegion(descriptor.key.clone()))?;
                if region.generation != descriptor.generation {
                    let encoded = region.recovery_point()?.encode()?;
                    let durable = RegionRecoveryPoint::decode(&encoded)?;
                    let handoff = RegionHandoffState::prepare(durable, descriptor.generation)?;
                    let digest = *handoff.digest();
                    region = TopologyRegionSnapshot::from_recovered(
                        handoff.install(&descriptor.key, digest)?,
                    )?;
                }
                node_regions.push(region);
            }
            Ok(TopologyPartitionSnapshot {
                node,
                regions: node_regions,
            })
        })
        .collect::<Result<Vec<_>, TopologyError>>()?;
    if !regions.is_empty() {
        return Err(TopologyError::SnapshotLayoutMismatch);
    }
    Ok(partitions)
}

pub fn digest_snapshots(snapshots: &[TopologyPartitionSnapshot]) -> [u8; 32] {
    let mut regions = snapshots
        .iter()
        .flat_map(|snapshot| snapshot.regions.iter())
        .collect::<Vec<_>>();
    regions.sort_by(|left, right| left.key.cmp(&right.key));
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ferrite-topology-state-v1");
    for region in regions {
        hasher.update(&region.key.world().to_be_bytes());
        let dimension = region.key.dimension().resource().to_string();
        hasher.update(&(dimension.len() as u32).to_be_bytes());
        hasher.update(dimension.as_bytes());
        hasher.update(&region.key.coordinate().x().to_be_bytes());
        hasher.update(&region.key.coordinate().z().to_be_bytes());
        hasher.update(&region.key.mapping_version().get().to_be_bytes());
        hasher.update(&region.committed_tick.to_be_bytes());
        hasher.update(&region.value.to_be_bytes());
    }
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_and_in_process_topologies_converge() {
        let distributed = TopologyLayout::ring(8, 3).unwrap();
        let local = distributed.with_all_on_node(0).unwrap();
        let mut local =
            TopologyCluster::seeded(local, 16, TopologyTransport::SemanticLocal).unwrap();
        let mut distributed =
            TopologyCluster::seeded(distributed, 16, TopologyTransport::LatticeInProcess).unwrap();
        local.run_to(512).unwrap();
        distributed.run_to(512).unwrap();
        assert_eq!(local.digest(), distributed.digest());
    }

    #[test]
    fn ten_thousand_ticks_are_topology_independent() {
        let distributed = TopologyLayout::ring(12, 3).unwrap();
        let local = distributed.with_all_on_node(0).unwrap();
        let mut local =
            TopologyCluster::seeded(local, 32, TopologyTransport::SemanticLocal).unwrap();
        let mut distributed =
            TopologyCluster::seeded(distributed, 32, TopologyTransport::LatticeInProcess).unwrap();
        local.run_to(10_000).unwrap();
        distributed.run_to(10_000).unwrap();
        assert_eq!(local.digest(), distributed.digest());
        assert_eq!(
            blake3::Hash::from_bytes(local.digest()).to_hex().as_str(),
            "02ae8ad8bb897c569339b725bc3f44ed8ea49db653a25adf8d244ca68bf27685"
        );
    }

    #[test]
    fn duplicate_reordering_loss_and_stale_owner_are_explicit() {
        let layout = TopologyLayout::ring(6, 3).unwrap();
        let mut baseline =
            TopologyCluster::seeded(layout.clone(), 16, TopologyTransport::LatticeInProcess)
                .unwrap();
        let mut faulted =
            TopologyCluster::seeded(layout, 16, TopologyTransport::LatticeInProcess).unwrap();
        baseline.run_tick(GameTick::new(1)).unwrap();

        let mut messages = faulted.emit_tick(GameTick::new(1)).unwrap();
        let missing = messages.pop().unwrap();
        messages.reverse();
        messages.push(messages[0].clone());
        faulted.admit_all(GameTick::new(1), messages).unwrap();
        assert!(matches!(
            faulted.commit_all(GameTick::new(1)),
            Err(TopologyError::MissingRequiredBoundary { .. })
        ));
        assert_eq!(faulted.committed_tick().unwrap(), 0);
        faulted.admit_all(GameTick::new(1), vec![missing]).unwrap();
        faulted.commit_all(GameTick::new(1)).unwrap();
        assert_eq!(faulted.digest(), baseline.digest());

        let stale = faulted.emit_tick(GameTick::new(2)).unwrap();
        faulted.recover_node(1, 2).unwrap();
        assert!(matches!(
            faulted.admit_all(GameTick::new(2), stale),
            Err(TopologyError::StaleGeneration)
        ));
    }

    #[test]
    fn recovered_node_advances_generation_and_continues() {
        let layout = TopologyLayout::ring(6, 3).unwrap();
        let mut uninterrupted =
            TopologyCluster::seeded(layout.clone(), 16, TopologyTransport::LatticeInProcess)
                .unwrap();
        let mut cluster =
            TopologyCluster::seeded(layout, 16, TopologyTransport::LatticeInProcess).unwrap();
        uninterrupted.run_to(8).unwrap();
        cluster.run_to(8).unwrap();
        cluster.recover_node(1, 2).unwrap();
        uninterrupted.run_to(16).unwrap();
        cluster.run_to(16).unwrap();
        assert_eq!(cluster.committed_tick().unwrap(), 16);
        assert_eq!(cluster.digest(), uninterrupted.digest());
    }
}
