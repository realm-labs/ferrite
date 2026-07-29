//! Versioned deterministic mapping from chunks to simulation Regions.

use crate::bounds::{BoundsError, ChunkBounds};
use crate::coordinate::ChunkPos;
use crate::identity::{DimensionId, WorldId};
use crate::numeric::{NumericError, add_i32, floor_div_i32, multiply_i32};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU16;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegionMappingVersion(NonZeroU16);

impl RegionMappingVersion {
    pub const V1: Self = Self(NonZeroU16::MIN);

    pub const fn new(value: u16) -> Result<Self, RegionIdentityError> {
        match NonZeroU16::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(RegionIdentityError::ZeroMappingVersion),
        }
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegionSize(NonZeroU16);

impl RegionSize {
    pub const DEFAULT: Self = Self(NonZeroU16::new(8).expect("8 is nonzero"));

    pub const fn new(side_chunks: u16) -> Result<Self, RegionIdentityError> {
        match NonZeroU16::new(side_chunks) {
            Some(value) => Ok(Self(value)),
            None => Err(RegionIdentityError::ZeroRegionSize),
        }
    }

    pub const fn side_chunks(self) -> u16 {
        self.0.get()
    }

    const fn nonzero(self) -> NonZeroU16 {
        self.0
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RegionCoord {
    x: i32,
    z: i32,
}

impl RegionCoord {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub const fn x(self) -> i32 {
        self.x
    }

    pub const fn z(self) -> i32 {
        self.z
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SimulationRegionKey {
    world: WorldId,
    dimension: DimensionId,
    coordinate: RegionCoord,
    mapping_version: RegionMappingVersion,
}

impl SimulationRegionKey {
    pub const fn new(
        world: WorldId,
        dimension: DimensionId,
        coordinate: RegionCoord,
        mapping_version: RegionMappingVersion,
    ) -> Self {
        Self {
            world,
            dimension,
            coordinate,
            mapping_version,
        }
    }

    pub const fn world(&self) -> WorldId {
        self.world
    }

    pub const fn dimension(&self) -> &DimensionId {
        &self.dimension
    }

    pub const fn coordinate(&self) -> RegionCoord {
        self.coordinate
    }

    pub const fn mapping_version(&self) -> RegionMappingVersion {
        self.mapping_version
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionMapping {
    version: RegionMappingVersion,
    region_size: RegionSize,
}

impl RegionMapping {
    pub const V1: Self = Self {
        version: RegionMappingVersion::V1,
        region_size: RegionSize::DEFAULT,
    };

    pub const fn new(version: RegionMappingVersion, region_size: RegionSize) -> Self {
        Self {
            version,
            region_size,
        }
    }

    pub const fn version(self) -> RegionMappingVersion {
        self.version
    }

    pub const fn region_size(self) -> RegionSize {
        self.region_size
    }

    pub fn region_for_chunk(
        self,
        world: WorldId,
        dimension: DimensionId,
        chunk: ChunkPos,
    ) -> SimulationRegionKey {
        let coordinate = RegionCoord::new(
            floor_div_i32(chunk.x, self.region_size.nonzero()),
            floor_div_i32(chunk.z, self.region_size.nonzero()),
        );
        SimulationRegionKey::new(world, dimension, coordinate, self.version)
    }

    pub fn chunk_bounds(self, coordinate: RegionCoord) -> Result<ChunkBounds, RegionMappingError> {
        let side = i32::from(self.region_size.side_chunks());
        let minimum_x = multiply_i32(coordinate.x, side)?;
        let minimum_z = multiply_i32(coordinate.z, side)?;
        let maximum_x = add_i32(minimum_x, side)?;
        let maximum_z = add_i32(minimum_z, side)?;
        Ok(ChunkBounds::new(
            ChunkPos::new(minimum_x, minimum_z),
            ChunkPos::new(maximum_x, maximum_z),
        )?)
    }
}

impl Default for RegionMapping {
    fn default() -> Self {
        Self::V1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RegionIdentityError {
    #[error("Region mapping version cannot be zero")]
    ZeroMappingVersion,
    #[error("Region side length cannot be zero")]
    ZeroRegionSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RegionMappingError {
    #[error(transparent)]
    Numeric(#[from] NumericError),
    #[error(transparent)]
    Bounds(#[from] BoundsError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::ResourceId;

    fn overworld() -> DimensionId {
        DimensionId::new(ResourceId::minecraft("overworld").unwrap())
    }

    #[test]
    fn v1_maps_negative_chunks_with_euclidean_division() {
        let world = WorldId::new(1).unwrap();
        let mapping = RegionMapping::V1;
        for (chunk_x, expected_region_x) in [(-9, -2), (-8, -1), (-1, -1), (0, 0), (7, 0), (8, 1)] {
            let key = mapping.region_for_chunk(world, overworld(), ChunkPos::new(chunk_x, 0));
            assert_eq!(key.coordinate(), RegionCoord::new(expected_region_x, 0));
        }
    }

    #[test]
    fn region_bounds_are_half_open_and_round_trip_chunks() {
        let mapping = RegionMapping::V1;
        let coordinate = RegionCoord::new(-2, 3);
        let bounds = mapping.chunk_bounds(coordinate).unwrap();
        assert_eq!(bounds.minimum(), ChunkPos::new(-16, 24));
        assert_eq!(bounds.maximum_exclusive(), ChunkPos::new(-8, 32));

        let key =
            mapping.region_for_chunk(WorldId::new(1).unwrap(), overworld(), ChunkPos::new(-9, 31));
        assert_eq!(key.coordinate(), coordinate);
    }

    #[test]
    fn world_and_dimension_are_part_of_region_identity() {
        let mapping = RegionMapping::V1;
        let chunk = ChunkPos::new(0, 0);
        let first = mapping.region_for_chunk(WorldId::new(1).unwrap(), overworld(), chunk);
        let second = mapping.region_for_chunk(
            WorldId::new(2).unwrap(),
            DimensionId::new(ResourceId::minecraft("the_nether").unwrap()),
            chunk,
        );
        assert_ne!(first, second);
        assert_eq!(first.coordinate(), second.coordinate());
    }

    #[test]
    fn arbitrary_region_bounds_cannot_overflow() {
        assert!(
            RegionMapping::V1
                .chunk_bounds(RegionCoord::new(i32::MAX, 0))
                .is_err()
        );
        assert!(RegionSize::new(0).is_err());
        assert!(RegionMappingVersion::new(0).is_err());
    }
}
