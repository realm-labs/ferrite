//! Behavior-bearing projection of locked noise-generator settings records.

use ferrite_registry::bundle::BundleEntry;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::generation::noise_fill::NoiseSettings;

#[derive(Debug, Clone, PartialEq)]
pub struct NoiseGeneratorSettingsRecord {
    pub name: String,
    pub noise: NoiseSettings,
    pub default_block: String,
    pub default_fluid: String,
    pub sea_level: i32,
    pub aquifers_enabled: bool,
    pub ore_veins_enabled: bool,
    pub disable_mob_generation: bool,
    pub legacy_random_source: bool,
    pub noise_router: Value,
    pub surface_rule: Value,
    pub spawn_target: Vec<Value>,
}

impl NoiseGeneratorSettingsRecord {
    pub fn decode(entry: &BundleEntry) -> Result<Self, NoiseSettingsRecordError> {
        let path = entry.persistent_id().resource().path();
        let name = path
            .strip_prefix("noise_settings/")
            .ok_or(NoiseSettingsRecordError::WrongRegistryPath)?
            .to_owned();
        let root = object(entry.value(), "noise settings")?;
        let noise = object(field(root, "noise")?, "noise shape")?;
        Ok(Self {
            name,
            noise: NoiseSettings {
                minimum_y: i32_value(field(noise, "min_y")?, "minimum Y")?,
                height: u32_value(field(noise, "height")?, "height")?,
                horizontal_size: u8_value(field(noise, "size_horizontal")?, "horizontal size")?,
                vertical_size: u8_value(field(noise, "size_vertical")?, "vertical size")?,
            },
            default_block: block_name(field(root, "default_block")?)?.to_owned(),
            default_fluid: block_name(field(root, "default_fluid")?)?.to_owned(),
            sea_level: i32_value(field(root, "sea_level")?, "sea level")?,
            aquifers_enabled: boolean(field(root, "aquifers_enabled")?, "aquifers flag")?,
            ore_veins_enabled: boolean(field(root, "ore_veins_enabled")?, "ore flag")?,
            disable_mob_generation: boolean(field(root, "disable_mob_generation")?, "mob flag")?,
            legacy_random_source: boolean(field(root, "legacy_random_source")?, "legacy RNG flag")?,
            noise_router: field(root, "noise_router")?.clone(),
            surface_rule: field(root, "surface_rule")?.clone(),
            spawn_target: array(field(root, "spawn_target")?, "spawn target")?.to_vec(),
        })
    }
}

fn block_name(value: &Value) -> Result<&str, NoiseSettingsRecordError> {
    let block = object(value, "block state")?;
    string(field(block, "Name")?, "block name")
}

fn field<'a>(
    object: &'a Map<String, Value>,
    name: &'static str,
) -> Result<&'a Value, NoiseSettingsRecordError> {
    object
        .get(name)
        .ok_or(NoiseSettingsRecordError::MissingField(name))
}

fn object<'a>(
    value: &'a Value,
    context: &'static str,
) -> Result<&'a Map<String, Value>, NoiseSettingsRecordError> {
    value
        .as_object()
        .ok_or(NoiseSettingsRecordError::WrongType(context))
}

fn array<'a>(
    value: &'a Value,
    context: &'static str,
) -> Result<&'a [Value], NoiseSettingsRecordError> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or(NoiseSettingsRecordError::WrongType(context))
}

fn string<'a>(
    value: &'a Value,
    context: &'static str,
) -> Result<&'a str, NoiseSettingsRecordError> {
    value
        .as_str()
        .ok_or(NoiseSettingsRecordError::WrongType(context))
}

fn boolean(value: &Value, context: &'static str) -> Result<bool, NoiseSettingsRecordError> {
    value
        .as_bool()
        .ok_or(NoiseSettingsRecordError::WrongType(context))
}

fn i32_value(value: &Value, context: &'static str) -> Result<i32, NoiseSettingsRecordError> {
    value
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(NoiseSettingsRecordError::WrongType(context))
}

fn u32_value(value: &Value, context: &'static str) -> Result<u32, NoiseSettingsRecordError> {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(NoiseSettingsRecordError::WrongType(context))
}

fn u8_value(value: &Value, context: &'static str) -> Result<u8, NoiseSettingsRecordError> {
    value
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .ok_or(NoiseSettingsRecordError::WrongType(context))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NoiseSettingsRecordError {
    #[error("content entry is not a noise_settings record")]
    WrongRegistryPath,
    #[error("noise settings are missing required field {0}")]
    MissingField(&'static str),
    #[error("{0} has the wrong JSON type or numeric range")]
    WrongType(&'static str),
}
