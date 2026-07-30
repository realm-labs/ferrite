use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::entity_state::metadata::MetadataSerializer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataAccessorDeclaration {
    pub declaring_class: &'static str,
    pub field: &'static str,
    pub slot: u8,
    pub serializer: MetadataSerializer,
}

impl MetadataAccessorDeclaration {
    const fn new(
        declaring_class: &'static str,
        field: &'static str,
        slot: u8,
        serializer: MetadataSerializer,
    ) -> Self {
        Self {
            declaring_class,
            field,
            slot,
            serializer,
        }
    }
}

include!(concat!(
    env!("OUT_DIR"),
    "/minecraft_java_26_2_entity_metadata_accessors.rs"
));

#[must_use]
pub const fn declarations() -> &'static [MetadataAccessorDeclaration] {
    ACCESSORS
}

pub fn schema_for_hierarchy(
    declaring_classes: &[&str],
) -> Result<BTreeMap<u8, MetadataSerializer>, MetadataAccessorSchemaError> {
    let classes = declaring_classes.iter().copied().collect::<BTreeSet<_>>();
    let mut schema = BTreeMap::new();
    for declaration in ACCESSORS
        .iter()
        .filter(|declaration| classes.contains(declaration.declaring_class))
    {
        if let Some(previous) = schema.insert(declaration.slot, declaration.serializer) {
            return Err(MetadataAccessorSchemaError::SlotCollision {
                slot: declaration.slot,
                first: previous,
                second: declaration.serializer,
            });
        }
    }
    Ok(schema)
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MetadataAccessorSchemaError {
    #[error("metadata hierarchy assigns slot {slot} to both {first:?} and {second:?}")]
    SlotCollision {
        slot: u8,
        first: MetadataSerializer,
        second: MetadataSerializer,
    },
}
