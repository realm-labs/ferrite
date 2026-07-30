use thiserror::Error;

use crate::java_26_2::play::serverbound::recipe_book::packet::{
    PlaceRecipe, RecipeBookChangeSettings, RecipeBookSeenRecipe, RecipeBookType,
};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecipeBookServerboundCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("recipe-book type ordinal {value} is invalid")]
    InvalidBookType { value: i32 },
}

pub fn decode_place_recipe(
    reader: &mut WireReader<'_>,
) -> Result<PlaceRecipe, RecipeBookServerboundCodecError> {
    Ok(PlaceRecipe {
        container_id: reader.read_var_i32()?,
        display_id: reader.read_var_i32()?,
        use_maximum_items: reader.read_bool()?,
    })
}

pub fn encode_place_recipe(
    writer: &mut WireWriter,
    packet: PlaceRecipe,
) -> Result<(), RecipeBookServerboundCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_var_i32(packet.display_id)?;
    writer.write_bool(packet.use_maximum_items)?;
    Ok(())
}

pub fn decode_change_settings(
    reader: &mut WireReader<'_>,
) -> Result<RecipeBookChangeSettings, RecipeBookServerboundCodecError> {
    let value = reader.read_var_i32()?;
    let book_type = RecipeBookType::from_wire(value)
        .ok_or(RecipeBookServerboundCodecError::InvalidBookType { value })?;
    Ok(RecipeBookChangeSettings {
        book_type,
        open: reader.read_bool()?,
        filtering: reader.read_bool()?,
    })
}

pub fn encode_change_settings(
    writer: &mut WireWriter,
    packet: RecipeBookChangeSettings,
) -> Result<(), RecipeBookServerboundCodecError> {
    writer.write_var_i32(packet.book_type.to_wire())?;
    writer.write_bool(packet.open)?;
    writer.write_bool(packet.filtering)?;
    Ok(())
}

pub fn decode_seen_recipe(
    reader: &mut WireReader<'_>,
) -> Result<RecipeBookSeenRecipe, RecipeBookServerboundCodecError> {
    Ok(RecipeBookSeenRecipe {
        display_id: reader.read_var_i32()?,
    })
}

pub fn encode_seen_recipe(
    writer: &mut WireWriter,
    packet: RecipeBookSeenRecipe,
) -> Result<(), RecipeBookServerboundCodecError> {
    writer.write_var_i32(packet.display_id)?;
    Ok(())
}
