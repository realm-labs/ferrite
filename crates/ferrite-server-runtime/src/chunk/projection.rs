use std::collections::BTreeMap;

use ferrite_foundation::resource::ResourceId;
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::terrain::packet::{
    BlockEntityData, ChunkCoordinate, FullChunk, HeightmapType, LightData, LightLayerUpdate,
    SectionData, TerrainPacket,
};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::projection::{ChunkSnapshot, ClientHeightmap, LightLayer};
use thiserror::Error;

use crate::chunk::stream::ChunkStreamEvent;

const MAX_BLOCK_STATE_RAW_ID: i32 = 32_365;
const MAX_BLOCK_ENTITY_RAW_ID: i32 = 48;

#[derive(Debug, Clone)]
pub struct JavaTerrainRegistryMap {
    maximum_entries: usize,
    air: BlockStateId,
    block_states: BTreeMap<BlockStateId, i32>,
    biomes: BTreeMap<BiomeId, i32>,
    block_entities: BTreeMap<ResourceId, i32>,
}

impl JavaTerrainRegistryMap {
    pub fn new(maximum_entries: usize, air: BlockStateId) -> Result<Self, TerrainProjectionError> {
        if maximum_entries == 0 {
            return Err(TerrainProjectionError::ZeroRegistryCapacity);
        }
        Ok(Self {
            maximum_entries,
            air,
            block_states: BTreeMap::new(),
            biomes: BTreeMap::new(),
            block_entities: BTreeMap::new(),
        })
    }

    pub fn insert_block_state(
        &mut self,
        state: BlockStateId,
        raw_id: i32,
    ) -> Result<(), TerrainProjectionError> {
        if !(0..=MAX_BLOCK_STATE_RAW_ID).contains(&raw_id) {
            return Err(TerrainProjectionError::BlockStateRawId { raw_id });
        }
        insert_bounded(&mut self.block_states, state, raw_id, self.maximum_entries)
    }

    pub fn insert_biome(
        &mut self,
        biome: BiomeId,
        raw_id: i32,
    ) -> Result<(), TerrainProjectionError> {
        if raw_id < 0 {
            return Err(TerrainProjectionError::BiomeRawId { raw_id });
        }
        insert_bounded(&mut self.biomes, biome, raw_id, self.maximum_entries)
    }

    pub fn insert_block_entity(
        &mut self,
        kind: ResourceId,
        raw_id: i32,
    ) -> Result<(), TerrainProjectionError> {
        if !(0..=MAX_BLOCK_ENTITY_RAW_ID).contains(&raw_id) {
            return Err(TerrainProjectionError::BlockEntityRawId { raw_id });
        }
        insert_bounded(&mut self.block_entities, kind, raw_id, self.maximum_entries)
    }

    fn block_state(&self, state: BlockStateId) -> Result<i32, TerrainProjectionError> {
        self.block_states
            .get(&state)
            .copied()
            .ok_or(TerrainProjectionError::UnmappedBlockState(state))
    }

    fn biome(&self, biome: BiomeId) -> Result<i32, TerrainProjectionError> {
        self.biomes
            .get(&biome)
            .copied()
            .ok_or(TerrainProjectionError::UnmappedBiome(biome))
    }

    fn block_entity(&self, kind: &ResourceId) -> Result<i32, TerrainProjectionError> {
        self.block_entities
            .get(kind)
            .copied()
            .ok_or_else(|| TerrainProjectionError::UnmappedBlockEntity(kind.clone()))
    }
}

pub fn project_chunk(
    snapshot: &ChunkSnapshot,
    registries: &JavaTerrainRegistryMap,
) -> Result<FullChunk, TerrainProjectionError> {
    let mut sections = Vec::with_capacity(snapshot.sections().len());
    for section in snapshot.sections() {
        let block_states = section
            .blocks()
            .values()
            .map(|state| registries.block_state(state))
            .collect::<Result<Vec<_>, _>>()?;
        let biomes = section
            .biomes()
            .values()
            .map(|biome| registries.biome(biome))
            .collect::<Result<Vec<_>, _>>()?;
        let non_empty_blocks = i16::try_from(
            section
                .blocks()
                .values()
                .filter(|state| *state != registries.air)
                .count(),
        )
        .expect("a section contains at most 4096 blocks");
        sections.push(SectionData {
            non_empty_blocks,
            fluid_count: 0,
            block_states,
            biomes,
        });
    }

    let layout = snapshot.layout();
    let minimum_y = layout.sections().minimum() * 16;
    let height = i32::from(layout.sections().count()) * 16;
    let heightmaps = snapshot
        .heightmaps()
        .iter()
        .map(|(kind, values)| {
            Ok((
                map_heightmap(*kind),
                pack_heightmap(values.as_ref(), minimum_y, height)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, TerrainProjectionError>>()?;

    let mut block_entities = Vec::with_capacity(snapshot.block_entities().len());
    for entity in snapshot.block_entities() {
        let local = entity.position.local();
        let packed_local_xz = ((local.x() << 4) | local.z()) as i8;
        let y =
            i16::try_from(entity.position.y).map_err(|_| TerrainProjectionError::BlockEntityY {
                y: entity.position.y,
            })?;
        block_entities.push(BlockEntityData {
            packed_local_xz,
            y,
            type_raw_id: registries.block_entity(&entity.kind)?,
            update_tag: None,
        });
    }

    Ok(FullChunk {
        position: ChunkCoordinate {
            x: snapshot.position().x,
            z: snapshot.position().z,
        },
        heightmaps,
        sections,
        block_entities,
        light: LightData {
            sky: snapshot.light().sky().iter().map(project_light).collect(),
            block: snapshot.light().block().iter().map(project_light).collect(),
        },
    })
}

pub fn project_stream_events(
    events: Vec<ChunkStreamEvent>,
    registries: &JavaTerrainRegistryMap,
) -> Result<Vec<PlayClientboundPacket>, TerrainProjectionError> {
    events
        .into_iter()
        .map(|event| {
            let packet = match event {
                ChunkStreamEvent::SetCenter(position) => {
                    TerrainPacket::SetChunkCacheCenter(ChunkCoordinate {
                        x: position.x,
                        z: position.z,
                    })
                }
                ChunkStreamEvent::SetViewDistance(distance) => {
                    TerrainPacket::SetChunkCacheRadius(i32::from(distance))
                }
                ChunkStreamEvent::SetSimulationDistance(distance) => {
                    TerrainPacket::SetSimulationDistance(i32::from(distance))
                }
                ChunkStreamEvent::Forget(position) => {
                    TerrainPacket::ForgetLevelChunk(ChunkCoordinate {
                        x: position.x,
                        z: position.z,
                    })
                }
                ChunkStreamEvent::BatchStart => TerrainPacket::ChunkBatchStart,
                ChunkStreamEvent::Chunk(snapshot) => {
                    TerrainPacket::LevelChunkWithLight(project_chunk(&snapshot, registries)?)
                }
                ChunkStreamEvent::BatchFinish { chunks } => {
                    let chunks = i32::try_from(chunks)
                        .map_err(|_| TerrainProjectionError::BatchSize { chunks })?;
                    TerrainPacket::ChunkBatchFinished(chunks)
                }
            };
            Ok(PlayClientboundPacket::Terrain(packet))
        })
        .collect()
}

fn insert_bounded<K: Ord>(
    entries: &mut BTreeMap<K, i32>,
    key: K,
    raw_id: i32,
    maximum: usize,
) -> Result<(), TerrainProjectionError> {
    if let Some(current) = entries.get(&key).copied() {
        return if current == raw_id {
            Ok(())
        } else {
            Err(TerrainProjectionError::EntryRemap {
                current,
                requested: raw_id,
            })
        };
    }
    if entries.len() == maximum {
        return Err(TerrainProjectionError::RegistryFull { maximum });
    }
    if entries.values().any(|candidate| *candidate == raw_id) {
        return Err(TerrainProjectionError::DuplicateRawId { raw_id });
    }
    entries.insert(key, raw_id);
    Ok(())
}

fn map_heightmap(kind: ClientHeightmap) -> HeightmapType {
    match kind {
        ClientHeightmap::WorldSurface => HeightmapType::WorldSurface,
        ClientHeightmap::MotionBlocking => HeightmapType::MotionBlocking,
        ClientHeightmap::MotionBlockingNoLeaves => HeightmapType::MotionBlockingNoLeaves,
    }
}

fn pack_heightmap(
    values: &[i32; 256],
    minimum_y: i32,
    height: i32,
) -> Result<Vec<i64>, TerrainProjectionError> {
    let bits = bits_for(height as u32);
    let per_long = 64 / usize::from(bits);
    let mut packed = vec![0u64; values.len().div_ceil(per_long)];
    let mask = (1u64 << bits) - 1;
    for (index, value) in values.iter().enumerate() {
        let relative = value
            .checked_sub(minimum_y)
            .ok_or(TerrainProjectionError::HeightmapValue { value: *value })?;
        if !(0..=height).contains(&relative) {
            return Err(TerrainProjectionError::HeightmapValue { value: *value });
        }
        let word = index / per_long;
        let shift = (index % per_long) * usize::from(bits);
        packed[word] |= (relative as u64 & mask) << shift;
    }
    Ok(packed.into_iter().map(|word| word as i64).collect())
}

fn project_light(layer: &LightLayer) -> LightLayerUpdate {
    match layer {
        LightLayer::Empty => LightLayerUpdate::Empty,
        LightLayer::Data(data) => LightLayerUpdate::Data(data.clone()),
    }
}

const fn bits_for(value: u32) -> u8 {
    let bits = u32::BITS - value.leading_zeros();
    if bits == 0 { 1 } else { bits as u8 }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TerrainProjectionError {
    #[error("terrain registry-map capacity cannot be zero")]
    ZeroRegistryCapacity,
    #[error("terrain registry map reached its {maximum}-entry bound")]
    RegistryFull { maximum: usize },
    #[error("wire raw ID {raw_id} is already mapped")]
    DuplicateRawId { raw_id: i32 },
    #[error("registry entry is already mapped to {current}, not requested ID {requested}")]
    EntryRemap { current: i32, requested: i32 },
    #[error("block-state raw ID {raw_id} is outside 0..=32365")]
    BlockStateRawId { raw_id: i32 },
    #[error("biome raw ID {raw_id} is negative")]
    BiomeRawId { raw_id: i32 },
    #[error("block-entity raw ID {raw_id} is outside 0..=48")]
    BlockEntityRawId { raw_id: i32 },
    #[error("block state {0:?} has no Java 26.2 projection")]
    UnmappedBlockState(BlockStateId),
    #[error("biome {0:?} has no Java 26.2 projection")]
    UnmappedBiome(BiomeId),
    #[error("block entity {0} has no Java 26.2 projection")]
    UnmappedBlockEntity(ResourceId),
    #[error("heightmap absolute value {value} is outside the dimension")]
    HeightmapValue { value: i32 },
    #[error("block entity Y {y} cannot be represented as a signed short")]
    BlockEntityY { y: i32 },
    #[error("chunk batch size {chunks} cannot be represented as a signed VarInt")]
    BatchSize { chunks: usize },
}
