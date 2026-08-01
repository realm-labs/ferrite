//! Bounded per-session routing for committed composite projections.

use std::collections::VecDeque;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::{StableEntityId, StableIdError};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_protocol::java_26_2::play::clientbound::packet::{BlockUpdate, PlayClientboundPacket};
use ferrite_world::id::BlockStateId;
use thiserror::Error;

use crate::chunk::projection::{JavaTerrainRegistryMap, TerrainProjectionError};
use crate::composite::model::{CompositeOwner, CompositeProjection};
use crate::player::block::replication::AuthoritativeBlockUpdate;

const PLAYER_PROJECTION_PATH: &str = "composite/player/projection_v1";
const ENTITY_PROJECTION_PATH: &str = "composite/entity/projection_v1";
const BLOCK_PROJECTION_PATH: &str = "composite/simulation/block_update_v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionAudience {
    AllPlayers,
    RegionPlayers(SimulationRegionKey),
    Player(StableEntityId),
}

impl ProjectionAudience {
    pub fn includes(&self, player: StableEntityId, region: &SimulationRegionKey) -> bool {
        match self {
            Self::AllPlayers => true,
            Self::RegionPlayers(target) => target == region,
            Self::Player(target) => *target == player,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferredProjectionKind {
    PlayerService,
    EntityService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeferredSessionProjection {
    pub sequence: u64,
    pub kind: DeferredProjectionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionProjectionAction {
    Block(AuthoritativeBlockUpdate),
    Deferred(DeferredProjectionKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionProjection {
    sequence: u64,
    audience: ProjectionAudience,
    action: SessionProjectionAction,
}

impl SessionProjection {
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn audience(&self) -> ProjectionAudience {
        self.audience.clone()
    }

    pub const fn action(&self) -> &SessionProjectionAction {
        &self.action
    }

    #[must_use]
    pub fn scoped_to_region(mut self, region: SimulationRegionKey) -> Self {
        if self.audience == ProjectionAudience::AllPlayers {
            self.audience = ProjectionAudience::RegionPlayers(region);
        }
        self
    }
}

pub fn decode_projection(
    projection: &CompositeProjection,
) -> Result<SessionProjection, SessionProjectionError> {
    if projection.sequence() == 0 {
        return Err(SessionProjectionError::ZeroSequence);
    }
    match (
        projection.owner(),
        projection.kind().namespace(),
        projection.kind().path(),
    ) {
        (CompositeOwner::PlayerService, "ferrite", PLAYER_PROJECTION_PATH) => {
            decode_player_projection(projection)
        }
        (CompositeOwner::EntityService, "ferrite", ENTITY_PROJECTION_PATH) => {
            decode_entity_projection(projection)
        }
        (CompositeOwner::Simulation, "ferrite", BLOCK_PROJECTION_PATH) => {
            decode_block_projection(projection)
        }
        _ => Err(SessionProjectionError::UnsupportedKind(
            projection.kind().to_string(),
        )),
    }
}

fn decode_player_projection(
    projection: &CompositeProjection,
) -> Result<SessionProjection, SessionProjectionError> {
    let payload = projection.payload();
    let player = read_player(payload)?;
    let tail = payload
        .get(24..)
        .ok_or(SessionProjectionError::MalformedPlayer)?;
    match tail.first().copied() {
        Some(0) if tail.len() == 9 => {}
        Some(1) if tail.len() == 12 => {}
        Some(2) => validate_full_player_projection(tail)?,
        _ => return Err(SessionProjectionError::MalformedPlayer),
    }
    Ok(SessionProjection {
        sequence: projection.sequence(),
        audience: ProjectionAudience::Player(player),
        action: SessionProjectionAction::Deferred(DeferredProjectionKind::PlayerService),
    })
}

fn validate_full_player_projection(tail: &[u8]) -> Result<(), SessionProjectionError> {
    if tail.len() < 11 || tail[1] > 3 {
        return Err(SessionProjectionError::MalformedPlayer);
    }
    match tail[10] {
        0 if tail.len() == 11 => Ok(()),
        1 if tail.len() == 14 => Ok(()),
        _ => Err(SessionProjectionError::MalformedPlayer),
    }
}

fn decode_entity_projection(
    projection: &CompositeProjection,
) -> Result<SessionProjection, SessionProjectionError> {
    let payload = projection.payload();
    let observer = read_player(payload)?;
    if payload.len() < 33 || payload[32] > 2 {
        return Err(SessionProjectionError::MalformedEntity);
    }
    Ok(SessionProjection {
        sequence: projection.sequence(),
        audience: ProjectionAudience::Player(observer),
        action: SessionProjectionAction::Deferred(DeferredProjectionKind::EntityService),
    })
}

fn decode_block_projection(
    projection: &CompositeProjection,
) -> Result<SessionProjection, SessionProjectionError> {
    let payload: &[u8; 16] = projection
        .payload()
        .try_into()
        .map_err(|_| SessionProjectionError::MalformedBlock)?;
    let position = BlockPos::new(
        i32::from_be_bytes(payload[0..4].try_into().expect("fixed block x width")),
        i32::from_be_bytes(payload[4..8].try_into().expect("fixed block y width")),
        i32::from_be_bytes(payload[8..12].try_into().expect("fixed block z width")),
    );
    let state = BlockStateId::new(u32::from_be_bytes(
        payload[12..16].try_into().expect("fixed block-state width"),
    ));
    Ok(SessionProjection {
        sequence: projection.sequence(),
        audience: ProjectionAudience::AllPlayers,
        action: SessionProjectionAction::Block(AuthoritativeBlockUpdate { position, state }),
    })
}

fn read_player(payload: &[u8]) -> Result<StableEntityId, SessionProjectionError> {
    let bytes: [u8; 16] = payload
        .get(..16)
        .ok_or(SessionProjectionError::MissingAudience)?
        .try_into()
        .expect("validated stable identity width");
    Ok(StableEntityId::new(u128::from_be_bytes(bytes))?)
}

#[derive(Debug, Clone)]
pub struct SessionProjectionQueue {
    capacity: usize,
    queue: VecDeque<SessionProjection>,
}

impl SessionProjectionQueue {
    pub fn new(capacity: usize) -> Result<Self, SessionProjectionError> {
        if capacity == 0 {
            return Err(SessionProjectionError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            queue: VecDeque::new(),
        })
    }

    pub fn admit(
        &mut self,
        player: StableEntityId,
        region: &SimulationRegionKey,
        projections: &[SessionProjection],
    ) -> Result<usize, SessionProjectionError> {
        let additional = projections
            .iter()
            .filter(|projection| projection.audience.includes(player, region))
            .count();
        let used =
            self.queue
                .len()
                .checked_add(additional)
                .ok_or(SessionProjectionError::Full {
                    capacity: self.capacity,
                })?;
        if used > self.capacity {
            return Err(SessionProjectionError::Full {
                capacity: self.capacity,
            });
        }
        self.queue.extend(
            projections
                .iter()
                .filter(|projection| projection.audience.includes(player, region))
                .cloned(),
        );
        Ok(additional)
    }

    pub fn project(
        &mut self,
        maximum: usize,
        registries: &JavaTerrainRegistryMap,
    ) -> Result<SessionProjectionBatch, SessionProjectionError> {
        let count = maximum.min(self.queue.len());
        let mut packets = Vec::new();
        let mut deferred = Vec::new();
        for projection in self.queue.iter().take(count) {
            match projection.action {
                SessionProjectionAction::Block(update) => {
                    packets.push(PlayClientboundPacket::BlockUpdate(BlockUpdate {
                        position: update.position,
                        state: registries.block_state(update.state)?,
                    }));
                }
                SessionProjectionAction::Deferred(kind) => {
                    deferred.push(DeferredSessionProjection {
                        sequence: projection.sequence,
                        kind,
                    });
                }
            }
        }
        self.queue.drain(..count);
        Ok(SessionProjectionBatch { packets, deferred })
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionProjectionBatch {
    pub packets: Vec<PlayClientboundPacket>,
    pub deferred: Vec<DeferredSessionProjection>,
}

#[derive(Debug, Error)]
pub enum SessionProjectionError {
    #[error("session projection capacity cannot be zero")]
    ZeroCapacity,
    #[error("session projection queue reached its {capacity}-record bound")]
    Full { capacity: usize },
    #[error("composite projection sequence cannot be zero")]
    ZeroSequence,
    #[error("composite projection has no stable player audience")]
    MissingAudience,
    #[error("composite player projection is malformed")]
    MalformedPlayer,
    #[error("composite entity projection is malformed")]
    MalformedEntity,
    #[error("composite block projection is malformed")]
    MalformedBlock,
    #[error("composite projection kind {0} is unsupported")]
    UnsupportedKind(String),
    #[error(transparent)]
    StableIdentity(#[from] StableIdError),
    #[error(transparent)]
    Terrain(#[from] TerrainProjectionError),
}
