//! Bounded stable encoding for Region-owned chunk columns.

use std::collections::BTreeMap;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

use crate::chunk::{ChunkAccessError, ChunkColumn, ChunkLayout, VerticalSectionRange};
use crate::id::{BiomeId, BlockStateId};
use crate::section::{BIOMES_PER_SECTION, BLOCKS_PER_SECTION, ChunkSection, SectionError};

const CHUNK_MAGIC: &[u8; 4] = b"FWC1";
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
    output.extend_from_slice(CHUNK_MAGIC);
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
    cursor.expect(CHUNK_MAGIC)?;
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
    cursor.finish()?;
    ChunkColumn::from_durable_parts(position, layout, sections, block_entities, revision)
        .map_err(Into::into)
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

    fn expect(&mut self, expected: &[u8]) -> Result<(), DurableChunkError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(DurableChunkError::WrongMagic)
        }
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
    #[error("durable chunk section tag {0} is invalid")]
    InvalidSectionTag(u8),
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
    Chunk(#[from] ChunkAccessError),
}
