//! Validated lowering of the locally imported Minecraft block report.

use crate::block_state::{
    BlockState, BlockStateError, BlockStateSchema, PropertyAssignment, PropertyName,
    PropertySchema, PropertyValue, StateIndex,
};
use crate::bundle::{BundleRegistry, FamilyName};
use crate::registry::PersistentId;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const BLOCK_REGISTRY: &str = "minecraft:block";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftBlockDefinition {
    persistent_id: PersistentId,
    family: FamilyName,
    schema: BlockStateSchema,
    raw_states: Vec<u32>,
}

impl MinecraftBlockDefinition {
    pub const fn persistent_id(&self) -> &PersistentId {
        &self.persistent_id
    }

    pub const fn family(&self) -> &FamilyName {
        &self.family
    }

    pub const fn schema(&self) -> &BlockStateSchema {
        &self.schema
    }

    pub fn raw_state(&self, index: StateIndex) -> Option<u32> {
        self.raw_states.get(index.get() as usize).copied()
    }

    pub fn raw_state_of(&self, state: &BlockState) -> Result<u32, BlockCatalogError> {
        let index = self.schema.index_of(state)?;
        self.raw_state(index)
            .ok_or(BlockCatalogError::MissingCanonicalState {
                block: self.persistent_id.clone(),
                index: index.get(),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawBlockState {
    block: PersistentId,
    state_index: StateIndex,
}

impl RawBlockState {
    pub const fn block(&self) -> &PersistentId {
        &self.block
    }

    pub const fn state_index(&self) -> StateIndex {
        self.state_index
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftBlockCatalog {
    definitions: BTreeMap<PersistentId, MinecraftBlockDefinition>,
    raw_states: BTreeMap<u32, RawBlockState>,
}

impl MinecraftBlockCatalog {
    pub fn from_registry(registry: &BundleRegistry) -> Result<Self, BlockCatalogError> {
        if registry.name().to_string() != BLOCK_REGISTRY {
            return Err(BlockCatalogError::WrongRegistry {
                actual: registry.name().to_string(),
            });
        }

        let mut definitions = BTreeMap::new();
        let mut raw_states = BTreeMap::new();
        for entry in registry.entries() {
            let definition = lower_definition(
                entry.persistent_id().clone(),
                entry.family().clone(),
                entry.value(),
            )?;
            for index in 0..definition.schema.state_count() {
                let state_index = StateIndex::new(index);
                let raw = definition.raw_state(state_index).ok_or_else(|| {
                    BlockCatalogError::MissingCanonicalState {
                        block: definition.persistent_id.clone(),
                        index,
                    }
                })?;
                if raw_states
                    .insert(
                        raw,
                        RawBlockState {
                            block: definition.persistent_id.clone(),
                            state_index,
                        },
                    )
                    .is_some()
                {
                    return Err(BlockCatalogError::DuplicateRawState { raw });
                }
            }
            let id = definition.persistent_id.clone();
            if definitions.insert(id.clone(), definition).is_some() {
                return Err(BlockCatalogError::DuplicateBlock { block: id });
            }
        }

        Ok(Self {
            definitions,
            raw_states,
        })
    }

    pub fn definitions(
        &self,
    ) -> impl ExactSizeIterator<Item = &MinecraftBlockDefinition> + DoubleEndedIterator {
        self.definitions.values()
    }

    pub fn definition(&self, id: &PersistentId) -> Option<&MinecraftBlockDefinition> {
        self.definitions.get(id)
    }

    pub fn state_by_raw(&self, raw: u32) -> Option<(&MinecraftBlockDefinition, BlockState)> {
        let state = self.raw_states.get(&raw)?;
        let definition = self.definitions.get(&state.block)?;
        Some((definition, definition.schema.state_at(state.state_index)?))
    }
}

fn lower_definition(
    block: PersistentId,
    family: FamilyName,
    value: &Value,
) -> Result<MinecraftBlockDefinition, BlockCatalogError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(&block, "entry value is not an object"))?;
    let states = array_field(object, "states", &block)?;
    if states.is_empty() {
        return Err(invalid(&block, "states is empty"));
    }
    let default = unique_default(states, &block)?;
    let property_values = optional_object_field(object, "properties", &block)?;
    let default_properties = state_properties(default, &block)?;

    let mut properties = Vec::with_capacity(property_values.len());
    for (name, values) in property_values {
        let name = PropertyName::new(name.clone())?;
        let values = values
            .as_array()
            .ok_or_else(|| invalid(&block, "property values is not an array"))?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| invalid(&block, "property value is not a string"))
                    .and_then(|value| PropertyValue::new(value).map_err(Into::into))
            })
            .collect::<Result<Vec<_>, BlockCatalogError>>()?;
        let default = default_properties
            .get(name.as_str())
            .and_then(Value::as_str)
            .ok_or_else(|| invalid(&block, "default state omits a declared property"))?;
        properties.push(PropertySchema::new(
            name,
            values,
            &PropertyValue::new(default)?,
        )?);
    }

    let schema = BlockStateSchema::new(block.clone(), properties)?;
    if schema.state_count() as usize != states.len() {
        return Err(BlockCatalogError::StateCardinality {
            block,
            declared: states.len(),
            calculated: schema.state_count(),
        });
    }

    let mut raw_states = vec![None; states.len()];
    let mut seen_raw = BTreeSet::new();
    for value in states {
        let object = value
            .as_object()
            .ok_or_else(|| invalid(&block, "state is not an object"))?;
        let raw = u32_field(object, "id", &block)?;
        if !seen_raw.insert(raw) {
            return Err(BlockCatalogError::DuplicateRawState { raw });
        }
        let assignments = state_assignments(&schema, object, &block)?;
        let index = schema.index_of(&BlockState::new(block.clone(), assignments))?;
        let slot = &mut raw_states[index.get() as usize];
        if slot.replace(raw).is_some() {
            return Err(BlockCatalogError::DuplicateCanonicalState {
                block,
                index: index.get(),
            });
        }
    }

    let raw_states = raw_states
        .into_iter()
        .enumerate()
        .map(|(index, raw)| {
            raw.ok_or_else(|| BlockCatalogError::MissingCanonicalState {
                block: block.clone(),
                index: index as u32,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MinecraftBlockDefinition {
        persistent_id: block,
        family,
        schema,
        raw_states,
    })
}

fn state_assignments(
    schema: &BlockStateSchema,
    state: &Map<String, Value>,
    block: &PersistentId,
) -> Result<Vec<PropertyAssignment>, BlockCatalogError> {
    let properties = state_properties(state, block)?;
    if properties.len() != schema.properties().len() {
        return Err(invalid(block, "state property count does not match schema"));
    }
    schema
        .properties()
        .map(|schema| {
            let value = properties
                .get(schema.name().as_str())
                .and_then(Value::as_str)
                .ok_or_else(|| invalid(block, "state omits a declared property"))?;
            Ok(PropertyAssignment::new(
                schema.name().clone(),
                PropertyValue::new(value)?,
            ))
        })
        .collect()
}

fn unique_default<'a>(
    states: &'a [Value],
    block: &PersistentId,
) -> Result<&'a Map<String, Value>, BlockCatalogError> {
    let defaults = states
        .iter()
        .filter_map(|state| {
            let object = state.as_object()?;
            (object.get("default").and_then(Value::as_bool) == Some(true)).then_some(object)
        })
        .collect::<Vec<_>>();
    if defaults.len() != 1 {
        return Err(invalid(block, "block must have exactly one default state"));
    }
    Ok(defaults[0])
}

fn state_properties<'a>(
    state: &'a Map<String, Value>,
    block: &PersistentId,
) -> Result<&'a Map<String, Value>, BlockCatalogError> {
    match state.get("properties") {
        Some(value) => value
            .as_object()
            .ok_or_else(|| invalid(block, "state properties is not an object")),
        None => Ok(empty_object()),
    }
}

fn empty_object() -> &'static Map<String, Value> {
    static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
    EMPTY.get_or_init(Map::new)
}

fn array_field<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
    block: &PersistentId,
) -> Result<&'a Vec<Value>, BlockCatalogError> {
    object
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(block, "required array field is absent"))
}

fn optional_object_field<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
    block: &PersistentId,
) -> Result<&'a Map<String, Value>, BlockCatalogError> {
    match object.get(field) {
        Some(value) => value
            .as_object()
            .ok_or_else(|| invalid(block, "property schema is not an object")),
        None => Ok(empty_object()),
    }
}

fn u32_field(
    object: &Map<String, Value>,
    field: &'static str,
    block: &PersistentId,
) -> Result<u32, BlockCatalogError> {
    let value = object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(block, "state ID is absent or negative"))?;
    u32::try_from(value).map_err(|_| invalid(block, "state ID exceeds u32"))
}

fn invalid(block: &PersistentId, reason: &'static str) -> BlockCatalogError {
    BlockCatalogError::InvalidReport {
        block: block.clone(),
        reason,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlockCatalogError {
    #[error("expected registry minecraft:block, found {actual}")]
    WrongRegistry { actual: String },
    #[error("invalid imported block report for {block}: {reason}")]
    InvalidReport {
        block: PersistentId,
        reason: &'static str,
    },
    #[error("block {block} has {declared} states, schema calculates {calculated}")]
    StateCardinality {
        block: PersistentId,
        declared: usize,
        calculated: u32,
    },
    #[error("duplicate block {block}")]
    DuplicateBlock { block: PersistentId },
    #[error("duplicate raw block state ID {raw}")]
    DuplicateRawState { raw: u32 },
    #[error("duplicate canonical state {index} for {block}")]
    DuplicateCanonicalState { block: PersistentId, index: u32 },
    #[error("missing canonical state {index} for {block}")]
    MissingCanonicalState { block: PersistentId, index: u32 },
    #[error(transparent)]
    State(#[from] BlockStateError),
}
