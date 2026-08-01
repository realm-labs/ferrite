#![forbid(unsafe_code)]

//! Offline Region snapshot, journal, and world inspection entry point.

use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::io;
use std::path::PathBuf;

use ferrite_persistence::snapshot::{RegionRecoveryPoint, SnapshotRecord, SnapshotRecordKind};
use ferrite_persistence::store::RegionFileStore;
use ferrite_world::durable::decode_chunk;
use serde_json::{Value, json};

const LEGACY_WORLD_CHUNK_DOMAIN: &str = "ferrite:phase8/chunk_v1";
const LEGACY_WORLD_LEVEL_DOMAIN: &str = "ferrite:phase8/level_v1";
const CURRENT_WORLD_CHUNK_DOMAIN: &str = "ferrite:world-service/chunk_v1";
const CURRENT_WORLD_LEVEL_DOMAIN: &str = "ferrite:world-service/level_v1";

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os();
    let executable = arguments
        .next()
        .and_then(|value| PathBuf::from(value).file_name().map(OsStr::to_owned))
        .unwrap_or_else(|| "world-inspector".into());
    let store_path = PathBuf::from(required(&mut arguments, &executable, "store directory")?);
    let world_text = required(&mut arguments, &executable, "world id")?
        .to_string_lossy()
        .into_owned();
    let world = u128::from_str_radix(&world_text, 16)?;
    let dimension = required(&mut arguments, &executable, "dimension")?
        .to_string_lossy()
        .into_owned();
    let region_x = parse_i32(required(&mut arguments, &executable, "region x")?)?;
    let region_z = parse_i32(required(&mut arguments, &executable, "region z")?)?;
    if arguments.next().is_some() {
        return Err(usage_error(&executable, "unexpected extra argument").into());
    }
    if !store_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("store directory does not exist: {}", store_path.display()),
        )
        .into());
    }
    let store = RegionFileStore::open(&store_path)?;
    let point = store
        .load_named(world, &dimension, region_x, region_z, 1)?
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "no committed recovery point for {world_text}/{dimension}/{region_x}/{region_z}"
                ),
            )
        })?;
    println!("{}", serde_json::to_string_pretty(&inspect(&point)?)?);
    Ok(())
}

fn inspect(point: &RegionRecoveryPoint) -> Result<Value, Box<dyn Error>> {
    let header = point.snapshot().header();
    let continuity_generation = continuity_generation(point)?;
    let mut chunks = Vec::new();
    let mut auxiliary_records = 0usize;
    for record in materialized_records(point) {
        if !is_world_chunk_domain(&record) {
            auxiliary_records += 1;
            continue;
        }
        chunks.push(inspect_chunk(&record)?);
    }
    chunks.sort_by_key(|chunk| {
        (
            chunk["x"].as_i64().unwrap_or_default(),
            chunk["z"].as_i64().unwrap_or_default(),
        )
    });
    Ok(json!({
        "world": header.key.world().to_string(),
        "dimension": header.key.dimension().to_string(),
        "region_x": header.key.coordinate().x(),
        "region_z": header.key.coordinate().z(),
        "activation_generation": header.generation.get(),
        "committed_tick": point.committed_tick(),
        "persistence_revision": header.persistence_revision.get(),
        "content_manifest": encode_hex(&header.content_manifest),
        "continuity_generation": continuity_generation,
        "snapshot_state_hash_matches": canonical_state_hash(point.snapshot().records())
            == header.state_hash,
        "auxiliary_records": auxiliary_records,
        "chunks": chunks,
    }))
}

fn continuity_generation(point: &RegionRecoveryPoint) -> Result<&'static str, io::Error> {
    let mut generation = None;
    for record in point.snapshot().records().iter().chain(
        point
            .journal_tail()
            .iter()
            .flat_map(|frame| frame.records()),
    ) {
        let identity = record.domain().to_string();
        let candidate = match identity.as_str() {
            LEGACY_WORLD_CHUNK_DOMAIN if record.kind() == SnapshotRecordKind::Chunk => {
                Some("legacy")
            }
            LEGACY_WORLD_LEVEL_DOMAIN if record.kind() == SnapshotRecordKind::Extension => {
                Some("legacy")
            }
            CURRENT_WORLD_CHUNK_DOMAIN if record.kind() == SnapshotRecordKind::Chunk => {
                Some("current")
            }
            CURRENT_WORLD_LEVEL_DOMAIN if record.kind() == SnapshotRecordKind::Extension => {
                Some("current")
            }
            LEGACY_WORLD_CHUNK_DOMAIN
            | LEGACY_WORLD_LEVEL_DOMAIN
            | CURRENT_WORLD_CHUNK_DOMAIN
            | CURRENT_WORLD_LEVEL_DOMAIN => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("wrong record kind for world-service continuity identity {identity}"),
                ));
            }
            value
                if value.starts_with("ferrite:phase8/")
                    || value.starts_with("ferrite:world-service/") =>
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unsupported world-service continuity identity {value}"),
                ));
            }
            _ => None,
        };
        if let Some(candidate) = candidate {
            match generation {
                Some(existing) if existing != candidate => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "mixed legacy and current world-service continuity identities",
                    ));
                }
                None => generation = Some(candidate),
                _ => {}
            }
        }
    }
    Ok(generation.unwrap_or("none"))
}

fn is_world_chunk_domain(record: &SnapshotRecord) -> bool {
    record.kind() == SnapshotRecordKind::Chunk
        && matches!(
            record.domain().to_string().as_str(),
            LEGACY_WORLD_CHUNK_DOMAIN | CURRENT_WORLD_CHUNK_DOMAIN
        )
}

fn inspect_chunk(record: &SnapshotRecord) -> Result<Value, Box<dyn Error>> {
    let value = record.value();
    if value.len() < 7 || &value[..4] != b"P8C1" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid world-service chunk record",
        )
        .into());
    }
    let status = *STATUS_NAMES.get(usize::from(value[4])).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid world-service chunk status",
        )
    })?;
    let activity = *ACTIVITY_NAMES.get(usize::from(value[5])).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid world-service chunk activity",
        )
    })?;
    let (offset, pending_unload_token) = match value[6] {
        0 => (7, None),
        1 if value.len() >= 23 => (
            23,
            Some(u64::from_be_bytes(
                value[7..15].try_into().expect("fixed slice"),
            )),
        ),
        _ => {
            return Err(
                io::Error::new(io::ErrorKind::InvalidData, "invalid pending-unload state").into(),
            );
        }
    };
    let chunk = decode_chunk(&value[offset..])?;
    Ok(json!({
        "x": chunk.position().x,
        "z": chunk.position().z,
        "status": status,
        "activity": activity,
        "revision": chunk.revision().get(),
        "pending_unload_token": pending_unload_token,
    }))
}

fn materialized_records(point: &RegionRecoveryPoint) -> Vec<SnapshotRecord> {
    use std::collections::BTreeMap;

    let mut records = BTreeMap::new();
    for record in point.snapshot().records().iter().chain(
        point
            .journal_tail()
            .iter()
            .flat_map(|frame| frame.records()),
    ) {
        records.insert(
            (
                record.kind(),
                record.domain().clone(),
                record.key().to_vec(),
            ),
            record.clone(),
        );
    }
    records.into_values().collect()
}

fn canonical_state_hash(records: &[SnapshotRecord]) -> [u8; 32] {
    let mut ordered = records.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        (left.kind(), left.domain(), left.key()).cmp(&(right.kind(), right.domain(), right.key()))
    });
    let mut hasher = blake3::Hasher::new();
    for record in ordered {
        hasher.update(&[record.kind() as u8]);
        hash_bytes(&mut hasher, record.domain().to_string().as_bytes());
        hash_bytes(&mut hasher, record.key());
        hash_bytes(&mut hasher, record.value());
    }
    *hasher.finalize().as_bytes()
}

fn hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
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

fn parse_i32(value: OsString) -> Result<i32, Box<dyn Error>> {
    Ok(value.to_string_lossy().parse()?)
}

fn required(
    arguments: &mut impl Iterator<Item = OsString>,
    executable: &OsStr,
    name: &'static str,
) -> Result<OsString, io::Error> {
    arguments
        .next()
        .ok_or_else(|| usage_error(executable, &format!("missing {name}")))
}

fn usage_error(executable: &OsStr, reason: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "{reason}\nusage: {} <store-directory> <world-id-hex> <dimension> <region-x> <region-z>",
            executable.to_string_lossy()
        ),
    )
}

const STATUS_NAMES: [&str; 12] = [
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

const ACTIVITY_NAMES: [&str; 4] = ["dormant", "accessible", "block_ticking", "entity_ticking"];

#[cfg(test)]
mod tests {
    use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
    use ferrite_foundation::resource::ResourceId;
    use ferrite_persistence::snapshot::{
        PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
        SnapshotRecord, SnapshotRecordKind,
    };

    use super::{
        CURRENT_WORLD_CHUNK_DOMAIN, CURRENT_WORLD_LEVEL_DOMAIN, LEGACY_WORLD_CHUNK_DOMAIN,
        LEGACY_WORLD_LEVEL_DOMAIN, continuity_generation,
    };

    fn point(domains: &[&str]) -> RegionRecoveryPoint {
        let records = domains
            .iter()
            .enumerate()
            .map(|(index, domain)| {
                SnapshotRecord::new(
                    if domain.ends_with("chunk_v1") {
                        SnapshotRecordKind::Chunk
                    } else {
                        SnapshotRecordKind::Extension
                    },
                    domain.parse().unwrap(),
                    vec![index as u8],
                    Vec::new(),
                )
                .unwrap()
            })
            .collect();
        let snapshot = RegionCommitSnapshot::new(
            RegionSnapshotHeader {
                key: SimulationRegionKey::new(
                    WorldId::new(1).unwrap(),
                    DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
                    RegionCoord::new(0, 0),
                    RegionMappingVersion::V1,
                ),
                generation: ActivationGeneration::INITIAL,
                committed_tick: 0,
                persistence_revision: PersistenceRevision::INITIAL,
                region_side_chunks: 8,
                content_manifest: [0; 32],
                state_hash: [0; 32],
            },
            records,
        )
        .unwrap();
        RegionRecoveryPoint::new(snapshot, Vec::new()).unwrap()
    }

    #[test]
    fn classifies_complete_legacy_and_current_world_generations() {
        assert_eq!(
            continuity_generation(&point(&[
                LEGACY_WORLD_CHUNK_DOMAIN,
                LEGACY_WORLD_LEVEL_DOMAIN,
            ]))
            .unwrap(),
            "legacy"
        );
        assert_eq!(
            continuity_generation(&point(&[
                CURRENT_WORLD_CHUNK_DOMAIN,
                CURRENT_WORLD_LEVEL_DOMAIN,
            ]))
            .unwrap(),
            "current"
        );
    }

    #[test]
    fn rejects_mixed_and_unsupported_world_generations() {
        assert!(
            continuity_generation(&point(&[
                LEGACY_WORLD_CHUNK_DOMAIN,
                CURRENT_WORLD_LEVEL_DOMAIN,
            ]))
            .is_err()
        );
        assert!(continuity_generation(&point(&["ferrite:world-service/chunk_v2"])).is_err());
    }
}
