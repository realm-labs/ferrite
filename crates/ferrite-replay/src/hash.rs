//! Canonical Region and world state hashes.

use crate::codec::{CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError, Encoder};
use ferrite_foundation::identity::WorldId;
use ferrite_foundation::region::SimulationRegionKey;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

const REGION_HASH_DOMAIN: &[u8] = b"ferrite:region-state:v1\0";
const WORLD_HASH_DOMAIN: &[u8] = b"ferrite:world-state:v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateHash([u8; 32]);

impl StateHash {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for StateHash {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl CanonicalEncode for StateHash {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_magic(&self.0);
        Ok(())
    }
}

impl CanonicalDecode for StateHash {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self(decoder.read_fixed()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionHashRecord {
    region: SimulationRegionKey,
    hash: StateHash,
}

impl RegionHashRecord {
    pub const fn new(region: SimulationRegionKey, hash: StateHash) -> Self {
        Self { region, hash }
    }

    pub const fn region(&self) -> &SimulationRegionKey {
        &self.region
    }

    pub const fn hash(&self) -> StateHash {
        self.hash
    }
}

impl CanonicalEncode for RegionHashRecord {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        self.region.encode(encoder)?;
        self.hash.encode(encoder)
    }
}

impl CanonicalDecode for RegionHashRecord {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self::new(
            SimulationRegionKey::decode(decoder)?,
            StateHash::decode(decoder)?,
        ))
    }
}

pub fn hash_region<T: CanonicalEncode>(
    region: &SimulationRegionKey,
    committed_tick: u64,
    projection: &T,
) -> Result<StateHash, StateHashError> {
    let mut encoder = Encoder::new();
    encoder.write_magic(REGION_HASH_DOMAIN);
    region.encode(&mut encoder)?;
    encoder.write_u64(committed_tick);
    projection.encode(&mut encoder)?;
    Ok(blake3_hash(encoder.as_slice()))
}

pub fn hash_world(
    world: WorldId,
    committed_tick: u64,
    content_manifest: StateHash,
    regions: impl IntoIterator<Item = RegionHashRecord>,
) -> Result<StateHash, StateHashError> {
    let mut regions = regions
        .into_iter()
        .map(|record| {
            if record.region.world() != world {
                return Err(StateHashError::WrongWorld {
                    expected: world,
                    actual: record.region.world(),
                });
            }
            let mut key = Encoder::new();
            record.region.encode(&mut key)?;
            Ok((key.into_bytes(), record))
        })
        .collect::<Result<Vec<_>, StateHashError>>()?;
    regions.sort_by(|left, right| left.0.cmp(&right.0));
    if regions.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(StateHashError::DuplicateRegion);
    }

    let mut encoder = Encoder::new();
    encoder.write_magic(WORLD_HASH_DOMAIN);
    world.encode(&mut encoder)?;
    encoder.write_u64(committed_tick);
    content_manifest.encode(&mut encoder)?;
    encoder.write_var_u64(regions.len() as u64);
    for (_, region) in regions {
        region.encode(&mut encoder)?;
    }
    Ok(blake3_hash(encoder.as_slice()))
}

fn blake3_hash(bytes: &[u8]) -> StateHash {
    StateHash::from_bytes(*blake3::hash(bytes).as_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StateHashError {
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error("Region hash input repeats a Region key")]
    DuplicateRegion,
    #[error("Region belongs to world {actual}, expected {expected}")]
    WrongWorld { expected: WorldId, actual: WorldId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::DimensionId;
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};
    use ferrite_foundation::resource::ResourceId;

    struct Projection(u64);

    impl CanonicalEncode for Projection {
        fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
            encoder.write_u64(self.0);
            Ok(())
        }
    }

    fn region(world: WorldId, x: i32) -> SimulationRegionKey {
        SimulationRegionKey::new(
            world,
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(x, 0),
            RegionMappingVersion::V1,
        )
    }

    #[test]
    fn world_hash_sorts_regions_and_rejects_duplicates() {
        let world = WorldId::new(1).unwrap();
        let left = RegionHashRecord::new(region(world, -1), StateHash::from_bytes([1; 32]));
        let right = RegionHashRecord::new(region(world, 1), StateHash::from_bytes([2; 32]));
        let manifest = StateHash::from_bytes([3; 32]);
        let first = hash_world(world, 7, manifest, [left.clone(), right.clone()]).unwrap();
        assert_eq!(
            first.to_string(),
            "ce6684c7f8d13e037e2e5e558e49f87c7cb7d3bee57e6bcb8ebfcf27e9a54022"
        );
        let second = hash_world(world, 7, manifest, [right, left.clone()]).unwrap();
        assert_eq!(first, second);
        assert!(hash_world(world, 7, manifest, [left.clone(), left]).is_err());
    }

    #[test]
    fn region_hash_changes_with_tick_key_and_projection() {
        let world = WorldId::new(1).unwrap();
        let first = hash_region(&region(world, 0), 1, &Projection(9)).unwrap();
        assert_eq!(
            first.to_string(),
            "a409c54a43a1f415869c58546cccb1487b86c41817f01ee2a3b274f134fe541a"
        );
        assert_ne!(
            first,
            hash_region(&region(world, 0), 2, &Projection(9)).unwrap()
        );
        assert_ne!(
            first,
            hash_region(&region(world, 1), 1, &Projection(9)).unwrap()
        );
        assert_ne!(
            first,
            hash_region(&region(world, 0), 1, &Projection(10)).unwrap()
        );
    }
}
