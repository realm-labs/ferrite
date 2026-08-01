//! Bounded stable encoding for Region-owned chunk columns.

use std::collections::BTreeMap;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

use crate::chunk::{ChunkAccessError, ChunkColumn, ChunkLayout, VerticalSectionRange};
use crate::generation::structure_state::{
    ChunkStructureState, MAX_STRUCTURE_REFERENCES_PER_CHUNK, MAX_STRUCTURE_STARTS_PER_CHUNK,
    StructureBounds, StructurePlacement, StructureStateError,
};
use crate::id::{BiomeId, BlockStateId};
use crate::projection::{ChunkLightState, LIGHT_LAYER_BYTES, LightLayer};
use crate::section::{BIOMES_PER_SECTION, BLOCKS_PER_SECTION, ChunkSection, SectionError};

const CHUNK_MAGIC_V1: [u8; 4] = *b"FWC1";
const CHUNK_MAGIC_V2: [u8; 4] = *b"FWC2";
const CHUNK_MAGIC_V3: [u8; 4] = *b"FWC3";
pub const MAX_DURABLE_CHUNK_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_DURABLE_SECTIONS: usize = 128;
pub const MAX_DURABLE_BLOCK_ENTITIES: usize = 65_536;
const MAX_RESOURCE_ID_BYTES: usize = u16::MAX as usize;

pub fn encode_chunk(chunk: &ChunkColumn) -> Result<Vec<u8>, DurableChunkError> {
    if chunk.section_slots().len() > MAX_DURABLE_SECTIONS {
        return Err(DurableChunkError::TooManySections);
    }
    if chunk.block_entities().len() > MAX_DURABLE_BLOCK_ENTITIES {
        return Err(DurableChunkError::TooManyBlockEntities);
    }
    let mut output = Vec::new();
    output.extend_from_slice(&CHUNK_MAGIC_V3);
    push_i32(&mut output, chunk.position().x);
    push_i32(&mut output, chunk.position().z);
    let layout = chunk.layout();
    push_i32(&mut output, layout.sections().minimum());
    push_u16(&mut output, layout.sections().count());
    push_u32(&mut output, layout.default_block().get());
    push_u32(&mut output, layout.default_biome().get());
    push_u64(&mut output, chunk.revision().get());
    for section in chunk.section_slots() {
        match section {
            None => output.push(0),
            Some(section) => {
                output.push(1);
                push_u64(&mut output, section.revision().get());
                for block in section.blocks().values() {
                    push_u32(&mut output, block.get());
                }
                for biome in section.biomes().values() {
                    push_u32(&mut output, biome.get());
                }
            }
        }
    }
    push_u32(
        &mut output,
        u32::try_from(chunk.block_entities().len())
            .map_err(|_| DurableChunkError::TooManyBlockEntities)?,
    );
    for (position, kind) in chunk.block_entities() {
        push_i32(&mut output, position.x);
        push_i32(&mut output, position.y);
        push_i32(&mut output, position.z);
        push_identity(&mut output, kind)?;
    }
    push_u16(&mut output, chunk.structures().version());
    push_u16(
        &mut output,
        u16::try_from(chunk.structures().starts().len())
            .map_err(|_| DurableChunkError::TooManyStructureStarts)?,
    );
    for start in chunk.structures().starts() {
        push_structure_placement(&mut output, start)?;
    }
    push_u16(
        &mut output,
        u16::try_from(chunk.structures().references().len())
            .map_err(|_| DurableChunkError::TooManyStructureReferences)?,
    );
    for reference in chunk.structures().references() {
        push_structure_placement(&mut output, reference)?;
    }
    match chunk.light() {
        None => output.push(0),
        Some(light) => {
            output.push(1);
            for layer in light.sky().iter().chain(light.block()) {
                match layer {
                    LightLayer::Empty => output.push(0),
                    LightLayer::Data(bytes) => {
                        output.push(1);
                        output.extend_from_slice(bytes.as_ref());
                    }
                }
            }
        }
    }
    if output.len() > MAX_DURABLE_CHUNK_BYTES {
        return Err(DurableChunkError::PayloadTooLarge);
    }
    Ok(output)
}

pub fn decode_chunk(bytes: &[u8]) -> Result<ChunkColumn, DurableChunkError> {
    if bytes.len() > MAX_DURABLE_CHUNK_BYTES {
        return Err(DurableChunkError::PayloadTooLarge);
    }
    let mut cursor = Cursor::new(bytes);
    let (has_structure_state, has_light_state) = match cursor.fixed::<4>()? {
        CHUNK_MAGIC_V1 => (false, false),
        CHUNK_MAGIC_V2 => (true, false),
        CHUNK_MAGIC_V3 => (true, true),
        _ => return Err(DurableChunkError::WrongMagic),
    };
    let position = ChunkPos::new(cursor.i32()?, cursor.i32()?);
    let minimum_section = cursor.i32()?;
    let section_count = cursor.u16()?;
    if usize::from(section_count) > MAX_DURABLE_SECTIONS {
        return Err(DurableChunkError::TooManySections);
    }
    let layout = ChunkLayout::new(
        VerticalSectionRange::new(minimum_section, section_count)?,
        BlockStateId::new(cursor.u32()?),
        BiomeId::new(cursor.u32()?),
    );
    let revision = cursor.u64()?;
    let mut sections = Vec::with_capacity(usize::from(section_count));
    for _ in 0..section_count {
        let section = match cursor.u8()? {
            0 => None,
            1 => {
                let section_revision = cursor.u64()?;
                let mut blocks = Vec::with_capacity(BLOCKS_PER_SECTION);
                for _ in 0..BLOCKS_PER_SECTION {
                    blocks.push(BlockStateId::new(cursor.u32()?));
                }
                let mut biomes = Vec::with_capacity(BIOMES_PER_SECTION);
                for _ in 0..BIOMES_PER_SECTION {
                    biomes.push(BiomeId::new(cursor.u32()?));
                }
                Some(ChunkSection::from_durable_values(
                    &blocks,
                    &biomes,
                    section_revision,
                )?)
            }
            tag => return Err(DurableChunkError::InvalidSectionTag(tag)),
        };
        sections.push(section);
    }
    let block_entity_count = cursor.u32()? as usize;
    if block_entity_count > MAX_DURABLE_BLOCK_ENTITIES {
        return Err(DurableChunkError::TooManyBlockEntities);
    }
    let mut block_entities = BTreeMap::new();
    for _ in 0..block_entity_count {
        let block_position = BlockPos::new(cursor.i32()?, cursor.i32()?, cursor.i32()?);
        let kind = cursor.identity()?;
        if block_entities.insert(block_position, kind).is_some() {
            return Err(DurableChunkError::DuplicateBlockEntity);
        }
    }
    let structures = if has_structure_state {
        let version = cursor.u16()?;
        let start_count = usize::from(cursor.u16()?);
        if start_count > MAX_STRUCTURE_STARTS_PER_CHUNK {
            return Err(DurableChunkError::TooManyStructureStarts);
        }
        let mut starts = Vec::with_capacity(start_count);
        for _ in 0..start_count {
            starts.push(cursor.structure_placement()?);
        }
        let reference_count = usize::from(cursor.u16()?);
        if reference_count > MAX_STRUCTURE_REFERENCES_PER_CHUNK {
            return Err(DurableChunkError::TooManyStructureReferences);
        }
        let mut references = Vec::with_capacity(reference_count);
        for _ in 0..reference_count {
            references.push(cursor.structure_placement()?);
        }
        ChunkStructureState::from_durable_parts(version, position, starts, references)?
    } else {
        ChunkStructureState::empty()
    };
    let light = if has_light_state {
        match cursor.u8()? {
            0 => None,
            1 => {
                let layer_count = usize::from(section_count) + 2;
                let mut layers = Vec::with_capacity(layer_count * 2);
                for _ in 0..layer_count * 2 {
                    layers.push(match cursor.u8()? {
                        0 => LightLayer::Empty,
                        1 => LightLayer::Data(Box::new(cursor.fixed::<LIGHT_LAYER_BYTES>()?)),
                        tag => return Err(DurableChunkError::InvalidLightLayerTag(tag)),
                    });
                }
                let block = layers.split_off(layer_count);
                Some(ChunkLightState::new(layers, block, section_count)?)
            }
            tag => return Err(DurableChunkError::InvalidLightStateTag(tag)),
        }
    } else {
        None
    };
    cursor.finish()?;
    ChunkColumn::from_durable_parts(
        position,
        layout,
        sections,
        block_entities,
        structures,
        light,
        revision,
    )
    .map_err(Into::into)
}

fn push_structure_placement(
    output: &mut Vec<u8>,
    placement: &StructurePlacement,
) -> Result<(), DurableChunkError> {
    push_identity(output, &placement.structure)?;
    push_i32(output, placement.start_chunk.x);
    push_i32(output, placement.start_chunk.z);
    for value in [
        placement.bounds.minimum_x,
        placement.bounds.minimum_y,
        placement.bounds.minimum_z,
        placement.bounds.maximum_x,
        placement.bounds.maximum_y,
        placement.bounds.maximum_z,
    ] {
        push_i32(output, value);
    }
    push_u64(output, placement.placement_seed);
    Ok(())
}

fn push_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn push_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn push_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn push_identity(output: &mut Vec<u8>, identity: &ResourceId) -> Result<(), DurableChunkError> {
    let value = identity.to_string();
    let length = u16::try_from(value.len()).map_err(|_| DurableChunkError::IdentityTooLong)?;
    push_u16(output, length);
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], DurableChunkError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(DurableChunkError::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(DurableChunkError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], DurableChunkError> {
        self.take(N)?
            .try_into()
            .map_err(|_| DurableChunkError::Truncated)
    }

    fn u8(&mut self) -> Result<u8, DurableChunkError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, DurableChunkError> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    fn u32(&mut self) -> Result<u32, DurableChunkError> {
        Ok(u32::from_be_bytes(self.fixed()?))
    }

    fn u64(&mut self) -> Result<u64, DurableChunkError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    fn i32(&mut self) -> Result<i32, DurableChunkError> {
        Ok(i32::from_be_bytes(self.fixed()?))
    }

    fn identity(&mut self) -> Result<ResourceId, DurableChunkError> {
        let length = usize::from(self.u16()?);
        if length > MAX_RESOURCE_ID_BYTES {
            return Err(DurableChunkError::IdentityTooLong);
        }
        std::str::from_utf8(self.take(length)?)
            .map_err(|_| DurableChunkError::InvalidIdentity)?
            .parse()
            .map_err(|_| DurableChunkError::InvalidIdentity)
    }

    fn structure_placement(&mut self) -> Result<StructurePlacement, DurableChunkError> {
        let structure = self.identity()?;
        let start_chunk = ChunkPos::new(self.i32()?, self.i32()?);
        let bounds = StructureBounds::new(
            self.i32()?,
            self.i32()?,
            self.i32()?,
            self.i32()?,
            self.i32()?,
            self.i32()?,
        )?;
        Ok(StructurePlacement::new(
            structure,
            start_chunk,
            bounds,
            self.u64()?,
        ))
    }

    fn finish(self) -> Result<(), DurableChunkError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(DurableChunkError::TrailingBytes)
        }
    }
}

#[derive(Debug, Error)]
pub enum DurableChunkError {
    #[error("durable chunk has the wrong magic")]
    WrongMagic,
    #[error("durable chunk is truncated")]
    Truncated,
    #[error("durable chunk has trailing bytes")]
    TrailingBytes,
    #[error("durable chunk payload exceeds the bounded snapshot value limit")]
    PayloadTooLarge,
    #[error("durable chunk exceeds the section limit")]
    TooManySections,
    #[error("durable chunk exceeds the block-entity limit")]
    TooManyBlockEntities,
    #[error("durable chunk exceeds the structure-start limit")]
    TooManyStructureStarts,
    #[error("durable chunk exceeds the structure-reference limit")]
    TooManyStructureReferences,
    #[error("durable chunk section tag {0} is invalid")]
    InvalidSectionTag(u8),
    #[error("durable chunk light-state tag {0} is invalid")]
    InvalidLightStateTag(u8),
    #[error("durable chunk light-layer tag {0} is invalid")]
    InvalidLightLayerTag(u8),
    #[error("durable chunk has duplicate block-entity coordinates")]
    DuplicateBlockEntity,
    #[error("durable chunk resource identity exceeds the encoded limit")]
    IdentityTooLong,
    #[error("durable chunk contains an invalid resource identity")]
    InvalidIdentity,
    #[error(transparent)]
    Layout(#[from] crate::chunk::ChunkLayoutError),
    #[error(transparent)]
    Section(#[from] SectionError),
    #[error(transparent)]
    Structure(#[from] StructureStateError),
    #[error(transparent)]
    Projection(#[from] crate::projection::ChunkProjectionError),
    #[error(transparent)]
    Chunk(#[from] ChunkAccessError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_chunk(position: ChunkPos) -> ChunkColumn {
        ChunkColumn::new(
            position,
            ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).unwrap(),
                BlockStateId::new(0),
                BiomeId::new(0),
            ),
        )
    }

    #[test]
    fn structure_state_round_trips_in_current_chunk_format() {
        let position = ChunkPos::new(2, -3);
        let placement = StructurePlacement::new(
            ResourceId::new("ferrite", "waystone_ruin").unwrap(),
            position,
            StructureBounds::new(46, 70, -34, 49, 74, -31).unwrap(),
            91,
        );
        let mut chunk = empty_chunk(position);
        chunk
            .replace_structures(
                ChunkStructureState::v1(position, [placement.clone()], [placement.clone()])
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(decode_chunk(&encode_chunk(&chunk).unwrap()).unwrap(), chunk);
    }

    #[test]
    fn legacy_chunk_format_migrates_to_empty_versioned_structure_state() {
        let chunk = empty_chunk(ChunkPos::new(0, 0));
        let mut legacy = encode_chunk(&chunk).unwrap();
        legacy[3] = b'1';
        legacy.truncate(legacy.len() - 7);
        let decoded = decode_chunk(&legacy).unwrap();
        assert_eq!(decoded.structures(), &ChunkStructureState::empty());
        assert_eq!(decoded, chunk);
    }

    #[test]
    fn structure_format_migrates_without_synthesizing_light() {
        let chunk = empty_chunk(ChunkPos::new(0, 0));
        let mut legacy = encode_chunk(&chunk).unwrap();
        legacy[3] = b'2';
        legacy.truncate(legacy.len() - 1);
        let decoded = decode_chunk(&legacy).unwrap();
        assert_eq!(decoded.structures(), chunk.structures());
        assert!(decoded.light().is_none());
    }

    #[test]
    fn authoritative_light_round_trips_in_current_chunk_format() {
        let mut chunk = empty_chunk(ChunkPos::new(1, 2));
        let count = usize::from(chunk.layout().sections().count()) + 2;
        let mut sky = vec![LightLayer::Empty; count];
        sky[1] = LightLayer::full_brightness();
        chunk
            .replace_light(
                ChunkLightState::new(
                    sky,
                    vec![LightLayer::Empty; count],
                    chunk.layout().sections().count(),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(decode_chunk(&encode_chunk(&chunk).unwrap()).unwrap(), chunk);
    }
}
