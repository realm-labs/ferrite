//! Deterministic Ferrite overworld terrain and biome stages.

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::numeric::{NumericError, add_i32};
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

use crate::chunk::{ChunkAccessError, ChunkColumn};
use crate::generation::feature::random::LegacyRandom;
use crate::generation::noise::{NoiseParameters, NormalNoise};
use crate::generation::status::ChunkStatus;
use crate::generation::structure_state::{
    ChunkStructureState, StructureBounds, StructurePlacement, StructureStateError,
};
use crate::id::{BiomeId, BlockStateId};
use crate::section::BIOMES_PER_SECTION;

const MINIMUM_TERRAIN_Y: i32 = 48;
const MAXIMUM_TERRAIN_Y: i32 = 112;
const BASE_TERRAIN_Y: f64 = 70.0;
const STRUCTURE_SPACING_CHUNKS: i32 = 8;

#[derive(Debug, Clone)]
pub struct OverworldGeneratorV1 {
    height: NormalNoise,
    detail: NormalNoise,
    temperature: NormalNoise,
    humidity: NormalNoise,
    caves: NormalNoise,
    seed: i64,
    stone: BlockStateId,
    surface: BlockStateId,
    biomes: [BiomeId; 3],
}

impl OverworldGeneratorV1 {
    #[must_use]
    pub fn new(
        seed: i64,
        stone: BlockStateId,
        surface: BlockStateId,
        biomes: [BiomeId; 3],
    ) -> Self {
        Self {
            height: normal_noise(seed, b"height", -4, &[1.0, 1.0, 0.5, 0.25]),
            detail: normal_noise(seed, b"detail", -2, &[1.0, 0.5, 0.25]),
            temperature: normal_noise(seed, b"temperature", -3, &[1.0, 0.5, 0.25]),
            humidity: normal_noise(seed, b"humidity", -3, &[1.0, 0.5, 0.25]),
            caves: normal_noise(seed, b"caves", -2, &[1.0, 0.5, 0.25]),
            seed,
            stone,
            surface,
            biomes,
        }
    }

    pub fn apply_stage(
        &self,
        chunk: &mut ChunkColumn,
        target: ChunkStatus,
    ) -> Result<(), OverworldGenerationError> {
        match target {
            ChunkStatus::StructureStarts => self.create_structure_starts(chunk),
            ChunkStatus::StructureReferences => self.create_structure_references(chunk),
            ChunkStatus::Biomes => self.create_biomes(chunk),
            ChunkStatus::Noise => self.fill_noise(chunk),
            ChunkStatus::Surface => self.build_surface(chunk),
            ChunkStatus::Carvers => self.apply_carvers(chunk),
            ChunkStatus::Features => {
                self.decorate_features(chunk)?;
                self.place_structures(chunk)
            }
            ChunkStatus::Spawn => self.prepare_spawn(chunk),
            _ => Ok(()),
        }
    }

    fn create_structure_starts(
        &self,
        chunk: &mut ChunkColumn,
    ) -> Result<(), OverworldGenerationError> {
        let position = chunk.position();
        let starts = self
            .is_structure_start(position)
            .then(|| self.structure_placement(position))
            .transpose()?
            .into_iter();
        chunk.replace_structures(ChunkStructureState::v1(position, starts, [])?)?;
        Ok(())
    }

    fn create_structure_references(
        &self,
        chunk: &mut ChunkColumn,
    ) -> Result<(), OverworldGenerationError> {
        let position = chunk.position();
        let mut references = Vec::new();
        for offset_x in -1..=0 {
            for offset_z in -1..=0 {
                let candidate = ChunkPos::new(
                    add_i32(position.x, offset_x)?,
                    add_i32(position.z, offset_z)?,
                );
                if self.is_structure_start(candidate) {
                    let placement = self.structure_placement(candidate)?;
                    if placement.bounds.intersects_chunk(position) {
                        references.push(placement);
                    }
                }
            }
        }
        chunk.replace_structures(ChunkStructureState::v1(
            position,
            chunk.structures().starts().iter().cloned(),
            references,
        )?)?;
        Ok(())
    }

    fn create_biomes(&self, chunk: &mut ChunkColumn) -> Result<(), OverworldGenerationError> {
        let sections = chunk.layout().sections();
        let (origin_x, origin_z) = chunk_origin(chunk.position())?;
        for section_y in sections.minimum()..sections.maximum_exclusive() {
            let mut biomes = [self.biomes[0]; BIOMES_PER_SECTION];
            for quart_y in 0..4 {
                for quart_z in 0..4 {
                    for quart_x in 0..4 {
                        let block_x = origin_x + quart_x * 4;
                        let block_y = section_y * 16 + quart_y * 4;
                        let block_z = origin_z + quart_z * 4;
                        let temperature = self.temperature.sample(
                            f64::from(block_x) / 384.0,
                            f64::from(block_y) / 384.0,
                            f64::from(block_z) / 384.0,
                        );
                        let humidity = self.humidity.sample(
                            f64::from(block_x) / 320.0,
                            f64::from(block_y) / 320.0,
                            f64::from(block_z) / 320.0,
                        );
                        let biome = if temperature < -0.12 {
                            self.biomes[1]
                        } else if humidity > 0.18 {
                            self.biomes[2]
                        } else {
                            self.biomes[0]
                        };
                        let index = ((quart_y * 4 + quart_z) * 4 + quart_x) as usize;
                        biomes[index] = biome;
                    }
                }
            }
            chunk.set_section_biomes(section_y, biomes)?;
        }
        Ok(())
    }

    fn fill_noise(&self, chunk: &mut ChunkColumn) -> Result<(), OverworldGenerationError> {
        let sections = chunk.layout().sections();
        let minimum_y = sections.minimum() * 16;
        for section_y in sections.minimum()..=MINIMUM_TERRAIN_Y.div_euclid(16) - 1 {
            chunk.set_uniform_section_blocks(section_y, self.stone)?;
        }
        let (origin_x, origin_z) = chunk_origin(chunk.position())?;
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = origin_x + local_x;
                let z = origin_z + local_z;
                let height = self.surface_height(x, z);
                for y in MINIMUM_TERRAIN_Y.max(minimum_y)..=height {
                    chunk.set_block(BlockPos::new(x, y, z), self.stone)?;
                }
            }
        }
        Ok(())
    }

    fn build_surface(&self, chunk: &mut ChunkColumn) -> Result<(), OverworldGenerationError> {
        let (origin_x, origin_z) = chunk_origin(chunk.position())?;
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = origin_x + local_x;
                let z = origin_z + local_z;
                chunk.set_block(BlockPos::new(x, self.surface_height(x, z), z), self.surface)?;
            }
        }
        Ok(())
    }

    fn apply_carvers(&self, chunk: &mut ChunkColumn) -> Result<(), OverworldGenerationError> {
        let (origin_x, origin_z) = chunk_origin(chunk.position())?;
        let air = chunk.layout().default_block();
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = origin_x + local_x;
                let z = origin_z + local_z;
                let ceiling = self.surface_height(x, z) - 5;
                for y in 8..=ceiling {
                    let cave = self.caves.sample(
                        f64::from(x) / 42.0,
                        f64::from(y) / 34.0,
                        f64::from(z) / 42.0,
                    );
                    if cave.abs() > 0.32 {
                        chunk.set_block(BlockPos::new(x, y, z), air)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn decorate_features(&self, chunk: &mut ChunkColumn) -> Result<(), OverworldGenerationError> {
        let (origin_x, origin_z) = chunk_origin(chunk.position())?;
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = origin_x + local_x;
                let z = origin_z + local_z;
                if positional_hash(self.seed, x, z, b"surface-outcrop") & 63 == 0 {
                    let y = self.surface_height(x, z) + 1;
                    chunk.set_block(BlockPos::new(x, y, z), self.stone)?;
                }
            }
        }
        Ok(())
    }

    fn place_structures(&self, chunk: &mut ChunkColumn) -> Result<(), OverworldGenerationError> {
        let references = chunk.structures().references().to_vec();
        let (chunk_minimum_x, chunk_minimum_z) = chunk_origin(chunk.position())?;
        let chunk_maximum_x = add_i32(chunk_minimum_x, 15)?;
        let chunk_maximum_z = add_i32(chunk_minimum_z, 15)?;
        for placement in references {
            let bounds = placement.bounds;
            let minimum_x = bounds.minimum_x.max(chunk_minimum_x);
            let maximum_x = bounds.maximum_x.min(chunk_maximum_x);
            let minimum_z = bounds.minimum_z.max(chunk_minimum_z);
            let maximum_z = bounds.maximum_z.min(chunk_maximum_z);
            for x in minimum_x..=maximum_x {
                for z in minimum_z..=maximum_z {
                    let edge_x = x == bounds.minimum_x || x == bounds.maximum_x;
                    let edge_z = z == bounds.minimum_z || z == bounds.maximum_z;
                    for y in bounds.minimum_y..=bounds.maximum_y {
                        let corner_post = edge_x && edge_z;
                        let top_beam = y == bounds.maximum_y && (edge_x || edge_z);
                        if corner_post || top_beam {
                            chunk.set_block(BlockPos::new(x, y, z), self.stone)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn is_structure_start(&self, position: ChunkPos) -> bool {
        let cell_x = position.x.div_euclid(STRUCTURE_SPACING_CHUNKS);
        let cell_z = position.z.div_euclid(STRUCTURE_SPACING_CHUNKS);
        let hash = positional_hash(self.seed, cell_x, cell_z, b"waystone-start");
        let offset_x = (hash & 7) as i32;
        let offset_z = ((hash >> 3) & 7) as i32;
        i64::from(position.x)
            == i64::from(cell_x) * i64::from(STRUCTURE_SPACING_CHUNKS) + i64::from(offset_x)
            && i64::from(position.z)
                == i64::from(cell_z) * i64::from(STRUCTURE_SPACING_CHUNKS) + i64::from(offset_z)
    }

    fn structure_placement(
        &self,
        start_chunk: ChunkPos,
    ) -> Result<StructurePlacement, OverworldGenerationError> {
        let (origin_x, origin_z) = chunk_origin(start_chunk)?;
        let center_x = add_i32(origin_x, 15)?;
        let center_z = add_i32(origin_z, 15)?;
        let minimum_y = add_i32(self.surface_height(center_x, center_z), 1)?;
        let bounds = StructureBounds::new(
            add_i32(center_x, -1)?,
            minimum_y,
            add_i32(center_z, -1)?,
            add_i32(center_x, 2)?,
            add_i32(minimum_y, 3)?,
            add_i32(center_z, 2)?,
        )?;
        Ok(StructurePlacement::new(
            ResourceId::new("ferrite", "waystone_ruin").expect("static resource identity"),
            start_chunk,
            bounds,
            positional_hash(
                self.seed,
                start_chunk.x,
                start_chunk.z,
                b"waystone-placement",
            ),
        ))
    }

    fn prepare_spawn(&self, chunk: &mut ChunkColumn) -> Result<(), OverworldGenerationError> {
        let (origin_x, origin_z) = chunk_origin(chunk.position())?;
        let maximum_y = chunk.layout().sections().maximum_exclusive() * 16 - 1;
        for local_x in 0..16 {
            for local_z in 0..16 {
                let height = self.surface_height(origin_x + local_x, origin_z + local_z);
                if height >= maximum_y {
                    return Err(OverworldGenerationError::NoSpawnHeadroom);
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn surface_height(&self, x: i32, z: i32) -> i32 {
        let broad = self
            .height
            .sample(f64::from(x) / 192.0, 0.0, f64::from(z) / 192.0);
        let detail = self
            .detail
            .sample(f64::from(x) / 48.0, 0.0, f64::from(z) / 48.0);
        (BASE_TERRAIN_Y + broad * 24.0 + detail * 7.0)
            .round()
            .clamp(f64::from(MINIMUM_TERRAIN_Y), f64::from(MAXIMUM_TERRAIN_Y)) as i32
    }
}

fn normal_noise(
    seed: i64,
    stream: &'static [u8],
    first_octave: i32,
    amplitudes: &[f64],
) -> NormalNoise {
    NormalNoise::keyed(
        NoiseParameters {
            first_octave,
            amplitudes: amplitudes.to_vec(),
        },
        |lane, octave| LegacyRandom::new(named_stream_seed(seed, stream, lane, octave)),
    )
}

fn chunk_origin(position: ChunkPos) -> Result<(i32, i32), NumericError> {
    Ok((
        position.checked_min_block_x()?,
        position.checked_min_block_z()?,
    ))
}

fn named_stream_seed(seed: i64, stream: &[u8], lane: u8, octave: i32) -> i64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64 ^ seed as u64;
    for byte in stream
        .iter()
        .copied()
        .chain([lane])
        .chain(octave.to_be_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash as i64
}

fn positional_hash(seed: i64, x: i32, z: i32, stream: &[u8]) -> u64 {
    let mut value = seed as u64 ^ 0x9e37_79b9_7f4a_7c15;
    value ^= (x as u32 as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= (z as u32 as u64).wrapping_mul(0x94d0_49bb_1331_11eb);
    for byte in stream {
        value = (value ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    value ^ (value >> 31)
}

#[derive(Debug, Error)]
pub enum OverworldGenerationError {
    #[error(transparent)]
    Chunk(#[from] ChunkAccessError),
    #[error(transparent)]
    Numeric(#[from] NumericError),
    #[error(transparent)]
    Structure(#[from] StructureStateError),
    #[error("generated terrain leaves no vertical spawn headroom")]
    NoSpawnHeadroom,
}

#[cfg(test)]
mod tests {
    use crate::chunk::{ChunkLayout, VerticalSectionRange};

    use super::*;

    fn chunk(position: ferrite_foundation::coordinate::ChunkPos) -> ChunkColumn {
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
    fn same_seed_and_coordinate_are_identical_while_seed_changes_terrain() {
        let first = OverworldGeneratorV1::new(
            7,
            BlockStateId::new(1),
            BlockStateId::new(2),
            [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)],
        );
        let changed = OverworldGeneratorV1::new(
            8,
            BlockStateId::new(1),
            BlockStateId::new(2),
            [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)],
        );
        let mut left = chunk(ferrite_foundation::coordinate::ChunkPos::new(-2, 3));
        let mut right = left.clone();
        for status in [
            ChunkStatus::Biomes,
            ChunkStatus::Noise,
            ChunkStatus::Surface,
            ChunkStatus::Carvers,
            ChunkStatus::Features,
            ChunkStatus::Spawn,
        ] {
            first.apply_stage(&mut left, status).unwrap();
            first.apply_stage(&mut right, status).unwrap();
        }
        assert_eq!(left, right);
        assert_ne!(
            first.surface_height(-24, 56),
            changed.surface_height(-24, 56)
        );
    }

    #[test]
    fn terrain_crosses_chunk_edges_without_flat_fixture_height() {
        let generator = OverworldGeneratorV1::new(
            42,
            BlockStateId::new(1),
            BlockStateId::new(2),
            [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)],
        );
        let left = generator.surface_height(15, -3);
        let right = generator.surface_height(16, -3);
        assert!((left - right).abs() <= 2);
        let heights = (-64..=64)
            .step_by(8)
            .map(|x| generator.surface_height(x, 0))
            .collect::<std::collections::BTreeSet<_>>();
        assert!(heights.len() > 1);
    }

    #[test]
    fn carvers_features_and_spawn_preparation_mutate_only_bounded_generated_terrain() {
        let generator = OverworldGeneratorV1::new(
            42,
            BlockStateId::new(1),
            BlockStateId::new(2),
            [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)],
        );
        let mut chunk = chunk(ferrite_foundation::coordinate::ChunkPos::new(0, 0));
        for status in [
            ChunkStatus::Biomes,
            ChunkStatus::Noise,
            ChunkStatus::Surface,
            ChunkStatus::Carvers,
            ChunkStatus::Features,
            ChunkStatus::Spawn,
        ] {
            generator.apply_stage(&mut chunk, status).unwrap();
        }
        let mut caves = 0;
        let mut outcrops = 0;
        for x in 0..16 {
            for z in 0..16 {
                let height = generator.surface_height(x, z);
                caves += (8..=height - 5)
                    .filter(|y| {
                        chunk.block_state(BlockPos::new(x, *y, z)).unwrap() == BlockStateId::new(0)
                    })
                    .count();
                outcrops += usize::from(
                    chunk.block_state(BlockPos::new(x, height + 1, z)).unwrap()
                        == BlockStateId::new(1),
                );
            }
        }
        assert!(caves > 0);
        assert!(outcrops > 0);
    }

    #[test]
    fn structure_start_references_and_blocks_are_deterministic_across_chunk_edges() {
        let generator = OverworldGeneratorV1::new(
            42,
            BlockStateId::new(1),
            BlockStateId::new(2),
            [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)],
        );
        let start = (0..8)
            .flat_map(|x| (0..8).map(move |z| ChunkPos::new(x, z)))
            .find(|position| generator.is_structure_start(*position))
            .unwrap();
        let mut start_chunk = chunk(start);
        for status in ChunkStatus::ALL.iter().copied().skip(1).take(7) {
            generator.apply_stage(&mut start_chunk, status).unwrap();
        }
        assert_eq!(start_chunk.structures().starts().len(), 1);
        assert_eq!(start_chunk.structures().references().len(), 1);
        let placement = start_chunk.structures().starts()[0].clone();
        assert_eq!(
            start_chunk
                .block_state(BlockPos::new(
                    placement.bounds.minimum_x,
                    placement.bounds.maximum_y,
                    placement.bounds.minimum_z,
                ))
                .unwrap(),
            BlockStateId::new(1)
        );

        let neighbor_position = ChunkPos::new(start.x + 1, start.z);
        let mut neighbor = chunk(neighbor_position);
        generator
            .apply_stage(&mut neighbor, ChunkStatus::StructureStarts)
            .unwrap();
        generator
            .apply_stage(&mut neighbor, ChunkStatus::StructureReferences)
            .unwrap();
        assert!(
            neighbor
                .structures()
                .references()
                .iter()
                .any(|reference| reference.start_chunk == start)
        );
    }
}
