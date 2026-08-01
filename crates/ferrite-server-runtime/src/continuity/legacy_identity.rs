//! Read-only identities emitted by the historical Goal 01 implementation.
//!
//! These byte strings are persisted compatibility inputs. Production writers must use the
//! responsibility-owned identities in `continuity::identity`; this module is used only while
//! classifying and atomically migrating an existing recovery point.

use crate::continuity::identity::ContinuityDomain;

pub(super) const RESERVED_PREFIXES: [&str; 8] = [
    "phase5/",
    "phase6/",
    "phase7/",
    "phase8/",
    "simulation/",
    "player-service/",
    "entity-service/",
    "world-service/",
];

pub(super) const fn path(domain: ContinuityDomain) -> Option<&'static str> {
    match domain {
        ContinuityDomain::SimulationRuntime => Some("phase5/runtime_v1"),
        ContinuityDomain::ScheduledBlock => Some("phase5/scheduled_block_v1"),
        ContinuityDomain::ScheduledFluid => Some("phase5/scheduled_fluid_v1"),
        ContinuityDomain::SimulationBoundaryReceipt => Some("phase5/boundary_receipt_v1"),
        ContinuityDomain::Player => Some("phase6/player_v1"),
        ContinuityDomain::Entity => Some("phase7/entity_v1"),
        ContinuityDomain::EntityTransferReceipt => Some("phase7/applied_transfer_v1"),
        ContinuityDomain::WorldChunk => Some("phase8/chunk_v1"),
        ContinuityDomain::WorldLevel => Some("phase8/level_v1"),
        ContinuityDomain::WorldMetadata => None,
    }
}
