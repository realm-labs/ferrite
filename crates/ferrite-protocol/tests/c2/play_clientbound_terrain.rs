use std::collections::{BTreeMap, BTreeSet};

use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::terrain::batch::ChunkBatchCalculator;
use ferrite_protocol::java_26_2::play::clientbound::terrain::bundle::{
    BundleError, BundledPlayPackets, ClientboundBundleAssembler, MAX_BUNDLE_SUBPACKETS,
};
use ferrite_protocol::java_26_2::play::clientbound::terrain::packet::{
    BlockEntityData, ChunkBiomes, ChunkCoordinate, ChunkLightUpdate, FullChunk, HeightmapType,
    LightData, LightLayerUpdate, SectionData, TerrainPacket,
};
use ferrite_protocol::java_26_2::play::clientbound::terrain::projection::{
    LightSectionCoordinate, TerrainProjection,
};
use ferrite_protocol::java_26_2::play::clientbound::terrain::readiness::{
    LevelLoadState, LevelLoadTracker, PlayerChunkObservation,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{BIOME, DIMENSION_TYPE, PlayRegistries};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::{NbtQuota, NetworkNbt, TextComponentNbt};
use ferrite_protocol::java_26_2::wire::frame::MAX_FRAME_LENGTH;
use ferrite_protocol::java_26_2::wire::primitive::{WireReader, WireWriter};

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

fn registries_with_biomes(count: usize) -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(id(DIMENSION_TYPE), vec![id("minecraft:overworld")]);
    registries.insert(
        id(BIOME),
        (0..count)
            .map(|index| id(&format!("ferrite:test_biome_{index}")))
            .collect(),
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

fn unchanged_light() -> LightData {
    LightData {
        sky: vec![LightLayerUpdate::Unchanged; 4],
        block: vec![LightLayerUpdate::Unchanged; 4],
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

fn simple_chunk() -> FullChunk {
    FullChunk {
        position: ChunkCoordinate { x: 0, z: 0 },
        heightmaps: BTreeMap::new(),
        sections: vec![
            SectionData {
                non_empty_blocks: 0,
                fluid_count: 0,
                block_states: vec![0; 4_096],
                biomes: vec![0; 64],
            },
            SectionData {
                non_empty_blocks: 0,
                fluid_count: 0,
                block_states: vec![0; 4_096],
                biomes: vec![0; 64],
            },
        ],
        block_entities: Vec::new(),
        light: unchanged_light(),
    }
}

fn raw_light_packet(sky_data: &[u64], sky_empty: &[u64], sky_updates: &[Vec<u8>]) -> Vec<u8> {
    let mut writer = WireWriter::new(MAX_FRAME_LENGTH);
    writer.write_var_i32(48).unwrap();
    writer.write_var_i32(0).unwrap();
    writer.write_var_i32(0).unwrap();
    for words in [sky_data, &[], sky_empty, &[]] {
        writer
            .write_count("test bitset", words.len(), words.len())
            .unwrap();
        for word in words {
            writer.write_i64(*word as i64).unwrap();
        }
    }
    writer
        .write_count("test light arrays", sky_updates.len(), sky_updates.len())
        .unwrap();
    for update in sky_updates {
        writer.write_byte_array(update, 2_048).unwrap();
    }
    writer.write_var_i32(0).unwrap();
    writer.into_inner()
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
fn palette_boundaries_select_global_storage_and_validate_dynamic_registry_values() {
    let registries = registries_with_biomes(10);
    let mut chunk = simple_chunk();
    for (index, state) in chunk.sections[0].block_states.iter_mut().enumerate() {
        *state = (index % 257) as i32;
    }
    for (index, biome) in chunk.sections[0].biomes.iter_mut().enumerate() {
        *biome = (index % 9) as i32;
    }
    let packet = PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(chunk));
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );

    let mut reader = WireReader::new(&encoded);
    assert_eq!(reader.read_var_i32().unwrap(), 45);
    reader.read_i32().unwrap();
    reader.read_i32().unwrap();
    assert_eq!(reader.read_var_i32().unwrap(), 0);
    let section_blob = reader.read_byte_array(2_097_152).unwrap();
    let mut section_reader = WireReader::new(section_blob);
    section_reader.read_i16().unwrap();
    section_reader.read_i16().unwrap();
    assert_eq!(section_reader.read_i8().unwrap(), 15);

    let mut unknown_biome = simple_chunk();
    unknown_biome.sections[0].biomes[0] = 10;
    assert!(
        encode_packet(
            &PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(unknown_biome)),
            &registries
        )
        .is_err()
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
fn all_ten_terrain_packet_identities_have_locked_default_bytes() {
    let registries = registries();
    let vectors = [
        (
            PlayClientboundPacket::Terrain(TerrainPacket::BundleDelimiter),
            vec![0],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::ChunkBatchFinished(0)),
            vec![11, 0],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::ChunkBatchStart),
            vec![12],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::ChunksBiomes(Vec::new())),
            vec![13, 0],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::ForgetLevelChunk(ChunkCoordinate {
                x: 0,
                z: 0,
            })),
            vec![37, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::LightUpdate(ChunkLightUpdate {
                position: ChunkCoordinate { x: 0, z: 0 },
                light: unchanged_light(),
            })),
            vec![48, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::SetChunkCacheCenter(ChunkCoordinate {
                x: 0,
                z: 0,
            })),
            vec![94, 0, 0],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::SetChunkCacheRadius(0)),
            vec![95, 0],
        ),
        (
            PlayClientboundPacket::Terrain(TerrainPacket::SetSimulationDistance(0)),
            vec![111, 0],
        ),
    ];
    for (packet, expected) in vectors {
        assert_eq!(encode_packet(&packet, &registries).unwrap(), expected);
        assert_eq!(
            decode_packet(&expected, context(&registries)).unwrap(),
            packet
        );
    }
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(full_chunk())),
            &registries
        )
        .unwrap()[0],
        45
    );
}

#[test]
fn bundles_hold_order_allow_empty_and_reject_terminal_or_oversized_contents() {
    let delimiter = || PlayClientboundPacket::Terrain(TerrainPacket::BundleDelimiter);
    let payload =
        |value| PlayClientboundPacket::Terrain(TerrainPacket::SetSimulationDistance(value));
    let mut assembler = ClientboundBundleAssembler::new();
    assert_eq!(
        assembler.push(payload(1)).unwrap(),
        Some(BundledPlayPackets::Single(payload(1)))
    );
    assert_eq!(assembler.push(delimiter()).unwrap(), None);
    assert_eq!(
        assembler.push(delimiter()).unwrap(),
        Some(BundledPlayPackets::Bundle(Vec::new()))
    );
    assembler.push(delimiter()).unwrap();
    assembler.push(payload(2)).unwrap();
    assembler.push(payload(3)).unwrap();
    assert_eq!(
        assembler.push(delimiter()).unwrap(),
        Some(BundledPlayPackets::Bundle(vec![payload(2), payload(3)]))
    );

    assembler.push(delimiter()).unwrap();
    let disconnect = PlayClientboundPacket::Disconnect(TextComponentNbt::literal("bye").unwrap());
    assert_eq!(assembler.push(disconnect), Err(BundleError::TerminalPacket));

    let mut oversized = ClientboundBundleAssembler::new();
    oversized.push(delimiter()).unwrap();
    for index in 0..MAX_BUNDLE_SUBPACKETS {
        oversized.push(payload(index as i32)).unwrap();
    }
    assert_eq!(oversized.open_len(), Some(MAX_BUNDLE_SUBPACKETS));
    assert_eq!(
        oversized.push(payload(-1)),
        Err(BundleError::TooManySubpackets {
            maximum: MAX_BUNDLE_SUBPACKETS
        })
    );
}

#[test]
fn chunk_batch_estimator_matches_clamping_weighting_and_feedback_rules() {
    let mut calculator = ChunkBatchCalculator::new(100);
    assert_eq!(calculator.desired_chunks_per_tick(), 3.5);
    assert_eq!(calculator.on_batch_finished(0, 200), 3.5);
    assert_eq!(calculator.old_sample_weight(), 1);

    calculator.on_batch_start(1_000);
    let desired = calculator.on_batch_finished(2, 9_001_000);
    assert_eq!(calculator.aggregated_nanos_per_chunk(), 3_250_000.0);
    assert!((desired - (7_000_000.0 / 3_250_000.0) as f32).abs() < f32::EPSILON);
    assert_eq!(calculator.old_sample_weight(), 2);

    calculator.on_batch_start(10_000);
    calculator.on_batch_start(20_000);
    assert_eq!(calculator.batch_start_nanos(), 20_000);
    calculator.on_batch_finished(1, 19_000);
    assert_eq!(calculator.old_sample_weight(), 3);
    for index in 0..100 {
        calculator.on_batch_start(index);
        calculator.on_batch_finished(1, index + 2_000_000);
    }
    assert_eq!(calculator.old_sample_weight(), 49);
}

#[test]
fn readiness_waits_for_server_then_uses_strict_timeout_and_close_delay() {
    let mut tracker = LevelLoadTracker::new(0).unwrap();
    tracker.start_client_load(100);
    tracker.tick(40_000, PlayerChunkObservation::default());
    assert!(matches!(
        tracker.state(),
        Some(LevelLoadState::WaitingForServer { .. })
    ));
    tracker.loading_packets_received();
    tracker.tick(30_100, PlayerChunkObservation::default());
    assert!(matches!(
        tracker.state(),
        Some(LevelLoadState::WaitingForPlayerChunk { .. })
    ));
    tracker.tick(30_101, PlayerChunkObservation::default());
    assert_eq!(
        tracker.state(),
        Some(LevelLoadState::ClientLevelReady {
            ready_at_millis: 30_101
        })
    );
    assert!(tracker.take_player_loaded(30_101));
    assert_eq!(tracker.state(), None);

    let mut integrated = LevelLoadTracker::new(500).unwrap();
    integrated.start_client_load(0);
    integrated.loading_packets_received();
    integrated.player_section_compiled();
    integrated.tick(1, PlayerChunkObservation::default());
    assert!(!integrated.take_player_loaded(500));
    assert!(integrated.take_player_loaded(501));
}

#[test]
fn readiness_exemptions_do_not_depend_on_chunk_batch_finish() {
    for observation in [
        PlayerChunkObservation {
            player_outside_build_height: true,
            ..PlayerChunkObservation::default()
        },
        PlayerChunkObservation {
            camera_outside_build_height: true,
            ..PlayerChunkObservation::default()
        },
        PlayerChunkObservation {
            spectator: true,
            ..PlayerChunkObservation::default()
        },
        PlayerChunkObservation {
            alive: false,
            ..PlayerChunkObservation::default()
        },
    ] {
        let mut tracker = LevelLoadTracker::new(0).unwrap();
        tracker.start_client_load(0);
        tracker.loading_packets_received();
        tracker.tick(1, observation);
        assert!(tracker.take_player_loaded(1));
    }
    let mut tracker = LevelLoadTracker::new(0).unwrap();
    tracker.start_client_load(0);
    tracker.loading_packets_received();
    let mut projection = TerrainProjection::new();
    projection
        .apply(TerrainPacket::ChunkBatchFinished(10))
        .unwrap();
    tracker.tick(1, PlayerChunkObservation::default());
    assert!(!tracker.take_player_loaded(1));
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
fn full_chunk_accepts_negative_heightmap_count_and_ignores_section_blob_extras() {
    let registries = registries();
    let expected =
        PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(simple_chunk()));
    let encoded = encode_packet(&expected, &registries).unwrap();
    assert_eq!(encoded[9], 0);
    assert_eq!(encoded[10], 16);
    assert_eq!(
        &encoded[11..27],
        &[0; 16],
        "26.2 palettes use fixed-size storage and have no VarInt long-count prefix"
    );

    let mut negative_heightmaps = encoded.clone();
    negative_heightmaps.splice(9..10, [255, 255, 255, 255, 15]);
    assert_eq!(
        decode_packet(&negative_heightmaps, context(&registries)).unwrap(),
        expected
    );

    let mut extra_section_byte = encoded;
    extra_section_byte[10] += 1;
    extra_section_byte.insert(27, 0xaa);
    assert_eq!(
        decode_packet(&extra_section_byte, context(&registries)).unwrap(),
        expected
    );
}

#[test]
fn full_chunk_rejects_negative_entity_count_and_noncompound_update_tag() {
    let registries = registries();
    let packet = PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(simple_chunk()));
    let mut negative_entities = encode_packet(&packet, &registries).unwrap();
    assert_eq!(negative_entities[27], 0);
    negative_entities.splice(27..28, [255, 255, 255, 255, 15]);
    assert!(decode_packet(&negative_entities, context(&registries)).is_err());

    let mut invalid_tag = full_chunk();
    invalid_tag.block_entities[0].update_tag =
        Some(NetworkNbt::literal_component("not a compound").unwrap());
    assert!(
        encode_packet(
            &PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(invalid_tag)),
            &registries
        )
        .is_err()
    );
}

#[test]
fn light_masks_prefer_data_and_ignore_out_of_range_bits_and_surplus_arrays() {
    let registries = registries();
    let data = vec![0x5a; 2_048];
    let decoded = decode_packet(
        &raw_light_packet(&[1], &[1], std::slice::from_ref(&data)),
        context(&registries),
    )
    .unwrap();
    let PlayClientboundPacket::Terrain(TerrainPacket::LightUpdate(update)) = decoded else {
        panic!("expected light update");
    };
    assert_eq!(
        update.light.sky[0],
        LightLayerUpdate::Data(Box::new([0x5a; 2_048]))
    );

    let high_only = decode_packet(
        &raw_light_packet(&[0, 1 << 6], &[], &[]),
        context(&registries),
    )
    .unwrap();
    let PlayClientboundPacket::Terrain(TerrainPacket::LightUpdate(update)) = high_only else {
        panic!("expected light update");
    };
    assert_eq!(update.light.sky, unchanged_light().sky);

    assert!(
        decode_packet(
            &raw_light_packet(&[], &[], &[vec![1]]),
            context(&registries)
        )
        .is_ok()
    );
}

#[test]
fn in_range_light_data_requires_one_exact_2048_byte_array_per_set_bit() {
    let registries = registries();
    assert!(decode_packet(&raw_light_packet(&[1], &[], &[]), context(&registries)).is_err());
    assert!(
        decode_packet(
            &raw_light_packet(&[1], &[], &[vec![1]]),
            context(&registries)
        )
        .is_err()
    );
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
fn projection_applies_light_outside_cache_and_forget_removes_all_light_state() {
    let origin = ChunkCoordinate { x: 0, z: 0 };
    let outside = ChunkCoordinate { x: 100, z: -100 };
    let mut projection = TerrainProjection::new();
    projection
        .apply(TerrainPacket::SetChunkCacheRadius(2))
        .unwrap();
    projection
        .apply(TerrainPacket::SetChunkCacheCenter(origin))
        .unwrap();
    let mut chunk = simple_chunk();
    chunk.position = outside;
    chunk.light.sky[0] = LightLayerUpdate::Data(Box::new([7; 2_048]));
    projection
        .apply(TerrainPacket::LevelChunkWithLight(chunk))
        .unwrap();
    assert_eq!(projection.chunk_count(), 0);
    assert!(projection.lighting_enabled(outside));
    assert!(matches!(
        projection.light(outside).unwrap().sky[0],
        LightLayerUpdate::Data(_)
    ));

    let mut replacement = unchanged_light();
    replacement.sky[0] = LightLayerUpdate::Empty;
    replacement.sky[1] = LightLayerUpdate::Data(Box::new([9; 2_048]));
    projection
        .apply(TerrainPacket::LightUpdate(ChunkLightUpdate {
            position: outside,
            light: replacement,
        }))
        .unwrap();
    assert_eq!(
        projection.light(outside).unwrap().sky[0],
        LightLayerUpdate::Empty
    );
    assert!(matches!(
        projection.light(outside).unwrap().sky[1],
        LightLayerUpdate::Data(_)
    ));
    assert_eq!(projection.dirty_light_sections().len(), 36);
    assert!(
        projection
            .dirty_light_sections()
            .contains(&LightSectionCoordinate {
                x: outside.x,
                y: -2,
                z: outside.z
            })
    );

    projection
        .apply(TerrainPacket::ForgetLevelChunk(outside))
        .unwrap();
    assert!(projection.light(outside).is_none());
    assert!(!projection.lighting_enabled(outside));
}

#[test]
fn biome_refresh_notifies_absent_chunks_and_dirties_the_surrounding_three_by_three() {
    let position = ChunkCoordinate {
        x: i32::MAX,
        z: i32::MIN,
    };
    let mut projection = TerrainProjection::new();
    projection
        .apply(TerrainPacket::ChunksBiomes(vec![ChunkBiomes {
            position,
            sections: Vec::new(),
        }]))
        .unwrap();
    assert_eq!(projection.take_biome_notifications(), vec![position]);
    assert_eq!(projection.dirty_chunks().len(), 9);
    assert!(projection.dirty_chunks().contains(&ChunkCoordinate {
        x: i32::MIN,
        z: i32::MAX
    }));
}

#[test]
fn projection_tracks_heightmap_repair_and_exact_block_entity_type_matching() {
    let position = ChunkCoordinate { x: -2, z: 7 };
    let mut projection = TerrainProjection::with_capacity(64, 4).unwrap();
    projection.register_block_entity_type(1, 1).unwrap();
    projection
        .apply(TerrainPacket::SetChunkCacheRadius(2))
        .unwrap();
    projection
        .apply(TerrainPacket::SetChunkCacheCenter(position))
        .unwrap();
    projection
        .apply(TerrainPacket::LevelChunkWithLight(full_chunk()))
        .unwrap();
    assert_eq!(projection.chunk(position).unwrap().block_entities.len(), 1);
    assert_eq!(
        projection.recomputed_heightmaps(position).unwrap(),
        &BTreeSet::from([HeightmapType::WorldSurface, HeightmapType::MotionBlocking])
    );

    projection.register_block_entity_type(1, 2).unwrap();
    projection
        .apply(TerrainPacket::LevelChunkWithLight(full_chunk()))
        .unwrap();
    assert!(
        projection
            .chunk(position)
            .unwrap()
            .block_entities
            .is_empty()
    );
}

#[test]
fn cache_range_uses_java_int_subtraction_and_abs_overflow() {
    let mut projection = TerrainProjection::new();
    projection
        .apply(TerrainPacket::SetChunkCacheRadius(2))
        .unwrap();
    projection
        .apply(TerrainPacket::SetChunkCacheCenter(ChunkCoordinate {
            x: 0,
            z: 0,
        }))
        .unwrap();
    let mut overflow_delta = simple_chunk();
    overflow_delta.position = ChunkCoordinate { x: i32::MIN, z: 0 };
    projection
        .apply(TerrainPacket::LevelChunkWithLight(overflow_delta))
        .unwrap();
    assert!(
        projection
            .chunk(ChunkCoordinate { x: i32::MIN, z: 0 })
            .is_some()
    );
}
