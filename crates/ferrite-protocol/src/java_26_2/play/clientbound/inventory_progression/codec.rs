use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::inventory_progression::packet::{
    Advancement, AdvancementFrame, AdvancementHolder, AdvancementProgress, DisplayInfo,
    MapDecoration, MapItemData, MapPatch, TagQuery, UpdateAdvancements,
};
use crate::java_26_2::play::context::PlayDecodeContext;
use crate::java_26_2::play::item::{ItemCodecError, read_stack_template, write_stack_template};
use crate::java_26_2::play::registry::{MAP_DECORATION_TYPE, PlayRegistries, PlayRegistryError};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_CRITERION_NAME: usize = 32_767;

pub(crate) fn read_map(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<MapItemData, InventoryProgressionCodecError> {
    let map_id = reader.read_var_i32()?;
    let scale = reader.read_i8()?;
    let locked = reader.read_bool()?;
    let decorations = if reader.read_bool()? {
        let count = reader.read_count("map decorations", reader.remaining())?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            let decoration_type =
                registries.resolve(MAP_DECORATION_TYPE, reader.read_var_i32()?)?;
            let x = reader.read_i8()?;
            let y = reader.read_i8()?;
            let rotation = (reader.read_i8()? as u8) & 0x0f;
            let name = if reader.read_bool()? {
                Some(read_component(reader, NbtQuota::Trusted)?)
            } else {
                None
            };
            values.push(MapDecoration {
                decoration_type,
                x,
                y,
                rotation,
                name,
            });
        }
        Some(values)
    } else {
        None
    };
    let width = reader.read_u8()?;
    let patch = if width == 0 {
        None
    } else {
        Some(MapPatch {
            width,
            height: reader.read_u8()?,
            start_x: reader.read_u8()?,
            start_y: reader.read_u8()?,
            colors: reader.read_byte_array(MAX_INFLATED_PACKET_LENGTH)?.to_vec(),
        })
    };
    Ok(MapItemData {
        map_id,
        scale,
        locked,
        decorations,
        patch,
    })
}

pub(crate) fn write_map(
    writer: &mut WireWriter,
    packet: &MapItemData,
    registries: &PlayRegistries,
) -> Result<(), InventoryProgressionCodecError> {
    writer.write_var_i32(packet.map_id)?;
    writer.write_i8(packet.scale)?;
    writer.write_bool(packet.locked)?;
    writer.write_bool(packet.decorations.is_some())?;
    if let Some(decorations) = &packet.decorations {
        writer.write_count(
            "map decorations",
            decorations.len(),
            MAX_INFLATED_PACKET_LENGTH,
        )?;
        for decoration in decorations {
            writer.write_var_i32(
                registries.raw_id(MAP_DECORATION_TYPE, &decoration.decoration_type)?,
            )?;
            writer.write_i8(decoration.x)?;
            writer.write_i8(decoration.y)?;
            writer.write_i8((decoration.rotation & 0x0f) as i8)?;
            writer.write_bool(decoration.name.is_some())?;
            if let Some(name) = &decoration.name {
                name.network_nbt().write(writer)?;
            }
        }
    }
    if let Some(patch) = &packet.patch {
        if patch.width == 0 {
            return Err(InventoryProgressionCodecError::ZeroPatchWidth);
        }
        writer.write_u8(patch.width)?;
        writer.write_u8(patch.height)?;
        writer.write_u8(patch.start_x)?;
        writer.write_u8(patch.start_y)?;
        writer.write_byte_array(&patch.colors, MAX_INFLATED_PACKET_LENGTH)?;
    } else {
        writer.write_u8(0)?;
    }
    Ok(())
}

pub(crate) fn read_tag_query(
    reader: &mut WireReader<'_>,
) -> Result<TagQuery, InventoryProgressionCodecError> {
    let transaction = reader.read_var_i32()?;
    let tag = NetworkNbt::read_nullable(reader, NbtQuota::Default)?;
    if let Some(tag) = &tag
        && tag.root_tag_id() != 10
    {
        return Err(InventoryProgressionCodecError::InvalidTagQueryRoot {
            tag_id: tag.root_tag_id(),
        });
    }
    Ok(TagQuery { transaction, tag })
}

pub(crate) fn write_tag_query(
    writer: &mut WireWriter,
    packet: &TagQuery,
) -> Result<(), InventoryProgressionCodecError> {
    if let Some(tag) = &packet.tag
        && tag.root_tag_id() != 10
    {
        return Err(InventoryProgressionCodecError::InvalidTagQueryRoot {
            tag_id: tag.root_tag_id(),
        });
    }
    writer.write_var_i32(packet.transaction)?;
    NetworkNbt::write_nullable(packet.tag.as_ref(), writer)?;
    Ok(())
}

pub(crate) fn read_advancements(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<UpdateAdvancements, InventoryProgressionCodecError> {
    let reset = reader.read_bool()?;
    let added_count = reader.read_count("added advancements", reader.remaining())?;
    let mut added = Vec::with_capacity(added_count);
    for _ in 0..added_count {
        added.push(AdvancementHolder {
            id: read_identifier(reader)?,
            advancement: read_advancement(reader, context)?,
        });
    }
    let removed_count = reader.read_count("removed advancements", reader.remaining())?;
    let mut removed = BTreeSet::new();
    for _ in 0..removed_count {
        removed.insert(read_identifier(reader)?);
    }
    let progress_count = reader.read_count("advancement progress", reader.remaining())?;
    let mut progress = BTreeMap::new();
    for _ in 0..progress_count {
        progress.insert(read_identifier(reader)?, read_progress(reader)?);
    }
    Ok(UpdateAdvancements {
        reset,
        added,
        removed,
        progress,
        show_advancements: reader.read_bool()?,
    })
}

pub(crate) fn write_advancements(
    writer: &mut WireWriter,
    packet: &UpdateAdvancements,
    registries: &PlayRegistries,
) -> Result<(), InventoryProgressionCodecError> {
    writer.write_bool(packet.reset)?;
    write_count(writer, "added advancements", packet.added.len())?;
    for holder in &packet.added {
        holder.id.write(writer)?;
        write_advancement(writer, &holder.advancement, registries)?;
    }
    write_count(writer, "removed advancements", packet.removed.len())?;
    for removed in &packet.removed {
        removed.write(writer)?;
    }
    write_count(writer, "advancement progress", packet.progress.len())?;
    for (id, progress) in &packet.progress {
        id.write(writer)?;
        write_progress(writer, progress)?;
    }
    writer.write_bool(packet.show_advancements)?;
    Ok(())
}

fn read_advancement(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<Advancement, InventoryProgressionCodecError> {
    let parent = read_optional_identifier(reader)?;
    let display = if reader.read_bool()? {
        Some(read_display(reader, context)?)
    } else {
        None
    };
    let outer = reader.read_count("advancement requirement groups", reader.remaining())?;
    let mut requirements = Vec::with_capacity(outer);
    for _ in 0..outer {
        let inner = reader.read_count("advancement requirements", reader.remaining())?;
        let mut group = Vec::with_capacity(inner);
        for _ in 0..inner {
            group.push(reader.read_utf(MAX_CRITERION_NAME)?.into_owned());
        }
        requirements.push(group);
    }
    Ok(Advancement {
        parent,
        display,
        requirements,
        sends_telemetry_event: reader.read_bool()?,
    })
}

fn write_advancement(
    writer: &mut WireWriter,
    advancement: &Advancement,
    registries: &PlayRegistries,
) -> Result<(), InventoryProgressionCodecError> {
    write_optional_identifier(writer, advancement.parent.as_ref())?;
    writer.write_bool(advancement.display.is_some())?;
    if let Some(display) = &advancement.display {
        write_display(writer, display, registries)?;
    }
    write_count(
        writer,
        "advancement requirement groups",
        advancement.requirements.len(),
    )?;
    for group in &advancement.requirements {
        write_count(writer, "advancement requirements", group.len())?;
        for criterion in group {
            writer.write_utf(criterion, MAX_CRITERION_NAME)?;
        }
    }
    writer.write_bool(advancement.sends_telemetry_event)?;
    Ok(())
}

fn read_display(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<DisplayInfo, InventoryProgressionCodecError> {
    let title = read_component(reader, NbtQuota::Trusted)?;
    let description = read_component(reader, NbtQuota::Trusted)?;
    let icon = read_stack_template(reader, context)?;
    let frame_id = reader.read_var_i32()?;
    let frame = AdvancementFrame::from_id(frame_id)
        .ok_or(InventoryProgressionCodecError::InvalidAdvancementFrame { frame_id })?;
    let flags = reader.read_i32()?;
    let background = if flags & 1 != 0 {
        Some(read_identifier(reader)?)
    } else {
        None
    };
    Ok(DisplayInfo {
        title,
        description,
        icon,
        frame,
        background,
        show_toast: flags & 2 != 0,
        hidden: flags & 4 != 0,
        x: reader.read_f32()?,
        y: reader.read_f32()?,
    })
}

fn write_display(
    writer: &mut WireWriter,
    display: &DisplayInfo,
    registries: &PlayRegistries,
) -> Result<(), InventoryProgressionCodecError> {
    display.title.network_nbt().write(writer)?;
    display.description.network_nbt().write(writer)?;
    write_stack_template(writer, &display.icon, registries)?;
    writer.write_var_i32(display.frame.id())?;
    let flags = i32::from(display.background.is_some())
        | (i32::from(display.show_toast) << 1)
        | (i32::from(display.hidden) << 2);
    writer.write_i32(flags)?;
    if let Some(background) = &display.background {
        background.write(writer)?;
    }
    writer.write_f32(display.x)?;
    writer.write_f32(display.y)?;
    Ok(())
}

fn read_progress(
    reader: &mut WireReader<'_>,
) -> Result<AdvancementProgress, InventoryProgressionCodecError> {
    let count = reader.read_count("advancement criteria", reader.remaining())?;
    let mut criteria = BTreeMap::new();
    for _ in 0..count {
        let name = reader.read_utf(MAX_CRITERION_NAME)?.into_owned();
        let obtained = reader.read_bool()?.then(|| reader.read_i64()).transpose()?;
        criteria.insert(name, obtained);
    }
    Ok(AdvancementProgress { criteria })
}

fn write_progress(
    writer: &mut WireWriter,
    progress: &AdvancementProgress,
) -> Result<(), InventoryProgressionCodecError> {
    write_count(writer, "advancement criteria", progress.criteria.len())?;
    for (name, obtained) in &progress.criteria {
        writer.write_utf(name, MAX_CRITERION_NAME)?;
        writer.write_bool(obtained.is_some())?;
        if let Some(timestamp) = obtained {
            writer.write_i64(*timestamp)?;
        }
    }
    Ok(())
}

fn read_component(
    reader: &mut WireReader<'_>,
    quota: NbtQuota,
) -> Result<TextComponentNbt, InventoryProgressionCodecError> {
    Ok(TextComponentNbt::from_network_nbt(NetworkNbt::read(
        reader, quota,
    )?)?)
}

fn read_optional_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Option<Identifier>, InventoryProgressionCodecError> {
    if reader.read_bool()? {
        Ok(Some(read_identifier(reader)?))
    } else {
        Ok(None)
    }
}

fn write_optional_identifier(
    writer: &mut WireWriter,
    value: Option<&Identifier>,
) -> Result<(), InventoryProgressionCodecError> {
    writer.write_bool(value.is_some())?;
    if let Some(value) = value {
        value.write(writer)?;
    }
    Ok(())
}

fn read_identifier(
    reader: &mut WireReader<'_>,
) -> Result<Identifier, InventoryProgressionCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

fn write_count(
    writer: &mut WireWriter,
    field: &'static str,
    count: usize,
) -> Result<(), InventoryProgressionCodecError> {
    writer.write_count(field, count, MAX_INFLATED_PACKET_LENGTH)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InventoryProgressionCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
    #[error(transparent)]
    Item(#[from] ItemCodecError),
    #[error("map patch width zero is reserved for the absent sentinel")]
    ZeroPatchWidth,
    #[error("tag-query NBT requires a nullable compound root, got tag {tag_id}")]
    InvalidTagQueryRoot { tag_id: u8 },
    #[error("advancement frame ordinal {frame_id} is outside 0..=2")]
    InvalidAdvancementFrame { frame_id: i32 },
}
