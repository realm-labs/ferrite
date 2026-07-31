//! Flat-generator layer preparation, base fill, queries, and share parsing.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::id::{BiomeId, BlockStateId};

pub const MAX_FLAT_HEIGHT: usize = 4_064;
pub const FLAT_PRESET_PARTITION_SHA256: &str =
    "49260ebde924b29055bed20f2ec94e1ba99d09b4a509622ae7a0b7a1a5459b5a";
pub const VISIBLE_FLAT_PRESETS: [&str; 9] = [
    "classic_flat",
    "tunnelers_dream",
    "water_world",
    "overworld",
    "snowy_kingdom",
    "bottomless_pit",
    "desert",
    "redstone_ready",
    "the_void",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatLayer {
    pub height: u16,
    pub state: BlockStateId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructureOverrides<T> {
    Absent,
    Present(Vec<T>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatSettings<T> {
    pub structure_overrides: StructureOverrides<T>,
    pub layers: Vec<FlatLayer>,
    pub lakes: bool,
    pub features: bool,
    pub biome: BiomeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFlatLayers {
    pub base_layers: Vec<Option<BlockStateId>>,
    pub decoration_layers: Vec<(usize, BlockStateId)>,
    pub void: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlatHeightmap {
    OceanFloorWorldGeneration,
    WorldSurfaceWorldGeneration,
}

pub trait FlatFillWorld {
    fn minimum_y(&self) -> i32;

    fn height(&self) -> usize;

    fn offer_flat_block(&mut self, position: BlockPos, state: BlockStateId);

    fn update_heightmap(
        &mut self,
        heightmap: FlatHeightmap,
        position: BlockPos,
        state: BlockStateId,
    );
}

impl<T> FlatSettings<T> {
    pub fn validate(&self) -> Result<(), FlatError> {
        let total = self
            .layers
            .iter()
            .try_fold(0_usize, |total, layer| {
                total.checked_add(usize::from(layer.height))
            })
            .ok_or(FlatError::LayerHeightOverflow)?;
        if total > MAX_FLAT_HEIGHT {
            Err(FlatError::LayerHeightOverflow)
        } else {
            Ok(())
        }
    }

    pub fn prepare_layers(
        &self,
        mut is_air: impl FnMut(BlockStateId) -> bool,
        mut is_motion_blocking: impl FnMut(BlockStateId) -> bool,
    ) -> Result<PreparedFlatLayers, FlatError> {
        self.validate()?;
        let mut expanded = Vec::new();
        for layer in &self.layers {
            expanded.extend(std::iter::repeat_n(layer.state, usize::from(layer.height)));
        }
        let void = expanded.iter().all(|state| is_air(*state));
        let mut base_layers = Vec::with_capacity(expanded.len());
        let mut decoration_layers = Vec::new();
        for (offset, state) in expanded.into_iter().enumerate() {
            if is_motion_blocking(state) {
                base_layers.push(Some(state));
            } else {
                base_layers.push(None);
                decoration_layers.push((offset, state));
            }
        }
        Ok(PreparedFlatLayers {
            base_layers,
            decoration_layers,
            void,
        })
    }

    pub fn with_edited_layers_and_biome(self, layers: Vec<FlatLayer>, biome: BiomeId) -> Self {
        Self {
            layers,
            biome,
            ..self
        }
    }
}

pub fn fill_flat_chunk(
    world: &mut impl FlatFillWorld,
    chunk_minimum_x: i32,
    chunk_minimum_z: i32,
    layers: &[Option<BlockStateId>],
) -> Result<(), FlatError> {
    let limit = world.height().min(layers.len());
    for (offset, state) in layers.iter().copied().take(limit).enumerate() {
        let Some(state) = state else {
            continue;
        };
        let y = world
            .minimum_y()
            .checked_add(i32::try_from(offset).map_err(|_| FlatError::PositionOverflow)?)
            .ok_or(FlatError::PositionOverflow)?;
        for local_x in 0..16 {
            for local_z in 0..16 {
                let position = BlockPos::new(
                    chunk_minimum_x
                        .checked_add(local_x)
                        .ok_or(FlatError::PositionOverflow)?,
                    y,
                    chunk_minimum_z
                        .checked_add(local_z)
                        .ok_or(FlatError::PositionOverflow)?,
                );
                world.offer_flat_block(position, state);
                world.update_heightmap(FlatHeightmap::OceanFloorWorldGeneration, position, state);
                world.update_heightmap(FlatHeightmap::WorldSurfaceWorldGeneration, position, state);
            }
        }
    }
    Ok(())
}

pub fn flat_spawn_height(minimum_y: i32, height: usize, layers: &[Option<BlockStateId>]) -> i32 {
    minimum_y.saturating_add(height.min(layers.len()) as i32)
}

pub fn flat_base_height(
    minimum_y: i32,
    accessor_height: usize,
    layers: &[Option<BlockStateId>],
    mut opaque_for_heightmap: impl FnMut(BlockStateId) -> bool,
) -> i32 {
    let maximum_index = accessor_height
        .saturating_sub(1)
        .min(layers.len().saturating_sub(1));
    if layers.is_empty() || accessor_height == 0 {
        return minimum_y;
    }
    for offset in (0..=maximum_index).rev() {
        if layers[offset].is_some_and(&mut opaque_for_heightmap) {
            return minimum_y.saturating_add(offset as i32 + 1);
        }
    }
    minimum_y
}

pub fn flat_base_column(
    accessor_height: usize,
    layers: &[Option<BlockStateId>],
    air: BlockStateId,
) -> Vec<BlockStateId> {
    layers
        .iter()
        .take(accessor_height)
        .map(|state| state.unwrap_or(air))
        .collect()
}

pub trait FlatShareResolver {
    fn block_state(&self, identifier: &str) -> Option<BlockStateId>;

    fn block_identifier(&self, state: BlockStateId) -> Option<&str>;

    fn biome(&self, identifier: &str) -> Option<BiomeId>;

    fn biome_identifier(&self, biome: BiomeId) -> Option<&str>;
}

pub fn parse_flat_share<T: Clone>(
    input: &str,
    selected: FlatSettings<T>,
    fallback: &FlatSettings<T>,
    plains: BiomeId,
    resolver: &impl FlatShareResolver,
) -> FlatSettings<T> {
    let mut parts = input.splitn(3, ';');
    let layers_text = parts.next().unwrap_or_default();
    let biome_text = parts.next();
    let Some(layers) = parse_layers(layers_text, resolver) else {
        return fallback.clone();
    };
    if layers.is_empty() {
        return fallback.clone();
    }
    let biome = biome_text
        .and_then(|identifier| resolver.biome(identifier))
        .unwrap_or(plains);
    selected.with_edited_layers_and_biome(layers, biome)
}

pub fn export_flat_share<T>(
    settings: &FlatSettings<T>,
    resolver: &impl FlatShareResolver,
) -> Result<String, FlatError> {
    let mut layers = Vec::with_capacity(settings.layers.len());
    for layer in &settings.layers {
        let identifier = resolver
            .block_identifier(layer.state)
            .ok_or(FlatError::UnknownIdentifier)?;
        if layer.height == 1 {
            layers.push(identifier.to_owned());
        } else {
            layers.push(format!("{}*{identifier}", layer.height));
        }
    }
    let biome = resolver
        .biome_identifier(settings.biome)
        .ok_or(FlatError::UnknownIdentifier)?;
    Ok(format!("{};{biome}", layers.join(",")))
}

fn parse_layers(input: &str, resolver: &impl FlatShareResolver) -> Option<Vec<FlatLayer>> {
    let mut result = Vec::new();
    let mut used = 0_usize;
    for token in input.split(',') {
        let (height, identifier) = match token.split_once('*') {
            Some((height, identifier)) => {
                let parsed = height.parse::<i32>().ok()?.max(0) as usize;
                (parsed, identifier)
            }
            None => (1, token),
        };
        let state = resolver.block_state(identifier)?;
        let retained = height.min(MAX_FLAT_HEIGHT.saturating_sub(used));
        used += retained;
        result.push(FlatLayer {
            height: retained as u16,
            state,
        });
    }
    Some(result)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FlatError {
    #[error("flat layers exceed the maximum encoded height")]
    LayerHeightOverflow,
    #[error("flat generation position overflow")]
    PositionOverflow,
    #[error("flat share references an unknown registry identifier")]
    UnknownIdentifier,
}
