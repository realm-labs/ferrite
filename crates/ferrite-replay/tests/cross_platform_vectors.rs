use ferrite_foundation::identity::{DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_replay::codec::{CanonicalEncode, EncodeError, Encoder};
use ferrite_replay::hash::hash_region;
use ferrite_simulation::random::DeterministicRng;

const VECTOR_SCHEMA: &str = "ferrite-cross-platform-vectors-v1";

struct LockedProjection;

impl CanonicalEncode for LockedProjection {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_bool(true);
        encoder.write_i32(-12_345);
        encoder.write_u128(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
        encoder.write_f32(-0.0)?;
        encoder.write_f64(12_345.25)?;
        encoder.write_var_u64(0x1234_5678);
        encoder.write_string("Ferrite/26.2/跨平台", 64)
    }
}

#[test]
fn canonical_rng_encoding_and_hash_vectors_match_on_every_supported_runner() {
    let region = SimulationRegionKey::new(
        WorldId::new(0x1020_3040_5060_7080_90a0_b0c0_d0e0_f000).unwrap(),
        DimensionId::new(ResourceId::minecraft("the_nether").unwrap()),
        RegionCoord::new(-17, 33),
        RegionMappingVersion::V1,
    );
    let region_hash = hash_region(&region, 0x0102_0304_0506_0708, &LockedProjection).unwrap();
    let mut random = DeterministicRng::from_seed(0);
    let mut aggregate = Encoder::new();
    aggregate.write_string(VECTOR_SCHEMA, 64).unwrap();
    for _ in 0..5 {
        aggregate.write_u64(random.next_u64());
    }
    aggregate.write_magic(region_hash.as_bytes());
    LockedProjection.encode(&mut aggregate).unwrap();
    let digest = blake3::hash(aggregate.as_slice()).to_hex().to_string();
    println!(
        "cross-platform deterministic vectors verified: schema={VECTOR_SCHEMA} bytes={} digest={digest}",
        aggregate.as_slice().len()
    );
    assert_eq!(
        digest,
        "11d18ab3881d50117cab7211fd9bd41355a4b7009843a908520e3ba6e4b4d1ba"
    );
}
