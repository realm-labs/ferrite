//! Durable identity and compatibility metadata for the formal world.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use ferrite_persistence::snapshot::{
    PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
    SnapshotError, SnapshotRecord, SnapshotRecordKind,
};
use ferrite_persistence::store::{CommitReceipt, RegionFileStore, StoreError};
use thiserror::Error;

use crate::config::ValidatedServerConfig;
use crate::continuity::identity::{ContinuityDomain, ContinuityGeneration, domain_id};
use crate::continuity::migration::{
    ContinuityMigrationError, StoreMigrationError, canonical_record_hash, commit_current_point,
    normalize_recovery_point,
};
use crate::world_config::SpawnPolicy;
use crate::world_service::continuity::materialized_records;

const METADATA_MAGIC: &[u8; 4] = b"FWM0";
const METADATA_SCHEMA_V1: u16 = 1;
const CHUNK_FORMAT_V1: u16 = 1;
const REGION_SIDE_CHUNKS: u16 = 8;
const MAX_RESOURCE_BYTES: usize = 256;
const MAX_DIMENSIONS: usize = 3;
const METADATA_KEY: &[u8] = b"world";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorldMetadata {
    world: WorldId,
    seed: i64,
    generator: ResourceId,
    spawn: BlockPos,
    dimensions: Vec<DimensionId>,
    mapping_version: RegionMappingVersion,
    chunk_format: u16,
    content_manifest: [u8; 32],
}

impl WorldMetadata {
    fn from_config(
        config: &ValidatedServerConfig,
        content_manifest: [u8; 32],
    ) -> Result<Self, WorldMetadataError> {
        let world = &config.config().world;
        let generator = world.generator.parse::<ResourceId>()?;
        let dimensions = world
            .dimensions
            .iter()
            .map(|dimension| dimension.parse::<DimensionId>())
            .collect::<Result<Vec<_>, _>>()?;
        let spawn = match world.spawn {
            SpawnPolicy::Generated => BlockPos::new(8, 64, 8),
            SpawnPolicy::Fixed { x, y, z } => BlockPos::new(x, y, z),
        };
        Ok(Self {
            world: config.world_id(),
            seed: world.seed,
            generator,
            spawn,
            dimensions,
            mapping_version: RegionMappingVersion::V1,
            chunk_format: CHUNK_FORMAT_V1,
            content_manifest,
        })
    }

    pub(crate) const fn world(&self) -> WorldId {
        self.world
    }

    pub(crate) const fn spawn(&self) -> BlockPos {
        self.spawn
    }

    pub(crate) fn overworld(&self) -> &DimensionId {
        self.dimensions
            .first()
            .expect("validated world metadata always contains the overworld")
    }

    pub(crate) fn dimensions(&self) -> &[DimensionId] {
        &self.dimensions
    }

    fn encode_record(&self) -> Result<SnapshotRecord, WorldMetadataError> {
        let mut value = Vec::new();
        value.extend_from_slice(METADATA_MAGIC);
        value.extend_from_slice(&METADATA_SCHEMA_V1.to_be_bytes());
        value.extend_from_slice(&self.world.get().to_be_bytes());
        value.extend_from_slice(&self.seed.to_be_bytes());
        push_resource(&mut value, &self.generator)?;
        value.extend_from_slice(&self.spawn.x.to_be_bytes());
        value.extend_from_slice(&self.spawn.y.to_be_bytes());
        value.extend_from_slice(&self.spawn.z.to_be_bytes());
        value.push(u8::try_from(self.dimensions.len()).expect("dimension bound fits in u8"));
        for dimension in &self.dimensions {
            push_resource(&mut value, dimension.resource())?;
        }
        value.extend_from_slice(&self.mapping_version.get().to_be_bytes());
        value.extend_from_slice(&self.chunk_format.to_be_bytes());
        value.extend_from_slice(&self.content_manifest);
        SnapshotRecord::new(
            SnapshotRecordKind::Extension,
            metadata_domain(),
            METADATA_KEY.to_vec(),
            value,
        )
        .map_err(Into::into)
    }

    fn decode_record(record: &SnapshotRecord) -> Result<Option<Self>, WorldMetadataError> {
        if record.domain() != &metadata_domain() {
            return Ok(None);
        }
        if record.kind() != SnapshotRecordKind::Extension || record.key() != METADATA_KEY {
            return Err(WorldMetadataError::InvalidRecordIdentity);
        }
        let mut cursor = Cursor::new(record.value());
        cursor.expect(METADATA_MAGIC)?;
        let schema = cursor.u16()?;
        if schema != METADATA_SCHEMA_V1 {
            return Err(WorldMetadataError::UnsupportedSchema(schema));
        }
        let world =
            WorldId::new(cursor.u128()?).map_err(|_| WorldMetadataError::InvalidWorldIdentity)?;
        let seed = cursor.i64()?;
        let generator = cursor.resource()?;
        let spawn = BlockPos::new(cursor.i32()?, cursor.i32()?, cursor.i32()?);
        let dimension_count = usize::from(cursor.u8()?);
        if dimension_count == 0 || dimension_count > MAX_DIMENSIONS {
            return Err(WorldMetadataError::InvalidDimensionCount(dimension_count));
        }
        let dimensions = (0..dimension_count)
            .map(|_| cursor.resource().map(DimensionId::new))
            .collect::<Result<Vec<_>, _>>()?;
        let mapping_version = RegionMappingVersion::new(cursor.u16()?)
            .map_err(|_| WorldMetadataError::InvalidMappingVersion)?;
        let chunk_format = cursor.u16()?;
        let content_manifest = cursor.fixed()?;
        cursor.finish()?;
        Ok(Some(Self {
            world,
            seed,
            generator,
            spawn,
            dimensions,
            mapping_version,
            chunk_format,
            content_manifest,
        }))
    }
}

pub(crate) struct DurableWorldMetadata {
    metadata: WorldMetadata,
    control_point: RegionRecoveryPoint,
}

impl DurableWorldMetadata {
    pub(crate) const fn metadata(&self) -> &WorldMetadata {
        &self.metadata
    }

    pub(crate) const fn control_point(&self) -> &RegionRecoveryPoint {
        &self.control_point
    }

    pub(crate) fn metadata_record(&self) -> Result<SnapshotRecord, WorldMetadataError> {
        self.metadata.encode_record()
    }
}

pub(crate) fn load_or_create(
    config: &ValidatedServerConfig,
    content_manifest: [u8; 32],
) -> Result<DurableWorldMetadata, WorldMetadataError> {
    let expected = WorldMetadata::from_config(config, content_manifest)?;
    let key = control_region_key(&expected);
    let store_root = control_region_store(config.config().storage.root.as_path(), &key)?;
    let pristine = directory_is_empty(&store_root)?;
    let mut store = RegionFileStore::open(&store_root)?;
    let (metadata, control_point) = match store.load(&key)? {
        Some(point) => (load_metadata(&point, &expected)?, point),
        None if pristine => {
            let point = initial_recovery_point(&key, &expected)?;
            let receipt = commit_current_point(&mut store, &point)?;
            validate_initial_receipt(receipt)?;
            (expected, point)
        }
        None => return Err(WorldMetadataError::ExistingStoreWithoutMetadata(store_root)),
    };
    Ok(DurableWorldMetadata {
        metadata,
        control_point,
    })
}

fn load_metadata(
    point: &RegionRecoveryPoint,
    expected: &WorldMetadata,
) -> Result<WorldMetadata, WorldMetadataError> {
    let point = normalize_recovery_point(point)?;
    let header = point.snapshot().header();
    if header.region_side_chunks != REGION_SIDE_CHUNKS {
        return Err(WorldMetadataError::MetadataMismatch("Region size"));
    }
    if header.content_manifest != expected.content_manifest {
        return Err(WorldMetadataError::MetadataMismatch("content manifest"));
    }
    let records = materialized_records(&point);
    let mut decoded = records
        .iter()
        .filter_map(|record| WorldMetadata::decode_record(record).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    if decoded.len() != 1 {
        return Err(WorldMetadataError::MetadataRecordCount(decoded.len()));
    }
    let actual = decoded.pop().expect("one decoded metadata record");
    validate_compatibility(&actual, expected)?;
    Ok(actual)
}

fn validate_compatibility(
    actual: &WorldMetadata,
    expected: &WorldMetadata,
) -> Result<(), WorldMetadataError> {
    let fields = [
        (actual.world == expected.world, "world identity"),
        (actual.seed == expected.seed, "seed"),
        (actual.generator == expected.generator, "generator"),
        (actual.spawn == expected.spawn, "spawn"),
        (
            actual.dimensions == expected.dimensions,
            "dimension catalog",
        ),
        (
            actual.mapping_version == expected.mapping_version,
            "Region mapping",
        ),
        (actual.chunk_format == expected.chunk_format, "chunk format"),
        (
            actual.content_manifest == expected.content_manifest,
            "content manifest",
        ),
    ];
    for (matches, field) in fields {
        if !matches {
            return Err(WorldMetadataError::MetadataMismatch(field));
        }
    }
    Ok(())
}

fn initial_recovery_point(
    key: &SimulationRegionKey,
    metadata: &WorldMetadata,
) -> Result<RegionRecoveryPoint, WorldMetadataError> {
    let records = vec![metadata.encode_record()?];
    let snapshot = RegionCommitSnapshot::new(
        RegionSnapshotHeader {
            key: key.clone(),
            generation: ActivationGeneration::INITIAL,
            committed_tick: 0,
            persistence_revision: PersistenceRevision::INITIAL,
            region_side_chunks: REGION_SIDE_CHUNKS,
            content_manifest: metadata.content_manifest,
            state_hash: canonical_record_hash(&records),
        },
        records,
    )?;
    RegionRecoveryPoint::new(snapshot, Vec::new()).map_err(Into::into)
}

fn validate_initial_receipt(receipt: CommitReceipt) -> Result<(), WorldMetadataError> {
    if receipt.revision() == PersistenceRevision::INITIAL && receipt.committed_tick() == 0 {
        Ok(())
    } else {
        Err(WorldMetadataError::InvalidInitialReceipt)
    }
}

fn control_region_key(metadata: &WorldMetadata) -> SimulationRegionKey {
    SimulationRegionKey::new(
        metadata.world,
        metadata.overworld().clone(),
        RegionCoord::new(0, 0),
        metadata.mapping_version,
    )
}

fn control_region_store(
    storage_root: &Path,
    key: &SimulationRegionKey,
) -> Result<PathBuf, WorldMetadataError> {
    fs::create_dir_all(storage_root).map_err(|source| WorldMetadataError::Io {
        path: storage_root.to_path_buf(),
        source,
    })?;
    let containment_root =
        fs::canonicalize(storage_root).map_err(|source| WorldMetadataError::Io {
            path: storage_root.to_path_buf(),
            source,
        })?;
    let mut current = containment_root.clone();
    for component in ["worlds", &key.world().to_string(), "dimensions"] {
        current = checked_child(&current, component, &containment_root)?;
    }
    current = checked_child(
        &current,
        key.dimension().resource().namespace(),
        &containment_root,
    )?;
    for component in key.dimension().resource().path().split('/') {
        current = checked_child(&current, component, &containment_root)?;
    }
    current = checked_child(&current, "regions", &containment_root)?;
    current = checked_child(&current, &region_directory_name(key), &containment_root)?;
    Ok(current)
}

fn checked_child(
    parent: &Path,
    component: &str,
    containment_root: &Path,
) -> Result<PathBuf, WorldMetadataError> {
    if component.is_empty() || matches!(component, "." | "..") || component.contains(['/', '\\']) {
        return Err(WorldMetadataError::UnsafePathComponent(
            component.to_owned(),
        ));
    }
    let child = parent.join(component);
    match fs::create_dir(&child) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(source) => {
            return Err(WorldMetadataError::Io {
                path: child,
                source,
            });
        }
    }
    let metadata = fs::symlink_metadata(&child).map_err(|source| WorldMetadataError::Io {
        path: child.clone(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(WorldMetadataError::UnsafeStorePath(child));
    }
    let canonical = fs::canonicalize(&child).map_err(|source| WorldMetadataError::Io {
        path: child,
        source,
    })?;
    if !canonical.starts_with(containment_root) {
        return Err(WorldMetadataError::UnsafeStorePath(canonical));
    }
    Ok(canonical)
}

pub(crate) fn region_store_root(
    storage_root: &Path,
    key: &SimulationRegionKey,
) -> Result<PathBuf, WorldMetadataError> {
    control_region_store(storage_root, key)
}

fn region_directory_name(key: &SimulationRegionKey) -> String {
    format!("r.{}.{}", key.coordinate().x(), key.coordinate().z())
}

pub(crate) fn directory_is_empty(path: &Path) -> Result<bool, WorldMetadataError> {
    let mut entries = fs::read_dir(path).map_err(|source| WorldMetadataError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(entries
        .next()
        .transpose()
        .map_err(|source| WorldMetadataError::Io {
            path: path.to_path_buf(),
            source,
        })?
        .is_none())
}

fn metadata_domain() -> ResourceId {
    domain_id(
        ContinuityDomain::WorldMetadata,
        ContinuityGeneration::Current,
    )
}

fn push_resource(output: &mut Vec<u8>, resource: &ResourceId) -> Result<(), WorldMetadataError> {
    let value = resource.to_string();
    if value.len() > MAX_RESOURCE_BYTES {
        return Err(WorldMetadataError::ResourceTooLong(value.len()));
    }
    output.extend_from_slice(
        &u16::try_from(value.len())
            .expect("resource bound fits in u16")
            .to_be_bytes(),
    );
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], WorldMetadataError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(WorldMetadataError::Truncated)?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or(WorldMetadataError::Truncated)?;
        self.offset = end;
        Ok(bytes)
    }

    fn expect(&mut self, expected: &[u8]) -> Result<(), WorldMetadataError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(WorldMetadataError::WrongMagic)
        }
    }

    fn u8(&mut self) -> Result<u8, WorldMetadataError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, WorldMetadataError> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    fn i32(&mut self) -> Result<i32, WorldMetadataError> {
        Ok(i32::from_be_bytes(self.fixed()?))
    }

    fn i64(&mut self) -> Result<i64, WorldMetadataError> {
        Ok(i64::from_be_bytes(self.fixed()?))
    }

    fn u128(&mut self) -> Result<u128, WorldMetadataError> {
        Ok(u128::from_be_bytes(self.fixed()?))
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], WorldMetadataError> {
        self.take(N)?
            .try_into()
            .map_err(|_| WorldMetadataError::Truncated)
    }

    fn resource(&mut self) -> Result<ResourceId, WorldMetadataError> {
        let length = usize::from(self.u16()?);
        if length > MAX_RESOURCE_BYTES {
            return Err(WorldMetadataError::ResourceTooLong(length));
        }
        let value =
            std::str::from_utf8(self.take(length)?).map_err(|_| WorldMetadataError::InvalidUtf8)?;
        value.parse().map_err(Into::into)
    }

    fn finish(self) -> Result<(), WorldMetadataError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(WorldMetadataError::TrailingBytes)
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum WorldMetadataError {
    #[error("world metadata has the wrong magic")]
    WrongMagic,
    #[error("world metadata is truncated")]
    Truncated,
    #[error("world metadata contains trailing bytes")]
    TrailingBytes,
    #[error("world metadata contains invalid UTF-8")]
    InvalidUtf8,
    #[error("world metadata schema {0} is unsupported")]
    UnsupportedSchema(u16),
    #[error("world metadata has an invalid world identity")]
    InvalidWorldIdentity,
    #[error("world metadata has invalid Region mapping")]
    InvalidMappingVersion,
    #[error("world metadata dimension count {0} is outside 1..={MAX_DIMENSIONS}")]
    InvalidDimensionCount(usize),
    #[error("world metadata resource length {0} exceeds {MAX_RESOURCE_BYTES}")]
    ResourceTooLong(usize),
    #[error("world metadata record has the wrong kind or key")]
    InvalidRecordIdentity,
    #[error("world metadata contains {0} canonical records instead of one")]
    MetadataRecordCount(usize),
    #[error("durable world metadata does not match configured {0}")]
    MetadataMismatch(&'static str),
    #[error("durable Region store exists without committed world metadata: {0}")]
    ExistingStoreWithoutMetadata(PathBuf),
    #[error("durable world path component is unsafe: {0:?}")]
    UnsafePathComponent(String),
    #[error("durable world store path is not a contained directory: {0}")]
    UnsafeStorePath(PathBuf),
    #[error("durable world metadata commit returned an invalid initial receipt")]
    InvalidInitialReceipt,
    #[error("durable world metadata I/O at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Resource(#[from] ResourceIdError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Migration(#[from] ContinuityMigrationError),
    #[error(transparent)]
    StoreMigration(#[from] StoreMigrationError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;

    fn validated_config(root: &Path) -> ValidatedServerConfig {
        let config = ServerConfig::development_node(1, 1, 30_000, root).unwrap();
        ServerConfig::from_toml(&config.to_toml().unwrap()).unwrap()
    }

    #[test]
    fn metadata_codec_round_trips_and_rejects_future_schema() {
        let temporary = tempfile::tempdir().unwrap();
        let expected =
            WorldMetadata::from_config(&validated_config(temporary.path()), [7; 32]).unwrap();
        let record = expected.encode_record().unwrap();
        assert_eq!(
            WorldMetadata::decode_record(&record).unwrap(),
            Some(expected)
        );

        let mut future = record.value().to_vec();
        future[4..6].copy_from_slice(&2_u16.to_be_bytes());
        let record = SnapshotRecord::new(
            SnapshotRecordKind::Extension,
            metadata_domain(),
            METADATA_KEY.to_vec(),
            future,
        )
        .unwrap();
        assert!(matches!(
            WorldMetadata::decode_record(&record),
            Err(WorldMetadataError::UnsupportedSchema(2))
        ));
    }

    #[test]
    fn first_boot_creates_metadata_and_restart_loads_it() {
        let temporary = tempfile::tempdir().unwrap();
        let config = validated_config(temporary.path());
        let first = load_or_create(&config, [8; 32]).unwrap();
        let expected = WorldMetadata::from_config(&config, [8; 32]).unwrap();
        let key = control_region_key(&expected);
        let store_root = control_region_store(&config.config().storage.root, &key).unwrap();
        assert!(store_root.join("region-journal.log").is_file());

        let second = load_or_create(&config, [8; 32]).unwrap();
        assert_eq!(second.metadata(), first.metadata());
    }

    #[test]
    fn incompatible_config_and_uncommitted_existing_store_fail_closed() {
        let temporary = tempfile::tempdir().unwrap();
        let config = validated_config(temporary.path());
        load_or_create(&config, [9; 32]).unwrap();

        let mut changed = config.config().clone();
        changed.world.seed = 42;
        let changed = ServerConfig::from_toml(&changed.to_toml().unwrap()).unwrap();
        assert!(matches!(
            load_or_create(&changed, [9; 32]),
            Err(WorldMetadataError::MetadataMismatch("seed"))
        ));

        let other = tempfile::tempdir().unwrap();
        let config = validated_config(other.path());
        let expected = WorldMetadata::from_config(&config, [9; 32]).unwrap();
        let key = control_region_key(&expected);
        let root = control_region_store(&config.config().storage.root, &key).unwrap();
        fs::write(root.join("region-journal.log"), b"torn").unwrap();
        assert!(load_or_create(&config, [9; 32]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_world_component_is_rejected() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let config = validated_config(temporary.path());
        fs::create_dir_all(&config.config().storage.root).unwrap();
        let outside = temporary.path().join("outside");
        fs::create_dir(&outside).unwrap();
        symlink(&outside, config.config().storage.root.join("worlds")).unwrap();
        assert!(matches!(
            load_or_create(&config, [3; 32]),
            Err(WorldMetadataError::UnsafeStorePath(_))
        ));
    }
}
