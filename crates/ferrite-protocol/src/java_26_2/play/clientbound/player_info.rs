use thiserror::Error;

use crate::java_26_2::login::profile::ProfileProperty;
use crate::java_26_2::play::clientbound::packet::GameMode;
use crate::java_26_2::value::nbt::{NbtError, NbtQuota, NetworkNbt, TextComponentNbt};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_PROFILE_NAME_CODE_UNITS: usize = 16;
const MAX_PROFILE_PROPERTIES: usize = 16;
const MAX_PROPERTY_NAME_CODE_UNITS: usize = 64;
const MAX_PROPERTY_VALUE_CODE_UNITS: usize = 32_767;
const MAX_PROPERTY_SIGNATURE_CODE_UNITS: usize = 1_024;
const MAX_PUBLIC_KEY_BYTES: usize = 512;
const MAX_KEY_SIGNATURE_BYTES: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInfoUpdate {
    pub actions: PlayerInfoActions,
    pub entries: Vec<PlayerInfoEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerInfoActions(u8);

impl PlayerInfoActions {
    pub const ADD_PLAYER: u8 = 1 << 0;
    pub const INITIALIZE_CHAT: u8 = 1 << 1;
    pub const UPDATE_GAME_MODE: u8 = 1 << 2;
    pub const UPDATE_LISTED: u8 = 1 << 3;
    pub const UPDATE_LATENCY: u8 = 1 << 4;
    pub const UPDATE_DISPLAY_NAME: u8 = 1 << 5;
    pub const UPDATE_LIST_ORDER: u8 = 1 << 6;
    pub const UPDATE_HAT: u8 = 1 << 7;

    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn all() -> Self {
        Self(u8::MAX)
    }

    #[must_use]
    pub const fn contains(self, action: u8) -> bool {
        self.0 & action != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInfoEntry {
    pub profile_id: u128,
    pub added_profile: Option<AddedProfile>,
    pub chat_session: Option<Option<ChatSession>>,
    pub game_mode: Option<GameMode>,
    pub listed: Option<bool>,
    pub latency_millis: Option<i32>,
    pub display_name: Option<Option<TextComponentNbt>>,
    pub list_order: Option<i32>,
    pub show_hat: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddedProfile {
    pub name: String,
    pub properties: Vec<ProfileProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSession {
    pub session_id: u128,
    pub expires_at_millis: i64,
    pub public_key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

pub(crate) fn read(reader: &mut WireReader<'_>) -> Result<PlayerInfoUpdate, PlayerInfoError> {
    let actions = PlayerInfoActions::from_bits(reader.read_u8()?);
    let count = reader.read_count("player info entries", reader.remaining())?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(read_entry(reader, actions)?);
    }
    Ok(PlayerInfoUpdate { actions, entries })
}

pub(crate) fn write(
    writer: &mut WireWriter,
    update: &PlayerInfoUpdate,
) -> Result<(), PlayerInfoError> {
    writer.write_u8(update.actions.bits())?;
    writer.write_count(
        "player info entries",
        update.entries.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for entry in &update.entries {
        write_entry(writer, update.actions, entry)?;
    }
    Ok(())
}

fn read_entry(
    reader: &mut WireReader<'_>,
    actions: PlayerInfoActions,
) -> Result<PlayerInfoEntry, PlayerInfoError> {
    let profile_id = reader.read_u128()?;
    let added_profile = if actions.contains(PlayerInfoActions::ADD_PLAYER) {
        let name = reader.read_utf(MAX_PROFILE_NAME_CODE_UNITS)?.into_owned();
        let count = reader.read_count("profile properties", MAX_PROFILE_PROPERTIES)?;
        let mut properties = Vec::with_capacity(count);
        for _ in 0..count {
            properties.push(read_property(reader)?);
        }
        Some(AddedProfile { name, properties })
    } else {
        None
    };
    let chat_session = if actions.contains(PlayerInfoActions::INITIALIZE_CHAT) {
        Some(if reader.read_bool()? {
            Some(ChatSession {
                session_id: reader.read_u128()?,
                expires_at_millis: reader.read_i64()?,
                public_key: reader.read_byte_array(MAX_PUBLIC_KEY_BYTES)?.to_vec(),
                key_signature: reader.read_byte_array(MAX_KEY_SIGNATURE_BYTES)?.to_vec(),
            })
        } else {
            None
        })
    } else {
        None
    };
    let game_mode = actions
        .contains(PlayerInfoActions::UPDATE_GAME_MODE)
        .then(|| reader.read_var_i32().map(GameMode::from_i32_or_survival))
        .transpose()?;
    let listed = actions
        .contains(PlayerInfoActions::UPDATE_LISTED)
        .then(|| reader.read_bool())
        .transpose()?;
    let latency_millis = actions
        .contains(PlayerInfoActions::UPDATE_LATENCY)
        .then(|| reader.read_var_i32())
        .transpose()?;
    let display_name = if actions.contains(PlayerInfoActions::UPDATE_DISPLAY_NAME) {
        Some(if reader.read_bool()? {
            let nbt = NetworkNbt::read(reader, NbtQuota::Trusted)?;
            Some(TextComponentNbt::from_network_nbt(nbt)?)
        } else {
            None
        })
    } else {
        None
    };
    let list_order = actions
        .contains(PlayerInfoActions::UPDATE_LIST_ORDER)
        .then(|| reader.read_var_i32())
        .transpose()?;
    let show_hat = actions
        .contains(PlayerInfoActions::UPDATE_HAT)
        .then(|| reader.read_bool())
        .transpose()?;
    Ok(PlayerInfoEntry {
        profile_id,
        added_profile,
        chat_session,
        game_mode,
        listed,
        latency_millis,
        display_name,
        list_order,
        show_hat,
    })
}

fn write_entry(
    writer: &mut WireWriter,
    actions: PlayerInfoActions,
    entry: &PlayerInfoEntry,
) -> Result<(), PlayerInfoError> {
    writer.write_u128(entry.profile_id)?;
    if actions.contains(PlayerInfoActions::ADD_PLAYER) {
        let profile = required(&entry.added_profile, "add player")?;
        writer.write_utf(&profile.name, MAX_PROFILE_NAME_CODE_UNITS)?;
        writer.write_count(
            "profile properties",
            profile.properties.len(),
            MAX_PROFILE_PROPERTIES,
        )?;
        for property in &profile.properties {
            write_property(writer, property)?;
        }
    }
    if actions.contains(PlayerInfoActions::INITIALIZE_CHAT) {
        let chat = required(&entry.chat_session, "initialize chat")?;
        writer.write_bool(chat.is_some())?;
        if let Some(chat) = chat {
            writer.write_u128(chat.session_id)?;
            writer.write_i64(chat.expires_at_millis)?;
            writer.write_byte_array(&chat.public_key, MAX_PUBLIC_KEY_BYTES)?;
            writer.write_byte_array(&chat.key_signature, MAX_KEY_SIGNATURE_BYTES)?;
        }
    }
    if actions.contains(PlayerInfoActions::UPDATE_GAME_MODE) {
        writer.write_var_i32(required(&entry.game_mode, "game mode")?.id())?;
    }
    if actions.contains(PlayerInfoActions::UPDATE_LISTED) {
        writer.write_bool(*required(&entry.listed, "listed")?)?;
    }
    if actions.contains(PlayerInfoActions::UPDATE_LATENCY) {
        writer.write_var_i32(*required(&entry.latency_millis, "latency")?)?;
    }
    if actions.contains(PlayerInfoActions::UPDATE_DISPLAY_NAME) {
        let display_name = required(&entry.display_name, "display name")?;
        writer.write_bool(display_name.is_some())?;
        if let Some(display_name) = display_name {
            display_name.network_nbt().write(writer)?;
        }
    }
    if actions.contains(PlayerInfoActions::UPDATE_LIST_ORDER) {
        writer.write_var_i32(*required(&entry.list_order, "list order")?)?;
    }
    if actions.contains(PlayerInfoActions::UPDATE_HAT) {
        writer.write_bool(*required(&entry.show_hat, "show hat")?)?;
    }
    Ok(())
}

fn read_property(reader: &mut WireReader<'_>) -> Result<ProfileProperty, PlayerInfoError> {
    Ok(ProfileProperty {
        name: reader.read_utf(MAX_PROPERTY_NAME_CODE_UNITS)?.into_owned(),
        value: reader.read_utf(MAX_PROPERTY_VALUE_CODE_UNITS)?.into_owned(),
        signature: if reader.read_bool()? {
            Some(
                reader
                    .read_utf(MAX_PROPERTY_SIGNATURE_CODE_UNITS)?
                    .into_owned(),
            )
        } else {
            None
        },
    })
}

fn write_property(
    writer: &mut WireWriter,
    property: &ProfileProperty,
) -> Result<(), PlayerInfoError> {
    writer.write_utf(&property.name, MAX_PROPERTY_NAME_CODE_UNITS)?;
    writer.write_utf(&property.value, MAX_PROPERTY_VALUE_CODE_UNITS)?;
    writer.write_bool(property.signature.is_some())?;
    if let Some(signature) = &property.signature {
        writer.write_utf(signature, MAX_PROPERTY_SIGNATURE_CODE_UNITS)?;
    }
    Ok(())
}

fn required<'a, T>(value: &'a Option<T>, action: &'static str) -> Result<&'a T, PlayerInfoError> {
    value
        .as_ref()
        .ok_or(PlayerInfoError::MissingActionField { action })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlayerInfoError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidNbt(#[from] NbtError),
    #[error("player-info entry is missing the {action} action field")]
    MissingActionField { action: &'static str },
}
