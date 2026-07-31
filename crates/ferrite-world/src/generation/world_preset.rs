//! Strict runtime projection of world-preset records from the content bundle.

use std::collections::BTreeMap;

use ferrite_registry::bundle::BundleEntry;
use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DimensionSlot {
    Overworld,
    Nether,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldPreset {
    pub name: String,
    pub dimensions: BTreeMap<DimensionSlot, DimensionDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionDescriptor {
    pub dimension_type: String,
    pub generator: GeneratorDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorDescriptor {
    Noise {
        settings: String,
        biome_source: BiomeSourceDescriptor,
    },
    Flat(FlatSettings),
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiomeSourceDescriptor {
    Fixed { biome: String },
    MultiNoise { preset: String },
    TheEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatSettings {
    pub biome: String,
    pub features: bool,
    pub lakes: bool,
    pub layers: Vec<FlatLayer>,
    pub structures: StructureSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLayer {
    pub block: String,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructureSelection {
    All,
    None,
    Listed(Vec<String>),
}

impl WorldPreset {
    pub fn decode(entry: &BundleEntry) -> Result<Self, WorldPresetError> {
        let path = entry.persistent_id().resource().path();
        let name = path
            .strip_prefix("world_preset/")
            .ok_or(WorldPresetError::WrongRegistryPath)?
            .to_owned();
        let root = object(entry.value(), "world preset")?;
        let dimensions = object(field(root, "dimensions")?, "dimensions")?;
        let mut decoded = BTreeMap::new();
        for (name, value) in dimensions {
            let slot = match name.as_str() {
                "minecraft:overworld" => DimensionSlot::Overworld,
                "minecraft:the_nether" => DimensionSlot::Nether,
                "minecraft:the_end" => DimensionSlot::End,
                _ => return Err(WorldPresetError::UnknownDimension(name.clone())),
            };
            let dimension = object(value, "dimension")?;
            let dimension_type = string(field(dimension, "type")?, "dimension type")?.to_owned();
            let generator = decode_generator(field(dimension, "generator")?)?;
            if decoded
                .insert(
                    slot,
                    DimensionDescriptor {
                        dimension_type,
                        generator,
                    },
                )
                .is_some()
            {
                return Err(WorldPresetError::DuplicateDimension(slot));
            }
        }
        for required in [
            DimensionSlot::Overworld,
            DimensionSlot::Nether,
            DimensionSlot::End,
        ] {
            if !decoded.contains_key(&required) {
                return Err(WorldPresetError::MissingDimension(required));
            }
        }
        Ok(Self {
            name,
            dimensions: decoded,
        })
    }
}

fn decode_generator(value: &Value) -> Result<GeneratorDescriptor, WorldPresetError> {
    let generator = object(value, "generator")?;
    match string(field(generator, "type")?, "generator type")? {
        "minecraft:noise" => Ok(GeneratorDescriptor::Noise {
            settings: string(field(generator, "settings")?, "noise settings")?.to_owned(),
            biome_source: decode_biome_source(field(generator, "biome_source")?)?,
        }),
        "minecraft:flat" => Ok(GeneratorDescriptor::Flat(decode_flat(field(
            generator, "settings",
        )?)?)),
        "minecraft:debug" => Ok(GeneratorDescriptor::Debug),
        other => Err(WorldPresetError::UnknownGenerator(other.to_owned())),
    }
}

fn decode_biome_source(value: &Value) -> Result<BiomeSourceDescriptor, WorldPresetError> {
    let source = object(value, "biome source")?;
    match string(field(source, "type")?, "biome source type")? {
        "minecraft:fixed" => Ok(BiomeSourceDescriptor::Fixed {
            biome: string(field(source, "biome")?, "fixed biome")?.to_owned(),
        }),
        "minecraft:multi_noise" => Ok(BiomeSourceDescriptor::MultiNoise {
            preset: string(field(source, "preset")?, "multi-noise preset")?.to_owned(),
        }),
        "minecraft:the_end" => Ok(BiomeSourceDescriptor::TheEnd),
        other => Err(WorldPresetError::UnknownBiomeSource(other.to_owned())),
    }
}

fn decode_flat(value: &Value) -> Result<FlatSettings, WorldPresetError> {
    let settings = object(value, "flat settings")?;
    let layers = array(field(settings, "layers")?, "flat layers")?;
    let mut decoded_layers = Vec::with_capacity(layers.len());
    let mut total_height = 0_u32;
    for value in layers {
        let layer = object(value, "flat layer")?;
        let raw_height = integer(field(layer, "height")?, "flat layer height")?;
        let height = u16::try_from(raw_height)
            .ok()
            .filter(|height| *height <= 4_064)
            .ok_or(WorldPresetError::InvalidLayerHeight(raw_height))?;
        total_height += u32::from(height);
        if total_height > 4_064 {
            return Err(WorldPresetError::LayerHeightOverflow(total_height));
        }
        decoded_layers.push(FlatLayer {
            block: string(field(layer, "block")?, "flat layer block")?.to_owned(),
            height,
        });
    }
    let structures = match settings.get("structure_overrides") {
        None => StructureSelection::All,
        Some(value) => {
            let values = array(value, "structure overrides")?;
            if values.is_empty() {
                StructureSelection::None
            } else {
                StructureSelection::Listed(
                    values
                        .iter()
                        .map(|value| string(value, "structure override").map(ToOwned::to_owned))
                        .collect::<Result<_, _>>()?,
                )
            }
        }
    };
    Ok(FlatSettings {
        biome: settings
            .get("biome")
            .map(|value| string(value, "flat biome"))
            .transpose()?
            .unwrap_or("minecraft:plains")
            .to_owned(),
        features: optional_bool(settings, "features")?,
        lakes: optional_bool(settings, "lakes")?,
        layers: decoded_layers,
        structures,
    })
}

fn field<'a>(
    object: &'a Map<String, Value>,
    name: &'static str,
) -> Result<&'a Value, WorldPresetError> {
    object.get(name).ok_or(WorldPresetError::MissingField(name))
}

fn object<'a>(
    value: &'a Value,
    context: &'static str,
) -> Result<&'a Map<String, Value>, WorldPresetError> {
    value
        .as_object()
        .ok_or(WorldPresetError::WrongType(context))
}

fn array<'a>(value: &'a Value, context: &'static str) -> Result<&'a [Value], WorldPresetError> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or(WorldPresetError::WrongType(context))
}

fn string<'a>(value: &'a Value, context: &'static str) -> Result<&'a str, WorldPresetError> {
    value.as_str().ok_or(WorldPresetError::WrongType(context))
}

fn integer(value: &Value, context: &'static str) -> Result<i64, WorldPresetError> {
    value.as_i64().ok_or(WorldPresetError::WrongType(context))
}

fn optional_bool(
    object: &Map<String, Value>,
    name: &'static str,
) -> Result<bool, WorldPresetError> {
    object
        .get(name)
        .map(|value| value.as_bool().ok_or(WorldPresetError::WrongType(name)))
        .transpose()
        .map(|value| value.unwrap_or(false))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorldPresetError {
    #[error("content entry is not a world_preset record")]
    WrongRegistryPath,
    #[error("world preset is missing required field {0}")]
    MissingField(&'static str),
    #[error("{0} has the wrong JSON type")]
    WrongType(&'static str),
    #[error("unknown world-preset dimension {0}")]
    UnknownDimension(String),
    #[error("world preset repeats dimension {0:?}")]
    DuplicateDimension(DimensionSlot),
    #[error("world preset omits dimension {0:?}")]
    MissingDimension(DimensionSlot),
    #[error("unknown chunk generator {0}")]
    UnknownGenerator(String),
    #[error("unknown biome source {0}")]
    UnknownBiomeSource(String),
    #[error("flat layer height {0} lies outside 0..=4064")]
    InvalidLayerHeight(i64),
    #[error("flat layers total {0}, above the 4064-block codec limit")]
    LayerHeightOverflow(u32),
}
