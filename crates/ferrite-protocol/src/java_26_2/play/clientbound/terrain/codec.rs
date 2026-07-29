use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::play::clientbound::terrain::packet::{
    BlockEntityData, ChunkBiomes, ChunkCoordinate, ChunkLightUpdate, FullChunk, HeightmapType,
    LIGHT_LAYER_BYTES, LightData, LightLayerUpdate, MAX_SECTION_BLOB_BYTES, SectionData,
    TerrainPacket,
};
use crate::java_26_2::play::clientbound::terrain::palette::{self, PaletteCodecError, PaletteKind};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, Copy)]
pub struct TerrainCodecContext {
    pub section_count: usize,
    pub biome_registry_size: usize,
}

impl TerrainCodecContext {
    pub fn validate(self) -> Result<Self, TerrainCodecError> {
        if self.section_count == 0 {
            return Err(TerrainCodecError::ZeroSections);
        }
        if self.biome_registry_size == 0 {
            return Err(TerrainCodecError::EmptyBiomeRegistry);
        }
        Ok(self)
    }
}

pub fn decode_body(
    identity: &'static str,
    reader: &mut WireReader<'_>,
    context: TerrainCodecContext,
) -> Result<TerrainPacket, TerrainCodecError> {
    let context = context.validate()?;
    match identity {
        "minecraft:bundle_delimiter" => Ok(TerrainPacket::BundleDelimiter),
        "minecraft:chunk_batch_finished" => {
            Ok(TerrainPacket::ChunkBatchFinished(reader.read_var_i32()?))
        }
        "minecraft:chunk_batch_start" => Ok(TerrainPacket::ChunkBatchStart),
        "minecraft:chunks_biomes" => Ok(TerrainPacket::ChunksBiomes(read_chunks_biomes(
            reader, context,
        )?)),
        "minecraft:forget_level_chunk" => Ok(TerrainPacket::ForgetLevelChunk(unpack_chunk(
            reader.read_i64()?,
        ))),
        "minecraft:level_chunk_with_light" => Ok(TerrainPacket::LevelChunkWithLight(
            read_full_chunk(reader, context)?,
        )),
        "minecraft:light_update" => Ok(TerrainPacket::LightUpdate(ChunkLightUpdate {
            position: ChunkCoordinate {
                x: reader.read_var_i32()?,
                z: reader.read_var_i32()?,
            },
            light: read_light(reader, context.section_count + 2)?,
        })),
        "minecraft:set_chunk_cache_center" => {
            Ok(TerrainPacket::SetChunkCacheCenter(ChunkCoordinate {
                x: reader.read_var_i32()?,
                z: reader.read_var_i32()?,
            }))
        }
        "minecraft:set_chunk_cache_radius" => {
            Ok(TerrainPacket::SetChunkCacheRadius(reader.read_var_i32()?))
        }
        "minecraft:set_simulation_distance" => {
            Ok(TerrainPacket::SetSimulationDistance(reader.read_var_i32()?))
        }
        _ => Err(TerrainCodecError::UnsupportedIdentity { identity }),
    }
}

pub fn encode_body(
    packet: &TerrainPacket,
    writer: &mut WireWriter,
    context: TerrainCodecContext,
) -> Result<(), TerrainCodecError> {
    let context = context.validate()?;
    match packet {
        TerrainPacket::BundleDelimiter | TerrainPacket::ChunkBatchStart => {}
        TerrainPacket::ChunkBatchFinished(size)
        | TerrainPacket::SetChunkCacheRadius(size)
        | TerrainPacket::SetSimulationDistance(size) => writer.write_var_i32(*size)?,
        TerrainPacket::ChunksBiomes(chunks) => write_chunks_biomes(writer, chunks, context)?,
        TerrainPacket::ForgetLevelChunk(position) => writer.write_i64(pack_chunk(*position))?,
        TerrainPacket::LevelChunkWithLight(chunk) => write_full_chunk(writer, chunk, context)?,
        TerrainPacket::LightUpdate(update) => {
            writer.write_var_i32(update.position.x)?;
            writer.write_var_i32(update.position.z)?;
            write_light(writer, &update.light, context.section_count + 2)?;
        }
        TerrainPacket::SetChunkCacheCenter(position) => {
            writer.write_var_i32(position.x)?;
            writer.write_var_i32(position.z)?;
        }
    }
    Ok(())
}

#[must_use]
pub const fn identity(packet: &TerrainPacket) -> &'static str {
    match packet {
        TerrainPacket::BundleDelimiter => "minecraft:bundle_delimiter",
        TerrainPacket::ChunkBatchFinished(_) => "minecraft:chunk_batch_finished",
        TerrainPacket::ChunkBatchStart => "minecraft:chunk_batch_start",
        TerrainPacket::ChunksBiomes(_) => "minecraft:chunks_biomes",
        TerrainPacket::ForgetLevelChunk(_) => "minecraft:forget_level_chunk",
        TerrainPacket::LevelChunkWithLight(_) => "minecraft:level_chunk_with_light",
        TerrainPacket::LightUpdate(_) => "minecraft:light_update",
        TerrainPacket::SetChunkCacheCenter(_) => "minecraft:set_chunk_cache_center",
        TerrainPacket::SetChunkCacheRadius(_) => "minecraft:set_chunk_cache_radius",
        TerrainPacket::SetSimulationDistance(_) => "minecraft:set_simulation_distance",
    }
}

#[must_use]
pub fn is_terrain_identity(identity: &str) -> bool {
    matches!(
        identity,
        "minecraft:bundle_delimiter"
            | "minecraft:chunk_batch_finished"
            | "minecraft:chunk_batch_start"
            | "minecraft:chunks_biomes"
            | "minecraft:forget_level_chunk"
            | "minecraft:level_chunk_with_light"
            | "minecraft:light_update"
            | "minecraft:set_chunk_cache_center"
            | "minecraft:set_chunk_cache_radius"
            | "minecraft:set_simulation_distance"
    )
}

fn read_full_chunk(
    reader: &mut WireReader<'_>,
    context: TerrainCodecContext,
) -> Result<FullChunk, TerrainCodecError> {
    let position = ChunkCoordinate {
        x: reader.read_i32()?,
        z: reader.read_i32()?,
    };
    let heightmap_count = reader.read_var_i32()?;
    let mut heightmaps = BTreeMap::new();
    for _ in 0..heightmap_count.max(0) {
        let kind = HeightmapType::from_raw_or_world_surface_worldgen(reader.read_var_i32()?);
        let long_count = reader.read_count("heightmap longs", reader.remaining() / 8)?;
        let mut values = Vec::with_capacity(long_count);
        for _ in 0..long_count {
            values.push(reader.read_i64()?);
        }
        heightmaps.insert(kind, values);
    }
    let section_blob = reader.read_byte_array(MAX_SECTION_BLOB_BYTES)?;
    let mut section_reader = WireReader::new(section_blob);
    let mut sections = Vec::with_capacity(context.section_count);
    for _ in 0..context.section_count {
        sections.push(read_section(
            &mut section_reader,
            context.biome_registry_size,
        )?);
    }
    let block_entity_count =
        reader.read_count("full chunk block entities", reader.remaining().max(1))?;
    let mut block_entities = Vec::with_capacity(block_entity_count);
    for _ in 0..block_entity_count {
        let packed_local_xz = reader.read_i8()?;
        let y = reader.read_i16()?;
        let type_raw_id = reader.read_var_i32()?;
        if !(0..49).contains(&type_raw_id) {
            return Err(TerrainCodecError::UnknownBlockEntityType { type_raw_id });
        }
        let update_tag = NetworkNbt::read_nullable(reader, NbtQuota::Default)?;
        block_entities.push(BlockEntityData {
            packed_local_xz,
            y,
            type_raw_id,
            update_tag,
        });
    }
    let light = read_light(reader, context.section_count + 2)?;
    Ok(FullChunk {
        position,
        heightmaps,
        sections,
        block_entities,
        light,
    })
}

fn write_full_chunk(
    writer: &mut WireWriter,
    chunk: &FullChunk,
    context: TerrainCodecContext,
) -> Result<(), TerrainCodecError> {
    require_sections(chunk.sections.len(), context.section_count)?;
    writer.write_i32(chunk.position.x)?;
    writer.write_i32(chunk.position.z)?;
    writer.write_count("heightmaps", chunk.heightmaps.len(), 6)?;
    for (kind, values) in &chunk.heightmaps {
        writer.write_var_i32(kind.raw_id())?;
        writer.write_count("heightmap longs", values.len(), MAX_SECTION_BLOB_BYTES / 8)?;
        for value in values {
            writer.write_i64(*value)?;
        }
    }
    let mut section_writer = WireWriter::new(MAX_SECTION_BLOB_BYTES);
    for section in &chunk.sections {
        write_section(&mut section_writer, section, context.biome_registry_size)?;
    }
    writer.write_byte_array(section_writer.as_slice(), MAX_SECTION_BLOB_BYTES)?;
    writer.write_count(
        "full chunk block entities",
        chunk.block_entities.len(),
        MAX_SECTION_BLOB_BYTES,
    )?;
    for entity in &chunk.block_entities {
        if !(0..49).contains(&entity.type_raw_id) {
            return Err(TerrainCodecError::UnknownBlockEntityType {
                type_raw_id: entity.type_raw_id,
            });
        }
        writer.write_i8(entity.packed_local_xz)?;
        writer.write_i16(entity.y)?;
        writer.write_var_i32(entity.type_raw_id)?;
        NetworkNbt::write_nullable(entity.update_tag.as_ref(), writer)?;
    }
    write_light(writer, &chunk.light, context.section_count + 2)
}

fn read_section(
    reader: &mut WireReader<'_>,
    biome_registry_size: usize,
) -> Result<SectionData, TerrainCodecError> {
    Ok(SectionData {
        non_empty_blocks: reader.read_i16()?,
        fluid_count: reader.read_i16()?,
        block_states: palette::read(reader, PaletteKind::Blocks)?,
        biomes: palette::read(
            reader,
            PaletteKind::Biomes {
                registry_size: biome_registry_size,
            },
        )?,
    })
}

fn write_section(
    writer: &mut WireWriter,
    section: &SectionData,
    biome_registry_size: usize,
) -> Result<(), TerrainCodecError> {
    writer.write_i16(section.non_empty_blocks)?;
    writer.write_i16(section.fluid_count)?;
    palette::write(writer, &section.block_states, PaletteKind::Blocks)?;
    palette::write(
        writer,
        &section.biomes,
        PaletteKind::Biomes {
            registry_size: biome_registry_size,
        },
    )?;
    Ok(())
}

fn read_chunks_biomes(
    reader: &mut WireReader<'_>,
    context: TerrainCodecContext,
) -> Result<Vec<ChunkBiomes>, TerrainCodecError> {
    let count = reader.read_count("biome chunks", reader.remaining().max(1))?;
    let mut chunks = Vec::with_capacity(count);
    for _ in 0..count {
        let position = unpack_chunk(reader.read_i64()?);
        let bytes = reader.read_byte_array(MAX_SECTION_BLOB_BYTES)?;
        let mut biome_reader = WireReader::new(bytes);
        let mut sections = Vec::with_capacity(context.section_count);
        for _ in 0..context.section_count {
            sections.push(palette::read(
                &mut biome_reader,
                PaletteKind::Biomes {
                    registry_size: context.biome_registry_size,
                },
            )?);
        }
        chunks.push(ChunkBiomes { position, sections });
    }
    Ok(chunks)
}

fn write_chunks_biomes(
    writer: &mut WireWriter,
    chunks: &[ChunkBiomes],
    context: TerrainCodecContext,
) -> Result<(), TerrainCodecError> {
    writer.write_count("biome chunks", chunks.len(), MAX_INFLATED_PACKET_LENGTH)?;
    for chunk in chunks {
        require_sections(chunk.sections.len(), context.section_count)?;
        writer.write_i64(pack_chunk(chunk.position))?;
        let mut biome_writer = WireWriter::new(MAX_SECTION_BLOB_BYTES);
        for section in &chunk.sections {
            palette::write(
                &mut biome_writer,
                section,
                PaletteKind::Biomes {
                    registry_size: context.biome_registry_size,
                },
            )?;
        }
        writer.write_byte_array(biome_writer.as_slice(), MAX_SECTION_BLOB_BYTES)?;
    }
    Ok(())
}

fn read_light(
    reader: &mut WireReader<'_>,
    layer_count: usize,
) -> Result<LightData, TerrainCodecError> {
    let sky_data = read_bitset(reader)?;
    let block_data = read_bitset(reader)?;
    let sky_empty = read_bitset(reader)?;
    let block_empty = read_bitset(reader)?;
    let sky_updates = read_light_arrays(reader)?;
    let block_updates = read_light_arrays(reader)?;
    Ok(LightData {
        sky: resolve_light_layers(layer_count, &sky_data, &sky_empty, &sky_updates)?,
        block: resolve_light_layers(layer_count, &block_data, &block_empty, &block_updates)?,
    })
}

fn write_light(
    writer: &mut WireWriter,
    light: &LightData,
    layer_count: usize,
) -> Result<(), TerrainCodecError> {
    require_light_layers(light.sky.len(), layer_count)?;
    require_light_layers(light.block.len(), layer_count)?;
    let (sky_data, sky_empty, sky_updates) = split_light_layers(&light.sky);
    let (block_data, block_empty, block_updates) = split_light_layers(&light.block);
    write_bitset(writer, &sky_data)?;
    write_bitset(writer, &block_data)?;
    write_bitset(writer, &sky_empty)?;
    write_bitset(writer, &block_empty)?;
    write_light_arrays(writer, &sky_updates)?;
    write_light_arrays(writer, &block_updates)
}

fn read_bitset(reader: &mut WireReader<'_>) -> Result<Vec<u64>, TerrainCodecError> {
    let count = reader.read_count("bitset longs", reader.remaining() / 8)?;
    let mut words = Vec::with_capacity(count);
    for _ in 0..count {
        words.push(reader.read_i64()? as u64);
    }
    Ok(words)
}

fn write_bitset(writer: &mut WireWriter, words: &[u64]) -> Result<(), TerrainCodecError> {
    let retained = words
        .iter()
        .rposition(|word| *word != 0)
        .map_or(0, |index| index + 1);
    writer.write_count("bitset longs", retained, words.len())?;
    for word in &words[..retained] {
        writer.write_i64(*word as i64)?;
    }
    Ok(())
}

fn read_light_arrays(reader: &mut WireReader<'_>) -> Result<Vec<Vec<u8>>, TerrainCodecError> {
    let count = reader.read_count("light updates", reader.remaining().max(1))?;
    let mut updates = Vec::with_capacity(count);
    for _ in 0..count {
        updates.push(reader.read_byte_array(LIGHT_LAYER_BYTES)?.to_vec());
    }
    Ok(updates)
}

fn write_light_arrays(
    writer: &mut WireWriter,
    updates: &[&[u8; LIGHT_LAYER_BYTES]],
) -> Result<(), TerrainCodecError> {
    writer.write_count("light updates", updates.len(), MAX_INFLATED_PACKET_LENGTH)?;
    for update in updates {
        writer.write_byte_array(update.as_slice(), LIGHT_LAYER_BYTES)?;
    }
    Ok(())
}

fn resolve_light_layers(
    layer_count: usize,
    data_mask: &[u64],
    empty_mask: &[u64],
    updates: &[Vec<u8>],
) -> Result<Vec<LightLayerUpdate>, TerrainCodecError> {
    let mut resolved = vec![LightLayerUpdate::Unchanged; layer_count];
    let mut update_index = 0;
    let highest_bit = data_mask.len().saturating_mul(64);
    for bit in 0..highest_bit {
        if !bit_is_set(data_mask, bit) {
            continue;
        }
        let update = updates
            .get(update_index)
            .ok_or(TerrainCodecError::MissingLightArray { bit })?;
        update_index += 1;
        if update.len() != LIGHT_LAYER_BYTES {
            return Err(TerrainCodecError::LightArrayLength {
                bit,
                actual: update.len(),
            });
        }
        if let Some(layer) = resolved.get_mut(bit) {
            let data: [u8; LIGHT_LAYER_BYTES] = update
                .as_slice()
                .try_into()
                .expect("validated light arrays have fixed length");
            *layer = LightLayerUpdate::Data(Box::new(data));
        }
    }
    for (bit, layer) in resolved.iter_mut().enumerate() {
        if matches!(layer, LightLayerUpdate::Unchanged) && bit_is_set(empty_mask, bit) {
            *layer = LightLayerUpdate::Empty;
        }
    }
    Ok(resolved)
}

fn split_light_layers(
    layers: &[LightLayerUpdate],
) -> (Vec<u64>, Vec<u64>, Vec<&[u8; LIGHT_LAYER_BYTES]>) {
    let words = layers.len().div_ceil(64);
    let mut data_mask = vec![0; words];
    let mut empty_mask = vec![0; words];
    let mut updates = Vec::new();
    for (bit, layer) in layers.iter().enumerate() {
        match layer {
            LightLayerUpdate::Unchanged => {}
            LightLayerUpdate::Empty => set_bit(&mut empty_mask, bit),
            LightLayerUpdate::Data(data) => {
                set_bit(&mut data_mask, bit);
                updates.push(data.as_ref());
            }
        }
    }
    (data_mask, empty_mask, updates)
}

fn bit_is_set(words: &[u64], bit: usize) -> bool {
    words
        .get(bit / 64)
        .is_some_and(|word| word & (1u64 << (bit % 64)) != 0)
}

fn set_bit(words: &mut [u64], bit: usize) {
    words[bit / 64] |= 1u64 << (bit % 64);
}

fn require_sections(actual: usize, expected: usize) -> Result<(), TerrainCodecError> {
    if actual == expected {
        Ok(())
    } else {
        Err(TerrainCodecError::SectionCount { expected, actual })
    }
}

fn require_light_layers(actual: usize, expected: usize) -> Result<(), TerrainCodecError> {
    if actual == expected {
        Ok(())
    } else {
        Err(TerrainCodecError::LightLayerCount { expected, actual })
    }
}

const fn pack_chunk(position: ChunkCoordinate) -> i64 {
    (((position.z as u32 as u64) << 32) | (position.x as u32 as u64)) as i64
}

const fn unpack_chunk(value: i64) -> ChunkCoordinate {
    let bits = value as u64;
    ChunkCoordinate {
        x: bits as u32 as i32,
        z: (bits >> 32) as u32 as i32,
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum TerrainCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Palette(#[from] PaletteCodecError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error("terrain codec requires at least one section")]
    ZeroSections,
    #[error("terrain codec requires a nonempty configured biome registry")]
    EmptyBiomeRegistry,
    #[error("packet {identity} is outside the C2 terrain family")]
    UnsupportedIdentity { identity: &'static str },
    #[error("terrain payload has {actual} sections, expected {expected}")]
    SectionCount { expected: usize, actual: usize },
    #[error("light payload has {actual} layers, expected {expected}")]
    LightLayerCount { expected: usize, actual: usize },
    #[error("light data bit {bit} has no corresponding update array")]
    MissingLightArray { bit: usize },
    #[error("light data bit {bit} has {actual} bytes instead of 2048")]
    LightArrayLength { bit: usize, actual: usize },
    #[error("block entity type raw ID {type_raw_id} is absent from the locked registry")]
    UnknownBlockEntityType { type_raw_id: i32 },
}
