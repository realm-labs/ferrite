//! Ferrite spatial Region mapping onto Lattice placement shards.

use crate::lattice::authority::{
    LatticeNodeIdentity, RegionAuthorityAdapter, RegionAuthorityError,
};
use ferrite_foundation::region::SimulationRegionKey;
use lattice_core::actor_ref::{
    ConfigFingerprint, EntityId, EntityType, PlacementDomainId, ProtocolId, ReferenceError,
};
use lattice_placement::mapping::{ShardMapper, ShardMappingError};
use lattice_placement::region::{EntityConfig, RegionError};
use lattice_placement::types::ShardId;
use thiserror::Error;
use xxhash_rust::xxh3::xxh3_64_with_seed;

pub const FERRITE_SPATIAL_MAPPER_ID: &str = "ferrite-spatial-region";
pub const FERRITE_SPATIAL_MAPPER_VERSION: u32 = 1;
pub const FERRITE_SPATIAL_ENCODING_VERSION: u16 = 1;
pub const FERRITE_SPATIAL_SEED: u64 = 0x4645_5252_4954_4531;
const ENTITY_TYPE: &str = "ferrite-region-cell";
const ALLOCATION_POLICY: &str = "weighted-least-load";
const ALLOCATION_POLICY_VERSION: u32 = 1;
const ENTITY_ID_MAGIC: &[u8; 4] = b"FSPR";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialPlacementConfig {
    domain: String,
    cell_span_regions: u16,
    shard_count: u32,
    protocol_id: u64,
}

impl SpatialPlacementConfig {
    pub fn new(
        domain: impl Into<String>,
        cell_span_regions: u16,
        shard_count: u32,
        protocol_id: u64,
    ) -> Result<Self, SpatialAdapterError> {
        let domain = domain.into();
        PlacementDomainId::new(domain.clone())?;
        if cell_span_regions == 0 {
            return Err(SpatialAdapterError::ZeroCellSpan);
        }
        if shard_count == 0 {
            return Err(SpatialAdapterError::ZeroShardCount);
        }
        ProtocolId::new(protocol_id)?;
        Ok(Self {
            domain,
            cell_span_regions,
            shard_count,
            protocol_id,
        })
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub const fn cell_span_regions(&self) -> u16 {
        self.cell_span_regions
    }

    pub const fn shard_count(&self) -> u32 {
        self.shard_count
    }

    pub const fn protocol_id(&self) -> u64 {
        self.protocol_id
    }
}

#[derive(Debug, Clone)]
pub struct SpatialPlacementAdapter {
    config: SpatialPlacementConfig,
    entity: EntityConfig,
    mapper: FerriteSpatialShardMapper,
}

impl SpatialPlacementAdapter {
    pub fn new(config: SpatialPlacementConfig) -> Result<Self, SpatialAdapterError> {
        let mapper = FerriteSpatialShardMapper;
        let entity = EntityConfig::new(
            PlacementDomainId::new(config.domain.clone())?,
            EntityType::new(ENTITY_TYPE)?,
            ProtocolId::new(config.protocol_id)?,
            config.shard_count,
            ALLOCATION_POLICY,
            ALLOCATION_POLICY_VERSION,
            Vec::new(),
        )?
        .with_shard_mapper(&mapper)?;
        entity.validate_mapper(&mapper)?;
        Ok(Self {
            config,
            entity,
            mapper,
        })
    }

    pub const fn config(&self) -> &SpatialPlacementConfig {
        &self.config
    }

    pub fn route(
        &self,
        region: &SimulationRegionKey,
    ) -> Result<SpatialPlacementRoute, SpatialAdapterError> {
        let cell_x = region
            .coordinate()
            .x()
            .div_euclid(i32::from(self.config.cell_span_regions));
        let cell_z = region
            .coordinate()
            .z()
            .div_euclid(i32::from(self.config.cell_span_regions));
        let entity_id = encode_cell_entity_id(region, cell_x, cell_z)?;
        let lattice_id = EntityId::new(entity_id.clone())?;
        let shard = self.entity.shard_for_with(&self.mapper, &lattice_id)?;
        Ok(SpatialPlacementRoute {
            domain: self.config.domain.clone(),
            cell_x,
            cell_z,
            shard: shard.get(),
            entity_id: entity_id.into_boxed_slice(),
        })
    }

    pub(crate) fn slot_descriptor(
        &self,
        route: &SpatialPlacementRoute,
    ) -> Result<LatticeSlotDescriptor, SpatialAdapterError> {
        if route.domain != self.config.domain || route.shard >= self.config.shard_count {
            return Err(SpatialAdapterError::RouteConfigMismatch);
        }
        Ok(LatticeSlotDescriptor {
            domain: self.entity.domain.clone(),
            entity_type: self.entity.entity_type.clone(),
            shard: ShardId::new(route.shard),
            fingerprint: self.entity.fingerprint(),
        })
    }

    pub fn authority_adapter(
        &self,
        route: &SpatialPlacementRoute,
        local: LatticeNodeIdentity,
        safety_margin_millis: u64,
    ) -> Result<RegionAuthorityAdapter, RegionAuthorityError> {
        let slot = self
            .slot_descriptor(route)
            .map_err(RegionAuthorityError::Spatial)?;
        RegionAuthorityAdapter::new(local, slot, safety_margin_millis)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialPlacementRoute {
    domain: String,
    cell_x: i32,
    cell_z: i32,
    shard: u32,
    entity_id: Box<[u8]>,
}

impl SpatialPlacementRoute {
    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub const fn cell_x(&self) -> i32 {
        self.cell_x
    }

    pub const fn cell_z(&self) -> i32 {
        self.cell_z
    }

    pub const fn shard(&self) -> u32 {
        self.shard
    }

    pub fn entity_id(&self) -> &[u8] {
        &self.entity_id
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LatticeSlotDescriptor {
    pub(crate) domain: PlacementDomainId,
    pub(crate) entity_type: EntityType,
    pub(crate) shard: ShardId,
    pub(crate) fingerprint: ConfigFingerprint,
}

#[derive(Debug, Clone, Copy)]
struct FerriteSpatialShardMapper;

impl ShardMapper for FerriteSpatialShardMapper {
    fn mapper_id(&self) -> &'static str {
        FERRITE_SPATIAL_MAPPER_ID
    }

    fn mapper_version(&self) -> u32 {
        FERRITE_SPATIAL_MAPPER_VERSION
    }

    fn shard_for(
        &self,
        entity_id: &EntityId,
        shard_count: u32,
    ) -> Result<ShardId, ShardMappingError> {
        if shard_count == 0 {
            return Err(ShardMappingError::InvalidShardCount);
        }
        Ok(ShardId::new(
            (xxh3_64_with_seed(entity_id.as_bytes(), FERRITE_SPATIAL_SEED) % u64::from(shard_count))
                as u32,
        ))
    }
}

fn encode_cell_entity_id(
    region: &SimulationRegionKey,
    cell_x: i32,
    cell_z: i32,
) -> Result<Vec<u8>, SpatialAdapterError> {
    let dimension = region.dimension().resource().to_string();
    let dimension_length =
        u16::try_from(dimension.len()).map_err(|_| SpatialAdapterError::EntityIdTooLarge)?;
    let mut bytes = Vec::with_capacity(34 + dimension.len());
    bytes.extend_from_slice(ENTITY_ID_MAGIC);
    bytes.extend_from_slice(&FERRITE_SPATIAL_ENCODING_VERSION.to_le_bytes());
    bytes.extend_from_slice(&region.world().get().to_le_bytes());
    bytes.extend_from_slice(&dimension_length.to_le_bytes());
    bytes.extend_from_slice(dimension.as_bytes());
    bytes.extend_from_slice(&cell_x.to_le_bytes());
    bytes.extend_from_slice(&cell_z.to_le_bytes());
    bytes.extend_from_slice(&region.mapping_version().get().to_le_bytes());
    EntityId::new(bytes.clone()).map_err(|_| SpatialAdapterError::EntityIdTooLarge)?;
    Ok(bytes)
}

#[derive(Debug, Error)]
pub enum SpatialAdapterError {
    #[error("placement-cell span cannot be zero")]
    ZeroCellSpan,
    #[error("placement shard count cannot be zero")]
    ZeroShardCount,
    #[error("canonical placement entity ID exceeds the Lattice bound")]
    EntityIdTooLarge,
    #[error("spatial route was produced by another placement configuration")]
    RouteConfigMismatch,
    #[error(transparent)]
    Reference(#[from] ReferenceError),
    #[error(transparent)]
    Region(#[from] RegionError),
    #[error(transparent)]
    Mapping(#[from] ShardMappingError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};
    use ferrite_foundation::resource::ResourceId;

    fn region(x: i32, z: i32) -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(x, z),
            RegionMappingVersion::V1,
        )
    }

    #[test]
    fn spatial_cells_use_euclidean_mapping_and_locked_shards() {
        let adapter =
            SpatialPlacementAdapter::new(SpatialPlacementConfig::new("world-1", 4, 64, 1).unwrap())
                .unwrap();
        let negative = adapter.route(&region(-1, -5)).unwrap();
        assert_eq!((negative.cell_x(), negative.cell_z()), (-1, -2));
        assert_eq!(negative.shard(), 48);
        let same_cell = adapter.route(&region(-4, -8)).unwrap();
        assert_eq!(negative.entity_id(), same_cell.entity_id());
        assert_eq!(negative.shard(), same_cell.shard());
    }

    #[test]
    fn mapper_identity_is_persisted_in_the_lattice_config() {
        let adapter =
            SpatialPlacementAdapter::new(SpatialPlacementConfig::new("world-1", 4, 64, 1).unwrap())
                .unwrap();
        assert_eq!(adapter.entity.shard_mapper_id, FERRITE_SPATIAL_MAPPER_ID);
        assert_eq!(
            adapter.entity.shard_mapper_version,
            FERRITE_SPATIAL_MAPPER_VERSION
        );
    }
}
