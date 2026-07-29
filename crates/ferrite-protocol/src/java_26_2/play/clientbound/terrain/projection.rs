use std::collections::{BTreeMap, BTreeSet, VecDeque};

use thiserror::Error;

use crate::java_26_2::play::clientbound::terrain::packet::{
    BlockEntityData, ChunkCoordinate, FullChunk, HeightmapType, LightData, LightLayerUpdate,
    SectionData, TerrainPacket,
};

const DEFAULT_TRACKED_POSITIONS: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainProjection {
    center: Option<ChunkCoordinate>,
    cache_radius: i32,
    simulation_distance: i32,
    chunks: BTreeMap<ChunkCoordinate, FullChunk>,
    lights: BTreeMap<ChunkCoordinate, LightData>,
    lighting_enabled: BTreeSet<ChunkCoordinate>,
    dirty_chunks: BTreeSet<ChunkCoordinate>,
    dirty_light_sections: BTreeSet<LightSectionCoordinate>,
    biome_notifications: VecDeque<ChunkCoordinate>,
    recomputed_heightmaps: BTreeMap<ChunkCoordinate, BTreeSet<HeightmapType>>,
    block_entity_types: BTreeMap<i32, i32>,
    minimum_section_y: i32,
    maximum_tracked_positions: usize,
    batch_started: bool,
    last_finished_size: Option<i32>,
}

impl TerrainProjection {
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_TRACKED_POSITIONS, 0)
            .expect("the default terrain projection capacity is nonzero")
    }

    pub fn with_capacity(
        maximum_tracked_positions: usize,
        minimum_section_y: i32,
    ) -> Result<Self, TerrainProjectionError> {
        if maximum_tracked_positions == 0 {
            return Err(TerrainProjectionError::ZeroCapacity);
        }
        Ok(Self {
            center: None,
            cache_radius: 0,
            simulation_distance: 0,
            chunks: BTreeMap::new(),
            lights: BTreeMap::new(),
            lighting_enabled: BTreeSet::new(),
            dirty_chunks: BTreeSet::new(),
            dirty_light_sections: BTreeSet::new(),
            biome_notifications: VecDeque::new(),
            recomputed_heightmaps: BTreeMap::new(),
            block_entity_types: BTreeMap::new(),
            minimum_section_y,
            maximum_tracked_positions,
            batch_started: false,
            last_finished_size: None,
        })
    }

    pub fn register_block_entity_type(
        &mut self,
        block_state_raw_id: i32,
        block_entity_type_raw_id: i32,
    ) -> Result<(), TerrainProjectionError> {
        if !self.block_entity_types.contains_key(&block_state_raw_id)
            && self.block_entity_types.len() == self.maximum_tracked_positions
        {
            return Err(TerrainProjectionError::Capacity {
                collection: "block entity state mappings",
                maximum: self.maximum_tracked_positions,
            });
        }
        self.block_entity_types
            .insert(block_state_raw_id, block_entity_type_raw_id);
        Ok(())
    }

    pub fn apply(&mut self, packet: TerrainPacket) -> Result<(), TerrainProjectionError> {
        match packet {
            TerrainPacket::BundleDelimiter => {}
            TerrainPacket::ChunkBatchStart => self.batch_started = true,
            TerrainPacket::ChunkBatchFinished(size) => {
                self.batch_started = false;
                self.last_finished_size = Some(size);
            }
            TerrainPacket::ChunksBiomes(chunks) => {
                for update in chunks {
                    self.notify_biomes(update.position)?;
                    self.mark_neighborhood_dirty(update.position)?;
                    if let Some(chunk) = self.chunks.get_mut(&update.position) {
                        if update.sections.len() != chunk.sections.len() {
                            return Err(TerrainProjectionError::BiomeSectionCount {
                                expected: chunk.sections.len(),
                                actual: update.sections.len(),
                            });
                        }
                        for (section, biomes) in chunk.sections.iter_mut().zip(update.sections) {
                            section.biomes = biomes;
                        }
                    }
                }
            }
            TerrainPacket::ForgetLevelChunk(position) => self.forget(position)?,
            TerrainPacket::LevelChunkWithLight(mut chunk) => {
                let position = chunk.position;
                self.apply_light(position, &chunk.light)?;
                self.enable_lighting(position)?;
                if self.contains(position) {
                    self.normalize_chunk(&mut chunk)?;
                    self.insert_chunk(position, chunk)?;
                    self.mark_neighborhood_dirty(position)?;
                }
            }
            TerrainPacket::LightUpdate(update) => {
                self.apply_light(update.position, &update.light)?;
                self.enable_lighting(update.position)?;
                self.mark_touched_light_sections(update.position, &update.light)?;
            }
            TerrainPacket::SetChunkCacheCenter(center) => {
                self.center = Some(center);
                self.retain_visible();
            }
            TerrainPacket::SetChunkCacheRadius(radius) => {
                self.cache_radius = radius.max(2).wrapping_add(3);
                self.retain_visible();
            }
            TerrainPacket::SetSimulationDistance(distance) => {
                self.simulation_distance = distance;
            }
        }
        Ok(())
    }

    #[must_use]
    pub const fn center(&self) -> Option<ChunkCoordinate> {
        self.center
    }

    #[must_use]
    pub const fn cache_radius(&self) -> i32 {
        self.cache_radius
    }

    #[must_use]
    pub const fn simulation_distance(&self) -> i32 {
        self.simulation_distance
    }

    #[must_use]
    pub fn chunk(&self, position: ChunkCoordinate) -> Option<&FullChunk> {
        self.chunks.get(&position)
    }

    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    #[must_use]
    pub fn light(&self, position: ChunkCoordinate) -> Option<&LightData> {
        self.lights.get(&position)
    }

    #[must_use]
    pub fn lighting_enabled(&self, position: ChunkCoordinate) -> bool {
        self.lighting_enabled.contains(&position)
    }

    #[must_use]
    pub fn dirty_chunks(&self) -> &BTreeSet<ChunkCoordinate> {
        &self.dirty_chunks
    }

    pub fn clear_dirty_chunks(&mut self) {
        self.dirty_chunks.clear();
    }

    #[must_use]
    pub fn dirty_light_sections(&self) -> &BTreeSet<LightSectionCoordinate> {
        &self.dirty_light_sections
    }

    pub fn clear_dirty_light_sections(&mut self) {
        self.dirty_light_sections.clear();
    }

    pub fn take_biome_notifications(&mut self) -> Vec<ChunkCoordinate> {
        self.biome_notifications.drain(..).collect()
    }

    #[must_use]
    pub fn recomputed_heightmaps(
        &self,
        position: ChunkCoordinate,
    ) -> Option<&BTreeSet<HeightmapType>> {
        self.recomputed_heightmaps.get(&position)
    }

    #[must_use]
    pub const fn batch_started(&self) -> bool {
        self.batch_started
    }

    #[must_use]
    pub const fn last_finished_size(&self) -> Option<i32> {
        self.last_finished_size
    }

    fn normalize_chunk(&mut self, chunk: &mut FullChunk) -> Result<(), TerrainProjectionError> {
        let expected_heightmap_longs = expected_heightmap_longs(chunk.sections.len());
        let mismatched = chunk
            .heightmaps
            .iter()
            .filter_map(|(kind, values)| {
                (values.len() != expected_heightmap_longs).then_some(*kind)
            })
            .collect::<BTreeSet<_>>();
        if !mismatched.is_empty() {
            self.insert_recomputed_heightmaps(chunk.position, mismatched)?;
        }
        if !self.block_entity_types.is_empty() {
            let minimum_section_y = self.minimum_section_y;
            let mappings = &self.block_entity_types;
            let sections = &chunk.sections;
            chunk.block_entities.retain(|entity| {
                block_entity_matches(sections, entity, minimum_section_y, mappings)
            });
        }
        Ok(())
    }

    fn apply_light(
        &mut self,
        position: ChunkCoordinate,
        update: &LightData,
    ) -> Result<(), TerrainProjectionError> {
        if let Some(light) = self.lights.get_mut(&position) {
            merge_light_layers(&mut light.sky, &update.sky)?;
            merge_light_layers(&mut light.block, &update.block)?;
            return Ok(());
        }
        self.require_position_capacity(&self.lights, position, "chunk light")?;
        self.lights.insert(position, update.clone());
        Ok(())
    }

    fn enable_lighting(&mut self, position: ChunkCoordinate) -> Result<(), TerrainProjectionError> {
        insert_bounded(
            &mut self.lighting_enabled,
            position,
            self.maximum_tracked_positions,
            "lighting-enabled chunks",
        )
    }

    fn insert_chunk(
        &mut self,
        position: ChunkCoordinate,
        chunk: FullChunk,
    ) -> Result<(), TerrainProjectionError> {
        self.require_position_capacity(&self.chunks, position, "chunks")?;
        self.chunks.insert(position, chunk);
        Ok(())
    }

    fn insert_recomputed_heightmaps(
        &mut self,
        position: ChunkCoordinate,
        kinds: BTreeSet<HeightmapType>,
    ) -> Result<(), TerrainProjectionError> {
        self.require_position_capacity(&self.recomputed_heightmaps, position, "heightmap repairs")?;
        self.recomputed_heightmaps.insert(position, kinds);
        Ok(())
    }

    fn notify_biomes(&mut self, position: ChunkCoordinate) -> Result<(), TerrainProjectionError> {
        if self.biome_notifications.len() == self.maximum_tracked_positions {
            return Err(TerrainProjectionError::Capacity {
                collection: "biome notifications",
                maximum: self.maximum_tracked_positions,
            });
        }
        self.biome_notifications.push_back(position);
        Ok(())
    }

    fn mark_neighborhood_dirty(
        &mut self,
        position: ChunkCoordinate,
    ) -> Result<(), TerrainProjectionError> {
        for delta_x in -1i32..=1 {
            for delta_z in -1i32..=1 {
                let neighbor = ChunkCoordinate {
                    x: position.x.wrapping_add(delta_x),
                    z: position.z.wrapping_add(delta_z),
                };
                insert_bounded(
                    &mut self.dirty_chunks,
                    neighbor,
                    self.maximum_tracked_positions,
                    "dirty chunks",
                )?;
            }
        }
        Ok(())
    }

    fn mark_touched_light_sections(
        &mut self,
        position: ChunkCoordinate,
        light: &LightData,
    ) -> Result<(), TerrainProjectionError> {
        let layer_count = light.sky.len().max(light.block.len());
        for layer in 0..layer_count {
            let sky_changed = light
                .sky
                .get(layer)
                .is_some_and(|value| !matches!(value, LightLayerUpdate::Unchanged));
            let block_changed = light
                .block
                .get(layer)
                .is_some_and(|value| !matches!(value, LightLayerUpdate::Unchanged));
            if !sky_changed && !block_changed {
                continue;
            }
            let section_y = self
                .minimum_section_y
                .wrapping_sub(1)
                .wrapping_add(layer as i32);
            for delta_x in -1i32..=1 {
                for delta_y in -1i32..=1 {
                    for delta_z in -1i32..=1 {
                        let section = LightSectionCoordinate {
                            x: position.x.wrapping_add(delta_x),
                            y: section_y.wrapping_add(delta_y),
                            z: position.z.wrapping_add(delta_z),
                        };
                        if !self.dirty_light_sections.contains(&section)
                            && self.dirty_light_sections.len() == self.maximum_tracked_positions
                        {
                            return Err(TerrainProjectionError::Capacity {
                                collection: "dirty light sections",
                                maximum: self.maximum_tracked_positions,
                            });
                        }
                        self.dirty_light_sections.insert(section);
                    }
                }
            }
        }
        Ok(())
    }

    fn forget(&mut self, position: ChunkCoordinate) -> Result<(), TerrainProjectionError> {
        self.chunks.remove(&position);
        self.lights.remove(&position);
        self.lighting_enabled.remove(&position);
        self.recomputed_heightmaps.remove(&position);
        self.mark_neighborhood_dirty(position)
    }

    fn retain_visible(&mut self) {
        let Some(center) = self.center else {
            self.chunks.clear();
            self.recomputed_heightmaps.clear();
            return;
        };
        let radius = self.cache_radius;
        self.chunks
            .retain(|position, _| is_within_radius(center, *position, radius));
        self.recomputed_heightmaps
            .retain(|position, _| is_within_radius(center, *position, radius));
    }

    fn contains(&self, position: ChunkCoordinate) -> bool {
        self.center
            .is_some_and(|center| is_within_radius(center, position, self.cache_radius))
    }

    fn require_position_capacity<T>(
        &self,
        values: &BTreeMap<ChunkCoordinate, T>,
        position: ChunkCoordinate,
        collection: &'static str,
    ) -> Result<(), TerrainProjectionError> {
        if !values.contains_key(&position) && values.len() == self.maximum_tracked_positions {
            Err(TerrainProjectionError::Capacity {
                collection,
                maximum: self.maximum_tracked_positions,
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LightSectionCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for TerrainProjection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TerrainProjectionError {
    #[error("terrain projection capacity must be nonzero")]
    ZeroCapacity,
    #[error("{collection} reached its {maximum}-entry capacity")]
    Capacity {
        collection: &'static str,
        maximum: usize,
    },
    #[error("biome refresh has {actual} sections, expected {expected}")]
    BiomeSectionCount { expected: usize, actual: usize },
    #[error("light update has {actual} layers, expected {expected}")]
    LightLayerCount { expected: usize, actual: usize },
}

fn insert_bounded(
    values: &mut BTreeSet<ChunkCoordinate>,
    position: ChunkCoordinate,
    maximum: usize,
    collection: &'static str,
) -> Result<(), TerrainProjectionError> {
    if !values.contains(&position) && values.len() == maximum {
        return Err(TerrainProjectionError::Capacity {
            collection,
            maximum,
        });
    }
    values.insert(position);
    Ok(())
}

fn merge_light_layers(
    current: &mut [LightLayerUpdate],
    update: &[LightLayerUpdate],
) -> Result<(), TerrainProjectionError> {
    if current.len() != update.len() {
        return Err(TerrainProjectionError::LightLayerCount {
            expected: current.len(),
            actual: update.len(),
        });
    }
    for (current_layer, update_layer) in current.iter_mut().zip(update) {
        if !matches!(update_layer, LightLayerUpdate::Unchanged) {
            *current_layer = update_layer.clone();
        }
    }
    Ok(())
}

fn expected_heightmap_longs(section_count: usize) -> usize {
    let height = section_count.saturating_mul(16).saturating_add(1);
    let bits = usize::BITS as usize - height.saturating_sub(1).leading_zeros() as usize;
    256usize.saturating_mul(bits).div_ceil(64)
}

fn block_entity_matches(
    sections: &[SectionData],
    entity: &BlockEntityData,
    minimum_section_y: i32,
    mappings: &BTreeMap<i32, i32>,
) -> bool {
    let packed = entity.packed_local_xz as u8;
    let local_x = usize::from(packed >> 4);
    let local_z = usize::from(packed & 15);
    let y = i32::from(entity.y);
    let section_index = y.div_euclid(16).wrapping_sub(minimum_section_y);
    let Ok(section_index) = usize::try_from(section_index) else {
        return false;
    };
    let Some(section) = sections.get(section_index) else {
        return false;
    };
    let local_y = y.rem_euclid(16) as usize;
    let state_index = (local_y << 8) | (local_z << 4) | local_x;
    let Some(state) = section.block_states.get(state_index) else {
        return false;
    };
    mappings.get(state).copied() == Some(entity.type_raw_id)
}

fn is_within_radius(center: ChunkCoordinate, position: ChunkCoordinate, radius: i32) -> bool {
    let delta_x = position.x.wrapping_sub(center.x).wrapping_abs();
    let delta_z = position.z.wrapping_sub(center.z).wrapping_abs();
    delta_x.max(delta_z) <= radius
}
