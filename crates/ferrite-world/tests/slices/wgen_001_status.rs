use ferrite_world::generation::status::{
    ChunkKind, ChunkStatus, GenerationHeightmap, TaskExecution,
};

#[test]
fn twelve_statuses_lock_order_chunk_kind_heightmaps_and_execution() {
    assert_eq!(ChunkStatus::ALL.len(), 12);
    for (index, status) in ChunkStatus::ALL.into_iter().enumerate() {
        assert_eq!(status as usize, index);
        assert_eq!(
            status.chunk_kind(),
            if status == ChunkStatus::Full {
                ChunkKind::Level
            } else {
                ChunkKind::Proto
            }
        );
        if status <= ChunkStatus::Surface {
            assert_eq!(
                status.heightmaps(),
                [
                    GenerationHeightmap::OceanFloorWorldGeneration,
                    GenerationHeightmap::WorldSurfaceWorldGeneration,
                ]
            );
        } else {
            assert_eq!(status.heightmaps().len(), 4);
        }
    }
    assert_eq!(ChunkStatus::Biomes.execution(), TaskExecution::Asynchronous);
    assert_eq!(ChunkStatus::Noise.execution(), TaskExecution::Asynchronous);
    assert_eq!(
        ChunkStatus::InitializeLight.execution(),
        TaskExecution::Asynchronous
    );
    assert_eq!(ChunkStatus::Light.execution(), TaskExecution::Asynchronous);
    assert_eq!(ChunkStatus::Full.execution(), TaskExecution::MainThread);
}
