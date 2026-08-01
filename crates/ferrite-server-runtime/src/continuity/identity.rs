use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::SnapshotRecordKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContinuityDomain {
    SimulationRuntime,
    ScheduledBlock,
    ScheduledFluid,
    SimulationBoundaryReceipt,
    Player,
    Entity,
    EntityTransferReceipt,
    WorldChunk,
    WorldLevel,
    WorldMetadata,
}

impl ContinuityDomain {
    pub const ALL: [Self; 10] = [
        Self::SimulationRuntime,
        Self::ScheduledBlock,
        Self::ScheduledFluid,
        Self::SimulationBoundaryReceipt,
        Self::Player,
        Self::Entity,
        Self::EntityTransferReceipt,
        Self::WorldChunk,
        Self::WorldLevel,
        Self::WorldMetadata,
    ];

    pub const LEGACY: [Self; 9] = [
        Self::SimulationRuntime,
        Self::ScheduledBlock,
        Self::ScheduledFluid,
        Self::SimulationBoundaryReceipt,
        Self::Player,
        Self::Entity,
        Self::EntityTransferReceipt,
        Self::WorldChunk,
        Self::WorldLevel,
    ];

    pub const fn record_kind(self) -> SnapshotRecordKind {
        match self {
            Self::SimulationRuntime | Self::WorldLevel | Self::WorldMetadata => {
                SnapshotRecordKind::Extension
            }
            Self::ScheduledBlock | Self::ScheduledFluid => SnapshotRecordKind::ScheduledWork,
            Self::SimulationBoundaryReceipt | Self::EntityTransferReceipt => {
                SnapshotRecordKind::AppliedBoundary
            }
            Self::Player | Self::Entity => SnapshotRecordKind::Entity,
            Self::WorldChunk => SnapshotRecordKind::Chunk,
        }
    }

    const fn legacy_path(self) -> Option<&'static str> {
        match self {
            Self::SimulationRuntime => Some("phase5/runtime_v1"),
            Self::ScheduledBlock => Some("phase5/scheduled_block_v1"),
            Self::ScheduledFluid => Some("phase5/scheduled_fluid_v1"),
            Self::SimulationBoundaryReceipt => Some("phase5/boundary_receipt_v1"),
            Self::Player => Some("phase6/player_v1"),
            Self::Entity => Some("phase7/entity_v1"),
            Self::EntityTransferReceipt => Some("phase7/applied_transfer_v1"),
            Self::WorldChunk => Some("phase8/chunk_v1"),
            Self::WorldLevel => Some("phase8/level_v1"),
            Self::WorldMetadata => None,
        }
    }

    const fn current_path(self) -> &'static str {
        match self {
            Self::SimulationRuntime => "simulation/runtime_v1",
            Self::ScheduledBlock => "simulation/scheduled_block_v1",
            Self::ScheduledFluid => "simulation/scheduled_fluid_v1",
            Self::SimulationBoundaryReceipt => "simulation/boundary_receipt_v1",
            Self::Player => "player-service/player_v1",
            Self::Entity => "entity-service/entity_v1",
            Self::EntityTransferReceipt => "entity-service/applied_transfer_v1",
            Self::WorldChunk => "world-service/chunk_v1",
            Self::WorldLevel => "world-service/level_v1",
            Self::WorldMetadata => "world-service/world_v1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuityGeneration {
    Legacy,
    Current,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassifiedContinuityDomain {
    pub domain: ContinuityDomain,
    pub generation: ContinuityGeneration,
}

#[must_use]
pub fn domain_id(domain: ContinuityDomain, generation: ContinuityGeneration) -> ResourceId {
    let path = match generation {
        ContinuityGeneration::Legacy => domain
            .legacy_path()
            .expect("requested continuity domain has a legacy identity"),
        ContinuityGeneration::Current => domain.current_path(),
    };
    ResourceId::new("ferrite", path).expect("static continuity domain is valid")
}

#[must_use]
pub fn classify_domain(id: &ResourceId) -> Option<ClassifiedContinuityDomain> {
    ContinuityDomain::ALL.into_iter().find_map(|domain| {
        if domain
            .legacy_path()
            .is_some_and(|path| id.path() == path && id.namespace() == "ferrite")
        {
            Some(ClassifiedContinuityDomain {
                domain,
                generation: ContinuityGeneration::Legacy,
            })
        } else if id == &domain_id(domain, ContinuityGeneration::Current) {
            Some(ClassifiedContinuityDomain {
                domain,
                generation: ContinuityGeneration::Current,
            })
        } else {
            None
        }
    })
}

pub(crate) fn is_reserved_continuity_id(id: &ResourceId) -> bool {
    id.namespace() == "ferrite"
        && [
            "phase5/",
            "phase6/",
            "phase7/",
            "phase8/",
            "simulation/",
            "player-service/",
            "entity-service/",
            "world-service/",
        ]
        .iter()
        .any(|prefix| id.path().starts_with(prefix))
}
