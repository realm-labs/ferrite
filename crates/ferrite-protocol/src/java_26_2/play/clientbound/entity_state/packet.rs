use crate::java_26_2::play::clientbound::entity_state::metadata::MetadataEntry;
use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct SetEntityData {
    pub entity_id: i32,
    pub values: Vec<MetadataEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetEntityLink {
    pub source_entity_id: i32,
    pub destination_entity_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetEquipment {
    pub entity_id: i32,
    pub entries: Vec<EquipmentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentEntry {
    pub slot: EquipmentSlot,
    pub stack: ItemStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EquipmentSlot {
    Mainhand = 0,
    Offhand = 1,
    Feet = 2,
    Legs = 3,
    Chest = 4,
    Head = 5,
    Body = 6,
    Saddle = 7,
}

impl EquipmentSlot {
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn from_ordinal(ordinal: u8) -> Option<Self> {
        Some(match ordinal {
            0 => Self::Mainhand,
            1 => Self::Offhand,
            2 => Self::Feet,
            3 => Self::Legs,
            4 => Self::Chest,
            5 => Self::Head,
            6 => Self::Body,
            7 => Self::Saddle,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPassengers {
    pub vehicle_id: i32,
    pub passenger_ids: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateAttributes {
    pub entity_id: i32,
    pub snapshots: Vec<AttributeSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSnapshot {
    pub attribute: Identifier,
    pub base: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifier {
    pub identity: Identifier,
    pub amount: f64,
    pub operation: AttributeOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOperation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

impl AttributeOperation {
    #[must_use]
    pub const fn from_raw_id(raw_id: i32) -> Self {
        match raw_id {
            1 => Self::AddMultipliedBase,
            2 => Self::AddMultipliedTotal,
            _ => Self::AddValue,
        }
    }

    #[must_use]
    pub const fn raw_id(self) -> i32 {
        match self {
            Self::AddValue => 0,
            Self::AddMultipliedBase => 1,
            Self::AddMultipliedTotal => 2,
        }
    }
}
