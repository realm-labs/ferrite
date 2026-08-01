//! Topology-independent Region message conformance and fault harness.

pub mod cluster;
pub mod layout;
pub mod partition;

use crate::lattice::authority::RegionAuthorityError;
use crate::lattice::remoting::RemotingAdapterError;
use crate::lattice::spatial::SpatialAdapterError;
use ferrite_foundation::identity::ActivationGenerationError;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_persistence::recovery::RecoveryError;
use ferrite_persistence::snapshot::SnapshotError;
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TopologyError {
    #[error("topology must contain at least two Regions and one node")]
    EmptyLayout,
    #[error("each topology world/dimension/mapping domain requires at least two Regions")]
    SingletonRegionDomain,
    #[error("topology world count must be non-zero with at least two Regions per world")]
    InvalidWorldCount,
    #[error("topology contains duplicate Region {0:?}")]
    DuplicateRegion(SimulationRegionKey),
    #[error("topology node count cannot be zero")]
    ZeroNodes,
    #[error("topology mailbox capacity cannot be zero")]
    ZeroMailboxCapacity,
    #[error("topology does not contain Region {0:?}")]
    UnknownRegion(SimulationRegionKey),
    #[error("topology does not contain node {0}")]
    UnknownNode(u16),
    #[error("partition {node} cannot own Region assigned to node {assigned}")]
    WrongPartition { node: u16, assigned: u16 },
    #[error("Region {region:?} already exists in partition {node}")]
    DuplicatePartitionRegion {
        node: u16,
        region: SimulationRegionKey,
    },
    #[error("expected tick {expected:?}, got {actual:?}")]
    UnexpectedTick {
        expected: GameTick,
        actual: GameTick,
    },
    #[error("remote Region message is fenced by a stale source or target generation")]
    StaleGeneration,
    #[error("remote Region message targets a different partition")]
    WrongTargetPartition,
    #[error("remote Region message source is not the target's ring predecessor")]
    UnexpectedSource,
    #[error("remote Region message kind, sequence, or payload is invalid")]
    InvalidBoundaryMessage,
    #[error("remote Region mailbox reached its {capacity}-message bound")]
    MailboxFull { capacity: usize },
    #[error("duplicate remote Region identity carries conflicting content")]
    ConflictingDuplicate,
    #[error("topology partition cannot drain with pending remote messages")]
    DrainWithPendingMessages,
    #[error("topology partition drain did not fence admission and request Region drain")]
    DrainDidNotFence,
    #[error("tick {tick:?} is missing required boundary input for {target:?}")]
    MissingRequiredBoundary {
        tick: GameTick,
        target: SimulationRegionKey,
    },
    #[error("Region generation cannot advance")]
    GenerationExhausted,
    #[error("topology arithmetic overflowed")]
    ArithmeticOverflow,
    #[error("topology snapshot does not match the active layout")]
    SnapshotLayoutMismatch,
    #[error("Lattice placement claim is closed for Region {0:?}")]
    AuthorityClosed(SimulationRegionKey),
    #[error("durable topology recovery point is malformed")]
    InvalidRecoveryPoint,
    #[error(transparent)]
    Remoting(#[from] RemotingAdapterError),
    #[error(transparent)]
    Authority(#[from] RegionAuthorityError),
    #[error(transparent)]
    Spatial(#[from] SpatialAdapterError),
    #[error(transparent)]
    Generation(#[from] ActivationGenerationError),
    #[error(transparent)]
    Recovery(#[from] RecoveryError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}
