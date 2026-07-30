use crate::java_26_2::play::clientbound::recipe::book::{PlaceGhostRecipe, RecipeBookRemove};
use crate::java_26_2::play::clientbound::recipe::{RecipeError, display, write_count};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::registry::PlayRegistries;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const COLLECTION_ALLOCATION_CAPACITY: usize = 65_536;

pub(crate) fn read_ghost(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<PlaceGhostRecipe, RecipeError> {
    Ok(PlaceGhostRecipe {
        container_id: reader.read_var_i32()?,
        display: display::read(reader, context, 0)?,
    })
}

pub(crate) fn write_ghost(
    writer: &mut WireWriter,
    packet: &PlaceGhostRecipe,
    registries: &PlayRegistries,
) -> Result<(), RecipeError> {
    writer.write_var_i32(packet.container_id)?;
    display::write(writer, &packet.display, registries, 0)
}

pub(crate) fn read_remove(reader: &mut WireReader<'_>) -> Result<RecipeBookRemove, RecipeError> {
    let count = reader.read_count("removed recipe displays", reader.remaining())?;
    let mut display_ids = Vec::with_capacity(count.min(COLLECTION_ALLOCATION_CAPACITY));
    for _ in 0..count {
        display_ids.push(reader.read_var_i32()?);
    }
    Ok(RecipeBookRemove { display_ids })
}

pub(crate) fn write_remove(
    writer: &mut WireWriter,
    packet: &RecipeBookRemove,
) -> Result<(), RecipeError> {
    write_count(writer, "removed recipe displays", packet.display_ids.len())?;
    for display_id in &packet.display_ids {
        writer.write_var_i32(*display_id)?;
    }
    Ok(())
}
