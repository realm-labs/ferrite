//! Versioned formal-world identity, generation, dimension, and save policy.

use ferrite_foundation::identity::WorldId;
use ferrite_foundation::resource::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const CURRENT_WORLD_GENERATOR: &str = "ferrite:overworld_v1";
pub const LEGACY_WORLD_ID: &str = "00000000000000000000000000000001";
const OVERWORLD: &str = "minecraft:overworld";
const SUPPORTED_DIMENSIONS: [&str; 3] = [OVERWORLD, "minecraft:the_nether", "minecraft:the_end"];
const MINIMUM_BUILD_Y: i32 = -64;
const MAXIMUM_BUILD_Y: i32 = 383;
const MAXIMUM_SPAWN_COORDINATE: i32 = 29_999_974;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldConfig {
    pub id: String,
    pub seed: i64,
    pub generator: String,
    pub spawn: SpawnPolicy,
    pub view_distance: u16,
    pub simulation_distance: u16,
    pub dimensions: Vec<String>,
    pub save: WorldSavePolicy,
}

impl WorldConfig {
    pub(crate) fn legacy_defaults() -> Self {
        Self {
            id: LEGACY_WORLD_ID.to_owned(),
            seed: 0,
            generator: CURRENT_WORLD_GENERATOR.to_owned(),
            spawn: SpawnPolicy::Generated,
            view_distance: 10,
            simulation_distance: 10,
            dimensions: vec![OVERWORLD.to_owned()],
            save: WorldSavePolicy {
                autosave_interval_ticks: 6_000,
                max_pending_region_saves: 128,
                checkpoint_interval_commits: 64,
                shutdown_flush: ShutdownFlushPolicy::Required,
            },
        }
    }

    pub(crate) fn validate(&self) -> Result<WorldId, WorldConfigError> {
        let world_id = self
            .id
            .parse::<WorldId>()
            .map_err(|_| WorldConfigError::InvalidWorldId)?;
        if world_id.to_string() != self.id {
            return Err(WorldConfigError::NonCanonicalWorldId);
        }
        let generator = canonical_resource("generator", &self.generator)?;
        if generator.to_string() != CURRENT_WORLD_GENERATOR {
            return Err(WorldConfigError::UnsupportedGenerator(
                self.generator.clone(),
            ));
        }
        if !(2..=32).contains(&self.view_distance)
            || !(2..=32).contains(&self.simulation_distance)
            || self.simulation_distance > self.view_distance
        {
            return Err(WorldConfigError::InvalidDistances);
        }
        if self.dimensions.is_empty()
            || self.dimensions.first().map(String::as_str) != Some(OVERWORLD)
        {
            return Err(WorldConfigError::OverworldMustBeFirst);
        }
        let mut dimensions = BTreeSet::new();
        for dimension in &self.dimensions {
            let canonical = canonical_resource("dimension", dimension)?.to_string();
            if !SUPPORTED_DIMENSIONS.contains(&canonical.as_str()) {
                return Err(WorldConfigError::UnsupportedDimension(dimension.clone()));
            }
            if !dimensions.insert(canonical) {
                return Err(WorldConfigError::DuplicateDimension(dimension.clone()));
            }
        }
        if self.save.autosave_interval_ticks == 0
            || self.save.autosave_interval_ticks > 1_200_000
            || !(1..=4_096).contains(&self.save.max_pending_region_saves)
            || !(1..=65_536).contains(&self.save.checkpoint_interval_commits)
        {
            return Err(WorldConfigError::InvalidSavePolicy);
        }
        if let SpawnPolicy::Fixed { y, .. } = self.spawn
            && !(MINIMUM_BUILD_Y..=MAXIMUM_BUILD_Y).contains(&y)
        {
            return Err(WorldConfigError::InvalidSpawnHeight(y));
        }
        if let SpawnPolicy::Fixed { x, z, .. } = self.spawn
            && (x.unsigned_abs() > MAXIMUM_SPAWN_COORDINATE as u32
                || z.unsigned_abs() > MAXIMUM_SPAWN_COORDINATE as u32)
        {
            return Err(WorldConfigError::InvalidSpawnCoordinate { x, z });
        }
        Ok(world_id)
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self::legacy_defaults()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum SpawnPolicy {
    Generated,
    Fixed { x: i32, y: i32, z: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldSavePolicy {
    pub autosave_interval_ticks: u64,
    pub max_pending_region_saves: usize,
    pub checkpoint_interval_commits: u32,
    pub shutdown_flush: ShutdownFlushPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShutdownFlushPolicy {
    Required,
}

pub(crate) fn validate_legacy_storage_root(
    storage_root: &Path,
    expected_world: WorldId,
) -> Result<(), WorldConfigError> {
    let worlds = storage_root.join("worlds");
    let metadata = match fs::symlink_metadata(&worlds) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(WorldConfigError::InspectStorage {
                path: worlds,
                source,
            });
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(WorldConfigError::UnsafeWorldRoot(worlds));
    }
    let expected = expected_world.to_string();
    for entry in fs::read_dir(&worlds).map_err(|source| WorldConfigError::InspectStorage {
        path: worlds.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| WorldConfigError::InspectStorage {
            path: worlds.clone(),
            source,
        })?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| WorldConfigError::ConflictingWorld("<non-utf8>".to_owned()))?;
        let file_type = entry
            .file_type()
            .map_err(|source| WorldConfigError::InspectStorage {
                path: entry.path(),
                source,
            })?;
        if name != expected || file_type.is_symlink() || !file_type.is_dir() {
            return Err(WorldConfigError::ConflictingWorld(name.to_owned()));
        }
    }
    Ok(())
}

fn canonical_resource(label: &'static str, value: &str) -> Result<ResourceId, WorldConfigError> {
    let resource = value
        .parse::<ResourceId>()
        .map_err(|_| WorldConfigError::InvalidResource {
            label,
            value: value.to_owned(),
        })?;
    if resource.to_string() != value {
        return Err(WorldConfigError::NonCanonicalResource {
            label,
            value: value.to_owned(),
        });
    }
    Ok(resource)
}

#[derive(Debug, Error)]
pub enum WorldConfigError {
    #[error("world ID must be one nonzero 32-character lowercase hexadecimal identity")]
    InvalidWorldId,
    #[error("world ID has a noncanonical spelling")]
    NonCanonicalWorldId,
    #[error("world {label} resource {value:?} is invalid")]
    InvalidResource { label: &'static str, value: String },
    #[error("world {label} resource {value:?} is not canonical")]
    NonCanonicalResource { label: &'static str, value: String },
    #[error("world generator {0:?} is not supported by this server")]
    UnsupportedGenerator(String),
    #[error(
        "world view and simulation distances must be within 2..=32 and simulation cannot exceed view"
    )]
    InvalidDistances,
    #[error("the configured dimension list must be nonempty and begin with minecraft:overworld")]
    OverworldMustBeFirst,
    #[error("world dimension {0:?} is not supported")]
    UnsupportedDimension(String),
    #[error("world dimension {0:?} is duplicated")]
    DuplicateDimension(String),
    #[error("world save policy is outside its bounded ranges")]
    InvalidSavePolicy,
    #[error("fixed spawn Y {0} is outside the configured build range")]
    InvalidSpawnHeight(i32),
    #[error(
        "fixed spawn X/Z ({x}, {z}) must leave the bounded respawn search inside the world border"
    )]
    InvalidSpawnCoordinate { x: i32, z: i32 },
    #[error("inspect durable world root {path}: {source}")]
    InspectStorage {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("durable world root is not one contained directory: {0}")]
    UnsafeWorldRoot(PathBuf),
    #[error("schema-1 migration found conflicting durable world entry {0:?}")]
    ConflictingWorld(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_defaults_are_canonical_and_bounded() {
        let config = WorldConfig::legacy_defaults();
        assert_eq!(config.validate().unwrap().get(), 1);
        assert_eq!(config.dimensions, [OVERWORLD]);
    }

    #[test]
    fn identity_resources_dimensions_and_bounds_fail_closed() {
        let mut config = WorldConfig::legacy_defaults();
        config.id = "1".to_owned();
        assert!(matches!(
            config.validate(),
            Err(WorldConfigError::InvalidWorldId)
        ));

        let mut config = WorldConfig::legacy_defaults();
        config.generator = "overworld_v1".to_owned();
        assert!(matches!(
            config.validate(),
            Err(WorldConfigError::NonCanonicalResource { .. })
        ));

        let mut config = WorldConfig::legacy_defaults();
        config.dimensions.push(OVERWORLD.to_owned());
        assert!(matches!(
            config.validate(),
            Err(WorldConfigError::DuplicateDimension(_))
        ));

        let mut config = WorldConfig::legacy_defaults();
        config.simulation_distance = 11;
        assert!(matches!(
            config.validate(),
            Err(WorldConfigError::InvalidDistances)
        ));

        let mut config = WorldConfig::legacy_defaults();
        config.spawn = SpawnPolicy::Fixed {
            x: i32::MIN,
            y: 64,
            z: 0,
        };
        assert!(matches!(
            config.validate(),
            Err(WorldConfigError::InvalidSpawnCoordinate { .. })
        ));
    }
}
