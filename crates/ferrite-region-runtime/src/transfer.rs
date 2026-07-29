//! Deterministic semantic entity and player transfer records.

use bevy_ecs::prelude::Component;
use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::tick::GameTick;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const MAX_ENTITY_TRANSFER_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferRole {
    Entity,
    Player,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTransferHeader {
    pub tick: GameTick,
    pub source: SimulationRegionKey,
    pub target: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub target_generation: ActivationGeneration,
    pub source_sequence: u64,
    pub stable_id: StableEntityId,
    pub role: TransferRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTransfer {
    header: EntityTransferHeader,
    kind: ResourceId,
    state: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedEntityTransfer {
    pub tick: GameTick,
    pub source: SimulationRegionKey,
    pub target: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub target_generation: ActivationGeneration,
    pub stable_id: StableEntityId,
    pub role: TransferRole,
}

impl CommittedEntityTransfer {
    pub(crate) fn from_transfer(transfer: &EntityTransfer) -> Self {
        Self {
            tick: transfer.tick(),
            source: transfer.source().clone(),
            target: transfer.target().clone(),
            source_generation: transfer.source_generation(),
            target_generation: transfer.target_generation(),
            stable_id: transfer.stable_id(),
            role: transfer.role(),
        }
    }
}

impl EntityTransfer {
    pub fn new(
        header: EntityTransferHeader,
        kind: ResourceId,
        state: Vec<u8>,
    ) -> Result<Self, EntityTransferError> {
        validate_endpoints(&header.source, &header.target)?;
        if state.len() > MAX_ENTITY_TRANSFER_BYTES {
            return Err(EntityTransferError::StateTooLarge {
                actual: state.len(),
                maximum: MAX_ENTITY_TRANSFER_BYTES,
            });
        }
        Ok(Self {
            header,
            kind,
            state,
        })
    }

    pub const fn tick(&self) -> GameTick {
        self.header.tick
    }

    pub const fn source(&self) -> &SimulationRegionKey {
        &self.header.source
    }

    pub const fn target(&self) -> &SimulationRegionKey {
        &self.header.target
    }

    pub const fn source_generation(&self) -> ActivationGeneration {
        self.header.source_generation
    }

    pub const fn target_generation(&self) -> ActivationGeneration {
        self.header.target_generation
    }

    pub const fn source_sequence(&self) -> u64 {
        self.header.source_sequence
    }

    pub const fn stable_id(&self) -> StableEntityId {
        self.header.stable_id
    }

    pub const fn role(&self) -> TransferRole {
        self.header.role
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    fn order_key(&self) -> EntityTransferOrderKey {
        EntityTransferOrderKey {
            tick: self.tick(),
            target: self.target().clone(),
            source: self.source().clone(),
            source_sequence: self.source_sequence(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Component)]
#[component(immutable)]
pub struct TransferredEntityState {
    role: TransferRole,
    kind: ResourceId,
    state: Vec<u8>,
}

impl TransferredEntityState {
    pub(crate) fn from_transfer(transfer: &EntityTransfer) -> Self {
        Self {
            role: transfer.role(),
            kind: transfer.kind().clone(),
            state: transfer.state().to_vec(),
        }
    }

    pub const fn role(&self) -> TransferRole {
        self.role
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }
}

#[derive(Debug)]
pub(crate) struct EntityTransferQueue {
    capacity: usize,
    pending: BTreeMap<EntityTransferOrderKey, EntityTransfer>,
    admitted: BTreeSet<EntityTransferOrderKey>,
}

impl EntityTransferQueue {
    pub(crate) fn new(capacity: usize) -> Result<Self, EntityTransferError> {
        if capacity == 0 {
            return Err(EntityTransferError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            pending: BTreeMap::new(),
            admitted: BTreeSet::new(),
        })
    }

    pub(crate) fn admit(
        &mut self,
        transfer: EntityTransfer,
        source_generation: ActivationGeneration,
        target_generation: ActivationGeneration,
        committed_tick: GameTick,
    ) -> Result<(), EntityTransferError> {
        if transfer.source_generation() != source_generation {
            return Err(EntityTransferError::StaleSourceGeneration);
        }
        if transfer.target_generation() != target_generation {
            return Err(EntityTransferError::StaleTargetGeneration);
        }
        if transfer.tick() <= committed_tick {
            return Err(EntityTransferError::AlreadyCommitted);
        }
        let key = transfer.order_key();
        if self.admitted.contains(&key) {
            return Err(EntityTransferError::Duplicate);
        }
        if self.pending.len() == self.capacity || self.admitted.len() == self.capacity {
            return Err(EntityTransferError::Full {
                capacity: self.capacity,
            });
        }
        self.admitted.insert(key.clone());
        self.pending.insert(key, transfer);
        Ok(())
    }

    pub(crate) fn drain_tick(&mut self, tick: GameTick) -> Vec<EntityTransfer> {
        let keys = self
            .pending
            .keys()
            .filter(|key| key.tick == tick)
            .cloned()
            .collect::<Vec<_>>();
        keys.into_iter()
            .filter_map(|key| self.pending.remove(&key))
            .collect()
    }

    pub(crate) fn prune_committed(&mut self, committed_tick: GameTick) {
        self.admitted.retain(|key| key.tick > committed_tick);
    }

    pub(crate) fn has_tick_at_or_before(&self, tick: GameTick) -> bool {
        self.pending.keys().any(|key| key.tick <= tick)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EntityTransferOrderKey {
    tick: GameTick,
    target: SimulationRegionKey,
    source: SimulationRegionKey,
    source_sequence: u64,
}

fn validate_endpoints(
    source: &SimulationRegionKey,
    target: &SimulationRegionKey,
) -> Result<(), EntityTransferError> {
    if source == target {
        return Err(EntityTransferError::SelfTarget);
    }
    if source.world() != target.world()
        || source.dimension() != target.dimension()
        || source.mapping_version() != target.mapping_version()
    {
        return Err(EntityTransferError::IncompatibleEndpoints);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EntityTransferError {
    #[error("entity-transfer queue capacity cannot be zero")]
    ZeroCapacity,
    #[error("an entity transfer cannot target its source Region")]
    SelfTarget,
    #[error("entity transfer endpoints are in different ownership domains")]
    IncompatibleEndpoints,
    #[error("entity transfer state has {actual} bytes, exceeding the {maximum}-byte limit")]
    StateTooLarge { actual: usize, maximum: usize },
    #[error("entity transfer has a stale source generation")]
    StaleSourceGeneration,
    #[error("entity transfer has a stale target generation")]
    StaleTargetGeneration,
    #[error("entity transfer targets an already committed tick")]
    AlreadyCommitted,
    #[error("entity transfer order key is already admitted")]
    Duplicate,
    #[error("entity-transfer queue reached its {capacity}-transfer bound")]
    Full { capacity: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};

    fn region(x: i32) -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(x, 0),
            RegionMappingVersion::V1,
        )
    }

    fn transfer(source: i32, target: i32, sequence: u64) -> EntityTransfer {
        EntityTransfer::new(
            EntityTransferHeader {
                tick: GameTick::new(1),
                source: region(source),
                target: region(target),
                source_generation: ActivationGeneration::INITIAL,
                target_generation: ActivationGeneration::INITIAL,
                source_sequence: sequence,
                stable_id: StableEntityId::new(u128::from(sequence) + 1).unwrap(),
                role: TransferRole::Player,
            },
            ResourceId::minecraft("player").unwrap(),
            vec![1, 2],
        )
        .unwrap()
    }

    #[test]
    fn transfers_sort_by_target_source_and_sequence() {
        let mut queue = EntityTransferQueue::new(4).unwrap();
        for transfer in [transfer(2, 0, 2), transfer(1, 0, 1)] {
            queue
                .admit(
                    transfer,
                    ActivationGeneration::INITIAL,
                    ActivationGeneration::INITIAL,
                    GameTick::ZERO,
                )
                .unwrap();
        }
        let drained = queue.drain_tick(GameTick::new(1));
        assert_eq!(drained[0].source().coordinate().x(), 1);
        assert_eq!(drained[0].role(), TransferRole::Player);
    }

    #[test]
    fn stale_target_and_duplicate_transfers_fail_closed() {
        let transfer = transfer(1, 0, 1);
        let mut queue = EntityTransferQueue::new(1).unwrap();
        assert!(
            queue
                .admit(
                    transfer.clone(),
                    ActivationGeneration::INITIAL,
                    ActivationGeneration::new(2).unwrap(),
                    GameTick::ZERO,
                )
                .is_err()
        );
        queue
            .admit(
                transfer.clone(),
                ActivationGeneration::INITIAL,
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
            )
            .unwrap();
        assert!(
            queue
                .admit(
                    transfer,
                    ActivationGeneration::INITIAL,
                    ActivationGeneration::INITIAL,
                    GameTick::ZERO,
                )
                .is_err()
        );
    }
}
