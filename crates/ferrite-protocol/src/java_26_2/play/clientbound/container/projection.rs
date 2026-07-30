//! Bounded model of the locked client's ordinary-menu handlers.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::play::clientbound::container::packet::{
    ContainerSetContent, ContainerSetData, ContainerSetSlot, OpenScreen, SetPlayerInventory,
};
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuDefinition {
    pub slots: usize,
    pub data_slots: usize,
    pub has_screen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedMenu {
    pub container_id: i32,
    pub menu_type: Option<Identifier>,
    pub title: Option<TextComponentNbt>,
    pub slots: Vec<ItemStack>,
    pub data: Vec<i16>,
    pub carried: ItemStack,
    pub state_id: i32,
    pub pop_times: Vec<u8>,
}

impl ProjectedMenu {
    fn inventory(slots: usize) -> Self {
        Self {
            container_id: 0,
            menu_type: None,
            title: None,
            slots: vec![ItemStack::Empty; slots],
            data: Vec::new(),
            carried: ItemStack::Empty,
            state_id: 0,
            pop_times: vec![0; slots],
        }
    }

    fn opened(packet: &OpenScreen, definition: MenuDefinition) -> Self {
        Self {
            container_id: packet.container_id,
            menu_type: Some(packet.menu_type.clone()),
            title: Some(packet.title.clone()),
            slots: vec![ItemStack::Empty; definition.slots],
            data: vec![0; definition.data_slots],
            carried: ItemStack::Empty,
            state_id: 0,
            pop_times: vec![0; definition.slots],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInventoryProjection {
    pub ordinary: Vec<ItemStack>,
    pub armor: Vec<ItemStack>,
    pub offhand: ItemStack,
    pub body: ItemStack,
    pub saddle: ItemStack,
}

impl Default for PlayerInventoryProjection {
    fn default() -> Self {
        Self {
            ordinary: vec![ItemStack::Empty; 36],
            armor: vec![ItemStack::Empty; 4],
            offhand: ItemStack::Empty,
            body: ItemStack::Empty,
            saddle: ItemStack::Empty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerClientProjection {
    definitions: BTreeMap<Identifier, MenuDefinition>,
    inventory_menu: ProjectedMenu,
    current_menu: Option<ProjectedMenu>,
    player_inventory: PlayerInventoryProjection,
    creative_screen_visible: bool,
    tutorial_observations: usize,
    last_tutorial_item: Option<ItemStack>,
    local_broadcasts: usize,
}

impl ContainerClientProjection {
    pub fn new(
        inventory_menu_slots: usize,
        definitions: BTreeMap<Identifier, MenuDefinition>,
        maximum_slots: usize,
    ) -> Result<Self, ContainerProjectionError> {
        if inventory_menu_slots > maximum_slots {
            return Err(ContainerProjectionError::SlotCapacity {
                requested: inventory_menu_slots,
                maximum: maximum_slots,
            });
        }
        for definition in definitions.values() {
            if definition.slots > maximum_slots {
                return Err(ContainerProjectionError::SlotCapacity {
                    requested: definition.slots,
                    maximum: maximum_slots,
                });
            }
            if definition.data_slots > maximum_slots {
                return Err(ContainerProjectionError::DataCapacity {
                    requested: definition.data_slots,
                    maximum: maximum_slots,
                });
            }
        }
        Ok(Self {
            definitions,
            inventory_menu: ProjectedMenu::inventory(inventory_menu_slots),
            current_menu: None,
            player_inventory: PlayerInventoryProjection::default(),
            creative_screen_visible: false,
            tutorial_observations: 0,
            last_tutorial_item: None,
            local_broadcasts: 0,
        })
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
    ) -> Result<ContainerProjectionAction, ContainerProjectionError> {
        match packet {
            PlayClientboundPacket::ContainerClose(_) => {
                self.current_menu = None;
                Ok(ContainerProjectionAction::ScreenClosed)
            }
            PlayClientboundPacket::ContainerSetContent(packet) => {
                self.apply_content(packet)?;
                Ok(ContainerProjectionAction::None)
            }
            PlayClientboundPacket::ContainerSetData(packet) => {
                self.apply_data(*packet)?;
                Ok(ContainerProjectionAction::None)
            }
            PlayClientboundPacket::ContainerSetSlot(packet) => {
                self.observe_tutorial(&packet.item);
                self.apply_slot(packet)?;
                Ok(ContainerProjectionAction::None)
            }
            PlayClientboundPacket::OpenScreen(packet) => self.apply_open(packet),
            PlayClientboundPacket::SetCursorItem(packet) => {
                self.observe_tutorial(&packet.item);
                if !self.creative_screen_visible {
                    self.active_menu_mut().carried = packet.item.clone();
                }
                Ok(ContainerProjectionAction::None)
            }
            PlayClientboundPacket::SetPlayerInventory(packet) => {
                self.observe_tutorial(&packet.item);
                self.apply_player_inventory(packet)?;
                Ok(ContainerProjectionAction::None)
            }
            _ => Err(ContainerProjectionError::WrongPacketFamily),
        }
    }

    pub const fn set_creative_screen_visible(&mut self, visible: bool) {
        self.creative_screen_visible = visible;
    }

    #[must_use]
    pub fn active_menu(&self) -> &ProjectedMenu {
        self.current_menu.as_ref().unwrap_or(&self.inventory_menu)
    }

    #[must_use]
    pub const fn inventory_menu(&self) -> &ProjectedMenu {
        &self.inventory_menu
    }

    #[must_use]
    pub const fn player_inventory(&self) -> &PlayerInventoryProjection {
        &self.player_inventory
    }

    #[must_use]
    pub const fn tutorial_observations(&self) -> usize {
        self.tutorial_observations
    }

    #[must_use]
    pub const fn last_tutorial_item(&self) -> Option<&ItemStack> {
        self.last_tutorial_item.as_ref()
    }

    #[must_use]
    pub const fn local_broadcasts(&self) -> usize {
        self.local_broadcasts
    }

    fn apply_open(
        &mut self,
        packet: &OpenScreen,
    ) -> Result<ContainerProjectionAction, ContainerProjectionError> {
        let Some(definition) = self.definitions.get(&packet.menu_type).copied() else {
            return Err(ContainerProjectionError::MissingMenuDefinition {
                menu_type: packet.menu_type.clone(),
            });
        };
        if !definition.has_screen {
            return Ok(ContainerProjectionAction::MissingScreen {
                menu_type: packet.menu_type.clone(),
            });
        }
        self.current_menu = Some(ProjectedMenu::opened(packet, definition));
        Ok(ContainerProjectionAction::ScreenOpened {
            container_id: packet.container_id,
        })
    }

    fn apply_content(
        &mut self,
        packet: &ContainerSetContent,
    ) -> Result<(), ContainerProjectionError> {
        let Some(menu) = self.content_target_mut(packet.container_id) else {
            return Ok(());
        };
        for (index, item) in packet.slots.iter().enumerate() {
            let slots = menu.slots.len();
            let Some(slot) = menu.slots.get_mut(index) else {
                return Err(ContainerProjectionError::InvalidSlot {
                    container_id: packet.container_id,
                    slot: index as i64,
                    slots,
                });
            };
            *slot = item.clone();
        }
        menu.carried = packet.carried.clone();
        menu.state_id = packet.state_id;
        Ok(())
    }

    fn apply_data(&mut self, packet: ContainerSetData) -> Result<(), ContainerProjectionError> {
        let Some(menu) = self.exact_current_mut(packet.container_id) else {
            return Ok(());
        };
        let properties = menu.data.len();
        let index = usize::try_from(packet.property_id).map_err(|_| {
            ContainerProjectionError::InvalidDataSlot {
                container_id: packet.container_id,
                property: packet.property_id,
                properties,
            }
        })?;
        let Some(value) = menu.data.get_mut(index) else {
            return Err(ContainerProjectionError::InvalidDataSlot {
                container_id: packet.container_id,
                property: packet.property_id,
                properties,
            });
        };
        *value = packet.value;
        Ok(())
    }

    fn apply_slot(&mut self, packet: &ContainerSetSlot) -> Result<(), ContainerProjectionError> {
        if packet.container_id == 0 {
            set_menu_slot(&mut self.inventory_menu, packet)?;
        } else if self
            .current_menu
            .as_ref()
            .is_some_and(|menu| menu.container_id == packet.container_id)
        {
            set_menu_slot(
                self.current_menu
                    .as_mut()
                    .expect("matching current menu remains present"),
                packet,
            )?;
        }
        if self.creative_screen_visible {
            set_menu_slot(&mut self.inventory_menu, packet)?;
            self.local_broadcasts = self.local_broadcasts.saturating_add(1);
        }
        Ok(())
    }

    fn apply_player_inventory(
        &mut self,
        packet: &SetPlayerInventory,
    ) -> Result<(), ContainerProjectionError> {
        let slot = usize::try_from(packet.slot).map_err(|_| {
            ContainerProjectionError::InvalidPlayerInventorySlot { slot: packet.slot }
        })?;
        match slot {
            0..=35 => self.player_inventory.ordinary[slot] = packet.item.clone(),
            36..=39 => self.player_inventory.armor[slot - 36] = packet.item.clone(),
            40 => self.player_inventory.offhand = packet.item.clone(),
            41 => self.player_inventory.body = packet.item.clone(),
            42 => self.player_inventory.saddle = packet.item.clone(),
            _ => {}
        }
        Ok(())
    }

    fn observe_tutorial(&mut self, item: &ItemStack) {
        self.tutorial_observations = self.tutorial_observations.saturating_add(1);
        self.last_tutorial_item = Some(item.clone());
    }

    fn active_menu_mut(&mut self) -> &mut ProjectedMenu {
        self.current_menu
            .as_mut()
            .unwrap_or(&mut self.inventory_menu)
    }

    fn content_target_mut(&mut self, container_id: i32) -> Option<&mut ProjectedMenu> {
        if container_id == 0 {
            Some(&mut self.inventory_menu)
        } else {
            self.current_menu
                .as_mut()
                .filter(|menu| menu.container_id == container_id)
        }
    }

    fn exact_current_mut(&mut self, container_id: i32) -> Option<&mut ProjectedMenu> {
        if let Some(menu) = self.current_menu.as_mut() {
            (menu.container_id == container_id).then_some(menu)
        } else {
            (container_id == 0).then_some(&mut self.inventory_menu)
        }
    }
}

fn set_menu_slot(
    menu: &mut ProjectedMenu,
    packet: &ContainerSetSlot,
) -> Result<(), ContainerProjectionError> {
    let index =
        usize::try_from(packet.slot).map_err(|_| ContainerProjectionError::InvalidSlot {
            container_id: packet.container_id,
            slot: i64::from(packet.slot),
            slots: menu.slots.len(),
        })?;
    let slots = menu.slots.len();
    let old_count = menu.slots.get(index).map(ItemStack::count).ok_or(
        ContainerProjectionError::InvalidSlot {
            container_id: packet.container_id,
            slot: i64::from(packet.slot),
            slots,
        },
    )?;
    if menu.container_id == 0
        && (36..=44).contains(&index)
        && !packet.item.is_empty()
        && old_count < packet.item.count()
    {
        menu.pop_times[index] = 5;
    }
    menu.slots[index] = packet.item.clone();
    menu.state_id = packet.state_id;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerProjectionAction {
    None,
    ScreenOpened { container_id: i32 },
    ScreenClosed,
    MissingScreen { menu_type: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContainerProjectionError {
    #[error("packet is outside the clientbound container family")]
    WrongPacketFamily,
    #[error("menu definition is absent for {menu_type}")]
    MissingMenuDefinition { menu_type: Identifier },
    #[error("menu requests {requested} slots, above projection bound {maximum}")]
    SlotCapacity { requested: usize, maximum: usize },
    #[error("menu requests {requested} data slots, above projection bound {maximum}")]
    DataCapacity { requested: usize, maximum: usize },
    #[error("container {container_id} slot {slot} is outside its {slots} slots")]
    InvalidSlot {
        container_id: i32,
        slot: i64,
        slots: usize,
    },
    #[error("container {container_id} property {property} is outside its {properties} properties")]
    InvalidDataSlot {
        container_id: i32,
        property: i16,
        properties: usize,
    },
    #[error("player inventory slot {slot} follows the locked negative-index fault path")]
    InvalidPlayerInventorySlot { slot: i32 },
}
