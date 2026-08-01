use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_persistence::snapshot::SnapshotRecord;
use ferrite_region_runtime::transfer::{
    EntityTransfer, EntityTransferError, EntityTransferHeader, TransferRole,
};
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::continuity::migration::{ContinuityMigrationError, normalize_records};
use crate::entity_service::continuity::{
    EntityServiceContinuityError, decode_entity, decode_receipt, decode_transfer_state,
    encode_entity, encode_receipt, encode_transfer_state,
};
use crate::entity_service::model::{
    EntityCommandHeader, EntityLifecycleState, EntityMutation, EntityPersistentState,
    EntityProjection, EntityProjectionKind, EntityTransferRequest, LifecycleOutcome,
    ObserverOutcome, OutboundEntityTransfer, RemovalReason,
};
use crate::entity_service::transfer::{
    AppliedTransferKey, EntityTransferReceipt, TransferAcceptance,
};

#[derive(Debug, Clone)]
pub struct EntityServiceRegionRuntime {
    key: SimulationRegionKey,
    generation: ActivationGeneration,
    mapping: RegionMapping,
    entity_capacity: usize,
    observer_capacity: usize,
    projection_capacity_per_observer: usize,
    receipt_capacity: usize,
    next_projection_sequence: u64,
    entities: BTreeMap<StableEntityId, EntityPersistentState>,
    observers: BTreeMap<StableEntityId, VecDeque<EntityProjection>>,
    applied_transfers: BTreeSet<AppliedTransferKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityServiceRuntimeLimits {
    pub entity_capacity: usize,
    pub observer_capacity: usize,
    pub projection_capacity_per_observer: usize,
    pub receipt_capacity: usize,
}

impl EntityServiceRuntimeLimits {
    #[must_use]
    pub const fn new(
        entity_capacity: usize,
        observer_capacity: usize,
        projection_capacity_per_observer: usize,
        receipt_capacity: usize,
    ) -> Self {
        Self {
            entity_capacity,
            observer_capacity,
            projection_capacity_per_observer,
            receipt_capacity,
        }
    }
}

impl EntityServiceRegionRuntime {
    pub fn new(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        mapping: RegionMapping,
        limits: EntityServiceRuntimeLimits,
    ) -> Result<Self, EntityServiceRuntimeError> {
        validate_capacity("entities", limits.entity_capacity)?;
        validate_capacity("observers", limits.observer_capacity)?;
        validate_capacity(
            "projections per observer",
            limits.projection_capacity_per_observer,
        )?;
        validate_capacity("transfer receipts", limits.receipt_capacity)?;
        if key.mapping_version() != mapping.version() {
            return Err(EntityServiceRuntimeError::MappingVersionMismatch);
        }
        Ok(Self {
            key,
            generation,
            mapping,
            entity_capacity: limits.entity_capacity,
            observer_capacity: limits.observer_capacity,
            projection_capacity_per_observer: limits.projection_capacity_per_observer,
            receipt_capacity: limits.receipt_capacity,
            next_projection_sequence: 1,
            entities: BTreeMap::new(),
            observers: BTreeMap::new(),
            applied_transfers: BTreeSet::new(),
        })
    }

    pub fn restore(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        mapping: RegionMapping,
        limits: EntityServiceRuntimeLimits,
        records: &[SnapshotRecord],
    ) -> Result<Self, EntityServiceRuntimeError> {
        let normalized = normalize_records(records)?;
        let mut runtime = Self::new(key, generation, mapping, limits)?;
        for record in normalized.records() {
            if let Some((entity, state)) = decode_entity(record)? {
                if runtime.entities.len() == limits.entity_capacity {
                    return Err(EntityServiceRuntimeError::EntityCapacity {
                        entity_capacity: limits.entity_capacity,
                    });
                }
                runtime.validate_owned_state(&state)?;
                if runtime.entities.insert(entity, state).is_some() {
                    return Err(EntityServiceRuntimeError::DuplicateEntity(entity));
                }
            }
            if let Some(receipt) = decode_receipt(record)? {
                if runtime.applied_transfers.len() == limits.receipt_capacity {
                    return Err(EntityServiceRuntimeError::ReceiptCapacity {
                        receipt_capacity: limits.receipt_capacity,
                    });
                }
                if !runtime.applied_transfers.insert(receipt) {
                    return Err(EntityServiceRuntimeError::DuplicateReceipt);
                }
            }
        }
        Ok(runtime)
    }

    #[must_use]
    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    #[must_use]
    pub const fn generation(&self) -> ActivationGeneration {
        self.generation
    }

    #[must_use]
    pub fn state(&self, entity: StableEntityId) -> Option<&EntityPersistentState> {
        self.entities.get(&entity)
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    #[must_use]
    pub fn observer_count(&self) -> usize {
        self.observers.len()
    }

    #[must_use]
    pub fn projection_len(&self, observer: StableEntityId) -> Option<usize> {
        self.observers.get(&observer).map(VecDeque::len)
    }

    #[must_use]
    pub fn applied_transfer_count(&self) -> usize {
        self.applied_transfers.len()
    }

    pub fn snapshot_records(&self) -> Result<Vec<SnapshotRecord>, EntityServiceRuntimeError> {
        let mut records = Vec::with_capacity(self.entities.len() + self.applied_transfers.len());
        for (entity, state) in &self.entities {
            records.push(encode_entity(*entity, state)?);
        }
        for receipt in &self.applied_transfers {
            records.push(encode_receipt(receipt)?);
        }
        Ok(records)
    }

    pub fn insert(
        &mut self,
        entity: StableEntityId,
        state: EntityPersistentState,
    ) -> Result<(), EntityServiceRuntimeError> {
        if self.entities.contains_key(&entity) {
            return Err(EntityServiceRuntimeError::DuplicateEntity(entity));
        }
        if self.entities.len() == self.entity_capacity {
            return Err(EntityServiceRuntimeError::EntityCapacity {
                entity_capacity: self.entity_capacity,
            });
        }
        if matches!(state.lifecycle, EntityLifecycleState::OutboundPending(_)) {
            return Err(EntityServiceRuntimeError::PendingStateRequiresRestore);
        }
        self.validate_owned_state(&state)?;
        let publication = if state.lifecycle == EntityLifecycleState::Active {
            self.plan_publication()?
        } else {
            self.empty_publication()
        };
        let projection = spawn_projection(&state);
        self.entities.insert(entity, state);
        self.commit_publication(publication, entity, projection);
        Ok(())
    }

    pub fn add_observer(
        &mut self,
        observer: StableEntityId,
    ) -> Result<ObserverOutcome, EntityServiceRuntimeError> {
        if self.observers.contains_key(&observer) {
            return Ok(ObserverOutcome::AlreadyPresent);
        }
        if self.observers.len() == self.observer_capacity {
            return Err(EntityServiceRuntimeError::ObserverCapacity {
                observer_capacity: self.observer_capacity,
            });
        }
        let active = self
            .entities
            .iter()
            .filter(|(_, state)| state.lifecycle == EntityLifecycleState::Active)
            .collect::<Vec<_>>();
        if active.len() > self.projection_capacity_per_observer {
            return Err(EntityServiceRuntimeError::ProjectionCapacity {
                observer,
                capacity: self.projection_capacity_per_observer,
            });
        }
        let next = self.preflight_sequences(active.len())?;
        let mut sequence = self.next_projection_sequence;
        let queue = active
            .into_iter()
            .map(|(entity, state)| {
                let projection = EntityProjection {
                    sequence,
                    observer,
                    entity: *entity,
                    kind: spawn_projection(state),
                };
                sequence += 1;
                projection
            })
            .collect();
        self.next_projection_sequence = next;
        self.observers.insert(observer, queue);
        Ok(ObserverOutcome::Added)
    }

    pub fn remove_observer(&mut self, observer: StableEntityId) -> bool {
        self.observers.remove(&observer).is_some()
    }

    pub fn drain_projections(
        &mut self,
        observer: StableEntityId,
        maximum: usize,
    ) -> Result<Vec<EntityProjection>, EntityServiceRuntimeError> {
        let queue = self
            .observers
            .get_mut(&observer)
            .ok_or(EntityServiceRuntimeError::UnknownObserver(observer))?;
        let count = maximum.min(queue.len());
        Ok(queue.drain(..count).collect())
    }

    pub fn activate(
        &mut self,
        header: &EntityCommandHeader,
    ) -> Result<LifecycleOutcome, EntityServiceRuntimeError> {
        if self.command_already_applied(header)? {
            return Ok(LifecycleOutcome::AlreadyApplied);
        }
        let state = self
            .entities
            .get(&header.entity)
            .expect("validated entity remains present");
        if state.lifecycle != EntityLifecycleState::Inactive {
            return Err(EntityServiceRuntimeError::LifecycleMismatch);
        }
        let revision = next_revision(state.revision)?;
        let publication = self.plan_publication()?;
        let state = self
            .entities
            .get_mut(&header.entity)
            .expect("validated entity remains present");
        state.revision = revision;
        state.last_command_sequence = header.sequence;
        state.lifecycle = EntityLifecycleState::Active;
        let projection = spawn_projection(state);
        self.commit_publication(publication, header.entity, projection);
        Ok(LifecycleOutcome::Committed { revision })
    }

    pub fn deactivate(
        &mut self,
        header: &EntityCommandHeader,
    ) -> Result<LifecycleOutcome, EntityServiceRuntimeError> {
        if self.command_already_applied(header)? {
            return Ok(LifecycleOutcome::AlreadyApplied);
        }
        let state = self
            .entities
            .get(&header.entity)
            .expect("validated entity remains present");
        if state.lifecycle != EntityLifecycleState::Active {
            return Err(EntityServiceRuntimeError::LifecycleMismatch);
        }
        let revision = next_revision(state.revision)?;
        let publication = self.plan_publication()?;
        let state = self
            .entities
            .get_mut(&header.entity)
            .expect("validated entity remains present");
        state.revision = revision;
        state.last_command_sequence = header.sequence;
        state.lifecycle = EntityLifecycleState::Inactive;
        self.commit_publication(
            publication,
            header.entity,
            EntityProjectionKind::Remove {
                revision,
                reason: RemovalReason::Deactivated,
            },
        );
        Ok(LifecycleOutcome::Committed { revision })
    }

    pub fn apply_mutation(
        &mut self,
        header: &EntityCommandHeader,
        mutation: EntityMutation,
    ) -> Result<LifecycleOutcome, EntityServiceRuntimeError> {
        if self.command_already_applied(header)? {
            return Ok(LifecycleOutcome::AlreadyApplied);
        }
        self.validate_chunk_owner(&self.key, mutation.chunk)?;
        let state = self
            .entities
            .get(&header.entity)
            .expect("validated entity remains present");
        if state.lifecycle != EntityLifecycleState::Active {
            return Err(EntityServiceRuntimeError::LifecycleMismatch);
        }
        let revision = next_revision(state.revision)?;
        let publication = self.plan_publication()?;
        let state = self
            .entities
            .get_mut(&header.entity)
            .expect("validated entity remains present");
        state.chunk = mutation.chunk;
        state.payload = mutation.payload;
        state.revision = revision;
        state.last_command_sequence = header.sequence;
        let projection = update_projection(state);
        self.commit_publication(publication, header.entity, projection);
        Ok(LifecycleOutcome::Committed { revision })
    }

    pub fn despawn(
        &mut self,
        header: &EntityCommandHeader,
    ) -> Result<LifecycleOutcome, EntityServiceRuntimeError> {
        if self.command_already_applied(header)? {
            return Ok(LifecycleOutcome::AlreadyApplied);
        }
        let state = self
            .entities
            .get(&header.entity)
            .expect("validated entity remains present");
        if matches!(state.lifecycle, EntityLifecycleState::OutboundPending(_)) {
            return Err(EntityServiceRuntimeError::LifecycleMismatch);
        }
        let revision = next_revision(state.revision)?;
        let publication = if state.lifecycle == EntityLifecycleState::Active {
            self.plan_publication()?
        } else {
            self.empty_publication()
        };
        self.entities.remove(&header.entity);
        self.commit_publication(
            publication,
            header.entity,
            EntityProjectionKind::Remove {
                revision,
                reason: RemovalReason::Despawned,
            },
        );
        Ok(LifecycleOutcome::Committed { revision })
    }

    pub fn prepare_transfer(
        &mut self,
        request: EntityTransferRequest,
    ) -> Result<EntityTransfer, EntityServiceRuntimeError> {
        self.validate_transfer_request(&request)?;
        let command = EntityCommandHeader {
            region: request.source.clone(),
            generation: request.source_generation,
            entity: request.entity,
            expected_revision: request.expected_revision,
            sequence: request.sequence,
        };
        if self.command_already_applied(&command)? {
            let state = self
                .entities
                .get(&request.entity)
                .expect("validated entity remains present");
            let EntityLifecycleState::OutboundPending(pending) = &state.lifecycle else {
                return Err(EntityServiceRuntimeError::TransferReplayMismatch);
            };
            if pending.tick != request.tick
                || pending.target != request.target
                || pending.target_generation != request.target_generation
                || pending.source_sequence != request.sequence
                || pending.candidate_chunk != request.candidate.chunk
                || pending.candidate_payload != request.candidate.payload
                || pending.candidate_revision.checked_sub(1) != Some(request.expected_revision)
            {
                return Err(EntityServiceRuntimeError::TransferReplayMismatch);
            }
            return self.retry_transfer(request.entity);
        }
        self.validate_chunk_owner(&request.target, request.candidate.chunk)?;
        let state = self
            .entities
            .get(&request.entity)
            .expect("validated entity remains present");
        if state.lifecycle != EntityLifecycleState::Active {
            return Err(EntityServiceRuntimeError::LifecycleMismatch);
        }
        let candidate_revision = next_revision(state.revision)?;
        let pending = OutboundEntityTransfer {
            tick: request.tick,
            target: request.target.clone(),
            target_generation: request.target_generation,
            source_sequence: request.sequence,
            candidate_chunk: request.candidate.chunk,
            candidate_revision,
            candidate_payload: request.candidate.payload,
        };
        let transfer = self.build_transfer(request.entity, state, &pending)?;
        let publication = self.plan_publication()?;
        let state = self
            .entities
            .get_mut(&request.entity)
            .expect("validated entity remains present");
        state.revision = candidate_revision;
        state.last_command_sequence = request.sequence;
        state.lifecycle = EntityLifecycleState::OutboundPending(pending);
        self.commit_publication(
            publication,
            request.entity,
            EntityProjectionKind::Remove {
                revision: candidate_revision,
                reason: RemovalReason::Transferred,
            },
        );
        Ok(transfer)
    }

    pub fn retry_transfer(
        &self,
        entity: StableEntityId,
    ) -> Result<EntityTransfer, EntityServiceRuntimeError> {
        let state = self
            .entities
            .get(&entity)
            .ok_or(EntityServiceRuntimeError::UnknownEntity(entity))?;
        let EntityLifecycleState::OutboundPending(pending) = &state.lifecycle else {
            return Err(EntityServiceRuntimeError::LifecycleMismatch);
        };
        self.build_transfer(entity, state, pending)
    }

    pub fn abort_transfer(
        &mut self,
        entity: StableEntityId,
        source_sequence: u64,
    ) -> Result<(), EntityServiceRuntimeError> {
        let state = self
            .entities
            .get(&entity)
            .ok_or(EntityServiceRuntimeError::UnknownEntity(entity))?;
        let EntityLifecycleState::OutboundPending(pending) = &state.lifecycle else {
            return Err(EntityServiceRuntimeError::LifecycleMismatch);
        };
        if pending.source_sequence != source_sequence {
            return Err(EntityServiceRuntimeError::TransferReceiptMismatch);
        }
        let publication = self.plan_publication()?;
        let state = self
            .entities
            .get_mut(&entity)
            .expect("validated entity remains present");
        state.lifecycle = EntityLifecycleState::Active;
        let projection = spawn_projection(state);
        self.commit_publication(publication, entity, projection);
        Ok(())
    }

    pub fn accept_transfer(
        &mut self,
        transfer: &EntityTransfer,
    ) -> Result<TransferAcceptance, EntityServiceRuntimeError> {
        if transfer.target() != &self.key {
            return Err(EntityServiceRuntimeError::WrongRegion);
        }
        if transfer.target_generation() != self.generation {
            return Err(EntityServiceRuntimeError::StaleGeneration {
                expected: self.generation,
                actual: transfer.target_generation(),
            });
        }
        if transfer.role() != TransferRole::Entity {
            return Err(EntityServiceRuntimeError::WrongTransferRole);
        }
        let key = AppliedTransferKey::from_transfer(transfer);
        let receipt = EntityTransferReceipt::from_transfer(transfer);
        if self.applied_transfers.contains(&key) {
            return Ok(TransferAcceptance::AlreadyApplied(receipt));
        }
        if self.applied_transfers.len() == self.receipt_capacity {
            return Err(EntityServiceRuntimeError::ReceiptCapacity {
                receipt_capacity: self.receipt_capacity,
            });
        }
        if self.entities.contains_key(&transfer.stable_id()) {
            return Err(EntityServiceRuntimeError::DuplicateEntity(
                transfer.stable_id(),
            ));
        }
        if self.entities.len() == self.entity_capacity {
            return Err(EntityServiceRuntimeError::EntityCapacity {
                entity_capacity: self.entity_capacity,
            });
        }
        let state = decode_transfer_state(transfer.state())?;
        if &state.kind != transfer.kind() {
            return Err(EntityServiceRuntimeError::TransferKindMismatch);
        }
        self.validate_owned_state(&state)?;
        let publication = self.plan_publication()?;
        let projection = spawn_projection(&state);
        self.entities.insert(transfer.stable_id(), state);
        self.applied_transfers.insert(key);
        self.commit_publication(publication, transfer.stable_id(), projection);
        Ok(TransferAcceptance::Accepted(receipt))
    }

    pub fn commit_transfer(
        &mut self,
        receipt: &EntityTransferReceipt,
    ) -> Result<(), EntityServiceRuntimeError> {
        if receipt.source != self.key || receipt.source_generation != self.generation {
            return Err(EntityServiceRuntimeError::TransferReceiptMismatch);
        }
        let state = self
            .entities
            .get(&receipt.entity)
            .ok_or(EntityServiceRuntimeError::UnknownEntity(receipt.entity))?;
        let EntityLifecycleState::OutboundPending(pending) = &state.lifecycle else {
            return Err(EntityServiceRuntimeError::LifecycleMismatch);
        };
        if receipt.target != pending.target
            || receipt.target_generation != pending.target_generation
            || receipt.source_sequence != pending.source_sequence
            || receipt.tick != pending.tick
        {
            return Err(EntityServiceRuntimeError::TransferReceiptMismatch);
        }
        self.entities.remove(&receipt.entity);
        Ok(())
    }

    pub fn prune_applied_transfers(&mut self, through: GameTick) {
        self.applied_transfers
            .retain(|receipt| receipt.tick > through);
    }

    fn command_already_applied(
        &self,
        header: &EntityCommandHeader,
    ) -> Result<bool, EntityServiceRuntimeError> {
        if header.region != self.key {
            return Err(EntityServiceRuntimeError::WrongRegion);
        }
        if header.generation != self.generation {
            return Err(EntityServiceRuntimeError::StaleGeneration {
                expected: self.generation,
                actual: header.generation,
            });
        }
        let state = self
            .entities
            .get(&header.entity)
            .ok_or(EntityServiceRuntimeError::UnknownEntity(header.entity))?;
        if header.sequence <= state.last_command_sequence {
            return Ok(true);
        }
        let expected_sequence = state.last_command_sequence.checked_add(1).ok_or(
            EntityServiceRuntimeError::CommandSequenceExhausted(header.entity),
        )?;
        if header.sequence != expected_sequence {
            return Err(EntityServiceRuntimeError::CommandSequenceGap {
                expected: expected_sequence,
                actual: header.sequence,
            });
        }
        if header.expected_revision != state.revision {
            return Err(EntityServiceRuntimeError::RevisionMismatch {
                expected: state.revision,
                actual: header.expected_revision,
            });
        }
        Ok(false)
    }

    fn validate_transfer_request(
        &self,
        request: &EntityTransferRequest,
    ) -> Result<(), EntityServiceRuntimeError> {
        if request.source != self.key {
            return Err(EntityServiceRuntimeError::WrongRegion);
        }
        if request.source_generation != self.generation {
            return Err(EntityServiceRuntimeError::StaleGeneration {
                expected: self.generation,
                actual: request.source_generation,
            });
        }
        if request.source == request.target {
            return Err(EntityServiceRuntimeError::SelfTransfer);
        }
        if request.source.world() != request.target.world()
            || request.source.dimension() != request.target.dimension()
            || request.source.mapping_version() != request.target.mapping_version()
        {
            return Err(EntityServiceRuntimeError::IncompatibleTransfer);
        }
        Ok(())
    }

    fn build_transfer(
        &self,
        entity: StableEntityId,
        state: &EntityPersistentState,
        pending: &OutboundEntityTransfer,
    ) -> Result<EntityTransfer, EntityServiceRuntimeError> {
        EntityTransfer::new(
            EntityTransferHeader {
                tick: pending.tick,
                source: self.key.clone(),
                target: pending.target.clone(),
                source_generation: self.generation,
                target_generation: pending.target_generation,
                source_sequence: pending.source_sequence,
                stable_id: entity,
                role: TransferRole::Entity,
            },
            state.kind.clone(),
            encode_transfer_state(state, pending)?,
        )
        .map_err(Into::into)
    }

    fn validate_owned_state(
        &self,
        state: &EntityPersistentState,
    ) -> Result<(), EntityServiceRuntimeError> {
        self.validate_chunk_owner(&self.key, state.chunk)?;
        if let EntityLifecycleState::OutboundPending(pending) = &state.lifecycle {
            if pending.target == self.key {
                return Err(EntityServiceRuntimeError::SelfTransfer);
            }
            if pending.target.world() != self.key.world()
                || pending.target.dimension() != self.key.dimension()
                || pending.target.mapping_version() != self.key.mapping_version()
            {
                return Err(EntityServiceRuntimeError::IncompatibleTransfer);
            }
            self.validate_chunk_owner(&pending.target, pending.candidate_chunk)?;
            if pending.candidate_revision != state.revision {
                return Err(EntityServiceRuntimeError::PendingRevisionMismatch);
            }
            if pending.source_sequence != state.last_command_sequence {
                return Err(EntityServiceRuntimeError::PendingSequenceMismatch);
            }
        }
        Ok(())
    }

    fn validate_chunk_owner(
        &self,
        expected: &SimulationRegionKey,
        chunk: ChunkPos,
    ) -> Result<(), EntityServiceRuntimeError> {
        let actual =
            self.mapping
                .region_for_chunk(expected.world(), expected.dimension().clone(), chunk);
        if &actual == expected {
            Ok(())
        } else {
            Err(EntityServiceRuntimeError::WrongChunkOwner { chunk })
        }
    }

    fn plan_publication(&self) -> Result<PublicationPlan, EntityServiceRuntimeError> {
        for (observer, queue) in &self.observers {
            if queue.len() == self.projection_capacity_per_observer {
                return Err(EntityServiceRuntimeError::ProjectionCapacity {
                    observer: *observer,
                    capacity: self.projection_capacity_per_observer,
                });
            }
        }
        let next = self.preflight_sequences(self.observers.len())?;
        let events = self
            .observers
            .keys()
            .copied()
            .enumerate()
            .map(|(offset, observer)| {
                (
                    observer,
                    self.next_projection_sequence + u64::try_from(offset).unwrap(),
                )
            })
            .collect();
        Ok(PublicationPlan { events, next })
    }

    fn empty_publication(&self) -> PublicationPlan {
        PublicationPlan {
            events: Vec::new(),
            next: self.next_projection_sequence,
        }
    }

    fn preflight_sequences(&self, count: usize) -> Result<u64, EntityServiceRuntimeError> {
        let count =
            u64::try_from(count).map_err(|_| EntityServiceRuntimeError::ProjectionExhausted)?;
        self.next_projection_sequence
            .checked_add(count)
            .ok_or(EntityServiceRuntimeError::ProjectionExhausted)
    }

    fn commit_publication(
        &mut self,
        publication: PublicationPlan,
        entity: StableEntityId,
        kind: EntityProjectionKind,
    ) {
        for (observer, sequence) in publication.events {
            self.observers
                .get_mut(&observer)
                .expect("planned observer remains installed")
                .push_back(EntityProjection {
                    sequence,
                    observer,
                    entity,
                    kind: kind.clone(),
                });
        }
        self.next_projection_sequence = publication.next;
    }
}

#[derive(Debug)]
struct PublicationPlan {
    events: Vec<(StableEntityId, u64)>,
    next: u64,
}

fn spawn_projection(state: &EntityPersistentState) -> EntityProjectionKind {
    EntityProjectionKind::Spawn {
        kind: state.kind.clone(),
        chunk: state.chunk,
        revision: state.revision,
        state_digest: state.payload.digest(),
    }
}

fn update_projection(state: &EntityPersistentState) -> EntityProjectionKind {
    EntityProjectionKind::Update {
        chunk: state.chunk,
        revision: state.revision,
        state_digest: state.payload.digest(),
    }
}

fn next_revision(current: u64) -> Result<u64, EntityServiceRuntimeError> {
    current
        .checked_add(1)
        .ok_or(EntityServiceRuntimeError::RevisionExhausted)
}

fn validate_capacity(kind: &'static str, capacity: usize) -> Result<(), EntityServiceRuntimeError> {
    if capacity == 0 {
        Err(EntityServiceRuntimeError::ZeroCapacity { kind })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EntityServiceRuntimeError {
    #[error("{kind} capacity cannot be zero")]
    ZeroCapacity { kind: &'static str },
    #[error("Region mapping version does not match the runtime key")]
    MappingVersionMismatch,
    #[error("operation targets the wrong Region")]
    WrongRegion,
    #[error("operation generation {actual:?} does not match {expected:?}")]
    StaleGeneration {
        expected: ActivationGeneration,
        actual: ActivationGeneration,
    },
    #[error("entity capacity {entity_capacity} is full")]
    EntityCapacity { entity_capacity: usize },
    #[error("observer capacity {observer_capacity} is full")]
    ObserverCapacity { observer_capacity: usize },
    #[error("observer {observer:?} projection capacity {capacity} is full")]
    ProjectionCapacity {
        observer: StableEntityId,
        capacity: usize,
    },
    #[error("transfer receipt capacity {receipt_capacity} is full")]
    ReceiptCapacity { receipt_capacity: usize },
    #[error("entity {0:?} is already present")]
    DuplicateEntity(StableEntityId),
    #[error("entity {0:?} is not owned here")]
    UnknownEntity(StableEntityId),
    #[error("observer {0:?} is not installed")]
    UnknownObserver(StableEntityId),
    #[error("snapshot contains a duplicate applied transfer receipt")]
    DuplicateReceipt,
    #[error("entity command sequence expected {expected}, got {actual}")]
    CommandSequenceGap { expected: u64, actual: u64 },
    #[error("entity {0:?} command sequence is exhausted")]
    CommandSequenceExhausted(StableEntityId),
    #[error("entity revision expected {expected}, got {actual}")]
    RevisionMismatch { expected: u64, actual: u64 },
    #[error("entity revision is exhausted")]
    RevisionExhausted,
    #[error("projection sequence is exhausted")]
    ProjectionExhausted,
    #[error("entity lifecycle does not admit this operation")]
    LifecycleMismatch,
    #[error("outbound-pending state may only enter through restore")]
    PendingStateRequiresRestore,
    #[error("chunk {chunk:?} is not owned by the expected Region")]
    WrongChunkOwner { chunk: ChunkPos },
    #[error("entity transfer cannot target its source Region")]
    SelfTransfer,
    #[error("entity transfer endpoints are incompatible")]
    IncompatibleTransfer,
    #[error("entity transfer has the wrong role")]
    WrongTransferRole,
    #[error("entity transfer kind does not match its state")]
    TransferKindMismatch,
    #[error("entity transfer receipt does not match pending state")]
    TransferReceiptMismatch,
    #[error("entity transfer replay does not match the pending transfer")]
    TransferReplayMismatch,
    #[error("pending transfer revision does not match entity revision")]
    PendingRevisionMismatch,
    #[error("pending transfer sequence does not match entity command sequence")]
    PendingSequenceMismatch,
    #[error(transparent)]
    Continuity(#[from] EntityServiceContinuityError),
    #[error(transparent)]
    Transfer(#[from] EntityTransferError),
    #[error(transparent)]
    Migration(#[from] ContinuityMigrationError),
}
