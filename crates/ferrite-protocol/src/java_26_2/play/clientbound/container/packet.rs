use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerClose {
    pub container_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetContent {
    pub container_id: i32,
    pub state_id: i32,
    pub slots: Vec<ItemStack>,
    pub carried: ItemStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerSetData {
    pub container_id: i32,
    pub property_id: i16,
    pub value: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetSlot {
    pub container_id: i32,
    pub state_id: i32,
    pub slot: i16,
    pub item: ItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenScreen {
    pub container_id: i32,
    pub menu_type: Identifier,
    pub title: TextComponentNbt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCursorItem {
    pub item: ItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPlayerInventory {
    pub slot: i32,
    pub item: ItemStack,
}
