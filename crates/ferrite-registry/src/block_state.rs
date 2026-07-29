//! Validated block-property schemas and persistent block states.

use crate::registry::PersistentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PropertyName(String);

impl PropertyName {
    pub fn new(value: impl Into<String>) -> Result<Self, BlockStateError> {
        let value = value.into();
        validate_component(&value, PropertyComponentKind::Name)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PropertyName {
    type Error = BlockStateError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PropertyName> for String {
    fn from(value: PropertyName) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PropertyValue(String);

impl PropertyValue {
    pub fn new(value: impl Into<String>) -> Result<Self, BlockStateError> {
        let value = value.into();
        validate_component(&value, PropertyComponentKind::Value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PropertyValue {
    type Error = BlockStateError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PropertyValue> for String {
    fn from(value: PropertyValue) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PropertySchemaRepr", into = "PropertySchemaRepr")]
pub struct PropertySchema {
    name: PropertyName,
    values: Vec<PropertyValue>,
    default_index: u16,
}

impl PropertySchema {
    pub fn new(
        name: PropertyName,
        values: Vec<PropertyValue>,
        default: &PropertyValue,
    ) -> Result<Self, BlockStateError> {
        if values.is_empty() {
            return Err(BlockStateError::EmptyPropertyValues { name });
        }
        let mut unique_values = BTreeSet::new();
        for value in &values {
            if !unique_values.insert(value.clone()) {
                return Err(BlockStateError::DuplicatePropertyValue {
                    name,
                    value: value.clone(),
                });
            }
        }
        let default_index = values
            .iter()
            .position(|value| value == default)
            .ok_or_else(|| BlockStateError::MissingDefaultValue {
                name: name.clone(),
                value: default.clone(),
            })?;
        let default_index =
            u16::try_from(default_index).map_err(|_| BlockStateError::TooManyPropertyValues {
                name: name.clone(),
                actual: values.len(),
            })?;
        if values.len() > usize::from(u16::MAX) + 1 {
            return Err(BlockStateError::TooManyPropertyValues {
                name,
                actual: values.len(),
            });
        }
        Ok(Self {
            name,
            values,
            default_index,
        })
    }

    pub const fn name(&self) -> &PropertyName {
        &self.name
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = &PropertyValue> {
        self.values.iter()
    }

    pub fn default_value(&self) -> &PropertyValue {
        &self.values[usize::from(self.default_index)]
    }

    fn value_index(&self, value: &PropertyValue) -> Option<u32> {
        self.values
            .iter()
            .position(|candidate| candidate == value)
            .and_then(|index| u32::try_from(index).ok())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PropertySchemaRepr {
    name: PropertyName,
    values: Vec<PropertyValue>,
    default_index: u16,
}

impl TryFrom<PropertySchemaRepr> for PropertySchema {
    type Error = BlockStateError;

    fn try_from(value: PropertySchemaRepr) -> Result<Self, Self::Error> {
        let default = value
            .values
            .get(usize::from(value.default_index))
            .cloned()
            .ok_or_else(|| BlockStateError::DefaultIndexOutOfRange {
                name: value.name.clone(),
                index: value.default_index,
            })?;
        Self::new(value.name, value.values, &default)
    }
}

impl From<PropertySchema> for PropertySchemaRepr {
    fn from(value: PropertySchema) -> Self {
        Self {
            name: value.name,
            values: value.values,
            default_index: value.default_index,
        }
    }
}

/// Schema-local dense state identity. It intentionally has no serialization implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateIndex(u32);

impl StateIndex {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyAssignment {
    name: PropertyName,
    value: PropertyValue,
}

impl PropertyAssignment {
    pub const fn new(name: PropertyName, value: PropertyValue) -> Self {
        Self { name, value }
    }

    pub const fn name(&self) -> &PropertyName {
        &self.name
    }

    pub const fn value(&self) -> &PropertyValue {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockState {
    block: PersistentId,
    properties: Vec<PropertyAssignment>,
}

impl BlockState {
    pub const fn new(block: PersistentId, properties: Vec<PropertyAssignment>) -> Self {
        Self { block, properties }
    }

    pub const fn block(&self) -> &PersistentId {
        &self.block
    }

    pub fn properties(&self) -> impl ExactSizeIterator<Item = &PropertyAssignment> {
        self.properties.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "BlockStateSchemaRepr", into = "BlockStateSchemaRepr")]
pub struct BlockStateSchema {
    block: PersistentId,
    properties: Vec<PropertySchema>,
    state_count: u32,
}

impl BlockStateSchema {
    pub fn new(
        block: PersistentId,
        properties: Vec<PropertySchema>,
    ) -> Result<Self, BlockStateError> {
        let mut names = BTreeSet::new();
        let mut state_count = 1_u64;
        for property in &properties {
            if !names.insert(property.name.clone()) {
                return Err(BlockStateError::DuplicateProperty {
                    name: property.name.clone(),
                });
            }
            state_count = state_count
                .checked_mul(property.values.len() as u64)
                .ok_or(BlockStateError::TooManyStates)?;
            if state_count > u64::from(u32::MAX) {
                return Err(BlockStateError::TooManyStates);
            }
        }
        Ok(Self {
            block,
            properties,
            state_count: state_count as u32,
        })
    }

    pub const fn block(&self) -> &PersistentId {
        &self.block
    }

    pub fn properties(&self) -> impl ExactSizeIterator<Item = &PropertySchema> {
        self.properties.iter()
    }

    pub const fn state_count(&self) -> u32 {
        self.state_count
    }

    pub fn default_state(&self) -> BlockState {
        BlockState::new(
            self.block.clone(),
            self.properties
                .iter()
                .map(|property| {
                    PropertyAssignment::new(property.name.clone(), property.default_value().clone())
                })
                .collect(),
        )
    }

    pub fn state_at(&self, index: StateIndex) -> Option<BlockState> {
        if index.get() >= self.state_count {
            return None;
        }
        let mut remaining = index.get();
        let mut value_indices = vec![0_usize; self.properties.len()];
        for (property, value_index) in self.properties.iter().zip(value_indices.iter_mut()).rev() {
            let radix = u32::try_from(property.values.len()).ok()?;
            *value_index = usize::try_from(remaining % radix).ok()?;
            remaining /= radix;
        }
        let properties = self
            .properties
            .iter()
            .zip(value_indices)
            .map(|(property, value_index)| {
                PropertyAssignment::new(property.name.clone(), property.values[value_index].clone())
            })
            .collect();
        Some(BlockState::new(self.block.clone(), properties))
    }

    pub fn index_of(&self, state: &BlockState) -> Result<StateIndex, BlockStateError> {
        if state.block != self.block {
            return Err(BlockStateError::WrongBlock {
                expected: self.block.clone(),
                actual: state.block.clone(),
            });
        }
        if state.properties.len() != self.properties.len() {
            return Err(BlockStateError::WrongPropertyCount {
                expected: self.properties.len(),
                actual: state.properties.len(),
            });
        }

        let mut index = 0_u32;
        for (position, (schema, assignment)) in
            self.properties.iter().zip(&state.properties).enumerate()
        {
            if schema.name != assignment.name {
                return Err(BlockStateError::WrongProperty {
                    position,
                    expected: schema.name.clone(),
                    actual: assignment.name.clone(),
                });
            }
            let value_index = schema.value_index(&assignment.value).ok_or_else(|| {
                BlockStateError::UnknownPropertyValue {
                    name: schema.name.clone(),
                    value: assignment.value.clone(),
                }
            })?;
            let radix =
                u32::try_from(schema.values.len()).map_err(|_| BlockStateError::TooManyStates)?;
            index = index
                .checked_mul(radix)
                .and_then(|value| value.checked_add(value_index))
                .ok_or(BlockStateError::TooManyStates)?;
        }
        Ok(StateIndex::new(index))
    }

    pub fn set_value(
        &self,
        state: &BlockState,
        name: &PropertyName,
        value: &PropertyValue,
    ) -> Result<BlockState, BlockStateError> {
        self.index_of(state)?;
        let position = self
            .properties
            .iter()
            .position(|property| property.name() == name)
            .ok_or_else(|| BlockStateError::UnknownProperty { name: name.clone() })?;
        let property = &self.properties[position];
        if property.value_index(value).is_none() {
            return Err(BlockStateError::UnknownPropertyValue {
                name: name.clone(),
                value: value.clone(),
            });
        }
        if state.properties[position].value() == value {
            return Ok(state.clone());
        }
        let mut changed = state.clone();
        changed.properties[position] = PropertyAssignment::new(name.clone(), value.clone());
        let index = self.index_of(&changed)?;
        self.state_at(index)
            .ok_or(BlockStateError::CanonicalStateMissing { index })
    }

    pub fn apply_component_patch(
        &self,
        state: &BlockState,
        patch: &BTreeMap<String, String>,
    ) -> Result<BlockState, BlockStateError> {
        self.index_of(state)?;
        let mut changed = state.clone();
        for (name, value) in patch {
            let Ok(name) = PropertyName::new(name.clone()) else {
                continue;
            };
            let Ok(value) = PropertyValue::new(value.clone()) else {
                continue;
            };
            let Ok(next) = self.set_value(&changed, &name, &value) else {
                continue;
            };
            changed = next;
        }
        Ok(changed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BlockStateSchemaRepr {
    block: PersistentId,
    properties: Vec<PropertySchema>,
    state_count: u32,
}

impl TryFrom<BlockStateSchemaRepr> for BlockStateSchema {
    type Error = BlockStateError;

    fn try_from(value: BlockStateSchemaRepr) -> Result<Self, Self::Error> {
        let schema = Self::new(value.block, value.properties)?;
        if schema.state_count != value.state_count {
            return Err(BlockStateError::StateCountMismatch {
                declared: value.state_count,
                calculated: schema.state_count,
            });
        }
        Ok(schema)
    }
}

impl From<BlockStateSchema> for BlockStateSchemaRepr {
    fn from(value: BlockStateSchema) -> Self {
        Self {
            block: value.block,
            properties: value.properties,
            state_count: value.state_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyComponentKind {
    Name,
    Value,
}

fn validate_component(value: &str, kind: PropertyComponentKind) -> Result<(), BlockStateError> {
    if value.is_empty() {
        return Err(BlockStateError::EmptyComponent { kind });
    }
    for (index, character) in value.char_indices() {
        let valid = match kind {
            PropertyComponentKind::Name => matches!(character, 'a'..='z' | '0'..='9' | '_'),
            PropertyComponentKind::Value => {
                matches!(character, 'a'..='z' | '0'..='9' | '_' | '-')
            }
        };
        if !valid {
            return Err(BlockStateError::InvalidComponent {
                kind,
                character,
                index,
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlockStateError {
    #[error("block property {kind} cannot be empty")]
    EmptyComponent { kind: PropertyComponentKind },
    #[error("invalid block property {kind} character {character:?} at byte {index}")]
    InvalidComponent {
        kind: PropertyComponentKind,
        character: char,
        index: usize,
    },
    #[error("block property {name:?} has no values")]
    EmptyPropertyValues { name: PropertyName },
    #[error("block property {name:?} repeats value {value:?}")]
    DuplicatePropertyValue {
        name: PropertyName,
        value: PropertyValue,
    },
    #[error("block property {name:?} does not contain default value {value:?}")]
    MissingDefaultValue {
        name: PropertyName,
        value: PropertyValue,
    },
    #[error("block property {name:?} default index {index} is outside its values")]
    DefaultIndexOutOfRange { name: PropertyName, index: u16 },
    #[error("block property {name:?} has {actual} values, exceeding the supported maximum")]
    TooManyPropertyValues { name: PropertyName, actual: usize },
    #[error("block state schema repeats property {name:?}")]
    DuplicateProperty { name: PropertyName },
    #[error("block state schema has more than u32::MAX states")]
    TooManyStates,
    #[error("block state belongs to {actual}, expected {expected}")]
    WrongBlock {
        expected: PersistentId,
        actual: PersistentId,
    },
    #[error("block state has {actual} properties, expected {expected}")]
    WrongPropertyCount { expected: usize, actual: usize },
    #[error("block state schema does not contain property {name:?}")]
    UnknownProperty { name: PropertyName },
    #[error("property {position} is {actual:?}, expected {expected:?}")]
    WrongProperty {
        position: usize,
        expected: PropertyName,
        actual: PropertyName,
    },
    #[error("block property {name:?} does not contain value {value:?}")]
    UnknownPropertyValue {
        name: PropertyName,
        value: PropertyValue,
    },
    #[error("canonical block state {index:?} is absent")]
    CanonicalStateMissing { index: StateIndex },
    #[error("declared state count {declared} does not match calculated count {calculated}")]
    StateCountMismatch { declared: u32, calculated: u32 },
}

impl Display for PropertyComponentKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name => formatter.write_str("name"),
            Self::Value => formatter.write_str("value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::resource::ResourceId;

    fn property(name: &str, values: &[&str], default: &str) -> PropertySchema {
        let values = values
            .iter()
            .map(|value| PropertyValue::new(*value).unwrap())
            .collect::<Vec<_>>();
        PropertySchema::new(
            PropertyName::new(name).unwrap(),
            values,
            &PropertyValue::new(default).unwrap(),
        )
        .unwrap()
    }

    fn schema() -> BlockStateSchema {
        BlockStateSchema::new(
            PersistentId::new(ResourceId::minecraft("oak_log").unwrap()),
            vec![
                property("axis", &["x", "y", "z"], "y"),
                property("waterlogged", &["false", "true"], "false"),
            ],
        )
        .unwrap()
    }

    #[test]
    fn cartesian_states_round_trip_in_declared_order() {
        let schema = schema();
        assert_eq!(schema.state_count(), 6);
        for raw_index in 0..schema.state_count() {
            let index = StateIndex::new(raw_index);
            let state = schema.state_at(index).unwrap();
            assert_eq!(schema.index_of(&state).unwrap(), index);
        }
        let last = schema.state_at(StateIndex::new(5)).unwrap();
        let values = last
            .properties()
            .map(|property| property.value().as_str())
            .collect::<Vec<_>>();
        assert_eq!(values, ["z", "true"]);
    }

    #[test]
    fn default_state_uses_declared_property_defaults() {
        let schema = schema();
        let default = schema.default_state();
        let values = default
            .properties()
            .map(|property| property.value().as_str())
            .collect::<Vec<_>>();
        assert_eq!(values, ["y", "false"]);
        assert_eq!(schema.index_of(&default).unwrap().get(), 2);
    }

    #[test]
    fn malformed_schemas_and_states_are_rejected() {
        let duplicate = BlockStateSchema::new(
            PersistentId::new(ResourceId::minecraft("test").unwrap()),
            vec![
                property("axis", &["x", "y"], "x"),
                property("axis", &["x", "y"], "x"),
            ],
        );
        assert!(matches!(
            duplicate,
            Err(BlockStateError::DuplicateProperty { .. })
        ));
        assert!(PropertyName::new("Uppercase").is_err());
        assert!(
            property("axis", &["x", "y"], "x")
                .value_index(&PropertyValue::new("z").unwrap())
                .is_none()
        );
    }

    #[test]
    fn schema_deserialization_revalidates_derived_state_count() {
        let encoded = serde_json::to_string(&schema()).unwrap();
        let invalid = encoded.replace("\"state_count\":6", "\"state_count\":7");
        assert!(serde_json::from_str::<BlockStateSchema>(&invalid).is_err());
    }
}
