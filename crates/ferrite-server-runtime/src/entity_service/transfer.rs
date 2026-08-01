use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_region_runtime::transfer::{CommittedEntityTransfer, EntityTransfer, TransferRole};
use ferrite_simulation::tick::GameTick;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppliedTransferKey {
    pub tick: GameTick,
    pub source: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub target_generation: ActivationGeneration,
    pub source_sequence: u64,
    pub entity: StableEntityId,
}

impl AppliedTransferKey {
    #[must_use]
    pub fn from_transfer(transfer: &EntityTransfer) -> Self {
        Self {
            tick: transfer.tick(),
            source: transfer.source().clone(),
            source_generation: transfer.source_generation(),
            target_generation: transfer.target_generation(),
            source_sequence: transfer.source_sequence(),
            entity: transfer.stable_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTransferReceipt {
    pub tick: GameTick,
    pub source: SimulationRegionKey,
    pub target: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub target_generation: ActivationGeneration,
    pub source_sequence: u64,
    pub entity: StableEntityId,
}

impl EntityTransferReceipt {
    #[must_use]
    pub fn from_transfer(transfer: &EntityTransfer) -> Self {
        Self {
            tick: transfer.tick(),
            source: transfer.source().clone(),
            target: transfer.target().clone(),
            source_generation: transfer.source_generation(),
            target_generation: transfer.target_generation(),
            source_sequence: transfer.source_sequence(),
            entity: transfer.stable_id(),
        }
    }

    #[must_use]
    pub fn committed(&self) -> CommittedEntityTransfer {
        CommittedEntityTransfer {
            tick: self.tick,
            source: self.source.clone(),
            target: self.target.clone(),
            source_generation: self.source_generation,
            target_generation: self.target_generation,
            stable_id: self.entity,
            role: TransferRole::Entity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferAcceptance {
    Accepted(EntityTransferReceipt),
    AlreadyApplied(EntityTransferReceipt),
}
