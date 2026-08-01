//! One node's deterministic Region partition and bounded remote inbox.

use crate::lattice::authority::{
    LatticeNodeIdentity, RegionAuthorityAction, RegionAuthorityAdapter, RegionClaimGrant,
    RegionPlacementObservation, RegionPlacementState,
};
use crate::lattice::remoting::{
    LatticeRemotingAdapter, LatticeTransportFrame, RemoteRegionEnvelope,
    RemoteRegionEnvelopeHeader, RemoteRegionMessageKind,
};
use crate::lattice::spatial::{SpatialPlacementAdapter, SpatialPlacementConfig};
use crate::topology::TopologyError;
use crate::topology::layout::{TopologyLayout, TopologyRegionDescriptor};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::recovery::RecoveredRegion;
use ferrite_persistence::snapshot::{
    PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
    SnapshotRecord, SnapshotRecordKind,
};
use ferrite_simulation::tick::GameTick;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyWireMessage {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyRegionSnapshot {
    pub key: SimulationRegionKey,
    pub generation: ActivationGeneration,
    pub committed_tick: u64,
    pub value: u64,
}

impl TopologyRegionSnapshot {
    pub fn recovery_point(&self) -> Result<RegionRecoveryPoint, TopologyError> {
        let domain = ResourceId::new("ferrite", "topology-conformance/v1")
            .expect("the topology recovery domain is a valid resource ID");
        let state_hash = *blake3::hash(&self.value.to_be_bytes()).as_bytes();
        let revision = self
            .committed_tick
            .checked_add(1)
            .ok_or(TopologyError::ArithmeticOverflow)?;
        RegionRecoveryPoint::new(
            RegionCommitSnapshot::new(
                RegionSnapshotHeader {
                    key: self.key.clone(),
                    generation: self.generation,
                    committed_tick: self.committed_tick,
                    persistence_revision: PersistenceRevision::new(revision)?,
                    region_side_chunks: 8,
                    content_manifest: [0; 32],
                    state_hash,
                },
                vec![SnapshotRecord::new(
                    SnapshotRecordKind::Extension,
                    domain,
                    b"state".to_vec(),
                    self.value.to_be_bytes().to_vec(),
                )?],
            )?,
            Vec::new(),
        )
        .map_err(TopologyError::from)
    }

    pub fn from_recovered(recovered: RecoveredRegion) -> Result<Self, TopologyError> {
        let point = recovered.recovery_point();
        let records = point.snapshot().records();
        if records.len() != 1
            || records[0].kind() != SnapshotRecordKind::Extension
            || records[0].domain().namespace() != "ferrite"
            || records[0].domain().path() != "topology-conformance/v1"
            || records[0].key() != b"state"
            || records[0].value().len() != size_of::<u64>()
        {
            return Err(TopologyError::InvalidRecoveryPoint);
        }
        let value = u64::from_be_bytes(
            records[0]
                .value()
                .try_into()
                .map_err(|_| TopologyError::InvalidRecoveryPoint)?,
        );
        Ok(Self {
            key: recovered.key().clone(),
            generation: recovered.generation(),
            committed_tick: recovered.committed_tick(),
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyPartitionSnapshot {
    pub node: u16,
    pub regions: Vec<TopologyRegionSnapshot>,
}

pub struct TopologyPartition {
    node: u16,
    layout: TopologyLayout,
    regions: BTreeMap<SimulationRegionKey, TopologyRegionSnapshot>,
    authorities: BTreeMap<SimulationRegionKey, RegionAuthorityAdapter>,
    inbox: BTreeMap<RemoteIdentity, RemoteRegionEnvelope>,
    mailbox_capacity: usize,
    draining: bool,
}

impl TopologyPartition {
    pub fn seeded(
        node: u16,
        layout: TopologyLayout,
        mailbox_capacity: usize,
    ) -> Result<Self, TopologyError> {
        if mailbox_capacity == 0 {
            return Err(TopologyError::ZeroMailboxCapacity);
        }
        if node >= layout.node_count() {
            return Err(TopologyError::UnknownNode(node));
        }
        let regions = layout
            .descriptors()
            .filter(|descriptor| descriptor.node == node)
            .map(|descriptor| {
                let state = TopologyRegionSnapshot {
                    key: descriptor.key.clone(),
                    generation: descriptor.generation,
                    committed_tick: 0,
                    value: initial_value(descriptor),
                };
                (descriptor.key.clone(), state)
            })
            .collect();
        let authorities = build_authorities(node, &layout)?;
        Ok(Self {
            node,
            layout,
            regions,
            authorities,
            inbox: BTreeMap::new(),
            mailbox_capacity,
            draining: false,
        })
    }

    pub fn restore(
        snapshot: TopologyPartitionSnapshot,
        layout: TopologyLayout,
        mailbox_capacity: usize,
    ) -> Result<Self, TopologyError> {
        if mailbox_capacity == 0 {
            return Err(TopologyError::ZeroMailboxCapacity);
        }
        if snapshot.node >= layout.node_count() {
            return Err(TopologyError::UnknownNode(snapshot.node));
        }
        let mut regions = BTreeMap::new();
        for region in snapshot.regions {
            let descriptor = layout.descriptor(&region.key)?;
            if descriptor.node != snapshot.node || descriptor.generation != region.generation {
                return Err(TopologyError::SnapshotLayoutMismatch);
            }
            if regions.insert(region.key.clone(), region).is_some() {
                return Err(TopologyError::SnapshotLayoutMismatch);
            }
        }
        let expected = layout
            .descriptors()
            .filter(|descriptor| descriptor.node == snapshot.node)
            .count();
        if regions.len() != expected {
            return Err(TopologyError::SnapshotLayoutMismatch);
        }
        let authorities = build_authorities(snapshot.node, &layout)?;
        Ok(Self {
            node: snapshot.node,
            layout,
            regions,
            authorities,
            inbox: BTreeMap::new(),
            mailbox_capacity,
            draining: false,
        })
    }

    pub const fn node(&self) -> u16 {
        self.node
    }

    pub fn pending_messages(&self) -> usize {
        self.inbox.len()
    }

    pub fn snapshot(&self) -> TopologyPartitionSnapshot {
        TopologyPartitionSnapshot {
            node: self.node,
            regions: self.regions.values().cloned().collect(),
        }
    }

    pub fn begin_drain(&mut self, target_node: u16, move_id: u128) -> Result<(), TopologyError> {
        if !self.inbox.is_empty() {
            return Err(TopologyError::DrainWithPendingMessages);
        }
        if self.draining {
            return Ok(());
        }
        if target_node >= self.layout.node_count() {
            return Err(TopologyError::UnknownNode(target_node));
        }
        let target = topology_node_identity(target_node)?;
        let regions = self
            .regions
            .values()
            .map(|region| (region.key.clone(), region.generation))
            .collect::<Vec<_>>();
        for (key, generation) in regions {
            let authority = self
                .authorities
                .get_mut(&key)
                .ok_or_else(|| TopologyError::UnknownRegion(key.clone()))?;
            authority.reconcile(RegionPlacementObservation {
                generation,
                coordinator_term: 1,
                revision: generation
                    .get()
                    .checked_add(1)
                    .ok_or(TopologyError::ArithmeticOverflow)?,
                state: RegionPlacementState::BeginHandoff,
                target: Some(target.clone()),
                move_id: Some(move_id),
            })?;
            let outcome = authority.begin_drain()?;
            if !outcome.contains(RegionAuthorityAction::FenceAdmission)
                || !outcome.contains(RegionAuthorityAction::DrainRegion)
            {
                return Err(TopologyError::DrainDidNotFence);
            }
        }
        self.draining = true;
        Ok(())
    }

    pub fn emit(
        &self,
        tick: GameTick,
        adapter: &LatticeRemotingAdapter,
    ) -> Result<Vec<TopologyWireMessage>, TopologyError> {
        self.regions
            .values()
            .map(|region| {
                let authority = self
                    .authorities
                    .get(&region.key)
                    .ok_or_else(|| TopologyError::UnknownRegion(region.key.clone()))?;
                if !authority.admission_open(region.generation, tick.get()) {
                    return Err(TopologyError::AuthorityClosed(region.key.clone()));
                }
                let expected = GameTick::new(region.committed_tick)
                    .checked_next()
                    .map_err(|_| TopologyError::ArithmeticOverflow)?;
                if tick != expected {
                    return Err(TopologyError::UnexpectedTick {
                        expected,
                        actual: tick,
                    });
                }
                let target = self.layout.successor(&region.key)?;
                let contribution = contribution(region, tick);
                let envelope = RemoteRegionEnvelope::new(
                    RemoteRegionEnvelopeHeader {
                        kind: RemoteRegionMessageKind::Boundary,
                        tick,
                        source: region.key.clone(),
                        target: target.key.clone(),
                        source_generation: region.generation,
                        target_generation: target.generation,
                        source_sequence: tick.get(),
                    },
                    contribution.to_le_bytes().to_vec(),
                )?;
                let frame = adapter.encode(&envelope)?;
                Ok(TopologyWireMessage {
                    bytes: frame.transport_payload().to_vec(),
                })
            })
            .collect()
    }

    pub fn emit_tick(
        &self,
        tick: u64,
        adapter: &LatticeRemotingAdapter,
    ) -> Result<Vec<TopologyWireMessage>, TopologyError> {
        self.emit(GameTick::new(tick), adapter)
    }

    pub fn admit(
        &mut self,
        message: TopologyWireMessage,
        tick: GameTick,
        adapter: &LatticeRemotingAdapter,
    ) -> Result<AdmissionOutcome, TopologyError> {
        let frame = LatticeTransportFrame::from_transport_payload(message.bytes);
        let envelope = adapter.decode(&frame)?;
        self.validate_envelope(&envelope, tick)?;
        let identity = RemoteIdentity::from_envelope(&envelope);
        if let Some(existing) = self.inbox.get(&identity) {
            return if existing == &envelope {
                Ok(AdmissionOutcome::Duplicate)
            } else {
                Err(TopologyError::ConflictingDuplicate)
            };
        }
        if self.inbox.len() == self.mailbox_capacity {
            return Err(TopologyError::MailboxFull {
                capacity: self.mailbox_capacity,
            });
        }
        self.inbox.insert(identity, envelope);
        Ok(AdmissionOutcome::Accepted)
    }

    pub fn admit_tick(
        &mut self,
        message: TopologyWireMessage,
        tick: u64,
        adapter: &LatticeRemotingAdapter,
    ) -> Result<AdmissionOutcome, TopologyError> {
        self.admit(message, GameTick::new(tick), adapter)
    }

    pub fn commit(&mut self, tick: GameTick) -> Result<(), TopologyError> {
        let updates = self.preflight_updates(tick)?;
        for (key, value) in updates {
            let region = self
                .regions
                .get_mut(&key)
                .ok_or_else(|| TopologyError::UnknownRegion(key.clone()))?;
            region.value = value;
            region.committed_tick = tick.get();
        }
        self.inbox.clear();
        Ok(())
    }

    pub fn commit_tick(&mut self, tick: u64) -> Result<(), TopologyError> {
        self.commit(GameTick::new(tick))
    }

    pub fn can_commit(&self, tick: GameTick) -> Result<(), TopologyError> {
        self.preflight_updates(tick).map(|_| ())
    }

    pub fn can_commit_tick(&self, tick: u64) -> Result<(), TopologyError> {
        self.can_commit(GameTick::new(tick))
    }

    fn preflight_updates(
        &self,
        tick: GameTick,
    ) -> Result<Vec<(SimulationRegionKey, u64)>, TopologyError> {
        let mut updates = Vec::with_capacity(self.regions.len());
        for region in self.regions.values() {
            let expected_tick = GameTick::new(region.committed_tick)
                .checked_next()
                .map_err(|_| TopologyError::ArithmeticOverflow)?;
            if tick != expected_tick {
                return Err(TopologyError::UnexpectedTick {
                    expected: expected_tick,
                    actual: tick,
                });
            }
            let source = self.layout.predecessor(&region.key)?;
            let identity = RemoteIdentity {
                tick: tick.get(),
                source: source.key.clone(),
                target: region.key.clone(),
                source_sequence: tick.get(),
            };
            let envelope = self.inbox.get(&identity).ok_or_else(|| {
                TopologyError::MissingRequiredBoundary {
                    tick,
                    target: region.key.clone(),
                }
            })?;
            let contribution = u64::from_le_bytes(
                envelope
                    .payload()
                    .try_into()
                    .map_err(|_| TopologyError::InvalidBoundaryMessage)?,
            );
            updates.push((
                region.key.clone(),
                mix(region.value, contribution, tick.get(), &region.key),
            ));
        }
        Ok(updates)
    }

    fn validate_envelope(
        &self,
        envelope: &RemoteRegionEnvelope,
        tick: GameTick,
    ) -> Result<(), TopologyError> {
        if envelope.tick() != tick {
            return Err(TopologyError::UnexpectedTick {
                expected: tick,
                actual: envelope.tick(),
            });
        }
        let source = self.layout.descriptor(envelope.source())?;
        let target = self.layout.descriptor(envelope.target())?;
        if target.node != self.node {
            return Err(TopologyError::WrongTargetPartition);
        }
        if source.generation != envelope.source_generation()
            || target.generation != envelope.target_generation()
        {
            return Err(TopologyError::StaleGeneration);
        }
        let target_authority = self
            .authorities
            .get(&target.key)
            .ok_or_else(|| TopologyError::UnknownRegion(target.key.clone()))?;
        if !target_authority.admission_open(target.generation, tick.get()) {
            return Err(TopologyError::AuthorityClosed(target.key.clone()));
        }
        if self.layout.predecessor(&target.key)?.key != source.key {
            return Err(TopologyError::UnexpectedSource);
        }
        if envelope.kind() != RemoteRegionMessageKind::Boundary
            || envelope.source_sequence() != tick.get()
            || envelope.payload().len() != size_of::<u64>()
        {
            return Err(TopologyError::InvalidBoundaryMessage);
        }
        Ok(())
    }
}

fn build_authorities(
    node: u16,
    layout: &TopologyLayout,
) -> Result<BTreeMap<SimulationRegionKey, RegionAuthorityAdapter>, TopologyError> {
    let spatial = SpatialPlacementAdapter::new(SpatialPlacementConfig::new(
        "ferrite-region-v1",
        1,
        4_096,
        1,
    )?)?;
    let local = topology_node_identity(node)?;
    layout
        .descriptors()
        .filter(|descriptor| descriptor.node == node)
        .map(|descriptor| {
            let route = spatial.route(&descriptor.key)?;
            let mut authority = spatial.authority_adapter(&route, local.clone(), 1)?;
            authority.reconcile(RegionPlacementObservation {
                generation: descriptor.generation,
                coordinator_term: 1,
                revision: descriptor.generation.get(),
                state: RegionPlacementState::Running,
                target: None,
                move_id: None,
            })?;
            authority.install_claim(
                RegionClaimGrant {
                    generation: descriptor.generation,
                    coordinator_term: 1,
                    grant_sequence: descriptor.generation.get(),
                    ttl_millis: 100_000,
                },
                0,
            )?;
            Ok((descriptor.key.clone(), authority))
        })
        .collect()
}

fn topology_node_identity(node: u16) -> Result<LatticeNodeIdentity, TopologyError> {
    Ok(LatticeNodeIdentity::new(
        format!("topology-node-{node}"),
        "127.0.0.1",
        7_000_u16
            .checked_add(node)
            .ok_or(TopologyError::ArithmeticOverflow)?,
        u128::from(node) + 1,
    )?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionOutcome {
    Accepted,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RemoteIdentity {
    tick: u64,
    source: SimulationRegionKey,
    target: SimulationRegionKey,
    source_sequence: u64,
}

impl RemoteIdentity {
    fn from_envelope(envelope: &RemoteRegionEnvelope) -> Self {
        Self {
            tick: envelope.tick().get(),
            source: envelope.source().clone(),
            target: envelope.target().clone(),
            source_sequence: envelope.source_sequence(),
        }
    }
}

fn initial_value(descriptor: &TopologyRegionDescriptor) -> u64 {
    let coordinate = descriptor.key.coordinate();
    mix(
        0x4645_5252_4954_4501,
        coordinate.x() as u64,
        coordinate.z() as u64,
        &descriptor.key,
    )
}

fn contribution(region: &TopologyRegionSnapshot, tick: GameTick) -> u64 {
    mix(region.value, tick.get(), 0x5254_4f50_4f4c_4f47, &region.key)
}

fn mix(left: u64, middle: u64, right: u64, key: &SimulationRegionKey) -> u64 {
    let coordinate = key.coordinate();
    let mut value = left ^ middle.rotate_left(17) ^ right.rotate_left(41);
    value ^= (coordinate.x() as u64).rotate_left(7);
    value ^= (coordinate.z() as u64).rotate_left(29);
    value = value.wrapping_mul(0x9e37_79b1_85eb_ca87);
    value ^ value.rotate_right(23)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mailbox_overload_retains_accepted_work_and_corruption_fails_closed() {
        let layout = TopologyLayout::ring(4, 1).unwrap();
        let adapter = LatticeRemotingAdapter::new(16 * 1024).unwrap();
        let emitter = TopologyPartition::seeded(0, layout.clone(), 8).unwrap();
        let mut receiver = TopologyPartition::seeded(0, layout, 1).unwrap();
        let messages = emitter.emit(GameTick::new(1), &adapter).unwrap();

        assert_eq!(
            receiver
                .admit(messages[0].clone(), GameTick::new(1), &adapter)
                .unwrap(),
            AdmissionOutcome::Accepted
        );
        assert!(matches!(
            receiver.admit(messages[1].clone(), GameTick::new(1), &adapter),
            Err(TopologyError::MailboxFull { capacity: 1 })
        ));
        assert_eq!(receiver.pending_messages(), 1);

        let mut corrupted = messages[2].clone();
        corrupted.bytes[0] ^= 0xff;
        assert!(matches!(
            receiver.admit(corrupted, GameTick::new(1), &adapter),
            Err(TopologyError::Remoting(_))
        ));
        assert_eq!(receiver.pending_messages(), 1);
    }

    #[test]
    fn exact_duplicates_are_idempotent_and_conflicts_are_rejected() {
        let layout = TopologyLayout::ring(2, 1).unwrap();
        let adapter = LatticeRemotingAdapter::new(16 * 1024).unwrap();
        let emitter = TopologyPartition::seeded(0, layout.clone(), 4).unwrap();
        let mut receiver = TopologyPartition::seeded(0, layout, 4).unwrap();
        let message = emitter.emit(GameTick::new(1), &adapter).unwrap()[0].clone();
        receiver
            .admit(message.clone(), GameTick::new(1), &adapter)
            .unwrap();
        assert_eq!(
            receiver
                .admit(message.clone(), GameTick::new(1), &adapter)
                .unwrap(),
            AdmissionOutcome::Duplicate
        );
        let mut conflict = message;
        let last = conflict.bytes.len() - 1;
        conflict.bytes[last] ^= 1;
        assert!(matches!(
            receiver.admit(conflict, GameTick::new(1), &adapter),
            Err(TopologyError::ConflictingDuplicate)
        ));
        assert_eq!(receiver.pending_messages(), 1);
    }

    #[test]
    fn handoff_reconcile_fences_partition_admission_before_drain() {
        let layout = TopologyLayout::ring(4, 2).unwrap();
        let adapter = LatticeRemotingAdapter::new(16 * 1024).unwrap();
        let mut partition = TopologyPartition::seeded(0, layout, 4).unwrap();

        partition.begin_drain(1, 7).unwrap();
        partition.begin_drain(1, 7).unwrap();
        assert!(matches!(
            partition.emit(GameTick::new(1), &adapter),
            Err(TopologyError::AuthorityClosed(_))
        ));
    }
}
