//! Bounded committed block-state aggregation for client projection.

use crate::chunk::projection::JavaTerrainRegistryMap;
use crate::player::block::replication::{
    AuthoritativeBlockUpdate, BlockReplicationError, project_authoritative_updates,
};
use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_world::id::BlockStateId;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionAdmission {
    pub new_positions: usize,
    pub replaced_positions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationProjectionBuffer {
    capacity: usize,
    updates: BTreeMap<BlockPos, BlockStateId>,
}

impl SimulationProjectionBuffer {
    pub const fn new(capacity: usize) -> Result<Self, SimulationProjectionError> {
        if capacity == 0 {
            return Err(SimulationProjectionError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            updates: BTreeMap::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.updates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }

    pub fn additional_positions(
        &self,
        updates: &[AuthoritativeBlockUpdate],
    ) -> Result<usize, SimulationProjectionError> {
        let mut incoming = BTreeSet::new();
        for update in updates {
            incoming.insert(update.position);
        }
        let additional = incoming
            .iter()
            .filter(|position| !self.updates.contains_key(position))
            .count();
        if self.updates.len() + additional > self.capacity {
            return Err(SimulationProjectionError::Full {
                used: self.updates.len(),
                additional,
                capacity: self.capacity,
            });
        }
        Ok(additional)
    }

    pub fn enqueue(
        &mut self,
        updates: &[AuthoritativeBlockUpdate],
    ) -> Result<ProjectionAdmission, SimulationProjectionError> {
        let new_positions = self.additional_positions(updates)?;
        let mut replaced_positions = 0;
        for update in updates {
            if self.updates.insert(update.position, update.state).is_some() {
                replaced_positions += 1;
            }
        }
        Ok(ProjectionAdmission {
            new_positions,
            replaced_positions,
        })
    }

    pub fn project_and_clear(
        &mut self,
        registries: &JavaTerrainRegistryMap,
    ) -> Result<Vec<PlayClientboundPacket>, SimulationProjectionError> {
        let updates = self.updates();
        let packets = project_authoritative_updates(updates, registries)?;
        self.updates.clear();
        Ok(packets)
    }

    pub fn take_updates(&mut self) -> Vec<AuthoritativeBlockUpdate> {
        let updates = self.updates();
        self.updates.clear();
        updates
    }

    fn updates(&self) -> Vec<AuthoritativeBlockUpdate> {
        self.updates
            .iter()
            .map(|(position, state)| AuthoritativeBlockUpdate {
                position: *position,
                state: *state,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SimulationProjectionError {
    #[error("Simulation projection capacity cannot be zero")]
    ZeroCapacity,
    #[error(
        "Simulation projection uses {used}/{capacity} positions and cannot admit {additional} more"
    )]
    Full {
        used: usize,
        additional: usize,
        capacity: usize,
    },
    #[error(transparent)]
    Block(#[from] BlockReplicationError),
}
