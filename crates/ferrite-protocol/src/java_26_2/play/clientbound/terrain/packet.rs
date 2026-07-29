use std::collections::BTreeMap;

use crate::java_26_2::value::nbt::NetworkNbt;

pub const MAX_SECTION_BLOB_BYTES: usize = 2_097_152;
pub const LIGHT_LAYER_BYTES: usize = 2_048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainPacket {
    BundleDelimiter,
    ChunkBatchFinished(i32),
    ChunkBatchStart,
    ChunksBiomes(Vec<ChunkBiomes>),
    ForgetLevelChunk(ChunkCoordinate),
    LevelChunkWithLight(FullChunk),
    LightUpdate(ChunkLightUpdate),
    SetChunkCacheCenter(ChunkCoordinate),
    SetChunkCacheRadius(i32),
    SetSimulationDistance(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullChunk {
    pub position: ChunkCoordinate,
    pub heightmaps: BTreeMap<HeightmapType, Vec<i64>>,
    pub sections: Vec<SectionData>,
    pub block_entities: Vec<BlockEntityData>,
    pub light: LightData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeightmapType {
    WorldSurfaceWorldgen,
    WorldSurface,
    OceanFloorWorldgen,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

impl HeightmapType {
    #[must_use]
    pub const fn raw_id(self) -> i32 {
        match self {
            Self::WorldSurfaceWorldgen => 0,
            Self::WorldSurface => 1,
            Self::OceanFloorWorldgen => 2,
            Self::OceanFloor => 3,
            Self::MotionBlocking => 4,
            Self::MotionBlockingNoLeaves => 5,
        }
    }

    #[must_use]
    pub const fn from_raw_or_world_surface_worldgen(raw: i32) -> Self {
        match raw {
            1 => Self::WorldSurface,
            2 => Self::OceanFloorWorldgen,
            3 => Self::OceanFloor,
            4 => Self::MotionBlocking,
            5 => Self::MotionBlockingNoLeaves,
            _ => Self::WorldSurfaceWorldgen,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionData {
    pub non_empty_blocks: i16,
    pub fluid_count: i16,
    pub block_states: Vec<i32>,
    pub biomes: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityData {
    pub packed_local_xz: i8,
    pub y: i16,
    pub type_raw_id: i32,
    pub update_tag: Option<NetworkNbt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLightUpdate {
    pub position: ChunkCoordinate,
    pub light: LightData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightData {
    pub sky: Vec<LightLayerUpdate>,
    pub block: Vec<LightLayerUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightLayerUpdate {
    Unchanged,
    Empty,
    Data(Box<[u8; LIGHT_LAYER_BYTES]>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkBiomes {
    pub position: ChunkCoordinate,
    pub sections: Vec<Vec<i32>>,
}
