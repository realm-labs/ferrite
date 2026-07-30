use crate::java_26_2::play::clientbound::recipe::{RecipeError, write_count};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::item::{
    DataComponentPatch, read_component_patch, write_component_patch,
};
use crate::java_26_2::play::registry::{
    DATA_COMPONENT_TYPE, ITEM, PlayRegistries, SLOT_DISPLAY, TRIM_PATTERN,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierReadError};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_SLOT_DEPTH: usize = 512;

#[derive(Debug, Clone, PartialEq)]
pub enum SlotDisplay {
    Empty,
    AnyFuel,
    WithAnyPotion,
    OnlyWithComponent {
        source: Box<SlotDisplay>,
        component: Identifier,
    },
    Item(Identifier),
    ItemStack(ItemStackTemplate),
    Tag(Identifier),
    Dyed {
        dye: Box<SlotDisplay>,
        target: Box<SlotDisplay>,
    },
    SmithingTrim {
        base: Box<SlotDisplay>,
        material: Box<SlotDisplay>,
        pattern: Identifier,
    },
    WithRemainder {
        input: Box<SlotDisplay>,
        remainder: Box<SlotDisplay>,
    },
    Composite(Vec<SlotDisplay>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackTemplate {
    pub item: Identifier,
    pub count: i32,
    pub components: DataComponentPatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSet {
    Tag(Identifier),
    Direct(Vec<Identifier>),
}

pub(crate) fn read(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
    depth: usize,
) -> Result<SlotDisplay, RecipeError> {
    enum Task {
        Node(usize),
        OnlyWithComponent,
        Dyed,
        SmithingTrim,
        WithRemainder,
        Composite(usize),
    }

    let mut tasks = vec![Task::Node(depth)];
    let mut values = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Node(node_depth) => {
                require_depth(node_depth)?;
                let raw_id = reader.read_var_i32()?;
                let identity = context.registries.resolve(SLOT_DISPLAY, raw_id)?;
                match (identity.namespace(), identity.path()) {
                    ("minecraft", "empty") => values.push(SlotDisplay::Empty),
                    ("minecraft", "any_fuel") => values.push(SlotDisplay::AnyFuel),
                    ("minecraft", "with_any_potion") => {
                        values.push(SlotDisplay::WithAnyPotion);
                    }
                    ("minecraft", "only_with_component") => {
                        tasks.push(Task::OnlyWithComponent);
                        tasks.push(Task::Node(node_depth + 1));
                    }
                    ("minecraft", "item") => values.push(SlotDisplay::Item(
                        context.registries.resolve(ITEM, reader.read_var_i32()?)?,
                    )),
                    ("minecraft", "item_stack") => {
                        values.push(SlotDisplay::ItemStack(read_item_stack(reader, context)?));
                    }
                    ("minecraft", "tag") => {
                        values.push(SlotDisplay::Tag(read_identifier(reader)?));
                    }
                    ("minecraft", "dyed") => {
                        tasks.push(Task::Dyed);
                        tasks.push(Task::Node(node_depth + 1));
                        tasks.push(Task::Node(node_depth + 1));
                    }
                    ("minecraft", "smithing_trim") => {
                        tasks.push(Task::SmithingTrim);
                        tasks.push(Task::Node(node_depth + 1));
                        tasks.push(Task::Node(node_depth + 1));
                    }
                    ("minecraft", "with_remainder") => {
                        tasks.push(Task::WithRemainder);
                        tasks.push(Task::Node(node_depth + 1));
                        tasks.push(Task::Node(node_depth + 1));
                    }
                    ("minecraft", "composite") => {
                        let count =
                            reader.read_count("composite slot displays", reader.remaining())?;
                        tasks.push(Task::Composite(count));
                        tasks.extend((0..count).map(|_| Task::Node(node_depth + 1)));
                    }
                    _ => return Err(RecipeError::SlotDisplayMismatch { identity }),
                }
            }
            Task::OnlyWithComponent => {
                let source = pop_value(&mut values);
                let component = context
                    .registries
                    .resolve(DATA_COMPONENT_TYPE, reader.read_var_i32()?)?;
                values.push(SlotDisplay::OnlyWithComponent {
                    source: Box::new(source),
                    component,
                });
            }
            Task::Dyed => {
                let target = pop_value(&mut values);
                let dye = pop_value(&mut values);
                values.push(SlotDisplay::Dyed {
                    dye: Box::new(dye),
                    target: Box::new(target),
                });
            }
            Task::SmithingTrim => {
                let material = pop_value(&mut values);
                let base = pop_value(&mut values);
                let pattern = context
                    .registries
                    .resolve(TRIM_PATTERN, reader.read_var_i32()?)?;
                values.push(SlotDisplay::SmithingTrim {
                    base: Box::new(base),
                    material: Box::new(material),
                    pattern,
                });
            }
            Task::WithRemainder => {
                let remainder = pop_value(&mut values);
                let input = pop_value(&mut values);
                values.push(SlotDisplay::WithRemainder {
                    input: Box::new(input),
                    remainder: Box::new(remainder),
                });
            }
            Task::Composite(count) => {
                let start = values.len() - count;
                let contents = values.split_off(start);
                values.push(SlotDisplay::Composite(contents));
            }
        }
    }
    debug_assert_eq!(values.len(), 1);
    Ok(pop_value(&mut values))
}

pub(crate) fn write(
    writer: &mut WireWriter,
    display: &SlotDisplay,
    registries: &PlayRegistries,
    depth: usize,
) -> Result<(), RecipeError> {
    enum Task<'a> {
        Node(&'a SlotDisplay, usize),
        Component(&'a Identifier),
        Pattern(&'a Identifier),
    }

    let mut tasks = vec![Task::Node(display, depth)];
    while let Some(task) = tasks.pop() {
        match task {
            Task::Node(node, node_depth) => {
                require_depth(node_depth)?;
                let identity = Identifier::parse(slot_identity(node))?;
                writer.write_var_i32(registries.raw_id(SLOT_DISPLAY, &identity)?)?;
                match node {
                    SlotDisplay::Empty | SlotDisplay::AnyFuel | SlotDisplay::WithAnyPotion => {}
                    SlotDisplay::OnlyWithComponent { source, component } => {
                        tasks.push(Task::Component(component));
                        tasks.push(Task::Node(source, node_depth + 1));
                    }
                    SlotDisplay::Item(item) => {
                        writer.write_var_i32(registries.raw_id(ITEM, item)?)?;
                    }
                    SlotDisplay::ItemStack(stack) => {
                        write_item_stack(writer, stack, registries)?;
                    }
                    SlotDisplay::Tag(tag) => tag.write(writer)?,
                    SlotDisplay::Dyed { dye, target } => {
                        tasks.push(Task::Node(target, node_depth + 1));
                        tasks.push(Task::Node(dye, node_depth + 1));
                    }
                    SlotDisplay::SmithingTrim {
                        base,
                        material,
                        pattern,
                    } => {
                        tasks.push(Task::Pattern(pattern));
                        tasks.push(Task::Node(material, node_depth + 1));
                        tasks.push(Task::Node(base, node_depth + 1));
                    }
                    SlotDisplay::WithRemainder { input, remainder } => {
                        tasks.push(Task::Node(remainder, node_depth + 1));
                        tasks.push(Task::Node(input, node_depth + 1));
                    }
                    SlotDisplay::Composite(contents) => {
                        write_count(writer, "composite slot displays", contents.len())?;
                        tasks.extend(
                            contents
                                .iter()
                                .rev()
                                .map(|content| Task::Node(content, node_depth + 1)),
                        );
                    }
                }
            }
            Task::Component(component) => {
                writer.write_var_i32(registries.raw_id(DATA_COMPONENT_TYPE, component)?)?;
            }
            Task::Pattern(pattern) => {
                writer.write_var_i32(registries.raw_id(TRIM_PATTERN, pattern)?)?;
            }
        }
    }
    Ok(())
}

fn pop_value(values: &mut Vec<SlotDisplay>) -> SlotDisplay {
    values
        .pop()
        .unwrap_or_else(|| unreachable!("slot-display traversal maintains its value stack"))
}

pub(crate) fn read_holder_set(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<HolderSet, RecipeError> {
    let encoded = reader.read_var_i32()?;
    match encoded {
        0 => Ok(HolderSet::Tag(read_identifier(reader)?)),
        value if value > 0 => {
            let count =
                usize::try_from(value - 1).map_err(|_| RecipeError::InvalidHolderSet { value })?;
            if count > reader.remaining() {
                return Err(WireError::LengthLimit {
                    field: "direct holder set",
                    length: count,
                    maximum: reader.remaining(),
                }
                .into());
            }
            let mut values = Vec::with_capacity(count);
            for _ in 0..count {
                values.push(registries.resolve(ITEM, reader.read_var_i32()?)?);
            }
            Ok(HolderSet::Direct(values))
        }
        value => Err(RecipeError::InvalidHolderSet { value }),
    }
}

pub(crate) fn write_holder_set(
    writer: &mut WireWriter,
    set: &HolderSet,
    registries: &PlayRegistries,
) -> Result<(), RecipeError> {
    match set {
        HolderSet::Tag(tag) => {
            writer.write_var_i32(0)?;
            tag.write(writer)?;
        }
        HolderSet::Direct(values) => {
            let encoded = i32::try_from(values.len())
                .ok()
                .and_then(|count| count.checked_add(1))
                .ok_or(RecipeError::InvalidHolderSet { value: i32::MAX })?;
            writer.write_var_i32(encoded)?;
            for value in values {
                writer.write_var_i32(registries.raw_id(ITEM, value)?)?;
            }
        }
    }
    Ok(())
}

fn read_item_stack(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<ItemStackTemplate, RecipeError> {
    let item = context.registries.resolve(ITEM, reader.read_var_i32()?)?;
    let count = reader.read_var_i32()?;
    Ok(ItemStackTemplate {
        item,
        count,
        components: read_component_patch(reader, context)?,
    })
}

fn write_item_stack(
    writer: &mut WireWriter,
    stack: &ItemStackTemplate,
    registries: &PlayRegistries,
) -> Result<(), RecipeError> {
    writer.write_var_i32(registries.raw_id(ITEM, &stack.item)?)?;
    writer.write_var_i32(stack.count)?;
    write_component_patch(writer, &stack.components, registries)?;
    Ok(())
}

fn slot_identity(display: &SlotDisplay) -> &'static str {
    match display {
        SlotDisplay::Empty => "minecraft:empty",
        SlotDisplay::AnyFuel => "minecraft:any_fuel",
        SlotDisplay::WithAnyPotion => "minecraft:with_any_potion",
        SlotDisplay::OnlyWithComponent { .. } => "minecraft:only_with_component",
        SlotDisplay::Item(_) => "minecraft:item",
        SlotDisplay::ItemStack(_) => "minecraft:item_stack",
        SlotDisplay::Tag(_) => "minecraft:tag",
        SlotDisplay::Dyed { .. } => "minecraft:dyed",
        SlotDisplay::SmithingTrim { .. } => "minecraft:smithing_trim",
        SlotDisplay::WithRemainder { .. } => "minecraft:with_remainder",
        SlotDisplay::Composite(_) => "minecraft:composite",
    }
}

fn require_depth(depth: usize) -> Result<(), RecipeError> {
    if depth >= MAX_SLOT_DEPTH {
        Err(RecipeError::SlotDisplayDepth {
            maximum: MAX_SLOT_DEPTH,
        })
    } else {
        Ok(())
    }
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, RecipeError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
