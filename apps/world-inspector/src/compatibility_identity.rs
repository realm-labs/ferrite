//! Read-only world-service continuity identities accepted by the offline inspector.
//!
//! Historical planning-phase strings are persisted bytes. They stay isolated here so the
//! inspector can diagnose and migrate old stores without presenting those names as current
//! production responsibilities.

use std::io;

use ferrite_persistence::snapshot::{SnapshotRecord, SnapshotRecordKind};

pub(super) const LEGACY_WORLD_CHUNK_DOMAIN: &str = "ferrite:phase8/chunk_v1";
pub(super) const LEGACY_WORLD_LEVEL_DOMAIN: &str = "ferrite:phase8/level_v1";
pub(super) const CURRENT_WORLD_CHUNK_DOMAIN: &str = "ferrite:world-service/chunk_v1";
pub(super) const CURRENT_WORLD_LEVEL_DOMAIN: &str = "ferrite:world-service/level_v1";
pub(super) const CURRENT_WORLD_METADATA_DOMAIN: &str = "ferrite:world-service/world_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContinuityGeneration {
    Legacy,
    Current,
}

impl ContinuityGeneration {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Current => "current",
        }
    }
}

pub(super) fn classify_world_record(
    record: &SnapshotRecord,
) -> Result<Option<ContinuityGeneration>, io::Error> {
    let identity = record.domain().to_string();
    let generation = match identity.as_str() {
        LEGACY_WORLD_CHUNK_DOMAIN if record.kind() == SnapshotRecordKind::Chunk => {
            Some(ContinuityGeneration::Legacy)
        }
        LEGACY_WORLD_LEVEL_DOMAIN if record.kind() == SnapshotRecordKind::Extension => {
            Some(ContinuityGeneration::Legacy)
        }
        CURRENT_WORLD_CHUNK_DOMAIN if record.kind() == SnapshotRecordKind::Chunk => {
            Some(ContinuityGeneration::Current)
        }
        CURRENT_WORLD_LEVEL_DOMAIN | CURRENT_WORLD_METADATA_DOMAIN
            if record.kind() == SnapshotRecordKind::Extension =>
        {
            Some(ContinuityGeneration::Current)
        }
        LEGACY_WORLD_CHUNK_DOMAIN
        | LEGACY_WORLD_LEVEL_DOMAIN
        | CURRENT_WORLD_CHUNK_DOMAIN
        | CURRENT_WORLD_LEVEL_DOMAIN
        | CURRENT_WORLD_METADATA_DOMAIN => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("wrong record kind for world-service continuity identity {identity}"),
            ));
        }
        value if is_reserved_world_identity(value) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported world-service continuity identity {value}"),
            ));
        }
        _ => None,
    };
    Ok(generation)
}

pub(super) fn is_world_chunk_record(record: &SnapshotRecord) -> bool {
    record.kind() == SnapshotRecordKind::Chunk
        && matches!(
            record.domain().to_string().as_str(),
            LEGACY_WORLD_CHUNK_DOMAIN | CURRENT_WORLD_CHUNK_DOMAIN
        )
}

fn is_reserved_world_identity(value: &str) -> bool {
    value.starts_with("ferrite:phase8/") || value.starts_with("ferrite:world-service/")
}
