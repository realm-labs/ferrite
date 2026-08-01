//! Shared deterministic world-service conformance fixtures.

use std::fs;
use std::path::{Path, PathBuf};

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_registry::bundle::ContentBundle;
use ferrite_registry::digest::ContentDigest;
use ferrite_server_runtime::world_service::model::{
    ChunkActivity, GenerationOutcome, WorldServiceRuntimeConfig,
};
use ferrite_server_runtime::world_service::runtime::WorldServiceRegionRuntime;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::generation::status::ChunkStatus;
use ferrite_world::id::{BiomeId, BlockStateId};

pub const REGION_SIDE_CHUNKS: i32 = 8;

#[derive(serde::Deserialize)]
struct ContentBundleLock {
    content_manifest_digest: ContentDigest,
}

#[must_use]
pub fn bundle_available() -> bool {
    bundle_path().is_file()
}

pub fn bundle() -> ContentBundle {
    let path = bundle_path();
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "world-service conformance requires the generated 26.2 content bundle at {}: {error}",
            path.display()
        )
    });
    serde_json::from_slice(&bytes).expect("generated content bundle is schema-valid")
}

pub fn content_manifest() -> [u8; 32] {
    if bundle_available() {
        return *bundle()
            .content_manifest()
            .expect("fixture bundle has a valid content manifest")
            .digest()
            .as_bytes();
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/reference/minecraft-java-26.2/content-bundle.lock.toml");
    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "world-service conformance requires the committed content bundle lock at {}: {error}",
            path.display()
        )
    });
    let lock =
        toml::from_str::<ContentBundleLock>(&source).expect("content bundle lock is schema-valid");
    *lock.content_manifest_digest.as_bytes()
}

fn bundle_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ferrite-content/26.2/content-bundle.json")
}

pub fn dimension(path: &str) -> DimensionId {
    DimensionId::new(ResourceId::minecraft(path).expect("fixture dimension is valid"))
}

pub fn region(coordinate_x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).expect("fixture world is nonzero"),
        dimension("overworld"),
        RegionCoord::new(coordinate_x, 0),
        RegionMappingVersion::V1,
    )
}

pub fn owner_of(chunk: ChunkPos) -> SimulationRegionKey {
    region(chunk.x.div_euclid(REGION_SIDE_CHUNKS))
}

#[must_use]
pub fn layout() -> ChunkLayout {
    ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).expect("fixture layout is valid"),
        BlockStateId::new(0),
        BiomeId::new(1),
    )
}

pub fn config(manifest: [u8; 32], event_capacity: usize) -> WorldServiceRuntimeConfig {
    WorldServiceRuntimeConfig {
        mapping: RegionMapping::V1,
        layout: layout(),
        region_side_chunks: REGION_SIDE_CHUNKS as u16,
        chunk_capacity: 64,
        event_capacity,
        content_manifest: manifest,
    }
}

pub fn runtime(coordinate_x: i32, manifest: [u8; 32]) -> WorldServiceRegionRuntime {
    WorldServiceRegionRuntime::new(
        region(coordinate_x),
        ActivationGeneration::INITIAL,
        config(manifest, 4_096),
    )
    .expect("fixture world-service runtime is valid")
}

pub fn generate_full(runtime: &mut WorldServiceRegionRuntime, chunk: ChunkPos, seed: u64) {
    runtime
        .demand_chunk(chunk)
        .expect("fixture chunk demand is owned and bounded");
    for target in ChunkStatus::ALL.into_iter().skip(1) {
        let request = runtime
            .begin_generation(chunk, target)
            .expect("fixture generation advances adjacent statuses");
        let mut generated = request.source.clone();
        let salt = seed
            .wrapping_add(u64::from(target as u8).wrapping_mul(0x9e37_79b9))
            .wrapping_add(chunk.x as u64)
            .wrapping_add((chunk.z as u64).rotate_left(17));
        let local_x = (salt & 15) as i32;
        let local_z = ((salt >> 4) & 15) as i32;
        let y = -64 + ((salt >> 8) % 384) as i32;
        let state = BlockStateId::new(1 + (salt as u32 % 127));
        generated
            .set_block(
                BlockPos::new(chunk.x * 16 + local_x, y, chunk.z * 16 + local_z),
                state,
            )
            .expect("generated position belongs to its chunk");
        if target >= ChunkStatus::InitializeLight {
            ferrite_world::light::recompute_chunk_light(&mut generated)
                .expect("fixture light follows generated block authority");
        }
        assert!(matches!(
            runtime
                .apply_generated(request.complete(generated))
                .expect("fixture completion passes generation fences"),
            GenerationOutcome::Published { .. }
        ));
    }
    for activity in [
        ChunkActivity::Accessible,
        ChunkActivity::BlockTicking,
        ChunkActivity::EntityTicking,
    ] {
        runtime
            .promote(chunk, activity)
            .expect("fixture activity advances in source order");
    }
}
