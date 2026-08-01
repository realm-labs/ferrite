//! Typed, generation-fenced transactions for mechanics crossing a Region boundary.

use crate::simulation::budget::SimulationQueueKind;
use crate::simulation::continuity::{AppliedBoundaryReceipt, ScheduledQueueKind};
use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::scheduled_tick::record::TickPriority;
use ferrite_simulation::tick::GameTick;
use ferrite_world::id::BlockStateId;
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BoundaryMechanic {
    Neighbor,
    Fluid,
    Redstone,
    Piston,
    Explosion,
    Lighting,
}

impl BoundaryMechanic {
    pub const fn queue_kind(self) -> SimulationQueueKind {
        match self {
            Self::Neighbor | Self::Piston | Self::Explosion => {
                SimulationQueueKind::ImmediateNeighbors
            }
            Self::Fluid => SimulationQueueKind::Fluids,
            Self::Redstone => SimulationQueueKind::Redstone,
            Self::Lighting => SimulationQueueKind::Lighting,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryMutation {
    pub order: u32,
    pub position: BlockPos,
    pub expected: BlockStateId,
    pub replacement: BlockStateId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundarySchedule {
    pub order: u32,
    pub kind: ScheduledQueueKind,
    pub type_identity: ResourceId,
    pub position: BlockPos,
    pub delay: i32,
    pub priority: TickPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryTransactionLimits {
    pub maximum_mutations: usize,
    pub maximum_schedules: usize,
}

impl BoundaryTransactionLimits {
    pub const fn new(maximum_mutations: usize, maximum_schedules: usize) -> Self {
        Self {
            maximum_mutations,
            maximum_schedules,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanicBoundaryTransaction {
    tick: GameTick,
    source: SimulationRegionKey,
    source_generation: ActivationGeneration,
    target: SimulationRegionKey,
    target_generation: ActivationGeneration,
    source_sequence: u64,
    mechanic: BoundaryMechanic,
    mutations: Box<[BoundaryMutation]>,
    schedules: Box<[BoundarySchedule]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryTransactionHeader {
    pub tick: GameTick,
    pub source: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub target: SimulationRegionKey,
    pub target_generation: ActivationGeneration,
    pub source_sequence: u64,
}

impl MechanicBoundaryTransaction {
    pub fn new(
        header: BoundaryTransactionHeader,
        mechanic: BoundaryMechanic,
        mut mutations: Vec<BoundaryMutation>,
        mut schedules: Vec<BoundarySchedule>,
        mapping: RegionMapping,
        limits: BoundaryTransactionLimits,
    ) -> Result<Self, BoundaryTransactionError> {
        validate_endpoints(&header.source, &header.target, mapping)?;
        validate_limit(
            BoundaryCollection::Mutations,
            mutations.len(),
            limits.maximum_mutations,
        )?;
        validate_limit(
            BoundaryCollection::Schedules,
            schedules.len(),
            limits.maximum_schedules,
        )?;
        mutations.sort_by_key(|mutation| mutation.order);
        schedules.sort_by_key(|schedule| schedule.order);
        reject_duplicate_orders(
            BoundaryCollection::Mutations,
            mutations.iter().map(|mutation| mutation.order),
        )?;
        reject_duplicate_orders(
            BoundaryCollection::Schedules,
            schedules.iter().map(|schedule| schedule.order),
        )?;
        let mut positions = BTreeSet::new();
        for mutation in &mutations {
            validate_target_position(&header.target, mutation.position, mapping)?;
            if !positions.insert(mutation.position) {
                return Err(BoundaryTransactionError::DuplicateMutationPosition(
                    mutation.position,
                ));
            }
        }
        let mut scheduled = BTreeSet::new();
        for schedule in &schedules {
            validate_target_position(&header.target, schedule.position, mapping)?;
            if !scheduled.insert((
                schedule.kind,
                schedule.type_identity.clone(),
                schedule.position,
            )) {
                return Err(BoundaryTransactionError::DuplicateSchedule {
                    kind: schedule.kind,
                    position: schedule.position,
                });
            }
        }
        Ok(Self {
            tick: header.tick,
            source: header.source,
            source_generation: header.source_generation,
            target: header.target,
            target_generation: header.target_generation,
            source_sequence: header.source_sequence,
            mechanic,
            mutations: mutations.into_boxed_slice(),
            schedules: schedules.into_boxed_slice(),
        })
    }

    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub const fn source(&self) -> &SimulationRegionKey {
        &self.source
    }

    pub const fn source_generation(&self) -> ActivationGeneration {
        self.source_generation
    }

    pub const fn target(&self) -> &SimulationRegionKey {
        &self.target
    }

    pub const fn target_generation(&self) -> ActivationGeneration {
        self.target_generation
    }

    pub const fn source_sequence(&self) -> u64 {
        self.source_sequence
    }

    pub const fn mechanic(&self) -> BoundaryMechanic {
        self.mechanic
    }

    pub const fn mutations(&self) -> &[BoundaryMutation] {
        &self.mutations
    }

    pub const fn schedules(&self) -> &[BoundarySchedule] {
        &self.schedules
    }

    pub fn receipt(&self) -> AppliedBoundaryReceipt {
        AppliedBoundaryReceipt {
            source: self.source.coordinate(),
            source_generation: self.source_generation,
            source_sequence: self.source_sequence,
        }
    }
}

fn validate_endpoints(
    source: &SimulationRegionKey,
    target: &SimulationRegionKey,
    mapping: RegionMapping,
) -> Result<(), BoundaryTransactionError> {
    if source == target {
        return Err(BoundaryTransactionError::SameRegion);
    }
    if source.world() != target.world()
        || source.dimension() != target.dimension()
        || source.mapping_version() != target.mapping_version()
    {
        return Err(BoundaryTransactionError::IncompatibleEndpoints);
    }
    if source.mapping_version() != mapping.version() {
        return Err(BoundaryTransactionError::MappingVersionMismatch);
    }
    Ok(())
}

fn validate_limit(
    collection: BoundaryCollection,
    actual: usize,
    maximum: usize,
) -> Result<(), BoundaryTransactionError> {
    if maximum == 0 {
        return Err(BoundaryTransactionError::ZeroLimit { collection });
    }
    if actual > maximum {
        return Err(BoundaryTransactionError::TooManyEntries {
            collection,
            actual,
            maximum,
        });
    }
    Ok(())
}

fn reject_duplicate_orders(
    collection: BoundaryCollection,
    orders: impl Iterator<Item = u32>,
) -> Result<(), BoundaryTransactionError> {
    let mut previous = None;
    for order in orders {
        if previous == Some(order) {
            return Err(BoundaryTransactionError::DuplicateOrder { collection, order });
        }
        previous = Some(order);
    }
    Ok(())
}

fn validate_target_position(
    target: &SimulationRegionKey,
    position: BlockPos,
    mapping: RegionMapping,
) -> Result<(), BoundaryTransactionError> {
    let actual =
        mapping.region_for_chunk(target.world(), target.dimension().clone(), position.chunk());
    if &actual == target {
        Ok(())
    } else {
        Err(BoundaryTransactionError::WrongTargetOwner { position })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryCollection {
    Mutations,
    Schedules,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BoundaryTransactionError {
    #[error("a boundary transaction must cross between two Regions")]
    SameRegion,
    #[error("boundary transaction endpoints do not share world, dimension, and mapping")]
    IncompatibleEndpoints,
    #[error("boundary transaction mapping does not match its endpoints")]
    MappingVersionMismatch,
    #[error("boundary transaction {collection:?} limit cannot be zero")]
    ZeroLimit { collection: BoundaryCollection },
    #[error("boundary transaction has {actual} {collection:?}, exceeding {maximum}")]
    TooManyEntries {
        collection: BoundaryCollection,
        actual: usize,
        maximum: usize,
    },
    #[error("boundary transaction {collection:?} order {order} is duplicated")]
    DuplicateOrder {
        collection: BoundaryCollection,
        order: u32,
    },
    #[error("boundary transaction mutates {0:?} more than once")]
    DuplicateMutationPosition(BlockPos),
    #[error("boundary transaction schedules duplicate {kind:?} work at {position:?}")]
    DuplicateSchedule {
        kind: ScheduledQueueKind,
        position: BlockPos,
    },
    #[error("boundary transaction position {position:?} is not owned by its target Region")]
    WrongTargetOwner { position: BlockPos },
}
