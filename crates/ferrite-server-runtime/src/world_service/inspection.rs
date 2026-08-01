use ferrite_persistence::snapshot::RegionRecoveryPoint;
use serde::Serialize;

use crate::continuity::identity::{ContinuityGeneration, classify_domain};
use crate::continuity::migration::normalize_recovery_point;
use crate::world_service::continuity::{
    WorldServiceContinuityError, canonical_state_hash, decode_chunk_record, materialized_records,
};
use crate::world_service::model::ChunkActivity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldInspection {
    pub world: String,
    pub dimension: String,
    pub region_x: i32,
    pub region_z: i32,
    pub activation_generation: u64,
    pub committed_tick: u64,
    pub persistence_revision: u64,
    pub content_manifest: String,
    pub continuity_generation: &'static str,
    pub snapshot_state_hash_matches: bool,
    pub auxiliary_records: usize,
    pub chunks: Vec<InspectedChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InspectedChunk {
    pub x: i32,
    pub z: i32,
    pub status: &'static str,
    pub activity: &'static str,
    pub revision: u64,
    pub structure_starts: usize,
    pub structure_references: usize,
    pub pending_generation: Option<InspectedGenerationContinuation>,
    pub pending_unload_token: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InspectedGenerationContinuation {
    pub version: u16,
    pub request_id: u64,
    pub expected_revision: u64,
    pub target_status: &'static str,
    pub content_manifest: String,
}

pub fn inspect_recovery_point(
    point: &RegionRecoveryPoint,
) -> Result<WorldInspection, WorldServiceContinuityError> {
    let continuity_generation = point
        .snapshot()
        .records()
        .iter()
        .chain(
            point
                .journal_tail()
                .iter()
                .flat_map(|frame| frame.records()),
        )
        .find_map(|record| classify_domain(record.domain()))
        .map_or("none", |classified| match classified.generation {
            ContinuityGeneration::Legacy => "legacy",
            ContinuityGeneration::Current => "current",
        });
    let normalized = normalize_recovery_point(point)?;
    let header = point.snapshot().header();
    let mut chunks = Vec::new();
    let mut auxiliary_records = 0;
    for record in materialized_records(&normalized) {
        if let Some((chunk, lifecycle)) = decode_chunk_record(&record)? {
            chunks.push(InspectedChunk {
                x: chunk.position().x,
                z: chunk.position().z,
                status: status_name(lifecycle.status as u8),
                activity: activity_name(lifecycle.activity),
                revision: chunk.revision().get(),
                structure_starts: chunk.structures().starts().len(),
                structure_references: chunk.structures().references().len(),
                pending_generation: lifecycle.pending_generation.map(|pending| {
                    InspectedGenerationContinuation {
                        version: pending.continuation_version,
                        request_id: pending.request_id,
                        expected_revision: pending.expected_revision,
                        target_status: status_name(pending.target_status as u8),
                        content_manifest: encode_hex(&pending.content_manifest),
                    }
                }),
                pending_unload_token: lifecycle.pending_unload.map(|pending| pending.token),
            });
        } else {
            auxiliary_records += 1;
        }
    }
    chunks.sort_by_key(|chunk| (chunk.x, chunk.z));
    Ok(WorldInspection {
        world: header.key.world().to_string(),
        dimension: header.key.dimension().to_string(),
        region_x: header.key.coordinate().x(),
        region_z: header.key.coordinate().z(),
        activation_generation: header.generation.get(),
        committed_tick: point.committed_tick(),
        persistence_revision: header.persistence_revision.get(),
        content_manifest: encode_hex(&header.content_manifest),
        continuity_generation,
        snapshot_state_hash_matches: canonical_state_hash(point.snapshot().records())
            == header.state_hash,
        auxiliary_records,
        chunks,
    })
}

fn status_name(tag: u8) -> &'static str {
    const NAMES: [&str; 12] = [
        "empty",
        "structure_starts",
        "structure_references",
        "biomes",
        "noise",
        "surface",
        "carvers",
        "features",
        "initialize_light",
        "light",
        "spawn",
        "full",
    ];
    NAMES[usize::from(tag)]
}

const fn activity_name(activity: ChunkActivity) -> &'static str {
    match activity {
        ChunkActivity::Dormant => "dormant",
        ChunkActivity::Accessible => "accessible",
        ChunkActivity::BlockTicking => "block_ticking",
        ChunkActivity::EntityTicking => "entity_ticking",
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
