use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

pub const MAX_ENTITY_PAYLOAD_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPayload {
    bytes: Box<[u8]>,
    digest: [u8; 32],
}

impl EntityPayload {
    pub fn new(bytes: Vec<u8>) -> Result<Self, EntityPayloadError> {
        if bytes.len() > MAX_ENTITY_PAYLOAD_BYTES {
            return Err(EntityPayloadError::TooLarge {
                actual: bytes.len(),
                maximum: MAX_ENTITY_PAYLOAD_BYTES,
            });
        }
        Ok(Self {
            digest: *blake3::hash(&bytes).as_bytes(),
            bytes: bytes.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

impl Default for EntityPayload {
    fn default() -> Self {
        Self::new(Vec::new()).expect("empty entity payload is bounded")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EntityPayloadError {
    #[error("entity payload has {actual} bytes, exceeding {maximum}")]
    TooLarge { actual: usize, maximum: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundEntityTransfer {
    pub tick: GameTick,
    pub target: SimulationRegionKey,
    pub target_generation: ActivationGeneration,
    pub source_sequence: u64,
    pub candidate_chunk: ChunkPos,
    pub candidate_revision: u64,
    pub candidate_payload: EntityPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityLifecycleState {
    Active,
    Inactive,
    OutboundPending(OutboundEntityTransfer),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPersistentState {
    pub kind: ResourceId,
    pub chunk: ChunkPos,
    pub revision: u64,
    pub last_command_sequence: u64,
    pub payload: EntityPayload,
    pub lifecycle: EntityLifecycleState,
}

impl EntityPersistentState {
    #[must_use]
    pub fn active(kind: ResourceId, chunk: ChunkPos, payload: EntityPayload) -> Self {
        Self {
            kind,
            chunk,
            revision: 0,
            last_command_sequence: 0,
            payload,
            lifecycle: EntityLifecycleState::Active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityCommandHeader {
    pub region: SimulationRegionKey,
    pub generation: ActivationGeneration,
    pub entity: StableEntityId,
    pub expected_revision: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityMutation {
    pub chunk: ChunkPos,
    pub payload: EntityPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTransferRequest {
    pub tick: GameTick,
    pub source: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub target: SimulationRegionKey,
    pub target_generation: ActivationGeneration,
    pub entity: StableEntityId,
    pub expected_revision: u64,
    pub sequence: u64,
    pub candidate: EntityMutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalReason {
    Deactivated,
    Despawned,
    Transferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityProjectionKind {
    Spawn {
        kind: ResourceId,
        chunk: ChunkPos,
        revision: u64,
        state_digest: [u8; 32],
    },
    Update {
        chunk: ChunkPos,
        revision: u64,
        state_digest: [u8; 32],
    },
    Remove {
        revision: u64,
        reason: RemovalReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityProjection {
    pub sequence: u64,
    pub observer: StableEntityId,
    pub entity: StableEntityId,
    pub kind: EntityProjectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleOutcome {
    Committed { revision: u64 },
    AlreadyApplied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObserverOutcome {
    Added,
    AlreadyPresent,
}
