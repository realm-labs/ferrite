//! Logical tick and fixed phase identities.

use thiserror::Error;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameTick(u64);

impl GameTick {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn checked_next(self) -> Result<Self, TickError> {
        match self.0.checked_add(1) {
            Some(value) => Ok(Self(value)),
            None => Err(TickError::Exhausted),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TickPhase {
    Begin = 0,
    Ingress = 1,
    NormalizeCommands = 2,
    PlayerIntent = 3,
    ScheduledBlocks = 4,
    RandomBlocks = 5,
    ImmediateNeighbors = 6,
    BlockEntities = 7,
    Fluids = 8,
    Redstone = 9,
    EntityAi = 10,
    EntityPhysics = 11,
    EntityResolution = 12,
    DeferredChanges = 13,
    ResultingNeighbors = 14,
    EcsStructuralChanges = 15,
    EmitBoundary = 16,
    ReconcileBoundary = 17,
    Replication = 18,
    Commit = 19,
}

impl TickPhase {
    pub const ALL: [Self; 20] = [
        Self::Begin,
        Self::Ingress,
        Self::NormalizeCommands,
        Self::PlayerIntent,
        Self::ScheduledBlocks,
        Self::RandomBlocks,
        Self::ImmediateNeighbors,
        Self::BlockEntities,
        Self::Fluids,
        Self::Redstone,
        Self::EntityAi,
        Self::EntityPhysics,
        Self::EntityResolution,
        Self::DeferredChanges,
        Self::ResultingNeighbors,
        Self::EcsStructuralChanges,
        Self::EmitBoundary,
        Self::ReconcileBoundary,
        Self::Replication,
        Self::Commit,
    ];

    pub const fn stable_tag(self) -> u8 {
        self as u8
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Begin => Some(Self::Ingress),
            Self::Ingress => Some(Self::NormalizeCommands),
            Self::NormalizeCommands => Some(Self::PlayerIntent),
            Self::PlayerIntent => Some(Self::ScheduledBlocks),
            Self::ScheduledBlocks => Some(Self::RandomBlocks),
            Self::RandomBlocks => Some(Self::ImmediateNeighbors),
            Self::ImmediateNeighbors => Some(Self::BlockEntities),
            Self::BlockEntities => Some(Self::Fluids),
            Self::Fluids => Some(Self::Redstone),
            Self::Redstone => Some(Self::EntityAi),
            Self::EntityAi => Some(Self::EntityPhysics),
            Self::EntityPhysics => Some(Self::EntityResolution),
            Self::EntityResolution => Some(Self::DeferredChanges),
            Self::DeferredChanges => Some(Self::ResultingNeighbors),
            Self::ResultingNeighbors => Some(Self::EcsStructuralChanges),
            Self::EcsStructuralChanges => Some(Self::EmitBoundary),
            Self::EmitBoundary => Some(Self::ReconcileBoundary),
            Self::ReconcileBoundary => Some(Self::Replication),
            Self::Replication => Some(Self::Commit),
            Self::Commit => None,
        }
    }

    pub const fn contract(self) -> PhaseContract {
        let barrier = match self {
            Self::EcsStructuralChanges => PhaseBarrier::StructuralChanges,
            Self::EmitBoundary => PhaseBarrier::BoundaryEmission,
            Self::ReconcileBoundary => PhaseBarrier::RequiredReconciliation,
            Self::Commit => PhaseBarrier::Commit,
            _ => PhaseBarrier::None,
        };
        PhaseContract {
            phase: self,
            barrier,
            overflow: OverflowPolicy::RetainAndBackpressure,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseContract {
    phase: TickPhase,
    barrier: PhaseBarrier,
    overflow: OverflowPolicy,
}

impl PhaseContract {
    pub const fn phase(self) -> TickPhase {
        self.phase
    }

    pub const fn barrier(self) -> PhaseBarrier {
        self.barrier
    }

    pub const fn overflow(self) -> OverflowPolicy {
        self.overflow
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseBarrier {
    None,
    StructuralChanges,
    BoundaryEmission,
    RequiredReconciliation,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    RetainAndBackpressure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TickError {
    #[error("logical tick is exhausted")]
    Exhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_tags_and_successors_are_locked() {
        for (index, phase) in TickPhase::ALL.into_iter().enumerate() {
            assert_eq!(usize::from(phase.stable_tag()), index);
            assert_eq!(phase.next(), TickPhase::ALL.get(index + 1).copied());
        }
        assert_eq!(
            TickPhase::ReconcileBoundary.contract().barrier(),
            PhaseBarrier::RequiredReconciliation
        );
    }
}
