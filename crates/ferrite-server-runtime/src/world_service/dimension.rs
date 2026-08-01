//! Configured formal-dimension identities, layouts, and deterministic generation.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::DimensionId;
use ferrite_world::chunk::{ChunkAccessError, ChunkColumn, ChunkLayout, VerticalSectionRange};
use ferrite_world::generation::end_island::EndIslandDensity;
use ferrite_world::generation::overworld::{OverworldGenerationError, OverworldGeneratorV1};
use ferrite_world::generation::status::ChunkStatus;
use ferrite_world::id::{AIR, BiomeId, END_STONE, GRASS_BLOCK, NETHERRACK, STONE};
use ferrite_world::light::{ChunkLightError, recompute_chunk_light};
use thiserror::Error;

pub(crate) const OVERWORLD_BIOMES: [BiomeId; 3] =
    [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)];
pub(crate) const NETHER_WASTES: BiomeId = BiomeId::new(3);
pub(crate) const THE_END: BiomeId = BiomeId::new(4);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormalDimensionKind {
    Overworld,
    Nether,
    End,
}

impl FormalDimensionKind {
    pub(crate) fn from_dimension(dimension: &DimensionId) -> Result<Self, DimensionRuntimeError> {
        match dimension.to_string().as_str() {
            "minecraft:overworld" => Ok(Self::Overworld),
            "minecraft:the_nether" => Ok(Self::Nether),
            "minecraft:the_end" => Ok(Self::End),
            _ => Err(DimensionRuntimeError::UnsupportedDimension(
                dimension.clone(),
            )),
        }
    }

    pub(crate) fn layout(self) -> ChunkLayout {
        let (minimum, count, biome) = match self {
            Self::Overworld => (-4, 24, OVERWORLD_BIOMES[0]),
            Self::Nether => (0, 16, NETHER_WASTES),
            Self::End => (0, 16, THE_END),
        };
        ChunkLayout::new(
            VerticalSectionRange::new(minimum, count)
                .expect("locked formal dimension layout is valid"),
            AIR,
            biome,
        )
    }

    pub(crate) const fn sea_level(self) -> i32 {
        match self {
            Self::Overworld => 63,
            Self::Nether => 32,
            Self::End => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FormalDimensionGenerator {
    Overworld(Box<OverworldGeneratorV1>),
    Nether(NetherGeneratorV1),
    End(Box<EndGeneratorV1>),
}

impl FormalDimensionGenerator {
    pub(crate) fn new(dimension: &DimensionId, seed: i64) -> Result<Self, DimensionRuntimeError> {
        Ok(match FormalDimensionKind::from_dimension(dimension)? {
            FormalDimensionKind::Overworld => Self::Overworld(Box::new(OverworldGeneratorV1::new(
                seed,
                STONE,
                GRASS_BLOCK,
                OVERWORLD_BIOMES,
            ))),
            FormalDimensionKind::Nether => Self::Nether(NetherGeneratorV1 { seed }),
            FormalDimensionKind::End => Self::End(Box::new(EndGeneratorV1 {
                density: EndIslandDensity::new(seed),
            })),
        })
    }

    pub(crate) fn apply_stage(
        &self,
        chunk: &mut ChunkColumn,
        target: ChunkStatus,
    ) -> Result<(), DimensionRuntimeError> {
        match self {
            Self::Overworld(generator) => generator.apply_stage(chunk, target).map_err(Into::into),
            Self::Nether(generator) => generator.apply_stage(chunk, target),
            Self::End(generator) => generator.apply_stage(chunk, target),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NetherGeneratorV1 {
    seed: i64,
}

impl NetherGeneratorV1 {
    fn apply_stage(
        &self,
        chunk: &mut ChunkColumn,
        target: ChunkStatus,
    ) -> Result<(), DimensionRuntimeError> {
        match target {
            ChunkStatus::Noise => self.fill_noise(chunk),
            ChunkStatus::InitializeLight | ChunkStatus::Light => {
                recompute_chunk_light(chunk).map_err(Into::into)
            }
            _ => Ok(()),
        }
    }

    fn fill_noise(&self, chunk: &mut ChunkColumn) -> Result<(), DimensionRuntimeError> {
        let origin_x = chunk.position().checked_min_block_x()?;
        let origin_z = chunk.position().checked_min_block_z()?;
        chunk.set_uniform_section_blocks(0, NETHERRACK)?;
        chunk.set_uniform_section_blocks(7, NETHERRACK)?;
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = origin_x + local_x;
                let z = origin_z + local_z;
                let floor = 40
                    + i32::try_from(positional_hash(self.seed, x, z) % 25)
                        .expect("bounded Nether height fits i32");
                let ceiling = 119
                    - i32::try_from(positional_hash(self.seed ^ -7046029254386353131, x, z) % 12)
                        .expect("bounded Nether ceiling fits i32");
                for y in 16..=floor {
                    chunk.set_block(BlockPos::new(x, y, z), NETHERRACK)?;
                }
                for y in ceiling..=111 {
                    chunk.set_block(BlockPos::new(x, y, z), NETHERRACK)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EndGeneratorV1 {
    density: EndIslandDensity,
}

impl EndGeneratorV1 {
    fn apply_stage(
        &self,
        chunk: &mut ChunkColumn,
        target: ChunkStatus,
    ) -> Result<(), DimensionRuntimeError> {
        match target {
            ChunkStatus::Noise => self.fill_island(chunk),
            ChunkStatus::InitializeLight | ChunkStatus::Light => {
                recompute_chunk_light(chunk).map_err(Into::into)
            }
            _ => Ok(()),
        }
    }

    fn fill_island(&self, chunk: &mut ChunkColumn) -> Result<(), DimensionRuntimeError> {
        let origin_x = chunk.position().checked_min_block_x()?;
        let origin_z = chunk.position().checked_min_block_z()?;
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = origin_x + local_x;
                let z = origin_z + local_z;
                let density = self.density.sample(x, z);
                if density <= -0.2 {
                    continue;
                }
                let surface = (64.0 + density * 28.0).round().clamp(48.0, 80.0) as i32;
                let depth = (8.0 + density.max(0.0) * 18.0).round() as i32;
                for y in surface.saturating_sub(depth)..=surface {
                    chunk.set_block(BlockPos::new(x, y, z), END_STONE)?;
                }
            }
        }
        Ok(())
    }
}

fn positional_hash(seed: i64, x: i32, z: i32) -> u64 {
    let mut value = seed as u64 ^ 0x9e37_79b9_7f4a_7c15;
    value ^= (x as u32 as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= (z as u32 as u64).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Debug, Error)]
pub(crate) enum DimensionRuntimeError {
    #[error("formal world does not support dimension {0}")]
    UnsupportedDimension(DimensionId),
    #[error(transparent)]
    Chunk(#[from] ChunkAccessError),
    #[error(transparent)]
    Numeric(#[from] ferrite_foundation::numeric::NumericError),
    #[error(transparent)]
    Light(#[from] ChunkLightError),
    #[error(transparent)]
    Overworld(#[from] OverworldGenerationError),
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::coordinate::ChunkPos;
    use ferrite_foundation::resource::ResourceId;

    use super::*;

    fn dimension(path: &str) -> DimensionId {
        DimensionId::new(ResourceId::minecraft(path).unwrap())
    }

    fn generate(path: &str, seed: i64) -> ChunkColumn {
        let dimension = dimension(path);
        let kind = FormalDimensionKind::from_dimension(&dimension).unwrap();
        let generator = FormalDimensionGenerator::new(&dimension, seed).unwrap();
        let mut chunk = ChunkColumn::new(ChunkPos::new(0, 0), kind.layout());
        for status in ChunkStatus::ALL.into_iter().skip(1) {
            generator.apply_stage(&mut chunk, status).unwrap();
        }
        chunk
    }

    #[test]
    fn configured_dimensions_have_locked_independent_layouts() {
        assert_eq!(
            FormalDimensionKind::Overworld.layout().sections().minimum(),
            -4
        );
        assert_eq!(
            FormalDimensionKind::Overworld.layout().sections().count(),
            24
        );
        assert_eq!(FormalDimensionKind::Nether.layout().sections().minimum(), 0);
        assert_eq!(FormalDimensionKind::Nether.layout().sections().count(), 16);
        assert_eq!(FormalDimensionKind::End.layout().sections().count(), 16);
    }

    #[test]
    fn nether_and_end_generation_are_seeded_and_dimension_distinct() {
        let nether = generate("the_nether", 7);
        let nether_replay = generate("the_nether", 7);
        let nether_changed = generate("the_nether", 8);
        let end = generate("the_end", 7);
        assert_eq!(nether, nether_replay);
        assert_ne!(nether, nether_changed);
        assert_ne!(nether.layout(), end.layout());
        assert_eq!(
            nether.block_state(BlockPos::new(0, 0, 0)).unwrap(),
            NETHERRACK
        );
        assert_eq!(end.block_state(BlockPos::new(0, 64, 0)).unwrap(), END_STONE);
    }
}
