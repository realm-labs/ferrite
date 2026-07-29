use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::play::clientbound::recipe::display::RecipeDisplay;
use crate::java_26_2::play::clientbound::recipe::slot::HolderSet;
use crate::java_26_2::play::context::{ComponentValueError, PlayDecodeContext};
use crate::java_26_2::play::registry::{
    ITEM, PlayRegistries, PlayRegistryError, RECIPE_BOOK_CATEGORY,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub mod display;
pub mod slot;

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeBookAdd {
    pub entries: Vec<RecipeBookEntry>,
    pub replace: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeBookEntry {
    pub display_id: i32,
    pub display: RecipeDisplay,
    pub group: Option<i32>,
    pub category: Identifier,
    pub crafting_requirements: Option<Vec<HolderSet>>,
    pub show_notification: bool,
    pub highlight: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecipeBookSettings {
    pub crafting_open: bool,
    pub crafting_filtering: bool,
    pub furnace_open: bool,
    pub furnace_filtering: bool,
    pub blast_furnace_open: bool,
    pub blast_furnace_filtering: bool,
    pub smoker_open: bool,
    pub smoker_filtering: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeProjection {
    pub properties: BTreeMap<Identifier, Vec<Identifier>>,
    pub stonecutter: Vec<StonecutterSelection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StonecutterSelection {
    pub input: HolderSet,
    pub display: slot::SlotDisplay,
}

pub(crate) fn read_book_add(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<RecipeBookAdd, RecipeError> {
    let count = reader.read_count("recipe-book entries", reader.remaining())?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let display_id = reader.read_var_i32()?;
        let recipe_display = display::read(reader, context, 0)?;
        let encoded_group = reader.read_var_i32()?;
        let group = match encoded_group {
            0 => None,
            value if value > 0 => Some(value - 1),
            value => return Err(RecipeError::InvalidOptionalGroup { value }),
        };
        let category_raw = reader.read_var_i32()?;
        let category = context
            .registries
            .resolve(RECIPE_BOOK_CATEGORY, category_raw)?;
        let crafting_requirements = if reader.read_bool()? {
            let requirement_count =
                reader.read_count("crafting requirements", reader.remaining())?;
            let mut requirements = Vec::with_capacity(requirement_count);
            for _ in 0..requirement_count {
                requirements.push(slot::read_holder_set(reader, context.registries)?);
            }
            Some(requirements)
        } else {
            None
        };
        let flags = reader.read_u8()?;
        entries.push(RecipeBookEntry {
            display_id,
            display: recipe_display,
            group,
            category,
            crafting_requirements,
            show_notification: flags & 1 != 0,
            highlight: flags & 2 != 0,
        });
    }
    Ok(RecipeBookAdd {
        entries,
        replace: reader.read_bool()?,
    })
}

pub(crate) fn write_book_add(
    writer: &mut WireWriter,
    packet: &RecipeBookAdd,
    registries: &PlayRegistries,
) -> Result<(), RecipeError> {
    write_count(writer, "recipe-book entries", packet.entries.len())?;
    for entry in &packet.entries {
        writer.write_var_i32(entry.display_id)?;
        display::write(writer, &entry.display, registries, 0)?;
        let encoded_group = entry.group.map_or(Ok(0), |group| {
            group
                .checked_add(1)
                .filter(|encoded| *encoded > 0)
                .ok_or(RecipeError::InvalidOptionalGroup { value: group })
        })?;
        writer.write_var_i32(encoded_group)?;
        writer.write_var_i32(registries.raw_id(RECIPE_BOOK_CATEGORY, &entry.category)?)?;
        writer.write_bool(entry.crafting_requirements.is_some())?;
        if let Some(requirements) = &entry.crafting_requirements {
            write_count(writer, "crafting requirements", requirements.len())?;
            for requirement in requirements {
                slot::write_holder_set(writer, requirement, registries)?;
            }
        }
        writer.write_u8(u8::from(entry.show_notification) | (u8::from(entry.highlight) << 1))?;
    }
    writer.write_bool(packet.replace)?;
    Ok(())
}

pub(crate) fn read_book_settings(
    reader: &mut WireReader<'_>,
) -> Result<RecipeBookSettings, RecipeError> {
    Ok(RecipeBookSettings {
        crafting_open: reader.read_bool()?,
        crafting_filtering: reader.read_bool()?,
        furnace_open: reader.read_bool()?,
        furnace_filtering: reader.read_bool()?,
        blast_furnace_open: reader.read_bool()?,
        blast_furnace_filtering: reader.read_bool()?,
        smoker_open: reader.read_bool()?,
        smoker_filtering: reader.read_bool()?,
    })
}

pub(crate) fn write_book_settings(
    writer: &mut WireWriter,
    settings: RecipeBookSettings,
) -> Result<(), RecipeError> {
    writer.write_bool(settings.crafting_open)?;
    writer.write_bool(settings.crafting_filtering)?;
    writer.write_bool(settings.furnace_open)?;
    writer.write_bool(settings.furnace_filtering)?;
    writer.write_bool(settings.blast_furnace_open)?;
    writer.write_bool(settings.blast_furnace_filtering)?;
    writer.write_bool(settings.smoker_open)?;
    writer.write_bool(settings.smoker_filtering)?;
    Ok(())
}

pub(crate) fn read_projection(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<RecipeProjection, RecipeError> {
    let property_count = reader.read_count("recipe property sets", reader.remaining())?;
    let mut properties = BTreeMap::new();
    for _ in 0..property_count {
        let key = read_identifier(reader)?;
        let item_count = reader.read_count("recipe property items", reader.remaining())?;
        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            items.push(context.registries.resolve(ITEM, reader.read_var_i32()?)?);
        }
        if properties.insert(key.clone(), items).is_some() {
            return Err(RecipeError::DuplicatePropertySet { key });
        }
    }
    let stonecutter_count = reader.read_count("stonecutter selections", reader.remaining())?;
    let mut stonecutter = Vec::with_capacity(stonecutter_count);
    for _ in 0..stonecutter_count {
        stonecutter.push(StonecutterSelection {
            input: slot::read_holder_set(reader, context.registries)?,
            display: slot::read(reader, context, 0)?,
        });
    }
    Ok(RecipeProjection {
        properties,
        stonecutter,
    })
}

pub(crate) fn write_projection(
    writer: &mut WireWriter,
    projection: &RecipeProjection,
    registries: &PlayRegistries,
) -> Result<(), RecipeError> {
    write_count(writer, "recipe property sets", projection.properties.len())?;
    for (key, items) in &projection.properties {
        key.write(writer)?;
        write_count(writer, "recipe property items", items.len())?;
        for item in items {
            writer.write_var_i32(registries.raw_id(ITEM, item)?)?;
        }
    }
    write_count(
        writer,
        "stonecutter selections",
        projection.stonecutter.len(),
    )?;
    for selection in &projection.stonecutter {
        slot::write_holder_set(writer, &selection.input, registries)?;
        slot::write(writer, &selection.display, registries, 0)?;
    }
    Ok(())
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, RecipeError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

pub(crate) fn write_count(
    writer: &mut WireWriter,
    field: &'static str,
    count: usize,
) -> Result<(), RecipeError> {
    writer.write_count(field, count, MAX_INFLATED_PACKET_LENGTH)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecipeError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidIdentifier(#[from] IdentifierError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error(transparent)]
    ComponentValue(#[from] ComponentValueError),
    #[error("recipe optional-group encoding {value} is negative or overflows group + 1")]
    InvalidOptionalGroup { value: i32 },
    #[error("recipe projection repeats property set {key}")]
    DuplicatePropertySet { key: Identifier },
    #[error("slot-display nesting exceeds {maximum}")]
    SlotDisplayDepth { maximum: usize },
    #[error("slot-display holder-set prefix {value} is negative")]
    InvalidHolderSet { value: i32 },
    #[error("recipe display {identity} has a mismatched payload")]
    RecipeDisplayMismatch { identity: Identifier },
    #[error("slot display {identity} has a mismatched payload")]
    SlotDisplayMismatch { identity: Identifier },
    #[error("shaped recipe dimensions {width}x{height} do not match {ingredients} ingredients")]
    ShapedDimensions {
        width: i32,
        height: i32,
        ingredients: usize,
    },
    #[error("data-component patch repeats {component}")]
    DuplicateComponent { component: Identifier },
}
