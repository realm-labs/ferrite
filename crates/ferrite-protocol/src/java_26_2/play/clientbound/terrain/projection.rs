use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::play::clientbound::terrain::packet::{
    ChunkCoordinate, FullChunk, TerrainPacket,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainProjection {
    center: Option<ChunkCoordinate>,
    cache_radius: i32,
    simulation_distance: i32,
    chunks: BTreeMap<ChunkCoordinate, FullChunk>,
    batch_started: bool,
    last_finished_size: Option<i32>,
}

impl TerrainProjection {
    #[must_use]
    pub fn new() -> Self {
        Self {
            center: None,
            cache_radius: 0,
            simulation_distance: 0,
            chunks: BTreeMap::new(),
            batch_started: false,
            last_finished_size: None,
        }
    }

    pub fn apply(&mut self, packet: TerrainPacket) -> Result<(), TerrainProjectionError> {
        match packet {
            TerrainPacket::BundleDelimiter | TerrainPacket::LightUpdate(_) => {}
            TerrainPacket::ChunkBatchStart => self.batch_started = true,
            TerrainPacket::ChunkBatchFinished(size) => {
                self.batch_started = false;
                self.last_finished_size = Some(size);
            }
            TerrainPacket::ChunksBiomes(chunks) => {
                for update in chunks {
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
            TerrainPacket::ForgetLevelChunk(position) => {
                self.chunks.remove(&position);
            }
            TerrainPacket::LevelChunkWithLight(chunk) => {
                if self.contains(chunk.position) {
                    self.chunks.insert(chunk.position, chunk);
                }
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
    pub const fn batch_started(&self) -> bool {
        self.batch_started
    }

    #[must_use]
    pub const fn last_finished_size(&self) -> Option<i32> {
        self.last_finished_size
    }

    fn retain_visible(&mut self) {
        let Some(center) = self.center else {
            self.chunks.clear();
            return;
        };
        let radius = self.cache_radius;
        self.chunks
            .retain(|position, _| is_within_radius(center, *position, radius));
    }

    fn contains(&self, position: ChunkCoordinate) -> bool {
        self.center
            .is_some_and(|center| is_within_radius(center, position, self.cache_radius))
    }
}

impl Default for TerrainProjection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TerrainProjectionError {
    #[error("biome refresh has {actual} sections, expected {expected}")]
    BiomeSectionCount { expected: usize, actual: usize },
}

fn is_within_radius(center: ChunkCoordinate, position: ChunkCoordinate, radius: i32) -> bool {
    let delta_x = i64::from(position.x) - i64::from(center.x);
    let delta_z = i64::from(position.z) - i64::from(center.z);
    delta_x.abs().max(delta_z.abs()) <= i64::from(radius)
}
