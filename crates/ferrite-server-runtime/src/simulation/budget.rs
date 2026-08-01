//! Atomic bounded-work reservations shared by Simulation integration queues.

use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SimulationQueueKind {
    ScheduledBlocks,
    ScheduledFluids,
    BoundaryTransactions,
    ImmediateNeighbors,
    Fluids,
    Redstone,
    Lighting,
    ProjectionPositions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueuePressure {
    pub used: usize,
    pub capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueReservation {
    amounts: BTreeMap<SimulationQueueKind, usize>,
}

impl QueueReservation {
    pub fn amount(&self, kind: SimulationQueueKind) -> usize {
        self.amounts.get(&kind).copied().unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.amounts.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationQueueBudget {
    capacity: BTreeMap<SimulationQueueKind, usize>,
    used: BTreeMap<SimulationQueueKind, usize>,
}

impl SimulationQueueBudget {
    pub fn new(
        capacities: impl IntoIterator<Item = (SimulationQueueKind, usize)>,
    ) -> Result<Self, QueueBudgetError> {
        let mut capacity = BTreeMap::new();
        for (kind, limit) in capacities {
            if limit == 0 {
                return Err(QueueBudgetError::ZeroCapacity { kind });
            }
            if capacity.insert(kind, limit).is_some() {
                return Err(QueueBudgetError::DuplicateKind { kind });
            }
        }
        Ok(Self {
            capacity,
            used: BTreeMap::new(),
        })
    }

    pub fn pressure(&self, kind: SimulationQueueKind) -> Result<QueuePressure, QueueBudgetError> {
        Ok(QueuePressure {
            used: self.used.get(&kind).copied().unwrap_or(0),
            capacity: self.capacity(kind)?,
        })
    }

    pub fn try_reserve(
        &mut self,
        requests: impl IntoIterator<Item = (SimulationQueueKind, usize)>,
    ) -> Result<QueueReservation, QueueBudgetError> {
        let mut amounts = BTreeMap::<SimulationQueueKind, usize>::new();
        for (kind, amount) in requests {
            let combined = amounts
                .get(&kind)
                .copied()
                .unwrap_or(0)
                .checked_add(amount)
                .ok_or(QueueBudgetError::ArithmeticOverflow)?;
            amounts.insert(kind, combined);
        }
        for (kind, amount) in &amounts {
            let capacity = self.capacity(*kind)?;
            let used = self.used.get(kind).copied().unwrap_or(0);
            let requested = used
                .checked_add(*amount)
                .ok_or(QueueBudgetError::ArithmeticOverflow)?;
            if requested > capacity {
                return Err(QueueBudgetError::Full {
                    kind: *kind,
                    used,
                    requested: *amount,
                    capacity,
                });
            }
        }
        for (kind, amount) in &amounts {
            *self.used.entry(*kind).or_default() += *amount;
        }
        Ok(QueueReservation { amounts })
    }

    pub fn release(&mut self, reservation: QueueReservation) -> Result<(), QueueBudgetError> {
        self.release_usage(reservation.amounts)
    }

    pub fn release_usage(
        &mut self,
        amounts: impl IntoIterator<Item = (SimulationQueueKind, usize)>,
    ) -> Result<(), QueueBudgetError> {
        let mut combined = BTreeMap::<SimulationQueueKind, usize>::new();
        for (kind, amount) in amounts {
            let amount = combined
                .get(&kind)
                .copied()
                .unwrap_or(0)
                .checked_add(amount)
                .ok_or(QueueBudgetError::ArithmeticOverflow)?;
            combined.insert(kind, amount);
        }
        for (kind, amount) in &combined {
            let used = self.used.get(kind).copied().unwrap_or(0);
            if *amount > used {
                return Err(QueueBudgetError::ReleaseExceedsUsage {
                    kind: *kind,
                    used,
                    released: *amount,
                });
            }
        }
        for (kind, amount) in combined {
            let used = self.used.get(&kind).copied().unwrap_or(0);
            if amount == used {
                self.used.remove(&kind);
            } else {
                self.used.insert(kind, used - amount);
            }
        }
        Ok(())
    }

    fn capacity(&self, kind: SimulationQueueKind) -> Result<usize, QueueBudgetError> {
        self.capacity
            .get(&kind)
            .copied()
            .ok_or(QueueBudgetError::UnconfiguredKind { kind })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum QueueBudgetError {
    #[error("Simulation queue {kind:?} cannot have zero capacity")]
    ZeroCapacity { kind: SimulationQueueKind },
    #[error("Simulation queue {kind:?} is configured more than once")]
    DuplicateKind { kind: SimulationQueueKind },
    #[error("Simulation queue {kind:?} has no configured capacity")]
    UnconfiguredKind { kind: SimulationQueueKind },
    #[error(
        "Simulation queue {kind:?} uses {used}/{capacity} entries and cannot reserve {requested} more"
    )]
    Full {
        kind: SimulationQueueKind,
        used: usize,
        requested: usize,
        capacity: usize,
    },
    #[error("Simulation queue {kind:?} uses {used} entries and cannot release {released} entries")]
    ReleaseExceedsUsage {
        kind: SimulationQueueKind,
        used: usize,
        released: usize,
    },
    #[error("Simulation queue usage arithmetic overflowed")]
    ArithmeticOverflow,
}
