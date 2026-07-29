use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, GameMode, PlayClientboundPacket,
};
use ferrite_protocol::java_26_2::play::clientbound::session::Respawn;
use ferrite_protocol::java_26_2::play::clientbound::terrain::packet::{
    BlockEntityData, ChunkBiomes, ChunkCoordinate, FullChunk, HeightmapType, LightData,
    LightLayerUpdate, SectionData, TerrainPacket,
};
use ferrite_protocol::java_26_2::play::clientbound::terrain::projection::TerrainProjection;
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{BIOME, DIMENSION_TYPE, PlayRegistries};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::{NbtQuota, NetworkNbt};

static REJECT_COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(id(DIMENSION_TYPE), vec![id("minecraft:overworld")]);
    registries.insert(
        id(BIOME),
        vec![id("minecraft:plains"), id("minecraft:desert")],
    );
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &REJECT_COMPONENTS,
        dimension_section_count: 2,
    }
}

fn light() -> LightData {
    LightData {
        sky: vec![
            LightLayerUpdate::Data(Box::new([0xff; 2_048])),
            LightLayerUpdate::Empty,
            LightLayerUpdate::Unchanged,
            LightLayerUpdate::Data(Box::new([0x11; 2_048])),
        ],
        block: vec![
            LightLayerUpdate::Empty,
            LightLayerUpdate::Unchanged,
            LightLayerUpdate::Empty,
            LightLayerUpdate::Unchanged,
        ],
    }
}

fn full_chunk() -> FullChunk {
    let mut mixed_blocks = vec![0; 4_096];
    for (index, state) in mixed_blocks.iter_mut().enumerate() {
        *state = (index % 17 != 0) as i32;
    }
    FullChunk {
        position: ChunkCoordinate { x: -2, z: 7 },
        heightmaps: BTreeMap::from([
            (HeightmapType::WorldSurface, vec![1, -2]),
            (HeightmapType::MotionBlocking, vec![3, -4]),
        ]),
        sections: vec![
            SectionData {
                non_empty_blocks: 4_096,
                fluid_count: 0,
                block_states: vec![1; 4_096],
                biomes: vec![0; 64],
            },
            SectionData {
                non_empty_blocks: 3_855,
                fluid_count: 0,
                block_states: mixed_blocks,
                biomes: (0..64).map(|index| index % 2).collect(),
            },
        ],
        block_entities: vec![BlockEntityData {
            packed_local_xz: 0x2f,
            y: 70,
            type_raw_id: 1,
            update_tag: Some(NetworkNbt::from_bytes(vec![10, 0], NbtQuota::Default).unwrap()),
        }],
        light: light(),
    }
}

#[test]
fn full_chunk_round_trip_covers_palettes_heightmaps_entities_and_light() {
    let registries = registries();
    let packet = PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(full_chunk()));
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(encoded[0], 45);
    assert!(
        encoded
            .windows(6)
            .any(|bytes| bytes == [0x2f, 0, 70, 1, 10, 0])
    );
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );

    let mut null_tag_chunk = full_chunk();
    null_tag_chunk.block_entities[0].update_tag = None;
    let null_tag_packet =
        PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(null_tag_chunk));
    let null_tag_bytes = encode_packet(&null_tag_packet, &registries).unwrap();
    assert!(
        null_tag_bytes
            .windows(5)
            .any(|bytes| bytes == [0x2f, 0, 70, 1, 0])
    );
    assert_eq!(
        decode_packet(&null_tag_bytes, context(&registries)).unwrap(),
        null_tag_packet
    );
}

#[test]
fn cache_controls_and_batch_markers_have_locked_bytes() {
    let registries = registries();
    let vectors = [
        (
            PlayClientboundPacket::Terrain(TerrainPacket::ChunkBatchStart),
            vec![12],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::ChunkBatchFinished(9)),
            vec![11, 9],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::SetChunkCacheCenter(ChunkCoordinate {
                x: 2,
                z: -1,
            })),
            vec![94, 2, 255, 255, 255, 255, 15],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::ForgetLevelChunk(ChunkCoordinate {
                x: -1,
                z: 2,
            })),
            vec![37, 0, 0, 0, 2, 255, 255, 255, 255],
        ),
    ];
    for (packet, expected) in vectors {
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(encoded, expected);
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
}

#[test]
fn malformed_terrain_fails_closed_and_blob_trailing_bytes_are_isolated() {
    let registries = registries();
    let packet = PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(full_chunk()));
    let mut truncated = encode_packet(&packet, &registries).unwrap();
    truncated.pop();
    assert!(decode_packet(&truncated, context(&registries)).is_err());

    let wrong_context = PlayDecodeContext {
        dimension_section_count: 3,
        ..context(&registries)
    };
    assert!(decode_packet(&encode_packet(&packet, &registries).unwrap(), wrong_context,).is_err());
}

#[test]
fn biome_refresh_and_unload_replace_only_present_chunks() {
    let position = ChunkCoordinate { x: -2, z: 7 };
    let mut projection = TerrainProjection::new();
    projection
        .apply(TerrainPacket::SetChunkCacheRadius(2))
        .unwrap();
    projection
        .apply(TerrainPacket::SetChunkCacheCenter(position))
        .unwrap();
    projection
        .apply(TerrainPacket::LevelChunkWithLight(full_chunk()))
        .unwrap();
    projection
        .apply(TerrainPacket::ChunksBiomes(vec![
            ChunkBiomes {
                position,
                sections: vec![vec![1; 64], vec![0; 64]],
            },
            ChunkBiomes {
                position: ChunkCoordinate { x: 50, z: 50 },
                sections: vec![vec![1; 64], vec![1; 64]],
            },
        ]))
        .unwrap();
    assert_eq!(projection.chunk(position).unwrap().sections[0].biomes[0], 1);
    projection
        .apply(TerrainPacket::ForgetLevelChunk(position))
        .unwrap();
    assert_eq!(projection.chunk_count(), 0);
}

#[test]
fn cache_projection_clamps_radius_and_rejects_out_of_range_full_chunks() {
    let position = ChunkCoordinate { x: -2, z: 7 };
    let mut projection = TerrainProjection::new();
    projection
        .apply(TerrainPacket::SetChunkCacheRadius(-20))
        .unwrap();
    projection
        .apply(TerrainPacket::SetChunkCacheCenter(position))
        .unwrap();
    assert_eq!(projection.cache_radius(), 5);

    projection
        .apply(TerrainPacket::LevelChunkWithLight(full_chunk()))
        .unwrap();
    assert_eq!(projection.chunk_count(), 1);

    projection
        .apply(TerrainPacket::SetChunkCacheCenter(ChunkCoordinate {
            x: i32::MAX,
            z: i32::MIN,
        }))
        .unwrap();
    assert_eq!(projection.chunk_count(), 0);

    projection
        .apply(TerrainPacket::LevelChunkWithLight(full_chunk()))
        .unwrap();
    assert_eq!(projection.chunk_count(), 0);

    projection
        .apply(TerrainPacket::SetChunkCacheRadius(i32::MAX))
        .unwrap();
    assert_eq!(projection.cache_radius(), i32::MIN + 2);
}

#[test]
fn respawn_codec_preserves_independent_keep_bits_and_ignores_high_bits() {
    let registries = registries();
    let spawn = CommonSpawnInfo {
        dimension_type: id("minecraft:overworld"),
        dimension: id("minecraft:overworld"),
        obfuscated_seed: -7,
        game_mode: GameMode::Survival,
        previous_game_mode: Some(GameMode::Creative),
        is_debug: false,
        is_flat: true,
        last_death: None,
        portal_cooldown: 20,
        sea_level: 63,
    };
    for (mask, attributes, entity_data) in [
        (0, false, false),
        (1, true, false),
        (2, false, true),
        (3, true, true),
        (-1, true, true),
    ] {
        let respawn = Respawn {
            spawn: spawn.clone(),
            data_to_keep: mask,
        };
        assert_eq!(respawn.retention().attributes, attributes);
        assert_eq!(respawn.retention().entity_data, entity_data);
        let packet = PlayClientboundPacket::Respawn(respawn);
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(encoded[0], 82);
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
}
