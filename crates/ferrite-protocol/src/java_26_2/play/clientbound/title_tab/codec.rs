use thiserror::Error;

use crate::java_26_2::play::clientbound::title_tab::packet::{
    ClearTitles, SelectAdvancementsTab, SetActionBarText, SetSubtitleText, SetTitleText,
    SetTitlesAnimation, TabList,
};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, PartialEq, Error)]
pub enum TitleTabCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error(transparent)]
    Nbt(#[from] NbtError),
}

pub(crate) fn read_clear(reader: &mut WireReader<'_>) -> Result<ClearTitles, TitleTabCodecError> {
    Ok(ClearTitles {
        reset_times: reader.read_bool()?,
    })
}

pub(crate) fn write_clear(
    writer: &mut WireWriter,
    packet: ClearTitles,
) -> Result<(), TitleTabCodecError> {
    writer.write_bool(packet.reset_times)?;
    Ok(())
}

pub(crate) fn read_select(
    reader: &mut WireReader<'_>,
) -> Result<SelectAdvancementsTab, TitleTabCodecError> {
    Ok(SelectAdvancementsTab {
        tab: reader
            .read_bool()?
            .then(|| read_identifier(reader))
            .transpose()?,
    })
}

pub(crate) fn write_select(
    writer: &mut WireWriter,
    packet: &SelectAdvancementsTab,
) -> Result<(), TitleTabCodecError> {
    writer.write_bool(packet.tab.is_some())?;
    if let Some(tab) = &packet.tab {
        tab.write(writer)?;
    }
    Ok(())
}

pub(crate) fn read_action_bar(
    reader: &mut WireReader<'_>,
) -> Result<SetActionBarText, TitleTabCodecError> {
    Ok(SetActionBarText {
        text: read_component(reader)?,
    })
}

pub(crate) fn write_action_bar(
    writer: &mut WireWriter,
    packet: &SetActionBarText,
) -> Result<(), TitleTabCodecError> {
    write_component(writer, &packet.text)
}

pub(crate) fn read_subtitle(
    reader: &mut WireReader<'_>,
) -> Result<SetSubtitleText, TitleTabCodecError> {
    Ok(SetSubtitleText {
        text: read_component(reader)?,
    })
}

pub(crate) fn write_subtitle(
    writer: &mut WireWriter,
    packet: &SetSubtitleText,
) -> Result<(), TitleTabCodecError> {
    write_component(writer, &packet.text)
}

pub(crate) fn read_title(reader: &mut WireReader<'_>) -> Result<SetTitleText, TitleTabCodecError> {
    Ok(SetTitleText {
        text: read_component(reader)?,
    })
}

pub(crate) fn write_title(
    writer: &mut WireWriter,
    packet: &SetTitleText,
) -> Result<(), TitleTabCodecError> {
    write_component(writer, &packet.text)
}

pub(crate) fn read_animation(
    reader: &mut WireReader<'_>,
) -> Result<SetTitlesAnimation, TitleTabCodecError> {
    Ok(SetTitlesAnimation {
        fade_in: reader.read_i32()?,
        stay: reader.read_i32()?,
        fade_out: reader.read_i32()?,
    })
}

pub(crate) fn write_animation(
    writer: &mut WireWriter,
    packet: SetTitlesAnimation,
) -> Result<(), TitleTabCodecError> {
    writer.write_i32(packet.fade_in)?;
    writer.write_i32(packet.stay)?;
    writer.write_i32(packet.fade_out)?;
    Ok(())
}

pub(crate) fn read_tab_list(reader: &mut WireReader<'_>) -> Result<TabList, TitleTabCodecError> {
    Ok(TabList {
        header: read_component(reader)?,
        footer: read_component(reader)?,
    })
}

pub(crate) fn write_tab_list(
    writer: &mut WireWriter,
    packet: &TabList,
) -> Result<(), TitleTabCodecError> {
    write_component(writer, &packet.header)?;
    write_component(writer, &packet.footer)
}

fn read_component(reader: &mut WireReader<'_>) -> Result<TextComponentNbt, TitleTabCodecError> {
    Ok(TextComponentNbt::from_network_nbt(NetworkNbt::read(
        reader,
        NbtQuota::Trusted,
    )?)?)
}

fn write_component(
    writer: &mut WireWriter,
    component: &TextComponentNbt,
) -> Result<(), TitleTabCodecError> {
    writer.write_bytes(component.network_nbt().as_bytes())?;
    Ok(())
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, TitleTabCodecError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}
