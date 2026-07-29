use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::terrain::packet::TerrainPacket;
use ferrite_protocol::semantic::{PlayAdmission, SessionId, SessionIdentity};
use ferrite_server_runtime::chunk::projection::{
    JavaTerrainRegistryMap, TerrainProjectionError, project_chunk, project_stream_events,
};
use ferrite_server_runtime::chunk::session::{
    ChunkSessionLimits, ClientChunkSession, ClientChunkSessionError,
};
use ferrite_server_runtime::chunk::stream::ChunkStreamEvent;
use ferrite_server_runtime::chunk::ticket::{ACCESSIBLE_LEVEL, ENTITY_TICKING_LEVEL};
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::terrain::MinimalTerrain;

fn admission() -> PlayAdmission {
    PlayAdmission {
        session: SessionId::new(7).unwrap(),
        identity: SessionIdentity {
            profile_id: 44,
            name: "FerriteUser".to_owned(),
        },
        player: StableEntityId::new(44).unwrap(),
        region: SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        ),
        spawn_chunk: ChunkPos::new(0, 0),
        requested_view_distance: 1,
        transferred: false,
    }
}

fn limits() -> ChunkSessionLimits {
    ChunkSessionLimits {
        maximum_tracked_chunks: 25,
        maximum_tickets: 26,
        maximum_chunks_per_batch: 3,
    }
}

fn terrain() -> MinimalTerrain {
    let layout = ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).unwrap(),
        BlockStateId::new(0),
        BiomeId::new(0),
    );
    MinimalTerrain::new(
        layout,
        BlockStateId::new(0),
        BlockStateId::new(1),
        BiomeId::new(0),
        63,
    )
    .unwrap()
}

fn registry_map() -> JavaTerrainRegistryMap {
    let mut map = JavaTerrainRegistryMap::new(8, BlockStateId::new(0)).unwrap();
    map.insert_block_state(BlockStateId::new(0), 0).unwrap();
    map.insert_block_state(BlockStateId::new(1), 1).unwrap();
    map.insert_biome(BiomeId::new(0), 0).unwrap();
    map
}

#[test]
fn join_clamps_view_installs_tickets_and_orders_initial_controls() {
    let session = ClientChunkSession::join(&admission(), 8, 10, limits()).unwrap();
    assert_eq!(
        session.initial_events(),
        [
            ChunkStreamEvent::SetCenter(ChunkPos::new(0, 0)),
            ChunkStreamEvent::SetViewDistance(2),
            ChunkStreamEvent::SetSimulationDistance(10),
        ]
    );
    assert_eq!(session.stream().interest().view().len(), 25);
    assert_eq!(session.tickets().len(), 26);
    assert_eq!(
        session
            .tickets()
            .effective_level(ChunkPos::new(2, 2))
            .unwrap()
            .get(),
        ACCESSIBLE_LEVEL
    );
    assert_eq!(
        session
            .tickets()
            .effective_level(ChunkPos::new(0, 0))
            .unwrap()
            .get(),
        ENTITY_TICKING_LEVEL.saturating_sub(10)
    );
}

#[test]
fn bounded_stream_sends_nearest_ready_chunks_and_waits_for_feedback() {
    let mut session = ClientChunkSession::join(&admission(), 8, 10, limits()).unwrap();
    for position in [
        ChunkPos::new(2, 0),
        ChunkPos::new(0, 1),
        ChunkPos::new(-1, 0),
        ChunkPos::new(0, 0),
    ] {
        session.mark_ready(position).unwrap();
    }
    let terrain = terrain();
    let prepared = session
        .prepare_next_batch(|position| terrain.snapshot(position).ok())
        .unwrap();
    let prepared = prepared.expect("ready snapshots produce a batch");
    let events = prepared.events().to_vec();
    session.commit_prepared_batch(prepared).unwrap();
    assert!(matches!(events.first(), Some(ChunkStreamEvent::BatchStart)));
    assert!(matches!(
        events.last(),
        Some(ChunkStreamEvent::BatchFinish { chunks: 3 })
    ));
    let sent = events
        .iter()
        .filter_map(|event| match event {
            ChunkStreamEvent::Chunk(snapshot) => Some(snapshot.position()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sent,
        [
            ChunkPos::new(0, 0),
            ChunkPos::new(-1, 0),
            ChunkPos::new(0, 1),
        ]
    );
    assert!(
        session
            .prepare_next_batch(|position| terrain.snapshot(position).ok())
            .unwrap()
            .is_none()
    );
    session.acknowledge_batch(f32::NAN).unwrap();
    assert_eq!(session.stream().desired_chunks_per_tick(), 0.01);
    session.acknowledge_batch(f32::INFINITY).unwrap();
    assert_eq!(session.stream().desired_chunks_per_tick(), 64.0);
    session.acknowledge_batch(f32::NEG_INFINITY).unwrap();
    assert_eq!(session.stream().desired_chunks_per_tick(), 0.01);
    assert_eq!(session.stream().unacknowledged_batches(), 0);
}

#[test]
fn recenter_sends_center_before_only_previously_sent_unloads() {
    let mut session = ClientChunkSession::join(&admission(), 8, 10, limits()).unwrap();
    session.mark_ready(ChunkPos::new(-2, 0)).unwrap();
    session.mark_ready(ChunkPos::new(-2, 1)).unwrap();
    let terrain = terrain();
    let prepared = session
        .prepare_next_batch(|position| {
            (position == ChunkPos::new(-2, 0)).then(|| terrain.snapshot(position).unwrap())
        })
        .unwrap()
        .unwrap();
    session.commit_prepared_batch(prepared).unwrap();
    let events = session.recenter(ChunkPos::new(2, 0), true).unwrap();
    assert_eq!(
        events,
        [
            ChunkStreamEvent::SetCenter(ChunkPos::new(2, 0)),
            ChunkStreamEvent::Forget(ChunkPos::new(-2, 0)),
        ]
    );
    assert_eq!(session.tickets().len(), 26);
}

#[test]
fn stream_events_project_to_versioned_packets_without_world_types_leaking() {
    let events = vec![
        ChunkStreamEvent::SetCenter(ChunkPos::new(-1, 2)),
        ChunkStreamEvent::SetViewDistance(2),
        ChunkStreamEvent::SetSimulationDistance(10),
        ChunkStreamEvent::BatchStart,
        ChunkStreamEvent::Chunk(terrain().snapshot(ChunkPos::new(-1, 2)).unwrap()),
        ChunkStreamEvent::BatchFinish { chunks: 1 },
    ];
    let packets = project_stream_events(events, &registry_map()).unwrap();
    assert!(matches!(
        &packets[0],
        PlayClientboundPacket::Terrain(TerrainPacket::SetChunkCacheCenter(position))
            if position.x == -1 && position.z == 2
    ));
    assert!(matches!(
        &packets[4],
        PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(chunk))
            if chunk.sections.len() == 24 && chunk.heightmaps.len() == 3
    ));
}

#[test]
fn terrain_registry_projection_is_bounded_idempotent_and_fail_closed() {
    let air = BlockStateId::new(0);
    let mut map = JavaTerrainRegistryMap::new(3, air).unwrap();
    map.insert_block_state(air, 0).unwrap();
    map.insert_block_state(air, 0).unwrap();
    assert_eq!(
        map.insert_block_state(air, 1).unwrap_err(),
        TerrainProjectionError::EntryRemap {
            current: 0,
            requested: 1,
        }
    );
    assert_eq!(
        map.insert_block_state(BlockStateId::new(1), 0).unwrap_err(),
        TerrainProjectionError::DuplicateRawId { raw_id: 0 }
    );
    assert_eq!(
        map.insert_block_state(BlockStateId::new(1), 32_366)
            .unwrap_err(),
        TerrainProjectionError::BlockStateRawId { raw_id: 32_366 }
    );

    map.insert_biome(BiomeId::new(0), 0).unwrap();
    assert_eq!(
        project_chunk(&terrain().snapshot(ChunkPos::new(0, 0)).unwrap(), &map).unwrap_err(),
        TerrainProjectionError::UnmappedBlockState(BlockStateId::new(1))
    );
}

#[test]
fn failed_or_stale_prepared_batches_do_not_mark_chunks_sent() {
    let mut session = ClientChunkSession::join(&admission(), 8, 10, limits()).unwrap();
    session.mark_ready(ChunkPos::new(0, 0)).unwrap();
    let terrain = terrain();
    let prepared = session
        .prepare_next_batch(|position| terrain.snapshot(position).ok())
        .unwrap()
        .unwrap();
    let mut incomplete_map = JavaTerrainRegistryMap::new(2, BlockStateId::new(0)).unwrap();
    incomplete_map
        .insert_block_state(BlockStateId::new(0), 0)
        .unwrap();
    incomplete_map.insert_biome(BiomeId::new(0), 0).unwrap();
    assert!(project_stream_events(prepared.events().to_vec(), &incomplete_map).is_err());

    let stale = session
        .prepare_next_batch(|position| terrain.snapshot(position).ok())
        .unwrap()
        .expect("projection failure leaves the chunk pending");
    session.mark_ready(ChunkPos::new(1, 0)).unwrap();
    assert!(matches!(
        session.commit_prepared_batch(stale),
        Err(ClientChunkSessionError::StalePreparedBatch { .. })
    ));
    assert!(
        session
            .prepare_next_batch(|position| terrain.snapshot(position).ok())
            .unwrap()
            .is_some()
    );
}
