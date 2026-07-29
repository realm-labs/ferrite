//! Stable semantic Region projections used by canonical state hashes.

use crate::codec::{CanonicalEncode, EncodeError, Encoder};
use crate::hash::{RegionHashRecord, StateHash, StateHashError, hash_region, hash_world};
use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::{StableEntityId, WorldId};
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

pub const MAX_PROJECTED_BLOCKS: usize = 16 * 1024 * 1024;
pub const MAX_PROJECTED_ENTITIES: usize = 1024 * 1024;
pub const MAX_EXTENSION_RECORDS: usize = 1024 * 1024;
pub const MAX_PROJECTION_BYTES: usize = 1024 * 1024;
const PROJECTION_SCHEMA_V1: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateProjection {
    position: BlockPos,
    state: ResourceId,
}

impl BlockStateProjection {
    pub const fn new(position: BlockPos, state: ResourceId) -> Self {
        Self { position, state }
    }

    pub const fn position(&self) -> BlockPos {
        self.position
    }

    pub const fn state(&self) -> &ResourceId {
        &self.state
    }
}

impl CanonicalEncode for BlockStateProjection {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        self.position.encode(encoder)?;
        self.state.encode(encoder)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityStateProjection {
    stable_id: StableEntityId,
    kind: ResourceId,
    state: ProjectionBytes,
}

impl EntityStateProjection {
    pub fn new(
        stable_id: StableEntityId,
        kind: ResourceId,
        state: Vec<u8>,
    ) -> Result<Self, ProjectionError> {
        Ok(Self {
            stable_id,
            kind,
            state: ProjectionBytes::new(state)?,
        })
    }

    pub const fn stable_id(&self) -> StableEntityId {
        self.stable_id
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn state(&self) -> &[u8] {
        self.state.as_slice()
    }
}

impl CanonicalEncode for EntityStateProjection {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        self.stable_id.encode(encoder)?;
        self.kind.encode(encoder)?;
        self.state.encode(encoder)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionStateProjection {
    domain: ResourceId,
    key: ProjectionBytes,
    value: ProjectionBytes,
}

impl ExtensionStateProjection {
    pub fn new(domain: ResourceId, key: Vec<u8>, value: Vec<u8>) -> Result<Self, ProjectionError> {
        Ok(Self {
            domain,
            key: ProjectionBytes::new(key)?,
            value: ProjectionBytes::new(value)?,
        })
    }

    pub const fn domain(&self) -> &ResourceId {
        &self.domain
    }

    pub fn key(&self) -> &[u8] {
        self.key.as_slice()
    }

    pub fn value(&self) -> &[u8] {
        self.value.as_slice()
    }
}

impl CanonicalEncode for ExtensionStateProjection {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        self.domain.encode(encoder)?;
        self.key.encode(encoder)?;
        self.value.encode(encoder)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionStateProjection {
    key: SimulationRegionKey,
    region_side_chunks: u16,
    blocks: Box<[BlockStateProjection]>,
    entities: Box<[EntityStateProjection]>,
    extensions: Box<[ExtensionStateProjection]>,
}

impl RegionStateProjection {
    pub fn new(
        key: SimulationRegionKey,
        mapping: RegionMapping,
        mut blocks: Vec<BlockStateProjection>,
        mut entities: Vec<EntityStateProjection>,
        mut extensions: Vec<ExtensionStateProjection>,
    ) -> Result<Self, ProjectionError> {
        if key.mapping_version() != mapping.version() {
            return Err(ProjectionError::MappingVersionMismatch);
        }
        enforce_count("blocks", blocks.len(), MAX_PROJECTED_BLOCKS)?;
        enforce_count("entities", entities.len(), MAX_PROJECTED_ENTITIES)?;
        enforce_count("extension records", extensions.len(), MAX_EXTENSION_RECORDS)?;
        for block in &blocks {
            let owner = mapping.region_for_chunk(
                key.world(),
                key.dimension().clone(),
                block.position.chunk(),
            );
            if owner != key {
                return Err(ProjectionError::BlockOutsideRegion(block.position));
            }
        }
        blocks.sort_by_key(BlockStateProjection::position);
        if blocks
            .windows(2)
            .any(|pair| pair[0].position == pair[1].position)
        {
            return Err(ProjectionError::DuplicateBlock);
        }
        entities.sort_by_key(EntityStateProjection::stable_id);
        if entities
            .windows(2)
            .any(|pair| pair[0].stable_id == pair[1].stable_id)
        {
            return Err(ProjectionError::DuplicateEntity);
        }
        extensions.sort_by(|left, right| {
            (&left.domain, left.key.as_slice()).cmp(&(&right.domain, right.key.as_slice()))
        });
        if extensions
            .windows(2)
            .any(|pair| pair[0].domain == pair[1].domain && pair[0].key == pair[1].key)
        {
            return Err(ProjectionError::DuplicateExtension);
        }
        Ok(Self {
            key,
            region_side_chunks: mapping.region_size().side_chunks(),
            blocks: blocks.into_boxed_slice(),
            entities: entities.into_boxed_slice(),
            extensions: extensions.into_boxed_slice(),
        })
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub fn blocks(&self) -> &[BlockStateProjection] {
        &self.blocks
    }

    pub fn entities(&self) -> &[EntityStateProjection] {
        &self.entities
    }

    pub fn extensions(&self) -> &[ExtensionStateProjection] {
        &self.extensions
    }

    pub fn hash(&self, committed_tick: GameTick) -> Result<StateHash, ProjectionHashError> {
        Ok(hash_region(&self.key, committed_tick.get(), self)?)
    }
}

impl CanonicalEncode for RegionStateProjection {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u16(PROJECTION_SCHEMA_V1);
        encoder.write_u16(self.region_side_chunks);
        encode_slice(encoder, &self.blocks)?;
        encode_slice(encoder, &self.entities)?;
        encode_slice(encoder, &self.extensions)
    }
}

pub fn hash_projected_world(
    world: WorldId,
    committed_tick: GameTick,
    content_manifest: StateHash,
    projections: &[RegionStateProjection],
) -> Result<StateHash, ProjectionHashError> {
    let records = projections
        .iter()
        .map(|projection| {
            Ok(RegionHashRecord::new(
                projection.key().clone(),
                projection.hash(committed_tick)?,
            ))
        })
        .collect::<Result<Vec<_>, ProjectionHashError>>()?;
    Ok(hash_world(
        world,
        committed_tick.get(),
        content_manifest,
        records,
    )?)
}

fn encode_slice<T: CanonicalEncode>(
    encoder: &mut Encoder,
    values: &[T],
) -> Result<(), EncodeError> {
    encoder.write_var_u64(values.len() as u64);
    for value in values {
        value.encode(encoder)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectionBytes(Vec<u8>);

impl ProjectionBytes {
    fn new(bytes: Vec<u8>) -> Result<Self, ProjectionError> {
        if bytes.len() > MAX_PROJECTION_BYTES {
            return Err(ProjectionError::PayloadTooLarge {
                actual: bytes.len(),
                maximum: MAX_PROJECTION_BYTES,
            });
        }
        Ok(Self(bytes))
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl CanonicalEncode for ProjectionBytes {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_bytes(&self.0, MAX_PROJECTION_BYTES)
    }
}

fn enforce_count(kind: &'static str, actual: usize, maximum: usize) -> Result<(), ProjectionError> {
    if actual > maximum {
        return Err(ProjectionError::TooManyRecords {
            kind,
            actual,
            maximum,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProjectionError {
    #[error("Region projection mapping version does not match its key")]
    MappingVersionMismatch,
    #[error("projected block {0:?} is outside its Region")]
    BlockOutsideRegion(BlockPos),
    #[error("Region projection contains a duplicate block position")]
    DuplicateBlock,
    #[error("Region projection contains a duplicate stable entity")]
    DuplicateEntity,
    #[error("Region projection contains a duplicate extension domain/key")]
    DuplicateExtension,
    #[error("{kind} has {actual} records, exceeding the {maximum}-record limit")]
    TooManyRecords {
        kind: &'static str,
        actual: usize,
        maximum: usize,
    },
    #[error("projection payload has {actual} bytes, exceeding the {maximum}-byte limit")]
    PayloadTooLarge { actual: usize, maximum: usize },
}

#[derive(Debug, Error)]
pub enum ProjectionHashError {
    #[error(transparent)]
    Hash(#[from] StateHashError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};

    fn region() -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        )
    }

    fn projection(reversed: bool) -> RegionStateProjection {
        let mut blocks = vec![
            BlockStateProjection::new(
                BlockPos::new(1, 2, 3),
                ResourceId::minecraft("stone").unwrap(),
            ),
            BlockStateProjection::new(
                BlockPos::new(4, 5, 6),
                ResourceId::minecraft("dirt").unwrap(),
            ),
        ];
        if reversed {
            blocks.reverse();
        }
        RegionStateProjection::new(
            region(),
            RegionMapping::V1,
            blocks,
            vec![
                EntityStateProjection::new(
                    StableEntityId::new(2).unwrap(),
                    ResourceId::minecraft("pig").unwrap(),
                    vec![7],
                )
                .unwrap(),
            ],
            vec![
                ExtensionStateProjection::new(
                    ResourceId::new("ferrite", "random/test").unwrap(),
                    vec![1],
                    vec![2],
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    #[test]
    fn semantic_projection_hash_is_sorted_and_locked() {
        let first = projection(false).hash(GameTick::new(9)).unwrap();
        let second = projection(true).hash(GameTick::new(9)).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first.to_string(),
            "1190d3fff85fab396901cd9f79ffd5d8ed3f103501f44e2805292889d5400417"
        );
    }

    #[test]
    fn projected_world_hash_is_locked() {
        let hash = hash_projected_world(
            WorldId::new(1).unwrap(),
            GameTick::new(9),
            StateHash::from_bytes([3; 32]),
            &[projection(false)],
        )
        .unwrap();
        assert_eq!(
            hash.to_string(),
            "40995a86f3bc08ccd137be0f85cceb2677f2041250a66197ee9097203a3ad000"
        );
    }

    #[test]
    fn duplicate_and_cross_region_records_fail_closed() {
        let duplicate = BlockStateProjection::new(
            BlockPos::new(0, 0, 0),
            ResourceId::minecraft("stone").unwrap(),
        );
        assert!(
            RegionStateProjection::new(
                region(),
                RegionMapping::V1,
                vec![duplicate.clone(), duplicate],
                vec![],
                vec![],
            )
            .is_err()
        );
        assert!(
            RegionStateProjection::new(
                region(),
                RegionMapping::V1,
                vec![BlockStateProjection::new(
                    BlockPos::new(128, 0, 0),
                    ResourceId::minecraft("stone").unwrap(),
                )],
                vec![],
                vec![],
            )
            .is_err()
        );
    }
}
