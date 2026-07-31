//! Persisted chunk-generation milestones and their direct dependencies.

use ferrite_foundation::coordinate::ChunkPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ChunkStatus {
    Empty = 0,
    StructureStarts = 1,
    StructureReferences = 2,
    Biomes = 3,
    Noise = 4,
    Surface = 5,
    Carvers = 6,
    Features = 7,
    InitializeLight = 8,
    Light = 9,
    Spawn = 10,
    Full = 11,
}

impl ChunkStatus {
    pub const ALL: [Self; 12] = [
        Self::Empty,
        Self::StructureStarts,
        Self::StructureReferences,
        Self::Biomes,
        Self::Noise,
        Self::Surface,
        Self::Carvers,
        Self::Features,
        Self::InitializeLight,
        Self::Light,
        Self::Spawn,
        Self::Full,
    ];

    #[must_use]
    pub const fn chunk_kind(self) -> ChunkKind {
        match self {
            Self::Full => ChunkKind::Level,
            _ => ChunkKind::Proto,
        }
    }

    #[must_use]
    pub const fn heightmaps(self) -> &'static [GenerationHeightmap] {
        const WORLD_GENERATION: [GenerationHeightmap; 2] = [
            GenerationHeightmap::OceanFloorWorldGeneration,
            GenerationHeightmap::WorldSurfaceWorldGeneration,
        ];
        const FINAL: [GenerationHeightmap; 4] = [
            GenerationHeightmap::OceanFloor,
            GenerationHeightmap::WorldSurface,
            GenerationHeightmap::MotionBlocking,
            GenerationHeightmap::MotionBlockingNoLeaves,
        ];
        match self {
            Self::Empty
            | Self::StructureStarts
            | Self::StructureReferences
            | Self::Biomes
            | Self::Noise
            | Self::Surface => &WORLD_GENERATION,
            _ => &FINAL,
        }
    }

    #[must_use]
    pub const fn write_radius(self) -> Option<u8> {
        match self {
            Self::Noise | Self::Surface | Self::Carvers => Some(0),
            Self::Features => Some(1),
            _ => None,
        }
    }

    #[must_use]
    pub const fn direct_requirement(self, radius: u8) -> Option<Self> {
        match (self, radius) {
            (Self::Empty, _) => None,
            (Self::StructureStarts, 0) => Some(Self::Empty),
            (Self::StructureReferences, 0..=8) => Some(Self::StructureStarts),
            (Self::Biomes, 0) => Some(Self::StructureReferences),
            (Self::Biomes, 1..=8) => Some(Self::StructureStarts),
            (Self::Noise, 0..=1) => Some(Self::Biomes),
            (Self::Noise, 2..=8) => Some(Self::StructureStarts),
            (Self::Surface, 0) => Some(Self::Noise),
            (Self::Surface, 1) => Some(Self::Biomes),
            (Self::Surface, 2..=8) => Some(Self::StructureStarts),
            (Self::Carvers, 0) => Some(Self::Surface),
            (Self::Carvers, 1..=8) => Some(Self::StructureStarts),
            (Self::Features, 0..=1) => Some(Self::Carvers),
            (Self::Features, 2..=8) => Some(Self::StructureStarts),
            (Self::InitializeLight, 0) => Some(Self::Features),
            (Self::Light, 0..=1) => Some(Self::InitializeLight),
            (Self::Spawn, 0) => Some(Self::Light),
            (Self::Spawn, 1) => Some(Self::Biomes),
            (Self::Full, 0) => Some(Self::Spawn),
            _ => None,
        }
    }

    #[must_use]
    pub const fn execution(self) -> TaskExecution {
        match self {
            Self::Biomes | Self::Noise | Self::InitializeLight | Self::Light => {
                TaskExecution::Asynchronous
            }
            Self::Full => TaskExecution::MainThread,
            _ => TaskExecution::Synchronous,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
    Proto,
    Level,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskExecution {
    Synchronous,
    Asynchronous,
    MainThread,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationHeightmap {
    OceanFloorWorldGeneration,
    WorldSurfaceWorldGeneration,
    OceanFloor,
    WorldSurface,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[must_use]
pub fn chebyshev_distance(left: ChunkPos, right: ChunkPos) -> u32 {
    left.x.abs_diff(right.x).max(left.z.abs_diff(right.z))
}
