//! Runtime decoding for locked jigsaw structure and structure-set records.

use std::num::NonZeroU32;

use ferrite_registry::bundle::BundleEntry;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::jigsaw::{AliasBinding, JigsawStartConfig, Padding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAdaptation {
    None,
    Bury,
    BeardThin,
    BeardBox,
    Encapsulate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartHeight {
    Absolute(i32),
    UniformAbsolute { minimum: i32, maximum: i32 },
}

impl StartHeight {
    pub fn sample(self, random: &mut impl GenerationRandom) -> i32 {
        match self {
            Self::Absolute(value) => value,
            Self::UniformAbsolute { minimum, maximum } => {
                let width = maximum
                    .checked_sub(minimum)
                    .and_then(|difference| difference.checked_add(1))
                    .and_then(|value| u32::try_from(value).ok())
                    .and_then(NonZeroU32::new)
                    .expect("decoded uniform height has a positive i32 range");
                minimum + i32::try_from(random.next_u32(width)).expect("height draw fits i32")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JigsawStructureRecord {
    pub name: String,
    pub biomes: String,
    pub step: String,
    pub terrain_adaptation: TerrainAdaptation,
    pub start_pool: String,
    pub start_jigsaw_name: Option<String>,
    pub start_height: StartHeight,
    pub project_to_world_surface: bool,
    pub size: u8,
    pub maximum_distance_from_center: u16,
    pub maximum_vertical_distance: Option<u16>,
    pub expansion_hack: bool,
    pub dimension_padding: Padding,
    pub ignore_waterlogging: bool,
    pub pool_aliases: Vec<AliasBinding>,
    pub spawn_overrides: Map<String, Value>,
}

impl JigsawStructureRecord {
    pub fn decode(entry: &BundleEntry) -> Result<Self, JigsawRecordError> {
        let name = entry
            .persistent_id()
            .resource()
            .path()
            .strip_prefix("structure/")
            .ok_or(JigsawRecordError::WrongRegistry)?
            .to_owned();
        let object = entry.value().as_object().ok_or(JigsawRecordError::Object)?;
        if string(object, "type")? != "minecraft:jigsaw" {
            return Err(JigsawRecordError::WrongType);
        }
        let terrain_adaptation = match optional_string(object, "terrain_adaptation")? {
            None | Some("none") => TerrainAdaptation::None,
            Some("bury") => TerrainAdaptation::Bury,
            Some("beard_thin") => TerrainAdaptation::BeardThin,
            Some("beard_box") => TerrainAdaptation::BeardBox,
            Some("encapsulate") => TerrainAdaptation::Encapsulate,
            Some(value) => return Err(JigsawRecordError::Adaptation(value.into())),
        };
        let start_height = decode_height(
            object
                .get("start_height")
                .ok_or(JigsawRecordError::Missing("start_height"))?,
        )?;
        let pool_aliases = match object.get("pool_aliases") {
            Some(value) => array(value)?
                .iter()
                .map(decode_alias)
                .collect::<Result<_, _>>()?,
            None => Vec::new(),
        };
        let spawn_overrides = match object.get("spawn_overrides") {
            Some(value) => object_value(value)?.clone(),
            None => Map::new(),
        };
        let project_to_world_surface = match optional_string(object, "project_start_to_heightmap")?
        {
            None => false,
            Some("WORLD_SURFACE_WG") => true,
            Some(value) => return Err(JigsawRecordError::Heightmap(value.into())),
        };
        let (maximum_distance_from_center, maximum_vertical_distance) = decode_distance(
            object
                .get("max_distance_from_center")
                .ok_or(JigsawRecordError::Missing("max_distance_from_center"))?,
        )?;
        let dimension_padding = object
            .get("dimension_padding")
            .map(decode_padding)
            .transpose()?
            .unwrap_or(Padding::ZERO);
        let size = ranged_u8(integer(object, "size")?, "size", 0, 20)?;
        let reserve = if terrain_adaptation == TerrainAdaptation::None {
            0
        } else {
            12
        };
        if u32::from(maximum_distance_from_center) + reserve > 128 {
            return Err(JigsawRecordError::Range("max_distance_from_center"));
        }
        let ignore_waterlogging = match optional_string(object, "liquid_settings")? {
            None | Some("apply_waterlogging") => false,
            Some("ignore_waterlogging") => true,
            Some(value) => return Err(JigsawRecordError::Liquid(value.into())),
        };
        Ok(Self {
            name,
            biomes: string(object, "biomes")?.into(),
            step: string(object, "step")?.into(),
            terrain_adaptation,
            start_pool: string(object, "start_pool")?.into(),
            start_jigsaw_name: optional_string(object, "start_jigsaw_name")?.map(str::to_owned),
            start_height,
            project_to_world_surface,
            size,
            maximum_distance_from_center,
            maximum_vertical_distance,
            expansion_hack: boolean(object, "use_expansion_hack")?,
            dimension_padding,
            ignore_waterlogging,
            pool_aliases,
            spawn_overrides,
        })
    }

    pub fn sample_start_y(
        &self,
        surface_height: Option<i32>,
        random: &mut impl GenerationRandom,
    ) -> i32 {
        let sampled = self.start_height.sample(random);
        if self.project_to_world_surface {
            sampled.wrapping_add(surface_height.unwrap_or_default())
        } else {
            sampled
        }
    }

    pub fn start_config(
        &self,
        dimension_min_y: i32,
        dimension_max_y: i32,
    ) -> Option<JigsawStartConfig> {
        let dimension_height = dimension_max_y
            .checked_sub(dimension_min_y)?
            .checked_add(1)?;
        let vertical_distance = self
            .maximum_vertical_distance
            .map(i32::from)
            .unwrap_or(dimension_height);
        Some(JigsawStartConfig {
            dimension_min_y,
            dimension_max_y,
            padding: self.dimension_padding,
            maximum_depth: self.size,
            horizontal_distance: i32::from(self.maximum_distance_from_center),
            vertical_distance,
            use_expansion_hack: self.expansion_hack,
        })
    }
}

fn decode_alias(value: &Value) -> Result<AliasBinding, JigsawRecordError> {
    let value = object_value(value)?;
    match string(value, "type")? {
        "minecraft:direct" => Ok(AliasBinding::Direct {
            alias: string(value, "alias")?.into(),
            target: string(value, "target")?.into(),
        }),
        "minecraft:random" => Ok(AliasBinding::Random {
            alias: string(value, "alias")?.into(),
            targets: decode_weighted(
                value
                    .get("targets")
                    .ok_or(JigsawRecordError::Missing("targets"))?,
                |value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .ok_or(JigsawRecordError::String("data"))
                },
            )?,
        }),
        "minecraft:random_group" => Ok(AliasBinding::RandomGroup(decode_weighted(
            value
                .get("groups")
                .ok_or(JigsawRecordError::Missing("groups"))?,
            |value| {
                array(value)?
                    .iter()
                    .map(decode_alias)
                    .collect::<Result<Vec<_>, _>>()
            },
        )?)),
        value => Err(JigsawRecordError::Alias(value.into())),
    }
}

fn decode_weighted<T>(
    value: &Value,
    mut decode: impl FnMut(&Value) -> Result<T, JigsawRecordError>,
) -> Result<Vec<(T, u16)>, JigsawRecordError> {
    array(value)?
        .iter()
        .map(|value| {
            let value = object_value(value)?;
            let weight = ranged_u16(integer(value, "weight")?, "weight", 1, u16::MAX)?;
            let data = value
                .get("data")
                .ok_or(JigsawRecordError::Missing("data"))?;
            Ok((decode(data)?, weight))
        })
        .collect()
}

fn decode_distance(value: &Value) -> Result<(u16, Option<u16>), JigsawRecordError> {
    if let Some(value) = value.as_i64() {
        let value = ranged_u16(value, "max_distance_from_center", 1, 128)?;
        return Ok((value, Some(value)));
    }
    let value = object_value(value)?;
    let horizontal = ranged_u16(
        integer(value, "horizontal")?,
        "max_distance_from_center.horizontal",
        1,
        128,
    )?;
    let vertical = optional_integer(value, "vertical")?
        .map(|value| ranged_u16(value, "max_distance_from_center.vertical", 1, u16::MAX))
        .transpose()?;
    Ok((horizontal, vertical))
}

fn decode_padding(value: &Value) -> Result<Padding, JigsawRecordError> {
    if let Some(value) = value.as_i64() {
        let value = ranged_u16(value, "dimension_padding", 0, u16::MAX)?;
        return Ok(Padding::new(u32::from(value), u32::from(value)));
    }
    let value = object_value(value)?;
    let bottom = optional_integer(value, "bottom")?
        .unwrap_or(0)
        .try_into()
        .map_err(|_| JigsawRecordError::Integer("dimension_padding.bottom"))?;
    let top = optional_integer(value, "top")?
        .unwrap_or(0)
        .try_into()
        .map_err(|_| JigsawRecordError::Integer("dimension_padding.top"))?;
    Ok(Padding::new(bottom, top))
}

fn ranged_u8(
    value: i64,
    field: &'static str,
    minimum: u8,
    maximum: u8,
) -> Result<u8, JigsawRecordError> {
    let value = u8::try_from(value).map_err(|_| JigsawRecordError::Integer(field))?;
    if !(minimum..=maximum).contains(&value) {
        return Err(JigsawRecordError::Range(field));
    }
    Ok(value)
}

fn ranged_u16(
    value: i64,
    field: &'static str,
    minimum: u16,
    maximum: u16,
) -> Result<u16, JigsawRecordError> {
    let value = u16::try_from(value).map_err(|_| JigsawRecordError::Integer(field))?;
    if !(minimum..=maximum).contains(&value) {
        return Err(JigsawRecordError::Range(field));
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedStructure {
    pub structure: String,
    pub weight: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureSetRecord {
    pub name: String,
    pub structures: Vec<WeightedStructure>,
    pub spacing: u32,
    pub separation: u32,
    pub salt: u32,
    pub frequency: f64,
    pub frequency_reduction_method: String,
    pub exclusion_zone: Option<(String, u32)>,
}

impl StructureSetRecord {
    pub fn decode(entry: &BundleEntry) -> Result<Self, JigsawRecordError> {
        let name = entry
            .persistent_id()
            .resource()
            .path()
            .strip_prefix("structure_set/")
            .ok_or(JigsawRecordError::WrongRegistry)?
            .to_owned();
        let root = entry.value().as_object().ok_or(JigsawRecordError::Object)?;
        let structures = array(
            root.get("structures")
                .ok_or(JigsawRecordError::Missing("structures"))?,
        )?
        .iter()
        .map(|value| {
            let item = object_value(value)?;
            Ok(WeightedStructure {
                structure: string(item, "structure")?.into(),
                weight: integer(item, "weight")?
                    .try_into()
                    .map_err(|_| JigsawRecordError::Integer("weight"))?,
            })
        })
        .collect::<Result<Vec<_>, JigsawRecordError>>()?;
        let placement = object_value(
            root.get("placement")
                .ok_or(JigsawRecordError::Missing("placement"))?,
        )?;
        if string(placement, "type")? != "minecraft:random_spread" {
            return Err(JigsawRecordError::WrongType);
        }
        let exclusion_zone = placement
            .get("exclusion_zone")
            .map(|value| {
                let exclusion = object_value(value)?;
                Ok((
                    string(exclusion, "other_set")?.into(),
                    integer(exclusion, "chunk_count")?
                        .try_into()
                        .map_err(|_| JigsawRecordError::Integer("chunk_count"))?,
                ))
            })
            .transpose()?;
        Ok(Self {
            name,
            structures,
            spacing: u32_value(placement, "spacing")?,
            separation: u32_value(placement, "separation")?,
            salt: u32_value(placement, "salt")?,
            frequency: placement.get("frequency").map_or(Ok(1.0), |value| {
                value.as_f64().ok_or(JigsawRecordError::Number("frequency"))
            })?,
            frequency_reduction_method: optional_string(placement, "frequency_reduction_method")?
                .unwrap_or("default")
                .into(),
            exclusion_zone,
        })
    }
}

fn decode_height(value: &Value) -> Result<StartHeight, JigsawRecordError> {
    let height = object_value(value)?;
    if let Some(value) = height.get("absolute") {
        return Ok(StartHeight::Absolute(
            value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or(JigsawRecordError::Integer("absolute"))?,
        ));
    }
    if string(height, "type")? != "minecraft:uniform" {
        return Err(JigsawRecordError::Height);
    }
    let endpoint = |key| {
        object_value(height.get(key).ok_or(JigsawRecordError::Missing(key))?)?
            .get("absolute")
            .and_then(Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .ok_or(JigsawRecordError::Integer(key))
    };
    let minimum = endpoint("min_inclusive")?;
    let maximum = endpoint("max_inclusive")?;
    if minimum > maximum {
        return Err(JigsawRecordError::Height);
    }
    Ok(StartHeight::UniformAbsolute { minimum, maximum })
}

fn object_value(value: &Value) -> Result<&Map<String, Value>, JigsawRecordError> {
    value.as_object().ok_or(JigsawRecordError::Object)
}

fn array(value: &Value) -> Result<&[Value], JigsawRecordError> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or(JigsawRecordError::Array)
}

fn string<'a>(
    object: &'a Map<String, Value>,
    key: &'static str,
) -> Result<&'a str, JigsawRecordError> {
    optional_string(object, key)?.ok_or(JigsawRecordError::Missing(key))
}

fn optional_string<'a>(
    object: &'a Map<String, Value>,
    key: &'static str,
) -> Result<Option<&'a str>, JigsawRecordError> {
    object
        .get(key)
        .map(|value| value.as_str().ok_or(JigsawRecordError::String(key)))
        .transpose()
}

fn integer(object: &Map<String, Value>, key: &'static str) -> Result<i64, JigsawRecordError> {
    optional_integer(object, key)?.ok_or(JigsawRecordError::Missing(key))
}

fn optional_integer(
    object: &Map<String, Value>,
    key: &'static str,
) -> Result<Option<i64>, JigsawRecordError> {
    object
        .get(key)
        .map(|value| value.as_i64().ok_or(JigsawRecordError::Integer(key)))
        .transpose()
}

fn u32_value(object: &Map<String, Value>, key: &'static str) -> Result<u32, JigsawRecordError> {
    integer(object, key)?
        .try_into()
        .map_err(|_| JigsawRecordError::Integer(key))
}

fn boolean(object: &Map<String, Value>, key: &'static str) -> Result<bool, JigsawRecordError> {
    object
        .get(key)
        .and_then(Value::as_bool)
        .ok_or(JigsawRecordError::Boolean(key))
}

#[derive(Debug, Error, PartialEq)]
pub enum JigsawRecordError {
    #[error("record belongs to the wrong worldgen registry")]
    WrongRegistry,
    #[error("record has the wrong codec type")]
    WrongType,
    #[error("expected an object")]
    Object,
    #[error("expected an array")]
    Array,
    #[error("missing field {0}")]
    Missing(&'static str),
    #[error("{0} must be a string")]
    String(&'static str),
    #[error("{0} must be an integer in range")]
    Integer(&'static str),
    #[error("{0} must be a number")]
    Number(&'static str),
    #[error("{0} must be a boolean")]
    Boolean(&'static str),
    #[error("unknown terrain adaptation {0}")]
    Adaptation(String),
    #[error("invalid start height")]
    Height,
    #[error("{0} lies outside its codec range")]
    Range(&'static str),
    #[error("unknown start heightmap {0}")]
    Heightmap(String),
    #[error("unknown structure liquid setting {0}")]
    Liquid(String),
    #[error("unknown pool alias binding {0}")]
    Alias(String),
}
