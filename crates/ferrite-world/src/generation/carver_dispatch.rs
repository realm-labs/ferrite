//! Ordered `17×17` configured-carver dispatch and legacy stream resets.

use thiserror::Error;

use crate::generation::carver_path::{
    CanyonPathConfig, CarverPathError, CarverVolume, CavePathConfig, generate_canyon_path,
    generate_cave_path,
};
use crate::generation::feature::provider::HeightContext;
use crate::generation::feature::random::{GenerationRandom, LegacyRandom};

#[derive(Debug, Clone, PartialEq)]
pub enum ConfiguredCarver {
    Cave(CavePathConfig),
    Canyon(CanyonPathConfig),
}

impl ConfiguredCarver {
    fn probability(&self) -> f32 {
        match self {
            Self::Cave(config) => config.probability,
            Self::Canyon(config) => config.probability,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarverAttempt {
    pub source_chunk: [i32; 2],
    pub list_index: u32,
    pub seed: i64,
    pub start_roll: f32,
    pub started: bool,
    pub debug_void_skipped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarverVolumeOrigin {
    pub source_chunk: [i32; 2],
    pub list_index: u32,
}

pub fn apply_carver_sources(
    world_seed: i64,
    height: HeightContext,
    target_chunk: [i32; 2],
    debug_void_target: bool,
    mut carvers_for_source: impl FnMut([i32; 2]) -> Vec<ConfiguredCarver>,
    mut observe_attempt: impl FnMut(CarverAttempt),
    mut emit: impl FnMut(CarverVolumeOrigin, CarverVolume),
) -> Result<(), CarverDispatchError> {
    for x_offset in -8_i32..=8 {
        for z_offset in -8_i32..=8 {
            let source_chunk = [
                target_chunk[0]
                    .checked_add(x_offset)
                    .ok_or(CarverDispatchError::ChunkOverflow)?,
                target_chunk[1]
                    .checked_add(z_offset)
                    .ok_or(CarverDispatchError::ChunkOverflow)?,
            ];
            let configured = carvers_for_source(source_chunk);
            for (index, carver) in configured.iter().enumerate() {
                let list_index =
                    u32::try_from(index).map_err(|_| CarverDispatchError::TooManyCarvers)?;
                let index =
                    i64::try_from(index).map_err(|_| CarverDispatchError::TooManyCarvers)?;
                let seed = world_seed.wrapping_add(index);
                let mut random = LegacyRandom::new(0);
                random.set_large_feature_seed(seed, source_chunk[0], source_chunk[1]);
                let start_roll = random.next_f32();
                let started = start_roll <= carver.probability();
                let debug_void_skipped = started && debug_void_target;
                observe_attempt(CarverAttempt {
                    source_chunk,
                    list_index,
                    seed,
                    start_roll,
                    started,
                    debug_void_skipped,
                });
                if !started || debug_void_target {
                    continue;
                }
                let origin = CarverVolumeOrigin {
                    source_chunk,
                    list_index,
                };
                match carver {
                    ConfiguredCarver::Cave(config) => generate_cave_path(
                        &mut random,
                        height,
                        source_chunk,
                        target_chunk,
                        config,
                        |volume| emit(origin, volume),
                    )?,
                    ConfiguredCarver::Canyon(config) => generate_canyon_path(
                        &mut random,
                        height,
                        source_chunk,
                        target_chunk,
                        config,
                        |volume| emit(origin, volume),
                    )?,
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CarverDispatchError {
    #[error(transparent)]
    Path(#[from] CarverPathError),
    #[error("carver source chunk coordinate overflow")]
    ChunkOverflow,
    #[error("configured carver list cannot be indexed by a Java integer")]
    TooManyCarvers,
}
