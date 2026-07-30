use std::collections::{BTreeMap, BTreeSet};

use crate::java_26_2::play::item::ItemStackTemplate;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::{NetworkNbt, TextComponentNbt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapItemData {
    pub map_id: i32,
    pub scale: i8,
    pub locked: bool,
    pub decorations: Option<Vec<MapDecoration>>,
    pub patch: Option<MapPatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDecoration {
    pub decoration_type: Identifier,
    pub x: i8,
    pub y: i8,
    pub rotation: u8,
    pub name: Option<TextComponentNbt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapPatch {
    pub width: u8,
    pub height: u8,
    pub start_x: u8,
    pub start_y: u8,
    pub colors: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagQuery {
    pub transaction: i32,
    pub tag: Option<NetworkNbt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateAdvancements {
    pub reset: bool,
    pub added: Vec<AdvancementHolder>,
    pub removed: BTreeSet<Identifier>,
    pub progress: BTreeMap<Identifier, AdvancementProgress>,
    pub show_advancements: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancementHolder {
    pub id: Identifier,
    pub advancement: Advancement,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Advancement {
    pub parent: Option<Identifier>,
    pub display: Option<DisplayInfo>,
    pub requirements: Vec<Vec<String>>,
    pub sends_telemetry_event: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfo {
    pub title: TextComponentNbt,
    pub description: TextComponentNbt,
    pub icon: ItemStackTemplate,
    pub frame: AdvancementFrame,
    pub background: Option<Identifier>,
    pub show_toast: bool,
    pub hidden: bool,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementFrame {
    Task,
    Challenge,
    Goal,
}

impl AdvancementFrame {
    pub(crate) fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Task),
            1 => Some(Self::Challenge),
            2 => Some(Self::Goal),
            _ => None,
        }
    }

    pub(crate) const fn id(self) -> i32 {
        match self {
            Self::Task => 0,
            Self::Challenge => 1,
            Self::Goal => 2,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementProgress {
    pub criteria: BTreeMap<String, Option<i64>>,
}
