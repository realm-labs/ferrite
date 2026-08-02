use std::collections::BTreeMap;

use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::projection::{ChunkSnapshot, ClientHeightmap, LightLayer};
use thiserror::Error;

use crate::worldgen_oracle::model::{
    BIOMES_PER_SECTION, BLOCKS_PER_SECTION, CanonicalNbt, ChunkCoordinate, NORMALIZATION_SCHEMA,
    SemanticBlockEntity, SemanticChunk, SemanticSection, SemanticSource,
};

pub struct FerriteSemanticInput<'a> {
    pub chunk: &'a ChunkSnapshot,
    pub data_version: i32,
    pub dimension: &'a str,
    pub status: &'a str,
    pub block_state_name: &'a dyn Fn(BlockStateId) -> Option<String>,
    pub fluid_state_name: &'a dyn Fn(BlockStateId) -> Option<String>,
    pub biome_name: &'a dyn Fn(BiomeId) -> Option<String>,
    pub post_processing: CanonicalNbt,
    pub structure_starts: CanonicalNbt,
    pub structure_references: CanonicalNbt,
    pub scheduled_block_ticks: CanonicalNbt,
    pub scheduled_fluid_ticks: CanonicalNbt,
    pub light_initialized: bool,
    pub inhabited_time: i64,
    pub generation_metadata: BTreeMap<String, CanonicalNbt>,
}

pub fn generate_current_ferrite_chunk(
    dimension: &str,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<SemanticChunk, FerriteWorldgenOracleError> {
    if dimension != "minecraft:overworld" {
        return Err(FerriteWorldgenOracleError::UnsupportedDimension(
            dimension.to_owned(),
        ));
    }
    let layout = ferrite_world::chunk::ChunkLayout::new(
        ferrite_world::chunk::VerticalSectionRange::new(-4, 24)
            .expect("locked Overworld layout is valid"),
        ferrite_world::id::AIR,
        BiomeId::new(0),
    );
    let mut chunk = ferrite_world::chunk::ChunkColumn::new(
        ferrite_foundation::coordinate::ChunkPos::new(chunk_x, chunk_z),
        layout,
    );
    let generator = ferrite_world::generation::overworld::OverworldGeneratorV1::new(
        seed,
        ferrite_world::id::STONE,
        ferrite_world::id::GRASS_BLOCK,
        [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)],
    );
    for status in ferrite_world::generation::status::ChunkStatus::ALL
        .into_iter()
        .skip(1)
    {
        generator.apply_stage(&mut chunk, status)?;
    }
    let light = chunk
        .light()
        .ok_or(FerriteWorldgenOracleError::MissingLight)?
        .snapshot(layout.sections().count())?;
    let snapshot = chunk.snapshot(light, |_, state| state != ferrite_world::id::AIR)?;
    let empty = || CanonicalNbt::empty_compound();
    normalize_ferrite_chunk(FerriteSemanticInput {
        chunk: &snapshot,
        data_version: 4_903,
        dimension,
        status: "minecraft:full",
        block_state_name: &block_state_name,
        fluid_state_name: &fluid_state_name,
        biome_name: &biome_name,
        post_processing: CanonicalNbt::empty_list(),
        structure_starts: empty(),
        structure_references: empty(),
        scheduled_block_ticks: CanonicalNbt::empty_list(),
        scheduled_fluid_ticks: CanonicalNbt::empty_list(),
        light_initialized: true,
        inhabited_time: 0,
        generation_metadata: BTreeMap::new(),
    })
    .map_err(Into::into)
}

pub fn normalize_ferrite_chunk(
    input: FerriteSemanticInput<'_>,
) -> Result<SemanticChunk, FerriteNormalizationError> {
    let layout = input.chunk.layout();
    let sections =
        input
            .chunk
            .sections()
            .iter()
            .enumerate()
            .map(|(offset, section)| {
                let y = layout.sections().minimum() + offset as i32;
                let block_states = (0..BLOCKS_PER_SECTION)
                    .map(|index| {
                        let state = section.blocks().get(index).map_err(|_| {
                            FerriteNormalizationError::SectionShape { section_y: y }
                        })?;
                        (input.block_state_name)(state)
                            .ok_or(FerriteNormalizationError::UnknownBlockState(state.get()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let fluid_states = (0..BLOCKS_PER_SECTION)
                    .map(|index| {
                        let state = section.blocks().get(index).map_err(|_| {
                            FerriteNormalizationError::SectionShape { section_y: y }
                        })?;
                        (input.fluid_state_name)(state)
                            .ok_or(FerriteNormalizationError::UnknownFluidState(state.get()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let biomes = (0..BIOMES_PER_SECTION)
                    .map(|index| {
                        let biome = section.biome(index).map_err(|_| {
                            FerriteNormalizationError::SectionShape { section_y: y }
                        })?;
                        (input.biome_name)(biome)
                            .ok_or(FerriteNormalizationError::UnknownBiome(biome.get()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(SemanticSection {
                    y,
                    block_states,
                    fluid_states,
                    biomes,
                    sky_light: layer_bytes(&input.chunk.light().sky()[offset + 1]),
                    block_light: layer_bytes(&input.chunk.light().block()[offset + 1]),
                })
            })
            .collect::<Result<Vec<_>, FerriteNormalizationError>>()?;

    let heightmaps = input
        .chunk
        .heightmaps()
        .iter()
        .map(|(kind, values)| (heightmap_name(*kind).to_owned(), values.to_vec()))
        .collect();
    let mut block_entities = input
        .chunk
        .block_entities()
        .iter()
        .map(|block_entity| SemanticBlockEntity {
            position: [
                block_entity.position.x,
                block_entity.position.y,
                block_entity.position.z,
            ],
            kind: block_entity.kind.to_string(),
            data: CanonicalNbt::empty_compound(),
        })
        .collect::<Vec<_>>();
    block_entities.sort_by_key(|block_entity| block_entity.position);

    let position = input.chunk.position();
    let chunk = SemanticChunk {
        schema: NORMALIZATION_SCHEMA.to_owned(),
        source: SemanticSource::Ferrite,
        reference_version: "26.2".to_owned(),
        data_version: input.data_version,
        dimension: input.dimension.to_owned(),
        position: ChunkCoordinate {
            x: position.x,
            z: position.z,
        },
        status: input.status.to_owned(),
        sections,
        heightmaps,
        block_entities,
        post_processing: input.post_processing,
        structure_starts: input.structure_starts,
        structure_references: input.structure_references,
        scheduled_block_ticks: input.scheduled_block_ticks,
        scheduled_fluid_ticks: input.scheduled_fluid_ticks,
        light_initialized: input.light_initialized,
        inhabited_time: input.inhabited_time,
        generation_metadata: input.generation_metadata,
    };
    chunk
        .validate_shape()
        .map_err(FerriteNormalizationError::Shape)?;
    Ok(chunk)
}

fn layer_bytes(layer: &LightLayer) -> Option<Vec<u8>> {
    match layer {
        LightLayer::Empty => None,
        LightLayer::Data(bytes) => Some(bytes.to_vec()),
    }
}

fn block_state_name(state: BlockStateId) -> Option<String> {
    let name = match state {
        ferrite_world::id::AIR => "minecraft:air",
        ferrite_world::id::STONE => "minecraft:stone",
        ferrite_world::id::GRASS_BLOCK => "minecraft:grass_block[snowy=false]",
        ferrite_world::id::WATER => "minecraft:water[level=0]",
        ferrite_world::id::LAVA => "minecraft:lava[level=0]",
        ferrite_world::id::FIRE => "minecraft:fire",
        ferrite_world::id::NETHERRACK => "minecraft:netherrack",
        ferrite_world::id::END_STONE => "minecraft:end_stone",
        ferrite_world::id::OBSIDIAN => "minecraft:obsidian",
        ferrite_world::id::NETHER_PORTAL_X => "minecraft:nether_portal[axis=x]",
        ferrite_world::id::NETHER_PORTAL_Z => "minecraft:nether_portal[axis=z]",
        ferrite_world::id::END_PORTAL => "minecraft:end_portal",
        _ => return None,
    };
    Some(name.to_owned())
}

fn fluid_state_name(state: BlockStateId) -> Option<String> {
    let name = if state == ferrite_world::id::WATER {
        "minecraft:water[level=0]"
    } else if state == ferrite_world::id::LAVA {
        "minecraft:lava[level=0]"
    } else if block_state_name(state).is_some() {
        "minecraft:empty"
    } else {
        return None;
    };
    Some(name.to_owned())
}

fn biome_name(biome: BiomeId) -> Option<String> {
    let name = match biome.get() {
        0 => "minecraft:plains",
        1 => "minecraft:snowy_plains",
        2 => "minecraft:forest",
        3 => "minecraft:nether_wastes",
        4 => "minecraft:the_end",
        _ => return None,
    };
    Some(name.to_owned())
}

const fn heightmap_name(kind: ClientHeightmap) -> &'static str {
    match kind {
        ClientHeightmap::WorldSurface => "WORLD_SURFACE",
        ClientHeightmap::MotionBlocking => "MOTION_BLOCKING",
        ClientHeightmap::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
    }
}

#[derive(Debug, Error)]
pub enum FerriteNormalizationError {
    #[error("section {section_y} does not have the fixed semantic shape")]
    SectionShape { section_y: i32 },
    #[error("block state runtime ID {0} has no stable 26.2 identity")]
    UnknownBlockState(u32),
    #[error("block state runtime ID {0} has no derived stable fluid identity")]
    UnknownFluidState(u32),
    #[error("biome runtime ID {0} has no stable 26.2 identity")]
    UnknownBiome(u32),
    #[error("normalized Ferrite chunk shape is invalid: {0}")]
    Shape(String),
}

#[derive(Debug, Error)]
pub enum FerriteWorldgenOracleError {
    #[error("current Ferrite generator does not expose dimension {0} to the oracle")]
    UnsupportedDimension(String),
    #[error("generated Ferrite chunk has no authoritative light state")]
    MissingLight,
    #[error(transparent)]
    Generation(#[from] ferrite_world::generation::overworld::OverworldGenerationError),
    #[error(transparent)]
    Projection(#[from] ferrite_world::projection::ChunkProjectionError),
    #[error(transparent)]
    Normalization(#[from] FerriteNormalizationError),
}
