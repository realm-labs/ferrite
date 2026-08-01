use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_persistence::snapshot::{PersistenceRevision, RegionRecoveryPoint, SnapshotRecord};
use ferrite_world::chunk::{ChunkColumn, ChunkLayout};
use ferrite_world::generation::status::ChunkStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ChunkActivity {
    Dormant = 0,
    Accessible = 1,
    BlockTicking = 2,
    EntityTicking = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingGeneration {
    pub request_id: u64,
    pub expected_revision: u64,
    pub target_status: ChunkStatus,
    pub content_manifest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingUnload {
    pub token: u64,
    pub expected_revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkLifecycle {
    pub status: ChunkStatus,
    pub activity: ChunkActivity,
    pub pending_generation: Option<PendingGeneration>,
    pub pending_unload: Option<PendingUnload>,
}

impl ChunkLifecycle {
    pub const fn empty() -> Self {
        Self {
            status: ChunkStatus::Empty,
            activity: ChunkActivity::Dormant,
            pending_generation: None,
            pending_unload: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationRequest {
    pub region: SimulationRegionKey,
    pub generation: ActivationGeneration,
    pub chunk: ChunkPos,
    pub request_id: u64,
    pub expected_revision: u64,
    pub target_status: ChunkStatus,
    pub content_manifest: [u8; 32],
    pub source: ChunkColumn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationResult {
    pub region: SimulationRegionKey,
    pub generation: ActivationGeneration,
    pub chunk: ChunkPos,
    pub request_id: u64,
    pub expected_revision: u64,
    pub target_status: ChunkStatus,
    pub content_manifest: [u8; 32],
    pub generated: ChunkColumn,
}

impl GenerationRequest {
    #[must_use]
    pub fn complete(self, generated: ChunkColumn) -> GenerationResult {
        GenerationResult {
            region: self.region,
            generation: self.generation,
            chunk: self.chunk,
            request_id: self.request_id,
            expected_revision: self.expected_revision,
            target_status: self.target_status,
            content_manifest: self.content_manifest,
            generated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationOutcome {
    Published { revision: u64 },
    StaleRevision { expected: u64, actual: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketOutcome {
    Loaded,
    AlreadyLoaded,
    CancelledUnload { token: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkEventKind {
    GenerationPublished { status: ChunkStatus, revision: u64 },
    Accessible,
    PersistedTicksUnpacked,
    BlockTicking,
    EntityTicking,
    Demoted { activity: ChunkActivity },
    UnloadCancelled { token: u64 },
    Saved { token: u64 },
    Unloaded { token: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkEvent {
    pub sequence: u64,
    pub chunk: ChunkPos,
    pub kind: ChunkEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase8RuntimeConfig {
    pub mapping: RegionMapping,
    pub layout: ChunkLayout,
    pub region_side_chunks: u16,
    pub chunk_capacity: usize,
    pub event_capacity: usize,
    pub content_manifest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingUnloadIdentity {
    pub chunk: ChunkPos,
    pub token: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedWorldSave {
    point: RegionRecoveryPoint,
    pending_unloads: Box<[PendingUnloadIdentity]>,
}

impl PreparedWorldSave {
    pub(crate) fn new(
        point: RegionRecoveryPoint,
        pending_unloads: Vec<PendingUnloadIdentity>,
    ) -> Self {
        Self {
            point,
            pending_unloads: pending_unloads.into_boxed_slice(),
        }
    }

    pub const fn recovery_point(&self) -> &RegionRecoveryPoint {
        &self.point
    }

    pub const fn persistence_revision(&self) -> PersistenceRevision {
        self.point.persistence_revision()
    }

    pub fn committed_tick(&self) -> u64 {
        self.point.committed_tick()
    }

    pub fn digest(&self) -> Result<[u8; 32], ferrite_persistence::snapshot::SnapshotError> {
        self.point.digest()
    }

    pub fn records(&self) -> &[SnapshotRecord] {
        self.point.snapshot().records()
    }

    pub(crate) fn pending_unloads(&self) -> &[PendingUnloadIdentity] {
        &self.pending_unloads
    }
}
