use thiserror::Error;

use crate::java_26_2::play::serverbound::inventory_auxiliary::packet::{
    BundleItemSelected, EditBook, SeenAdvancements,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_BOOK_PAGES: usize = 100;
const MAX_PAGE_CODE_UNITS: usize = 1_024;
const MAX_TITLE_CODE_UNITS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InventoryAuxiliaryCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error("bundle selected-content index {selected} is below -1")]
    InvalidBundleSelection { selected: i32 },
    #[error("seen-advancements action ordinal {action} is invalid")]
    InvalidAdvancementAction { action: i32 },
}

pub fn decode_bundle_selection(
    reader: &mut WireReader<'_>,
) -> Result<BundleItemSelected, InventoryAuxiliaryCodecError> {
    let slot = reader.read_var_i32()?;
    let selected = reader.read_var_i32()?;
    require_bundle_selection(selected)?;
    Ok(BundleItemSelected { slot, selected })
}

pub fn encode_bundle_selection(
    writer: &mut WireWriter,
    packet: BundleItemSelected,
) -> Result<(), InventoryAuxiliaryCodecError> {
    require_bundle_selection(packet.selected)?;
    writer.write_var_i32(packet.slot)?;
    writer.write_var_i32(packet.selected)?;
    Ok(())
}

pub fn decode_edit_book(
    reader: &mut WireReader<'_>,
) -> Result<EditBook, InventoryAuxiliaryCodecError> {
    let slot = reader.read_var_i32()?;
    let page_count = reader.read_count("book pages", MAX_BOOK_PAGES)?;
    let mut pages = Vec::with_capacity(page_count);
    for _ in 0..page_count {
        pages.push(reader.read_utf(MAX_PAGE_CODE_UNITS)?.into_owned());
    }
    let title = reader
        .read_bool()?
        .then(|| reader.read_utf(MAX_TITLE_CODE_UNITS))
        .transpose()?
        .map(|title| title.into_owned());
    Ok(EditBook { slot, pages, title })
}

pub fn encode_edit_book(
    writer: &mut WireWriter,
    packet: &EditBook,
) -> Result<(), InventoryAuxiliaryCodecError> {
    writer.write_var_i32(packet.slot)?;
    writer.write_count("book pages", packet.pages.len(), MAX_BOOK_PAGES)?;
    for page in &packet.pages {
        writer.write_utf(page, MAX_PAGE_CODE_UNITS)?;
    }
    writer.write_bool(packet.title.is_some())?;
    if let Some(title) = &packet.title {
        writer.write_utf(title, MAX_TITLE_CODE_UNITS)?;
    }
    Ok(())
}

pub fn decode_seen_advancements(
    reader: &mut WireReader<'_>,
) -> Result<SeenAdvancements, InventoryAuxiliaryCodecError> {
    match reader.read_var_i32()? {
        0 => Ok(SeenAdvancements::OpenedTab(read_identifier(reader)?)),
        1 => Ok(SeenAdvancements::ClosedScreen),
        action => Err(InventoryAuxiliaryCodecError::InvalidAdvancementAction { action }),
    }
}

pub fn encode_seen_advancements(
    writer: &mut WireWriter,
    packet: &SeenAdvancements,
) -> Result<(), InventoryAuxiliaryCodecError> {
    match packet {
        SeenAdvancements::OpenedTab(identifier) => {
            writer.write_var_i32(0)?;
            identifier.write(writer)?;
        }
        SeenAdvancements::ClosedScreen => writer.write_var_i32(1)?,
    }
    Ok(())
}

fn require_bundle_selection(selected: i32) -> Result<(), InventoryAuxiliaryCodecError> {
    if selected < -1 {
        Err(InventoryAuxiliaryCodecError::InvalidBundleSelection { selected })
    } else {
        Ok(())
    }
}

fn read_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Identifier, InventoryAuxiliaryCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
