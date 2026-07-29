use crate::java_26_2::play::clientbound::recipe::slot::{
    SlotDisplay, read as read_slot, write as write_slot,
};
use crate::java_26_2::play::clientbound::recipe::{RecipeError, write_count};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::registry::{PlayRegistries, RECIPE_DISPLAY};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, PartialEq)]
pub enum RecipeDisplay {
    CraftingShapeless {
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    CraftingShaped {
        width: i32,
        height: i32,
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    Furnace {
        ingredient: SlotDisplay,
        fuel: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
        duration: i32,
        experience: f32,
    },
    Stonecutter {
        input: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    Smithing {
        template: SlotDisplay,
        base: SlotDisplay,
        addition: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
}

pub(crate) fn read(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
    depth: usize,
) -> Result<RecipeDisplay, RecipeError> {
    let raw_id = reader.read_var_i32()?;
    let identity = context.registries.resolve(RECIPE_DISPLAY, raw_id)?;
    match identity.to_string().as_str() {
        "minecraft:crafting_shapeless" => {
            let ingredients = read_slots(reader, context, depth)?;
            Ok(RecipeDisplay::CraftingShapeless {
                ingredients,
                result: SlotDisplay::read_for_display(reader, context, depth)?,
                crafting_station: SlotDisplay::read_for_display(reader, context, depth)?,
            })
        }
        "minecraft:crafting_shaped" => {
            let width = reader.read_var_i32()?;
            let height = reader.read_var_i32()?;
            let ingredients = read_slots(reader, context, depth)?;
            validate_dimensions(width, height, ingredients.len())?;
            Ok(RecipeDisplay::CraftingShaped {
                width,
                height,
                ingredients,
                result: SlotDisplay::read_for_display(reader, context, depth)?,
                crafting_station: SlotDisplay::read_for_display(reader, context, depth)?,
            })
        }
        "minecraft:furnace" => Ok(RecipeDisplay::Furnace {
            ingredient: SlotDisplay::read_for_display(reader, context, depth)?,
            fuel: SlotDisplay::read_for_display(reader, context, depth)?,
            result: SlotDisplay::read_for_display(reader, context, depth)?,
            crafting_station: SlotDisplay::read_for_display(reader, context, depth)?,
            duration: reader.read_var_i32()?,
            experience: reader.read_f32()?,
        }),
        "minecraft:stonecutter" => Ok(RecipeDisplay::Stonecutter {
            input: SlotDisplay::read_for_display(reader, context, depth)?,
            result: SlotDisplay::read_for_display(reader, context, depth)?,
            crafting_station: SlotDisplay::read_for_display(reader, context, depth)?,
        }),
        "minecraft:smithing" => Ok(RecipeDisplay::Smithing {
            template: SlotDisplay::read_for_display(reader, context, depth)?,
            base: SlotDisplay::read_for_display(reader, context, depth)?,
            addition: SlotDisplay::read_for_display(reader, context, depth)?,
            result: SlotDisplay::read_for_display(reader, context, depth)?,
            crafting_station: SlotDisplay::read_for_display(reader, context, depth)?,
        }),
        _ => Err(RecipeError::RecipeDisplayMismatch { identity }),
    }
}

pub(crate) fn write(
    writer: &mut WireWriter,
    display: &RecipeDisplay,
    registries: &PlayRegistries,
    depth: usize,
) -> Result<(), RecipeError> {
    let identity = Identifier::parse(display_identity(display))?;
    writer.write_var_i32(registries.raw_id(RECIPE_DISPLAY, &identity)?)?;
    match display {
        RecipeDisplay::CraftingShapeless {
            ingredients,
            result,
            crafting_station,
        } => {
            write_slots(writer, ingredients, registries, depth)?;
            result.write_for_display(writer, registries, depth)?;
            crafting_station.write_for_display(writer, registries, depth)?;
        }
        RecipeDisplay::CraftingShaped {
            width,
            height,
            ingredients,
            result,
            crafting_station,
        } => {
            validate_dimensions(*width, *height, ingredients.len())?;
            writer.write_var_i32(*width)?;
            writer.write_var_i32(*height)?;
            write_slots(writer, ingredients, registries, depth)?;
            result.write_for_display(writer, registries, depth)?;
            crafting_station.write_for_display(writer, registries, depth)?;
        }
        RecipeDisplay::Furnace {
            ingredient,
            fuel,
            result,
            crafting_station,
            duration,
            experience,
        } => {
            ingredient.write_for_display(writer, registries, depth)?;
            fuel.write_for_display(writer, registries, depth)?;
            result.write_for_display(writer, registries, depth)?;
            crafting_station.write_for_display(writer, registries, depth)?;
            writer.write_var_i32(*duration)?;
            writer.write_f32(*experience)?;
        }
        RecipeDisplay::Stonecutter {
            input,
            result,
            crafting_station,
        } => {
            input.write_for_display(writer, registries, depth)?;
            result.write_for_display(writer, registries, depth)?;
            crafting_station.write_for_display(writer, registries, depth)?;
        }
        RecipeDisplay::Smithing {
            template,
            base,
            addition,
            result,
            crafting_station,
        } => {
            template.write_for_display(writer, registries, depth)?;
            base.write_for_display(writer, registries, depth)?;
            addition.write_for_display(writer, registries, depth)?;
            result.write_for_display(writer, registries, depth)?;
            crafting_station.write_for_display(writer, registries, depth)?;
        }
    }
    Ok(())
}

impl SlotDisplay {
    fn read_for_display(
        reader: &mut WireReader<'_>,
        context: PlayDecodeContext<'_>,
        depth: usize,
    ) -> Result<Self, RecipeError> {
        read_slot(reader, context, depth + 1)
    }

    fn write_for_display(
        &self,
        writer: &mut WireWriter,
        registries: &PlayRegistries,
        depth: usize,
    ) -> Result<(), RecipeError> {
        write_slot(writer, self, registries, depth + 1)
    }
}

fn read_slots(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
    depth: usize,
) -> Result<Vec<SlotDisplay>, RecipeError> {
    let count = reader.read_count("recipe display ingredients", reader.remaining())?;
    let mut slots = Vec::with_capacity(count);
    for _ in 0..count {
        slots.push(SlotDisplay::read_for_display(reader, context, depth)?);
    }
    Ok(slots)
}

fn write_slots(
    writer: &mut WireWriter,
    slots: &[SlotDisplay],
    registries: &PlayRegistries,
    depth: usize,
) -> Result<(), RecipeError> {
    write_count(writer, "recipe display ingredients", slots.len())?;
    for slot in slots {
        slot.write_for_display(writer, registries, depth)?;
    }
    Ok(())
}

fn validate_dimensions(width: i32, height: i32, ingredients: usize) -> Result<(), RecipeError> {
    let expected = usize::try_from(width).ok().and_then(|width| {
        usize::try_from(height)
            .ok()
            .and_then(|height| width.checked_mul(height))
    });
    if expected == Some(ingredients) {
        Ok(())
    } else {
        Err(RecipeError::ShapedDimensions {
            width,
            height,
            ingredients,
        })
    }
}

fn display_identity(display: &RecipeDisplay) -> &'static str {
    match display {
        RecipeDisplay::CraftingShapeless { .. } => "minecraft:crafting_shapeless",
        RecipeDisplay::CraftingShaped { .. } => "minecraft:crafting_shaped",
        RecipeDisplay::Furnace { .. } => "minecraft:furnace",
        RecipeDisplay::Stonecutter { .. } => "minecraft:stonecutter",
        RecipeDisplay::Smithing { .. } => "minecraft:smithing",
    }
}
