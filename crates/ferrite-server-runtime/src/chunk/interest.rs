use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_world::chunk::ChunkRevision;
use thiserror::Error;

pub const MINIMUM_VIEW_DISTANCE: u16 = 2;
pub const MAXIMUM_VIEW_DISTANCE: u16 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownChunkState {
    Pending,
    Sent { revision: ChunkRevision },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterestDelta {
    pub center_changed: bool,
    pub entered: Vec<ChunkPos>,
    pub forgotten: Vec<ChunkPos>,
}

#[derive(Debug, Clone)]
pub struct ClientInterest {
    center: ChunkPos,
    view_distance: u16,
    simulation_distance: u16,
    maximum_tracked_chunks: usize,
    view: BTreeSet<ChunkPos>,
    known: BTreeMap<ChunkPos, KnownChunkState>,
}

impl ClientInterest {
    pub fn new(
        center: ChunkPos,
        requested_view_distance: i8,
        server_view_distance: u16,
        simulation_distance: u16,
        maximum_tracked_chunks: usize,
    ) -> Result<Self, InterestError> {
        validate_server_distance(server_view_distance)?;
        if maximum_tracked_chunks == 0 {
            return Err(InterestError::ZeroCapacity);
        }
        let view_distance = effective_view_distance(requested_view_distance, server_view_distance);
        let view = positions(center, view_distance)?;
        if view.len() > maximum_tracked_chunks {
            return Err(InterestError::Capacity {
                required: view.len(),
                maximum: maximum_tracked_chunks,
            });
        }
        Ok(Self {
            center,
            view_distance,
            simulation_distance,
            maximum_tracked_chunks,
            view,
            known: BTreeMap::new(),
        })
    }

    pub fn recenter(&mut self, center: ChunkPos) -> Result<InterestDelta, InterestError> {
        let next = positions(center, self.view_distance)?;
        if next.len() > self.maximum_tracked_chunks {
            return Err(InterestError::Capacity {
                required: next.len(),
                maximum: self.maximum_tracked_chunks,
            });
        }
        let entered = next.difference(&self.view).copied().collect();
        let leaving = self.view.difference(&next).copied().collect::<Vec<_>>();
        let mut forgotten = Vec::new();
        for position in leaving {
            if matches!(
                self.known.remove(&position),
                Some(KnownChunkState::Sent { .. })
            ) {
                forgotten.push(position);
            }
        }
        let center_changed = center != self.center;
        self.center = center;
        self.view = next;
        Ok(InterestDelta {
            center_changed,
            entered,
            forgotten,
        })
    }

    pub fn mark_ready(&mut self, position: ChunkPos) -> Result<bool, InterestError> {
        if !self.view.contains(&position) || self.known.contains_key(&position) {
            return Ok(false);
        }
        if self.known.len() == self.maximum_tracked_chunks {
            return Err(InterestError::Capacity {
                required: self.known.len().saturating_add(1),
                maximum: self.maximum_tracked_chunks,
            });
        }
        self.known.insert(position, KnownChunkState::Pending);
        Ok(true)
    }

    pub(crate) fn requeue(&mut self, position: ChunkPos) -> Result<bool, InterestError> {
        if !self.view.contains(&position) {
            return Ok(false);
        }
        if !self.known.contains_key(&position) && self.known.len() == self.maximum_tracked_chunks {
            return Err(InterestError::Capacity {
                required: self.known.len().saturating_add(1),
                maximum: self.maximum_tracked_chunks,
            });
        }
        let changed = !matches!(self.known.get(&position), Some(KnownChunkState::Pending));
        self.known.insert(position, KnownChunkState::Pending);
        Ok(changed)
    }

    pub fn mark_sent(
        &mut self,
        position: ChunkPos,
        revision: ChunkRevision,
    ) -> Result<(), InterestError> {
        match self.known.get_mut(&position) {
            Some(state @ KnownChunkState::Pending) => {
                *state = KnownChunkState::Sent { revision };
                Ok(())
            }
            _ => Err(InterestError::NotPending(position)),
        }
    }

    #[must_use]
    pub fn pending_by_distance(&self) -> Vec<ChunkPos> {
        let mut pending = self
            .known
            .iter()
            .filter_map(|(position, state)| {
                matches!(state, KnownChunkState::Pending).then_some(*position)
            })
            .collect::<Vec<_>>();
        pending.sort_by_key(|position| {
            let dx = i64::from(position.x) - i64::from(self.center.x);
            let dz = i64::from(position.z) - i64::from(self.center.z);
            (dx * dx + dz * dz, position.x, position.z)
        });
        pending
    }

    #[must_use]
    pub const fn center(&self) -> ChunkPos {
        self.center
    }

    #[must_use]
    pub const fn view_distance(&self) -> u16 {
        self.view_distance
    }

    #[must_use]
    pub const fn simulation_distance(&self) -> u16 {
        self.simulation_distance
    }

    #[must_use]
    pub fn view(&self) -> &BTreeSet<ChunkPos> {
        &self.view
    }

    #[must_use]
    pub fn known(&self) -> &BTreeMap<ChunkPos, KnownChunkState> {
        &self.known
    }
}

fn effective_view_distance(requested: i8, server: u16) -> u16 {
    let requested = u16::try_from(requested).unwrap_or(MINIMUM_VIEW_DISTANCE);
    requested.clamp(MINIMUM_VIEW_DISTANCE, server)
}

fn validate_server_distance(distance: u16) -> Result<(), InterestError> {
    if (MINIMUM_VIEW_DISTANCE..=MAXIMUM_VIEW_DISTANCE).contains(&distance) {
        Ok(())
    } else {
        Err(InterestError::ServerViewDistance { distance })
    }
}

fn positions(center: ChunkPos, distance: u16) -> Result<BTreeSet<ChunkPos>, InterestError> {
    let distance = i32::from(distance);
    let minimum_x = center
        .x
        .checked_sub(distance)
        .ok_or(InterestError::CoordinateOverflow)?;
    let maximum_x = center
        .x
        .checked_add(distance)
        .ok_or(InterestError::CoordinateOverflow)?;
    let minimum_z = center
        .z
        .checked_sub(distance)
        .ok_or(InterestError::CoordinateOverflow)?;
    let maximum_z = center
        .z
        .checked_add(distance)
        .ok_or(InterestError::CoordinateOverflow)?;
    Ok((minimum_x..=maximum_x)
        .flat_map(|x| (minimum_z..=maximum_z).map(move |z| ChunkPos::new(x, z)))
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InterestError {
    #[error("server view distance {distance} is outside 2..=32")]
    ServerViewDistance { distance: u16 },
    #[error("client interest capacity cannot be zero")]
    ZeroCapacity,
    #[error("client interest requires {required} chunks, exceeding {maximum}")]
    Capacity { required: usize, maximum: usize },
    #[error("chunk view coordinates overflow i32")]
    CoordinateOverflow,
    #[error("chunk {0:?} was not pending")]
    NotPending(ChunkPos),
}
