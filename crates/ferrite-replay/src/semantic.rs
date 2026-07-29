use crate::codec::{CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError, Encoder};
use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::identity::{DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::random::RandomAlgorithm;
use ferrite_simulation::tick::{GameTick, TickPhase};

const MAX_RESOURCE_ID_BYTES: usize = 32 * 1024;

impl CanonicalEncode for ResourceId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_string(&self.to_string(), MAX_RESOURCE_ID_BYTES)
    }
}

impl CanonicalDecode for ResourceId {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        decoder
            .read_string(MAX_RESOURCE_ID_BYTES)?
            .parse()
            .map_err(|_| DecodeError::InvalidSemantic {
                kind: "resource identifier",
            })
    }
}

impl CanonicalEncode for WorldId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u128(self.get());
        Ok(())
    }
}

impl CanonicalDecode for WorldId {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        WorldId::new(decoder.read_u128()?).map_err(|_| DecodeError::InvalidSemantic {
            kind: "world identity",
        })
    }
}

impl CanonicalEncode for StableEntityId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u128(self.get());
        Ok(())
    }
}

impl CanonicalDecode for StableEntityId {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        StableEntityId::new(decoder.read_u128()?).map_err(|_| DecodeError::InvalidSemantic {
            kind: "entity identity",
        })
    }
}

impl CanonicalEncode for DimensionId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        self.resource().encode(encoder)
    }
}

impl CanonicalDecode for DimensionId {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self::new(ResourceId::decode(decoder)?))
    }
}

impl CanonicalEncode for RegionCoord {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_i32(self.x());
        encoder.write_i32(self.z());
        Ok(())
    }
}

impl CanonicalDecode for RegionCoord {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self::new(decoder.read_i32()?, decoder.read_i32()?))
    }
}

impl CanonicalEncode for RegionMappingVersion {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u16(self.get());
        Ok(())
    }
}

impl CanonicalDecode for RegionMappingVersion {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Self::new(decoder.read_u16()?).map_err(|_| DecodeError::InvalidSemantic {
            kind: "Region mapping version",
        })
    }
}

impl CanonicalEncode for SimulationRegionKey {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        self.world().encode(encoder)?;
        self.dimension().encode(encoder)?;
        self.coordinate().encode(encoder)?;
        self.mapping_version().encode(encoder)
    }
}

impl CanonicalDecode for SimulationRegionKey {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self::new(
            WorldId::decode(decoder)?,
            DimensionId::decode(decoder)?,
            RegionCoord::decode(decoder)?,
            RegionMappingVersion::decode(decoder)?,
        ))
    }
}

impl CanonicalEncode for BlockPos {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_i32(self.x);
        encoder.write_i32(self.y);
        encoder.write_i32(self.z);
        Ok(())
    }
}

impl CanonicalDecode for BlockPos {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self::new(
            decoder.read_i32()?,
            decoder.read_i32()?,
            decoder.read_i32()?,
        ))
    }
}

impl CanonicalEncode for ActivationGeneration {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u64(self.get());
        Ok(())
    }
}

impl CanonicalDecode for ActivationGeneration {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Self::new(decoder.read_u64()?).map_err(|_| DecodeError::InvalidSemantic {
            kind: "activation generation",
        })
    }
}

impl CanonicalEncode for GameTick {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u64(self.get());
        Ok(())
    }
}

impl CanonicalDecode for GameTick {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self::new(decoder.read_u64()?))
    }
}

impl CanonicalEncode for TickPhase {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(self.stable_tag());
        Ok(())
    }
}

impl CanonicalDecode for TickPhase {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let tag = decoder.read_u8()?;
        TickPhase::ALL
            .get(usize::from(tag))
            .copied()
            .ok_or(DecodeError::InvalidEnumTag {
                kind: "tick phase",
                tag: u64::from(tag),
            })
    }
}

impl CanonicalEncode for RandomAlgorithm {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u16(self.stable_tag());
        Ok(())
    }
}

impl CanonicalDecode for RandomAlgorithm {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        match decoder.read_u16()? {
            1 => Ok(Self::Xoshiro256StarStarV1),
            tag => Err(DecodeError::InvalidEnumTag {
                kind: "random algorithm",
                tag: u64::from(tag),
            }),
        }
    }
}
