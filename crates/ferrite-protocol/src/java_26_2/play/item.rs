//! Shared Java 26.2 item-stack and data-component wire forms.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::context::{ComponentValueError, PlayDecodeContext};
use crate::java_26_2::play::registry::{
    DATA_COMPONENT_TYPE, ITEM, PlayRegistries, PlayRegistryError,
};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentPatch {
    pub added: Vec<EncodedComponentValue>,
    pub removed: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedComponentValue {
    pub component: Identifier,
    pub encoded_value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackContents {
    pub item: Identifier,
    pub count: i32,
    pub components: DataComponentPatch,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ItemStack {
    #[default]
    Empty,
    Present(StackContents),
}

impl ItemStack {
    #[must_use]
    pub fn present(item: Identifier, count: i32, components: DataComponentPatch) -> Self {
        if count <= 0 || is_air(&item) {
            Self::Empty
        } else {
            Self::Present(StackContents {
                item,
                count,
                components,
            })
        }
    }

    #[must_use]
    pub const fn contents(&self) -> Option<&StackContents> {
        match self {
            Self::Empty => None,
            Self::Present(contents) => Some(contents),
        }
    }

    #[must_use]
    pub const fn count(&self) -> i32 {
        match self {
            Self::Empty => 0,
            Self::Present(contents) => contents.count,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

pub(crate) fn read_optional_stack(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<ItemStack, ItemCodecError> {
    let count = reader.read_var_i32()?;
    if count <= 0 {
        return Ok(ItemStack::Empty);
    }
    let item = context.registries.resolve(ITEM, reader.read_var_i32()?)?;
    let components = read_component_patch(reader, context)?;
    Ok(ItemStack::present(item, count, components))
}

pub(crate) fn write_optional_stack(
    writer: &mut WireWriter,
    stack: &ItemStack,
    registries: &PlayRegistries,
) -> Result<(), ItemCodecError> {
    let ItemStack::Present(contents) = stack else {
        writer.write_var_i32(0)?;
        return Ok(());
    };
    if contents.count <= 0 || is_air(&contents.item) {
        writer.write_var_i32(0)?;
        return Ok(());
    }
    writer.write_var_i32(contents.count)?;
    writer.write_var_i32(registries.raw_id(ITEM, &contents.item)?)?;
    write_component_patch(writer, &contents.components, registries)
}

pub(crate) fn read_component_patch(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<DataComponentPatch, ItemCodecError> {
    let added_count = reader.read_var_i32()?;
    let removed_count = reader.read_var_i32()?;
    let combined = added_count.wrapping_add(removed_count);
    if combined < 0 {
        return Err(ItemCodecError::NegativeComponentCapacity {
            added: added_count,
            removed: removed_count,
            combined,
        });
    }
    let mut values = BTreeMap::new();
    for _ in 0..added_count {
        let component = context
            .registries
            .resolve(DATA_COMPONENT_TYPE, reader.read_var_i32()?)?;
        let encoded_value = context.component_values.decode_value(&component, reader)?;
        values.insert(component, Some(encoded_value));
    }
    for _ in 0..removed_count {
        let component = context
            .registries
            .resolve(DATA_COMPONENT_TYPE, reader.read_var_i32()?)?;
        values.insert(component, None);
    }
    let mut added = Vec::new();
    let mut removed = Vec::new();
    for (component, value) in values {
        if let Some(encoded_value) = value {
            added.push(EncodedComponentValue {
                component,
                encoded_value,
            });
        } else {
            removed.push(component);
        }
    }
    Ok(DataComponentPatch { added, removed })
}

pub(crate) fn write_component_patch(
    writer: &mut WireWriter,
    patch: &DataComponentPatch,
    registries: &PlayRegistries,
) -> Result<(), ItemCodecError> {
    writer.write_count(
        "added data components",
        patch.added.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    writer.write_count(
        "removed data components",
        patch.removed.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    let mut seen = BTreeSet::new();
    for value in &patch.added {
        require_unique(&mut seen, &value.component)?;
        writer.write_var_i32(registries.raw_id(DATA_COMPONENT_TYPE, &value.component)?)?;
        writer.write_bytes(&value.encoded_value)?;
    }
    for component in &patch.removed {
        require_unique(&mut seen, component)?;
        writer.write_var_i32(registries.raw_id(DATA_COMPONENT_TYPE, component)?)?;
    }
    Ok(())
}

fn require_unique(
    seen: &mut BTreeSet<Identifier>,
    component: &Identifier,
) -> Result<(), ItemCodecError> {
    if seen.insert(component.clone()) {
        Ok(())
    } else {
        Err(ItemCodecError::DuplicateComponent {
            component: component.clone(),
        })
    }
}

fn is_air(item: &Identifier) -> bool {
    item.namespace() == "minecraft" && item.path() == "air"
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ItemCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error(transparent)]
    ComponentValue(#[from] ComponentValueError),
    #[error("data-component patch repeats {component}")]
    DuplicateComponent { component: Identifier },
    #[error("component counts {added} + {removed} wrap to negative initial capacity {combined}")]
    NegativeComponentCapacity {
        added: i32,
        removed: i32,
        combined: i32,
    },
}
