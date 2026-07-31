//! Exact status-task planning independent of the execution substrate.

use crate::generation::status::{ChunkStatus, GenerationHeightmap, TaskExecution};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyramidMode {
    Generation,
    Loading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskOptions {
    pub generate_structures: bool,
    pub upgrading: bool,
    pub below_zero_retrogen: bool,
    pub apply_bedrock_hole_mask: bool,
    pub debug_disable_features: bool,
    pub light_correct: bool,
}

impl Default for TaskOptions {
    fn default() -> Self {
        Self {
            generate_structures: true,
            upgrading: false,
            below_zero_retrogen: false,
            apply_bedrock_hole_mask: false,
            debug_disable_features: false,
            light_correct: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusTaskPlan {
    pub target: ChunkStatus,
    pub execution: TaskExecution,
    pub operations: Vec<TaskOperation>,
}

impl StatusTaskPlan {
    #[must_use]
    pub fn new(
        target: ChunkStatus,
        mode: PyramidMode,
        persisted: ChunkStatus,
        options: TaskOptions,
    ) -> Self {
        let already_lighted = persisted >= ChunkStatus::Light && options.light_correct;
        let mut operations = Vec::new();
        match target {
            ChunkStatus::Empty => operations.push(TaskOperation::PassThrough),
            ChunkStatus::StructureStarts => {
                if mode == PyramidMode::Generation && options.generate_structures {
                    operations.push(TaskOperation::GenerateStructureStarts);
                }
                operations.push(TaskOperation::NotifyStructureStartsAvailable);
            }
            ChunkStatus::StructureReferences => {
                operations.push(TaskOperation::CreateStructureManager);
                operations.push(TaskOperation::CreateStructureReferences);
            }
            ChunkStatus::Biomes => {
                operations.push(TaskOperation::CreateStructureManager);
                operations.push(TaskOperation::CreateBiomes);
            }
            ChunkStatus::Noise => {
                operations.push(TaskOperation::FillFromNoise);
                if options.below_zero_retrogen {
                    operations.push(TaskOperation::ReplaceOldBedrock);
                    if options.apply_bedrock_hole_mask {
                        operations.push(TaskOperation::ApplyBedrockHoleMask);
                    }
                }
            }
            ChunkStatus::Surface => operations.push(TaskOperation::BuildSurface),
            ChunkStatus::Carvers => {
                operations.push(TaskOperation::InstallOldChunkCarvingMask);
                operations.push(TaskOperation::ApplyCarvers);
            }
            ChunkStatus::Features => {
                for heightmap in [
                    GenerationHeightmap::OceanFloor,
                    GenerationHeightmap::WorldSurface,
                    GenerationHeightmap::MotionBlocking,
                    GenerationHeightmap::MotionBlockingNoLeaves,
                ] {
                    operations.push(TaskOperation::PrimeHeightmap(heightmap));
                }
                if !options.debug_disable_features {
                    operations.push(TaskOperation::DecorateBiomes);
                }
                operations.push(TaskOperation::GenerateBlendingBorderTicks);
            }
            ChunkStatus::InitializeLight => {
                operations.push(TaskOperation::InitializeLightSources);
                operations.push(TaskOperation::InstallLightEngine);
                operations.push(TaskOperation::InitializeLight { already_lighted });
            }
            ChunkStatus::Light => {
                operations.push(TaskOperation::LightChunk { already_lighted });
            }
            ChunkStatus::Spawn => {
                if !options.upgrading {
                    operations.push(TaskOperation::SpawnOriginalMobs);
                }
            }
            ChunkStatus::Full => {
                operations.extend([
                    TaskOperation::ResolveOrConstructLevelChunk,
                    TaskOperation::ReplaceProtochunkWhenConstructed,
                    TaskOperation::InstallFullStatusSupplier,
                    TaskOperation::LoadPostLoadEntities,
                    TaskOperation::MarkLoaded,
                    TaskOperation::RegisterBlockEntities,
                    TaskOperation::RegisterTickContainers,
                    TaskOperation::InstallUnsavedListener,
                ]);
            }
        }
        Self {
            target,
            execution: target.execution(),
            operations,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskOperation {
    PassThrough,
    GenerateStructureStarts,
    NotifyStructureStartsAvailable,
    CreateStructureManager,
    CreateStructureReferences,
    CreateBiomes,
    FillFromNoise,
    ReplaceOldBedrock,
    ApplyBedrockHoleMask,
    BuildSurface,
    InstallOldChunkCarvingMask,
    ApplyCarvers,
    PrimeHeightmap(GenerationHeightmap),
    DecorateBiomes,
    GenerateBlendingBorderTicks,
    InitializeLightSources,
    InstallLightEngine,
    InitializeLight { already_lighted: bool },
    LightChunk { already_lighted: bool },
    SpawnOriginalMobs,
    ResolveOrConstructLevelChunk,
    ReplaceProtochunkWhenConstructed,
    InstallFullStatusSupplier,
    LoadPostLoadEntities,
    MarkLoaded,
    RegisterBlockEntities,
    RegisterTickContainers,
    InstallUnsavedListener,
}
