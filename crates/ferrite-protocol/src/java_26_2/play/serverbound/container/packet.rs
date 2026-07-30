use std::collections::{BTreeMap, BTreeSet};

use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerButtonClick {
    pub container_id: i32,
    pub button_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerClick {
    pub container_id: i32,
    pub state_id: i32,
    pub slot: i16,
    pub button: i8,
    pub input: ContainerInput,
    pub changed_slots: BTreeMap<i16, HashedStack>,
    pub carried: HashedStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerClose {
    pub container_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerSlotStateChanged {
    pub slot_id: i32,
    pub container_id: i32,
    pub new_state: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetCarriedItem {
    pub slot: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerInput {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

impl ContainerInput {
    #[must_use]
    pub const fn from_wire(value: i32) -> Self {
        match value {
            1 => Self::QuickMove,
            2 => Self::Swap,
            3 => Self::Clone,
            4 => Self::Throw,
            5 => Self::QuickCraft,
            6 => Self::PickupAll,
            _ => Self::Pickup,
        }
    }

    #[must_use]
    pub const fn to_wire(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum HashedStack {
    #[default]
    Empty,
    Present(HashedStackContents),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedStackContents {
    pub item: Identifier,
    pub count: i32,
    pub components: HashedComponentPatch,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HashedComponentPatch {
    pub added: BTreeMap<Identifier, i32>,
    pub removed: BTreeSet<Identifier>,
}
